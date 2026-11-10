use crate::format::{num, parse_num, parse_or_zero};
use crate::grid::{resize_grid, resize_names, resize_vec};
use crate::ip::{branch_and_bound, BbNode, BbResult, NodeStatus};
use crate::lp::{self, GraphData, LpModel, LpReport};
use crate::report::*;
use crate::simplex::{ColKind, Method, Rel, Sense, SimplexRun, Status, Tableau};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LpInput {
    pub title: String,
    pub maximize: bool,
    pub nvars: usize,
    pub ncons: usize,
    pub var_names: Vec<String>,
    pub obj: Vec<String>,
    pub con_names: Vec<String>,
    pub a: Vec<Vec<String>>,
    pub rel: Vec<String>,
    pub rhs: Vec<String>,
    pub lower: Vec<String>,
    pub upper: Vec<String>,
    pub unrestricted: Vec<bool>,
    pub integer: Vec<bool>,
}

impl LpInput {
    pub fn blank(nv: usize, nc: usize, integer: bool) -> Self {
        let mut s = Self {
            title: String::new(),
            maximize: true,
            nvars: nv,
            ncons: nc,
            var_names: vec![],
            obj: vec![],
            con_names: vec![],
            a: vec![],
            rel: vec![],
            rhs: vec![],
            lower: vec![],
            upper: vec![],
            unrestricted: vec![],
            integer: vec![],
        };
        s.fit(integer);
        s
    }

    pub fn fit(&mut self, integer_default: bool) {
        resize_names(&mut self.var_names, self.nvars, "x");
        resize_vec(&mut self.obj, self.nvars, String::new());
        resize_names(&mut self.con_names, self.ncons, "Restr ");
        resize_grid(&mut self.a, self.ncons, self.nvars);
        resize_vec(&mut self.rel, self.ncons, "<=".to_string());
        resize_vec(&mut self.rhs, self.ncons, String::new());
        resize_vec(&mut self.lower, self.nvars, "0".to_string());
        resize_vec(&mut self.upper, self.nvars, "∞".to_string());
        resize_vec(&mut self.unrestricted, self.nvars, false);
        resize_vec(&mut self.integer, self.nvars, integer_default);
    }

    pub fn to_model(&self) -> Result<LpModel, String> {
        let n = self.nvars;
        let mut lower = Vec::new();
        let mut upper = Vec::new();
        for j in 0..n {
            let lo = if self.unrestricted[j] {
                f64::NEG_INFINITY
            } else {
                crate::mechanics::bound_value(parse_num(&self.lower[j]).unwrap_or(0.0))
            };
            let up = crate::mechanics::bound_value(parse_num(&self.upper[j]).unwrap_or(f64::INFINITY));
            if !self.unrestricted[j] && lo < 0.0 && lo.is_infinite() {
                return Err("Variável não negativa: o limite inferior deve ser finito.".into());
            }
            lower.push(lo);
            upper.push(up);
        }
        Ok(LpModel {
            sense: if self.maximize { Sense::Max } else { Sense::Min },
            var_names: self.var_names.clone(),
            c: self.obj.iter().map(|s| parse_or_zero(s)).collect(),
            a: self.a.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect(),
            rel: self.rel.iter().map(|s| Rel::parse(s)).collect(),
            b: self.rhs.iter().map(|s| parse_or_zero(s)).collect(),
            con_names: self.con_names.clone(),
            lower,
            upper,
            integer: self.integer.clone(),
        })
    }

    pub fn example(integer_mode: bool) -> Self {
        let mut inp = LpInput::blank(2, 3, integer_mode);
        if integer_mode {
            inp.title = "Exemplo inteiro: dois produtos".into();
            inp.obj = vec!["7".into(), "5".into()];
            inp.a = vec![vec!["3".into(), "2".into()], vec!["1".into(), "3".into()], vec!["0".into(), "1".into()]];
            inp.rel = vec!["<=".into(), "<=".into(), "<=".into()];
            inp.rhs = vec!["13.5".into(), "12".into(), "3.5".into()];
        } else {
            inp.title = "Exemplo: mix de duas tintas".into();
            inp.obj = vec!["4".into(), "3".into()];
            inp.a = vec![vec!["2".into(), "1".into()], vec!["1".into(), "3".into()], vec!["1".into(), "0".into()]];
            inp.rel = vec!["<=".into(), "<=".into(), "<=".into()];
            inp.rhs = vec!["10".into(), "15".into(), "4".into()];
        }
        inp.con_names = vec!["Recurso A".into(), "Recurso B".into(), "Recurso C".into()];
        inp
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterMethod {
    Auto,
    M,
    TwoPhase,
    Dual,
}

impl IterMethod {
    pub const ALL: [IterMethod; 4] = [IterMethod::Auto, IterMethod::M, IterMethod::TwoPhase, IterMethod::Dual];

    pub fn label(&self) -> &'static str {
        match self {
            IterMethod::Auto => "Automático",
            IterMethod::M => "M grande",
            IterMethod::TwoPhase => "Duas fases",
            IterMethod::Dual => "Dual simplex",
        }
    }

    fn method(&self) -> Option<Method> {
        match self {
            IterMethod::Auto => None,
            IterMethod::M => Some(Method::MMethod),
            IterMethod::TwoPhase => Some(Method::TwoPhase),
            IterMethod::Dual => Some(Method::DualSimplex),
        }
    }
}

pub struct LpSolved {
    pub results: Vec<Section>,
    pub notice: Option<String>,
}

pub fn lp_results(input: &LpInput, model: &LpModel) -> Result<LpSolved, String> {
    let sol = lp::solve(model, None, 100.0)?;
    Ok(LpSolved {
        results: summary_sections(input, model, &sol.report),
        notice: sol.notice,
    })
}

pub struct LpIterations {
    pub sections: Vec<Section>,
    pub notice: Option<String>,
}

pub fn lp_iterations(model: &LpModel, method: IterMethod, big_m: &str) -> Result<LpIterations, String> {
    let big_m = parse_num(big_m).unwrap_or(crate::mechanics::BIG_M_DEFAULT);
    let (mut run, _tr, notice) = lp::prepare(model, method.method(), big_m)?;
    run.run_all();
    let mut secs: Vec<Section> = Vec::new();
    if let Some(nt) = &notice {
        let mut s = Section::new("Aviso");
        s.note(nt.clone());
        secs.push(s);
    }
    secs.extend(tableau_sections(&run, model.sense));
    Ok(LpIterations { sections: secs, notice })
}

pub struct IpOutcome {
    pub results: Vec<Section>,
    pub iterations: Vec<Section>,
    pub bb: BbResult,
}

pub fn ip_solve(model: &LpModel) -> IpOutcome {
    let res = branch_and_bound(model);
    let mut out = Vec::new();
    let mut s = Section::new("Programação inteira: resumo do Branch & Bound");
    match res.best {
        Some(id) => {
            let node = &res.nodes[id - 1];
            s.line(format!(
                "Solução ótima encontrada no subproblema {}, valor objetivo = {}",
                id,
                num(node.z.unwrap_or(0.0))
            ));
            if let Some(last) = res.nodes.last() {
                s.line(format!("Resultado verificado na iteração {}", last.id));
            }
            out.push(s);
            let mut v = Section::new("Solução ótima").headers(&["Variável", "Valor", "Coef. obj.", "Contribuição"]);
            for j in 0..model.c.len() {
                v.row(vec![
                    cs(model.var_names[j].clone(), CellStyle::Head),
                    cs(num(node.x[j]), CellStyle::Bold),
                    n(model.c[j]),
                    n(model.c[j] * node.x[j]),
                ]);
            }
            out.push(v);
        }
        None => {
            s.line(if res.nodes.iter().any(|n| n.status == NodeStatus::Unbounded) {
                "A solução é ilimitada.".to_string()
            } else {
                "Nenhuma solução viável encontrada.".to_string()
            });
            out.push(s);
        }
    }
    if res.aborted {
        let mut a = Section::new("Busca excessiva");
        a.note("O número de subproblemas examinados excedeu 32.000. Execução interrompida.");
        out.push(a);
    }
    if !res.improvements.is_empty() {
        let mut f = Section::new("Soluções viáveis (em ordem de melhoria)").headers(&["Subproblema", "Valor objetivo", "Solução"]);
        for &id in &res.improvements {
            let nd = &res.nodes[id - 1];
            f.row(vec![c(format!("N{}", id)), n(nd.z.unwrap_or(0.0)), c(fmt_x(&model.var_names, &nd.x))]);
        }
        out.push(f);
    }
    let mut it = Vec::new();
    for nd in &res.nodes {
        let mut s = Section::new(format!("Subproblema N{}", nd.id)).sub(match nd.parent {
            Some(p) => format!("Ramo a partir de N{}: {}", p, nd.branch),
            None => "LP contínuo (sem restrições de integralidade)".into(),
        });
        let status = node_status_text(nd.status, nd.branch_var.map(|j| model.var_names[j].as_str()));
        s.line(format!("Situação: {}", status));
        if let Some(z) = nd.z {
            s.line(format!("Valor objetivo z = {}", num(z)));
        }
        if !nd.x.is_empty() {
            s.line(format!("Solução: {}", fmt_x(&model.var_names, &nd.x)));
        }
        it.push(s);
    }
    IpOutcome { results: out, iterations: it, bb: res }
}

pub fn path_text(nodes: &[BbNode], id: usize) -> String {
    let mut parts = Vec::new();
    let mut cur = Some(id);
    while let Some(i) = cur {
        let nd = &nodes[i - 1];
        if nd.parent.is_some() {
            parts.push(nd.branch.clone());
        }
        cur = nd.parent;
    }
    parts.reverse();
    if parts.is_empty() {
        "nenhum (LP contínuo)".into()
    } else {
        parts.join(", ")
    }
}

pub fn node_status_text(s: NodeStatus, var: Option<&str>) -> String {
    match s {
        NodeStatus::Infeasible => "solução inviável, sondado".into(),
        NodeStatus::Unbounded => "ilimitado".into(),
        NodeStatus::Integer => "solução inteira".into(),
        NodeStatus::Fathomed => "sondado (valor não melhora a melhor solução)".into(),
        NodeStatus::Branched => format!("ramificado na variável {}", var.unwrap_or("?")),
    }
}

pub fn fmt_x(names: &[String], x: &[f64]) -> String {
    names
        .iter()
        .zip(x.iter())
        .map(|(n, v)| format!("{} = {}", n, num(*v)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub type Segment = ((f64, f64), (f64, f64));

pub fn line_in_box(a: f64, b: f64, r: f64, xmax: f64, ymax: f64) -> Option<Segment> {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut push = |p: (f64, f64)| {
        if p.0 >= -1e-9
            && p.0 <= xmax + 1e-9
            && p.1 >= -1e-9
            && p.1 <= ymax + 1e-9
            && !pts.iter().any(|q| (q.0 - p.0).abs() < 1e-9 && (q.1 - p.1).abs() < 1e-9)
        {
            pts.push(p);
        }
    };
    if b.abs() > 1e-12 {
        push((0.0, r / b));
        push((xmax, (r - a * xmax) / b));
    }
    if a.abs() > 1e-12 {
        push((r / a, 0.0));
        push(((r - b * ymax) / a, ymax));
    }
    if pts.len() >= 2 {
        Some((pts[0], pts[1]))
    } else {
        None
    }
}

pub fn graph_summary(model: &LpModel, g: &GraphData) -> String {
    match (g.status, g.optimum) {
        (Status::Optimal, Some((x1, x2))) => format!(
            "Ótimo: {} = {}, {} = {}, z = {}",
            model.var_names[0],
            num(x1),
            model.var_names[1],
            num(x2),
            num(g.z)
        ),
        (Status::Unbounded, _) => "A solução ótima é ILIMITADA.".into(),
        _ => "O problema não tem solução viável.".into(),
    }
}

fn summary_sections(input: &LpInput, model: &LpModel, rep: &LpReport) -> Vec<Section> {
    let mut out = Vec::new();
    let mut head = Section::new("Resumo da solução: Programação Linear").sub(format!(
        "{}. {}. Iterações: {}",
        if input.title.is_empty() { "Sem título" } else { input.title.as_str() },
        rep.method.label(),
        rep.iterations
    ));
    match rep.status {
        Status::Optimal => head.line(format!(
            "Valor objetivo ({}) = {}",
            if model.sense == Sense::Max { "Max" } else { "Min" },
            num(rep.z)
        )),
        Status::Unbounded => head.line("A solução ótima é ilimitada.".to_string()),
        Status::Infeasible => head.line("O problema não tem solução viável.".to_string()),
    }
    if rep.alternative {
        head.note("Há soluções ótimas alternativas (veja a aba Iterações: variável não básica com zj-cj = 0 no último quadro).");
    }
    out.push(head);
    if rep.status == Status::Unbounded || rep.status == Status::Infeasible {
        return out;
    }
    let mut v = Section::new("Variáveis").headers(&["Variável", "Valor", "Coef. obj.", "Contribuição"]);
    for r in &rep.vars {
        v.row(vec![cs(r.name.clone(), CellStyle::Head), cs(num(r.value), CellStyle::Bold), n(r.c), n(r.contribution)]);
    }
    out.push(v);
    let mut cns = Section::new("Restrições").headers(&["Restrição", "LE", "", "LD", "Folga(-)/Excesso(+)"]);
    for r in &rep.cons {
        let sl = match r.rel {
            Rel::Le => format!("{}-", num(r.slack)),
            Rel::Ge => format!("{}+", num(r.slack)),
            Rel::Eq => num(0.0),
        };
        cns.row(vec![cs(r.name.clone(), CellStyle::Head), n(r.lhs), c(r.rel.symbol()), n(r.rhs), c(sl)]);
    }
    out.push(cns);
    let mut s1 = Section::new("Análise de sensibilidade: coeficientes do objetivo").headers(&["Variável", "Coef. atual", "Coef. mín.", "Coef. máx.", "Custo reduzido"]);
    for r in &rep.vars {
        s1.row(vec![
            cs(r.name.clone(), CellStyle::Head),
            n(r.c),
            r.cmin.map(n).unwrap_or_else(|| c("–")),
            r.cmax.map(n).unwrap_or_else(|| c("–")),
            r.reduced.map(|v| n(v.abs())).unwrap_or_else(|| c("–")),
        ]);
    }
    out.push(s1);
    let mut s2 = Section::new("Análise de sensibilidade: lado direito").headers(&["Restrição", "Preço dual", "LD mín.", "LD atual", "LD máx."]);
    for r in &rep.cons {
        s2.row(vec![cs(r.name.clone(), CellStyle::Head), n(r.dual), n(r.rmin), n(r.rhs), n(r.rmax)]);
    }
    s2.note("Preço dual = variação do valor objetivo por unidade de aumento no lado direito, dentro do intervalo [mín., máx.].");
    out.push(s2);
    out
}

fn tableau_sections(run: &SimplexRun, sense: Sense) -> Vec<Section> {
    run.history
        .iter()
        .enumerate()
        .map(|(k, t)| tableau_section(t, k, run.method, sense))
        .collect()
}

fn tableau_section(t: &Tableau, k: usize, method: Method, _sense: Sense) -> Section {
    let phase = match t.phase {
        1 => ": Fase 1".to_string(),
        2 => ": Fase 2".to_string(),
        _ => String::new(),
    };
    let mut s = Section::new(format!("Iteração {}{}", k + 1, phase));
    let mut sub = method.label().to_string();
    if let Some(e) = t.entering {
        sub.push_str(&format!(". Entra: {}", t.col_names[e]));
    }
    if let Some(r) = t.leaving {
        sub.push_str(&format!(". Sai: {}", t.col_names[t.basis[r]]));
    }
    s.subtitle = sub;
    let mut headers = vec!["Básica".to_string()];
    for (j, name) in t.col_names.iter().enumerate() {
        if t.blocked[j] {
            headers.push(format!("{} (bloq.)", name));
        } else {
            headers.push(name.clone());
        }
    }
    headers.push("Solução".into());
    s.headers = headers;
    let zlabel = if t.phase == 1 {
        "r (min)".to_string()
    } else if t.sense == Sense::Max {
        "z (max)".to_string()
    } else {
        "z (min)".to_string()
    };
    let mut zr = vec![cs(zlabel, CellStyle::Bold)];
    for j in 0..t.col_names.len() {
        let st = if Some(j) == t.entering {
            CellStyle::Green
        } else if t.blocked[j] {
            CellStyle::Muted
        } else {
            CellStyle::Bold
        };
        zr.push(cs(if t.blocked[j] { "bloq.".into() } else { num(t.zrow[j]) }, st));
    }
    zr.push(cs(num(t.zval), CellStyle::Bold));
    s.row(zr);
    for i in 0..t.rows.len() {
        let leaving = Some(i) == t.leaving;
        let bname = t.col_names[t.basis[i]].clone();
        let mut row = vec![cs(bname, if leaving { CellStyle::Red } else { CellStyle::Head })];
        for j in 0..t.col_names.len() {
            let st = if leaving && Some(j) == t.entering {
                CellStyle::Pivot
            } else if leaving {
                CellStyle::Red
            } else if Some(j) == t.entering {
                CellStyle::Green
            } else if t.col_kind[j] == ColKind::Artificial && t.blocked[j] {
                CellStyle::Muted
            } else {
                CellStyle::Normal
            };
            row.push(cs(num(t.rows[i][j]), st));
        }
        row.push(cs(num(t.rhs[i]), if leaving { CellStyle::Red } else { CellStyle::Normal }));
        s.row(row);
    }
    if !t.note.is_empty() {
        s.note(t.note.clone());
    }
    s
}

pub fn report(input: &LpInput, integer_mode: bool, results: &[Section], iterations: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    let mut all = results.to_vec();
    if !integer_mode {
        all.extend(iterations.iter().cloned());
    }
    let t = if integer_mode { "PROGRAMAÇÃO INTEIRA" } else { "PROGRAMAÇÃO LINEAR" };
    Some(sections_to_text(&format!("{}: {}", t, input.title), &all))
}
