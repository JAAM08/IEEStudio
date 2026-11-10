//! Ventana principal: menús, espacios de trabajo, paneles acoplables y barra
//! de estado alrededor de un único proyecto.

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};
use egui_dock::{DockArea, DockState, Style};
use egui_phosphor::regular as icon;
use ieestudio_project::{example, Project};
use serde::{Deserialize, Serialize};

use crate::chooser;
use crate::commands::{Command, CommandPalette};
use crate::files::{self, Filter, Inbox, Picked, Purpose, Saved};
use crate::guided::{self, Step};
use crate::panels::{schedule, Shared, Viewer};
use crate::state::{empty_project, Doc, Selection};
use crate::symbols::Symbol;
use crate::theme::{self, Palette, ACCENTS};
use crate::widgets::{muted, strong, verdict_chips};
use crate::workspace::{self, Tab, Workspace};

const STORAGE_KEY: &str = "ieestudio-interfaz-1";
/// En el navegador, el proyecto abierto se guarda solo con esta clave, para
/// no perder trabajo si se cierra la pestaña.
const WEB_PROJECT_KEY: &str = "ieestudio-proyecto-web";

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
    accent: usize,
    workspace: Workspace,
    dock: DockState<Tab>,
    columns: schedule::Columns,
    diagram_zoom: f32,
    /// Proyectos abiertos o guardados hace poco; el primero se abre al
    /// arrancar.
    #[serde(default)]
    recent: Vec<PathBuf>,
    /// Vista guiada por pasos (la de omisión) o avanzada con paneles.
    #[serde(default = "yes")]
    guided: bool,
    #[serde(default)]
    step: Step,
}

fn yes() -> bool {
    true
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
}

pub struct App {
    doc: Doc,
    shared: Shared,
    dock: DockState<Tab>,
    workspace: Workspace,
    dark: bool,
    accent: usize,
    notice: Option<String>,
    done: Option<String>,
    about: bool,
    capture: Option<Capture>,
    /// Acción detenida por cambios sin guardar.
    pending: Option<Pending>,
    /// Resultado de la última importación de Excel, con el proyecto de antes
    /// para poder deshacerla.
    imported: Option<(ieestudio_exchange::ImportReport, Project)>,
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
    /// Vista guiada por pasos; si no, la avanzada con paneles acoplables.
    guided: bool,
    step: Step,
    /// Cálculo rápido como ventana flotante, en la vista guiada.
    quick_window: bool,
}

#[derive(Clone)]
pub struct Options {
    pub open: Option<PathBuf>,
    pub capture: Option<PathBuf>,
    pub light: bool,
    pub workspace: Option<Workspace>,
    pub focus: Option<Tab>,
    /// Empieza sin paneles, para revisar esa pantalla.
    pub no_panels: bool,
    /// Circuito o tablero que se selecciona al abrir, por su ID.
    pub select: Option<String>,
    /// Pestaña inicial de Propiedades: `datos`, `resultados` o `normativa`.
    pub properties: Option<String>,
    /// Empieza en la pantalla de inicio, para revisarla.
    pub home: bool,
    /// Empieza con la paleta de comandos abierta, para revisarla.
    pub palette: bool,
    /// Paso de la vista guiada en el que se empieza.
    pub step: Option<Step>,
    /// Empieza en la vista avanzada, con paneles.
    pub advanced: bool,
    /// Empieza con la ventana «¿Qué vas a conectar?» abierta, para revisarla.
    pub chooser: bool,
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
        let (saved_guided, saved_step) = prefs
            .as_ref()
            .map_or((true, Step::Circuits), |p| (p.guided, p.step));
        // Las opciones de la vista avanzada (espacio, panel, pestaña de
        // propiedades) la piden; un paso pide la guiada.
        let advanced = options.advanced
            || options.properties.is_some()
            || options.workspace.is_some()
            || options.focus.is_some()
            || options.no_panels;
        let guided_view = options.step.is_some() || (!advanced && saved_guided);
        let start_step = options.step.unwrap_or(saved_step);
        let (dark, accent, workspace, dock, columns, zoom) = match prefs {
            Some(p) => (
                p.dark,
                p.accent.min(ACCENTS.len() - 1),
                p.workspace,
                // Un acomodo guardado sin paneles se repone.
                workspace::sanitize(p.dock, p.workspace),
                p.columns,
                p.diagram_zoom,
            ),
            None => {
                let ws = options.workspace.unwrap_or(Workspace::Calculation);
                (
                    !options.light,
                    0,
                    ws,
                    ws.layout(),
                    schedule::Columns::default(),
                    1.3,
                )
            }
        };
        let palette = Palette::new(dark, ACCENTS[accent].1);
        theme::apply(&cc.egui_ctx, &palette);
        let mut shared = Shared::new(palette);
        shared.columns = columns;
        shared.diagram_zoom = zoom;
        let mut app = App {
            doc: Doc::new(example::house(), None),
            shared,
            dock,
            workspace,
            dark,
            accent,
            notice: None,
            done: None,
            about: false,
            capture: options.capture.map(|path| Capture { path, frames: 0 }),
            pending: None,
            imported: None,
            allow_close: false,
            quit: false,
            recent,
            inbox: Inbox::default(),
            ctx: cc.egui_ctx.clone(),
            commands: CommandPalette::default(),
            home: false,
            guided: guided_view,
            step: start_step,
            quick_window: false,
        };
        app.doc.selection = Selection::Circuit(0, 1);
        // Abre lo que se pidió; si no, el último proyecto, si todavía existe.
        let last = app.recent.first().filter(|p| p.exists()).cloned();
        let mut resumed = false;
        if let Some(path) = options.open.or(last) {
            app.load(path);
            resumed = true;
        } else if files::WEB {
            // En el navegador se retoma el proyecto en el que se trabajaba.
            let saved = cc
                .storage
                .and_then(|s| s.get_string(WEB_PROJECT_KEY))
                .and_then(|text| Project::from_json(&text).ok());
            if let Some(project) = saved {
                app.set_project(project, None);
                resumed = true;
            }
        }
        // Sin nada que retomar se empieza en la pantalla de inicio; las
        // capturas de prueba solo la muestran si se pide.
        app.home = options.home || (!resumed && !capturing);
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
        if let Some(tab) = options.focus {
            workspace::open(&mut app.dock, tab);
        }
        if options.no_panels {
            for tab in Tab::ALL {
                workspace::close(&mut app.dock, tab);
            }
        }
        if options.chooser {
            app.shared.choose_load = app.doc.panel_index();
        }
        app
    }

    // ------------------------------------------------------------ archivos

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.doc = Doc::new(project, path);
        self.home = false;
    }

    /// Hace lo que se eligió en la paleta de comandos.
    fn run_command(&mut self, command: Command) {
        match command {
            Command::Home => self.home = true,
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
                    self.done = self.doc.balance_phases(p).map(|(before, after)| {
                        format!("Fases balanceadas: desbalance de {before:.1} % a {after:.1} %")
                    });
                }
            }
            Command::Quick => {
                self.home = false;
                if self.guided {
                    self.quick_window = true;
                } else {
                    workspace::open(&mut self.dock, Tab::Quick);
                }
            }
            Command::Panel(tab) => {
                self.home = false;
                self.guided = false;
                workspace::open(&mut self.dock, tab);
            }
            Command::Workspace(ws) => {
                self.home = false;
                self.guided = false;
                self.set_workspace(ws);
            }
            Command::ResetWorkspace => {
                self.home = false;
                self.guided = false;
                self.dock = self.workspace.layout();
            }
            Command::Step(step) => {
                self.home = false;
                self.guided = true;
                self.step = step;
            }
            Command::ToggleView => {
                self.home = false;
                self.guided = !self.guided;
            }
            Command::ToggleTheme => {
                self.dark = !self.dark;
                let ctx = self.ctx.clone();
                self.restyle(&ctx);
            }
            Command::Select(selection) => {
                self.home = false;
                self.doc.selection = selection;
                if self.guided {
                    self.step = match selection {
                        Selection::Circuit(..) => Step::Circuits,
                        _ => Step::Panels,
                    };
                } else {
                    workspace::open(&mut self.dock, Tab::Schedule);
                }
            }
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
            (
                self.doc
                    .result
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|r| ieestudio_reports::project_pdf(&self.doc.project, r)),
                "memoria-de-calculo.pdf",
            )
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
        workspace::open(&mut self.dock, Tab::Schedule);
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

    fn import_picked(&mut self, picked: Picked) {
        let bytes = match picked.bytes {
            Ok(b) => b,
            Err(e) => {
                self.notice = Some(e);
                return;
            }
        };
        let before = self.doc.project.clone();
        match ieestudio_exchange::import_xlsx(&mut self.doc.project, &bytes) {
            Ok(report) => {
                self.notice = None;
                if self.doc.project != before {
                    self.doc.edited();
                }
                self.imported = Some((report, before));
            }
            Err(e) => {
                self.doc.project = before;
                self.notice = Some(format!("No se importó: {e}"));
            }
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
                ui.label(RichText::new(glyph).size(20.0).color(color));
                ui.label(strong("Importación del cuadro de cargas").size(15.0));
            });
            ui.add_space(4.0);
            ui.label(format!(
                "{}.",
                crate::widgets::capitalize(&report.summary())
            ));
            ui.label(
                muted(
                    &p,
                    "Los tableros y circuitos que no están en el libro quedaron como estaban.",
                )
                .size(11.0),
            );
            if !report.errors.is_empty() {
                ui.add_space(6.0);
                ui.label(strong("Filas que no se importaron"));
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for e in &report.errors {
                            ui.label(RichText::new(e).color(p.warn).size(12.0));
                        }
                    });
            }
            ui.add_space(10.0);
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

    fn set_workspace(&mut self, ws: Workspace) {
        self.workspace = ws;
        self.dock = ws.layout();
    }

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
                self.step = Step::Project;
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

    /// Casillas para abrir y cerrar cada panel, con su tecla.
    fn panel_checkboxes(&mut self, ui: &mut egui::Ui) {
        for tab in Tab::ALL {
            let mut open = workspace::is_open(&self.dock, tab);
            if ui
                .add(egui::Checkbox::new(
                    &mut open,
                    format!("{}  {}", tab.icon(), tab.title()),
                ))
                .on_hover_text(tab.shortcut())
                .changed()
            {
                workspace::toggle(&mut self.dock, tab);
            }
        }
    }

    fn restyle(&mut self, ctx: &egui::Context) {
        let palette = Palette::new(self.dark, ACCENTS[self.accent].1);
        self.shared.palette = palette;
        theme::apply(ctx, &palette);
    }

    // ------------------------------------------------------------ barras

    /// Opciones del menú Archivo; las comparten la vista guiada y la avanzada.
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

    /// Barra superior de la vista guiada: marca, Archivo, los cinco pasos y
    /// lo que siempre debe estar a la mano.
    fn guided_bar(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
        ui.horizontal(|ui| {
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            ui.label(RichText::new(icon::LIGHTNING).size(17.0).color(p.accent));
            ui.label(strong("IEEStudio").size(14.0));
            let dirty = if self.doc.dirty { " •" } else { "" };
            ui.add(
                egui::Label::new(muted(&p, format!("· {}{dirty}", self.doc.project.name)))
                    .truncate(),
            )
            .on_hover_text("Proyecto abierto; • significa cambios sin guardar");
            ui.add_space(6.0);
            ui.menu_button("Archivo", |ui| self.file_menu(ui));
            if ui
                .add_enabled(
                    self.doc.can_undo(),
                    egui::Button::new(RichText::new(icon::ARROW_COUNTER_CLOCKWISE).size(14.0)),
                )
                .on_hover_text("Deshacer (Ctrl+Z)")
                .clicked()
            {
                self.doc.undo();
            }
            if ui
                .add_enabled(
                    self.doc.can_redo(),
                    egui::Button::new(RichText::new(icon::ARROW_CLOCKWISE).size(14.0)),
                )
                .on_hover_text("Rehacer (Ctrl+Y)")
                .clicked()
            {
                self.doc.redo();
            }
            ui.add_space(18.0);
            if let Some(step) = guided::step_bar(ui, &p, if self.home { Step::ALL[0] } else { self.step }, &self.doc) {
                self.step = step;
                self.home = false;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (glyph, tip) = if self.dark {
                    (icon::SUN, "Cambiar a tema claro")
                } else {
                    (icon::MOON, "Cambiar a tema oscuro")
                };
                if ui
                    .add(egui::Button::new(RichText::new(glyph).size(15.0)).frame(false))
                    .on_hover_text(tip)
                    .clicked()
                {
                    self.dark = !self.dark;
                    self.restyle(ui.ctx());
                }
                if ui
                    .button(RichText::new(format!("{}  Vista avanzada", icon::LAYOUT)).color(p.ink2))
                    .on_hover_text("Paneles acoplables y espacios de trabajo, para quien ya domina el programa")
                    .clicked()
                {
                    self.guided = false;
                    self.home = false;
                }
                if ui
                    .button(RichText::new(format!("{}  Buscar", icon::MAGNIFYING_GLASS)).color(p.ink2))
                    .on_hover_text("Comandos, pasos, tableros y circuitos por su nombre (Ctrl+K)")
                    .clicked()
                {
                    self.commands.toggle();
                }
                ui.scope(|ui| {
                    let w = &mut ui.style_mut().visuals.widgets;
                    w.inactive.bg_stroke = egui::Stroke::new(1.0, p.line);
                    w.inactive.weak_bg_fill = p.panel;
                    if ui
                        .button(format!("{}  Cálculo rápido", icon::CALCULATOR))
                        .on_hover_text("Un circuito suelto, sin proyecto (F8)")
                        .clicked()
                    {
                        self.run_command(Command::Quick);
                    }
                });
            });
        });
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        if self.guided {
            self.guided_bar(ui);
            return;
        }
        let p = self.shared.palette;
        ui.horizontal(|ui| {
            // Menús de texto sin marco, como en las aplicaciones de escritorio.
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            ui.label(RichText::new(icon::LIGHTNING).size(17.0).color(p.accent));
            ui.label(strong("IEEStudio").size(14.0));
            ui.add_space(10.0);

            ui.menu_button("Archivo", |ui| self.file_menu(ui));

            ui.menu_button("Edición", |ui| {
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
                        egui::Button::new(format!("{}  {label}", icon::TRASH))
                            .shortcut_text("Supr"),
                    )
                    .clicked()
                {
                    ui.close();
                    self.doc.remove_selected();
                }
            });

            ui.menu_button("Ver", |ui| {
                let mut restyle = false;
                restyle |= ui
                    .radio_value(&mut self.dark, true, "Tema oscuro")
                    .changed();
                restyle |= ui
                    .radio_value(&mut self.dark, false, "Tema claro")
                    .changed();
                ui.separator();
                ui.label(muted(&p, "Color de acento"));
                for (i, (name, _)) in ACCENTS.iter().enumerate() {
                    restyle |= ui.radio_value(&mut self.accent, i, *name).changed();
                }
                if restyle {
                    self.restyle(ui.ctx());
                }
                ui.separator();
                ui.menu_button("Paneles", |ui| self.panel_checkboxes(ui));
                if ui
                    .button(format!("Restablecer «{}»", self.workspace.title()))
                    .on_hover_text("Vuelve a acomodar los paneles como al principio")
                    .clicked()
                {
                    ui.close();
                    self.dock = self.workspace.layout();
                }
            });

            ui.menu_button("Herramientas", |ui| {
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
                    workspace::open(&mut self.dock, Tab::Quick);
                }
            });

            ui.menu_button("Ventana", |ui| {
                self.panel_checkboxes(ui);
                ui.separator();
                ui.label(muted(&p, "Espacio de trabajo"));
                for ws in Workspace::ALL {
                    if ui.radio(self.workspace == ws, ws.title()).clicked() {
                        ui.close();
                        self.set_workspace(ws);
                    }
                }
                if ui
                    .button(format!("Restablecer «{}»", self.workspace.title()))
                    .clicked()
                {
                    ui.close();
                    self.dock = self.workspace.layout();
                }
            });

            ui.menu_button("Ayuda", |ui| {
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
            });

            // Lo que más se usa fuera de los menús: el cálculo de un circuito
            // suelto y la búsqueda de comandos.
            ui.add_space(14.0);
            if ui
                .button(RichText::new(format!("{}  Vista guiada", icon::LIST_NUMBERS)).color(p.accent_text))
                .on_hover_text("Volver a los cinco pasos: Proyecto, Tableros, Circuitos, Revisar y Memoria")
                .clicked()
            {
                self.guided = true;
            }
            ui.scope(|ui| {
                let w = &mut ui.style_mut().visuals.widgets;
                w.inactive.bg_stroke = egui::Stroke::new(1.0, p.line);
                w.inactive.weak_bg_fill = p.panel;
                if ui
                    .button(format!("{}  Cálculo rápido", icon::CALCULATOR))
                    .on_hover_text("Un circuito suelto, sin proyecto: caída de tensión, conductor e interruptor con su procedimiento (F8)")
                    .clicked()
                {
                    self.run_command(Command::Quick);
                }
                if ui
                    .button(
                        RichText::new(format!("{}  Buscar   Ctrl+K", icon::MAGNIFYING_GLASS))
                            .color(p.ink2),
                    )
                    .on_hover_text("Comandos, paneles, tableros y circuitos por su nombre")
                    .clicked()
                {
                    self.commands.toggle();
                }
            });

            // Espacios de trabajo a la derecha, como en las aplicaciones de Adobe.
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Tema claro u oscuro a un clic, como en las aplicaciones de Adobe.
                let (glyph, tip) = if self.dark {
                    (icon::SUN, "Cambiar a tema claro")
                } else {
                    (icon::MOON, "Cambiar a tema oscuro")
                };
                if ui
                    .add(egui::Button::new(RichText::new(glyph).size(15.0)).frame(false))
                    .on_hover_text(tip)
                    .clicked()
                {
                    self.dark = !self.dark;
                    self.restyle(ui.ctx());
                }
                ui.add_space(10.0);
                for ws in Workspace::ALL.iter().rev() {
                    let selected = self.workspace == *ws;
                    let text = if selected {
                        RichText::new(ws.title()).color(p.ink)
                    } else {
                        RichText::new(ws.title()).color(p.ink2)
                    };
                    let response = ui.add(egui::Button::new(text).frame(false));
                    if selected {
                        let r = response.rect;
                        ui.painter().hline(
                            r.x_range(),
                            r.bottom() + 3.0,
                            egui::Stroke::new(2.0, p.accent),
                        );
                    }
                    if response.clicked() && !selected {
                        self.set_workspace(*ws);
                    }
                }
                ui.label(muted(&p, icon::LAYOUT));
                ui.add_space(16.0);
                let file = self
                    .doc
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map_or_else(
                        // En el navegador el proyecto se guarda solo ahí.
                        || {
                            if files::WEB {
                                "en este navegador"
                            } else {
                                "sin guardar"
                            }
                            .to_string()
                        },
                        |n| n.to_string_lossy().into_owned(),
                    );
                let dirty = if self.doc.dirty { " •" } else { "" };
                ui.label(muted(&p, format!("{file}{dirty}")));
                ui.label(strong(&self.doc.project.name));
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
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
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                match &self.doc.result {
                    Ok(r) => verdict_chips(ui, &p, &r.verdict),
                    Err(e) => {
                        ui.label(
                            RichText::new(format!(
                                "{} No se pudo calcular: {e}",
                                icon::WARNING_CIRCLE
                            ))
                            .color(p.bad),
                        );
                    }
                }
            });
        });
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
                        if self.guided {
                            self.step = Step::Circuits;
                        }
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
                        if self.guided {
                            self.step = Step::Panels;
                        }
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
        let keys = [
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
        ];
        for (key, tab) in keys.into_iter().zip(Tab::ALL) {
            if pressed(Modifiers::NONE, key) {
                workspace::toggle(&mut self.dock, tab);
            }
        }
    }

    fn dock_style(&self, ui: &egui::Ui) -> Style {
        let p = &self.shared.palette;
        let mut style = Style::from_egui(ui.style().as_ref());
        style.dock_area_padding = None;
        style.main_surface_border_stroke = egui::Stroke::NONE;
        style.separator.width = 2.0;
        style.separator.color_idle = p.bg;
        style.separator.color_hovered = p.ink3;
        style.separator.color_dragged = p.accent;
        style.tab_bar.bg_fill = p.bg;
        style.tab_bar.height = 28.0;
        style.tab_bar.hline_color = p.bg;
        style.tab_bar.corner_radius = egui::CornerRadius::ZERO;
        style.tab.spacing = 2.0;
        style.tab.hline_below_active_tab_name = true;
        style.tab.tab_body.bg_fill = p.panel;
        style.tab.tab_body.stroke = egui::Stroke::NONE;
        style.tab.tab_body.inner_margin = egui::Margin::ZERO;
        style.tab.tab_body.corner_radius = egui::CornerRadius::ZERO;
        for (s, bg, text) in [
            (&mut style.tab.active, p.panel, p.ink),
            (&mut style.tab.focused, p.panel, p.ink),
            (&mut style.tab.inactive, p.bg, p.ink2),
            (&mut style.tab.hovered, p.raise, p.ink),
        ] {
            s.bg_fill = bg;
            s.text_color = text;
            s.outline_color = egui::Color32::TRANSPARENT;
            s.corner_radius = egui::CornerRadius {
                nw: 3,
                ne: 3,
                sw: 0,
                se: 0,
            };
        }
        style.buttons.close_tab_color = p.ink3;
        style.buttons.close_tab_active_color = p.ink;
        style.overlay.selection_color = p.accent.gamma_multiply(0.25);
        style
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
                ui.add_space(6.0);
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
                ui.label(RichText::new(icon::WARNING).size(20.0).color(p.warn));
                ui.label(strong("¿Guardar los cambios?").size(15.0));
            });
            ui.add_space(4.0);
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
            ui.add_space(10.0);
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
        if capture.frames == 8 {
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
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

        egui::Panel::top("menu")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(10, 5)),
            )
            .show(ui, |ui| self.menu_bar(ui));
        egui::Panel::bottom("estado")
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show(ui, |ui| self.status_bar(ui));

        let style = self.dock_style(ui);
        let revision = self.doc.revision;
        let mut viewer = Viewer {
            doc: &mut self.doc,
            shared: &mut self.shared,
            export_report: false,
        };
        let empty = workspace::main_is_empty(&self.dock);
        let mut restore = None;
        let mut start = None;
        let home = self.home;
        let recent = self.recent.clone();
        let guided_view = self.guided;
        let step = self.step;
        let mut guided_out = (None, None);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| {
                if home {
                    start = home_screen(ui, &p, &recent, &viewer.doc.project.name);
                    return;
                }
                if guided_view {
                    guided_out = guided::show(ui, step, viewer.doc, viewer.shared);
                    return;
                }
                let rect = ui.max_rect();
                DockArea::new(&mut self.dock)
                    .style(style)
                    .show_add_buttons(false)
                    .show_leaf_collapse_buttons(false)
                    .show_leaf_close_all_buttons(false)
                    .show_inside(ui, &mut viewer);
                if empty {
                    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                    restore = no_panels(&mut child, &p, self.workspace);
                }
            });

        match restore {
            Some(Restore::Workspace) => self.dock = self.workspace.layout(),
            Some(Restore::Panel(tab)) => workspace::open(&mut self.dock, tab),
            None => {}
        }
        let export = viewer.export_report;
        let (go, request) = guided_out;
        if let Some(step) = go {
            self.step = step;
        }
        match request {
            Some(guided::Export::Pdf) => self.export_pdf(false),
            Some(guided::Export::Excel) => self.export_excel(),
            Some(guided::Export::Dxf) => self.export_dxf(),
            None => {}
        }
        match start {
            Some(Start::New) => self.request(Pending::New),
            Some(Start::Open) => self.request(Pending::Open),
            Some(Start::Example) => self.request(Pending::Example),
            Some(Start::Quick) => self.run_command(Command::Quick),
            Some(Start::Recent(path)) => self.request(Pending::OpenPath(path)),
            Some(Start::Back) => self.home = false,
            None => {}
        }
        if self.doc.revision != revision {
            self.done = None;
        }
        if export {
            self.export_pdf(false);
        }
        if self.pending.is_none() {
            if let Some(command) = self.commands.show(&ctx, &p, &self.doc) {
                self.run_command(command);
            }
        }
        self.chooser_window(&ctx);
        self.about_window(&ctx);
        self.confirm_window(&ctx);
        self.import_window(&ctx);
        if let Some(tab) = self.shared.open_tab.take() {
            workspace::open(&mut self.dock, tab);
        }
        if let Some(done) = self.shared.done.take() {
            self.notice = None;
            self.done = Some(done);
        }
        if self.quick_window {
            let mut open = true;
            egui::Window::new(format!("{}  Cálculo rápido", icon::CALCULATOR))
                .open(&mut open)
                .default_size([1100.0, 700.0])
                .default_pos([160.0, 70.0])
                .collapsible(false)
                .show(&ctx, |ui| self.shared.quick.ui(ui, &p));
            self.quick_window = open;
        }
        if let Some(input) = self.shared.quick.take_request() {
            self.quick_to_project(&input);
            if self.guided {
                self.quick_window = false;
                self.step = Step::Circuits;
            }
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
        self.handle_capture(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let prefs = Prefs {
            dark: self.dark,
            accent: self.accent,
            workspace: self.workspace,
            dock: self.dock.clone(),
            columns: self.shared.columns.clone(),
            diagram_zoom: self.shared.diagram_zoom,
            recent: self.recent.clone(),
            guided: self.guided,
            step: self.step,
        };
        if let Ok(text) = serde_json::to_string(&prefs) {
            storage.set_string(STORAGE_KEY, text);
        }
        if files::WEB {
            storage.set_string(WEB_PROJECT_KEY, self.doc.project.to_json());
        }
    }
}

/// Qué volver a abrir desde la pantalla sin paneles.
enum Restore {
    Workspace,
    Panel(Tab),
}

/// Pantalla para cuando el usuario cerró todos los paneles: explica cómo
/// volver y ofrece hacerlo con un clic.
fn no_panels(ui: &mut egui::Ui, p: &Palette, ws: Workspace) -> Option<Restore> {
    let mut chosen = None;
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(ui.available_height() * 0.28);
        ui.label(RichText::new(icon::SQUARES_FOUR).size(34.0).color(p.ink3));
        ui.add_space(4.0);
        ui.label(strong("No hay paneles abiertos").size(16.0));
        ui.label(muted(
            p,
            "Los paneles se abren desde el menú Ventana o con las teclas F2 a F8.",
        ));
        ui.add_space(12.0);
        let button = egui::Button::new(
            RichText::new(format!(
                "{}  Restablecer el espacio «{}»",
                icon::ARROW_COUNTER_CLOCKWISE,
                ws.title()
            ))
            .color(p.on_accent),
        )
        .fill(p.accent);
        if ui.add(button).clicked() {
            chosen = Some(Restore::Workspace);
        }
        ui.add_space(10.0);
        ui.label(muted(p, "o abre solo un panel:"));
        ui.add_space(2.0);
        // Centra la fila de botones midiendo primero su ancho.
        let id = ui.id().with("ancho-botones");
        let width: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(600.0);
        ui.allocate_ui_with_layout(
            egui::vec2(width, 26.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                let start = ui.cursor().left();
                for tab in Tab::ALL {
                    if ui
                        .button(format!("{} {}", tab.icon(), tab.title()))
                        .on_hover_text(tab.shortcut())
                        .clicked()
                    {
                        chosen = Some(Restore::Panel(tab));
                    }
                }
                let used = ui.cursor().left() - start;
                ui.ctx().data_mut(|d| d.insert_temp(id, used));
            },
        );
    });
    chosen
}

/// Qué se eligió en la pantalla de inicio.
enum Start {
    New,
    Open,
    Example,
    Quick,
    Recent(PathBuf),
    /// Volver al proyecto abierto.
    Back,
}

/// Tarjeta de la pantalla de inicio: ícono, título y una línea de qué hace.
fn start_card(ui: &mut egui::Ui, p: &Palette, glyph: &str, title: &str, hint: &str) -> bool {
    let size = egui::vec2(172.0, 118.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hovered = response.hovered();
    ui.painter().rect(
        rect,
        6.0,
        if hovered { p.raise } else { p.panel },
        egui::Stroke::new(1.0, if hovered { p.ink3 } else { p.line }),
        egui::StrokeKind::Inside,
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(14.0))
            .layout(Layout::top_down(Align::Min)),
    );
    child.label(RichText::new(glyph).size(24.0).color(p.accent_text));
    child.add_space(6.0);
    child.label(strong(title).size(14.0));
    child.add(egui::Label::new(muted(p, hint).size(11.0)).wrap());
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

/// Pantalla de inicio: lo que se puede hacer al abrir el programa, sin
/// paneles vacíos alrededor.
fn home_screen(
    ui: &mut egui::Ui,
    p: &Palette,
    recent: &[PathBuf],
    open_name: &str,
) -> Option<Start> {
    let mut chosen = None;
    let width = 4.0 * 172.0 + 3.0 * 12.0;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.12).max(24.0));
        ui.label(RichText::new(icon::LIGHTNING).size(40.0).color(p.accent));
        ui.label(strong("IEEStudio").size(28.0));
        ui.label(muted(
            p,
            "Cálculo de instalaciones eléctricas de baja tensión · NOM-001-SEDE-2012",
        ));
        ui.add_space(26.0);
    });
    let left = ((ui.available_width() - width) / 2.0).max(16.0);
    ui.horizontal(|ui| {
        ui.add_space(left);
        ui.spacing_mut().item_spacing.x = 12.0;
        if start_card(
            ui,
            p,
            icon::FILE_PLUS,
            "Nuevo proyecto",
            "Un tablero vacío para empezar · Ctrl+N",
        ) {
            chosen = Some(Start::New);
        }
        if start_card(
            ui,
            p,
            icon::FOLDER_OPEN,
            "Abrir proyecto",
            "Un archivo .iees que ya tienes · Ctrl+O",
        ) {
            chosen = Some(Start::Open);
        }
        if start_card(
            ui,
            p,
            icon::HOUSE,
            "Proyecto de ejemplo",
            "Una casa con seis circuitos para conocer el programa",
        ) {
            chosen = Some(Start::Example);
        }
        if start_card(
            ui,
            p,
            icon::CALCULATOR,
            "Cálculo rápido",
            "Un circuito suelto con su procedimiento · F8",
        ) {
            chosen = Some(Start::Quick);
        }
    });
    ui.add_space(26.0);
    let existing: Vec<&PathBuf> = recent.iter().filter(|p| p.exists()).take(6).collect();
    if !files::WEB && !existing.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(left);
            ui.vertical(|ui| {
                ui.set_width(width);
                ui.label(
                    RichText::new("RECIENTES")
                        .size(10.5)
                        .color(p.ink3)
                        .family(theme::semibold()),
                );
                ui.add_space(2.0);
                for path in existing {
                    let name = path.file_name().map_or_else(
                        || path.display().to_string(),
                        |n| n.to_string_lossy().into_owned(),
                    );
                    let folder = path
                        .parent()
                        .map_or_else(String::new, |d| d.display().to_string());
                    let row = ui
                        .horizontal(|ui| {
                            ui.label(RichText::new(icon::FILE).color(p.ink2));
                            ui.label(RichText::new(name));
                            ui.label(muted(p, folder).size(11.0));
                        })
                        .response
                        .interact(egui::Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if row.clicked() {
                        chosen = Some(Start::Recent(path.clone()));
                    }
                }
            });
        });
        ui.add_space(18.0);
    }
    ui.vertical_centered(|ui| {
        if ui
            .button(format!("{}  Volver a «{open_name}»", icon::ARROW_LEFT))
            .clicked()
        {
            chosen = Some(Start::Back);
        }
        ui.add_space(14.0);
        ui.label(
            muted(
                p,
                "Ctrl+K busca cualquier comando, tablero o circuito. IEEStudio calcula y verifica; la aprobación es de quien firma.",
            )
            .size(11.0),
        );
    });
    chosen
}
