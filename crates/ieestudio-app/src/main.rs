//! IEEStudio: aplicación de escritorio.
//!
//! Uso: `ieestudio-app [proyecto.json] [--claro] [--espacio calculo|diagrama|documentacion]
//! [--panel rapido|unifilar|memoria|ninguno] [--seleccion ID|proyecto] [--captura imagen.png]`

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
        select: None,
        properties: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--claro" => options.light = true,
            "--propiedades" => options.properties = args.next(),
            "--seleccion" => options.select = args.next(),
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

    install_panic_report();

    // Primero wgpu (DirectX 12, Vulkan u OpenGL); si la tarjeta de video o su
    // controlador no lo admiten, OpenGL directo. Si ninguno arranca, se avisa
    // en pantalla en lugar de cerrar sin decir nada.
    let mut errors = Vec::new();
    for renderer in [eframe::Renderer::Wgpu, eframe::Renderer::Glow] {
        let options = options.clone();
        match eframe::run_native(
            "IEEStudio",
            native_options(renderer),
            Box::new(move |cc| Ok(Box::new(app::App::new(cc, options)))),
        ) {
            Ok(()) => return Ok(()),
            Err(e) => errors.push(format!("{renderer:?}: {e}")),
        }
    }
    report_failure(&format!(
        "IEEStudio no pudo abrir su ventana. Normalmente es el controlador de \
         la tarjeta de video: actualízalo desde Windows Update o la página del \
         fabricante.\n\nDetalle:\n{}",
        errors.join("\n")
    ));
    std::process::exit(1);
}

fn native_options(renderer: eframe::Renderer) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("IEEStudio")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 640.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../../assets/icon/ieestudio-256.png"
                ))
                .unwrap_or_default(),
            ),
        ..Default::default()
    }
}

/// El programa de Windows no tiene consola: sin esto, una falla al arrancar lo
/// cierra sin mensaje. Así queda escrita y se muestra.
fn install_panic_report() {
    std::panic::set_hook(Box::new(|info| {
        report_failure(&format!(
            "IEEStudio se cerró por un error interno. Envía este mensaje o el \
             archivo de registro a quien mantiene el programa.\n\n{info}"
        ));
    }));
}

/// Escribe el error en `registro-errores.txt` (en la carpeta de datos del
/// usuario) y lo muestra en una ventana de aviso.
fn report_failure(message: &str) {
    let dir = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("IEEStudio");
    let log = dir.join("registro-errores.txt");
    let written = std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&log, message))
        .is_ok();
    eprintln!("{message}");
    let text = if written {
        format!("{message}\n\nQuedó guardado en:\n{}", log.display())
    } else {
        message.to_string()
    };
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("IEEStudio")
        .set_description(text)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
