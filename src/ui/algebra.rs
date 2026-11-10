use super::common::*;
use super::{iterations_view, results_view, solve_bar, tab_bar, title_row, Module, Tab};
pub use crate::studio::algebra::AlgebraInput;
use eframe::egui;

pub struct AlgebraModule {
    pub with_rhs: bool,
    pub input: AlgebraInput,
    tab: Tab,
    results: Vec<Section>,
    steps: Vec<Section>,
    shown: usize,
    error: Option<String>,
}

impl AlgebraModule {
    pub fn new(with_rhs: bool) -> Self {
        Self {
            with_rhs,
            input: AlgebraInput::blank(3),
            tab: Tab::Data,
            results: vec![],
            steps: vec![],
            shown: 1,
            error: None,
        }
    }

    fn solve(&mut self) {
        self.error = None;
        let out = crate::studio::algebra::solve(&self.input, self.with_rhs);
        self.steps = out.steps;
        self.shown = 1;
        self.results = out.results;
        self.tab = Tab::Results;
    }
}

impl Module for AlgebraModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        if self.with_rhs {
            "linear_equations"
        } else {
            "matrix_inverse"
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Iterations, "Passos")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    if stepper(ui, if self.with_rhs { "Nº de equações" } else { "Ordem da matriz" }, &mut self.input.n, 1, crate::mechanics::MAX_PROBLEM_SIZE) {
                        self.input.fit();
                    }
                });
                ui.add_space(10.0);
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("alg_grid").max_height(520.0).show(ui, |ui| {
                        egui::Grid::new("alg_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            ui.label("");
                            for j in 0..self.input.n {
                                head(ui, &format!("x{}", j + 1));
                            }
                            if self.with_rhs {
                                head(ui, "LD");
                            }
                            ui.end_row();
                            for i in 0..self.input.n {
                                head(ui, &format!("{}", i + 1));
                                for j in 0..self.input.n {
                                    cell_edit(ui, &mut self.input.a[i][j], 64.0);
                                }
                                if self.with_rhs {
                                    cell_edit(ui, &mut self.input.b[i], 64.0);
                                }
                                ui.end_row();
                            }
                        });
                    });
                });
                if let Some(e) = &self.error {
                    error_box(ui, e);
                }
                if solve_bar(ui) {
                    self.solve();
                }
            }
            Tab::Results => results_view(ui, "alg_res", &self.results),
            Tab::Iterations => iterations_view(ui, "alg_it", &self.steps, &mut self.shown),
            Tab::Graph => {}
        }
    }

    fn report(&self) -> Option<String> {
        crate::studio::algebra::report(&self.input, self.with_rhs, &self.results)
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: AlgebraInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit();
        self.input = inp;
        self.results.clear();
        self.steps.clear();
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        self.input = AlgebraInput::blank(3);
        self.results.clear();
        self.steps.clear();
        self.error = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        self.input = AlgebraInput::example(self.with_rhs);
        self.results.clear();
        self.steps.clear();
        self.tab = Tab::Data;
    }
}
