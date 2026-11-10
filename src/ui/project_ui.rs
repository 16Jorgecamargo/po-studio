use super::common::*;
use super::{iterations_view, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_num, parse_or_zero};
use crate::project::{cpm, normal_cdf, pert, Activity, CpmResult, PertActivity};
use eframe::egui::{self, Align2, CornerRadius, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectInput {
    pub title: String,
    pub rows: Vec<Vec<String>>,
}

pub struct ProjectModule {
    pub pert_mode: bool,
    pub input: ProjectInput,
    tab: Tab,
    results: Vec<Section>,
    iterations: Vec<Section>,
    shown: usize,
    error: Option<String>,
    schedule: Option<CpmResult>,
    delay_act: usize,
    delay: f64,
    path_stats: Option<(f64, f64)>,
    prob_time: String,
}

impl ProjectModule {
    pub fn new(pert_mode: bool) -> Self {
        let cols = if pert_mode { 6 } else { 4 };
        Self {
            pert_mode,
            input: ProjectInput {
                title: String::new(),
                rows: vec![vec![String::new(); cols]; 5],
            },
            tab: Tab::Data,
            results: vec![],
            iterations: vec![],
            shown: 1,
            error: None,
            schedule: None,
            delay_act: 0,
            delay: 0.0,
            path_stats: None,
            prob_time: String::new(),
        }
    }

    fn cols(&self) -> usize {
        if self.pert_mode {
            6
        } else {
            4
        }
    }

    fn headers(&self) -> Vec<&'static str> {
        if self.pert_mode {
            vec!["Atividade", "Nó de", "Nó para", "Otimista a", "Mais provável m", "Pessimista b"]
        } else {
            vec!["Atividade", "Nó de", "Nó para", "Duração"]
        }
    }

    fn filled_rows(&self) -> Vec<&Vec<String>> {
        self.input.rows.iter().filter(|r| r.iter().skip(1).any(|c| !c.trim().is_empty())).collect()
    }

    fn solve(&mut self) {
        self.error = None;
        let rows = self.filled_rows();
        let node = |s: &str| -> usize { parse_num(s).map(|v| v.max(0.0) as usize).unwrap_or(0) };
        let name = |r: &Vec<String>, k: usize| -> String {
            if r[0].trim().is_empty() {
                format!("A{}", k + 1)
            } else {
                r[0].trim().to_string()
            }
        };
        let title = if self.input.title.is_empty() { "Sem título".to_string() } else { self.input.title.clone() };
        let res = if self.pert_mode {
            let acts: Vec<PertActivity> = rows
                .iter()
                .enumerate()
                .map(|(k, r)| PertActivity {
                    name: name(r, k),
                    from: node(&r[1]),
                    to: node(&r[2]),
                    a: parse_or_zero(&r[3]),
                    m: parse_or_zero(&r[4]),
                    b: parse_or_zero(&r[5]),
                })
                .collect();
            pert(&acts).map(|p| {
                let mut mv = Section::new("Média e variância das atividades").headers(&["Atividade", "a", "m", "b", "Média", "Variância"]);
                for (i, a) in acts.iter().enumerate() {
                    let crit = p.cpm.activities[i].critical;
                    let st = if crit { CellStyle::RedText } else { CellStyle::Head };
                    mv.row(vec![cs(a.name.clone(), st), n(a.a), n(a.m), n(a.b), cs(num(p.means[i]), CellStyle::Bold), n(p.variances[i])]);
                }
                mv.note("Média = (a + 4m + b) / 6. Variância = ((b − a) / 6)²");
                let mut path = Section::new("Caminho mais longo pelas durações médias").sub(title.clone());
                path.line(format!("Caminho: {}", p.path.iter().map(|&i| acts[i].name.clone()).collect::<Vec<_>>().join(" → ")));
                path.line(format!("Média do caminho = {}. Desvio-padrão do caminho = {}", num(p.path_mean), num(p.path_std)));
                self.path_stats = Some((p.path_mean, p.path_std));
                (vec![path, mv], p.cpm)
            })
        } else {
            let acts: Vec<Activity> = rows
                .iter()
                .enumerate()
                .map(|(k, r)| Activity {
                    name: name(r, k),
                    from: node(&r[1]),
                    to: node(&r[2]),
                    duration: parse_or_zero(&r[3]),
                })
                .collect();
            cpm(&acts).map(|c| {
                self.path_stats = None;
                let mut h = Section::new("Cronograma CPM").sub(title.clone());
                h.line(format!("Duração do projeto = {}", num(c.duration)));
                (vec![h], c)
            })
        };
        match res {
            Err(e) => self.error = Some(e),
            Ok((mut secs, c)) => {
                let mut sch = Section::new("Cronograma das atividades").headers(&["Atividade", "De", "Para", "Duração", "Início cedo", "Término cedo", "Início tarde", "Término tarde", "Folga total", "Folga livre"]);
                for a in &c.activities {
                    let st = if a.critical { CellStyle::RedText } else { CellStyle::Head };
                    sch.row(vec![
                        cs(a.name.clone(), st),
                        c_(a.from),
                        c_(a.to),
                        n(a.duration),
                        n(a.es),
                        n(a.ef),
                        n(a.ls),
                        n(a.lf),
                        cs(num(a.total_float), if a.critical { CellStyle::RedText } else { CellStyle::Normal }),
                        n(a.free_float),
                    ]);
                }
                sch.note("Atividades críticas destacadas em vermelho.");
                if self.pert_mode {
                    secs.insert(1, sch);
                } else {
                    secs.push(sch);
                }
                self.results = secs;
                let mut fw = Section::new("Passagem para frente (tempos mais cedo dos nós)");
                for s in &c.forward_steps {
                    fw.line(s.clone());
                }
                fw.note("Passagem para frente concluída.");
                let mut bw = Section::new("Passagem para trás (tempos mais tarde dos nós)");
                for s in &c.backward_steps {
                    bw.line(s.clone());
                }
                bw.note("Passagem para trás concluída.");
                let mut ev = Section::new("Tempos dos eventos").headers(&["Nó", "Mais cedo", "Mais tarde"]);
                for (i, nd) in c.nodes.iter().enumerate() {
                    ev.row(vec![cs(nd.to_string(), CellStyle::Head), n(c.early[i]), n(c.late[i])]);
                }
                self.iterations = vec![fw, bw, ev];
                self.shown = 1;
                self.delay_act = 0;
                self.delay = 0.0;
                self.schedule = Some(c);
                self.tab = Tab::Results;
            }
        }
    }

    fn chart_ui(&mut self, ui: &mut egui::Ui) {
        let Some(c) = &self.schedule else {
            info_box(ui, "Clique em Resolver para gerar o gráfico de barras.");
            return;
        };
        let acts = &c.activities;
        if self.delay_act >= acts.len() {
            self.delay_act = 0;
        }
        card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Experimente mudanças: alterar início de").color(text_muted()));
                egui::ComboBox::from_id_salt("delay_act")
                    .selected_text(acts[self.delay_act].name.clone())
                    .show_ui(ui, |ui| {
                        for (i, a) in acts.iter().enumerate() {
                            if ui.selectable_value(&mut self.delay_act, i, &a.name).changed() {
                                self.delay = 0.0;
                            }
                        }
                    });
                let a = &acts[self.delay_act];
                if a.critical {
                    ui.label(RichText::new("Atividade crítica: seu cronograma não pode ser alterado.").color(red()));
                } else {
                    ui.label("atraso:");
                    ui.add(egui::Slider::new(&mut self.delay, 0.0..=a.total_float));
                    ui.label(RichText::new("(máximo = folga total)").small().color(text_muted()));
                }
            });
            let a = &acts[self.delay_act];
            if !a.critical && self.delay > 0.0 {
                let start = a.es + self.delay;
                let mut msg = format!(
                    "ATIVIDADE {}: INÍCIO = {}, ATRASO em relação ao início cedo = {}, FOLGA LIVRE = {}.",
                    a.name,
                    num(start),
                    num(self.delay),
                    num(a.free_float)
                );
                if self.delay > a.free_float + 1e-9 {
                    let succ: Vec<String> = acts.iter().filter(|b| b.from == a.to).map(|b| b.name.clone()).collect();
                    msg.push_str(&format!(
                        " Como o ATRASO excede a FOLGA LIVRE em {}, as atividades seguintes {{{}}} não podem começar antes do tempo {}.",
                        num(self.delay - a.free_float),
                        succ.join(", "),
                        num(start + a.duration)
                    ));
                }
                ui.label(RichText::new(msg).color(warn_text()));
            }
        });
        ui.add_space(10.0);
        let total = c.duration.max(1.0);
        let row_h = 30.0;
        let height = acts.len() as f32 * row_h + 60.0;
        egui::ScrollArea::vertical().id_salt("gantt").show(ui, |ui| {
            let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), height), egui::Sense::hover());
            let r = resp.rect;
            painter.rect_filled(r, CornerRadius::same(4), card_fill());
            painter.rect_stroke(r, CornerRadius::same(4), Stroke::new(1.0, border()), StrokeKind::Inside);
            let elapsed = super::motion::clock(ui.ctx(), "gantt", 1.2);
            let left = r.left() + 110.0;
            let right = r.right() - 20.0;
            let top = r.top() + 30.0;
            let x = |t: f64| left + (t / total) as f32 * (right - left);
            let steps = 10;
            for k in 0..=steps {
                let t = total * k as f64 / steps as f64;
                let px = x(t);
                painter.line_segment([Pos2::new(px, top - 6.0), Pos2::new(px, r.bottom() - 10.0)], Stroke::new(1.0, grid_line()));
                painter.text(Pos2::new(px, r.top() + 8.0), Align2::CENTER_TOP, num(t), mono(11.5), text_muted());
            }
            for (i, a) in acts.iter().enumerate() {
                let y = top + i as f32 * row_h;
                let grow = super::motion::ease_out_cubic(super::motion::phase(elapsed, (i as f32 * 0.04).min(0.5), 0.45));
                painter.text(Pos2::new(r.left() + 12.0, y + row_h / 2.0), Align2::LEFT_CENTER, &a.name, if a.critical { semibold(13.0) } else { sans(13.0) }, if a.critical { red() } else { text_body() });
                let shift = if i == self.delay_act { self.delay } else { 0.0 };
                if !a.critical {
                    let fr = Rect::from_min_max(Pos2::new(x(a.es), y + 9.0), Pos2::new(x(a.lf), y + row_h - 9.0));
                    painter.rect_filled(fr, CornerRadius::same(2), band_fill().gamma_multiply(grow));
                }
                let bar_left = x(a.es + shift);
                let bar_right = bar_left + ((x(a.ef + shift) - bar_left) * grow).max(2.0);
                let br = Rect::from_min_max(Pos2::new(bar_left, y + 5.0), Pos2::new(bar_right, y + row_h - 5.0));
                painter.rect_filled(br, CornerRadius::same(2), if a.critical { red() } else { accent() });
            }
        });
        ui.label(RichText::new("Vermelho: atividade crítica. Verde: não crítica. Faixa clara: intervalo em que pode ser realizada (folga total).").small().color(text_muted()));
    }
}

fn c_(v: usize) -> Cell {
    c(v.to_string())
}

impl Module for ProjectModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        if self.pert_mode {
            "pert"
        } else {
            "cpm"
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Iterations, "Cálculos"), (Tab::Graph, "Gráfico de barras")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    let mut k = self.input.rows.len();
                    if stepper(ui, "Nº de atividades", &mut k, 1, crate::mechanics::MAX_PROBLEM_SIZE) {
                        let cols = self.cols();
                        self.input.rows.resize(k, vec![String::new(); cols]);
                    }
                    ui.label(RichText::new("Cada atividade liga um nó (evento) a outro. Numere os nós a partir de 1.").small().color(text_muted()));
                });
                ui.add_space(10.0);
                let headers = self.headers();
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("pj_grid").max_height(540.0).show(ui, |ui| {
                        egui::Grid::new("pj_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            for h in &headers {
                                head(ui, h);
                            }
                            ui.end_row();
                            for r in self.input.rows.iter_mut() {
                                for (k, cell) in r.iter_mut().enumerate() {
                                    cell_edit(ui, cell, if k == 0 { 110.0 } else { 80.0 });
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
            Tab::Results => {
                results_view(ui, "pj_res", &self.results);
                if let Some((mean, sd)) = self.path_stats {
                    card(ui, |ui| {
                        ui.label(RichText::new("Probabilidade de concluir o projeto até um prazo").font(semibold(15.0)).color(text_strong()));
                        ui.horizontal(|ui| {
                            ui.label("Prazo T:");
                            ui.add(egui::TextEdit::singleline(&mut self.prob_time).desired_width(80.0));
                            if let Some(t) = parse_num(&self.prob_time) {
                                let p = if sd > 0.0 { normal_cdf((t - mean) / sd) } else if t >= mean { 1.0 } else { 0.0 };
                                ui.label(RichText::new(format!("P(duração ≤ {}) = {}", num(t), num(p))).font(mono_bold(14.0)).color(accent()));
                            }
                        });
                        ui.label(RichText::new("Aproximação normal com a média e o desvio-padrão do caminho mais longo.").small().color(text_muted()));
                    });
                }
            }
            Tab::Iterations => iterations_view(ui, "pj_it", &self.iterations, &mut self.shown),
            Tab::Graph => self.chart_ui(ui),
        }
    }

    fn report(&self) -> Option<String> {
        if self.results.is_empty() {
            return None;
        }
        let mut all = self.results.clone();
        all.extend(self.iterations.iter().cloned());
        let t = if self.pert_mode { "PLANEJAMENTO DE PROJETOS: PERT" } else { "PLANEJAMENTO DE PROJETOS: CPM" };
        Some(sections_to_text(&format!("{}: {}", t, self.input.title), &all))
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: ProjectInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        let cols = self.cols();
        for r in inp.rows.iter_mut() {
            r.resize(cols, String::new());
        }
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.schedule = None;
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        let cols = self.cols();
        self.input = ProjectInput {
            title: String::new(),
            rows: vec![vec![String::new(); cols]; 5],
        };
        self.results.clear();
        self.iterations.clear();
        self.schedule = None;
        self.error = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        let rows: Vec<Vec<&str>> = if self.pert_mode {
            vec![
                vec!["A", "1", "2", "2", "4", "6"],
                vec!["B", "1", "3", "3", "5", "9"],
                vec!["C", "2", "4", "1", "2", "3"],
                vec!["D", "3", "4", "4", "6", "10"],
                vec!["E", "2", "5", "5", "7", "9"],
                vec!["F", "4", "5", "2", "3", "4"],
                vec!["G", "5", "6", "1", "2", "5"],
            ]
        } else {
            vec![
                vec!["A", "1", "2", "4"],
                vec!["B", "1", "3", "6"],
                vec!["C", "2", "4", "3"],
                vec!["D", "3", "4", "5"],
                vec!["E", "2", "5", "8"],
                vec!["F", "4", "5", "2"],
                vec!["G", "5", "6", "3"],
            ]
        };
        self.input = ProjectInput {
            title: "Projeto de exemplo".into(),
            rows: rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
        };
        self.results.clear();
        self.iterations.clear();
        self.schedule = None;
        self.tab = Tab::Data;
    }
}
