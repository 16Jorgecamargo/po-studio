use super::common::*;
use super::{iterations_view, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_or_zero};
use crate::linalg::gauss_jordan;
use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AlgebraInput {
    pub title: String,
    pub n: usize,
    pub a: Vec<Vec<String>>,
    pub b: Vec<String>,
}

impl AlgebraInput {
    fn blank(n: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            n,
            a: vec![],
            b: vec![],
        };
        s.fit();
        s
    }
    fn fit(&mut self) {
        resize_grid(&mut self.a, self.n, self.n);
        resize_vec(&mut self.b, self.n, String::new());
    }
}

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
        let n = self.input.n;
        let a: Vec<Vec<f64>> = self.input.a.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect();
        let b: Vec<f64> = self.input.b.iter().map(|s| parse_or_zero(s)).collect();
        let res = gauss_jordan(&a, if self.with_rhs { Some(&b) } else { None });
        let mut out = Vec::new();
        let mut orig = Section::new(if self.with_rhs { "Equações originais" } else { "Matriz original" });
        let mut h: Vec<String> = (1..=n).map(|j| format!("x{}", j)).collect();
        if self.with_rhs {
            h.push("LD".into());
        }
        orig = orig.headers_owned(std::iter::once(String::new()).chain(h.into_iter()).collect());
        for i in 0..n {
            let mut r = vec![cs(format!("linha {}", i + 1), CellStyle::Head)];
            r.extend(a[i].iter().map(|v| n_(*v)));
            if self.with_rhs {
                r.push(n_(b[i]));
            }
            orig.row(r);
        }
        out.push(orig);
        match &res.inverse {
            None => {
                let mut s = Section::new("Sem inversa");
                s.note("Matriz singular: a inversa não existe (determinante = 0).");
                out.push(s);
            }
            Some(inv) => {
                let mut det = Section::new("Determinante");
                det.line(format!("Valor do determinante = {}", num(res.determinant)));
                out.push(det);
                if let Some(x) = &res.solution {
                    let mut s = Section::new("Solução").headers(&["Variável", "Valor"]);
                    for (i, v) in x.iter().enumerate() {
                        s.row(vec![cs(format!("x{}", i + 1), CellStyle::Head), cs(num(*v), CellStyle::Bold)]);
                    }
                    out.push(s);
                }
                let mut s = Section::new("Matriz inversa")
                    .headers_owned(std::iter::once(String::new()).chain((1..=n).map(|j| format!("col {}", j))).collect());
                for i in 0..n {
                    let mut r = vec![cs(format!("linha {}", i + 1), CellStyle::Head)];
                    r.extend(inv[i].iter().map(|v| n_(*v)));
                    s.row(r);
                }
                out.push(s);
            }
        }
        let mut st = Section::new("Passos da eliminação (Gauss-Jordan com pivoteamento parcial)");
        for l in &res.steps {
            st.line(l.clone());
        }
        self.steps = vec![st];
        self.shown = 1;
        self.results = out;
        self.tab = Tab::Results;
    }
}

fn n_(v: f64) -> Cell {
    n(v)
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
        if self.results.is_empty() {
            return None;
        }
        let t = if self.with_rhs { "SOLUÇÃO DE EQUAÇÕES LINEARES SIMULTÂNEAS" } else { "INVERSÃO DE MATRIZ" };
        Some(sections_to_text(&format!("{}: {}", t, self.input.title), &self.results))
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
        let a = [["2", "1", "-1"], ["-3", "-1", "2"], ["-2", "1", "2"]];
        let b = ["8", "-11", "-3"];
        self.input = AlgebraInput {
            title: if self.with_rhs { "Sistema 3x3 de exemplo".into() } else { "Matriz 3x3 de exemplo".into() },
            n: 3,
            a: a.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
            b: b.iter().map(|s| s.to_string()).collect(),
        };
        self.results.clear();
        self.steps.clear();
        self.tab = Tab::Data;
    }
}
