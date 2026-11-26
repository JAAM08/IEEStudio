//! Ventana principal: barra superior con las cinco vistas, regiones fijas y
//! barra de estado alrededor de un único proyecto.

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};
use egui_phosphor::regular as icon;
use ieestudio_project::{example, Project};
use serde::{Deserialize, Serialize};

use crate::chooser;
use crate::commands::{Command, CommandPalette};
use crate::compact;
use crate::files::{self, Filter, Inbox, Picked, Purpose, Saved};
use crate::home::{self, Start};
use crate::layout::{self, Class};
use crate::panels::{schedule, Shared};
use crate::shell::{self, Arrangement};
use crate::state::{empty_project, Doc, Selection};
use crate::symbols::Symbol;
use crate::theme::{self, Palette};
use crate::views::{Export, View};
use crate::widgets::{muted, strong, verdict_chips};

const STORAGE_KEY: &str = "ieestudio-interfaz-1";
/// Ancho de los botones de la derecha de la barra superior (cálculo rápido,
/// dos vistas, buscar y tema).
const RIGHT_TOOLS: f32 = 190.0;
/// Ancho aproximado de las cinco vistas con su nombre y reducidas a íconos.
const FULL_VIEWS: f32 = 600.0;
const COMPACT_VIEWS: f32 = 320.0;
/// En el navegador, el proyecto abierto se guarda solo con esta clave, para
/// no perder trabajo si se cierra la pestaña. Una clave por formato desde el
/// 4: el 5 agrega la topología declarada (decisión 0030) y el 6 deja sin
/// capturar la potencia y la longitud (decisión 0033).
const WEB_PROJECT_KEY: &str = "ieestudio-proyecto-web-6";
/// Donde lo guardaban las versiones anteriores, de la más nueva a la más
/// vieja. Se leen si la clave nueva está vacía y nunca se escriben: una
/// versión anterior que siga en la caché del navegador no lee el formato
/// nuevo y pondría un proyecto vacío en su lugar.
const WEB_PROJECT_OLD_KEYS: [&str; 3] = [
    "ieestudio-proyecto-web-5",
    "ieestudio-proyecto-web-4",
    "ieestudio-proyecto-web",
];
/// Copia de un proyecto guardado que esta versión no pudo leer, para no
/// perderlo al guardar el que se abre en su lugar.
const WEB_PROJECT_UNREADABLE_KEY: &str = "ieestudio-proyecto-web-no-leido";

const EXCEL_FILTER: Filter<'static> = Filter {
    name: "Libro de Excel",
    extensions: &["xlsx"],
};

const PROJECT_FILTER: Filter<'static> = Filter {
    name: "Proyecto de IEEStudio",
    extensions: &[EXTENSION, "vfp", "json"],
};

/// Preferencias de interfaz que se guardan al cerrar.
#[derive(Serialize, Deserialize)]
struct Prefs {
    dark: bool,
    columns: schedule::Columns,
    /// Juego de columnas del cuadro.
    #[serde(default)]
    column_set: schedule::ColumnSet,
    diagram_zoom: f32,
    /// Proyectos abiertos o guardados hace poco; el primero se abre al
    /// arrancar.
    #[serde(default)]
    recent: Vec<PathBuf>,
    /// Vista en la que se quedó. Las preferencias de versiones anteriores
    /// traían el modo guiado o avanzado, el acomodo de los paneles y el paso;
    /// ya no existen y se ignoran.
    #[serde(default)]
    view: View,
    /// Dos vistas a la vez: el cuadro y el unifilar.
    #[serde(default)]
    split: bool,
}

/// Cuántos proyectos recientes se recuerdan.
const MAX_RECENT: usize = 8;

/// Extensión de los proyectos. Por dentro es JSON; la extensión propia
/// permite abrirlos con doble clic sin adueñarse de todos los `.json`.
pub const EXTENSION: &str = "iees";

/// Acción que espera a que el usuario decida qué hacer con los cambios sin
/// guardar.
#[derive(Debug, Clone, PartialEq)]
enum Pending {
    New,
    Open,
    OpenPath(PathBuf),
    Example,
    Quit,
}

/// Captura de pantalla pedida por línea de comandos, para pruebas visuales.
struct Capture {
    path: PathBuf,
    frames: u32,
    /// Cuadros que se miden antes de tomar la imagen (`--medir`).
    measure: u32,
    /// Lo que tardó `App::ui` en cada cuadro, en milisegundos.
    times: Vec<f64>,
}

pub struct App {
    doc: Doc,
    shared: Shared,
    dark: bool,
    notice: Option<String>,
    /// En el navegador, el proyecto guardado que no se pudo leer: se copia a
    /// `WEB_PROJECT_UNREADABLE_KEY` antes de guardar otro encima.
    unreadable: Option<String>,
    done: Option<String>,
    about: bool,
    capture: Option<Capture>,
    /// Acción detenida por cambios sin guardar.
    pending: Option<Pending>,
    /// Resultado de la última importación de Excel, con el proyecto de antes
    /// para poder deshacerla.
    imported: Option<(ieestudio_exchange::ImportReport, Project)>,
    /// Importación de Excel en vista previa: lo que haría, sin aplicarlo
    /// todavía (paso 2 de 3).
    import_preview: Option<ImportPreview>,
    /// El usuario ya decidió cerrar; no se vuelve a preguntar.
    allow_close: bool,
    /// Cerrar la ventana al terminar este cuadro.
    quit: bool,
    recent: Vec<PathBuf>,
    /// Archivos elegidos para abrir; en el navegador llegan un momento después.
    inbox: Inbox,
    ctx: egui::Context,
    /// Paleta de comandos (Ctrl+K).
    commands: CommandPalette,
    /// Pantalla de inicio en lugar de los paneles: al arrancar sin nada que
    /// abrir, o cuando el usuario la pide.
    home: bool,
    /// Vista activa del marco.
    view: View,
    /// Dos vistas a la vez, si la ventana es lo bastante ancha.
    split: bool,
    /// Panel de procedimiento abierto abajo.
    procedure: bool,
    /// Cálculo rápido como ventana flotante, dentro de un proyecto.
    quick_window: bool,
    /// La experiencia «Cálculo rápido» en su propia pantalla, sin proyecto.
    quick_screen: bool,
    /// La pantalla se tocó: controles táctiles aunque la ventana sea ancha.
    touch: bool,
    /// Clase de tamaño de la ventana en el último cuadro.
    class: Class,
    /// Ventana «Personalizar columnas».
    columns_window: bool,
}

#[derive(Clone)]
pub struct Options {
    pub open: Option<PathBuf>,
    pub capture: Option<PathBuf>,
    pub light: bool,
    /// Circuito o tablero que se selecciona al abrir, por su ID.
    pub select: Option<String>,
    /// Pestaña inicial de Propiedades: `datos`, `resultados` o `normativa`.
    pub properties: Option<String>,
    /// Empieza en la pantalla de inicio, para revisarla.
    pub home: bool,
    /// Empieza con la paleta de comandos abierta, para revisarla.
    pub palette: bool,
    /// Vista en la que se empieza (`--vista`).
    pub view: Option<View>,
    /// Empieza con el panel de procedimiento abierto.
    pub procedure: bool,
    /// Empieza con dos vistas a la vez.
    pub split: bool,
    /// Empieza en la experiencia «Cálculo rápido».
    pub quick: bool,
    /// Empieza con la ficha del elemento elegido abierta: a pantalla completa
    /// en el teléfono, en su cajón en la tableta.
    pub ficha: bool,
    /// Empieza con la ventana «¿Qué vas a conectar?» abierta, para revisarla.
    pub chooser: bool,
    /// Con `capture`: cuadros que se miden antes de tomar la imagen.
    pub measure: u32,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, options: Options) -> Self {
        let capturing = options.capture.is_some();
        theme::install_fonts(&cc.egui_ctx);
        // Las capturas de prueba ignoran lo guardado para ser reproducibles.
        let prefs: Option<Prefs> = if options.capture.is_some() {
            None
        } else {
            cc.storage
                .and_then(|s| s.get_string(STORAGE_KEY))
                .and_then(|text| serde_json::from_str(&text).ok())
        };
        let recent = prefs.as_ref().map(|p| p.recent.clone()).unwrap_or_default();
        let saved_view = prefs.as_ref().map(|p| p.view);
        let saved_split = prefs.as_ref().is_some_and(|p| p.split);
        let (dark, columns, column_set, zoom) = match prefs {
            Some(p) => (p.dark, p.columns, p.column_set, p.diagram_zoom),
            None => (
                !options.light,
                schedule::Columns::default(),
                schedule::ColumnSet::default(),
                1.3,
            ),
        };
        let palette = Palette::new(dark);
        theme::apply(&cc.egui_ctx, &palette);
        let mut shared = Shared::new(palette);
        shared.columns = columns;
        shared.column_set = column_set;
        shared.diagram_zoom = zoom;
        let mut app = App {
            doc: Doc::new(example::house(), None),
            shared,
            dark,
            notice: None,
            unreadable: None,
            done: None,
            about: false,
            capture: options.capture.map(|path| Capture {
                path,
                frames: 0,
                measure: options.measure,
                times: Vec::new(),
            }),
            pending: None,
            imported: None,
            import_preview: None,
            allow_close: false,
            quit: false,
            recent,
            inbox: Inbox::default(),
            ctx: cc.egui_ctx.clone(),
            commands: CommandPalette::default(),
            home: false,
            view: options.view.or(saved_view).unwrap_or_default(),
            split: options.split || saved_split,
            procedure: options.procedure,
            quick_window: false,
            quick_screen: options.quick,
            touch: false,
            class: Class::Desktop,
            columns_window: false,
        };
        app.doc.selection = Selection::Circuit(0, 1);
        // Abre lo que se pidió; si no, el último proyecto, si todavía existe.
        let last = app.recent.first().filter(|p| p.exists()).cloned();
        let mut resumed = false;
        if let Some(path) = options.open.or(last) {
            app.load(path);
            resumed = true;
        } else if files::WEB {
            // En el navegador se retoma el proyecto en el que se trabajaba:
            // el del formato actual o, si no hay, el de una versión anterior.
            let saved = cc.storage.and_then(|s| {
                s.get_string(WEB_PROJECT_KEY).or_else(|| {
                    WEB_PROJECT_OLD_KEYS
                        .iter()
                        .find_map(|key| s.get_string(key))
                })
            });
            if let Some(text) = saved {
                match Project::from_json(&text) {
                    Ok(project) => {
                        app.set_project(project, None);
                        resumed = true;
                    }
                    // Nunca se pierde: se guarda una copia antes de escribir
                    // otro proyecto en su lugar.
                    Err(why) => {
                        app.unreadable = Some(text);
                        app.notice = Some(format!(
                            "No se pudo abrir el proyecto guardado en este navegador ({why}). Se conserva una copia; actualiza la página para usar la versión más reciente."
                        ));
                    }
                }
            }
        }
        // Sin nada que retomar se empieza en la pantalla de inicio; las
        // capturas de prueba solo la muestran si se pide.
        app.home = options.home || (!resumed && !capturing && !options.quick);
        // Abrir un proyecto sale del cálculo rápido; si se pidió, se vuelve.
        app.quick_screen = options.quick && !options.home;
        if options.palette {
            app.commands.toggle();
        }
        if let Some(id) = &options.select {
            if id == "proyecto" {
                app.doc.selection = Selection::Project;
            }
            let project = &app.doc.project;
            for (i, panel) in project.panels.iter().enumerate() {
                if &panel.id == id {
                    app.doc.selection = Selection::Feeder(i);
                }
                if let Some(c) = panel.circuits.iter().position(|c| &c.id == id) {
                    app.doc.selection = Selection::Circuit(i, c);
                }
            }
        }
        app.shared.props_tab = match options.properties.as_deref() {
            Some("resultados") => crate::panels::properties::PropsTab::Results,
            Some("normativa") => crate::panels::properties::PropsTab::Normative,
            _ => crate::panels::properties::PropsTab::Inputs,
        };
        if options.chooser {
            app.shared.choose_load = app.doc.panel_index();
        }
        app.shared.ficha_open = options.ficha;
        app.shared.inspector_drawer = options.ficha;
        if app.view == View::Diagram {
            app.shared.diagram_fit = true;
        }
        app
    }

    // ------------------------------------------------------------ archivos

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.doc = Doc::new(project, path);
        self.home = false;
        self.quick_screen = false;
    }

    /// Hace lo que se eligió en la paleta de comandos.
    fn run_command(&mut self, command: Command) {
        match command {
            Command::Home => {
                self.home = true;
                self.quick_screen = false;
            }
            Command::New => self.request(Pending::New),
            Command::Open => self.request(Pending::Open),
            Command::Save => self.save(false),
            Command::Example => self.request(Pending::Example),
            Command::ExportPdf => self.export_pdf(false),
            Command::ExportExcel => self.export_excel(),
            Command::ImportExcel => self.import_excel(),
            Command::ExportDxf => self.export_dxf(),
            Command::Undo => self.doc.undo(),
            Command::Redo => self.doc.redo(),
            Command::AddCircuit => match self.doc.panel_index() {
                Some(p) => self.shared.choose_load = Some(p),
                None => {
                    self.notice = Some("El proyecto no tiene tableros: agrega uno primero.".into())
                }
            },
            Command::AddPanel => self.doc.add_panel(),
            Command::Duplicate => self.doc.duplicate_circuit(),
            Command::Remove => self.doc.remove_selected(),
            Command::Balance => {
                if let Some(p) = self.doc.panel_index() {
                    self.done = Some(self.doc.balance_phases(p));
                }
            }
            Command::Quick => {
                // Desde el inicio o en el teléfono es su propia pantalla; dentro
                // de un proyecto en escritorio, una ventana encima.
                if self.home || self.class == Class::Phone {
                    self.home = false;
                    self.quick_screen = true;
                } else {
                    self.quick_window = true;
                }
            }
            Command::View(view) => self.go_view(view),
            Command::Procedure => self.toggle_procedure(),
            Command::Split => self.split = !self.split,
            Command::ToggleTheme => {
                self.dark = !self.dark;
                let ctx = self.ctx.clone();
                self.restyle(&ctx);
            }
            Command::Select(selection) => {
                self.home = false;
                self.doc.selection = selection;
                self.view = View::Schedule;
                // En el teléfono se va directo a la ficha de lo buscado.
                self.shared.ficha_open = self.class == Class::Phone;
            }
        }
    }

    /// Va a una vista del marco.
    fn go_view(&mut self, view: View) {
        self.home = false;
        self.view = view;
        if view == View::Diagram {
            // Al abrir el unifilar se ajusta al ancho de la ventana.
            self.shared.diagram_fit = true;
        }
        if view == View::Report {
            // Al entrar a Memoria se compone de nuevo si el proyecto cambió;
            // mientras se está en ella, solo con «Actualizar».
            self.shared.preview.entering(self.doc.revision);
        }
    }

    /// Va a la vista que pidió otra vista (por ejemplo, una incidencia de
    /// Revisar en el teléfono).
    fn go_to(&mut self, view: View) {
        self.go_view(view);
        // En el teléfono, lo que se pidió desde otra vista se abre en su ficha.
        if self.class == Class::Phone && self.doc.selection != Selection::Project {
            self.shared.ficha_open = true;
        }
    }

    /// Abre o cierra el panel de procedimiento; vive en las vistas de trabajo.
    fn toggle_procedure(&mut self) {
        self.home = false;
        if !self.view.has_regions() {
            self.view = View::Schedule;
            self.procedure = true;
        } else {
            self.procedure = !self.procedure;
        }
    }

    fn load(&mut self, path: PathBuf) {
        let loaded = std::fs::read_to_string(&path)
            .map_err(|e| format!("No se pudo leer {}: {e}", path.display()))
            .and_then(|text| Project::from_json(&text));
        match loaded {
            Ok(project) => {
                self.notice = None;
                self.remember(&path);
                self.set_project(project, Some(path));
            }
            Err(message) => self.notice = Some(message),
        }
    }

    /// Pone el archivo al principio de la lista de recientes.
    fn remember(&mut self, path: &std::path::Path) {
        // Ruta absoluta: el programa se puede abrir desde cualquier carpeta.
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        // En Windows, canonicalize agrega el prefijo \\?\, que confunde al usuario.
        let path = PathBuf::from(
            path.to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string(),
        );
        self.recent.retain(|p| p != &path);
        self.recent.insert(0, path);
        self.recent.truncate(MAX_RECENT);
    }

    /// Nombre sugerido al guardar: el del proyecto, sin caracteres que
    /// Windows no acepta en un nombre de archivo.
    fn suggested_name(&self) -> String {
        let clean: String = self
            .doc
            .project
            .name
            .chars()
            .filter(|c| !r#"\/:*?"<>|"#.contains(*c))
            .collect();
        let clean = clean.trim();
        let stem = if clean.is_empty() { "proyecto" } else { clean };
        format!("{stem}.{EXTENSION}")
    }

    fn save(&mut self, ask: bool) {
        let path = if ask { None } else { self.doc.path.clone() };
        let json = self.doc.project.to_json();
        match files::save(
            json.as_bytes(),
            &self.suggested_name(),
            PROJECT_FILTER,
            path,
        ) {
            Ok(Some(saved)) => {
                self.done = Some(format!("Guardado {}", saved.describe()));
                if let Saved::Path(path) = saved {
                    self.remember(&path);
                    self.doc.path = Some(path);
                }
                self.doc.dirty = false;
                self.notice = None;
            }
            Ok(None) => {}
            Err(e) => self.notice = Some(e),
        }
    }

    fn open_dialog(&mut self) {
        files::pick(&self.inbox, Purpose::Project, PROJECT_FILTER, &self.ctx);
    }

    /// Atiende los archivos que el usuario eligió abrir.
    fn receive(&mut self) {
        for picked in self.inbox.take() {
            match picked.purpose {
                Purpose::Project => self.open_picked(picked),
                Purpose::Excel => self.import_picked(picked),
            }
        }
    }

    fn open_picked(&mut self, picked: Picked) {
        let name = picked.name;
        let loaded = picked
            .bytes
            .and_then(|b| {
                String::from_utf8(b).map_err(|_| format!("{name} no es un proyecto de IEEStudio."))
            })
            .and_then(|text| Project::from_json(&text));
        match loaded {
            Ok(project) => {
                self.notice = None;
                if let Some(path) = &picked.path {
                    self.remember(path);
                }
                self.set_project(project, picked.path);
            }
            Err(message) => self.notice = Some(message),
        }
    }

    /// Escribe la memoria del proyecto, o la hoja si el cálculo rápido tiene el foco.
    fn export_pdf(&mut self, quick: bool) {
        let (pdf, name) = if quick {
            (self.shared.quick.pdf(), "hoja-de-calculo.pdf")
        } else {
            // Si la vista previa ya compuso esta misma versión, se usa esa.
            let pdf = self
                .shared
                .preview
                .pdf(self.doc.revision)
                .unwrap_or_else(|| {
                    self.doc
                        .result
                        .as_ref()
                        .map_err(Clone::clone)
                        .and_then(|r| ieestudio_reports::project_pdf(&self.doc.project, r))
                });
            (pdf, "memoria-de-calculo.pdf")
        };
        let pdf = match pdf {
            Ok(pdf) => pdf,
            Err(e) => {
                self.notice = Some(format!("No se pudo generar el PDF: {e}"));
                return;
            }
        };
        let filter = Filter {
            name: "PDF",
            extensions: &["pdf"],
        };
        self.report_saved("PDF guardado", files::save(&pdf, name, filter, None));
    }

    /// Agrega el circuito del cálculo rápido al tablero seleccionado.
    fn quick_to_project(&mut self, input: &ieestudio_project::quick::QuickInput) {
        let Some(panel) = self.doc.panel_index() else {
            self.notice = Some("El proyecto no tiene tableros: agrega uno primero.".into());
            return;
        };
        let id = self.doc.free_circuit_id(panel);
        let circuit = input.to_circuit(&id, &self.doc.project.settings);
        let panel_id = self.doc.project.panels[panel].id.clone();
        self.doc.project.panels[panel].circuits.push(circuit);
        let count = self.doc.project.panels[panel].circuits.len();
        self.doc.selection = Selection::Circuit(panel, count - 1);
        self.doc.edited();
        self.view = View::Schedule;
        self.notice = None;
        self.done = Some(if input.system != self.doc.project.source.system {
            format!("Circuito {id} agregado a {panel_id}. Ojo: el cálculo rápido usaba otro sistema; en el proyecto se calcula con el del proyecto.")
        } else {
            format!("Circuito {id} agregado a {panel_id}, con el calibre fijado")
        });
    }

    /// Diagrama unifilar en DXF, para seguir dibujando en CAD.
    fn export_dxf(&mut self) {
        let result = match &self.doc.result {
            Ok(r) => r,
            Err(e) => {
                self.notice = Some(format!("No se pudo generar el unifilar: {e}"));
                return;
            }
        };
        let bytes = ieestudio_diagrams::plan(&self.doc.project, result).to_dxf();
        let stem = self.suggested_name();
        let stem = stem.trim_end_matches(&format!(".{EXTENSION}"));
        let filter = Filter {
            name: "Dibujo DXF",
            extensions: &["dxf"],
        };
        let name = format!("{stem} - unifilar.dxf");
        self.report_saved(
            "Unifilar guardado",
            files::save(&bytes, &name, filter, None),
        );
    }

    /// Cuadro de cargas a Excel: tableros, circuitos y resultados.
    fn export_excel(&mut self) {
        let bytes =
            match ieestudio_exchange::export_xlsx(&self.doc.project, self.doc.result.as_ref().ok())
            {
                Ok(b) => b,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let stem = self.suggested_name();
        let stem = stem.trim_end_matches(&format!(".{EXTENSION}"));
        let name = format!("{stem} - cuadro de cargas.xlsx");
        let saved = files::save(&bytes, &name, EXCEL_FILTER, None);
        self.report_saved("Cuadro de cargas guardado", saved);
    }

    /// Avisa dónde quedó un archivo exportado, o por qué no se pudo.
    fn report_saved(&mut self, what: &str, saved: Result<Option<Saved>, String>) {
        match saved {
            Ok(Some(saved)) => {
                self.notice = None;
                self.done = Some(format!("{what} {}", saved.describe()));
            }
            Ok(None) => {}
            Err(e) => self.notice = Some(e),
        }
    }

    /// Agrega o actualiza tableros y circuitos desde un libro de Excel.
    fn import_excel(&mut self) {
        files::pick(&self.inbox, Purpose::Excel, EXCEL_FILTER, &self.ctx);
    }

    /// Paso 2 de 3: lee el libro sobre una copia del proyecto y muestra lo
    /// que haría. El proyecto abierto no cambia hasta pulsar «Importar».
    fn import_picked(&mut self, picked: Picked) {
        let bytes = match picked.bytes {
            Ok(b) => b,
            Err(e) => {
                self.notice = Some(e);
                return;
            }
        };
        let mut project = self.doc.project.clone();
        match ieestudio_exchange::import_xlsx(&mut project, &bytes) {
            Ok(report) => {
                self.notice = None;
                self.import_preview = Some(ImportPreview {
                    file: picked.name,
                    changes: project != self.doc.project,
                    report,
                    project,
                });
            }
            Err(e) => self.notice = Some(format!("No se importó: {e}")),
        }
    }

    /// Vista previa de la importación: lo que va a cambiar y las filas que no
    /// se pueden importar. «Importar» lo aplica y pasa al informe.
    fn import_preview_window(&mut self, ctx: &egui::Context) {
        let Some(preview) = &self.import_preview else {
            return;
        };
        let p = self.shared.palette;
        let mut answer = None;
        let modal = egui::Modal::new(egui::Id::new("importacion-vista-previa")).show(ctx, |ui| {
            ui.set_width(500.0);
            ui.label(
                muted(&p, "Importar cuadro de cargas · paso 2 de 3").size(crate::theme::SECONDARY),
            );
            ui.label(
                strong(format!("Vista previa de «{}»", preview.file)).size(crate::theme::HEAD),
            );
            ui.add_space(crate::theme::SP2);
            let r = &preview.report;
            egui::Grid::new("vista-previa-importacion")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Circuitos nuevos", r.circuits_added.to_string()),
                        (
                            "Circuitos que se actualizan",
                            r.circuits_updated.to_string(),
                        ),
                        ("Tableros que se actualizan", r.panels_updated.to_string()),
                        (
                            "Tableros nuevos",
                            if r.panels_created.is_empty() {
                                "0".to_string()
                            } else {
                                r.panels_created.join(", ")
                            },
                        ),
                        (
                            "Filas que no se pueden importar",
                            r.errors.len().to_string(),
                        ),
                    ] {
                        ui.label(muted(&p, label));
                        ui.label(strong(value));
                        ui.end_row();
                    }
                });
            if !r.errors.is_empty() {
                ui.add_space(crate::theme::SP2);
                egui::ScrollArea::vertical()
                    .max_height(140.0)
                    .show(ui, |ui| {
                        for e in &r.errors {
                            ui.label(RichText::new(e).color(p.warn).size(crate::theme::SECONDARY));
                        }
                    });
            }
            ui.add_space(crate::theme::SP1);
            ui.label(
                muted(
                    &p,
                    if preview.changes {
                        "Nada cambia hasta que pulses «Importar»; después se puede deshacer."
                    } else {
                        "El libro no cambia nada del proyecto."
                    },
                )
                .size(crate::theme::SECONDARY),
            );
            ui.add_space(crate::theme::SP3);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(
                        preview.changes,
                        egui::Button::new(RichText::new("Importar").color(p.on_accent))
                            .fill(p.accent),
                    )
                    .clicked()
                {
                    answer = Some(true);
                }
                if ui.button("Cancelar").clicked() {
                    answer = Some(false);
                }
            });
        });
        if modal.should_close() && answer.is_none() {
            answer = Some(false);
        }
        match answer {
            Some(true) => {
                if let Some(preview) = self.import_preview.take() {
                    let before = std::mem::replace(&mut self.doc.project, preview.project);
                    self.doc.selection = Selection::Project;
                    self.doc.edited();
                    self.imported = Some((preview.report, before));
                }
            }
            Some(false) => self.import_preview = None,
            None => {}
        }
    }

    fn import_window(&mut self, ctx: &egui::Context) {
        let Some((report, _)) = &self.imported else {
            return;
        };
        let p = self.shared.palette;
        let mut close = false;
        let mut undo = false;
        let modal = egui::Modal::new(egui::Id::new("importacion")).show(ctx, |ui| {
            ui.set_width(480.0);
            ui.horizontal(|ui| {
                let (glyph, color) = if report.errors.is_empty() {
                    (icon::CHECK_CIRCLE, p.ok)
                } else {
                    (icon::WARNING, p.warn)
                };
                ui.label(
                    RichText::new(glyph)
                        .size(crate::theme::ICON_BAR)
                        .color(color),
                );
                ui.label(strong("Importación del cuadro de cargas").size(crate::theme::ICON_BAR));
            });
            ui.label(muted(&p, "Paso 3 de 3: informe").size(crate::theme::ICON));
            ui.add_space(crate::theme::SP1);
            ui.label(format!(
                "{}.",
                crate::widgets::capitalize(&report.summary())
            ));
            ui.label(
                muted(
                    &p,
                    "Los tableros y circuitos que no están en el libro quedaron como estaban.",
                )
                .size(crate::theme::FOOT),
            );
            if !report.errors.is_empty() {
                ui.add_space(crate::theme::SP2);
                ui.label(strong("Filas que no se importaron"));
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for e in &report.errors {
                            ui.label(RichText::new(e).color(p.warn).size(crate::theme::SECONDARY));
                        }
                    });
            }
            ui.add_space(crate::theme::SP3);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let ok =
                    egui::Button::new(RichText::new("Aceptar").color(p.on_accent)).fill(p.accent);
                if ui.add(ok).clicked() {
                    close = true;
                }
                if ui.button("Deshacer la importación").clicked() {
                    undo = true;
                }
            });
        });
        if modal.should_close() {
            close = true;
        }
        if undo {
            if let Some((_, before)) = self.imported.take() {
                self.doc.project = before;
                self.doc.selection = crate::state::Selection::Project;
                self.doc.edited();
            }
        } else if close {
            self.imported = None;
        }
    }

    // ------------------------------------------------------------ paneles

    /// Pide hacer algo que reemplaza el proyecto o cierra el programa. Si hay
    /// cambios sin guardar, primero pregunta.
    fn request(&mut self, action: Pending) {
        if self.doc.dirty {
            self.pending = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn perform(&mut self, action: Pending) {
        match action {
            Pending::New => {
                self.set_project(empty_project(), None);
                // Un proyecto nuevo empieza por sus datos generales.
                self.view = View::Project;
            }
            Pending::Open => self.open_dialog(),
            Pending::OpenPath(path) => self.load(path),
            Pending::Example => self.set_project(example::house(), None),
            Pending::Quit => {
                self.allow_close = true;
                self.quit = true;
            }
        }
    }

    fn restyle(&mut self, ctx: &egui::Context) {
        let palette = Palette::new(self.dark);
        self.shared.palette = palette;
        theme::apply(ctx, &palette);
    }

    // ------------------------------------------------------------ barras

    /// Opciones del menú Archivo.
    fn file_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(
                egui::Button::new(format!("{}  Nuevo proyecto", icon::FILE_PLUS))
                    .shortcut_text("Ctrl+N"),
            )
            .clicked()
        {
            ui.close();
            self.request(Pending::New);
        }
        if ui
            .add(
                egui::Button::new(format!("{}  Abrir…", icon::FOLDER_OPEN)).shortcut_text("Ctrl+O"),
            )
            .clicked()
        {
            ui.close();
            self.request(Pending::Open);
        }
        let recent: Vec<PathBuf> = self.recent.clone();
        // En el navegador no hay rutas a las que volver.
        if !files::WEB {
            ui.add_enabled_ui(!recent.is_empty(), |ui| {
                ui.menu_button("     Abrir reciente", |ui| {
                    for path in &recent {
                        let name = path.file_name().map_or_else(
                            || path.display().to_string(),
                            |n| n.to_string_lossy().into_owned(),
                        );
                        let exists = path.exists();
                        let response = ui
                            .add_enabled(exists, egui::Button::new(name))
                            .on_hover_text(path.display().to_string())
                            .on_disabled_hover_text(format!("Ya no existe: {}", path.display()));
                        if response.clicked() {
                            ui.close();
                            self.request(Pending::OpenPath(path.clone()));
                        }
                    }
                    ui.separator();
                    if ui.button("Borrar la lista").clicked() {
                        ui.close();
                        self.recent.clear();
                    }
                });
            });
        }
        if ui
            .add(
                egui::Button::new(if files::WEB {
                    format!("{}  Descargar proyecto", icon::FLOPPY_DISK)
                } else {
                    format!("{}  Guardar", icon::FLOPPY_DISK)
                })
                .shortcut_text("Ctrl+S"),
            )
            .clicked()
        {
            ui.close();
            self.save(false);
        }
        // En el navegador «Guardar» ya descarga con el nombre del proyecto.
        if !files::WEB
            && ui
                .add(egui::Button::new("     Guardar como…").shortcut_text("Ctrl+Mayús+S"))
                .clicked()
        {
            ui.close();
            self.save(true);
        }
        ui.separator();
        if ui
            .add(
                egui::Button::new(format!("{}  Exportar memoria en PDF…", icon::FILE_PDF))
                    .shortcut_text("Ctrl+E"),
            )
            .clicked()
        {
            ui.close();
            self.export_pdf(false);
        }
        if ui
            .button("     Exportar hoja del cálculo rápido…")
            .clicked()
        {
            ui.close();
            self.export_pdf(true);
        }
        if ui
            .button("     Exportar unifilar a DXF…")
            .on_hover_text(
                "Líneas y textos en capas, para abrirlo en AutoCAD u otro programa de CAD",
            )
            .clicked()
        {
            ui.close();
            self.export_dxf();
        }
        ui.separator();
        if ui
            .button(format!(
                "{}  Exportar cuadro de cargas a Excel…",
                icon::FILE_XLS
            ))
            .on_hover_text("Tableros, circuitos y resultados en un libro .xlsx")
            .clicked()
        {
            ui.close();
            self.export_excel();
        }
        if ui
            .button("     Importar cuadro de cargas de Excel…")
            .on_hover_text("Agrega o actualiza tableros y circuitos por su ID. No borra nada.")
            .clicked()
        {
            ui.close();
            self.import_excel();
        }
        ui.separator();
        if ui
            .button(format!("{}  Inicio", icon::HOUSE_LINE))
            .on_hover_text("Nuevo, abrir, ejemplo, cálculo rápido y proyectos recientes")
            .clicked()
        {
            ui.close();
            self.home = true;
        }
        if ui.button("     Abrir el proyecto de ejemplo").clicked() {
            ui.close();
            self.request(Pending::Example);
        }
        // En el navegador se cierra la pestaña; el proyecto ya quedó guardado.
        if !files::WEB {
            ui.separator();
            if ui
                .add(egui::Button::new("     Salir").shortcut_text("Alt+F4"))
                .clicked()
            {
                ui.close();
                self.request(Pending::Quit);
            }
        }
    }

    /// Barra superior: menú, marca y proyecto, deshacer y rehacer, las cinco
    /// vistas y lo que siempre debe estar a la mano.
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        // En una ventana angosta no cabe todo en un renglón: la barra entera
        // se recorre de lado, sin encimar nada ni dejar botones fuera.
        // En tableta las vistas van solo con su ícono y la barra no se recorre:
        // así quedan a la vista los botones de los cajones.
        if layout::bars_in_one_row(ui.available_width()) || self.class == Class::Tablet {
            self.top_bar_row(ui, false);
        } else {
            egui::ScrollArea::horizontal()
                .id_salt("barra-superior")
                .show(ui, |ui| self.top_bar_row(ui, true));
        }
    }

    fn top_bar_row(&mut self, ui: &mut egui::Ui, scrolling: bool) {
        let p = self.shared.palette;
        ui.horizontal(|ui| {
            // Menús sin marco, como en las aplicaciones de escritorio.
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            ui.menu_button(
                RichText::new(icon::LIST).size(crate::theme::ICON_BAR),
                |ui| {
                    ui.menu_button("Archivo", |ui| self.file_menu(ui));
                    ui.menu_button("Edición", |ui| self.edit_menu(ui));
                    ui.menu_button("Ver", |ui| self.view_menu(ui));
                    ui.menu_button("Herramientas", |ui| self.tools_menu(ui));
                    ui.menu_button("Ayuda", |ui| self.help_menu(ui));
                },
            )
            .response
            .on_hover_text("Archivo, Edición, Ver, Herramientas y Ayuda");
            ui.label(
                RichText::new(icon::LIGHTNING)
                    .size(crate::theme::ICON_BAR)
                    .color(p.accent),
            );
            if self.class == Class::Desktop {
                ui.label(strong("IEEStudio").size(crate::theme::ICON));
            }
            let dirty = if self.doc.dirty { " •" } else { "" };
            let file = self
                .doc
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned());
            // En tableta el nombre ya va en la barra de estado; arriba solo
            // queda la marca de cambios sin guardar.
            let name = if self.class == Class::Desktop {
                format!("· {}{dirty}", self.doc.project.name)
            } else {
                dirty.trim().to_string()
            };
            ui.add(egui::Label::new(muted(&p, name)).truncate())
                .on_hover_text(format!(
                    "{}; • significa cambios sin guardar",
                    file.unwrap_or_else(|| {
                        // En el navegador el proyecto se guarda solo ahí.
                        if files::WEB {
                            "En este navegador".to_string()
                        } else {
                            "Sin guardar".to_string()
                        }
                    })
                ));
            if ui
                .add_enabled(
                    self.doc.can_undo(),
                    egui::Button::new(
                        RichText::new(icon::ARROW_COUNTER_CLOCKWISE).size(crate::theme::ICON),
                    ),
                )
                .on_hover_text("Deshacer (Ctrl+Z)")
                .clicked()
            {
                self.doc.undo();
            }
            if ui
                .add_enabled(
                    self.doc.can_redo(),
                    egui::Button::new(
                        RichText::new(icon::ARROW_CLOCKWISE).size(crate::theme::ICON),
                    ),
                )
                .on_hover_text("Rehacer (Ctrl+Y)")
                .clicked()
            {
                self.doc.redo();
            }
            ui.add_space(crate::theme::SP3);
            // Las vistas completas necesitan unos 600 px y lo de la derecha
            // 190: si no caben, las vistas se reducen a su ícono.
            let compact = scrolling
                || self.class != Class::Desktop
                || ui.available_width() < FULL_VIEWS + RIGHT_TOOLS;
            // Si aun así no caben, se recorren de lado en lugar de quedar
            // debajo de los botones de la derecha.
            let room = layout::remaining(ui.available_width(), RIGHT_TOOLS);
            let need = if compact { COMPACT_VIEWS } else { FULL_VIEWS };
            let current = if self.home { None } else { Some(self.view) };
            let chosen = if scrolling {
                shell::view_tabs(ui, &p, current, &self.doc, compact)
            } else if room >= need {
                // La cápsula va al centro de la ventana si cabe sin tocar lo de
                // los lados; su ancho es el del cuadro anterior.
                let width_id = egui::Id::new("ancho-de-la-capsula");
                let width: f32 = ui.ctx().data(|d| d.get_temp(width_id)).unwrap_or(need);
                let window = ui.ctx().content_rect();
                let start = window.center().x - width / 2.0;
                let x = ui.cursor().left();
                if start > x && start + width + RIGHT_TOOLS < window.right() {
                    ui.add_space(start - x);
                }
                let tabs = ui.scope(|ui| shell::view_tabs(ui, &p, current, &self.doc, compact));
                let measured = tabs.response.rect.width();
                ui.ctx().data_mut(|d| d.insert_temp(width_id, measured));
                tabs.inner
            } else {
                egui::ScrollArea::horizontal()
                    .id_salt("barra-de-vistas")
                    .max_width(room)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| shell::view_tabs(ui, &p, current, &self.doc, compact))
                            .inner
                    })
                    .inner
            };
            if let Some(view) = chosen {
                self.go_view(view);
            }
            if scrolling {
                ui.add_space(crate::theme::SP4);
                self.top_tools(ui, false);
            } else {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    self.top_tools(ui, true);
                });
            }
        });
    }

    /// Botones de la derecha de la barra superior: tema, búsqueda, dos vistas,
    /// procedimiento y cálculo rápido, en ese orden de derecha a izquierda.
    fn top_tools(&mut self, ui: &mut egui::Ui, right_to_left: bool) {
        let p = self.shared.palette;
        let order: [u8; 7] = if right_to_left {
            [0, 1, 2, 3, 4, 5, 6]
        } else {
            [6, 5, 4, 3, 2, 1, 0]
        };
        let drawers = self.class == Class::Tablet && !self.home && self.view.has_regions();
        let window_width = ui.ctx().content_rect().width();
        for tool in order {
            match tool {
                0 => {
                    let (glyph, tip) = if self.dark {
                        (icon::SUN, "Cambiar a tema claro")
                    } else {
                        (icon::MOON, "Cambiar a tema oscuro")
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(glyph).size(crate::theme::ICON_BAR))
                                .frame(false),
                        )
                        .on_hover_text(tip)
                        .clicked()
                    {
                        self.dark = !self.dark;
                        self.restyle(ui.ctx());
                    }
                }
                1 => {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(icon::MAGNIFYING_GLASS)
                                    .size(crate::theme::ICON_BAR)
                                    .color(p.ink2),
                            )
                            .frame(false),
                        )
                        .on_hover_text(
                            "Buscar comandos, vistas, tableros y circuitos por su nombre (Ctrl+K)",
                        )
                        .clicked()
                    {
                        self.commands.toggle();
                    }
                }
                2 if self.view.has_regions() => {
                    let on = self.procedure;
                    let color = if on { p.accent } else { p.ink2 };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(icon::LIST_BULLETS)
                                    .size(crate::theme::ICON_BAR)
                                    .color(color),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Procedimiento: abrir o cerrar el panel de abajo (F7)")
                        .clicked()
                    {
                        self.toggle_procedure();
                    }
                }
                3 if self.view.has_regions() && window_width >= layout::SPLIT_FROM => {
                    let on = self.split;
                    let color = if on { p.accent } else { p.ink2 };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(icon::COLUMNS)
                                    .size(crate::theme::ICON_BAR)
                                    .color(color),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Dos vistas a la vez: el cuadro y el unifilar")
                        .clicked()
                    {
                        self.split = !self.split;
                    }
                }
                // En tableta, el navegador y la ficha viven en cajones.
                5 if drawers => {
                    let on = self.shared.inspector_drawer;
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(icon::SLIDERS_HORIZONTAL)
                                    .size(crate::theme::ICON_BAR)
                                    .color(if on { p.accent } else { p.ink2 }),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Ficha del elemento elegido")
                        .clicked()
                    {
                        self.shared.inspector_drawer = !on;
                        self.shared.navigator_drawer = false;
                    }
                }
                6 if drawers => {
                    let on = self.shared.navigator_drawer;
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(icon::SIDEBAR)
                                    .size(crate::theme::ICON_BAR)
                                    .color(if on { p.accent } else { p.ink2 }),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Navegador: tableros y circuitos del proyecto")
                        .clicked()
                    {
                        self.shared.navigator_drawer = !on;
                        self.shared.inspector_drawer = false;
                    }
                }
                4 => {
                    ui.scope(|ui| {
                        let w = &mut ui.style_mut().visuals.widgets;
                        w.inactive.bg_stroke = egui::Stroke::new(1.0, p.line);
                        w.inactive.weak_bg_fill = p.panel;
                        let text = if self.class == Class::Desktop {
                            format!("{}  Cálculo rápido", icon::CALCULATOR)
                        } else {
                            icon::CALCULATOR.to_string()
                        };
                        if ui
                            .add(
                                egui::Button::new(text)
                                    .min_size(egui::vec2(0.0, crate::theme::BAR_CONTROL_H)),
                            )
                            .on_hover_text("Un circuito suelto, sin proyecto (F8)")
                            .clicked()
                        {
                            self.run_command(Command::Quick);
                        }
                    });
                }
                _ => {}
            }
        }
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(
                self.doc.can_undo(),
                egui::Button::new(format!("{}  Deshacer", icon::ARROW_COUNTER_CLOCKWISE))
                    .shortcut_text("Ctrl+Z"),
            )
            .clicked()
        {
            ui.close();
            self.doc.undo();
        }
        if ui
            .add_enabled(
                self.doc.can_redo(),
                egui::Button::new(format!("{}  Rehacer", icon::ARROW_CLOCKWISE))
                    .shortcut_text("Ctrl+Y"),
            )
            .clicked()
        {
            ui.close();
            self.doc.redo();
        }
        ui.separator();
        let panel = self.doc.panel_index();
        if ui
            .add_enabled(
                panel.is_some(),
                egui::Button::new(format!("{}  Agregar circuito", icon::PLUS)),
            )
            .clicked()
        {
            ui.close();
            self.shared.choose_load = panel;
        }
        let on_circuit = matches!(self.doc.selection, Selection::Circuit(..));
        if ui
            .add_enabled(
                on_circuit,
                egui::Button::new(format!("{}  Duplicar circuito", icon::COPY))
                    .shortcut_text("Ctrl+D"),
            )
            .clicked()
        {
            ui.close();
            self.doc.duplicate_circuit();
        }
        if ui
            .button(format!("{}  Agregar tablero", icon::SQUARES_FOUR))
            .clicked()
        {
            ui.close();
            self.doc.add_panel();
        }
        ui.separator();
        let label = match self.doc.selection {
            Selection::Circuit(..) => "Eliminar circuito",
            Selection::Feeder(_) => "Eliminar tablero",
            Selection::Secondary(_) => "Quitar el transformador",
            Selection::Project => "Eliminar",
        };
        if ui
            .add_enabled(
                self.doc.selection != Selection::Project,
                egui::Button::new(format!("{}  {label}", icon::TRASH)).shortcut_text("Supr"),
            )
            .clicked()
        {
            ui.close();
            self.doc.remove_selected();
        }
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
        let mut restyle = false;
        restyle |= ui
            .radio_value(&mut self.dark, true, "Tema oscuro")
            .changed();
        restyle |= ui
            .radio_value(&mut self.dark, false, "Tema claro")
            .changed();
        if restyle {
            self.restyle(ui.ctx());
        }
        ui.separator();
        ui.label(muted(&p, "Ir a"));
        for view in View::ALL {
            if ui
                .add(
                    egui::Button::new(format!("{}  {}", view.icon(), view.title()))
                        .shortcut_text(view.shortcut()),
                )
                .clicked()
            {
                ui.close();
                self.go_view(view);
            }
        }
        ui.separator();
        if ui
            .add(
                egui::Button::new(format!("{}  Procedimiento", icon::LIST_BULLETS))
                    .selected(self.procedure)
                    .shortcut_text("F7"),
            )
            .clicked()
        {
            ui.close();
            self.toggle_procedure();
        }
        if ui
            .button(format!("{}  Personalizar columnas…", icon::TABLE))
            .on_hover_text("Columnas del juego «Completo» del cuadro de cargas")
            .clicked()
        {
            ui.close();
            self.columns_window = true;
        }
        if ui
            .add(
                egui::Button::new(format!("{}  Dos vistas a la vez", icon::COLUMNS))
                    .selected(self.split),
            )
            .on_hover_text("El cuadro y el unifilar lado a lado, en ventanas de 1360 px o más")
            .clicked()
        {
            ui.close();
            self.split = !self.split;
        }
    }

    fn tools_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(
                egui::Button::new(format!(
                    "{}  Cálculo rápido de un circuito",
                    icon::CALCULATOR
                ))
                .shortcut_text("F8"),
            )
            .on_hover_text("Fuera del proyecto: no cambia el proyecto abierto")
            .clicked()
        {
            ui.close();
            self.run_command(Command::Quick);
        }
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(
                egui::Button::new(format!("{}  Buscar un comando", icon::MAGNIFYING_GLASS))
                    .shortcut_text("Ctrl+K"),
            )
            .clicked()
        {
            ui.close();
            self.commands.toggle();
        }
        if ui.button("Acerca de IEEStudio").clicked() {
            ui.close();
            self.about = true;
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui, window_width: f32) {
        let p = self.shared.palette;
        // En una ventana angosta el veredicto va en su propio renglón; en uno
        // solo se encimaba con la ruta de la izquierda.
        let two_rows = !crate::layout::bars_in_one_row(window_width);
        // «Siguiente» ofrece la vista que sigue en la ruta de trabajo, sin
        // obligar el orden.
        let next = if self.home { None } else { self.view.next() };
        let mut go_next = false;
        ui.horizontal(|ui| {
            let crumbs = self.doc.breadcrumb();
            for (i, part) in crumbs.iter().enumerate() {
                if i > 0 {
                    ui.label(muted(&p, icon::CARET_RIGHT));
                }
                ui.label(if i + 1 == crumbs.len() {
                    RichText::new(part)
                } else {
                    muted(&p, part)
                });
            }
            if let Some(notice) = &self.notice {
                ui.separator();
                ui.label(RichText::new(format!("{} {notice}", icon::WARNING_CIRCLE)).color(p.bad));
            } else if let Some(done) = &self.done {
                ui.separator();
                ui.label(RichText::new(format!("{} {done}", icon::CHECK)).color(p.ok));
            }
            if !two_rows {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(next) = next {
                        go_next |= next_button(ui, &p, next);
                        ui.add_space(crate::theme::SP2);
                    }
                    self.status_verdict(ui, &p);
                });
            }
        });
        if two_rows {
            ui.horizontal_wrapped(|ui| {
                self.status_verdict(ui, &p);
                if let Some(next) = next {
                    go_next |= next_button(ui, &p, next);
                }
            });
        }
        if let (true, Some(next)) = (go_next, next) {
            self.go_view(next);
        }
    }

    /// Veredicto de la barra de estado.
    fn status_verdict(&self, ui: &mut egui::Ui, p: &Palette) {
        match &self.doc.result {
            Ok(r) => verdict_chips(ui, p, &r.verdict),
            Err(e) => {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{} No se pudo calcular: {e}", icon::WARNING_CIRCLE))
                            .color(p.bad),
                    )
                    .wrap(),
                );
            }
        }
    }

    /// «¿Qué vas a conectar?»: agrega el circuito del tipo elegido, o un
    /// subtablero, al tablero que la pidió.
    fn chooser_window(&mut self, ctx: &egui::Context) {
        let Some(panel) = self.shared.choose_load else {
            return;
        };
        if panel >= self.doc.project.panels.len() {
            self.shared.choose_load = None;
            return;
        }
        let p = self.shared.palette;
        match chooser::show(ctx, &p, &self.doc, panel) {
            chooser::Outcome::Open => {}
            chooser::Outcome::Cancel => self.shared.choose_load = None,
            chooser::Outcome::Pick(symbol) => {
                self.shared.choose_load = None;
                match symbol {
                    Symbol::Load(kind) => {
                        self.doc.add_circuit_of(panel, kind);
                        self.view = View::Schedule;
                        // En el teléfono, el circuito nuevo abre su ficha.
                        self.shared.ficha_open = self.class == Class::Phone;
                        // La descripción queda lista para escribir.
                        crate::panels::properties::focus_description(ctx);
                    }
                    Symbol::Subpanel => {
                        let k = self.doc.add_subpanel(panel);
                        let panels = &self.doc.project.panels;
                        self.shared.done = Some(format!(
                            "Subtablero {} agregado desde {}: captura su alimentador.",
                            panels[k].id, panels[panel].id
                        ));
                        // Su alimentador se captura en la ficha del tablero.
                        self.view = View::Schedule;
                        self.shared.ficha_open = self.class == Class::Phone;
                    }
                }
            }
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, KeyboardShortcut, Modifiers};
        let pressed = |m: Modifiers, k: Key| {
            ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)))
        };
        if pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::S) {
            self.save(true);
        } else if pressed(Modifiers::COMMAND, Key::S) {
            self.save(false);
        }
        if pressed(Modifiers::COMMAND, Key::O) {
            self.request(Pending::Open);
        }
        if pressed(Modifiers::COMMAND, Key::N) {
            self.request(Pending::New);
        }
        if pressed(Modifiers::COMMAND, Key::E) {
            self.export_pdf(false);
        }
        if pressed(Modifiers::COMMAND, Key::K) {
            self.commands.toggle();
        }
        let typing = ctx.memory(|m| m.focused().is_some());
        if !typing {
            // Con un campo de texto activo, Ctrl+Z deshace dentro del campo.
            if pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                || pressed(Modifiers::COMMAND, Key::Y)
            {
                self.doc.redo();
            } else if pressed(Modifiers::COMMAND, Key::Z) {
                self.doc.undo();
            }
            if pressed(Modifiers::COMMAND, Key::D) {
                self.doc.duplicate_circuit();
            }
            if pressed(Modifiers::NONE, Key::Delete)
                && matches!(self.doc.selection, Selection::Circuit(..))
            {
                self.doc.remove_selected();
            }
        }
        // F2 a F6: las cinco vistas; F7, el procedimiento; F8, el cálculo rápido.
        let keys = [Key::F2, Key::F3, Key::F4, Key::F5, Key::F6];
        for (key, view) in keys.into_iter().zip(View::ALL) {
            if pressed(Modifiers::NONE, key) {
                self.go_view(view);
            }
        }
        if pressed(Modifiers::NONE, Key::F7) {
            self.toggle_procedure();
        }
        if pressed(Modifiers::NONE, Key::F8) {
            self.run_command(Command::Quick);
        }
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        let p = self.shared.palette;
        egui::Window::new("Acerca de IEEStudio")
            .open(&mut self.about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(strong(format!("IEEStudio {}", env!("CARGO_PKG_VERSION"))));
                ui.label("Cálculo de instalaciones eléctricas de baja tensión. Código abierto, MIT o Apache-2.0.");
                ui.add_space(crate::theme::SP2);
                ui.label(muted(&p, "Perfil normativo: NOM-001-SEDE-2012."));
                ui.label(muted(&p, "IEEStudio calcula y verifica; no otorga aprobación profesional. Los resultados debe revisarlos y firmarlos un profesional."));
            });
    }

    /// Pregunta qué hacer con los cambios sin guardar antes de seguir.
    fn confirm_window(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending.clone() else {
            return;
        };
        let p = self.shared.palette;
        #[derive(PartialEq)]
        enum Choice {
            Save,
            Discard,
            Cancel,
        }
        let mut choice = None;
        let continue_label = match action {
            Pending::Quit => "Salir sin guardar",
            _ => "No guardar",
        };
        let modal = egui::Modal::new(egui::Id::new("cambios-sin-guardar")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icon::WARNING)
                        .size(crate::theme::ICON_BAR)
                        .color(p.warn),
                );
                ui.label(strong("¿Guardar los cambios?").size(crate::theme::ICON_BAR));
            });
            ui.add_space(crate::theme::SP1);
            ui.label(if files::WEB {
                format!(
                    "«{}» tiene cambios que no has descargado. Si continúas, el proyecto \
                     guardado en este navegador se reemplaza.",
                    self.doc.project.name
                )
            } else {
                format!(
                    "«{}» tiene cambios sin guardar. Si continúas sin guardar, se pierden.",
                    self.doc.project.name
                )
            });
            ui.add_space(crate::theme::SP3);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = if files::WEB { "Descargar" } else { "Guardar" };
                let save = egui::Button::new(
                    RichText::new(format!("{}  {label}", icon::FLOPPY_DISK)).color(p.on_accent),
                )
                .fill(p.accent);
                if ui.add(save).clicked() {
                    choice = Some(Choice::Save);
                }
                if ui.button(continue_label).clicked() {
                    choice = Some(Choice::Discard);
                }
                if ui.button("Cancelar").clicked() {
                    choice = Some(Choice::Cancel);
                }
            });
        });
        if modal.should_close() && choice.is_none() {
            choice = Some(Choice::Cancel);
        }
        match choice {
            Some(Choice::Save) => {
                self.pending = None;
                self.save(false);
                // Si el usuario canceló el cuadro de guardar, no se pierde nada.
                if !self.doc.dirty {
                    self.perform(action);
                }
            }
            Some(Choice::Discard) => {
                self.pending = None;
                self.perform(action);
            }
            Some(Choice::Cancel) => self.pending = None,
            None => {}
        }
    }

    /// Guarda una imagen de la ventana y cierra, si se pidió con `--captura`.
    fn handle_capture(&mut self, ctx: &egui::Context) {
        let Some(capture) = &mut self.capture else {
            return;
        };
        capture.frames += 1;
        if capture.frames == 8 + capture.measure {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let [w, h] = image.size;
            if let Err(e) = image::save_buffer(
                &capture.path,
                image.as_raw(),
                w as u32,
                h as u32,
                image::ColorType::Rgba8,
            ) {
                eprintln!("No se pudo guardar la captura: {e}");
            }
            if capture.measure > 0 {
                println!("{}", frame_report(&capture.times));
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}

impl App {
    /// Un cuadro de la interfaz.
    /// Modo táctil: en el teléfono siempre; en otros tamaños, desde que la
    /// pantalla se toca hasta que se vuelve a mover el ratón.
    fn update_touch(&mut self, ctx: &egui::Context) {
        let (touched, moused) = ctx.input(|i| {
            let touched = i
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::Touch { .. }));
            let moved = i
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerMoved(_)));
            (touched, moved && !touched && !i.pointer.any_down())
        });
        if touched {
            self.touch = true;
        } else if moused {
            self.touch = false;
        }
        theme::set_touch(ctx, self.touch || self.class == Class::Phone);
    }

    /// Escritorio y tableta: barra superior con las vistas, barra de estado
    /// y la vista activa con sus regiones o cajones.
    fn desktop_frame(
        &mut self,
        ui: &mut egui::Ui,
        window_width: f32,
    ) -> (Option<shell::Outcome>, Option<Start>) {
        let p = self.shared.palette;
        egui::Panel::top("menu")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("estado")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| self.status_bar(ui, window_width));
        self.center(ui, window_width)
    }

    /// La pantalla de inicio o la vista activa, entre las barras.
    fn center(
        &mut self,
        ui: &mut egui::Ui,
        window_width: f32,
    ) -> (Option<shell::Outcome>, Option<Start>) {
        let p = self.shared.palette;
        let mut start = None;
        let home = self.home;
        let recent = self.recent.clone();
        let mut arrangement = Arrangement {
            view: self.view,
            regions: layout::regions(window_width, self.split),
            procedure: self.procedure,
            class: self.class,
        };
        let mut outcome = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| {
                if home {
                    start = home::show(ui, &p, &recent, &self.doc.project.name);
                    return;
                }
                outcome = Some(shell::show_view(
                    ui,
                    &mut arrangement,
                    &mut self.doc,
                    &mut self.shared,
                ));
            });
        self.procedure = arrangement.procedure;
        (outcome, start)
    }

    /// Teléfono: barra superior corta, la vista activa en una columna y la
    /// barra inferior con los cinco destinos.
    fn phone_frame(&mut self, ui: &mut egui::Ui) -> (Option<shell::Outcome>, Option<Start>) {
        let p = self.shared.palette;
        egui::Panel::top("menu")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ui, |ui| self.phone_top_bar(ui));
        if !self.home {
            egui::Panel::bottom("destinos")
                .frame(
                    egui::Frame::new()
                        .fill(p.bg)
                        .inner_margin(egui::Margin::symmetric(4, 4)),
                )
                .show(ui, |ui| {
                    if let Some(view) = compact::bottom_bar(ui, &p, Some(self.view), &self.doc) {
                        self.shared.sheet = None;
                        self.go_view(view);
                    }
                });
        }
        // Los avisos van en un renglón arriba de los destinos.
        let message = self
            .notice
            .as_ref()
            .map(|n| (icon::WARNING_CIRCLE, n.clone(), p.bad))
            .or_else(|| self.done.as_ref().map(|d| (icon::CHECK, d.clone(), p.ok)));
        if let Some((glyph, text, color)) = message {
            egui::Panel::bottom("aviso-telefono")
                .frame(
                    egui::Frame::new()
                        .fill(p.bg)
                        .inner_margin(egui::Margin::symmetric(12, 4)),
                )
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(format!("{glyph} {text}")).color(color))
                            .wrap(),
                    );
                });
        }
        let width = ui.available_width();
        self.center(ui, width)
    }

    /// Barra superior del teléfono: el nombre o «‹ atrás» a la izquierda;
    /// búsqueda y menú a la derecha.
    fn phone_top_bar(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
        ui.horizontal(|ui| {
            ui.set_min_height(theme::TOUCH_H);
            let back = if self.home {
                None
            } else {
                compact::back(self.view, &self.doc, &self.shared)
            };
            match back {
                Some(label) => {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("{}  {label}", icon::CARET_LEFT))
                                    .color(p.info),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        compact::go_back(&mut self.doc, &mut self.shared);
                    }
                }
                None => {
                    ui.label(
                        RichText::new(icon::LIGHTNING)
                            .size(theme::ICON_BAR)
                            .color(p.accent),
                    );
                    ui.label(strong("IEEStudio").size(theme::HEAD));
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                ui.menu_button(RichText::new(icon::LIST).size(theme::ICON_BAR), |ui| {
                    self.phone_menu(ui);
                });
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(icon::MAGNIFYING_GLASS).size(theme::ICON_BAR),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    self.commands.toggle();
                }
            });
        });
    }

    /// Menú del teléfono: lo de los menús de escritorio que sirve con el
    /// dedo, más deshacer, el tema y el cálculo rápido.
    fn phone_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(
                self.doc.can_undo(),
                egui::Button::new(format!("{}  Deshacer", icon::ARROW_COUNTER_CLOCKWISE)),
            )
            .clicked()
        {
            self.doc.undo();
            ui.close();
        }
        if ui
            .add_enabled(
                self.doc.can_redo(),
                egui::Button::new(format!("{}  Rehacer", icon::ARROW_CLOCKWISE)),
            )
            .clicked()
        {
            self.doc.redo();
            ui.close();
        }
        ui.separator();
        if ui
            .button(format!("{}  Cálculo rápido", icon::CALCULATOR))
            .clicked()
        {
            self.run_command(Command::Quick);
            ui.close();
        }
        if ui.button(format!("{}  Inicio", icon::HOUSE)).clicked() {
            self.run_command(Command::Home);
            ui.close();
        }
        let theme_label = if self.dark {
            format!("{}  Tema claro", icon::SUN)
        } else {
            format!("{}  Tema oscuro", icon::MOON)
        };
        if ui.button(theme_label).clicked() {
            self.run_command(Command::ToggleTheme);
            ui.close();
        }
        ui.separator();
        ui.menu_button("Archivo", |ui| self.file_menu(ui));
        ui.menu_button("Ayuda", |ui| self.help_menu(ui));
    }

    /// La experiencia «Cálculo rápido»: las herramientas de cálculo directo
    /// a pantalla completa, sin proyecto ni barra de vistas.
    fn quick_screen_ui(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
        egui::Panel::top("menu")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(theme::BAR_CONTROL_H);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("{}  Inicio", icon::CARET_LEFT))
                                    .color(p.info),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Volver a la pantalla de inicio")
                        .clicked()
                    {
                        self.home = true;
                        self.quick_screen = false;
                    }
                    ui.label(
                        RichText::new(icon::CALCULATOR)
                            .size(theme::ICON_BAR)
                            .color(p.accent),
                    );
                    ui.label(strong("Cálculo rápido").size(theme::HEAD));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (glyph, tip) = if self.dark {
                            (icon::SUN, "Cambiar a tema claro")
                        } else {
                            (icon::MOON, "Cambiar a tema oscuro")
                        };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(glyph).size(theme::ICON_BAR))
                                    .frame(false),
                            )
                            .on_hover_text(tip)
                            .clicked()
                        {
                            self.run_command(Command::ToggleTheme);
                        }
                    });
                });
            });
        let mut export = false;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| export = self.shared.quick.ui(ui, &p, true));
        if export {
            self.export_pdf(true);
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
        let ctx = ui.ctx().clone();
        // Cerrar la ventana con cambios sin guardar pregunta antes.
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allow_close
            && self.capture.is_none()
            && self.doc.dirty
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Pending::Quit);
        }
        if self.pending.is_none() && self.shared.choose_load.is_none() {
            self.shortcuts(&ctx);
        }
        self.receive();

        let window_width = ui.available_width();
        self.class = layout::class(window_width);
        self.update_touch(&ctx);
        let revision = self.doc.revision;
        let (outcome, start) = if self.quick_screen {
            self.quick_screen_ui(ui);
            (None, None)
        } else if self.class == Class::Phone {
            self.phone_frame(ui)
        } else {
            self.desktop_frame(ui, window_width)
        };
        if let Some(outcome) = outcome {
            if let Some(view) = outcome.go {
                self.go_to(view);
            }
            match outcome.request {
                Some(Export::Pdf) => self.export_pdf(false),
                Some(Export::Excel) => self.export_excel(),
                Some(Export::Dxf) => self.export_dxf(),
                None => {}
            }
        }
        match start {
            Some(Start::New) => self.request(Pending::New),
            Some(Start::Open) => self.request(Pending::Open),
            Some(Start::Example) => self.request(Pending::Example),
            Some(Start::Quick) => {
                self.home = false;
                self.quick_screen = true;
            }
            Some(Start::Recent(path)) => self.request(Pending::OpenPath(path)),
            Some(Start::Back) => self.home = false,
            None => {}
        }
        if self.doc.revision != revision {
            self.done = None;
        }
        if self.pending.is_none() {
            if let Some(command) = self.commands.show(&ctx, &p, &self.doc) {
                self.run_command(command);
            }
        }
        self.chooser_window(&ctx);
        if self.columns_window {
            let mut open = true;
            schedule::columns_window(&ctx, &mut self.shared, &mut open);
            self.columns_window = open;
        }
        self.about_window(&ctx);
        self.confirm_window(&ctx);
        self.import_preview_window(&ctx);
        self.import_window(&ctx);
        if std::mem::take(&mut self.shared.open_procedure) {
            self.home = false;
            self.view = if self.view.has_regions() {
                self.view
            } else {
                View::Schedule
            };
            self.procedure = true;
        }
        // Un panel pidió el unifilar completo: se pasa a esa vista.
        if std::mem::take(&mut self.shared.diagram_full) {
            self.view = View::Diagram;
        }
        if let Some(done) = self.shared.done.take() {
            self.notice = None;
            self.done = Some(done);
        }
        if self.quick_window {
            let mut open = true;
            let mut export = false;
            egui::Window::new(format!("{}  Cálculo rápido", icon::CALCULATOR))
                .open(&mut open)
                .default_size([1100.0, 700.0])
                .default_pos([160.0, 70.0])
                .collapsible(false)
                .show(&ctx, |ui| {
                    if self.shared.quick.ui(ui, &p, false) {
                        export = true;
                    }
                });
            self.quick_window = open;
            if export {
                self.export_pdf(true);
            }
        }
        if let Some(input) = self.shared.quick.take_request() {
            self.quick_to_project(&input);
            self.quick_window = false;
        }
        // El historial registra un cambio cuando la interacción terminó.
        let settled = !ctx.input(|i| i.pointer.any_down()) && ctx.memory(|m| m.focused().is_none());
        if settled {
            self.doc.commit();
        }
        if self.quit {
            self.quit = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Solo al medir con --medir, que no existe en el navegador.
        let started = self
            .capture
            .as_ref()
            .filter(|c| c.measure > 0)
            .map(|_| std::time::Instant::now());
        self.draw(ui);
        if let (Some(started), Some(capture)) = (started, &mut self.capture) {
            capture.times.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        self.handle_capture(&ui.ctx().clone());
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let prefs = Prefs {
            dark: self.dark,
            columns: self.shared.columns.clone(),
            column_set: self.shared.column_set,
            diagram_zoom: self.shared.diagram_zoom,
            recent: self.recent.clone(),
            view: self.view,
            split: self.split,
        };
        if let Ok(text) = serde_json::to_string(&prefs) {
            storage.set_string(STORAGE_KEY, text);
        }
        if files::WEB {
            if let Some(text) = self.unreadable.take() {
                storage.set_string(WEB_PROJECT_UNREADABLE_KEY, text);
            }
            storage.set_string(WEB_PROJECT_KEY, self.doc.project.to_json());
        }
    }
}

/// El botón «Siguiente: …» de la barra de estado. `true` si se pulsó.
fn next_button(ui: &mut egui::Ui, p: &Palette, next: View) -> bool {
    // Discreto: el amarillo es de la acción principal de cada vista.
    ui.add(
        egui::Button::new(
            RichText::new(format!(
                "Siguiente: {}  {}",
                next.title(),
                icon::ARROW_RIGHT
            ))
            .size(crate::theme::SECONDARY)
            .color(p.ink2)
            .family(crate::theme::semibold()),
        )
        .frame(false),
    )
    .on_hover_text(format!("Ir a {} ({})", next.title(), next.shortcut()))
    .clicked()
}

/// Lo que haría una importación de Excel, antes de aplicarla.
struct ImportPreview {
    file: String,
    report: ieestudio_exchange::ImportReport,
    /// El proyecto como quedaría.
    project: Project,
    /// Si cambia algo del proyecto abierto.
    changes: bool,
}

/// Resumen de los tiempos de `--medir`: el primer cuadro aparte, porque arma
/// lo que después queda guardado (unifilar, memoria).
fn frame_report(times: &[f64]) -> String {
    let Some((first, rest)) = times.split_first() else {
        return "Sin cuadros medidos.".to_string();
    };
    let mut sorted = rest.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |q: f64| {
        sorted
            .get(((sorted.len() as f64 * q).ceil() as usize).saturating_sub(1))
            .copied()
            .unwrap_or(0.0)
    };
    format!(
        "Cuadros medidos: {} · primero {first:.1} ms · después: mediana {:.2} ms, p95 {:.2} ms, máximo {:.2} ms",
        times.len(),
        at(0.5),
        at(0.95),
        sorted.last().copied().unwrap_or(0.0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Las preferencias que dejó una versión con modo guiado y avanzado traen
    // campos que ya no existen: se leen sin error y se ignoran.
    #[test]
    fn preferences_from_before_the_single_frame_still_load() {
        let old = serde_json::json!({
            "dark": false,
            "accent": 2,
            "workspace": "Diagram",
            "dock": { "surfaces": [], "focused_surface": null },
            "columns": serde_json::to_value(schedule::Columns::default()).unwrap(),
            "diagram_zoom": 0.9,
            "recent": ["/tmp/casa.iees"],
            "guided": false,
            "step": "Review"
        });
        let prefs: Prefs = serde_json::from_value(old).expect("se lee");
        assert!(!prefs.dark);
        assert_eq!(prefs.recent.len(), 1);
        // Sin vista guardada, se abre la de trabajo.
        assert_eq!(prefs.view, View::Schedule);
        assert!(!prefs.split);
    }

    #[test]
    fn the_view_and_the_split_are_remembered() {
        let prefs = Prefs {
            dark: true,
            columns: schedule::Columns::default(),
            column_set: schedule::ColumnSet::Complete,
            diagram_zoom: 1.3,
            recent: Vec::new(),
            view: View::Review,
            split: true,
        };
        let text = serde_json::to_string(&prefs).unwrap();
        let back: Prefs = serde_json::from_str(&text).unwrap();
        assert_eq!(back.view, View::Review);
        assert!(back.split);
    }
}
