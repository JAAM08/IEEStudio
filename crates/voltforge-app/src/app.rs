//! Ventana principal: menús, espacios de trabajo, paneles acoplables y barra
//! de estado alrededor de un único proyecto.

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};
use egui_dock::{DockArea, DockState, Style};
use egui_phosphor::regular as icon;
use serde::{Deserialize, Serialize};
use voltforge_project::{example, Project};

use crate::panels::{schedule, Shared, Viewer};
use crate::state::{empty_project, Doc, Selection};
use crate::theme::{self, Palette, ACCENTS};
use crate::widgets::{muted, strong, verdict_chips};
use crate::workspace::{self, Tab, Workspace};

const STORAGE_KEY: &str = "voltforge-interfaz-1";

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
}

/// Cuántos proyectos recientes se recuerdan.
const MAX_RECENT: usize = 8;

/// Extensión de los proyectos. Por dentro es JSON; la extensión propia
/// permite abrirlos con doble clic sin adueñarse de todos los `.json`.
pub const EXTENSION: &str = "vfp";

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
    imported: Option<(voltforge_exchange::ImportReport, Project)>,
    /// El usuario ya decidió cerrar; no se vuelve a preguntar.
    allow_close: bool,
    /// Cerrar la ventana al terminar este cuadro.
    quit: bool,
    recent: Vec<PathBuf>,
}

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
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, options: Options) -> Self {
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
        };
        app.doc.selection = Selection::Circuit(0, 1);
        // Abre lo que se pidió; si no, el último proyecto, si todavía existe.
        let last = app.recent.first().filter(|p| p.exists()).cloned();
        if let Some(path) = options.open.or(last) {
            app.load(path);
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
        app
    }

    // ------------------------------------------------------------ archivos

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.doc = Doc::new(project, path);
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
        let target = match (&self.doc.path, ask) {
            (Some(path), false) => Some(path.clone()),
            _ => rfd::FileDialog::new()
                .add_filter("Proyecto de VoltForge", &[EXTENSION])
                .set_file_name(self.suggested_name())
                .save_file(),
        };
        let Some(path) = target else {
            return;
        };
        match std::fs::write(&path, self.doc.project.to_json()) {
            Ok(()) => {
                self.done = Some(format!("Guardado en {}", path.display()));
                self.remember(&path);
                self.doc.path = Some(path);
                self.doc.dirty = false;
                self.notice = None;
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Proyecto de VoltForge", &[EXTENSION, "json"])
            .pick_file()
        {
            self.load(path);
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
                    .and_then(|r| voltforge_reports::project_pdf(&self.doc.project, r)),
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
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name(name)
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, pdf) {
            Ok(()) => {
                self.notice = None;
                self.done = Some(format!("PDF guardado en {}", path.display()));
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    /// Agrega el circuito del cálculo rápido al tablero seleccionado.
    fn quick_to_project(&mut self, input: &voltforge_project::quick::QuickInput) {
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
        let bytes = voltforge_diagrams::single_line(&self.doc.project, result).to_dxf();
        let stem = self.suggested_name();
        let stem = stem.trim_end_matches(&format!(".{EXTENSION}"));
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Dibujo DXF", &["dxf"])
            .set_file_name(format!("{stem} - unifilar.dxf"))
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, bytes) {
            Ok(()) => {
                self.notice = None;
                self.done = Some(format!("Unifilar guardado en {}", path.display()));
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    /// Cuadro de cargas a Excel: tableros, circuitos y resultados.
    fn export_excel(&mut self) {
        let bytes =
            match voltforge_exchange::export_xlsx(&self.doc.project, self.doc.result.as_ref().ok())
            {
                Ok(b) => b,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let stem = self.suggested_name();
        let stem = stem.trim_end_matches(&format!(".{EXTENSION}"));
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Libro de Excel", &["xlsx"])
            .set_file_name(format!("{stem} - cuadro de cargas.xlsx"))
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, bytes) {
            Ok(()) => {
                self.notice = None;
                self.done = Some(format!("Cuadro de cargas guardado en {}", path.display()));
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    /// Agrega o actualiza tableros y circuitos desde un libro de Excel.
    fn import_excel(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Libro de Excel", &["xlsx"])
            .pick_file()
        else {
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.notice = Some(format!("No se pudo leer {}: {e}", path.display()));
                return;
            }
        };
        let before = self.doc.project.clone();
        match voltforge_exchange::import_xlsx(&mut self.doc.project, &bytes) {
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
            Pending::New => self.set_project(empty_project(), None),
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

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let p = self.shared.palette;
        ui.horizontal(|ui| {
            // Menús de texto sin marco, como en las aplicaciones de escritorio.
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            ui.label(RichText::new(icon::LIGHTNING).size(17.0).color(p.accent));
            ui.label(strong("VoltForge").size(14.0));
            ui.add_space(10.0);

            ui.menu_button("Archivo", |ui| {
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
                        egui::Button::new(format!("{}  Abrir…", icon::FOLDER_OPEN))
                            .shortcut_text("Ctrl+O"),
                    )
                    .clicked()
                {
                    ui.close();
                    self.request(Pending::Open);
                }
                let recent: Vec<PathBuf> = self.recent.clone();
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
                                .on_disabled_hover_text(format!(
                                    "Ya no existe: {}",
                                    path.display()
                                ));
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
                if ui
                    .add(
                        egui::Button::new(format!("{}  Guardar", icon::FLOPPY_DISK))
                            .shortcut_text("Ctrl+S"),
                    )
                    .clicked()
                {
                    ui.close();
                    self.save(false);
                }
                if ui
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
                    .on_hover_text(
                        "Agrega o actualiza tableros y circuitos por su ID. No borra nada.",
                    )
                    .clicked()
                {
                    ui.close();
                    self.import_excel();
                }
                ui.separator();
                if ui.button("     Abrir el proyecto de ejemplo").clicked() {
                    ui.close();
                    self.request(Pending::Example);
                }
                ui.separator();
                if ui
                    .add(egui::Button::new("     Salir").shortcut_text("Alt+F4"))
                    .clicked()
                {
                    ui.close();
                    self.request(Pending::Quit);
                }
            });

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
                    if let Some(p) = panel {
                        self.doc.add_circuit(p);
                    }
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
                if ui.button("Acerca de VoltForge").clicked() {
                    ui.close();
                    self.about = true;
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
                    .map_or("sin guardar".to_string(), |n| {
                        n.to_string_lossy().into_owned()
                    });
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
        egui::Window::new("Acerca de VoltForge")
            .open(&mut self.about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(strong(format!("VoltForge {}", env!("CARGO_PKG_VERSION"))));
                ui.label("Cálculo de instalaciones eléctricas de baja tensión. Código abierto, MIT o Apache-2.0.");
                ui.add_space(6.0);
                ui.label(muted(&p, "Perfil normativo: NOM-001-SEDE-2012."));
                ui.label(muted(&p, "VoltForge calcula y verifica; no otorga aprobación profesional. Los resultados debe revisarlos y firmarlos un profesional."));
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
            ui.label(format!(
                "«{}» tiene cambios sin guardar. Si continúas sin guardar, se pierden.",
                self.doc.project.name
            ));
            ui.add_space(10.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let save = egui::Button::new(
                    RichText::new(format!("{}  Guardar", icon::FLOPPY_DISK)).color(p.on_accent),
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
        if self.pending.is_none() {
            self.shortcuts(&ctx);
        }

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
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| {
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
        if self.doc.revision != revision {
            self.done = None;
        }
        if export {
            self.export_pdf(false);
        }
        self.about_window(&ctx);
        self.confirm_window(&ctx);
        self.import_window(&ctx);
        if let Some(input) = self.shared.quick.take_request() {
            self.quick_to_project(&input);
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
        };
        if let Ok(text) = serde_json::to_string(&prefs) {
            storage.set_string(STORAGE_KEY, text);
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
