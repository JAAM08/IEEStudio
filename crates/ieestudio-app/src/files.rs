//! Abrir y guardar archivos, en escritorio y en el navegador.
//!
//! En escritorio se usan los cuadros de diálogo del sistema y el archivo se
//! escribe donde el usuario dice. En el navegador no hay carpetas: abrir es
//! elegir un archivo con el selector del navegador, que llega un momento
//! después, y guardar es descargarlo. Para que el resto del programa no tenga
//! que distinguir, lo que se abre siempre llega por [`Inbox`].

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Para qué se eligió un archivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    Project,
    Excel,
}

/// Un archivo que el usuario eligió abrir.
pub struct Picked {
    pub purpose: Purpose,
    pub name: String,
    /// Ruta en disco; en el navegador no la hay.
    pub path: Option<PathBuf>,
    /// Contenido, o el error al leerlo.
    pub bytes: Result<Vec<u8>, String>,
}

/// Buzón donde llegan los archivos elegidos. En el navegador se llena después
/// de que el usuario cierra el selector.
#[derive(Clone, Default)]
pub struct Inbox(Arc<Mutex<Vec<Picked>>>);

impl Inbox {
    pub fn take(&self) -> Vec<Picked> {
        self.0
            .lock()
            .map(|mut v| v.drain(..).collect())
            .unwrap_or_default()
    }

    fn put(&self, picked: Picked) {
        if let Ok(mut v) = self.0.lock() {
            v.push(picked);
        }
    }
}

/// Filtro de tipo de archivo para los selectores.
pub struct Filter<'a> {
    /// Nombre del tipo; el selector del navegador no lo muestra.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub name: &'a str,
    pub extensions: &'a [&'a str],
}

/// Si el programa corre en el navegador.
pub const WEB: bool = cfg!(target_arch = "wasm32");

/// Pide al usuario un archivo para abrir. Lo que elija llega a `inbox`.
pub fn pick(inbox: &Inbox, purpose: Purpose, filter: Filter<'_>, ctx: &eframe::egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ctx;
        let Some(path) = rfd::FileDialog::new()
            .add_filter(filter.name, filter.extensions)
            .pick_file()
        else {
            return;
        };
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bytes =
            std::fs::read(&path).map_err(|e| format!("No se pudo leer {}: {e}", path.display()));
        inbox.put(Picked {
            purpose,
            name,
            path: Some(path),
            bytes,
        });
    }
    #[cfg(target_arch = "wasm32")]
    if let Err(e) = pick_web(inbox, purpose, filter, ctx) {
        inbox.put(Picked {
            purpose,
            name: String::new(),
            path: None,
            bytes: Err(e),
        });
    }
}

/// Abre el selector de archivos del navegador con un `<input type="file">`.
#[cfg(target_arch = "wasm32")]
fn pick_web(
    inbox: &Inbox,
    purpose: Purpose,
    filter: Filter<'_>,
    ctx: &eframe::egui::Context,
) -> Result<(), String> {
    use wasm_bindgen::{closure::Closure, JsCast};
    let fail = || "El navegador no permitió abrir el selector de archivos.".to_string();
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(fail)?;
    let input: web_sys::HtmlInputElement = document
        .create_element("input")
        .map_err(|_| fail())?
        .dyn_into()
        .map_err(|_| fail())?;
    input.set_type("file");
    let accept: Vec<String> = filter.extensions.iter().map(|e| format!(".{e}")).collect();
    input.set_accept(&accept.join(","));
    let (inbox, ctx, source) = (inbox.clone(), ctx.clone(), input.clone());
    let on_change = Closure::once_into_js(move || {
        let Some(file) = source.files().and_then(|f| f.get(0)) else {
            return;
        };
        let name = file.name();
        let reading = wasm_bindgen_futures::JsFuture::from(file.array_buffer());
        wasm_bindgen_futures::spawn_local(async move {
            let bytes = match reading.await {
                Ok(buffer) => Ok(js_sys::Uint8Array::new(&buffer).to_vec()),
                Err(_) => Err(format!("No se pudo leer {name}.")),
            };
            inbox.put(Picked {
                purpose,
                name,
                path: None,
                bytes,
            });
            ctx.request_repaint();
        });
    });
    input.set_onchange(Some(on_change.unchecked_ref()));
    input.click();
    Ok(())
}

/// Dónde quedó un archivo guardado, para avisarle al usuario. Cada plataforma
/// usa solo una de las dos formas.
#[allow(dead_code)]
pub enum Saved {
    /// Escrito en esta ruta.
    Path(PathBuf),
    /// Descargado por el navegador con este nombre.
    Download(String),
}

impl Saved {
    pub fn describe(&self) -> String {
        match self {
            Saved::Path(p) => format!("en {}", p.display()),
            Saved::Download(name) => format!("como descarga: {name}"),
        }
    }
}

/// Guarda `bytes`. En escritorio pregunta dónde, salvo que se dé `path`; en el
/// navegador lo descarga con `name`. `Ok(None)` si el usuario canceló.
pub fn save(
    bytes: &[u8],
    name: &str,
    filter: Filter<'_>,
    path: Option<PathBuf>,
) -> Result<Option<Saved>, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let target = match path {
            Some(p) => Some(p),
            None => rfd::FileDialog::new()
                .add_filter(filter.name, filter.extensions)
                .set_file_name(name)
                .save_file(),
        };
        let Some(target) = target else {
            return Ok(None);
        };
        std::fs::write(&target, bytes)
            .map_err(|e| format!("No se pudo guardar {}: {e}", target.display()))?;
        Ok(Some(Saved::Path(target)))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (filter, path);
        let name = ascii_name(name);
        download(bytes, &name)?;
        Ok(Some(Saved::Download(name)))
    }
}

/// Hace que el navegador descargue `bytes` con el nombre `name`.
#[cfg(target_arch = "wasm32")]
fn download(bytes: &[u8], name: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    let fail = |what: &str| format!("El navegador no permitió descargar el archivo ({what}).");
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts).map_err(|_| fail("Blob"))?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|_| fail("URL"))?;
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| fail("documento"))?;
    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| fail("enlace"))?
        .dyn_into()
        .map_err(|_| fail("enlace"))?;
    anchor.set_href(&url);
    anchor.set_download(name);
    anchor.click();
    // La URL no se revoca: algunos navegadores cancelan la descarga si se
    // revoca enseguida, y es memoria de una sola descarga.
    Ok(())
}

/// Nombre de descarga sin acentos: algunos navegadores descartan el nombre
/// completo si tiene caracteres fuera de ASCII y descargan «download».
#[cfg(target_arch = "wasm32")]
fn ascii_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'Á' | 'À' | 'Ä' | 'Â' => 'A',
            'É' | 'È' | 'Ë' | 'Ê' => 'E',
            'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
            'Ó' | 'Ò' | 'Ö' | 'Ô' => 'O',
            'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
            'ñ' => 'n',
            'Ñ' => 'N',
            c if c.is_ascii() => c,
            _ => '_',
        })
        .collect()
}
