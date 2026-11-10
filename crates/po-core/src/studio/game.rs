use crate::format::{num, parse_or_zero};
use crate::game::solve as solve_game;
use crate::grid::resize_grid;
use crate::report::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameInput {
    pub title: String,
    pub m: usize,
    pub n: usize,
    pub payoff: Vec<Vec<String>>,
}

impl GameInput {
    pub fn blank(m: usize, n: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            m,
            n,
            payoff: vec![],
        };
        s.fit();
        s
    }

    pub fn fit(&mut self) {
        resize_grid(&mut self.payoff, self.m, self.n);
    }

    pub fn example() -> Self {
        let p = [["3", "-1", "2"], ["-2", "4", "1"]];
        Self {
            title: "Jogo de exemplo 2x3".into(),
            m: 2,
            n: 3,
            payoff: p.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
        }
    }

    pub fn matrix(&self) -> Vec<Vec<f64>> {
        self.payoff.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect()
    }
}

pub fn solve(input: &GameInput) -> Result<Vec<Section>, String> {
    let a = input.matrix();
    let g = solve_game(&a)?;
    let mut h = Section::new("Jogo de soma zero entre dois jogadores: resumo").sub(title_or_untitled(&input.title));
    h.line(format!("Valor do jogo para o jogador A = {}", num(g.value)));
    match g.saddle {
        Some((i, j)) => h.line(format!("Ponto de sela: estratégia pura A{} × B{} (maximin = minimax = {})", i + 1, j + 1, num(g.maximin))),
        None => h.line(format!("Sem ponto de sela (maximin = {}, minimax = {}): estratégias mistas.", num(g.maximin), num(g.minimax))),
    }
    let mut sa = Section::new("Estratégias ótimas do jogador A").headers(&["Estratégia", "Probabilidade"]);
    for (i, p) in g.a_mix.iter().enumerate() {
        sa.row(vec![cs(format!("A{}", i + 1), CellStyle::Head), cs(num(*p), CellStyle::Bold)]);
    }
    let mut sb = Section::new("Estratégias ótimas do jogador B").headers(&["Estratégia", "Probabilidade"]);
    for (j, p) in g.b_mix.iter().enumerate() {
        sb.row(vec![cs(format!("B{}", j + 1), CellStyle::Head), cs(num(*p), CellStyle::Bold)]);
    }
    sa.note("Podem existir ótimos alternativos para o jogador A.");
    sb.note("Podem existir ótimos alternativos para o jogador B.");
    let mut lp = Section::new("Formulação em PL do jogador A");
    if g.shift > 0.0 {
        lp.line(format!("Foi somada a constante k = {} a todos os pagamentos para torná-los positivos.", num(g.shift)));
    }
    lp.line(format!("Minimizar  {}", (1..=g.a_lp.c.len()).map(|i| format!("X{}", i)).collect::<Vec<_>>().join(" + ")));
    lp.line("sujeito a".to_string());
    for (j, row) in g.a_lp.a.iter().enumerate() {
        let terms: Vec<String> = row.iter().enumerate().map(|(i, v)| format!("{}·X{}", num(*v), i + 1)).collect();
        lp.line(format!("  {} >= 1    (contra B{})", terms.join(" + "), j + 1));
    }
    lp.line("  todos os Xi >= 0;  valor = 1 / ΣXi − k;  pi = Xi / ΣXi".to_string());
    let mut orig = Section::new("Dados originais do jogo (pagamentos ao jogador A)");
    orig.headers = std::iter::once(String::new()).chain((1..=input.n).map(|j| format!("B{}", j))).collect();
    for (i, r) in a.iter().enumerate() {
        let mut row = vec![cs(format!("A{}", i + 1), CellStyle::Head)];
        row.extend(r.iter().map(|v| n(*v)));
        orig.row(row);
    }
    Ok(vec![h, sa, sb, lp, orig])
}

#[derive(Clone, Debug, Serialize)]
pub struct GameGraph {
    pub row_player: bool,
    pub who: &'static str,
    pub lines: Vec<(f64, f64)>,
    pub best_p: f64,
    pub best_v: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub envelope: Vec<(f64, f64)>,
    pub heading: String,
    pub optimum: String,
    pub explanation: &'static str,
    pub marker: String,
    pub labels: Vec<String>,
    pub x_label: String,
}

pub const ENVELOPE_STEPS: usize = 200;

pub fn graph(input: &GameInput) -> Result<GameGraph, String> {
    let a = input.matrix();
    let (m, n) = (input.m, input.n);
    if m != 2 && n != 2 {
        return Err("A solução gráfica exige que um dos jogadores tenha exatamente 2 estratégias.".into());
    }
    let row_player = m == 2;
    let lines: Vec<(f64, f64)> = if row_player {
        (0..n).map(|j| (a[1][j], a[0][j])).collect()
    } else {
        (0..m).map(|i| (a[i][1], a[i][0])).collect()
    };
    let env = |p: f64| -> f64 {
        let vals = lines.iter().map(|(at0, at1)| at0 + (at1 - at0) * p);
        if row_player {
            vals.fold(f64::INFINITY, f64::min)
        } else {
            vals.fold(f64::NEG_INFINITY, f64::max)
        }
    };
    let mut best_p = 0.0;
    let mut best_v = env(0.0);
    let mut cands = vec![0.0, 1.0];
    for i in 0..lines.len() {
        for j in i + 1..lines.len() {
            let (a0, a1) = lines[i];
            let (b0, b1) = lines[j];
            let den = (a1 - a0) - (b1 - b0);
            if den.abs() > 1e-12 {
                let p = (b0 - a0) / den;
                if (0.0..=1.0).contains(&p) {
                    cands.push(p);
                }
            }
        }
    }
    for p in cands {
        let v = env(p);
        let better = if row_player { v > best_v + 1e-12 } else { v < best_v - 1e-12 };
        if better {
            best_v = v;
            best_p = p;
        }
    }
    let who = if row_player { "A" } else { "B" };
    let mut ymin = lines.iter().flat_map(|(x, y)| [*x, *y]).fold(f64::INFINITY, f64::min);
    let mut ymax = lines.iter().flat_map(|(x, y)| [*x, *y]).fold(f64::NEG_INFINITY, f64::max);
    if (ymax - ymin).abs() < 1e-9 {
        ymin -= 1.0;
        ymax += 1.0;
    }
    let envelope = (0..=ENVELOPE_STEPS)
        .map(|s| {
            let p = s as f64 / ENVELOPE_STEPS as f64;
            (p, env(p))
        })
        .collect();
    let labels = (0..lines.len()).map(|k| if row_player { format!("B{}", k + 1) } else { format!("A{}", k + 1) }).collect();
    Ok(GameGraph {
        row_player,
        who,
        lines,
        best_p,
        best_v,
        ymin,
        ymax,
        envelope,
        heading: format!("Solução gráfica, mistura do jogador {}: p = probabilidade de {}1", who, who),
        optimum: format!("Ótimo: {}1 = {}, {}2 = {}. Valor do jogo = {}", who, num(best_p), who, num(1.0 - best_p), num(best_v)),
        explanation: if row_player {
            "Linhas = pagamento esperado de A contra cada estratégia de B; A maximiza o envelope inferior."
        } else {
            "Linhas = pagamento esperado contra cada estratégia de A; B minimiza o envelope superior."
        },
        marker: format!("Ótimo ({}, {})", num(best_p), num(best_v)),
        labels,
        x_label: format!("p ({}1)", who),
    })
}

pub fn report(input: &GameInput, results: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    Some(sections_to_text(&format!("JOGO DE SOMA ZERO: {}", input.title), results))
}
