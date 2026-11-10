use super::common::*;
use super::{results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_num, parse_or_zero};
use crate::queue::{analyse, QueueKind, QueueResult};
use eframe::egui::{self, RichText};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct QueueInputGrid {
    pub title: String,
    pub rows: Vec<Vec<String>>,
}

pub struct QueueModule {
    pub input: QueueInputGrid,
    tab: Tab,
    results: Vec<Section>,
    details: Vec<Section>,
    error: Option<String>,
    selected: usize,
}

const HEADERS: [&str; 6] = ["Cenário", "Lambda (λ)", "Mu (μ)", "Servidores c", "Limite do sistema N", "Limite da fonte K"];

impl QueueModule {
    pub fn new() -> Self {
        Self {
            input: QueueInputGrid {
                title: String::new(),
                rows: vec![vec!["1".into(), String::new(), String::new(), "1".into(), "∞".into(), "∞".into()]],
            },
            tab: Tab::Data,
            results: vec![],
            details: vec![],
            error: None,
            selected: 0,
        }
    }

    fn solve(&mut self) {
        self.error = None;
        let mut res: Vec<QueueResult> = Vec::new();
        for (k, r) in self.input.rows.iter().enumerate() {
            let q = crate::queue::QueueInput {
                name: if r[0].trim().is_empty() { format!("{}", k + 1) } else { r[0].trim().to_string() },
                lambda: parse_or_zero(&r[1]),
                mu: parse_or_zero(&r[2]),
                servers: parse_num(&r[3]).map(|v| v.max(1.0) as usize).unwrap_or(1),
                system_limit: crate::mechanics::queue_limit(parse_num(&r[4]).unwrap_or(f64::INFINITY)),
                source_limit: crate::mechanics::queue_limit(parse_num(&r[5]).unwrap_or(f64::INFINITY)),
            };
            match analyse(&q) {
                Ok(x) => res.push(x),
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            }
        }
        let mut comp = Section::new("Análise comparativa").sub(if self.input.title.is_empty() { "Sem título".to_string() } else { self.input.title.clone() });
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
        self.results = vec![comp];
        self.details = res
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
        self.selected = 0;
        self.tab = Tab::Results;
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
                        let old = self.input.rows.len();
                        self.input.rows.resize(k, vec![String::new(), String::new(), String::new(), "1".into(), "∞".into(), "∞".into()]);
                        for i in old..k {
                            self.input.rows[i][0] = format!("{}", i + 1);
                        }
                    }
                    ui.label(RichText::new("Modelo (M/M/c):(GD/N/K). Use ∞ para fila ou fonte infinita. Nascimento puro: μ = 0 e λ = λt. Morte pura: λ = 0, μ = μt e N = tamanho inicial.").small().color(text_muted()));
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
        if self.results.is_empty() {
            return None;
        }
        let mut all = self.results.clone();
        all.extend(self.details.iter().cloned());
        Some(sections_to_text(&format!("ANÁLISE DE FILAS: {}", self.input.title), &all))
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: QueueInputGrid = serde_json::from_value(v).map_err(|e| e.to_string())?;
        for r in inp.rows.iter_mut() {
            r.resize(6, String::new());
        }
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
        let rows = [
            ["1", "6", "8", "1", "∞", "∞"],
            ["2", "6", "8", "2", "∞", "∞"],
            ["3", "6", "8", "1", "5", "∞"],
            ["4", "2", "3", "2", "10", "10"],
        ];
        self.input = QueueInputGrid {
            title: "Comparação de configurações de atendimento".into(),
            rows: rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
        };
        self.results.clear();
        self.details.clear();
        self.tab = Tab::Data;
    }
}
