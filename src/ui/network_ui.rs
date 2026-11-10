use super::common::*;
use super::{iterations_view, motion, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::num;
pub use crate::studio::network::{NetInput, NetKind, ShortAlgo};
use eframe::egui::{self, Align2, Pos2, RichText, Stroke, StrokeKind, Vec2};

pub struct NetworkModule {
    pub kind: NetKind,
    pub input: NetInput,
    tab: Tab,
    results: Vec<Section>,
    iterations: Vec<Section>,
    shown: usize,
    error: Option<String>,
    start: usize,
    target: usize,
    algo: ShortAlgo,
    highlight: Vec<(usize, usize, String)>,
}

impl NetworkModule {
    pub fn new(kind: NetKind) -> Self {
        Self {
            kind,
            input: NetInput::blank(5),
            tab: Tab::Data,
            results: vec![],
            iterations: vec![],
            shown: 1,
            error: None,
            start: 0,
            target: 4,
            algo: ShortAlgo::Dijkstra,
            highlight: vec![],
        }
    }

    fn matrix(&self) -> Vec<Vec<f64>> {
        self.input.distances()
    }

    fn solve(&mut self) {
        self.error = None;
        self.highlight.clear();
        let n = self.input.n;
        self.start = self.start.min(n - 1);
        self.target = self.target.min(n - 1);
        match crate::studio::network::solve(&self.input, self.kind, self.algo, self.start, self.target) {
            Err(e) => self.error = Some(e),
            Ok(out) => {
                self.results = out.results;
                self.iterations = out.iterations;
                self.highlight = out.highlight;
                self.start = out.start;
                self.target = out.target;
                self.shown = 1;
                self.tab = Tab::Results;
            }
        }
    }

    fn draw_network(&self, ui: &mut egui::Ui) {
        let d = self.matrix();
        let n = self.input.n;
        let h = (ui.available_height() - 10.0).max(360.0);
        let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), h), egui::Sense::hover());
        let r = resp.rect;
        painter.rect_filled(r, 4.0, card_fill());
        painter.rect_stroke(r, 4.0, Stroke::new(1.0, border()), StrokeKind::Inside);
        let elapsed = motion::clock(ui.ctx(), ("net_graph", self.key()), 0.5 + self.highlight.len() as f32 * 0.22 + 0.4);
        let center = r.center();
        let rad = (r.width().min(r.height()) / 2.0 - 50.0).max(60.0);
        let pos: Vec<Pos2> = (0..n)
            .map(|i| {
                let a = std::f32::consts::TAU * i as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
                center + Vec2::new(a.cos() * rad, a.sin() * rad)
            })
            .collect();
        let directed = self.kind != NetKind::Mst;
        for i in 0..n {
            for j in 0..n {
                if i == j || !d[i][j].is_finite() {
                    continue;
                }
                if !directed && j < i && d[j][i].is_finite() {
                    continue;
                }
                let found = self.highlight.iter().position(|(a, b, _)| (*a == i && *b == j) || (!directed && *a == j && *b == i));
                let lit = found.map(|k| motion::ease_in_out_cubic(motion::phase(elapsed, 0.45 + k as f32 * 0.22, 0.3))).unwrap_or(0.0);
                let done = lit >= 1.0;
                let reversed = found.is_some_and(|k| self.highlight[k].0 == j);
                let a = pos[i];
                let b = pos[j];
                let dir = (b - a).normalized();
                let bidir = directed && d[j][i].is_finite();
                let off = if bidir { Vec2::new(-dir.y, dir.x) * 5.0 } else { Vec2::ZERO };
                let a2 = a + dir * 20.0 + off;
                let b2 = b - dir * 22.0 + off;
                painter.line_segment([a2, b2], Stroke::new(1.3, edge_line()));
                if reversed {
                    partial_segment(&painter, b2, a2, lit, Stroke::new(3.0, red()));
                } else {
                    partial_segment(&painter, a2, b2, lit, Stroke::new(3.0, red()));
                }
                if directed {
                    let (col, w) = if done { (red(), 3.0) } else { (edge_line(), 1.3) };
                    let normal = Vec2::new(-dir.y, dir.x);
                    let size = 6.0 + w;
                    let base = b2 - dir * size * 1.6;
                    painter.add(egui::Shape::convex_polygon(
                        vec![b2, base + normal * size * 0.6, base - normal * size * 0.6],
                        col,
                        Stroke::NONE,
                    ));
                }
                let label = match found {
                    Some(k) if done && self.kind == NetKind::MaxFlow => format!("{}/{}", self.highlight[k].2, num(d[i][j])),
                    _ => num(d[i][j]),
                };
                let text_color = if done { highlight_text() } else { text_soft() };
                let font = if done { mono_bold(12.0) } else { mono(12.0) };
                let galley = painter.layout_no_wrap(label, font, text_color);
                let at = a2.lerp(b2, 0.5) + off * 2.0;
                let rect = egui::Rect::from_center_size(at, galley.size() + Vec2::new(6.0, 2.0));
                painter.rect_filled(rect, 2.0, label_backing());
                painter.galley(rect.min + Vec2::new(3.0, 1.0), galley, text_color);
            }
        }
        for i in 0..n {
            let special = (self.kind != NetKind::Mst && (i == self.start || i == self.target)) || (self.kind == NetKind::Mst && i == self.start);
            let fill = if special { accent() } else { node_fill() };
            let pop = motion::phase(elapsed, i as f32 * 0.04, 0.34);
            painter.circle_filled(pos[i], 18.0 * motion::ease_out_back(pop), fill);
            if pop > 0.5 {
                painter.text(pos[i], Align2::CENTER_CENTER, &self.input.names[i], semibold(12.5), on_color(fill));
            }
        }
    }

    fn node_combo(ui: &mut egui::Ui, id: &str, label: &str, value: &mut usize, names: &[String]) {
        ui.label(RichText::new(label).color(text_muted()));
        egui::ComboBox::from_id_salt(id)
            .selected_text(names.get(*value).cloned().unwrap_or_default())
            .show_ui(ui, |ui| {
                for (i, nm) in names.iter().enumerate() {
                    ui.selectable_value(value, i, nm);
                }
            });
    }
}

impl Module for NetworkModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        self.kind.key()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Iterations, "Iterações"), (Tab::Graph, "Rede")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    if stepper(ui, "Nº de nós", &mut self.input.n, 2, crate::mechanics::MAX_PROBLEM_SIZE) {
                        self.input.fit();
                    }
                    let names = self.input.names.clone();
                    self.start = self.start.min(self.input.n - 1);
                    self.target = self.target.min(self.input.n - 1);
                    ui.horizontal(|ui| match self.kind {
                        NetKind::Mst => Self::node_combo(ui, "mst_start", "Nó inicial:", &mut self.start, &names),
                        NetKind::Shortest => {
                            ui.radio_value(&mut self.algo, ShortAlgo::Dijkstra, "Dijkstra");
                            ui.radio_value(&mut self.algo, ShortAlgo::Floyd, "Floyd (todas as rotas)");
                            ui.add_space(16.0);
                            Self::node_combo(ui, "sr_from", "De:", &mut self.start, &names);
                            Self::node_combo(ui, "sr_to", "Para:", &mut self.target, &names);
                        }
                        NetKind::MaxFlow => {
                            Self::node_combo(ui, "mf_from", "Origem:", &mut self.start, &names);
                            Self::node_combo(ui, "mf_to", "Destino:", &mut self.target, &names);
                        }
                    });
                    let hint = self.kind.hint();
                    ui.label(RichText::new(hint).small().color(text_muted()));
                });
                ui.add_space(10.0);
                let n = self.input.n;
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("net_grid").max_height(540.0).show(ui, |ui| {
                        egui::Grid::new("net_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            head(ui, "De \\ Para");
                            for j in 0..n {
                                head(ui, &self.input.names[j]);
                            }
                            ui.end_row();
                            for i in 0..n {
                                ui.add(egui::TextEdit::singleline(&mut self.input.names[i]).desired_width(70.0).text_color(accent()));
                                for j in 0..n {
                                    if i == j {
                                        ui.label(RichText::new("–").color(text_muted()));
                                    } else {
                                        cell_edit(ui, &mut self.input.matrix[i][j], 56.0);
                                    }
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
            Tab::Results => results_view(ui, "net_res", &self.results),
            Tab::Iterations => iterations_view(ui, "net_it", &self.iterations, &mut self.shown),
            Tab::Graph => {
                if self.highlight.is_empty() {
                    ui.label(RichText::new("Rede informada (resolva para destacar a solução em vermelho).").color(text_muted()));
                } else {
                    ui.label(RichText::new("Arcos da solução em vermelho.").color(text_muted()));
                }
                self.draw_network(ui);
            }
        }
    }

    fn report(&self) -> Option<String> {
        crate::studio::network::report(&self.input, self.kind, &self.results, &self.iterations)
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: NetInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit();
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.highlight.clear();
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        self.input = NetInput::blank(5);
        self.results.clear();
        self.iterations.clear();
        self.highlight.clear();
        self.error = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        let (inp, start, target) = NetInput::example(self.kind);
        self.input = inp;
        self.start = start;
        self.target = target;
        self.results.clear();
        self.iterations.clear();
        self.highlight.clear();
        self.tab = Tab::Data;
    }
}
