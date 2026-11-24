//! IEEStudio: aplicación de escritorio y, compilada a WebAssembly, en el
//! navegador.
//!
//! Uso en escritorio: `ieestudio-app [proyecto.iees] [--claro]
//! [--vista proyecto|tableros|unifilar|revisar|memoria] [--seleccion ID|proyecto] [--ficha]
//! [--procedimiento] [--dividir] [--propiedades datos|resultados|normativa] [--inicio]
//! [--rapido] [--comandos] [--agregar] [--tamano 1024x768] [--captura imagen.png] [--medir 60]`

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod chooser;
mod commands;
mod compact;
mod files;
mod home;
mod layout;
mod panels;
mod shell;
mod state;
mod symbols;
mod theme;
mod views;
mod widgets;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use views::View;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let mut options = app::Options {
        open: None,
        capture: None,
        light: false,
        select: None,
        properties: None,
        home: false,
        palette: false,
        view: None,
        procedure: false,
        split: false,
        quick: false,
        ficha: false,
        chooser: false,
        measure: 0,
    };
    let mut size = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--claro" => options.light = true,
            "--propiedades" => options.properties = args.next(),
            "--seleccion" => options.select = args.next(),
            "--inicio" => options.home = true,
            "--comandos" => options.palette = true,
            "--procedimiento" => options.procedure = true,
            "--dividir" => options.split = true,
            "--agregar" => options.chooser = true,
            // Tamaño de la ventana para revisar pantallas angostas: 1024x768.
            "--tamano" => {
                size = args.next().and_then(|v| {
                    let (w, h) = v.split_once('x')?;
                    Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?])
                });
            }
            "--vista" => options.view = args.next().as_deref().and_then(view_named),
            "--captura" => options.capture = args.next().map(PathBuf::from),
            // Con --captura: mide cuánto tarda la interfaz en armar N cuadros
            // más antes de tomar la imagen, e imprime los tiempos.
            "--medir" => options.measure = args.next().and_then(|v| v.parse().ok()).unwrap_or(60),
            "--rapido" => options.quick = true,
            "--ficha" => options.ficha = true,
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
            native_options(renderer, size),
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

/// La vista que nombra una opción de línea de comandos.
#[cfg(not(target_arch = "wasm32"))]
fn view_named(name: &str) -> Option<View> {
    match name {
        "proyecto" => Some(View::Project),
        "tableros" | "circuitos" | "cuadro" => Some(View::Schedule),
        "unifilar" => Some(View::Diagram),
        "revisar" => Some(View::Review),
        "memoria" => Some(View::Report),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_options(renderer: eframe::Renderer, size: Option<[f32; 2]>) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("IEEStudio")
            .with_inner_size(size.unwrap_or([1440.0, 900.0]))
            .with_min_inner_size(
                size.map_or([1024.0, 640.0], |s| [s[0].min(1024.0), s[1].min(640.0)]),
            )
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../../assets/icon/ieestudio-256.png"
                ))
                .unwrap_or_default(),
            ),
        // Con un tamaño pedido no se repone el de la última sesión.
        persist_window: size.is_none(),
        ..Default::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

/// En el navegador: arranca en el lienzo `ieestudio` de `index.html`.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    std::panic::set_hook(Box::new(|info| {
        web_status(&format!(
            "IEEStudio se detuvo por un error interno. Recarga la página; tu \
             proyecto quedó guardado en este navegador.\n\n{info}"
        ));
    }));
    let options = app::Options {
        open: None,
        capture: None,
        light: false,
        select: None,
        properties: None,
        home: false,
        palette: false,
        view: None,
        procedure: false,
        split: false,
        quick: false,
        ficha: false,
        chooser: false,
        measure: 0,
    };
    wasm_bindgen_futures::spawn_local(async move {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("ieestudio"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
        let Some(canvas) = canvas else {
            web_status("Falta el lienzo de la aplicación en la página.");
            return;
        };
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| Ok(Box::new(app::App::new(cc, options)))),
            )
            .await;
        match started {
            Ok(()) => web_status(""),
            Err(e) => web_status(&format!(
                "Este navegador no pudo iniciar IEEStudio. Prueba con una versión \
                 reciente de Chrome, Edge o Firefox.\n\n{e:?}"
            )),
        }
    });
}

/// Muestra un mensaje en la página, sobre el lienzo; vacío lo oculta.
#[cfg(target_arch = "wasm32")]
fn web_status(message: &str) {
    let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("estado"))
    else {
        return;
    };
    el.set_text_content(Some(message));
    let _ = el.set_attribute("data-visible", if message.is_empty() { "no" } else { "si" });
}
