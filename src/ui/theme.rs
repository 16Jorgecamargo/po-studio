use super::motion;
use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

static DARK: AtomicBool = AtomicBool::new(false);
static MIX: AtomicU32 = AtomicU32::new(0);

const THEME_SECONDS: f32 = 0.26;

pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

fn mix() -> f32 {
    f32::from_bits(MIX.load(Ordering::Relaxed))
}

fn pick(light: Color32, dark: Color32) -> Color32 {
    let t = mix();
    if t <= 0.0 {
        light
    } else if t >= 1.0 {
        dark
    } else {
        light.lerp_to_gamma(dark, motion::ease_in_out_cubic(t))
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub const PIVOT: Color32 = rgb(242, 194, 48);
pub const PIVOT_INK: Color32 = rgb(16, 48, 43);

pub fn bg() -> Color32 {
    pick(rgb(241, 245, 240), rgb(15, 31, 28))
}

pub fn card_fill() -> Color32 {
    pick(rgb(251, 252, 250), rgb(21, 41, 37))
}

pub fn sidebar_fill() -> Color32 {
    pick(rgb(229, 236, 228), rgb(11, 24, 22))
}

pub fn input_fill() -> Color32 {
    pick(rgb(255, 255, 255), rgb(12, 25, 23))
}

pub fn button_fill() -> Color32 {
    pick(rgb(227, 234, 227), rgb(33, 58, 53))
}

pub fn button_hover_fill() -> Color32 {
    pick(rgb(213, 223, 215), rgb(43, 74, 67))
}

pub fn button_active_fill() -> Color32 {
    pick(rgb(195, 207, 198), rgb(53, 87, 79))
}

pub fn border() -> Color32 {
    pick(rgb(195, 207, 198), rgb(43, 69, 63))
}

pub fn control_line() -> Color32 {
    pick(rgb(128, 146, 138), rgb(92, 118, 110))
}

pub fn text_strong() -> Color32 {
    pick(rgb(16, 48, 43), rgb(240, 245, 241))
}

pub fn text_body() -> Color32 {
    pick(rgb(31, 59, 54), rgb(213, 224, 217))
}

pub fn text_soft() -> Color32 {
    pick(rgb(63, 90, 84), rgb(169, 187, 179))
}

pub fn text_muted() -> Color32 {
    pick(rgb(86, 109, 103), rgb(140, 161, 152))
}

pub fn accent() -> Color32 {
    pick(rgb(15, 107, 92), rgb(111, 211, 189))
}

pub fn red() -> Color32 {
    pick(rgb(179, 38, 30), rgb(255, 155, 143))
}

pub fn green() -> Color32 {
    pick(rgb(27, 122, 67), rgb(127, 220, 160))
}

pub fn neutral_mark() -> Color32 {
    pick(rgb(92, 106, 100), rgb(160, 174, 167))
}

pub fn accent_soft() -> Color32 {
    pick(rgb(220, 232, 225), rgb(29, 58, 52))
}

pub fn green_soft() -> Color32 {
    pick(rgb(207, 235, 214), rgb(28, 74, 51))
}

pub fn red_soft() -> Color32 {
    pick(rgb(248, 215, 210), rgb(90, 38, 34))
}

pub fn green_text() -> Color32 {
    pick(rgb(20, 90, 50), rgb(155, 229, 181))
}

pub fn red_text() -> Color32 {
    pick(rgb(140, 28, 19), rgb(255, 179, 168))
}

pub fn info_text() -> Color32 {
    text_body()
}

pub fn warn_text() -> Color32 {
    pick(rgb(122, 74, 0), rgb(242, 200, 122))
}

pub fn grid_line() -> Color32 {
    pick(rgb(225, 232, 226), rgb(32, 53, 47))
}

pub fn axis_line() -> Color32 {
    text_soft()
}

pub fn band_fill() -> Color32 {
    pick(rgb(213, 223, 215), rgb(43, 69, 63))
}

pub fn edge_line() -> Color32 {
    pick(rgb(155, 171, 162), rgb(92, 118, 110))
}

pub fn node_fill() -> Color32 {
    pick(rgb(44, 74, 68), rgb(61, 95, 87))
}

pub fn label_backing() -> Color32 {
    card_fill().gamma_multiply(0.92)
}

pub fn highlight_text() -> Color32 {
    red()
}

pub fn envelope_line() -> Color32 {
    text_strong().gamma_multiply(0.5)
}

pub fn series(i: usize) -> Color32 {
    const LIGHT: [Color32; 8] = [
        rgb(15, 107, 92),
        rgb(180, 71, 43),
        rgb(59, 75, 168),
        rgb(107, 106, 18),
        rgb(138, 47, 107),
        rgb(74, 90, 99),
        rgb(30, 111, 158),
        rgb(122, 74, 0),
    ];
    const DARK_SET: [Color32; 8] = [
        rgb(111, 211, 189),
        rgb(240, 138, 106),
        rgb(154, 168, 245),
        rgb(210, 207, 90),
        rgb(228, 138, 198),
        rgb(169, 186, 194),
        rgb(111, 192, 238),
        rgb(242, 200, 122),
    ];
    pick(LIGHT[i % 8], DARK_SET[i % 8])
}

pub fn on_color(fill: Color32) -> Color32 {
    let lin = |v: u8| {
        let c = v as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let l = 0.2126 * lin(fill.r()) + 0.7152 * lin(fill.g()) + 0.0722 * lin(fill.b());
    if l > 0.3 {
        PIVOT_INK
    } else {
        rgb(247, 250, 247)
    }
}

pub fn sans(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn mono_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("mono_bold".into()))
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let files: [(&str, &'static [u8]); 4] = [
        ("plex_sans", include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf")),
        ("plex_sans_semibold", include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf")),
        ("plex_mono", include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf")),
        ("plex_mono_semibold", include_bytes!("../../assets/fonts/IBMPlexMono-SemiBold.ttf")),
    ];
    for (name, bytes) in files {
        fonts.font_data.insert(name.to_owned(), Arc::new(egui::FontData::from_static(bytes)));
    }
    let mut sans_fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    sans_fallback.insert(0, "Hack".to_owned());
    let mono_fallback = fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let with = |first: &str, rest: &[String]| -> Vec<String> {
        let mut v = vec![first.to_owned()];
        v.extend(rest.iter().cloned());
        v
    };
    fonts.families.insert(FontFamily::Proportional, with("plex_sans", &sans_fallback));
    fonts.families.insert(FontFamily::Name("semibold".into()), with("plex_sans_semibold", &sans_fallback));
    fonts.families.insert(FontFamily::Monospace, with("plex_mono", &mono_fallback));
    fonts.families.insert(FontFamily::Name("mono_bold".into()), with("plex_mono_semibold", &mono_fallback));
    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    DARK.store(dark, Ordering::Relaxed);
    MIX.store((if dark { 1.0f32 } else { 0.0 }).to_bits(), Ordering::Relaxed);
    restyle(ctx);
}

pub fn toggle_theme(ctx: &egui::Context, dark: bool) {
    if motion::reduced() {
        apply_theme(ctx, dark);
        return;
    }
    DARK.store(dark, Ordering::Relaxed);
    restyle(ctx);
    ctx.request_repaint();
}

pub fn tick_theme(ctx: &egui::Context) {
    let target = if is_dark() { 1.0 } else { 0.0 };
    let current = mix();
    if (current - target).abs() < f32::EPSILON {
        return;
    }
    let step = ctx.input(|i| i.stable_dt).min(0.05) / THEME_SECONDS;
    let next = if target > current { (current + step).min(1.0) } else { (current - step).max(0.0) };
    MIX.store(next.to_bits(), Ordering::Relaxed);
    restyle(ctx);
    ctx.request_repaint();
}

fn restyle(ctx: &egui::Context) {
    let dark = is_dark();
    let theme = if dark { egui::Theme::Dark } else { egui::Theme::Light };
    ctx.set_theme(theme);
    ctx.style_mut_of(theme, |s| {
        s.visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        s.text_styles = [
            (TextStyle::Small, sans(12.0)),
            (TextStyle::Body, sans(14.0)),
            (TextStyle::Button, sans(14.0)),
            (TextStyle::Heading, semibold(22.0)),
            (TextStyle::Monospace, mono(13.5)),
        ]
        .into();
        s.animation_time = if motion::reduced() { 0.0 } else { 0.12 };
        s.spacing.item_spacing = Vec2::new(8.0, 6.0);
        s.spacing.button_padding = Vec2::new(10.0, 5.0);
        let v = &mut s.visuals;
        v.panel_fill = bg();
        v.window_fill = card_fill();
        v.window_stroke = Stroke::new(1.0, border());
        v.window_corner_radius = CornerRadius::same(4);
        v.menu_corner_radius = CornerRadius::same(4);
        v.override_text_color = Some(text_body());
        v.weak_text_color = Some(text_muted());
        v.extreme_bg_color = input_fill();
        v.text_edit_bg_color = Some(input_fill());
        v.faint_bg_color = grid_line();
        v.code_bg_color = accent_soft();
        v.hyperlink_color = accent();
        v.warn_fg_color = warn_text();
        v.error_fg_color = red();
        v.selection.bg_fill = Color32::from_rgba_unmultiplied(242, 194, 48, 120);
        v.selection.stroke = Stroke::new(1.5, accent());
        let radius = CornerRadius::same(3);
        v.widgets.noninteractive.corner_radius = radius;
        v.widgets.noninteractive.bg_fill = card_fill();
        v.widgets.noninteractive.weak_bg_fill = card_fill();
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_body());
        v.widgets.inactive.corner_radius = radius;
        v.widgets.inactive.bg_fill = button_fill();
        v.widgets.inactive.weak_bg_fill = button_fill();
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, control_line());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, text_body());
        v.widgets.inactive.expansion = 0.0;
        v.widgets.hovered.corner_radius = radius;
        v.widgets.hovered.bg_fill = button_hover_fill();
        v.widgets.hovered.weak_bg_fill = button_hover_fill();
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, text_soft());
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, text_strong());
        v.widgets.hovered.expansion = 0.0;
        v.widgets.active.corner_radius = radius;
        v.widgets.active.bg_fill = button_active_fill();
        v.widgets.active.weak_bg_fill = button_active_fill();
        v.widgets.active.bg_stroke = Stroke::new(2.0, accent());
        v.widgets.active.fg_stroke = Stroke::new(1.5, text_strong());
        v.widgets.active.expansion = 0.0;
        v.widgets.open.corner_radius = radius;
        v.widgets.open.bg_fill = button_hover_fill();
        v.widgets.open.weak_bg_fill = button_hover_fill();
        v.widgets.open.bg_stroke = Stroke::new(1.0, text_soft());
        v.widgets.open.fg_stroke = Stroke::new(1.0, text_strong());
    });
}
