#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;
mod format;
mod game;
mod ip;
mod legacy;
mod linalg;
mod lp;
mod mechanics;
mod network;
mod project;
mod queue;
mod simplex;
mod transport;
mod ui;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let screenshot = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let shot = screenshot.is_some();
    let demo = args
        .iter()
        .position(|a| a == "--demo")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let forced_dark = if args.iter().any(|a| a == "--dark") {
        Some(true)
    } else if args.iter().any(|a| a == "--light") {
        Some(false)
    } else {
        None
    };
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(app::APP_NAME)
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([760.0, 520.0])
        .with_app_id("po-studio");
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    if shot {
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(eframe::egui::WindowLevel::AlwaysOnBottom);
        }
    }
    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions {
        viewport,
        run_and_return: false,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    if shot {
        options.event_loop_builder = Some(Box::new(|builder| {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            builder.with_activate_ignoring_other_apps(false);
        }));
    }
    eframe::run_native(
        app::APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::OrApp::new(cc, screenshot, demo, forced_dark)))),
    )
}

fn window_icon() -> Option<eframe::egui::IconData> {
    let bytes = include_bytes!("../assets/icon/icon_256.png");
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(eframe::egui::IconData { rgba: img.into_raw(), width, height })
}
