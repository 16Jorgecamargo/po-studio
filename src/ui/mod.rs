pub mod algebra;
pub mod common;
pub mod game_ui;
pub mod lp_ui;
pub mod network_ui;
pub mod project_ui;
pub mod queue_ui;
pub mod theme;
pub mod transport_ui;

pub mod motion;

use common::*;
use eframe::egui::epaint::RectShape;
use eframe::egui::{self, Align2, CornerRadius, RichText, Sense, Shape, Stroke, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Data,
    Results,
    Iterations,
    Graph,
}

pub trait Module {
    fn key(&self) -> &'static str;
    fn ui(&mut self, ui: &mut egui::Ui);
    fn report(&self) -> Option<String>;
    fn save_json(&self) -> serde_json::Value;
    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String>;
    fn new_problem(&mut self);
    fn load_example(&mut self);
    fn solve_now(&mut self);
    fn set_tab(&mut self, t: Tab);
}

pub fn tab_bar(ui: &mut egui::Ui, tab: &mut Tab, tabs: &[(Tab, &str)]) {
    let ctx = ui.ctx().clone();
    let bar_id = ui.scope_id().with("tab_bar");
    let slide_id = bar_id.with("pivot");
    let origin = ui.cursor().min.to_vec2();
    let covered = motion::slide_peek(&ctx, slide_id).map(|r| r.translate(origin));
    let mut target = None;
    let row = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let slot = ui.painter().add(Shape::Noop);
        for (t, label) in tabs {
            let selected = *tab == *t;
            let measure = ui.painter().layout_no_wrap((*label).to_owned(), semibold(14.0), text_soft());
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(measure.size().x + 28.0, 34.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, selected, *label));
            let on_pivot = match covered {
                Some(r) => r.contains(rect.center()),
                None => selected,
            };
            let color = if on_pivot {
                PIVOT_INK
            } else if resp.hovered() || selected {
                text_strong()
            } else {
                text_soft()
            };
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, *label, semibold(14.0), color);
            if resp.has_focus() {
                focus_ring(ui.painter(), rect.shrink(3.0));
            }
            if selected {
                target = Some(rect);
            }
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *tab = *t;
            }
        }
        if let Some(rect) = target {
            let now = motion::slide_to(&ctx, slide_id, rect.translate(-origin), 0.3).translate(origin);
            ui.painter().set(slot, RectShape::filled(now, CornerRadius::same(2), PIVOT));
        }
    });
    let line_y = row.response.rect.bottom() + 1.0;
    ui.painter().hline(ui.max_rect().x_range(), line_y, Stroke::new(1.0, border()));
    ui.add_space(12.0);
    let last_id = bar_id.with("last");
    let last = ctx.data(|d| d.get_temp::<Tab>(last_id));
    if last != Some(*tab) {
        if last.is_some() {
            motion::bump();
        }
        ctx.data_mut(|d| d.insert_temp(last_id, *tab));
    }
    ui.multiply_opacity(motion::fade(&ctx, bar_id.with("fade"), 0.16));
}

pub fn title_row(ui: &mut egui::Ui, title: &mut String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Título do problema").color(text_soft()));
        ui.add(egui::TextEdit::singleline(title).desired_width(380.0).hint_text("Ex.: Mix de produção"));
    });
}

pub fn iterations_view(ui: &mut egui::Ui, id: &str, iterations: &[Section], shown: &mut usize) {
    if iterations.is_empty() {
        info_box(ui, "Clique em Resolver para gerar as iterações.");
        return;
    }
    ui.horizontal(|ui| {
        if primary_button(ui, "Próxima iteração") && *shown < iterations.len() {
            *shown += 1;
        }
        if secondary_button(ui, "Todas as iterações") {
            *shown = iterations.len();
        }
        if secondary_button(ui, "Reiniciar") {
            *shown = 1;
        }
        ui.label(RichText::new(format!("Mostrando {} de {}", (*shown).min(iterations.len()), iterations.len())).color(text_muted()));
    });
    ui.add_space(8.0);
    let k = (*shown).clamp(1, iterations.len());
    for (i, s) in iterations.iter().take(k).enumerate() {
        render_section(ui, s, (id, i));
    }
}

pub fn results_view(ui: &mut egui::Ui, id: &str, sections: &[Section]) {
    if sections.is_empty() {
        info_box(ui, "Preencha os dados e clique em Resolver.");
        return;
    }
    for (i, s) in sections.iter().enumerate() {
        render_section(ui, s, (id, i));
    }
}

pub fn solve_bar(ui: &mut egui::Ui) -> bool {
    ui.add_space(8.0);
    let mut clicked = false;
    ui.horizontal(|ui| {
        clicked = primary_button(ui, "Resolver");
    });
    if clicked {
        motion::bump();
    }
    clicked
}
