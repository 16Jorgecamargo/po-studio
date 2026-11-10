use crate::format::{get_format, set_format, NumFormat};
use crate::ui::algebra::AlgebraModule;
use crate::ui::common::*;
use crate::ui::game_ui::GameModule;
use crate::ui::lp_ui::LpModule;
use crate::ui::network_ui::{NetKind, NetworkModule};
use crate::ui::project_ui::ProjectModule;
use crate::ui::queue_ui::QueueModule;
use crate::ui::transport_ui::TransportModule;
use crate::ui::{motion, Module};
use eframe::egui::epaint::RectShape;
use eframe::egui::{self, Align2, CornerRadius, Pos2, Rect, RichText, Sense, Shape, Stroke, Vec2};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub const APP_NAME: &str = "PO Studio";

struct ModuleInfo {
    group: &'static str,
    name: &'static str,
    blurb: &'static str,
}

const MODULES: [ModuleInfo; 12] = [
    ModuleInfo { group: "Álgebra", name: "Equações lineares", blurb: "Resolve sistemas de equações simultâneas e calcula o determinante." },
    ModuleInfo { group: "Álgebra", name: "Inversão de matriz", blurb: "Inversa e determinante de uma matriz quadrada." },
    ModuleInfo { group: "Otimização", name: "Programação linear", blurb: "Simplex (folgas, M grande, duas fases, dual), sensibilidade e método gráfico." },
    ModuleInfo { group: "Otimização", name: "Programação inteira", blurb: "Branch & Bound com árvore de busca interativa." },
    ModuleInfo { group: "Otimização", name: "Modelo de transporte", blurb: "Noroeste, custo mínimo e Vogel, com quadros de iteração." },
    ModuleInfo { group: "Redes", name: "Árvore geradora mínima", blurb: "Conecta todos os nós com o menor comprimento total." },
    ModuleInfo { group: "Redes", name: "Caminho mais curto", blurb: "Algoritmos de Dijkstra e de Floyd." },
    ModuleInfo { group: "Redes", name: "Fluxo máximo", blurb: "Maior fluxo possível da origem ao destino." },
    ModuleInfo { group: "Projetos", name: "CPM", blurb: "Caminho crítico, folgas e gráfico de barras." },
    ModuleInfo { group: "Projetos", name: "PERT", blurb: "Durações incertas, média, variância e probabilidade de prazo." },
    ModuleInfo { group: "Probabilidade", name: "Filas", blurb: "Modelos (M/M/c):(GD/N/K), nascimento e morte puros." },
    ModuleInfo { group: "Probabilidade", name: "Jogos de soma zero", blurb: "Estratégias ótimas por PL e solução gráfica." },
];

pub struct OrApp {
    modules: Vec<Box<dyn Module>>,
    current: Option<usize>,
    status: Option<(String, bool)>,
    show_format: bool,
    files: Vec<Option<PathBuf>>,
    screenshot: Option<PathBuf>,
    frames: u32,
    shot_requested: bool,
    shot_done: Arc<Mutex<bool>>,
    dark: bool,
    reduce_motion: bool,
    last_current: Option<usize>,
}

impl OrApp {
    pub fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<PathBuf>, demo: Option<String>, forced_dark: Option<bool>) -> Self {
        setup_style(&cc.egui_ctx);
        let saved = if screenshot.is_some() { None } else { load_settings() };
        let dark = match (forced_dark, screenshot.is_some()) {
            (Some(d), _) => d,
            (None, true) => false,
            (None, false) => saved
                .as_ref()
                .and_then(|v| v.get("dark_mode")?.as_bool())
                .unwrap_or(cc.egui_ctx.system_theme() == Some(egui::Theme::Dark)),
        };
        let reduce_motion = saved
            .as_ref()
            .and_then(|v| v.get("reduce_motion")?.as_bool())
            .unwrap_or_else(system_reduced_motion);
        motion::set_reduced(reduce_motion);
        apply_theme(&cc.egui_ctx, dark);
        let modules: Vec<Box<dyn Module>> = vec![
            Box::new(AlgebraModule::new(true)),
            Box::new(AlgebraModule::new(false)),
            Box::new(LpModule::new(false)),
            Box::new(LpModule::new(true)),
            Box::new(TransportModule::new()),
            Box::new(NetworkModule::new(NetKind::Mst)),
            Box::new(NetworkModule::new(NetKind::Shortest)),
            Box::new(NetworkModule::new(NetKind::MaxFlow)),
            Box::new(ProjectModule::new(false)),
            Box::new(ProjectModule::new(true)),
            Box::new(QueueModule::new()),
            Box::new(GameModule::new()),
        ];
        let n = modules.len();
        let mut current = None;
        let mut modules = modules;
        if let Some(d) = demo {
            let mut parts = d.split(':');
            if let Some(i) = parts.next().and_then(|x| x.parse::<usize>().ok()).filter(|i| *i < n) {
                current = Some(i);
                modules[i].load_example();
                modules[i].solve_now();
                let tab = match parts.next().unwrap_or("results") {
                    "data" => crate::ui::Tab::Data,
                    "iterations" => crate::ui::Tab::Iterations,
                    "graph" => crate::ui::Tab::Graph,
                    _ => crate::ui::Tab::Results,
                };
                modules[i].set_tab(tab);
            }
        }
        Self {
            modules,
            current,
            status: None,
            show_format: false,
            files: vec![None; n],
            screenshot,
            frames: 0,
            shot_requested: false,
            shot_done: Arc::new(Mutex::new(false)),
            dark,
            reduce_motion,
            last_current: current,
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let panel = egui::Panel::left("sidebar")
            .resizable(false)
            .exact_size(232.0)
            .frame(egui::Frame::new().fill(sidebar_fill()).inner_margin(egui::Margin { left: 14, right: 14, top: 16, bottom: 12 }))
            .show(ui, |ui| {
                let home = ui
                    .add(egui::Button::new(RichText::new(APP_NAME).font(semibold(20.0)).color(text_strong())).frame(false))
                    .on_hover_text("Voltar à lista de módulos");
                if home.clicked() {
                    self.current = None;
                }
                ui.label(RichText::new("Pesquisa Operacional").size(12.5).color(text_muted()));
                ui.add_space(8.0);
                let mut toggle_theme_now = false;
                let mut toggle_motion_now = false;
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    let width = ui.available_width();
                    let motion_label = if self.reduce_motion { "Ativar animações" } else { "Reduzir animações" };
                    if ui
                        .add(egui::Button::new(RichText::new(motion_label).size(13.0)).min_size(Vec2::new(width, 30.0)))
                        .on_hover_text("Liga ou desliga os movimentos da interface")
                        .clicked()
                    {
                        toggle_motion_now = true;
                    }
                    let theme_label = if self.dark { "Modo claro" } else { "Modo escuro" };
                    if ui
                        .add(egui::Button::new(RichText::new(theme_label).size(13.0)).min_size(Vec2::new(width, 30.0)))
                        .on_hover_text("Alternar entre tema claro e escuro")
                        .clicked()
                    {
                        toggle_theme_now = true;
                    }
                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| self.module_list(ui));
                });
                if toggle_motion_now {
                    self.reduce_motion = !self.reduce_motion;
                    motion::set_reduced(self.reduce_motion);
                    apply_theme(ui.ctx(), self.dark);
                    save_settings(self.dark, self.reduce_motion);
                }
                if toggle_theme_now {
                    self.dark = !self.dark;
                    toggle_theme(ui.ctx(), self.dark);
                    save_settings(self.dark, self.reduce_motion);
                }
            });
        let edge = panel.response.rect;
        ui.painter().vline(edge.right(), edge.y_range(), Stroke::new(1.0, border()));
    }

    fn module_list(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let slide_id = egui::Id::unique("sidebar_pivot");
        egui::ScrollArea::vertical().id_salt("side").show(ui, |ui| {
            let origin = ui.cursor().min.to_vec2();
            let covered = if self.current.is_some() { motion::slide_peek(&ctx, slide_id).map(|r| r.translate(origin)) } else { None };
            let slot = ui.painter().add(Shape::Noop);
            let width = ui.available_width();
            let mut target = None;
            let mut last_group = "";
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, m) in MODULES.iter().enumerate() {
                if m.group != last_group {
                    ui.add_space(12.0);
                    ui.label(RichText::new(m.group).font(semibold(12.5)).color(text_soft()));
                    ui.add_space(2.0);
                    last_group = m.group;
                }
                let selected = self.current == Some(i);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, m.name));
                let on_pivot = match covered {
                    Some(r) => r.contains(rect.center()),
                    None => selected,
                };
                if resp.hovered() && !selected {
                    ui.painter().rect_filled(rect, CornerRadius::same(2), button_hover_fill());
                }
                let color = if on_pivot {
                    PIVOT_INK
                } else if resp.hovered() || selected {
                    text_strong()
                } else {
                    text_body()
                };
                let font = if selected { semibold(14.0) } else { sans(14.0) };
                ui.painter().text(Pos2::new(rect.left() + 10.0, rect.center().y), Align2::LEFT_CENTER, m.name, font, color);
                if resp.has_focus() {
                    focus_ring(ui.painter(), rect.shrink(3.0));
                }
                if selected {
                    target = Some(rect);
                }
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.current = Some(i);
                    self.status = None;
                }
            }
            match target {
                Some(rect) => {
                    let now = motion::slide_to(&ctx, slide_id, rect.translate(-origin), 0.32).translate(origin);
                    ui.painter().set(slot, RectShape::filled(now, CornerRadius::same(2), PIVOT));
                }
                None => motion::slide_clear(&ctx, slide_id),
            }
        });
    }

    fn actions(&mut self, ui: &mut egui::Ui, idx: usize, reversed: bool) {
        let mut order = [0usize, 1, 2, 3, 4, 5, 6];
        if reversed {
            order.reverse();
        }
        for k in order {
            match k {
                0 => {
                    if secondary_button(ui, "Novo") {
                        self.modules[idx].new_problem();
                        self.files[idx] = None;
                        self.status = None;
                    }
                }
                1 => {
                    if secondary_button(ui, "Exemplo") {
                        self.modules[idx].load_example();
                        self.status = Some(("Exemplo carregado. Clique em Resolver.".into(), false));
                    }
                }
                2 => {
                    if secondary_button(ui, "Abrir…") {
                        self.open(idx);
                    }
                }
                3 => {
                    if secondary_button(ui, "Salvar") {
                        self.save(idx, false);
                    }
                }
                4 => {
                    if secondary_button(ui, "Salvar como…") {
                        self.save(idx, true);
                    }
                }
                5 => {
                    if secondary_button(ui, "Exportar relatório") {
                        self.export_report(idx);
                    }
                }
                _ => {
                    if secondary_button(ui, "Formato numérico") {
                        self.show_format = !self.show_format;
                    }
                }
            }
        }
    }

    fn header(&mut self, ui: &mut egui::Ui, idx: usize) {
        let wide = ui.available_width() >= 940.0;
        if wide {
            ui.horizontal(|ui| {
                ui.label(RichText::new(MODULES[idx].name).font(semibold(26.0)).color(text_strong()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| self.actions(ui, idx, true));
            });
        } else {
            ui.label(RichText::new(MODULES[idx].name).font(semibold(26.0)).color(text_strong()));
        }
        ui.label(RichText::new(MODULES[idx].blurb).color(text_muted()));
        if !wide {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| self.actions(ui, idx, false));
        }
        if self.show_format {
            ui.add_space(6.0);
            card(ui, |ui| {
                let mut f = get_format();
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Formato de saída:").font(semibold(14.0)).color(text_strong()));
                    ui.radio_value(&mut f.scientific, false, "Decimal (NNNNN.DD)");
                    ui.radio_value(&mut f.scientific, true, "Notação científica (N.NNNNeDD)");
                    ui.add_space(12.0);
                    ui.label("Casas decimais (D):");
                    ui.add(egui::DragValue::new(&mut f.decimals).range(0..=10));
                    ui.label("Dígitos (N):");
                    ui.add(egui::DragValue::new(&mut f.width).range(1..=15));
                });
                ui.label(RichText::new("Clique em Resolver de novo para aplicar o formato aos resultados.").small().color(text_muted()));
                set_format(f);
            });
        }
        if let Some((msg, err)) = &self.status {
            ui.add_space(6.0);
            if *err {
                error_box(ui, msg);
            } else {
                info_box(ui, msg);
            }
        }
        ui.add_space(10.0);
    }

    fn save(&mut self, idx: usize, ask: bool) {
        let path = match (&self.files[idx], ask) {
            (Some(p), false) => Some(p.clone()),
            _ => rfd::FileDialog::new()
                .add_filter("Problema PO Studio", &["json"])
                .add_filter("Arquivo do programa original", &["txt"])
                .set_file_name(format!("{}.json", self.modules[idx].key()))
                .save_file(),
        };
        let Some(path) = path else {
            self.status = Some(("Você não indicou um nome de arquivo.".into(), true));
            return;
        };
        let legacy_target = path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("txt")).unwrap_or(false);
        let content = if legacy_target {
            match crate::legacy::export(self.modules[idx].key(), &self.modules[idx].save_json()) {
                Ok(t) => t,
                Err(e) => {
                    self.status = Some((e, true));
                    return;
                }
            }
        } else {
            let doc = serde_json::json!({
                "app": APP_NAME,
                "version": 1,
                "module": self.modules[idx].key(),
                "data": self.modules[idx].save_json(),
            });
            serde_json::to_string_pretty(&doc).unwrap_or_default()
        };
        match std::fs::write(&path, content) {
            Ok(_) => {
                self.status = Some((format!("Salvo em {}", path.display()), false));
                self.files[idx] = Some(path);
            }
            Err(e) => self.status = Some((format!("Não foi possível salvar: {}", e), true)),
        }
    }

    fn open(&mut self, idx: usize) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Problemas", &["json", "txt"])
            .add_filter("Problema PO Studio", &["json"])
            .add_filter("Arquivo do programa original", &["txt"])
            .pick_file()
        else {
            return;
        };
        let text = match std::fs::read(&path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(t) => t,
                Err(e) => e.into_bytes().iter().map(|&b| b as char).collect(),
            },
            Err(e) => {
                self.status = Some((format!("Não foi possível abrir: {}", e), true));
                return;
            }
        };
        let doc: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) if crate::legacy::looks_legacy(&text) => match crate::legacy::import(&text) {
                Ok(imp) => serde_json::json!({ "module": imp.module, "data": imp.data }),
                Err(e) => {
                    self.status = Some((format!("Arquivo do programa original não reconhecido: {}", e), true));
                    return;
                }
            },
            Err(_) => {
                self.status = Some(("O arquivo não é um problema válido.".into(), true));
                return;
            }
        };
        let module = doc.get("module").and_then(|v| v.as_str()).unwrap_or("");
        let target = self.modules.iter().position(|m| m.key() == module);
        let Some(t) = target else {
            self.status = Some(("O arquivo não corresponde a nenhum módulo.".into(), true));
            return;
        };
        if t != idx {
            self.current = Some(t);
        }
        match self.modules[t].load_json(doc.get("data").cloned().unwrap_or_default()) {
            Ok(_) => {
                self.files[t] = Some(path.clone());
                self.status = Some((format!("Aberto: {}", path.display()), false));
            }
            Err(e) => self.status = Some((format!("O arquivo não corresponde ao formato do módulo escolhido: {}", e), true)),
        }
    }

    fn export_report(&mut self, idx: usize) {
        let Some(text) = self.modules[idx].report() else {
            self.status = Some(("Resolva o problema antes de exportar o relatório.".into(), true));
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Texto", &["txt"])
            .set_file_name(format!("{}_relatorio.txt", self.modules[idx].key()))
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, text) {
            Ok(_) => self.status = Some((format!("Relatório salvo em {}", path.display()), false)),
            Err(e) => self.status = Some((format!("Não foi possível salvar: {}", e), true)),
        }
    }

    fn home(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.add_space(8.0);
        ui.label(RichText::new("Escolha um módulo").font(semibold(30.0)).color(text_strong()));
        ui.label(
            RichText::new(format!("{} módulos de Pesquisa Operacional. Cada um resolve o problema e mostra o caminho até a resposta.", MODULES.len()))
                .size(15.0)
                .color(text_muted()),
        );
        ui.add_space(20.0);
        let avail = ui.available_width();
        let side_label = avail >= 640.0;
        let label_w = if side_label { 132.0 } else { 0.0 };
        let cols = (((avail - label_w) / 220.0).floor() as usize).clamp(1, 3);
        let cell_w = ((avail - label_w) / cols as f32).floor();
        let pad = 14.0;
        let blurbs: Vec<_> = MODULES.iter().map(|m| ui.painter().layout(m.blurb.to_owned(), sans(13.0), text_soft(), cell_w - pad * 2.0)).collect();
        let blurb_h = blurbs.iter().map(|g| g.size().y).fold(0.0f32, f32::max);
        let cell_h = pad + 22.0 + 6.0 + blurb_h + pad;
        let stacked_label_h = if side_label { 0.0 } else { 30.0 };
        let mut groups: Vec<(&str, Vec<usize>)> = Vec::new();
        for (i, m) in MODULES.iter().enumerate() {
            match groups.last_mut() {
                Some((g, v)) if *g == m.group => v.push(i),
                _ => groups.push((m.group, vec![i])),
            }
        }
        let group_h = |n: usize| stacked_label_h + n.div_ceil(cols) as f32 * cell_h;
        let total_h: f32 = groups.iter().map(|(_, v)| group_h(v.len())).sum();
        let (area, _) = ui.allocate_exact_size(Vec2::new(avail, total_h), Sense::hover());
        let slide_id = egui::Id::unique("home_pivot");
        let covered = motion::slide_peek(&ctx, slide_id).map(|r| r.translate(area.min.to_vec2()));
        let glow = ctx.animate_bool_with_time(slide_id.with("on"), ctx.data(|d| d.get_temp::<bool>(slide_id.with("hot"))).unwrap_or(false), 0.14);
        let elapsed = motion::clock(&ctx, "home_wave", 1.2);
        let painter = ui.painter().clone();
        let slot = painter.add(Shape::Noop);
        let rule = Stroke::new(1.0, border());
        let mut hot = None;
        let mut clicked = None;
        let mut y = area.top();
        let mut wave_row = 0usize;
        for (group, items) in &groups {
            painter.hline(area.x_range(), y, Stroke::new(1.5, text_soft()));
            let label_pos = if side_label { Pos2::new(area.left() + 2.0, y + pad + 2.0) } else { Pos2::new(area.left() + 2.0, y + 7.0) };
            painter.text(label_pos, Align2::LEFT_TOP, *group, semibold(13.5), text_soft());
            let cells_top = y + stacked_label_h;
            let cells_left = area.left() + label_w;
            for (k, &i) in items.iter().enumerate() {
                let (r, c) = (k / cols, k % cols);
                let rect = Rect::from_min_size(Pos2::new(cells_left + c as f32 * cell_w, cells_top + r as f32 * cell_h), Vec2::new(cell_w, cell_h));
                let resp = ui.interact(rect, egui::Id::unique(("home_cell", i)), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, MODULES[i].name));
                if resp.hovered() || resp.has_focus() {
                    hot = Some(rect);
                }
                if resp.clicked() {
                    clicked = Some(i);
                }
                let p = motion::ease_out_cubic(motion::phase(elapsed, (wave_row + r + c) as f32 * 0.055, 0.4));
                let lift = Vec2::new(0.0, (1.0 - p) * 10.0);
                let on_pivot = glow > 0.5 && covered.is_some_and(|h| h.contains(rect.center()));
                let (name_color, blurb_color) = if on_pivot { (PIVOT_INK, PIVOT_INK) } else { (text_strong(), text_soft()) };
                if c > 0 || side_label {
                    painter.vline(rect.left(), rect.y_range(), rule);
                }
                if r > 0 && c == 0 {
                    painter.hline(cells_left..=area.right(), rect.top(), rule);
                }
                let at = rect.min + Vec2::new(pad, pad) + lift;
                painter.text(at, Align2::LEFT_TOP, MODULES[i].name, semibold(16.0), name_color.gamma_multiply(p));
                painter.galley_with_override_text_color(at + Vec2::new(0.0, 28.0), blurbs[i].clone(), blurb_color.gamma_multiply(p));
                if resp.has_focus() {
                    focus_ring(&painter, rect.shrink(4.0));
                }
                resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            }
            wave_row += items.len().div_ceil(cols);
            y += group_h(items.len());
        }
        painter.hline(area.x_range(), y, Stroke::new(1.5, text_soft()));
        ctx.data_mut(|d| d.insert_temp(slide_id.with("hot"), hot.is_some()));
        if let Some(rect) = hot {
            let rel = rect.shrink(1.0).translate(-area.min.to_vec2());
            if glow <= 0.01 {
                motion::slide_clear(&ctx, slide_id);
            }
            let now = motion::slide_to(&ctx, slide_id, rel, 0.26).translate(area.min.to_vec2());
            painter.set(slot, RectShape::filled(now, CornerRadius::same(2), PIVOT.gamma_multiply(glow.max(0.35))));
        } else if let Some(prev) = covered {
            if glow > 0.01 {
                painter.set(slot, RectShape::filled(prev, CornerRadius::same(2), PIVOT.gamma_multiply(glow)));
            }
        }
        ui.add_space(16.0);
        if let Some(i) = clicked {
            self.current = Some(i);
        }
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.frames == 1 {
            if let Some(size) = shot_size() {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
        }
        if *self.shot_done.lock().unwrap() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if self.frames >= shot_frame() && !self.shot_requested {
            self.shot_requested = true;
            let done = self.shot_done.clone();
            ctx.request_screenshot(move |img| {
                let [w, h] = img.size;
                let mut buf = Vec::with_capacity(w * h * 4);
                for p in &img.pixels {
                    buf.extend_from_slice(&[p.r(), p.g(), p.b(), 255]);
                }
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Some(im) = image::RgbaImage::from_raw(w as u32, h as u32, buf) {
                    let _ = im.save(&path);
                }
                *done.lock().unwrap() = true;
            });
        }
        if self.frames > shot_frame() + 480 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for OrApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_screenshot(&ctx);
        tick_theme(&ctx);
        if self.current != self.last_current {
            self.last_current = self.current;
            motion::bump();
        }
        self.sidebar(ui);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg()).inner_margin(egui::Margin::symmetric(26, 18)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().id_salt("main").auto_shrink([false, false]).show(ui, |ui| {
                    ui.multiply_opacity(motion::fade(ui.ctx(), "screen_fade", 0.2));
                    match self.current {
                        None => self.home(ui),
                        Some(idx) => {
                            self.header(ui, idx);
                            self.modules[idx].ui(ui);
                        }
                    }
                });
            });
    }
}

fn shot_frame() -> u32 {
    std::env::var("PO_STUDIO_SHOT_FRAME").ok().and_then(|v| v.parse().ok()).unwrap_or(120)
}

fn shot_size() -> Option<Vec2> {
    let v = std::env::var("PO_STUDIO_SHOT_SIZE").ok()?;
    let (w, h) = v.split_once('x')?;
    Some(Vec2::new(w.parse().ok()?, h.parse().ok()?))
}

fn setup_style(ctx: &egui::Context) {
    install_fonts(ctx);
    set_format(NumFormat::default());
}

fn system_reduced_motion() -> bool {
    let run = |cmd: &str, args: &[&str]| -> Option<String> {
        let out = std::process::Command::new(cmd).args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_lowercase())
    };
    if cfg!(target_os = "macos") {
        run("defaults", &["read", "com.apple.universalaccess", "reduceMotion"]).is_some_and(|v| v == "1")
    } else if cfg!(target_os = "linux") {
        run("gsettings", &["get", "org.gnome.desktop.interface", "enable-animations"]).is_some_and(|v| v == "false")
    } else {
        false
    }
}

fn settings_path() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    base.map(|b| b.join("PO Studio").join("settings.json"))
}

fn load_settings() -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(settings_path()?).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_settings(dark: bool, reduce_motion: bool) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, serde_json::json!({ "dark_mode": dark, "reduce_motion": reduce_motion }).to_string());
}
