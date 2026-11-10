use super::common::*;
use super::{results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_or_zero};
use crate::game::solve;
use eframe::egui::{self, Align2, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameInput {
    pub title: String,
    pub m: usize,
    pub n: usize,
    pub payoff: Vec<Vec<String>>,
}

impl GameInput {
    fn blank(m: usize, n: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            m,
            n,
            payoff: vec![],
        };
        s.fit();
        s
    }
    fn fit(&mut self) {
        resize_grid(&mut self.payoff, self.m, self.n);
    }
}

pub struct GameModule {
    pub input: GameInput,
    tab: Tab,
    results: Vec<Section>,
    error: Option<String>,
}

impl GameModule {
    pub fn new() -> Self {
        Self {
            input: GameInput::blank(2, 3),
            tab: Tab::Data,
            results: vec![],
            error: None,
        }
    }

    fn matrix(&self) -> Vec<Vec<f64>> {
        self.input.payoff.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect()
    }

    fn solve(&mut self) {
        self.error = None;
        let a = self.matrix();
        match solve(&a) {
            Err(e) => self.error = Some(e),
            Ok(g) => {
                let mut h = Section::new("Jogo de soma zero entre dois jogadores: resumo").sub(if self.input.title.is_empty() { "Sem título".to_string() } else { self.input.title.clone() });
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
                orig.headers = std::iter::once(String::new()).chain((1..=self.input.n).map(|j| format!("B{}", j))).collect();
                for (i, r) in a.iter().enumerate() {
                    let mut row = vec![cs(format!("A{}", i + 1), CellStyle::Head)];
                    row.extend(r.iter().map(|v| n(*v)));
                    orig.row(row);
                }
                self.results = vec![h, sa, sb, lp, orig];
                self.tab = Tab::Results;
            }
        }
    }

    fn graph_ui(&mut self, ui: &mut egui::Ui) {
        let a = self.matrix();
        let (m, n) = (self.input.m, self.input.n);
        if m != 2 && n != 2 {
            info_box(ui, "A solução gráfica exige que um dos jogadores tenha exatamente 2 estratégias.");
            return;
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
        card(ui, |ui| {
            ui.label(RichText::new(format!("Solução gráfica, mistura do jogador {}: p = probabilidade de {}1", who, who)).font(semibold(15.0)).color(text_strong()));
            ui.label(format!("Ótimo: {}1 = {}, {}2 = {}. Valor do jogo = {}", who, num(best_p), who, num(1.0 - best_p), num(best_v)));
            ui.label(RichText::new(if row_player { "Linhas = pagamento esperado de A contra cada estratégia de B; A maximiza o envelope inferior." } else { "Linhas = pagamento esperado contra cada estratégia de A; B minimiza o envelope superior." }).small().color(text_muted()));
        });
        ui.add_space(10.0);
        let mut ymin = lines.iter().flat_map(|(x, y)| [*x, *y]).fold(f64::INFINITY, f64::min);
        let mut ymax = lines.iter().flat_map(|(x, y)| [*x, *y]).fold(f64::NEG_INFINITY, f64::max);
        if (ymax - ymin).abs() < 1e-9 {
            ymin -= 1.0;
            ymax += 1.0;
        }
        let pad = (ymax - ymin) * 0.1;
        let h = (ui.available_height() - 10.0).max(360.0);
        let (painter, plot, _) = plot_area(ui, h, 0.0, 1.0, ymin - pad, ymax + pad, &format!("p ({}1)", who), "Pagamento esperado");
        let lines_end = 0.42 + 0.09 * lines.len().saturating_sub(1) as f32;
        let trace_end = lines_end + 0.7;
        let elapsed = super::motion::clock(ui.ctx(), "game_graph", trace_end + 0.9);
        for (k, (v0, v1)) in lines.iter().enumerate() {
            let col = series(k);
            let p0 = plot.to_screen(0.0, *v0);
            let p1 = plot.to_screen(1.0, *v1);
            let t = super::motion::ease_in_out_cubic(super::motion::phase(elapsed, k as f32 * 0.09, 0.42));
            partial_segment(&painter, p0, p1, t, Stroke::new(2.0, col));
            if t >= 1.0 {
                let lbl = if row_player { format!("B{}", k + 1) } else { format!("A{}", k + 1) };
                painter.text(p1 + Vec2::new(-4.0, -4.0), Align2::RIGHT_BOTTOM, lbl, semibold(12.5), col);
            }
        }
        let steps = 200;
        let traced = super::motion::ease_in_out_cubic(super::motion::phase(elapsed, lines_end, trace_end - lines_end));
        let shown = (steps as f32 * traced) as usize;
        if shown >= 1 {
            let pts: Vec<_> = (0..=shown)
                .map(|s| {
                    let p = s as f64 / steps as f64;
                    plot.to_screen(p, env(p))
                })
                .collect();
            painter.add(egui::Shape::line(pts, Stroke::new(3.5, envelope_line())));
        }
        let o = plot.to_screen(best_p, best_v);
        let pop = optimum_marker(&painter, o, elapsed, trace_end, red());
        if pop > 0.5 {
            painter.text(o + Vec2::new(10.0, -8.0), Align2::LEFT_BOTTOM, format!("Ótimo ({}, {})", num(best_p), num(best_v)), semibold(13.0), highlight_text());
        }
    }
}

impl Module for GameModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        "zero_sum_game"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Graph, "Gráfico")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let mut ch = stepper(ui, "Estratégias de A", &mut self.input.m, 1, crate::mechanics::MAX_PROBLEM_SIZE);
                        ui.add_space(20.0);
                        ch |= stepper(ui, "Estratégias de B", &mut self.input.n, 1, crate::mechanics::MAX_PROBLEM_SIZE);
                        if ch {
                            self.input.fit();
                        }
                    });
                    ui.label(RichText::new("Os pagamentos são do ponto de vista do jogador A (linhas).").small().color(text_muted()));
                });
                ui.add_space(10.0);
                let (m, n) = (self.input.m, self.input.n);
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("g_grid").max_height(540.0).show(ui, |ui| {
                        egui::Grid::new("g_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            head(ui, "A \\ B");
                            for j in 0..n {
                                head(ui, &format!("B{}", j + 1));
                            }
                            ui.end_row();
                            for i in 0..m {
                                head(ui, &format!("A{}", i + 1));
                                for j in 0..n {
                                    cell_edit(ui, &mut self.input.payoff[i][j], 64.0);
                                }
                                ui.end_row();
                            }
                        });
                    });
                });
                if let Some(e) = &self.error {
                    ui.add_space(8.0);
                    error_box(ui, e);
                }
                if solve_bar(ui) {
                    self.solve();
                }
            }
            Tab::Results => results_view(ui, "g_res", &self.results),
            Tab::Graph => self.graph_ui(ui),
            Tab::Iterations => {}
        }
    }

    fn report(&self) -> Option<String> {
        if self.results.is_empty() {
            return None;
        }
        Some(sections_to_text(&format!("JOGO DE SOMA ZERO: {}", self.input.title), &self.results))
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: GameInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit();
        self.input = inp;
        self.results.clear();
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        *self = Self::new();
    }

    fn load_example(&mut self) {
        let p = [["3", "-1", "2"], ["-2", "4", "1"]];
        self.input = GameInput {
            title: "Jogo de exemplo 2x3".into(),
            m: 2,
            n: 3,
            payoff: p.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
        };
        self.results.clear();
        self.tab = Tab::Data;
    }
}
