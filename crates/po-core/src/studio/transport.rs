use crate::format::{num, parse_or_zero};
use crate::grid::{resize_grid, resize_names, resize_vec};
use crate::report::*;
use crate::transport::{solve as solve_transport, StartMethod, TransportProblem};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TransportInput {
    pub title: String,
    pub ns: usize,
    pub nd: usize,
    pub sources: Vec<String>,
    pub dests: Vec<String>,
    pub cost: Vec<Vec<String>>,
    pub supply: Vec<String>,
    pub demand: Vec<String>,
}

impl TransportInput {
    pub fn blank(ns: usize, nd: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            ns,
            nd,
            sources: vec![],
            dests: vec![],
            cost: vec![],
            supply: vec![],
            demand: vec![],
        };
        s.fit();
        s
    }

    pub fn fit(&mut self) {
        resize_names(&mut self.sources, self.ns, "Origem ");
        resize_names(&mut self.dests, self.nd, "Destino ");
        resize_grid(&mut self.cost, self.ns, self.nd);
        resize_vec(&mut self.supply, self.ns, String::new());
        resize_vec(&mut self.demand, self.nd, String::new());
    }

    pub fn example() -> Self {
        let cost = [["8", "6", "10", "9"], ["9", "12", "13", "7"], ["14", "9", "16", "5"]];
        let mut inp = TransportInput::blank(3, 4);
        inp.title = "Exemplo: três fábricas, quatro lojas".into();
        inp.cost = cost.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect();
        inp.supply = vec!["35".into(), "50".into(), "40".into()];
        inp.demand = vec!["45".into(), "20".into(), "30".into(), "30".into()];
        inp
    }
}

pub struct TransportOutcome {
    pub results: Vec<Section>,
    pub iterations: Vec<Section>,
}

pub fn solve(input: &TransportInput, method: StartMethod) -> Result<TransportOutcome, String> {
    let p = TransportProblem {
        sources: input.sources.clone(),
        dests: input.dests.clone(),
        cost: input.cost.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect(),
        supply: input.supply.iter().map(|s| parse_or_zero(s)).collect(),
        demand: input.demand.iter().map(|s| parse_or_zero(s)).collect(),
    };
    let sol = solve_transport(&p, method)?;
    let q = &sol.problem;
    let last = sol.tableaus.last().unwrap();
    let mut head = Section::new("Resumo da solução: Modelo de Transporte").sub(format!(
        "{}. Solução inicial: {}. Iterações: {}",
        if input.title.is_empty() { "Sem título" } else { input.title.as_str() },
        method.label(),
        sol.tableaus.len()
    ));
    head.line(format!("Valor objetivo (custo mínimo) = {}", num(last.cost)));
    if q.sources.len() > p.sources.len() {
        head.note("Demanda total maior que a oferta: foi criada a origem fictícia DummyS.");
    }
    if q.dests.len() > p.dests.len() {
        head.note("Oferta total maior que a demanda: foi criado o destino fictício DummyD.");
    }
    let mut ship = Section::new("Remessas").headers(&["De", "Para", "Quantidade", "Custo unit.", "Contribuição"]);
    for i in 0..q.sources.len() {
        for j in 0..q.dests.len() {
            let a = last.alloc[i][j];
            if a > 1e-9 {
                ship.row(vec![
                    cs(q.sources[i].clone(), CellStyle::Head),
                    c(q.dests[j].clone()),
                    cs(num(a), CellStyle::Bold),
                    n(q.cost[i][j]),
                    n(a * q.cost[i][j]),
                ]);
            }
        }
    }
    let mut its = Vec::new();
    for (k, t) in sol.tableaus.iter().enumerate() {
        let mut s = Section::new(format!("Quadro de transporte {}", k + 1));
        let mut sub = format!("Custo total = {}", num(t.cost));
        if let Some((i, j)) = t.entering {
            sub.push_str(&format!(". Entra: ({}, {})", q.sources[i], q.dests[j]));
            sub.push_str(&format!(", θ = {}", num(t.theta)));
        }
        if let Some((i, j)) = t.leaving {
            sub.push_str(&format!(". Sai: ({}, {})", q.sources[i], q.dests[j]));
        }
        s.subtitle = sub;
        let mut h = vec![String::new()];
        h.extend(q.dests.iter().cloned());
        h.push("Oferta".into());
        h.push("u".into());
        s.headers = h;
        for i in 0..q.sources.len() {
            let mut row = vec![cs(q.sources[i].clone(), CellStyle::Head)];
            for j in 0..q.dests.len() {
                let pos = t.loop_cells.iter().position(|&cell| cell == (i, j));
                let cost = num(q.cost[i][j]);
                let txt = if t.basic[i][j] {
                    format!("{}  [c={}]", num(t.alloc[i][j]), cost)
                } else {
                    format!("({})  [c={}]", num(t.reduced[i][j]), cost)
                };
                let st = match pos {
                    Some(0) => CellStyle::Green,
                    Some(p) if p % 2 == 1 => CellStyle::Red,
                    Some(_) => CellStyle::Pivot,
                    None if t.basic[i][j] => CellStyle::Bold,
                    None => CellStyle::Muted,
                };
                let mark = match pos {
                    Some(p) if p % 2 == 0 => " +",
                    Some(_) => " −",
                    None => "",
                };
                row.push(cs(format!("{}{}", txt, mark), st));
            }
            row.push(n(q.supply[i]));
            row.push(cs(num(t.u[i]), CellStyle::Bold));
            s.row(row);
        }
        let mut dem = vec![cs("Demanda", CellStyle::Head)];
        dem.extend(q.demand.iter().map(|d| n(*d)));
        s.row(dem);
        let mut vr = vec![cs("v", CellStyle::Head)];
        vr.extend(t.v.iter().map(|v| cs(num(*v), CellStyle::Bold)));
        s.row(vr);
        if t.entering.is_none() {
            s.note("Último quadro: ótimo (todos os custos reduzidos cij − ui − vj ≥ 0).");
        } else {
            s.note("Valores entre parênteses = cij − ui − vj das células não básicas. Verde = entra; + / − = laço de troca de base.");
        }
        its.push(s);
    }
    Ok(TransportOutcome {
        results: vec![head, ship],
        iterations: its,
    })
}

pub fn report(input: &TransportInput, results: &[Section], iterations: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    let mut all = results.to_vec();
    all.extend(iterations.iter().cloned());
    Some(sections_to_text(&format!("MODELO DE TRANSPORTE: {}", input.title), &all))
}
