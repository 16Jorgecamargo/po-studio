use super::common::*;
use super::{results_view, solve_bar, tab_bar, title_row, Module, Tab};
pub use crate::studio::queue::{QueueInputGrid, HEADERS};
use eframe::egui::{self, RichText};

pub struct QueueModule {
    pub input: QueueInputGrid,
    tab: Tab,
    results: Vec<Section>,
    details: Vec<Section>,
    error: Option<String>,
    selected: usize,
}

impl QueueModule {
    pub fn new() -> Self {
        Self {
            input: QueueInputGrid::blank(),
            tab: Tab::Data,
            results: vec![],
            details: vec![],
            error: None,
            selected: 0,
        }
    }

    fn solve(&mut self) {
        self.error = None;
        match crate::studio::queue::solve(&self.input) {
            Err(e) => self.error = Some(e),
            Ok(out) => {
                self.results = out.results;
                self.details = out.details;
                self.selected = 0;
                self.tab = Tab::Results;
            }
        }
    }
}

impl Module for QueueModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        "queuing"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Comparação"), (Tab::Iterations, "Probabilidades")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    let mut k = self.input.rows.len();
                    if stepper(ui, "Nº de cenários", &mut k, 1, crate::mechanics::MAX_PROBLEM_SIZE) {
                        self.input.set_count(k);
                    }
                    ui.label(RichText::new(crate::studio::QUEUE_HINT).small().color(text_muted()));
                });
                ui.add_space(10.0);
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("q_grid").max_height(540.0).show(ui, |ui| {
                        egui::Grid::new("q_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            for h in HEADERS {
                                head(ui, h);
                            }
                            ui.end_row();
                            for r in self.input.rows.iter_mut() {
                                for cell in r.iter_mut() {
                                    cell_edit(ui, cell, 96.0);
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
            Tab::Results => results_view(ui, "q_res", &self.results),
            Tab::Iterations => {
                if self.details.is_empty() {
                    info_box(ui, "Clique em Resolver para calcular as probabilidades.");
                    return;
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Cenário:").color(text_muted()));
                    for i in 0..self.details.len() {
                        ui.selectable_value(&mut self.selected, i, format!("{}", self.input.rows.get(i).map(|r| r[0].clone()).unwrap_or_default()));
                    }
                });
                ui.add_space(6.0);
                let i = self.selected.min(self.details.len() - 1);
                render_section(ui, &self.details[i], ("q_det", i));
            }
            Tab::Graph => {}
        }
    }

    fn report(&self) -> Option<String> {
        crate::studio::queue::report(&self.input, &self.results, &self.details)
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: QueueInputGrid = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit();
        self.input = inp;
        self.results.clear();
        self.details.clear();
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        *self = Self::new();
    }

    fn load_example(&mut self) {
        self.input = QueueInputGrid::example();
        self.results.clear();
        self.details.clear();
        self.tab = Tab::Data;
    }
}
