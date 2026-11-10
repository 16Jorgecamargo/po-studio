use super::motion;
pub use super::theme::*;
use eframe::egui::{self, Align2, Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellStyle {
    Normal,
    Head,
    Green,
    Red,
    Pivot,
    Bold,
    Muted,
    RedText,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub text: String,
    pub style: CellStyle,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Normal,
        }
    }
    pub fn styled(text: impl Into<String>, style: CellStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

pub fn c(text: impl Into<String>) -> Cell {
    Cell::new(text)
}

pub fn cs(text: impl Into<String>, style: CellStyle) -> Cell {
    Cell::styled(text, style)
}

pub fn n(v: f64) -> Cell {
    Cell::new(crate::format::num(v))
}

#[derive(Clone, Debug, Default)]
pub struct Section {
    pub title: String,
    pub subtitle: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
    pub notes: Vec<String>,
    pub lines: Vec<String>,
}

impl Section {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }
    pub fn sub(mut self, s: impl Into<String>) -> Self {
        self.subtitle = s.into();
        self
    }
    pub fn headers(mut self, h: &[&str]) -> Self {
        self.headers = h.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn headers_owned(mut self, h: Vec<String>) -> Self {
        self.headers = h;
        self
    }
    pub fn row(&mut self, r: Vec<Cell>) {
        self.rows.push(r);
    }
    pub fn note(&mut self, s: impl Into<String>) {
        self.notes.push(s.into());
    }
    pub fn line(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(card_fill())
        .stroke(Stroke::new(1.0, border()))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

pub fn render_section(ui: &mut egui::Ui, s: &Section, id: impl std::hash::Hash + std::fmt::Debug) {
    card(ui, |ui| {
        if !s.title.is_empty() {
            ui.label(RichText::new(&s.title).font(semibold(15.5)).color(text_strong()));
        }
        if !s.subtitle.is_empty() {
            ui.label(RichText::new(&s.subtitle).size(13.0).color(text_muted()));
        }
        for l in &s.lines {
            ui.label(RichText::new(l).size(14.5).color(text_strong()));
        }
        if !s.headers.is_empty() || !s.rows.is_empty() {
            ui.add_space(8.0);
            egui::ScrollArea::horizontal().id_salt(egui::Id::new(&id).with("h")).show(ui, |ui| {
                tableau(ui, s, egui::Id::new(&id).with("tableau"));
            });
        }
        for nt in &s.notes {
            ui.add_space(4.0);
            ui.label(RichText::new(nt).size(13.0).color(text_soft()));
        }
    });
    ui.add_space(10.0);
}

fn looks_numeric(text: &str) -> bool {
    match text.chars().next() {
        Some(ch) => ch.is_ascii_digit() || matches!(ch, '-' | '−' | '+' | '(' | '[' | '∞' | '.' | '–'),
        None => false,
    }
}

fn cell_colors(style: CellStyle) -> (Option<Color32>, Color32, bool) {
    match style {
        CellStyle::Normal => (None, text_body(), false),
        CellStyle::Head => (None, text_strong(), true),
        CellStyle::Bold => (None, text_strong(), true),
        CellStyle::Muted => (None, text_muted(), false),
        CellStyle::Green => (Some(green_soft()), green_text(), false),
        CellStyle::Red => (Some(red_soft()), red_text(), false),
        CellStyle::Pivot => (Some(PIVOT), PIVOT_INK, true),
        CellStyle::RedText => (None, red(), true),
    }
}

fn tableau(ui: &mut egui::Ui, s: &Section, id: egui::Id) {
    const PAD_X: f32 = 12.0;
    const ROW_H: f32 = 27.0;
    const HEAD_H: f32 = 28.0;
    const STAGGER: f32 = 0.032;
    const STAGGER_CAP: f32 = 0.32;
    const ROW_SECONDS: f32 = 0.28;
    let ncol = s.headers.len().max(s.rows.iter().map(|r| r.len()).max().unwrap_or(0));
    if ncol == 0 {
        return;
    }
    let elapsed = motion::clock(ui.ctx(), id, STAGGER_CAP + ROW_SECONDS);
    let painter = ui.painter().clone();
    let head_galleys: Vec<_> = (0..ncol)
        .map(|j| painter.layout_no_wrap(s.headers.get(j).cloned().unwrap_or_default(), mono_bold(12.5), text_soft()))
        .collect();
    let mut numeric = vec![false; ncol];
    for (j, flag) in numeric.iter_mut().enumerate() {
        let filled: Vec<&Cell> = s.rows.iter().filter_map(|r| r.get(j)).filter(|cell| !cell.text.is_empty()).collect();
        let hits = filled.iter().filter(|cell| looks_numeric(&cell.text)).count();
        *flag = j > 0 && !filled.is_empty() && hits * 2 > filled.len();
    }
    let cell_galleys: Vec<Vec<_>> = s
        .rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|cell| {
                    let (_, color, bold) = cell_colors(cell.style);
                    let font = if bold { mono_bold(13.5) } else { mono(13.5) };
                    painter.layout_no_wrap(cell.text.clone(), font, color)
                })
                .collect()
        })
        .collect();
    let mut widths: Vec<f32> = head_galleys.iter().map(|g| g.size().x).collect();
    for row in &cell_galleys {
        for (j, g) in row.iter().enumerate() {
            widths[j] = widths[j].max(g.size().x);
        }
    }
    for w in widths.iter_mut() {
        *w = (*w + PAD_X * 2.0).max(56.0);
    }
    let head_h = if s.headers.is_empty() { 0.0 } else { HEAD_H };
    let size = Vec2::new(widths.iter().sum(), head_h + s.rows.len() as f32 * ROW_H);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.context_menu(|ui| {
        if ui.button("Copiar tabela").clicked() {
            let mut out = String::new();
            if !s.headers.is_empty() {
                out.push_str(&s.headers.join("\t"));
                out.push('\n');
            }
            for r in &s.rows {
                out.push_str(&r.iter().map(|cell| cell.text.clone()).collect::<Vec<_>>().join("\t"));
                out.push('\n');
            }
            ui.ctx().copy_text(out);
            ui.close();
        }
    });
    if !ui.is_rect_visible(rect) {
        return;
    }
    let mut lefts = Vec::with_capacity(ncol);
    let mut x = rect.left();
    for w in &widths {
        lefts.push(x);
        x += w;
    }
    let place = |j: usize, galley_w: f32| -> f32 {
        if numeric[j] {
            lefts[j] + widths[j] - PAD_X - galley_w
        } else {
            lefts[j] + PAD_X
        }
    };
    let head_alpha = motion::ease_out_cubic(motion::phase(elapsed, 0.0, 0.18));
    for (j, g) in head_galleys.iter().enumerate() {
        let pos = Pos2::new(place(j, g.size().x), rect.top() + (HEAD_H - g.size().y) / 2.0);
        painter.galley_with_override_text_color(pos, g.clone(), text_soft().gamma_multiply(head_alpha));
    }
    for (i, row) in cell_galleys.iter().enumerate() {
        let p = motion::ease_out_cubic(motion::phase(elapsed, (i as f32 * STAGGER).min(STAGGER_CAP), ROW_SECONDS));
        let top = rect.top() + head_h + i as f32 * ROW_H - (1.0 - p) * 8.0;
        if i > 0 {
            painter.hline(rect.x_range(), top, Stroke::new(1.0, grid_line().gamma_multiply(p)));
        }
        for (j, g) in row.iter().enumerate() {
            let cell = &s.rows[i][j];
            let (fill, color, _) = cell_colors(cell.style);
            let cell_rect = Rect::from_min_size(Pos2::new(lefts[j], top), Vec2::new(widths[j], ROW_H));
            if let Some(f) = fill {
                painter.rect_filled(cell_rect, CornerRadius::ZERO, f.gamma_multiply(p));
            }
            if cell.style == CellStyle::Pivot {
                painter.rect_stroke(cell_rect, CornerRadius::ZERO, Stroke::new(1.5, PIVOT_INK.gamma_multiply(p)), StrokeKind::Inside);
            }
            let pos = Pos2::new(place(j, g.size().x), top + (ROW_H - g.size().y) / 2.0);
            painter.galley_with_override_text_color(pos, g.clone(), color.gamma_multiply(p));
        }
    }
    if head_h > 0.0 {
        painter.hline(rect.x_range(), rect.top() + head_h, Stroke::new(1.5, text_soft().gamma_multiply(head_alpha)));
    }
    if ncol > 1 {
        painter.vline(lefts[1], rect.y_range(), Stroke::new(1.0, border().gamma_multiply(head_alpha)));
    }
}

pub fn sections_to_text(title: &str, sections: &[Section]) -> String {
    let mut out = String::new();
    out.push_str(title);
    out.push('\n');
    out.push_str(&"=".repeat(title.chars().count().max(10)));
    out.push_str("\n\n");
    for s in sections {
        if !s.title.is_empty() {
            out.push_str(&s.title);
            out.push('\n');
        }
        if !s.subtitle.is_empty() {
            out.push_str(&s.subtitle);
            out.push('\n');
        }
        for l in &s.lines {
            out.push_str(l);
            out.push('\n');
        }
        let ncol = s.headers.len().max(s.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if ncol > 0 {
            let mut w = vec![0usize; ncol];
            for (i, h) in s.headers.iter().enumerate() {
                w[i] = w[i].max(h.chars().count());
            }
            for r in &s.rows {
                for (i, cell) in r.iter().enumerate() {
                    w[i] = w[i].max(cell.text.chars().count());
                }
            }
            let fmt_row = |cells: Vec<String>| -> String {
                cells
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("{:>width$}", t, width = w[i] + 2))
                    .collect::<String>()
            };
            if !s.headers.is_empty() {
                out.push_str(&fmt_row(s.headers.clone()));
                out.push('\n');
            }
            for r in &s.rows {
                out.push_str(&fmt_row(r.iter().map(|c| c.text.clone()).collect()));
                out.push('\n');
            }
        }
        for nt in &s.notes {
            out.push_str(nt);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

pub fn stepper(ui: &mut egui::Ui, label: &str, value: &mut usize, min: usize, max: usize) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(text_soft()));
        let square = Vec2::new(28.0, 28.0);
        if ui.add_enabled(*value > min, egui::Button::new("−").min_size(square)).on_hover_text("Diminuir").clicked() {
            *value -= 1;
            changed = true;
        }
        let mut txt = value.to_string();
        let r = ui.add(egui::TextEdit::singleline(&mut txt).font(mono(13.5)).desired_width(36.0).horizontal_align(egui::Align::Center));
        if r.changed() {
            if let Ok(v) = txt.trim().parse::<usize>() {
                *value = v.clamp(min, max);
                changed = true;
            }
        }
        if ui.add_enabled(*value < max, egui::Button::new("+").min_size(square)).on_hover_text("Aumentar").clicked() {
            *value += 1;
            changed = true;
        }
    });
    changed
}

pub fn cell_edit(ui: &mut egui::Ui, s: &mut String, width: f32) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(s)
            .font(mono(13.5))
            .desired_width(width)
            .min_size(Vec2::new(width, 0.0))
            .horizontal_align(egui::Align::Center),
    )
}

pub fn head(ui: &mut egui::Ui, s: &str) {
    ui.label(RichText::new(s).font(semibold(13.0)).color(text_soft()));
}

pub fn resize_vec<T: Clone>(v: &mut Vec<T>, n: usize, fill: T) {
    v.resize(n, fill);
}

pub fn resize_grid(g: &mut Vec<Vec<String>>, rows: usize, cols: usize) {
    g.resize(rows, vec![String::new(); cols]);
    for r in g.iter_mut() {
        r.resize(cols, String::new());
    }
}

pub fn resize_names(v: &mut Vec<String>, n: usize, prefix: &str) {
    let start = v.len();
    v.truncate(n);
    for i in start..n {
        v.push(format!("{}{}", prefix, i + 1));
    }
}

fn message_box(ui: &mut egui::Ui, fill: Color32, text: RichText) {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(3))
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(text);
        });
    ui.add_space(8.0);
}

pub fn error_box(ui: &mut egui::Ui, msg: &str) {
    message_box(ui, red_soft(), RichText::new(format!("Atenção: {}", msg)).color(red_text()));
}

pub fn info_box(ui: &mut egui::Ui, msg: &str) {
    message_box(ui, accent_soft(), RichText::new(msg).color(info_text()));
}

pub fn focus_ring(painter: &egui::Painter, rect: Rect) {
    painter.rect_stroke(rect.expand(2.0), CornerRadius::same(4), Stroke::new(2.0, accent()), StrokeKind::Outside);
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> bool {
    let galley = ui.painter().layout_no_wrap(text.to_owned(), semibold(14.0), bg());
    let size = Vec2::new((galley.size().x + 32.0).max(120.0), 34.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, text));
    if ui.is_rect_visible(rect) {
        let down = resp.is_pointer_button_down_on();
        let press = if motion::reduced() { 0.0 } else { ui.ctx().animate_bool_with_time(resp.id.with("press"), down, 0.12) };
        let drawn = rect.scale_from_center(1.0 - 0.05 * press);
        let fill = if resp.hovered() { text_body() } else { text_strong() };
        let painter = ui.painter();
        painter.rect_filled(drawn, CornerRadius::same(3), fill);
        painter.galley(drawn.center() - galley.size() / 2.0, galley, bg());
        if resp.has_focus() {
            focus_ring(painter, rect);
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> bool {
    ui.add(egui::Button::new(RichText::new(text)).min_size(Vec2::new(0.0, 30.0))).clicked()
}

pub fn legend_dot(ui: &mut egui::Ui, col: Color32, text: &str) {
    let (r, p) = ui.allocate_painter(Vec2::new(14.0, 14.0), Sense::hover());
    p.circle_filled(r.rect.center(), 6.0, col);
    ui.label(text);
    ui.add_space(8.0);
}

pub fn partial_segment(painter: &egui::Painter, a: Pos2, b: Pos2, progress: f32, stroke: Stroke) {
    if progress <= 0.0 {
        return;
    }
    painter.line_segment([a, a + (b - a) * progress.min(1.0)], stroke);
}

pub fn optimum_marker(painter: &egui::Painter, at: Pos2, elapsed: f32, delay: f32, color: Color32) -> f32 {
    let pop = motion::phase(elapsed, delay, 0.42);
    if pop <= 0.0 {
        return 0.0;
    }
    let halo = motion::phase(elapsed, delay + 0.1, 0.7);
    if halo > 0.0 && halo < 1.0 {
        let e = motion::ease_out_cubic(halo);
        painter.circle_stroke(at, 6.0 + e * 18.0, Stroke::new(2.0, color.gamma_multiply(0.7 * (1.0 - e))));
    }
    painter.circle_filled(at, 6.0 * motion::ease_out_back(pop), color);
    pop
}

pub struct Plot {
    pub rect: Rect,
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
}

impl Plot {
    pub fn to_screen(&self, x: f64, y: f64) -> Pos2 {
        let fx = ((x - self.xmin) / (self.xmax - self.xmin)) as f32;
        let fy = ((y - self.ymin) / (self.ymax - self.ymin)) as f32;
        Pos2::new(
            self.rect.left() + fx * self.rect.width(),
            self.rect.bottom() - fy * self.rect.height(),
        )
    }
}

fn nice_step(range: f64) -> f64 {
    let raw = range / 8.0;
    let mag = 10f64.powf(raw.abs().log10().floor());
    let r = raw / mag;
    let s = if r < 1.5 {
        1.0
    } else if r < 3.0 {
        2.0
    } else if r < 7.0 {
        5.0
    } else {
        10.0
    };
    s * mag
}

pub fn plot_area(ui: &mut egui::Ui, height: f32, xmin: f64, xmax: f64, ymin: f64, ymax: f64, xlabel: &str, ylabel: &str) -> (egui::Painter, Plot, egui::Response) {
    let width = ui.available_width();
    let (resp, painter) = ui.allocate_painter(Vec2::new(width, height), Sense::click_and_drag());
    let outer = resp.rect;
    painter.rect_filled(outer, CornerRadius::same(4), card_fill());
    painter.rect_stroke(outer, CornerRadius::same(4), Stroke::new(1.0, border()), StrokeKind::Inside);
    let rect = Rect::from_min_max(
        Pos2::new(outer.left() + 60.0, outer.top() + 30.0),
        Pos2::new(outer.right() - 24.0, outer.bottom() - 44.0),
    );
    let plot = Plot {
        rect,
        xmin,
        xmax,
        ymin,
        ymax,
    };
    let grid = Stroke::new(1.0, grid_line());
    let axis = Stroke::new(1.5, axis_line());
    let font = mono(11.5);
    let sx = nice_step(xmax - xmin);
    let mut x = (xmin / sx).ceil() * sx;
    while x <= xmax + 1e-9 {
        let p = plot.to_screen(x, ymin);
        painter.line_segment([Pos2::new(p.x, rect.top()), Pos2::new(p.x, rect.bottom())], grid);
        painter.text(Pos2::new(p.x, rect.bottom() + 5.0), Align2::CENTER_TOP, short(x), font.clone(), text_muted());
        x += sx;
    }
    let sy = nice_step(ymax - ymin);
    let mut y = (ymin / sy).ceil() * sy;
    while y <= ymax + 1e-9 {
        let p = plot.to_screen(xmin, y);
        painter.line_segment([Pos2::new(rect.left(), p.y), Pos2::new(rect.right(), p.y)], grid);
        painter.text(Pos2::new(rect.left() - 7.0, p.y), Align2::RIGHT_CENTER, short(y), font.clone(), text_muted());
        y += sy;
    }
    let x0 = if xmin <= 0.0 && xmax >= 0.0 { 0.0 } else { xmin };
    let y0 = if ymin <= 0.0 && ymax >= 0.0 { 0.0 } else { ymin };
    painter.line_segment([plot.to_screen(xmin, y0), plot.to_screen(xmax, y0)], axis);
    painter.line_segment([plot.to_screen(x0, ymin), plot.to_screen(x0, ymax)], axis);
    painter.text(
        Pos2::new(rect.center().x, outer.bottom() - 8.0),
        Align2::CENTER_BOTTOM,
        xlabel,
        semibold(12.5),
        text_soft(),
    );
    painter.text(
        Pos2::new(outer.left() + 10.0, outer.top() + 8.0),
        Align2::LEFT_TOP,
        ylabel,
        semibold(12.5),
        text_soft(),
    );
    (painter.with_clip_rect(outer), plot, resp)
}

fn short(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
