use super::common::*;
use super::{iterations_view, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_or_zero};
use crate::transport::{solve, StartMethod, TransportProblem};
use eframe::egui::{self, RichText};
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
    fn blank(ns: usize, nd: usize) -> Self {
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
    fn fit(&mut self) {
        resize_names(&mut self.sources, self.ns, "Origem ");
        resize_names(&mut self.dests, self.nd, "Destino ");
        resize_grid(&mut self.cost, self.ns, self.nd);
        resize_vec(&mut self.supply, self.ns, String::new());
        resize_vec(&mut self.demand, self.nd, String::new());
    }
}

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
        let p = TransportProblem {
            sources: self.input.sources.clone(),
            dests: self.input.dests.clone(),
            cost: self.input.cost.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect(),
            supply: self.input.supply.iter().map(|s| parse_or_zero(s)).collect(),
            demand: self.input.demand.iter().map(|s| parse_or_zero(s)).collect(),
        };
        let sol = match solve(&p, self.method) {
            Ok(s) => s,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let q = &sol.problem;
        let last = sol.tableaus.last().unwrap();
        let mut head = Section::new("Resumo da solução: Modelo de Transporte").sub(format!(
            "{}. Solução inicial: {}. Iterações: {}",
            if self.input.title.is_empty() { "Sem título" } else { self.input.title.as_str() },
            self.method.label(),
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
        self.results = vec![head, ship];
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
        self.iterations = its;
        self.shown = 1;
        self.tab = Tab::Results;
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
        if self.results.is_empty() {
            return None;
        }
        let mut all = self.results.clone();
        all.extend(self.iterations.iter().cloned());
        Some(sections_to_text(&format!("MODELO DE TRANSPORTE: {}", self.input.title), &all))
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
        let cost = [["8", "6", "10", "9"], ["9", "12", "13", "7"], ["14", "9", "16", "5"]];
        let mut inp = TransportInput::blank(3, 4);
        inp.title = "Exemplo: três fábricas, quatro lojas".into();
        inp.cost = cost.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect();
        inp.supply = vec!["35".into(), "50".into(), "40".into()];
        inp.demand = vec!["45".into(), "20".into(), "30".into(), "30".into()];
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.tab = Tab::Data;
    }
}
