use crate::simplex::{all_le_after_normalising, Method, Rel, Sense, SimplexRun, Status, StdForm};

#[derive(Clone, Debug)]
pub struct LpModel {
    pub sense: Sense,
    pub var_names: Vec<String>,
    pub c: Vec<f64>,
    pub a: Vec<Vec<f64>>,
    pub rel: Vec<Rel>,
    pub b: Vec<f64>,
    pub con_names: Vec<String>,
    pub lower: Vec<f64>,
    pub upper: Vec<f64>,
    pub integer: Vec<bool>,
}

#[derive(Clone, Debug)]
pub enum VarMap {
    Shifted { col: usize, lower: f64 },
    Split { plus: usize, minus: usize },
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub vars: Vec<VarMap>,
    pub bound_rows: Vec<(usize, usize, f64)>,
    pub obj_const: f64,
    pub n_user_rows: usize,
}

pub fn to_std(model: &LpModel) -> Result<(StdForm, Transform), String> {
    let n = model.c.len();
    let m = model.a.len();
    for j in 0..n {
        if model.lower[j] > model.upper[j] {
            return Err(format!(
                "Limite inferior maior que o superior na variável {}.",
                model.var_names[j]
            ));
        }
    }
    for i in 0..m {
        if model.a[i].iter().all(|v| *v == 0.0) {
            return Err(format!(
                "Todos os coeficientes da restrição {} são zero. Corrija ou apague a restrição.",
                i + 1
            ));
        }
    }
    let mut names = Vec::new();
    let mut cols_c = Vec::new();
    let mut vars = Vec::new();
    let mut obj_const = 0.0;
    let mut b = model.b.clone();
    for j in 0..n {
        if model.lower[j].is_infinite() {
            let plus = names.len();
            names.push(format!("{}+", model.var_names[j]));
            cols_c.push(model.c[j]);
            let minus = names.len();
            names.push(format!("{}-", model.var_names[j]));
            cols_c.push(-model.c[j]);
            vars.push(VarMap::Split { plus, minus });
        } else {
            let l = model.lower[j];
            let col = names.len();
            names.push(model.var_names[j].clone());
            cols_c.push(model.c[j]);
            if l != 0.0 {
                obj_const += model.c[j] * l;
                for i in 0..m {
                    b[i] -= model.a[i][j] * l;
                }
            }
            vars.push(VarMap::Shifted { col, lower: l });
        }
    }
    let ncols = names.len();
    let mut a: Vec<Vec<f64>> = Vec::new();
    for i in 0..m {
        let mut row = vec![0.0; ncols];
        for j in 0..n {
            match vars[j] {
                VarMap::Shifted { col, .. } => row[col] += model.a[i][j],
                VarMap::Split { plus, minus } => {
                    row[plus] += model.a[i][j];
                    row[minus] -= model.a[i][j];
                }
            }
        }
        a.push(row);
    }
    let mut rel = model.rel.clone();
    let mut bound_rows = Vec::new();
    for j in 0..n {
        if model.upper[j].is_finite() {
            let mut row = vec![0.0; ncols];
            let rhs = match vars[j] {
                VarMap::Shifted { col, lower } => {
                    row[col] = 1.0;
                    model.upper[j] - lower
                }
                VarMap::Split { plus, minus } => {
                    row[plus] = 1.0;
                    row[minus] = -1.0;
                    model.upper[j]
                }
            };
            bound_rows.push((j, a.len(), model.upper[j]));
            a.push(row);
            rel.push(Rel::Le);
            b.push(rhs);
        }
    }
    Ok((
        StdForm {
            sense: model.sense,
            names,
            c: cols_c,
            a,
            rel,
            b,
        },
        Transform {
            vars,
            bound_rows,
            obj_const,
            n_user_rows: m,
        },
    ))
}

pub fn auto_method(form: &StdForm) -> Method {
    if all_le_after_normalising(form) {
        Method::AllSlack
    } else {
        Method::TwoPhase
    }
}

#[derive(Clone, Debug)]
pub struct VarReport {
    pub name: String,
    pub value: f64,
    pub c: f64,
    pub contribution: f64,
    pub reduced: Option<f64>,
    pub cmin: Option<f64>,
    pub cmax: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct ConReport {
    pub name: String,
    pub lhs: f64,
    pub rel: Rel,
    pub rhs: f64,
    pub slack: f64,
    pub dual: f64,
    pub rmin: f64,
    pub rmax: f64,
}

#[derive(Clone, Debug)]
pub struct LpReport {
    pub status: Status,
    pub z: f64,
    pub vars: Vec<VarReport>,
    pub cons: Vec<ConReport>,
    pub alternative: bool,
    pub iterations: usize,
    pub method: Method,
}

pub fn user_values(run: &SimplexRun, tr: &Transform) -> Vec<f64> {
    let colv = run.column_values();
    tr.vars
        .iter()
        .map(|v| match *v {
            VarMap::Shifted { col, lower } => colv[col] + lower,
            VarMap::Split { plus, minus } => colv[plus] - colv[minus],
        })
        .collect()
}

pub fn build_report(model: &LpModel, run: &SimplexRun, tr: &Transform) -> LpReport {
    let status = run.status.unwrap_or(Status::Infeasible);
    let x = user_values(run, tr);
    let z = run.objective_orig() + tr.obj_const;
    let red = run.reduced_orig();
    let duals = run.duals_std();
    let mut vars = Vec::new();
    for j in 0..model.c.len() {
        let (reduced, cmin, cmax) = match tr.vars[j] {
            VarMap::Shifted { col, .. } if status == Status::Optimal => {
                let (lo, hi) = run.cost_range(col);
                (Some(red[col]), Some(lo), Some(hi))
            }
            _ => (None, None, None),
        };
        vars.push(VarReport {
            name: model.var_names[j].clone(),
            value: x[j],
            c: model.c[j],
            contribution: model.c[j] * x[j],
            reduced,
            cmin,
            cmax,
        });
    }
    let mut cons = Vec::new();
    for i in 0..model.a.len() {
        let lhs: f64 = model.a[i].iter().zip(x.iter()).map(|(a, v)| a * v).sum();
        let slack = match model.rel[i] {
            Rel::Le => model.b[i] - lhs,
            Rel::Ge => lhs - model.b[i],
            Rel::Eq => 0.0,
        };
        let (dual, rmin, rmax) = if status == Status::Optimal {
            let (lo, hi) = run.rhs_range_std(i);
            (duals[i] * run.row_sign[i], model.b[i] + lo, model.b[i] + hi)
        } else {
            (f64::NAN, f64::NAN, f64::NAN)
        };
        cons.push(ConReport {
            name: model.con_names[i].clone(),
            lhs,
            rel: model.rel[i],
            rhs: model.b[i],
            slack,
            dual,
            rmin,
            rmax,
        });
    }
    for &(j, row, ub) in &tr.bound_rows {
        let lhs = x[j];
        let (dual, rmin, rmax) = if status == Status::Optimal {
            let (lo, hi) = run.rhs_range_std(row);
            (duals[row] * run.row_sign[row], ub + lo, ub + hi)
        } else {
            (f64::NAN, f64::NAN, f64::NAN)
        };
        cons.push(ConReport {
            name: format!("UB-{}", model.var_names[j]),
            lhs,
            rel: Rel::Le,
            rhs: ub,
            slack: ub - lhs,
            dual,
            rmin,
            rmax,
        });
    }
    LpReport {
        status,
        z,
        vars,
        cons,
        alternative: run.alternative,
        iterations: run.history.len(),
        method: run.method,
    }
}

pub struct LpSolve {
    pub run: SimplexRun,
    pub transform: Transform,
    pub report: LpReport,
    pub notice: Option<String>,
}

pub fn prepare(model: &LpModel, method: Option<Method>, big_m: f64) -> Result<(SimplexRun, Transform, Option<String>), String> {
    let (form, tr) = to_std(model)?;
    let auto = auto_method(&form);
    let wanted = method.unwrap_or(auto);
    let mut notice = None;
    let chosen = match wanted {
        Method::AllSlack if auto != Method::AllSlack => {
            notice = Some("O problema tem restrições >= e/ou =, exigindo M grande ou duas fases. Foi usado o método das duas fases.".to_string());
            Method::TwoPhase
        }
        Method::DualSimplex => match SimplexRun::new(&form, Method::DualSimplex, big_m) {
            Ok(run) => return Ok((run, tr, None)),
            Err(e) => {
                notice = Some(format!("{} Foi usado o método das duas fases.", e));
                Method::TwoPhase
            }
        },
        m => m,
    };
    let run = SimplexRun::new(&form, chosen, big_m)?;
    Ok((run, tr, notice))
}

pub fn solve(model: &LpModel, method: Option<Method>, big_m: f64) -> Result<LpSolve, String> {
    let (mut run, transform, notice) = prepare(model, method, big_m)?;
    run.run_all();
    let report = build_report(model, &run, &transform);
    Ok(LpSolve {
        run,
        transform,
        report,
        notice,
    })
}

pub fn lp_status_only(model: &LpModel) -> Option<(Status, f64, Vec<f64>)> {
    let (mut run, tr, _) = prepare(model, None, 1e6).ok()?;
    run.run_all();
    let st = run.status?;
    let x = user_values(&run, &tr);
    let z = run.objective_orig() + tr.obj_const;
    Some((st, z, x))
}

pub fn clip_polygon(poly: &[(f64, f64)], a: f64, b: f64, rhs: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if poly.is_empty() {
        return out;
    }
    let inside = |p: &(f64, f64)| a * p.0 + b * p.1 <= rhs + 1e-9;
    for i in 0..poly.len() {
        let cur = poly[i];
        let prev = poly[(i + poly.len() - 1) % poly.len()];
        let ci = inside(&cur);
        let pi = inside(&prev);
        if ci {
            if !pi {
                out.push(intersect(prev, cur, a, b, rhs));
            }
            out.push(cur);
        } else if pi {
            out.push(intersect(prev, cur, a, b, rhs));
        }
    }
    out
}

fn intersect(p: (f64, f64), q: (f64, f64), a: f64, b: f64, rhs: f64) -> (f64, f64) {
    let fp = a * p.0 + b * p.1 - rhs;
    let fq = a * q.0 + b * q.1 - rhs;
    let t = if (fp - fq).abs() < 1e-15 { 0.0 } else { fp / (fp - fq) };
    (p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1))
}

pub struct GraphData {
    pub xmax: f64,
    pub ymax: f64,
    pub region: Vec<(f64, f64)>,
    pub status: Status,
    pub optimum: Option<(f64, f64)>,
    pub z: f64,
}

pub fn graph_data(model: &LpModel) -> Result<GraphData, String> {
    if model.c.len() != 2 {
        return Err("O método gráfico exige exatamente 2 variáveis.".into());
    }
    if model.lower.iter().any(|l| *l != 0.0) || model.upper.iter().any(|u| u.is_finite()) {
        return Err("PLs com variáveis irrestritas e/ou limites explícitos não são aceitos pelo método gráfico.".into());
    }
    let mut xmax: f64 = 0.0;
    let mut ymax: f64 = 0.0;
    for i in 0..model.a.len() {
        let (a, b, r) = (model.a[i][0], model.a[i][1], model.b[i]);
        if a.abs() > 1e-12 && r / a > 0.0 {
            xmax = xmax.max(r / a);
        }
        if b.abs() > 1e-12 && r / b > 0.0 {
            ymax = ymax.max(r / b);
        }
    }
    let solved = lp_status_only(model);
    if let Some((Status::Optimal, _, ref x)) = solved {
        xmax = xmax.max(x[0]);
        ymax = ymax.max(x[1]);
    }
    if xmax <= 0.0 {
        xmax = ymax.max(10.0);
    }
    if ymax <= 0.0 {
        ymax = xmax.max(10.0);
    }
    xmax *= 1.25;
    ymax *= 1.25;
    let mut poly = vec![(0.0, 0.0), (xmax, 0.0), (xmax, ymax), (0.0, ymax)];
    for i in 0..model.a.len() {
        let (a, b, r) = (model.a[i][0], model.a[i][1], model.b[i]);
        match model.rel[i] {
            Rel::Le => poly = clip_polygon(&poly, a, b, r),
            Rel::Ge => poly = clip_polygon(&poly, -a, -b, -r),
            Rel::Eq => {
                poly = clip_polygon(&poly, a, b, r);
                poly = clip_polygon(&poly, -a, -b, -r);
            }
        }
    }
    let (status, z, optimum) = match solved {
        Some((st, z, x)) => (st, z, if st == Status::Optimal { Some((x[0], x[1])) } else { None }),
        None => (Status::Infeasible, 0.0, None),
    };
    Ok(GraphData {
        xmax,
        ymax,
        region: poly,
        status,
        optimum,
        z,
    })
}
