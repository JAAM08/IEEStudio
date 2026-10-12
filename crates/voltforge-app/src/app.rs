//! Ventana principal: menús, espacios de trabajo, paneles acoplables y barra
//! de estado alrededor de un único proyecto.

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_phosphor::regular as icon;
use serde::{Deserialize, Serialize};
use voltforge_project::{example, Project};

use crate::panels::{schedule, Shared, Viewer};
use crate::state::{empty_project, Doc, Selection};
use crate::theme::{self, Palette, ACCENTS};
use crate::widgets::{muted, strong, verdict_chips};
use crate::workspace::{Tab, Workspace};

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
}

pub struct Options {
    pub open: Option<PathBuf>,
    pub capture: Option<PathBuf>,
    pub light: bool,
    pub workspace: Option<Workspace>,
    pub focus: Option<Tab>,
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
        let (dark, accent, workspace, dock, columns, zoom) = match prefs {
            Some(p) => (
                p.dark,
                p.accent.min(ACCENTS.len() - 1),
                p.workspace,
                p.dock,
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
        };
        app.doc.selection = Selection::Circuit(0, 1);
        if let Some(path) = options.open {
            app.load(path);
        }
        app.shared.props_tab = match options.properties.as_deref() {
            Some("resultados") => crate::panels::properties::PropsTab::Results,
            Some("normativa") => crate::panels::properties::PropsTab::Normative,
            _ => crate::panels::properties::PropsTab::Inputs,
        };
        if let Some(tab) = options.focus {
            app.focus_tab(tab);
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
                self.set_project(project, Some(path));
            }
            Err(message) => self.notice = Some(message),
        }
    }

    fn save(&mut self, ask: bool) {
        let target = match (&self.doc.path, ask) {
            (Some(path), false) => Some(path.clone()),
            _ => rfd::FileDialog::new()
                .add_filter("Proyecto de VoltForge", &["json"])
                .set_file_name("proyecto.json")
                .save_file(),
        };
        let Some(path) = target else {
            return;
        };
        match std::fs::write(&path, self.doc.project.to_json()) {
            Ok(()) => {
                self.done = Some(format!("Guardado en {}", path.display()));
                self.doc.path = Some(path);
                self.doc.dirty = false;
                self.notice = None;
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Proyecto de VoltForge", &["json"])
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

    // ------------------------------------------------------------ paneles

    fn set_workspace(&mut self, ws: Workspace) {
        self.workspace = ws;
        self.dock = ws.layout();
    }

    fn is_open(&self, tab: Tab) -> bool {
        self.dock.find_tab(&tab).is_some()
    }

    /// Abre o cierra un panel. Los laterales vuelven a su lado.
    fn toggle_tab(&mut self, tab: Tab) {
        if let Some(path) = self.dock.find_tab(&tab) {
            self.dock.remove_tab(path);
            return;
        }
        let tree = self.dock.main_surface_mut();
        match tab {
            Tab::Browser => {
                tree.split_left(NodeIndex::root(), 0.16, vec![tab]);
            }
            Tab::Properties => {
                tree.split_right(NodeIndex::root(), 0.76, vec![tab]);
            }
            Tab::Procedure => {
                tree.split_below(NodeIndex::root(), 0.68, vec![tab]);
            }
            _ => {
                // Los documentos van junto a los otros documentos, al centro.
                let center = [Tab::Schedule, Tab::Diagram, Tab::Report, Tab::Quick]
                    .iter()
                    .find_map(|t| self.dock.find_tab(t));
                match center {
                    Some(path) => {
                        self.dock.set_focused_node_and_surface(path.node_path());
                        self.dock.push_to_focused_leaf(tab);
                    }
                    None => self.dock.push_to_first_leaf(tab),
                }
            }
        }
    }

    fn focus_tab(&mut self, tab: Tab) {
        if tab == Tab::Quick && !self.is_open(tab) {
            // El cálculo rápido no es parte del proyecto: abre en su propia
            // ventana flotante, que se puede acoplar arrastrando la pestaña.
            let surface = self.dock.add_window(vec![tab]);
            if let Some(window) = self.dock.get_window_state_mut(surface) {
                window
                    .set_position(egui::pos2(180.0, 70.0))
                    .set_size(egui::vec2(1120.0, 720.0));
            }
            return;
        }
        if !self.is_open(tab) {
            self.toggle_tab(tab);
        }
        if let Some(path) = self.dock.find_tab(&tab) {
            let _ = self.dock.set_active_tab(path);
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
                    .button(format!("{}  Nuevo proyecto", icon::FILE_PLUS))
                    .clicked()
                {
                    ui.close();
                    self.set_project(empty_project(), None);
                }
                if ui
                    .add(
                        egui::Button::new(format!("{}  Abrir…", icon::FOLDER_OPEN))
                            .shortcut_text("Ctrl+O"),
                    )
                    .clicked()
                {
                    ui.close();
                    self.open_dialog();
                }
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
                ui.separator();
                if ui.button("     Abrir el proyecto de ejemplo").clicked() {
                    ui.close();
                    self.set_project(example::house(), None);
                }
            });

            ui.menu_button("Edición", |ui| {
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
                    self.focus_tab(Tab::Quick);
                }
            });

            ui.menu_button("Ventana", |ui| {
                for tab in Tab::ALL {
                    let mut open = self.is_open(tab);
                    if ui
                        .add(egui::Checkbox::new(
                            &mut open,
                            format!("{}  {}", tab.icon(), tab.title()),
                        ))
                        .on_hover_text(tab.shortcut())
                        .changed()
                    {
                        self.toggle_tab(tab);
                    }
                }
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
                if let Ok(r) = &self.doc.result {
                    verdict_chips(ui, &p, &r.verdict);
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
            self.open_dialog();
        }
        if pressed(Modifiers::COMMAND, Key::E) {
            self.export_pdf(false);
        }
        let typing = ctx.memory(|m| m.focused().is_some());
        if !typing {
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
                self.toggle_tab(tab);
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
        self.shortcuts(ui.ctx());

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
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| {
                DockArea::new(&mut self.dock)
                    .style(style)
                    .show_add_buttons(false)
                    .show_leaf_collapse_buttons(false)
                    .show_leaf_close_all_buttons(false)
                    .show_inside(ui, &mut viewer);
            });
        let export = viewer.export_report;
        if self.doc.revision != revision {
            self.done = None;
        }
        if export {
            self.export_pdf(false);
        }
        self.about_window(ui.ctx());
        self.handle_capture(ui.ctx());
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let prefs = Prefs {
            dark: self.dark,
            accent: self.accent,
            workspace: self.workspace,
            dock: self.dock.clone(),
            columns: self.shared.columns.clone(),
            diagram_zoom: self.shared.diagram_zoom,
        };
        if let Ok(text) = serde_json::to_string(&prefs) {
            storage.set_string(STORAGE_KEY, text);
        }
    }
}
