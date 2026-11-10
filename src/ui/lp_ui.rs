use super::common::*;
use super::{iterations_view, motion, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::num;
use crate::ip::{BbResult, NodeStatus};
use crate::lp::{graph_data, LpModel};
use crate::studio::lp::{fmt_x, graph_summary, line_in_box, node_status_text, path_text};
pub use crate::studio::lp::{IterMethod, LpInput};
use eframe::egui::{self, Align2, Pos2, RichText, Shape, Stroke, Vec2};

pub struct LpModule {
    pub integer_mode: bool,
    pub input: LpInput,
    tab: Tab,
    results: Vec<Section>,
    iterations: Vec<Section>,
    shown: usize,
    error: Option<String>,
    notice: Option<String>,
    method: IterMethod,
    big_m: String,
    graph_visible: Vec<bool>,
    bb: Option<BbResult>,
    bb_model: Option<LpModel>,
    bb_selected: Option<usize>,
}

impl LpModule {
    pub fn new(integer_mode: bool) -> Self {
        Self {
            integer_mode,
            input: LpInput::blank(2, 2, integer_mode),
            tab: Tab::Data,
            results: vec![],
            iterations: vec![],
            shown: 1,
            error: None,
            notice: None,
            method: IterMethod::Auto,
            big_m: "100".into(),
            graph_visible: vec![],
            bb: None,
            bb_model: None,
            bb_selected: None,
        }
    }

    fn solve(&mut self) {
        self.error = None;
        self.notice = None;
        let model = match self.input.to_model() {
            Ok(m) => m,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        if self.integer_mode {
            self.solve_ip(model);
            return;
        }
        match crate::studio::lp::lp_results(&self.input, &model) {
            Err(e) => self.error = Some(e),
            Ok(sol) => {
                self.notice = sol.notice;
                self.results = sol.results;
                self.build_iterations(&model);
                self.graph_visible = vec![true; model.a.len()];
                self.tab = Tab::Results;
            }
        }
    }

    fn build_iterations(&mut self, model: &LpModel) {
        match crate::studio::lp::lp_iterations(model, self.method, &self.big_m) {
            Err(e) => {
                self.iterations.clear();
                self.error = Some(e);
            }
            Ok(it) => {
                if it.notice.is_some() && self.notice.is_none() {
                    self.notice = it.notice;
                }
                self.iterations = it.sections;
                self.shown = 1;
            }
        }
    }

    fn solve_ip(&mut self, model: LpModel) {
        let out = crate::studio::lp::ip_solve(&model);
        self.iterations = out.iterations;
        self.shown = 1;
        self.results = out.results;
        self.bb = Some(out.bb);
        self.bb_model = Some(model);
        self.bb_selected = None;
        self.tab = Tab::Results;
    }

    fn data_ui(&mut self, ui: &mut egui::Ui) {
        let integer_mode = self.integer_mode;
        card(ui, |ui| {
            title_row(ui, &mut self.input.title);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let mut ch = stepper(ui, "Variáveis", &mut self.input.nvars, 1, crate::mechanics::MAX_PROBLEM_SIZE);
                ui.add_space(20.0);
                ch |= stepper(ui, "Restrições", &mut self.input.ncons, 1, crate::mechanics::MAX_PROBLEM_SIZE);
                if ch {
                    self.input.fit(integer_mode);
                }
            });
            ui.label(RichText::new(crate::studio::LP_HINT).small().color(text_muted()));
        });
        ui.add_space(10.0);
        let nv = self.input.nvars;
        let nc = self.input.ncons;
        card(ui, |ui| {
            egui::ScrollArea::both().id_salt("lp_grid").max_height(560.0).show(ui, |ui| {
                egui::Grid::new("lp_in").spacing([6.0, 6.0]).show(ui, |ui| {
                    head(ui, "");
                    for j in 0..nv {
                        ui.add(egui::TextEdit::singleline(&mut self.input.var_names[j]).desired_width(64.0).horizontal_align(egui::Align::Center).text_color(accent()));
                    }
                    head(ui, "Tipo");
                    head(ui, "LD");
                    ui.end_row();
                    let label = if self.input.maximize { "Maximizar" } else { "Minimizar" };
                    if ui
                        .add(egui::Button::new(RichText::new(label).font(semibold(13.5)).color(on_color(accent()))).fill(accent()).min_size(Vec2::new(110.0, 26.0)))
                        .on_hover_text("Clique para alternar entre Maximizar e Minimizar")
                        .clicked()
                    {
                        self.input.maximize = !self.input.maximize;
                    }
                    for j in 0..nv {
                        cell_edit(ui, &mut self.input.obj[j], 64.0);
                    }
                    ui.label("");
                    ui.label("");
                    ui.end_row();
                    for i in 0..nc {
                        ui.add(egui::TextEdit::singleline(&mut self.input.con_names[i]).desired_width(110.0));
                        for j in 0..nv {
                            cell_edit(ui, &mut self.input.a[i][j], 64.0);
                        }
                        egui::ComboBox::from_id_salt(("rel", i))
                            .width(56.0)
                            .selected_text(self.input.rel[i].clone())
                            .show_ui(ui, |ui| {
                                for r in ["<=", ">=", "="] {
                                    ui.selectable_value(&mut self.input.rel[i], r.to_string(), r);
                                }
                            });
                        cell_edit(ui, &mut self.input.rhs[i], 64.0);
                        ui.end_row();
                    }
                    ui.label(RichText::new("Limite inferior").color(text_muted()));
                    for j in 0..nv {
                        if self.input.unrestricted[j] {
                            ui.label(RichText::new("-∞").color(text_muted()));
                        } else {
                            cell_edit(ui, &mut self.input.lower[j], 64.0);
                        }
                    }
                    ui.end_row();
                    ui.label(RichText::new("Limite superior").color(text_muted()));
                    for j in 0..nv {
                        cell_edit(ui, &mut self.input.upper[j], 64.0);
                    }
                    ui.end_row();
                    ui.label(RichText::new("Irrestrita?").color(text_muted()));
                    for j in 0..nv {
                        ui.checkbox(&mut self.input.unrestricted[j], "");
                    }
                    ui.end_row();
                    if integer_mode {
                        ui.label(RichText::new("Inteira?").color(text_muted()));
                        for j in 0..nv {
                            ui.checkbox(&mut self.input.integer[j], "");
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

    fn iterations_ui(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Método das iterações:").color(text_muted()));
                let mut changed = false;
                for m in IterMethod::ALL {
                    changed |= ui.radio_value(&mut self.method, m, m.label()).changed();
                }
                if self.method == IterMethod::M {
                    ui.label("Valor de M:");
                    changed |= ui.add(egui::TextEdit::singleline(&mut self.big_m).desired_width(60.0)).changed();
                }
                if changed {
                    if let Ok(model) = self.input.to_model() {
                        self.notice = None;
                        self.build_iterations(&model);
                    }
                }
            });
            ui.label(RichText::new("Coluna verde: variável que entra. Linha vermelha: variável que sai. Amarelo: pivô.").small().color(text_muted()));
        });
        ui.add_space(10.0);
        iterations_view(ui, "lp_it", &self.iterations, &mut self.shown);
    }

    fn graph_ui(&mut self, ui: &mut egui::Ui) {
        let model = match self.input.to_model() {
            Ok(m) => m,
            Err(e) => {
                error_box(ui, &e);
                return;
            }
        };
        let g = match graph_data(&model) {
            Ok(g) => g,
            Err(e) => {
                info_box(ui, &e);
                return;
            }
        };
        if self.graph_visible.len() != model.a.len() {
            self.graph_visible = vec![true; model.a.len()];
        }
        card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Restrições no gráfico:").color(text_muted()));
                for i in 0..model.a.len() {
                    ui.checkbox(&mut self.graph_visible[i], RichText::new(format!("({}) {}", i + 1, model.con_names[i])).color(series(i)));
                }
            });
            let summary = graph_summary(&model, &g);
            ui.label(RichText::new(summary).font(semibold(15.0)).color(text_strong()));
        });
        ui.add_space(10.0);
        let h = (ui.available_height() - 10.0).max(380.0);
        let (painter, plot, _) = plot_area(ui, h, 0.0, g.xmax, 0.0, g.ymax, &model.var_names[0], &model.var_names[1]);
        let count = model.a.len();
        let lines_end = 0.42 + 0.09 * count.saturating_sub(1) as f32;
        let sweep_start = lines_end + 0.45;
        let sweep_end = sweep_start + 0.7;
        let elapsed = motion::clock(ui.ctx(), "lp_graph", sweep_end + 0.9);
        if g.region.len() >= 3 {
            let a = motion::ease_out_cubic(motion::phase(elapsed, lines_end, 0.35));
            let pts: Vec<Pos2> = g.region.iter().map(|p| plot.to_screen(p.0, p.1)).collect();
            painter.add(Shape::convex_polygon(pts, accent().gamma_multiply(0.16 * a), Stroke::new(1.5, accent().gamma_multiply(a))));
        }
        for i in 0..count {
            if !self.graph_visible[i] {
                continue;
            }
            let (a, b, r) = (model.a[i][0], model.a[i][1], model.b[i]);
            let col = series(i);
            if let Some((p, q)) = line_in_box(a, b, r, g.xmax, g.ymax) {
                let p1 = plot.to_screen(p.0, p.1);
                let p2 = plot.to_screen(q.0, q.1);
                let t = motion::ease_in_out_cubic(motion::phase(elapsed, i as f32 * 0.09, 0.42));
                partial_segment(&painter, p1, p2, t, Stroke::new(2.0, col));
                if t > 0.0 {
                    painter.text(p1 + Vec2::new(4.0, -4.0), Align2::LEFT_BOTTOM, format!("({})", i + 1), semibold(12.5), col);
                }
            }
        }
        if let Some((x1, x2)) = g.optimum {
            let (c1, c2) = (model.c[0], model.c[1]);
            let z = c1 * x1 + c2 * x2;
            let sweep = motion::ease_in_out_cubic(motion::phase(elapsed, sweep_start, sweep_end - sweep_start));
            if elapsed >= sweep_start {
                if let Some((p, q)) = line_in_box(c1, c2, z * sweep as f64, g.xmax, g.ymax) {
                    let a = plot.to_screen(p.0, p.1);
                    let b = plot.to_screen(q.0, q.1);
                    painter.extend(Shape::dashed_line(&[a, b], Stroke::new(2.0, red()), 8.0, 6.0));
                }
            }
            let o = plot.to_screen(x1, x2);
            let pop = optimum_marker(&painter, o, elapsed, sweep_end, red());
            if pop > 0.5 {
                painter.text(o + Vec2::new(10.0, -8.0), Align2::LEFT_BOTTOM, format!("Ótimo ({}, {})", num(x1), num(x2)), semibold(13.0), highlight_text());
            }
        }
    }

    fn tree_ui(&mut self, ui: &mut egui::Ui) {
        let (Some(bb), Some(model)) = (&self.bb, &self.bb_model) else {
            info_box(ui, "Clique em Resolver para gerar a árvore do Branch & Bound.");
            return;
        };
        card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                legend_dot(ui, green(), "Melhor solução");
                legend_dot(ui, green_soft(), "Inteira");
                legend_dot(ui, accent(), "Ramificado");
                legend_dot(ui, neutral_mark(), "Sondado (z pior)");
                legend_dot(ui, red(), "Inviável");
                ui.label(RichText::new("Clique em um nó para ver a solução.").color(text_muted()));
            });
        });
        ui.add_space(8.0);
        let nodes = &bb.nodes;
        if nodes.len() > 400 {
            info_box(ui, "Árvore grande demais para desenhar. Veja os subproblemas na aba Subproblemas.");
            return;
        }
        let mut order_x = vec![0.0f32; nodes.len()];
        let mut leaf = 0.0f32;
        fn place(id: usize, nodes: &[crate::ip::BbNode], xs: &mut Vec<f32>, leaf: &mut f32) {
            let kids: Vec<usize> = nodes.iter().filter(|n| n.parent == Some(id)).map(|n| n.id).collect();
            if kids.is_empty() {
                xs[id - 1] = *leaf;
                *leaf += 1.0;
            } else {
                for k in &kids {
                    place(*k, nodes, xs, leaf);
                }
                let s: f32 = kids.iter().map(|k| xs[k - 1]).sum();
                xs[id - 1] = s / kids.len() as f32;
            }
        }
        place(1, nodes, &mut order_x, &mut leaf);
        let depth = nodes.iter().map(|n| n.depth).max().unwrap_or(0) as f32;
        let width = (leaf * 110.0).max(ui.available_width());
        let height = (depth + 1.0) * 90.0 + 40.0;
        egui::ScrollArea::both().id_salt("bbtree").show(ui, |ui| {
            let (resp, painter) = ui.allocate_painter(Vec2::new(width, height), egui::Sense::click());
            let r = resp.rect;
            let pos = |i: usize| -> Pos2 {
                let n = &nodes[i];
                Pos2::new(r.left() + 55.0 + order_x[i] * 110.0, r.top() + 34.0 + n.depth as f32 * 90.0)
            };
            let step = (1.3 / nodes.len().max(1) as f32).min(0.08);
            let elapsed = motion::clock(ui.ctx(), "bb_tree", step * nodes.len() as f32 + 0.6);
            for (i, nd) in nodes.iter().enumerate() {
                if let Some(p) = nd.parent {
                    let a = pos(p - 1);
                    let b = pos(i);
                    let grow = motion::ease_out_cubic(motion::phase(elapsed, i as f32 * step - 0.12, 0.22));
                    partial_segment(&painter, a, b, grow, Stroke::new(1.2, edge_line()));
                    if grow >= 1.0 {
                        let galley = painter.layout_no_wrap(nd.branch.clone(), mono(11.5), text_soft());
                        let at = egui::Rect::from_center_size(a.lerp(b, 0.68), galley.size() + Vec2::new(6.0, 2.0));
                        painter.rect_filled(at, 2.0, bg());
                        painter.galley(at.min + Vec2::new(3.0, 1.0), galley, text_soft());
                    }
                }
            }
            for (i, nd) in nodes.iter().enumerate() {
                let pop = motion::phase(elapsed, i as f32 * step, 0.34);
                if pop <= 0.0 {
                    continue;
                }
                let p = pos(i);
                let col = if Some(nd.id) == bb.best {
                    green()
                } else {
                    match nd.status {
                        NodeStatus::Infeasible | NodeStatus::Unbounded => red(),
                        NodeStatus::Fathomed => neutral_mark(),
                        NodeStatus::Integer => green_soft(),
                        NodeStatus::Branched => accent(),
                    }
                };
                painter.circle_filled(p, 20.0 * motion::ease_out_back(pop), col);
                if self.bb_selected == Some(nd.id) {
                    painter.circle_stroke(p, 24.0, Stroke::new(2.0, text_strong()));
                }
                if pop > 0.5 {
                    painter.text(p, Align2::CENTER_CENTER, format!("N{}", nd.id), semibold(12.5), on_color(col));
                    if let Some(z) = nd.z {
                        painter.text(p + Vec2::new(0.0, 24.0), Align2::CENTER_TOP, format!("z={}", num(z)), mono(11.5), text_body());
                    }
                }
            }
            if resp.clicked() {
                if let Some(pp) = resp.interact_pointer_pos() {
                    for i in 0..nodes.len() {
                        if pos(i).distance(pp) < 22.0 {
                            self.bb_selected = Some(nodes[i].id);
                        }
                    }
                }
            }
        });
        if let Some(id) = self.bb_selected {
            let nd = &nodes[id - 1];
            ui.add_space(8.0);
            card(ui, |ui| {
                ui.label(RichText::new(format!("Subproblema N{}", id)).font(semibold(15.0)).color(text_strong()));
                ui.label(format!("Ramos que o definem: {}", path_text(nodes, id)));
                ui.label(format!("Situação: {}", node_status_text(nd.status, nd.branch_var.map(|j| model.var_names[j].as_str()))));
                if let Some(z) = nd.z {
                    ui.label(format!("Valor objetivo: {}", num(z)));
                }
                if !nd.x.is_empty() {
                    ui.label(format!("Solução: {}", fmt_x(&model.var_names, &nd.x)));
                }
            });
        }
    }
}

impl Module for LpModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        if self.integer_mode {
            "integer_programming"
        } else {
            "linear_programming"
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if self.integer_mode {
            tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Graph, "Árvore B&B"), (Tab::Iterations, "Subproblemas")]);
        } else {
            tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Iterations, "Iterações"), (Tab::Graph, "Gráfico")]);
        }
        if let Some(nt) = &self.notice {
            if self.tab != Tab::Data {
                info_box(ui, nt);
            }
        }
        match self.tab {
            Tab::Data => self.data_ui(ui),
            Tab::Results => results_view(ui, "lp_res", &self.results),
            Tab::Iterations => {
                if self.integer_mode {
                    iterations_view(ui, "ip_it", &self.iterations, &mut self.shown)
                } else {
                    self.iterations_ui(ui)
                }
            }
            Tab::Graph => {
                if self.integer_mode {
                    self.tree_ui(ui)
                } else {
                    self.graph_ui(ui)
                }
            }
        }
    }

    fn report(&self) -> Option<String> {
        crate::studio::lp::report(&self.input, self.integer_mode, &self.results, &self.iterations)
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: LpInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit(self.integer_mode);
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.bb = None;
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        self.input = LpInput::blank(2, 2, self.integer_mode);
        self.results.clear();
        self.iterations.clear();
        self.bb = None;
        self.error = None;
        self.notice = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        self.input = LpInput::example(self.integer_mode);
        self.results.clear();
        self.iterations.clear();
        self.bb = None;
        self.error = None;
        self.tab = Tab::Data;
    }
}
