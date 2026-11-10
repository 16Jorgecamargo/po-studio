use super::common::*;
use super::{iterations_view, motion, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_num, parse_or_zero};
use crate::ip::{branch_and_bound, BbResult, NodeStatus};
use crate::lp::{self, graph_data, LpModel, LpReport};
use crate::simplex::{ColKind, Method, Rel, Sense, SimplexRun, Status, Tableau};
use eframe::egui::{self, Align2, Pos2, RichText, Shape, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LpInput {
    pub title: String,
    pub maximize: bool,
    pub nvars: usize,
    pub ncons: usize,
    pub var_names: Vec<String>,
    pub obj: Vec<String>,
    pub con_names: Vec<String>,
    pub a: Vec<Vec<String>>,
    pub rel: Vec<String>,
    pub rhs: Vec<String>,
    pub lower: Vec<String>,
    pub upper: Vec<String>,
    pub unrestricted: Vec<bool>,
    pub integer: Vec<bool>,
}

impl LpInput {
    fn blank(nv: usize, nc: usize, integer: bool) -> Self {
        let mut s = Self {
            title: String::new(),
            maximize: true,
            nvars: nv,
            ncons: nc,
            var_names: vec![],
            obj: vec![],
            con_names: vec![],
            a: vec![],
            rel: vec![],
            rhs: vec![],
            lower: vec![],
            upper: vec![],
            unrestricted: vec![],
            integer: vec![],
        };
        s.fit(integer);
        s
    }

    fn fit(&mut self, integer_default: bool) {
        resize_names(&mut self.var_names, self.nvars, "x");
        resize_vec(&mut self.obj, self.nvars, String::new());
        resize_names(&mut self.con_names, self.ncons, "Restr ");
        resize_grid(&mut self.a, self.ncons, self.nvars);
        resize_vec(&mut self.rel, self.ncons, "<=".to_string());
        resize_vec(&mut self.rhs, self.ncons, String::new());
        resize_vec(&mut self.lower, self.nvars, "0".to_string());
        resize_vec(&mut self.upper, self.nvars, "∞".to_string());
        resize_vec(&mut self.unrestricted, self.nvars, false);
        resize_vec(&mut self.integer, self.nvars, integer_default);
    }

    pub fn to_model(&self) -> Result<LpModel, String> {
        let n = self.nvars;
        let mut lower = Vec::new();
        let mut upper = Vec::new();
        for j in 0..n {
            let lo = if self.unrestricted[j] {
                f64::NEG_INFINITY
            } else {
                crate::mechanics::bound_value(parse_num(&self.lower[j]).unwrap_or(0.0))
            };
            let up = crate::mechanics::bound_value(parse_num(&self.upper[j]).unwrap_or(f64::INFINITY));
            if !self.unrestricted[j] && lo < 0.0 && lo.is_infinite() {
                return Err("Variável não negativa: o limite inferior deve ser finito.".into());
            }
            lower.push(lo);
            upper.push(up);
        }
        Ok(LpModel {
            sense: if self.maximize { Sense::Max } else { Sense::Min },
            var_names: self.var_names.clone(),
            c: self.obj.iter().map(|s| parse_or_zero(s)).collect(),
            a: self.a.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect(),
            rel: self.rel.iter().map(|s| Rel::parse(s)).collect(),
            b: self.rhs.iter().map(|s| parse_or_zero(s)).collect(),
            con_names: self.con_names.clone(),
            lower,
            upper,
            integer: self.integer.clone(),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IterMethod {
    Auto,
    M,
    TwoPhase,
    Dual,
}

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
        match lp::solve(&model, None, 100.0) {
            Err(e) => self.error = Some(e),
            Ok(sol) => {
                self.notice = sol.notice.clone();
                self.results = summary_sections(&self.input, &model, &sol.report);
                self.build_iterations(&model);
                self.graph_visible = vec![true; model.a.len()];
                self.tab = Tab::Results;
            }
        }
    }

    fn build_iterations(&mut self, model: &LpModel) {
        let big_m = parse_num(&self.big_m).unwrap_or(crate::mechanics::BIG_M_DEFAULT);
        let method = match self.method {
            IterMethod::Auto => None,
            IterMethod::M => Some(Method::MMethod),
            IterMethod::TwoPhase => Some(Method::TwoPhase),
            IterMethod::Dual => Some(Method::DualSimplex),
        };
        match lp::prepare(model, method, big_m) {
            Err(e) => {
                self.iterations.clear();
                self.error = Some(e);
            }
            Ok((mut run, _tr, notice)) => {
                run.run_all();
                if notice.is_some() && self.notice.is_none() {
                    self.notice = notice.clone();
                }
                let mut secs: Vec<Section> = Vec::new();
                if let Some(nt) = notice {
                    let mut s = Section::new("Aviso");
                    s.note(nt);
                    secs.push(s);
                }
                secs.extend(tableau_sections(&run, model.sense));
                self.iterations = secs;
                self.shown = 1;
            }
        }
    }

    fn solve_ip(&mut self, model: LpModel) {
        let res = branch_and_bound(&model);
        let mut out = Vec::new();
        let mut s = Section::new("Programação inteira: resumo do Branch & Bound");
        match res.best {
            Some(id) => {
                let node = &res.nodes[id - 1];
                s.line(format!(
                    "Solução ótima encontrada no subproblema {}, valor objetivo = {}",
                    id,
                    num(node.z.unwrap_or(0.0))
                ));
                if let Some(last) = res.nodes.last() {
                    s.line(format!("Resultado verificado na iteração {}", last.id));
                }
                out.push(s);
                let mut v = Section::new("Solução ótima").headers(&["Variável", "Valor", "Coef. obj.", "Contribuição"]);
                for j in 0..model.c.len() {
                    v.row(vec![
                        cs(model.var_names[j].clone(), CellStyle::Head),
                        cs(num(node.x[j]), CellStyle::Bold),
                        n(model.c[j]),
                        n(model.c[j] * node.x[j]),
                    ]);
                }
                out.push(v);
            }
            None => {
                s.line(if res.nodes.iter().any(|n| n.status == NodeStatus::Unbounded) {
                    "A solução é ilimitada.".to_string()
                } else {
                    "Nenhuma solução viável encontrada.".to_string()
                });
                out.push(s);
            }
        }
        if res.aborted {
            let mut a = Section::new("Busca excessiva");
            a.note("O número de subproblemas examinados excedeu 32.000. Execução interrompida.");
            out.push(a);
        }
        if !res.improvements.is_empty() {
            let mut f = Section::new("Soluções viáveis (em ordem de melhoria)").headers(&["Subproblema", "Valor objetivo", "Solução"]);
            for &id in &res.improvements {
                let nd = &res.nodes[id - 1];
                f.row(vec![
                    c(format!("N{}", id)),
                    n(nd.z.unwrap_or(0.0)),
                    c(fmt_x(&model.var_names, &nd.x)),
                ]);
            }
            out.push(f);
        }
        let mut it = Vec::new();
        for nd in &res.nodes {
            let mut s = Section::new(format!("Subproblema N{}", nd.id)).sub(match nd.parent {
                Some(p) => format!("Ramo a partir de N{}: {}", p, nd.branch),
                None => "LP contínuo (sem restrições de integralidade)".into(),
            });
            let status = node_status_text(nd.status, nd.branch_var.map(|j| model.var_names[j].as_str()));
            s.line(format!("Situação: {}", status));
            if let Some(z) = nd.z {
                s.line(format!("Valor objetivo z = {}", num(z)));
            }
            if !nd.x.is_empty() {
                s.line(format!("Solução: {}", fmt_x(&model.var_names, &nd.x)));
            }
            it.push(s);
        }
        self.iterations = it;
        self.shown = 1;
        self.results = out;
        self.bb = Some(res);
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
            ui.label(RichText::new("Tamanho máximo: 100 (como no original). Deixe em branco para zero; use ∞ ou inf para infinito.").small().color(text_muted()));
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
                for (m, l) in [
                    (IterMethod::Auto, "Automático"),
                    (IterMethod::M, "M grande"),
                    (IterMethod::TwoPhase, "Duas fases"),
                    (IterMethod::Dual, "Dual simplex"),
                ] {
                    changed |= ui.radio_value(&mut self.method, m, l).changed();
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
            let summary = match (g.status, g.optimum) {
                (Status::Optimal, Some((x1, x2))) => format!(
                    "Ótimo: {} = {}, {} = {}, z = {}",
                    model.var_names[0],
                    num(x1),
                    model.var_names[1],
                    num(x2),
                    num(g.z)
                ),
                (Status::Unbounded, _) => "A solução ótima é ILIMITADA.".into(),
                _ => "O problema não tem solução viável.".into(),
            };
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

fn path_text(nodes: &[crate::ip::BbNode], id: usize) -> String {
    let mut parts = Vec::new();
    let mut cur = Some(id);
    while let Some(i) = cur {
        let nd = &nodes[i - 1];
        if nd.parent.is_some() {
            parts.push(nd.branch.clone());
        }
        cur = nd.parent;
    }
    parts.reverse();
    if parts.is_empty() {
        "nenhum (LP contínuo)".into()
    } else {
        parts.join(", ")
    }
}

fn node_status_text(s: NodeStatus, var: Option<&str>) -> String {
    match s {
        NodeStatus::Infeasible => "solução inviável, sondado".into(),
        NodeStatus::Unbounded => "ilimitado".into(),
        NodeStatus::Integer => "solução inteira".into(),
        NodeStatus::Fathomed => "sondado (valor não melhora a melhor solução)".into(),
        NodeStatus::Branched => format!("ramificado na variável {}", var.unwrap_or("?")),
    }
}

fn fmt_x(names: &[String], x: &[f64]) -> String {
    names
        .iter()
        .zip(x.iter())
        .map(|(n, v)| format!("{} = {}", n, num(*v)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn line_in_box(a: f64, b: f64, r: f64, xmax: f64, ymax: f64) -> Option<((f64, f64), (f64, f64))> {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut push = |p: (f64, f64)| {
        if p.0 >= -1e-9 && p.0 <= xmax + 1e-9 && p.1 >= -1e-9 && p.1 <= ymax + 1e-9 {
            if !pts.iter().any(|q| (q.0 - p.0).abs() < 1e-9 && (q.1 - p.1).abs() < 1e-9) {
                pts.push(p);
            }
        }
    };
    if b.abs() > 1e-12 {
        push((0.0, r / b));
        push((xmax, (r - a * xmax) / b));
    }
    if a.abs() > 1e-12 {
        push((r / a, 0.0));
        push(((r - b * ymax) / a, ymax));
    }
    if pts.len() >= 2 {
        Some((pts[0], pts[1]))
    } else {
        None
    }
}

fn summary_sections(input: &LpInput, model: &LpModel, rep: &LpReport) -> Vec<Section> {
    let mut out = Vec::new();
    let mut head = Section::new("Resumo da solução: Programação Linear").sub(format!(
        "{}. {}. Iterações: {}",
        if input.title.is_empty() { "Sem título" } else { input.title.as_str() },
        rep.method.label(),
        rep.iterations
    ));
    match rep.status {
        Status::Optimal => head.line(format!(
            "Valor objetivo ({}) = {}",
            if model.sense == Sense::Max { "Max" } else { "Min" },
            num(rep.z)
        )),
        Status::Unbounded => head.line("A solução ótima é ilimitada.".to_string()),
        Status::Infeasible => head.line("O problema não tem solução viável.".to_string()),
    }
    if rep.alternative {
        head.note("Há soluções ótimas alternativas (veja a aba Iterações: variável não básica com zj-cj = 0 no último quadro).");
    }
    out.push(head);
    if rep.status == Status::Unbounded || rep.status == Status::Infeasible {
        return out;
    }
    let mut v = Section::new("Variáveis").headers(&["Variável", "Valor", "Coef. obj.", "Contribuição"]);
    for r in &rep.vars {
        v.row(vec![cs(r.name.clone(), CellStyle::Head), cs(num(r.value), CellStyle::Bold), n(r.c), n(r.contribution)]);
    }
    out.push(v);
    let mut cns = Section::new("Restrições").headers(&["Restrição", "LE", "", "LD", "Folga(-)/Excesso(+)"]);
    for r in &rep.cons {
        let sl = match r.rel {
            Rel::Le => format!("{}-", num(r.slack)),
            Rel::Ge => format!("{}+", num(r.slack)),
            Rel::Eq => num(0.0),
        };
        cns.row(vec![cs(r.name.clone(), CellStyle::Head), n(r.lhs), c(r.rel.symbol()), n(r.rhs), c(sl)]);
    }
    out.push(cns);
    let mut s1 = Section::new("Análise de sensibilidade: coeficientes do objetivo").headers(&["Variável", "Coef. atual", "Coef. mín.", "Coef. máx.", "Custo reduzido"]);
    for r in &rep.vars {
        s1.row(vec![
            cs(r.name.clone(), CellStyle::Head),
            n(r.c),
            r.cmin.map(n).unwrap_or_else(|| c("–")),
            r.cmax.map(n).unwrap_or_else(|| c("–")),
            r.reduced.map(|v| n(v.abs())).unwrap_or_else(|| c("–")),
        ]);
    }
    out.push(s1);
    let mut s2 = Section::new("Análise de sensibilidade: lado direito").headers(&["Restrição", "Preço dual", "LD mín.", "LD atual", "LD máx."]);
    for r in &rep.cons {
        s2.row(vec![cs(r.name.clone(), CellStyle::Head), n(r.dual), n(r.rmin), n(r.rhs), n(r.rmax)]);
    }
    s2.note("Preço dual = variação do valor objetivo por unidade de aumento no lado direito, dentro do intervalo [mín., máx.].");
    out.push(s2);
    out
}

fn tableau_sections(run: &SimplexRun, sense: Sense) -> Vec<Section> {
    let mut out = Vec::new();
    for (k, t) in run.history.iter().enumerate() {
        out.push(tableau_section(t, k, run.method, sense));
    }
    out
}

fn tableau_section(t: &Tableau, k: usize, method: Method, _sense: Sense) -> Section {
    let phase = match t.phase {
        1 => ": Fase 1".to_string(),
        2 => ": Fase 2".to_string(),
        _ => String::new(),
    };
    let mut s = Section::new(format!("Iteração {}{}", k + 1, phase));
    let mut sub = method.label().to_string();
    if let Some(e) = t.entering {
        sub.push_str(&format!(". Entra: {}", t.col_names[e]));
    }
    if let Some(r) = t.leaving {
        sub.push_str(&format!(". Sai: {}", t.col_names[t.basis[r]]));
    }
    s.subtitle = sub;
    let mut headers = vec!["Básica".to_string()];
    for (j, name) in t.col_names.iter().enumerate() {
        if t.blocked[j] {
            headers.push(format!("{} (bloq.)", name));
        } else {
            headers.push(name.clone());
        }
    }
    headers.push("Solução".into());
    s.headers = headers;
    let zlabel = if t.phase == 1 {
        "r (min)".to_string()
    } else if t.sense == Sense::Max {
        "z (max)".to_string()
    } else {
        "z (min)".to_string()
    };
    let mut zr = vec![cs(zlabel, CellStyle::Bold)];
    for j in 0..t.col_names.len() {
        let st = if Some(j) == t.entering {
            CellStyle::Green
        } else if t.blocked[j] {
            CellStyle::Muted
        } else {
            CellStyle::Bold
        };
        zr.push(cs(if t.blocked[j] { "bloq.".into() } else { num(t.zrow[j]) }, st));
    }
    zr.push(cs(num(t.zval), CellStyle::Bold));
    s.row(zr);
    for i in 0..t.rows.len() {
        let leaving = Some(i) == t.leaving;
        let bname = t.col_names[t.basis[i]].clone();
        let mut row = vec![cs(bname, if leaving { CellStyle::Red } else { CellStyle::Head })];
        for j in 0..t.col_names.len() {
            let st = if leaving && Some(j) == t.entering {
                CellStyle::Pivot
            } else if leaving {
                CellStyle::Red
            } else if Some(j) == t.entering {
                CellStyle::Green
            } else if t.col_kind[j] == ColKind::Artificial && t.blocked[j] {
                CellStyle::Muted
            } else {
                CellStyle::Normal
            };
            row.push(cs(num(t.rows[i][j]), st));
        }
        row.push(cs(num(t.rhs[i]), if leaving { CellStyle::Red } else { CellStyle::Normal }));
        s.row(row);
    }
    if !t.note.is_empty() {
        s.note(t.note.clone());
    }
    s
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
        if self.results.is_empty() {
            return None;
        }
        let mut all = self.results.clone();
        if !self.integer_mode {
            all.extend(self.iterations.iter().cloned());
        }
        let t = if self.integer_mode { "PROGRAMAÇÃO INTEIRA" } else { "PROGRAMAÇÃO LINEAR" };
        Some(sections_to_text(&format!("{}: {}", t, self.input.title), &all))
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
        let mut inp = LpInput::blank(2, 3, self.integer_mode);
        if self.integer_mode {
            inp.title = "Exemplo inteiro: dois produtos".into();
            inp.obj = vec!["7".into(), "5".into()];
            inp.a = vec![vec!["3".into(), "2".into()], vec!["1".into(), "3".into()], vec!["0".into(), "1".into()]];
            inp.rel = vec!["<=".into(), "<=".into(), "<=".into()];
            inp.rhs = vec!["13.5".into(), "12".into(), "3.5".into()];
        } else {
            inp.title = "Exemplo: mix de duas tintas".into();
            inp.obj = vec!["4".into(), "3".into()];
            inp.a = vec![vec!["2".into(), "1".into()], vec!["1".into(), "3".into()], vec!["1".into(), "0".into()]];
            inp.rel = vec!["<=".into(), "<=".into(), "<=".into()];
            inp.rhs = vec!["10".into(), "15".into(), "4".into()];
        }
        inp.con_names = vec!["Recurso A".into(), "Recurso B".into(), "Recurso C".into()];
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.bb = None;
        self.error = None;
        self.tab = Tab::Data;
    }
}
