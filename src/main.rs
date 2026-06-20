#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod keyboard;
mod mouse;
mod panels;

fn load_icon() -> egui::IconData {
    let bytes = include_bytes!("../icons/icon.png");
    let image = image::load_from_memory(bytes)
        .expect("embedded icon.png should decode")
        .to_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 580.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Trouble",
        options,
        Box::new(|cc| Ok(Box::new(app::TroubleApp::new(cc)))),
    )
}
