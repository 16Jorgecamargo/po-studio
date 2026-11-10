use crate::lp::{self, LpModel};
use crate::simplex::{Method, Rel, Sense, Status};

fn model(sense: Sense, c: Vec<f64>, a: Vec<Vec<f64>>, rel: Vec<Rel>, b: Vec<f64>) -> LpModel {
    let n = c.len();
    let m = a.len();
    LpModel {
        sense,
        var_names: (1..=n).map(|i| format!("x{}", i)).collect(),
        c,
        a,
        rel,
        b,
        con_names: (1..=m).map(|i| format!("c{}", i)).collect(),
        lower: vec![0.0; n],
        upper: vec![f64::INFINITY; n],
        integer: vec![true; n],
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn lp_max_all_slack() {
    let m = model(Sense::Max, vec![4.0, 3.0], vec![vec![2.0, 1.0], vec![1.0, 3.0], vec![1.0, 0.0]], vec![Rel::Le; 3], vec![10.0, 15.0, 4.0]);
    let s = lp::solve(&m, None, 100.0).unwrap();
    assert_eq!(s.report.status, Status::Optimal);
    assert!(close(s.report.z, 24.0));
    assert!(close(s.report.vars[0].value, 3.0));
    assert!(close(s.report.vars[1].value, 4.0));
    assert!(close(s.report.cons[0].dual, 1.8));
    assert!(close(s.report.cons[1].dual, 0.4));
    assert!(close(s.report.cons[2].dual, 0.0));
}

#[test]
fn lp_min_methods_agree() {
    let m = model(Sense::Min, vec![2.0, 3.0], vec![vec![1.0, 1.0], vec![1.0, 3.0]], vec![Rel::Ge; 2], vec![4.0, 6.0]);
    for method in [Method::TwoPhase, Method::MMethod, Method::DualSimplex] {
        let s = lp::solve(&m, Some(method), 100.0).unwrap();
        assert_eq!(s.report.status, Status::Optimal, "{:?}", method);
        assert!(close(s.report.z, 9.0), "{:?} {}", method, s.report.z);
        assert!(close(s.report.vars[0].value, 3.0));
        assert!(close(s.report.cons[0].dual, 1.5));
        assert!(close(s.report.cons[1].dual, 0.5));
    }
}

#[test]
fn lp_infeasible_and_unbounded() {
    let inf = model(Sense::Max, vec![1.0, 1.0], vec![vec![1.0, 1.0], vec![1.0, 1.0]], vec![Rel::Le, Rel::Ge], vec![2.0, 5.0]);
    assert_eq!(lp::solve(&inf, None, 100.0).unwrap().report.status, Status::Infeasible);
    let unb = model(Sense::Max, vec![1.0, 1.0], vec![vec![1.0, -1.0]], vec![Rel::Le], vec![2.0]);
    assert_eq!(lp::solve(&unb, None, 100.0).unwrap().report.status, Status::Unbounded);
}

#[test]
fn lp_bounds_and_free_vars() {
    let mut m = model(Sense::Min, vec![1.0, 1.0], vec![vec![1.0, 1.0]], vec![Rel::Ge], vec![-3.0]);
    m.lower = vec![f64::NEG_INFINITY, 1.0];
    m.upper = vec![f64::INFINITY, 2.0];
    let s = lp::solve(&m, None, 100.0).unwrap();
    assert_eq!(s.report.status, Status::Optimal);
    assert!(close(s.report.z, -3.0));
}

#[test]
fn ip_branch_and_bound() {
    let m = model(Sense::Max, vec![7.0, 5.0], vec![vec![3.0, 2.0], vec![1.0, 3.0], vec![0.0, 1.0]], vec![Rel::Le; 3], vec![13.5, 12.0, 3.5]);
    let r = crate::ip::branch_and_bound(&m);
    let best = &r.nodes[r.best.unwrap() - 1];
    assert!(close(best.z.unwrap(), 31.0));
    assert!(close(best.x[0], 3.0) && close(best.x[1], 2.0));
}

#[test]
fn transport_methods_agree() {
    let p = crate::transport::TransportProblem {
        sources: vec!["a".into(), "b".into(), "c".into()],
        dests: vec!["1".into(), "2".into(), "3".into(), "4".into()],
        cost: vec![vec![8.0, 6.0, 10.0, 9.0], vec![9.0, 12.0, 13.0, 7.0], vec![14.0, 9.0, 16.0, 5.0]],
        supply: vec![35.0, 50.0, 40.0],
        demand: vec![45.0, 20.0, 30.0, 30.0],
    };
    let mut costs = vec![];
    for m in [crate::transport::StartMethod::NorthWest, crate::transport::StartMethod::LeastCost, crate::transport::StartMethod::Vogel] {
        let s = crate::transport::solve(&p, m).unwrap();
        assert!(s.optimal);
        costs.push(s.tableaus.last().unwrap().cost);
    }
    let lp_m = {
        let mut a = vec![];
        let mut rel = vec![];
        let mut b = vec![];
        for i in 0..3 {
            let mut r = vec![0.0; 12];
            for j in 0..4 {
                r[i * 4 + j] = 1.0;
            }
            a.push(r);
            rel.push(Rel::Le);
            b.push(p.supply[i]);
        }
        for j in 0..4 {
            let mut r = vec![0.0; 12];
            for i in 0..3 {
                r[i * 4 + j] = 1.0;
            }
            a.push(r);
            rel.push(Rel::Ge);
            b.push(p.demand[j]);
        }
        let c: Vec<f64> = p.cost.iter().flatten().cloned().collect();
        model(Sense::Min, c, a, rel, b)
    };
    let z = lp::solve(&lp_m, None, 100.0).unwrap().report.z;
    for cst in costs {
        assert!(close(cst, z), "{} vs {}", cst, z);
    }
}

#[test]
fn queue_mm1_and_mm2() {
    let q = crate::queue::QueueInput { name: "1".into(), lambda: 6.0, mu: 8.0, servers: 1, system_limit: f64::INFINITY, source_limit: f64::INFINITY };
    let r = crate::queue::analyse(&q).unwrap();
    assert!((r.ls - 3.0).abs() < 1e-4);
    assert!((r.lq - 2.25).abs() < 1e-4);
    assert!((r.ws - 0.5).abs() < 1e-5);
    let q2 = crate::queue::QueueInput { servers: 2, ..q.clone() };
    let r2 = crate::queue::analyse(&q2).unwrap();
    let rho: f64 = 0.75;
    let p0 = 1.0 / (1.0 + rho + rho * rho / (2.0 * (1.0 - rho / 2.0)));
    let lq = rho.powi(3) / (2.0 - rho).powi(2) * p0;
    assert!((r2.lq - lq).abs() < 1e-5, "{} {}", r2.lq, lq);
    let q3 = crate::queue::QueueInput { system_limit: 5.0, ..q };
    let r3 = crate::queue::analyse(&q3).unwrap();
    assert!((r3.probs.iter().sum::<f64>() - 1.0).abs() < 1e-9);
}

#[test]
fn game_value_is_consistent() {
    let a = vec![vec![3.0, -1.0, 2.0], vec![-2.0, 4.0, 1.0]];
    let g = crate::game::solve(&a).unwrap();
    for j in 0..3 {
        let e: f64 = (0..2).map(|i| g.a_mix[i] * a[i][j]).sum();
        assert!(e >= g.value - 1e-7);
    }
    for i in 0..2 {
        let e: f64 = (0..3).map(|j| g.b_mix[j] * a[i][j]).sum();
        assert!(e <= g.value + 1e-7);
    }
}

#[test]
fn cpm_and_networks() {
    let acts = vec![
        crate::project::Activity { name: "A".into(), from: 1, to: 2, duration: 4.0 },
        crate::project::Activity { name: "B".into(), from: 1, to: 3, duration: 6.0 },
        crate::project::Activity { name: "C".into(), from: 2, to: 4, duration: 3.0 },
        crate::project::Activity { name: "D".into(), from: 3, to: 4, duration: 5.0 },
        crate::project::Activity { name: "E".into(), from: 2, to: 5, duration: 8.0 },
        crate::project::Activity { name: "F".into(), from: 4, to: 5, duration: 2.0 },
        crate::project::Activity { name: "G".into(), from: 5, to: 6, duration: 3.0 },
    ];
    let r = crate::project::cpm(&acts).unwrap();
    assert!(close(r.duration, 16.0));
    let inf = f64::INFINITY;
    let d = vec![
        vec![inf, 7.0, 3.0, inf],
        vec![inf, inf, inf, 2.0],
        vec![inf, 2.0, inf, 8.0],
        vec![inf, inf, inf, inf],
    ];
    let (dist, _, _) = crate::network::dijkstra(&d, 0);
    assert!(close(dist[3], 7.0));
    let fl = crate::network::floyd(&d);
    assert!(close(fl.last().unwrap().d[0][3], 7.0));
    assert_eq!(crate::network::floyd_route(&fl.last().unwrap().s, 0, 3), vec![0, 2, 1, 3]);
    let cap = vec![
        vec![0.0, 12.0, 9.0, 0.0],
        vec![0.0, 0.0, 4.0, 8.0],
        vec![0.0, 0.0, 0.0, 10.0],
        vec![0.0, 0.0, 0.0, 0.0],
    ];
    assert!(close(crate::network::max_flow(&cap, 0, 3).total, 18.0));
    let mst = crate::network::minimal_spanning_tree(&vec![vec![inf, 1.0, 4.0], vec![inf, inf, 2.0], vec![inf, inf, inf]], 0).unwrap();
    assert!(close(mst.last().unwrap().total, 3.0));
}

#[test]
fn linear_equations() {
    let a = vec![vec![2.0, 1.0, -1.0], vec![-3.0, -1.0, 2.0], vec![-2.0, 1.0, 2.0]];
    let r = crate::linalg::gauss_jordan(&a, Some(&[8.0, -11.0, -3.0]));
    let x = r.solution.unwrap();
    assert!(close(x[0], 2.0) && close(x[1], 3.0) && close(x[2], -1.0));
    assert!(close(r.determinant, -1.0));
}

#[test]
fn legacy_lp_file_imports() {
    let text = "5000\r\n5\r\n2\r\n1\r\n\"Exemplo\"\r\n2\r\n1\r\n\"Maximize\"\r\n\"x1\"\r\n\"x2\"\r\n\"\"\r\n\"\"\r\n\"3\"\r\n\"2\"\r\n\"\"\r\n\"\"\r\n\"1\"\r\n\"1\"\r\n\"<\"\r\n\"4.5\"\r\n\"0\"\r\n\"0\"\r\n\"\"\r\n\"\"\r\n\"infinity\"\r\n\"3\"\r\n\"\"\r\n\"\"\r\n\"n\"\r\n\"n\"\r\n\"\"\r\n\"\"\r\n\"y\"\r\n\"n\"\r\n\"\"\r\n\"\"\r\n";
    assert!(crate::legacy::looks_legacy(text));
    let imp = crate::legacy::import(text).unwrap();
    assert_eq!(imp.module, "integer_programming");
    let d = &imp.data;
    assert_eq!(d["nvars"], 2);
    assert_eq!(d["ncons"], 1);
    assert_eq!(d["maximize"], true);
    assert_eq!(d["rel"][0], "<=");
    assert_eq!(d["upper"][0], "∞");
    assert_eq!(d["upper"][1], "3");
    assert_eq!(d["integer"][0], true);
    assert_eq!(d["integer"][1], false);
    let back = crate::legacy::export(imp.module, d).unwrap();
    let again = crate::legacy::import(&back).unwrap();
    assert_eq!(again.data, imp.data);
}

#[test]
fn legacy_files_round_trip() {
    let samples = vec![
        ("transportation", serde_json::json!({"title":"T","ns":2,"nd":3,"sources":["A","B"],"dests":["X","Y","Z"],"cost":[["1","2","3"],["4","5","6"]],"supply":["10","20"],"demand":["5","10","15"]})),
        ("queuing", serde_json::json!({"title":"Q","rows":[["1","4","5","1","∞","∞"],["2","4","3","2","10","∞"]]})),
        ("shortest_route", serde_json::json!({"title":"R","n":3,"names":["N1","N2","N3"],"matrix":[["","2","5"],["","","1"],["","",""]]})),
        ("pert", serde_json::json!({"title":"P","rows":[["A","1","2","1","2","3"],["B","2","3","2","4","6"]]})),
        ("zero_sum_game", serde_json::json!({"title":"G","m":2,"n":2,"payoff":[["1","-1"],["-1","1"]]})),
        ("linear_equations", serde_json::json!({"title":"E","n":2,"a":[["1","2"],["3","4"]],"b":["5","6"]})),
    ];
    for (module, data) in samples {
        let text = crate::legacy::export(module, &data).unwrap();
        let imp = crate::legacy::import(&text).unwrap();
        assert_eq!(imp.module, module);
        assert_eq!(imp.data, data, "{}", module);
    }
}
