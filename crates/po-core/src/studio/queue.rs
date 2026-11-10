use crate::format::{num, parse_num, parse_or_zero};
use crate::queue::{analyse, QueueKind, QueueResult};
use crate::report::*;
use serde::{Deserialize, Serialize};

pub const HEADERS: [&str; 6] = ["Cenário", "Lambda (λ)", "Mu (μ)", "Servidores c", "Limite do sistema N", "Limite da fonte K"];

#[derive(Clone, Serialize, Deserialize)]
pub struct QueueInputGrid {
    pub title: String,
    pub rows: Vec<Vec<String>>,
}

fn default_row() -> Vec<String> {
    vec![String::new(), String::new(), String::new(), "1".into(), "∞".into(), "∞".into()]
}

impl QueueInputGrid {
    pub fn blank() -> Self {
        Self {
            title: String::new(),
            rows: vec![vec!["1".into(), String::new(), String::new(), "1".into(), "∞".into(), "∞".into()]],
        }
    }

    pub fn fit(&mut self) {
        for r in self.rows.iter_mut() {
            r.resize(6, String::new());
        }
    }

    pub fn set_count(&mut self, count: usize) {
        let old = self.rows.len();
        self.rows.resize(count, default_row());
        for i in old..count {
            self.rows[i][0] = format!("{}", i + 1);
        }
    }

    pub fn example() -> Self {
        let rows = [
            ["1", "6", "8", "1", "∞", "∞"],
            ["2", "6", "8", "2", "∞", "∞"],
            ["3", "6", "8", "1", "5", "∞"],
            ["4", "2", "3", "2", "10", "10"],
        ];
        Self {
            title: "Comparação de configurações de atendimento".into(),
            rows: rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
        }
    }
}

pub struct QueueOutcome {
    pub results: Vec<Section>,
    pub details: Vec<Section>,
}

pub fn solve(input: &QueueInputGrid) -> Result<QueueOutcome, String> {
    let mut res: Vec<QueueResult> = Vec::new();
    for (k, r) in input.rows.iter().enumerate() {
        let q = crate::queue::QueueInput {
            name: if r[0].trim().is_empty() { format!("{}", k + 1) } else { r[0].trim().to_string() },
            lambda: parse_or_zero(&r[1]),
            mu: parse_or_zero(&r[2]),
            servers: parse_num(&r[3]).map(|v| v.max(1.0) as usize).unwrap_or(1),
            system_limit: crate::mechanics::queue_limit(parse_num(&r[4]).unwrap_or(f64::INFINITY)),
            source_limit: crate::mechanics::queue_limit(parse_num(&r[5]).unwrap_or(f64::INFINITY)),
        };
        res.push(analyse(&q)?);
    }
    let mut comp = Section::new("Análise comparativa").sub(title_or_untitled(&input.title));
    comp.headers = ["Cenário", "Modelo", "c", "λ", "μ", "λ efetivo", "p0", "Ls", "Ws", "Lq", "Wq"].iter().map(|s| s.to_string()).collect();
    for r in &res {
        comp.row(vec![
            cs(r.input.name.clone(), CellStyle::Head),
            c(r.label.split(": ").nth(1).unwrap_or("").to_string()),
            c(r.input.servers.to_string()),
            n(r.input.lambda),
            n(r.input.mu),
            n(r.lambda_eff),
            n(r.probs.first().copied().unwrap_or(0.0)),
            cs(num(r.ls), CellStyle::Bold),
            n(r.ws),
            n(r.lq),
            n(r.wq),
        ]);
    }
    let details = res
        .iter()
        .map(|r| {
            let mut s = Section::new(format!("Cenário {}", r.label)).headers(&["n", "Probabilidade pn", "Acumulada Pn"]);
            if r.kind == QueueKind::General {
                s.line(format!(
                    "Lambda = {}   Mu = {}   λ efetivo = {}   ρ/c = {}",
                    num(r.input.lambda),
                    num(r.input.mu),
                    num(r.lambda_eff),
                    num(r.rho_c)
                ));
                s.line(format!("Ls = {}   Lq = {}   Ws = {}   Wq = {}", num(r.ls), num(r.lq), num(r.ws), num(r.wq)));
            } else {
                s.line(format!("Número médio no sistema = {}", num(r.ls)));
            }
            let mut cum = 0.0;
            for (k, p) in r.probs.iter().enumerate() {
                cum += p;
                s.row(vec![cs(k.to_string(), CellStyle::Head), n(*p), n(cum)]);
            }
            s
        })
        .collect();
    Ok(QueueOutcome { results: vec![comp], details })
}

pub fn report(input: &QueueInputGrid, results: &[Section], details: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    let mut all = results.to_vec();
    all.extend(details.iter().cloned());
    Some(sections_to_text(&format!("ANÁLISE DE FILAS: {}", input.title), &all))
}
