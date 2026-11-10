use super::common::*;
use super::{results_view, solve_bar, tab_bar, title_row, Module, Tab};
pub use crate::studio::game::GameInput;
use eframe::egui::{self, Align2, RichText, Stroke, Vec2};

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

    fn solve(&mut self) {
        self.error = None;
        match crate::studio::game::solve(&self.input) {
            Err(e) => self.error = Some(e),
            Ok(results) => {
                self.results = results;
                self.tab = Tab::Results;
            }
        }
    }

    fn graph_ui(&mut self, ui: &mut egui::Ui) {
        let g = match crate::studio::game::graph(&self.input) {
            Ok(g) => g,
            Err(e) => {
                info_box(ui, &e);
                return;
            }
        };
        card(ui, |ui| {
            ui.label(RichText::new(&g.heading).font(semibold(15.0)).color(text_strong()));
            ui.label(&g.optimum);
            ui.label(RichText::new(g.explanation).small().color(text_muted()));
        });
        ui.add_space(10.0);
        let pad = (g.ymax - g.ymin) * 0.1;
        let h = (ui.available_height() - 10.0).max(360.0);
        let (painter, plot, _) = plot_area(ui, h, 0.0, 1.0, g.ymin - pad, g.ymax + pad, &g.x_label, "Pagamento esperado");
        let lines_end = 0.42 + 0.09 * g.lines.len().saturating_sub(1) as f32;
        let trace_end = lines_end + 0.7;
        let elapsed = super::motion::clock(ui.ctx(), "game_graph", trace_end + 0.9);
        for (k, (v0, v1)) in g.lines.iter().enumerate() {
            let col = series(k);
            let p0 = plot.to_screen(0.0, *v0);
            let p1 = plot.to_screen(1.0, *v1);
            let t = super::motion::ease_in_out_cubic(super::motion::phase(elapsed, k as f32 * 0.09, 0.42));
            partial_segment(&painter, p0, p1, t, Stroke::new(2.0, col));
            if t >= 1.0 {
                painter.text(p1 + Vec2::new(-4.0, -4.0), Align2::RIGHT_BOTTOM, &g.labels[k], semibold(12.5), col);
            }
        }
        let steps = crate::studio::game::ENVELOPE_STEPS;
        let traced = super::motion::ease_in_out_cubic(super::motion::phase(elapsed, lines_end, trace_end - lines_end));
        let shown = (steps as f32 * traced) as usize;
        if shown >= 1 {
            let pts: Vec<_> = g.envelope.iter().take(shown + 1).map(|&(p, v)| plot.to_screen(p, v)).collect();
            painter.add(egui::Shape::line(pts, Stroke::new(3.5, envelope_line())));
        }
        let o = plot.to_screen(g.best_p, g.best_v);
        let pop = optimum_marker(&painter, o, elapsed, trace_end, red());
        if pop > 0.5 {
            painter.text(o + Vec2::new(10.0, -8.0), Align2::LEFT_BOTTOM, &g.marker, semibold(13.0), highlight_text());
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
                    ui.label(RichText::new(crate::studio::GAME_HINT).small().color(text_muted()));
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
        crate::studio::game::report(&self.input, &self.results)
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
        self.input = GameInput::example();
        self.results.clear();
        self.tab = Tab::Data;
    }
}
