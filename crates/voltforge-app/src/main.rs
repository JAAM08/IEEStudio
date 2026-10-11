//! VoltForge: aplicación de escritorio.
//!
//! Uso: `voltforge-app [proyecto.json] [--rapido] [--claro] [--captura imagen.png]`

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod quick_view;
mod theme;

use std::path::PathBuf;

fn main() -> eframe::Result {
    let mut open: Option<PathBuf> = None;
    let mut capture: Option<PathBuf> = None;
    let mut light = false;
    let mut quick = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--claro" => light = true,
            "--rapido" => quick = true,
            "--captura" => capture = args.next().map(PathBuf::from),
            _ => open = Some(PathBuf::from(arg)),
        }
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VoltForge")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "VoltForge",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, open, capture, light, quick)))),
    )
}
