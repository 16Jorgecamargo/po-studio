use po_core::format::{num, parse_num, set_format as core_set_format, NumFormat};
use po_core::ip::NodeStatus;
use po_core::lp::graph_data;
use po_core::project::ActivitySchedule;
use po_core::report::Section;
use po_core::simplex::Status;
use po_core::studio::algebra::{self, AlgebraInput};
use po_core::studio::game::{self, GameInput};
use po_core::studio::lp::{self, IterMethod, LpInput};
use po_core::studio::network::{self, NetInput, NetKind, ShortAlgo};
use po_core::studio::project::{self, ProjectInput};
use po_core::studio::queue::{self, QueueInputGrid};
use po_core::studio::transport::{self, TransportInput};
use po_core::studio::{self as studio, MODULES};
use po_core::transport::StartMethod;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

const APP_NAME: &str = "PO Studio";

fn ok(v: Value) -> String {
    json!({ "ok": true, "value": v }).to_string()
}

fn fail(e: impl Into<String>) -> String {
    json!({ "ok": false, "error": e.into() }).to_string()
}

fn info(e: impl Into<String>) -> String {
    json!({ "ok": false, "error": e.into(), "info": true }).to_string()
}

fn parse<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

fn to_value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

fn net_kind(module: &str) -> Option<NetKind> {
    match module {
        "minimal_spanning_tree" => Some(NetKind::Mst),
        "shortest_route" => Some(NetKind::Shortest),
        "maximal_flow" => Some(NetKind::MaxFlow),
        _ => None,
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Options {
    method: Option<String>,
    big_m: Option<String>,
    algo: Option<String>,
    start: Option<usize>,
    target: Option<usize>,
}

impl Options {
    fn iter_method(&self) -> IterMethod {
        match self.method.as_deref() {
            Some("m") => IterMethod::M,
            Some("two_phase") => IterMethod::TwoPhase,
            Some("dual") => IterMethod::Dual,
            _ => IterMethod::Auto,
        }
    }

    fn start_method(&self) -> StartMethod {
        match self.method.as_deref() {
            Some("least_cost") => StartMethod::LeastCost,
            Some("vogel") => StartMethod::Vogel,
            _ => StartMethod::NorthWest,
        }
    }

    fn algo(&self) -> ShortAlgo {
        match self.algo.as_deref() {
            Some("floyd") => ShortAlgo::Floyd,
            _ => ShortAlgo::Dijkstra,
        }
    }
}

fn default_options(module: &str) -> Value {
    match module {
        "linear_programming" => json!({ "method": "auto", "big_m": "100" }),
        "transportation" => json!({ "method": "north_west" }),
        "minimal_spanning_tree" | "shortest_route" | "maximal_flow" => json!({ "algo": "dijkstra", "start": 0, "target": 4 }),
        _ => json!({}),
    }
}

#[wasm_bindgen]
pub fn catalog() -> String {
    let modules: Vec<Value> = MODULES
        .iter()
        .map(|m| {
            let hint = match m.key {
                "linear_programming" | "integer_programming" => studio::LP_HINT,
                "cpm" | "pert" => studio::PROJECT_HINT,
                "queuing" => studio::QUEUE_HINT,
                "zero_sum_game" => studio::GAME_HINT,
                k => net_kind(k).map(|n| n.hint()).unwrap_or(""),
            };
            json!({ "key": m.key, "group": m.group, "name": m.name, "blurb": m.blurb, "hint": hint })
        })
        .collect();
    json!({
        "modules": modules,
        "maxSize": po_core::mechanics::MAX_PROBLEM_SIZE,
        "queueHeaders": queue::HEADERS,
        "cpmHeaders": project::headers(false),
        "pertHeaders": project::headers(true),
        "iterMethods": IterMethod::ALL.iter().map(|m| json!({ "key": to_value(m), "label": m.label() })).collect::<Vec<_>>(),
        "startMethods": [
            { "key": "north_west", "label": StartMethod::NorthWest.label() },
            { "key": "least_cost", "label": StartMethod::LeastCost.label() },
            { "key": "vogel", "label": StartMethod::Vogel.label() },
        ],
    })
    .to_string()
}

#[wasm_bindgen]
pub fn set_format(scientific: bool, width: usize, decimals: usize) {
    core_set_format(NumFormat { scientific, width, decimals });
}

#[wasm_bindgen]
pub fn format_num(v: f64) -> String {
    num(v)
}

fn blank_input(module: &str) -> Result<Value, String> {
    Ok(match module {
        "linear_equations" | "matrix_inverse" => to_value(&AlgebraInput::blank(3)),
        "linear_programming" => to_value(&LpInput::blank(2, 2, false)),
        "integer_programming" => to_value(&LpInput::blank(2, 2, true)),
        "transportation" => to_value(&TransportInput::blank(3, 4)),
        "minimal_spanning_tree" | "shortest_route" | "maximal_flow" => to_value(&NetInput::blank(5)),
        "cpm" => to_value(&ProjectInput::blank(false)),
        "pert" => to_value(&ProjectInput::blank(true)),
        "queuing" => to_value(&QueueInputGrid::blank()),
        "zero_sum_game" => to_value(&GameInput::blank(2, 3)),
        _ => return Err(format!("Módulo desconhecido: {}", module)),
    })
}

#[wasm_bindgen]
pub fn blank(module: &str) -> String {
    match blank_input(module) {
        Ok(input) => ok(json!({ "input": input, "options": default_options(module) })),
        Err(e) => fail(e),
    }
}

#[wasm_bindgen]
pub fn example(module: &str) -> String {
    let mut options = default_options(module);
    let input = match module {
        "linear_equations" => to_value(&AlgebraInput::example(true)),
        "matrix_inverse" => to_value(&AlgebraInput::example(false)),
        "linear_programming" => to_value(&LpInput::example(false)),
        "integer_programming" => to_value(&LpInput::example(true)),
        "transportation" => to_value(&TransportInput::example()),
        "cpm" => to_value(&ProjectInput::example(false)),
        "pert" => to_value(&ProjectInput::example(true)),
        "queuing" => to_value(&QueueInputGrid::example()),
        "zero_sum_game" => to_value(&GameInput::example()),
        k => match net_kind(k) {
            Some(kind) => {
                let (inp, s, t) = NetInput::example(kind);
                options["start"] = json!(s);
                options["target"] = json!(t);
                to_value(&inp)
            }
            None => return fail(format!("Módulo desconhecido: {}", module)),
        },
    };
    ok(json!({ "input": input, "options": options }))
}

fn fit_value(module: &str, data: Value) -> Result<Value, String> {
    let de = |e: serde_json::Error| e.to_string();
    Ok(match module {
        "linear_equations" | "matrix_inverse" => {
            let mut v: AlgebraInput = serde_json::from_value(data).map_err(de)?;
            v.n = v.n.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.fit();
            to_value(&v)
        }
        "linear_programming" | "integer_programming" => {
            let mut v: LpInput = serde_json::from_value(data).map_err(de)?;
            v.nvars = v.nvars.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.ncons = v.ncons.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.fit(module == "integer_programming");
            to_value(&v)
        }
        "transportation" => {
            let mut v: TransportInput = serde_json::from_value(data).map_err(de)?;
            v.ns = v.ns.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.nd = v.nd.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.fit();
            to_value(&v)
        }
        "minimal_spanning_tree" | "shortest_route" | "maximal_flow" => {
            let mut v: NetInput = serde_json::from_value(data).map_err(de)?;
            v.n = v.n.clamp(2, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.fit();
            to_value(&v)
        }
        "cpm" | "pert" => {
            let mut v: ProjectInput = serde_json::from_value(data).map_err(de)?;
            v.fit(module == "pert");
            to_value(&v)
        }
        "queuing" => {
            let mut v: QueueInputGrid = serde_json::from_value(data).map_err(de)?;
            v.fit();
            to_value(&v)
        }
        "zero_sum_game" => {
            let mut v: GameInput = serde_json::from_value(data).map_err(de)?;
            v.m = v.m.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.n = v.n.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
            v.fit();
            to_value(&v)
        }
        _ => return Err(format!("Módulo desconhecido: {}", module)),
    })
}

#[wasm_bindgen]
pub fn fit(module: &str, input: &str) -> String {
    match parse::<Value>(input).and_then(|v| fit_value(module, v)) {
        Ok(v) => ok(v),
        Err(e) => fail(e),
    }
}

#[wasm_bindgen]
pub fn set_count(module: &str, input: &str, count: usize) -> String {
    let count = count.clamp(1, po_core::mechanics::MAX_PROBLEM_SIZE);
    let res = match module {
        "queuing" => parse::<QueueInputGrid>(input).map(|mut v| {
            v.set_count(count);
            to_value(&v)
        }),
        "cpm" | "pert" => parse::<ProjectInput>(input).map(|mut v| {
            v.set_count(count, module == "pert");
            to_value(&v)
        }),
        _ => Err("Este módulo não usa contagem de linhas.".into()),
    };
    match res {
        Ok(v) => ok(v),
        Err(e) => fail(e),
    }
}

fn outcome(results: &[Section], iterations: &[Section], notice: Option<String>, report: Option<String>, view: Value) -> Value {
    json!({
        "results": results,
        "iterations": iterations,
        "notice": notice,
        "report": report.unwrap_or_default(),
        "view": view,
    })
}

fn node_status_key(s: NodeStatus) -> &'static str {
    match s {
        NodeStatus::Infeasible => "infeasible",
        NodeStatus::Unbounded => "unbounded",
        NodeStatus::Integer => "integer",
        NodeStatus::Fathomed => "fathomed",
        NodeStatus::Branched => "branched",
    }
}

fn solve_module(module: &str, input: &str, options: &Options) -> Result<Value, String> {
    match module {
        "linear_equations" | "matrix_inverse" => {
            let with_rhs = module == "linear_equations";
            let mut inp: AlgebraInput = parse(input)?;
            inp.fit();
            let o = algebra::solve(&inp, with_rhs);
            let report = algebra::report(&inp, with_rhs, &o.results);
            Ok(outcome(&o.results, &o.steps, None, report, json!({})))
        }
        "linear_programming" => {
            let mut inp: LpInput = parse(input)?;
            inp.fit(false);
            let model = inp.to_model()?;
            let solved = lp::lp_results(&inp, &model)?;
            let mut notice = solved.notice.clone();
            let (iterations, iter_error) = match lp::lp_iterations(&model, options.iter_method(), options.big_m.as_deref().unwrap_or("100")) {
                Ok(it) => {
                    if notice.is_none() {
                        notice = it.notice;
                    }
                    (it.sections, None)
                }
                Err(e) => (vec![], Some(e)),
            };
            let report = lp::report(&inp, false, &solved.results, &iterations);
            let mut out = outcome(&solved.results, &iterations, notice, report, json!({}));
            out["iterError"] = json!(iter_error);
            Ok(out)
        }
        "integer_programming" => {
            let mut inp: LpInput = parse(input)?;
            inp.fit(true);
            let model = inp.to_model()?;
            let o = lp::ip_solve(&model);
            let nodes: Vec<Value> = o
                .bb
                .nodes
                .iter()
                .map(|nd| {
                    json!({
                        "id": nd.id,
                        "parent": nd.parent,
                        "depth": nd.depth,
                        "branch": nd.branch,
                        "z": nd.z,
                        "zText": nd.z.map(num),
                        "status": node_status_key(nd.status),
                        "statusText": lp::node_status_text(nd.status, nd.branch_var.map(|j| model.var_names[j].as_str())),
                        "pathText": lp::path_text(&o.bb.nodes, nd.id),
                        "solutionText": if nd.x.is_empty() { Value::Null } else { json!(lp::fmt_x(&model.var_names, &nd.x)) },
                    })
                })
                .collect();
            let report = lp::report(&inp, true, &o.results, &o.iterations);
            Ok(outcome(&o.results, &o.iterations, None, report, json!({ "tree": { "nodes": nodes, "best": o.bb.best, "aborted": o.bb.aborted } })))
        }
        "transportation" => {
            let mut inp: TransportInput = parse(input)?;
            inp.fit();
            let o = transport::solve(&inp, options.start_method())?;
            let report = transport::report(&inp, &o.results, &o.iterations);
            Ok(outcome(&o.results, &o.iterations, None, report, json!({})))
        }
        "cpm" | "pert" => {
            let pert_mode = module == "pert";
            let mut inp: ProjectInput = parse(input)?;
            inp.fit(pert_mode);
            let o = project::solve(&inp, pert_mode)?;
            let report = project::report(&inp, pert_mode, &o.results, &o.iterations);
            let view = json!({
                "activities": o.schedule.activities,
                "duration": o.schedule.duration,
                "pathStats": o.path_stats.map(|(mean, sd)| json!({ "mean": mean, "sd": sd })),
            });
            Ok(outcome(&o.results, &o.iterations, None, report, view))
        }
        "queuing" => {
            let mut inp: QueueInputGrid = parse(input)?;
            inp.fit();
            let o = queue::solve(&inp)?;
            let report = queue::report(&inp, &o.results, &o.details);
            let names: Vec<String> = inp.rows.iter().map(|r| r[0].clone()).collect();
            Ok(outcome(&o.results, &o.details, None, report, json!({ "scenarios": names })))
        }
        "zero_sum_game" => {
            let mut inp: GameInput = parse(input)?;
            inp.fit();
            let results = game::solve(&inp)?;
            let report = game::report(&inp, &results);
            Ok(outcome(&results, &[], None, report, json!({})))
        }
        k => {
            let kind = net_kind(k).ok_or_else(|| format!("Módulo desconhecido: {}", k))?;
            let mut inp: NetInput = parse(input)?;
            inp.fit();
            let o = network::solve(&inp, kind, options.algo(), options.start.unwrap_or(0), options.target.unwrap_or(4))?;
            let report = network::report(&inp, kind, &o.results, &o.iterations);
            let highlight: Vec<Value> = o.highlight.iter().map(|(a, b, l)| json!({ "from": a, "to": b, "label": l })).collect();
            Ok(outcome(&o.results, &o.iterations, None, report, json!({ "highlight": highlight, "start": o.start, "target": o.target })))
        }
    }
}

#[wasm_bindgen]
pub fn solve(module: &str, input: &str, options: &str) -> String {
    let opts: Options = serde_json::from_str(options).unwrap_or_default();
    match solve_module(module, input, &opts) {
        Ok(v) => ok(v),
        Err(e) => fail(e),
    }
}

#[wasm_bindgen]
pub fn lp_iterations(input: &str, options: &str) -> String {
    let opts: Options = serde_json::from_str(options).unwrap_or_default();
    let res = parse::<LpInput>(input).and_then(|mut inp| {
        inp.fit(false);
        let model = inp.to_model()?;
        lp::lp_iterations(&model, opts.iter_method(), opts.big_m.as_deref().unwrap_or("100"))
    });
    match res {
        Ok(it) => ok(json!({ "iterations": it.sections, "notice": it.notice })),
        Err(e) => fail(e),
    }
}

#[wasm_bindgen]
pub fn lp_graph(input: &str) -> String {
    let mut inp: LpInput = match parse(input) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    inp.fit(false);
    let model = match inp.to_model() {
        Ok(m) => m,
        Err(e) => return fail(e),
    };
    let g = match graph_data(&model) {
        Ok(g) => g,
        Err(e) => return info(e),
    };
    let lines: Vec<Value> = (0..model.a.len())
        .map(|i| {
            let seg = lp::line_in_box(model.a[i][0], model.a[i][1], model.b[i], g.xmax, g.ymax);
            json!({
                "label": format!("({}) {}", i + 1, model.con_names[i]),
                "tag": format!("({})", i + 1),
                "segment": seg.map(|(p, q)| json!([[p.0, p.1], [q.0, q.1]])),
            })
        })
        .collect();
    let status = match g.status {
        Status::Optimal => "optimal",
        Status::Unbounded => "unbounded",
        Status::Infeasible => "infeasible",
    };
    ok(json!({
        "xmax": g.xmax,
        "ymax": g.ymax,
        "region": g.region.iter().map(|p| json!([p.0, p.1])).collect::<Vec<_>>(),
        "status": status,
        "optimum": g.optimum.map(|p| json!([p.0, p.1])),
        "z": g.z,
        "c": [model.c[0], model.c[1]],
        "summary": lp::graph_summary(&model, &g),
        "marker": g.optimum.map(|(x1, x2)| format!("Ótimo ({}, {})", num(x1), num(x2))),
        "xLabel": model.var_names[0],
        "yLabel": model.var_names[1],
        "lines": lines,
    }))
}

#[wasm_bindgen]
pub fn game_graph(input: &str) -> String {
    let mut inp: GameInput = match parse(input) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    inp.fit();
    match game::graph(&inp) {
        Ok(g) => ok(to_value(&g)),
        Err(e) => info(e),
    }
}

#[wasm_bindgen]
pub fn network_arcs(input: &str) -> String {
    let mut inp: NetInput = match parse(input) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    inp.fit();
    let d = inp.distances();
    let mut arcs = Vec::new();
    for (i, row) in d.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if i != j && v.is_finite() {
                arcs.push(json!({ "from": i, "to": j, "value": v, "label": num(*v) }));
            }
        }
    }
    ok(json!({ "names": inp.names, "arcs": arcs }))
}

#[wasm_bindgen]
pub fn delay_message(activities: &str, index: usize, delay: f64) -> String {
    let acts: Vec<ActivitySchedule> = serde_json::from_str(activities).unwrap_or_default();
    project::delay_message(&acts, index, delay).unwrap_or_default()
}

#[wasm_bindgen]
pub fn deadline_probability(mean: f64, sd: f64, deadline: &str) -> String {
    match parse_num(deadline) {
        Some(t) => format!("P(duração ≤ {}) = {}", num(t), num(project::deadline_probability(mean, sd, t))),
        None => String::new(),
    }
}

#[wasm_bindgen]
pub fn open_file(text: &str) -> String {
    let doc: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) if po_core::legacy::looks_legacy(text) => match po_core::legacy::import(text) {
            Ok(imp) => json!({ "module": imp.module, "data": imp.data }),
            Err(e) => return fail(format!("Arquivo do programa original não reconhecido: {}", e)),
        },
        Err(_) => return fail("O arquivo não é um problema válido."),
    };
    let module = doc.get("module").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !MODULES.iter().any(|m| m.key == module) {
        return fail("O arquivo não corresponde a nenhum módulo.");
    }
    match fit_value(&module, doc.get("data").cloned().unwrap_or_default()) {
        Ok(data) => ok(json!({ "module": module, "data": data })),
        Err(e) => fail(format!("O arquivo não corresponde ao formato do módulo escolhido: {}", e)),
    }
}

#[wasm_bindgen]
pub fn save_file(module: &str, data: &str, legacy: bool) -> String {
    let value: Value = match parse(data) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    if legacy {
        return match po_core::legacy::export(module, &value) {
            Ok(t) => ok(json!(t)),
            Err(e) => fail(e),
        };
    }
    let doc = json!({ "app": APP_NAME, "version": 1, "module": module, "data": value });
    ok(json!(serde_json::to_string_pretty(&doc).unwrap_or_default()))
}
