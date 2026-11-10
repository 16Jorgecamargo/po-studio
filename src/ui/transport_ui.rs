use super::common::*;
use super::{iterations_view, results_view, solve_bar, tab_bar, title_row, Module, Tab};
pub use crate::studio::transport::TransportInput;
use crate::transport::StartMethod;
use eframe::egui::{self, RichText};

pub struct TransportModule {
    pub input: TransportInput,
    method: StartMethod,
    tab: Tab,
    results: Vec<Section>,
    iterations: Vec<Section>,
    shown: usize,
    error: Option<String>,
}

impl TransportModule {
    pub fn new() -> Self {
        Self {
            input: TransportInput::blank(3, 4),
            method: StartMethod::NorthWest,
            tab: Tab::Data,
            results: vec![],
            iterations: vec![],
            shown: 1,
            error: None,
        }
    }

    fn solve(&mut self) {
        self.error = None;
        match crate::studio::transport::solve(&self.input, self.method) {
            Err(e) => self.error = Some(e),
            Ok(out) => {
                self.results = out.results;
                self.iterations = out.iterations;
                self.shown = 1;
                self.tab = Tab::Results;
            }
        }
    }
}

impl Module for TransportModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        "transportation"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Iterations, "Iterações")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let mut ch = stepper(ui, "Origens", &mut self.input.ns, 1, crate::mechanics::MAX_PROBLEM_SIZE);
                        ui.add_space(20.0);
                        ch |= stepper(ui, "Destinos", &mut self.input.nd, 1, crate::mechanics::MAX_PROBLEM_SIZE);
                        if ch {
                            self.input.fit();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Solução inicial:").color(text_muted()));
                        for m in [StartMethod::NorthWest, StartMethod::LeastCost, StartMethod::Vogel] {
                            ui.radio_value(&mut self.method, m, m.label());
                        }
                    });
                });
                ui.add_space(10.0);
                let (ns, nd) = (self.input.ns, self.input.nd);
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("tr_grid").max_height(540.0).show(ui, |ui| {
                        egui::Grid::new("tr_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            head(ui, "Custo unitário");
                            for j in 0..nd {
                                ui.add(egui::TextEdit::singleline(&mut self.input.dests[j]).desired_width(92.0).min_size(egui::Vec2::new(92.0, 0.0)).text_color(accent()));
                            }
                            head(ui, "Oferta");
                            ui.end_row();
                            for i in 0..ns {
                                ui.add(egui::TextEdit::singleline(&mut self.input.sources[i]).desired_width(100.0).text_color(accent()));
                                for j in 0..nd {
                                    cell_edit(ui, &mut self.input.cost[i][j], 92.0);
                                }
                                cell_edit(ui, &mut self.input.supply[i], 72.0);
                                ui.end_row();
                            }
                            head(ui, "Demanda");
                            for j in 0..nd {
                                cell_edit(ui, &mut self.input.demand[j], 92.0);
                            }
                            ui.end_row();
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
            Tab::Results => results_view(ui, "tr_res", &self.results),
            Tab::Iterations => iterations_view(ui, "tr_it", &self.iterations, &mut self.shown),
            Tab::Graph => {}
        }
    }

    fn report(&self) -> Option<String> {
        crate::studio::transport::report(&self.input, &self.results, &self.iterations)
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: TransportInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit();
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        self.input = TransportInput::blank(3, 4);
        self.results.clear();
        self.iterations.clear();
        self.error = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        self.input = TransportInput::example();
        self.results.clear();
        self.iterations.clear();
        self.tab = Tab::Data;
    }
}
