//! VoltForge: aplicación de escritorio.
//!
//! Uso: `voltforge-app [proyecto.json] [--claro] [--espacio calculo|diagrama|documentacion]
//! [--panel rapido|unifilar|memoria|ninguno] [--captura imagen.png]`

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod panels;
mod state;
mod theme;
mod widgets;
mod workspace;

use std::path::PathBuf;

use workspace::{Tab, Workspace};

fn main() -> eframe::Result {
    let mut options = app::Options {
        open: None,
        capture: None,
        light: false,
        workspace: None,
        focus: None,
        no_panels: false,
        properties: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--claro" => options.light = true,
            "--propiedades" => options.properties = args.next(),
            "--captura" => options.capture = args.next().map(PathBuf::from),
            "--espacio" => {
                options.workspace = match args.next().as_deref() {
                    Some("diagrama") => Some(Workspace::Diagram),
                    Some("documentacion") => Some(Workspace::Documentation),
                    _ => Some(Workspace::Calculation),
                }
            }
            "--panel" => {
                options.focus = match args.next().as_deref() {
                    Some("rapido") => Some(Tab::Quick),
                    Some("unifilar") => Some(Tab::Diagram),
                    Some("memoria") => Some(Tab::Report),
                    Some("ninguno") => {
                        options.no_panels = true;
                        None
                    }
                    _ => None,
                }
            }
            // Compatibilidad con las opciones anteriores.
            "--rapido" => options.focus = Some(Tab::Quick),
            "--unifilar" => options.focus = Some(Tab::Diagram),
            _ => options.open = Some(PathBuf::from(arg)),
        }
    }

    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VoltForge")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 640.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../../assets/icon/voltforge-256.png"
                ))
                .unwrap_or_default(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "VoltForge",
        native,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, options)))),
    )
}
