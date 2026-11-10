use super::common::*;
use super::{iterations_view, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_num};
use crate::project::CpmResult;
pub use crate::studio::project::ProjectInput;
use eframe::egui::{self, Align2, CornerRadius, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2};

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
        Self {
            pert_mode,
            input: ProjectInput::blank(pert_mode),
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

    fn headers(&self) -> Vec<&'static str> {
        crate::studio::project::headers(self.pert_mode)
    }

    fn solve(&mut self) {
        self.error = None;
        match crate::studio::project::solve(&self.input, self.pert_mode) {
            Err(e) => self.error = Some(e),
            Ok(out) => {
                self.results = out.results;
                self.iterations = out.iterations;
                self.path_stats = out.path_stats;
                self.shown = 1;
                self.delay_act = 0;
                self.delay = 0.0;
                self.schedule = Some(out.schedule);
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
            if let Some(msg) = crate::studio::project::delay_message(acts, self.delay_act, self.delay) {
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
                        self.input.set_count(k, self.pert_mode);
                    }
                    ui.label(RichText::new(crate::studio::PROJECT_HINT).small().color(text_muted()));
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
                                let p = crate::studio::project::deadline_probability(mean, sd, t);
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
        crate::studio::project::report(&self.input, self.pert_mode, &self.results, &self.iterations)
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: ProjectInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit(self.pert_mode);
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.schedule = None;
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        self.input = ProjectInput::blank(self.pert_mode);
        self.results.clear();
        self.iterations.clear();
        self.schedule = None;
        self.error = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        self.input = ProjectInput::example(self.pert_mode);
        self.results.clear();
        self.iterations.clear();
        self.schedule = None;
        self.tab = Tab::Data;
    }
}
