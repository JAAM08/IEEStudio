//! Los paneles acoplables y lo que comparten.

pub mod browser;
pub mod diagram;
pub mod procedure;
pub mod properties;
pub mod quick;
pub mod report;
pub mod schedule;

use eframe::egui::{self, Id, RichText, WidgetText};
use egui_dock::TabViewer;
use ieestudio_diagrams::{single_line_with, Mode, Options};

use crate::state::{Doc, Selection};
use crate::theme::Palette;
use crate::workspace::Tab;

/// Estado de la interfaz que no es del proyecto.
pub struct Shared {
    pub palette: Palette,
    pub columns: schedule::Columns,
    pub filter: String,
    pub status_filter: schedule::StatusFilter,
    pub props_tab: properties::PropsTab,
    pub diagram_zoom: f32,
    pub quick: quick::QuickView,
    pub preview: report::Preview,
    /// Aviso de una acción terminada, para la barra de estado.
    pub done: Option<String>,
    /// Panel que otro panel pidió abrir, por ejemplo el procedimiento desde
    /// la barra contextual del cuadro de cargas.
    pub open_tab: Option<Tab>,
    /// Tablero al que se va a agregar un circuito: mientras tenga valor está
    /// abierta la ventana «¿Qué vas a conectar?».
    pub choose_load: Option<usize>,
    /// Unifilar como revisión (colores) o como plano (hoja para imprimir).
    pub diagram_mode: ieestudio_diagrams::Mode,
}

impl Shared {
    pub fn new(palette: Palette) -> Self {
        Shared {
            palette,
            columns: schedule::Columns::default(),
            filter: String::new(),
            status_filter: schedule::StatusFilter::All,
            props_tab: properties::PropsTab::Inputs,
            diagram_zoom: 1.3,
            quick: quick::QuickView::default(),
            done: None,
            open_tab: None,
            choose_load: None,
            diagram_mode: ieestudio_diagrams::Mode::Review,
            preview: report::Preview::default(),
        }
    }
}

/// Dibuja cada panel con acceso al proyecto y al estado compartido.
pub struct Viewer<'a> {
    pub doc: &'a mut Doc,
    pub shared: &'a mut Shared,
    /// El panel de la memoria pidió exportar el PDF.
    pub export_report: bool,
}

fn padded(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, add);
}

impl TabViewer for Viewer<'_> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Tab) -> Id {
        Id::new(("panel", *tab))
    }

    fn title(&mut self, tab: &mut Tab) -> WidgetText {
        RichText::new(format!("{}  {}", tab.icon(), tab.title())).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        let p = self.shared.palette;
        match tab {
            Tab::Browser => padded(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("navegador")
                    .auto_shrink([false, false])
                    .show(ui, |ui| browser::show(ui, self.doc, self.shared));
            }),
            Tab::Schedule => schedule::show(ui, self.doc, self.shared),
            Tab::Properties => padded(ui, |ui| properties::show(ui, self.doc, self.shared)),
            Tab::Procedure => padded(ui, |ui| procedure::show(ui, self.doc, self.shared)),
            Tab::Diagram => padded(ui, |ui| match &self.doc.result {
                Ok(result) => {
                    let options = match self.shared.diagram_mode {
                        Mode::Review => Options::REVIEW,
                        Mode::Plan => Options::PLAN,
                    };
                    let d = single_line_with(&self.doc.project, result, options);
                    let selected = match self.doc.selection {
                        Selection::Feeder(i) | Selection::Secondary(i) => Some((i, None)),
                        Selection::Circuit(i, c) => Some((i, Some(c))),
                        Selection::Project => None,
                    };
                    if let Some(hit) = diagram::show(
                        ui,
                        &p,
                        &d,
                        &mut self.shared.diagram_zoom,
                        &mut self.shared.diagram_mode,
                        selected,
                    ) {
                        self.doc.selection = match hit {
                            (i, Some(c)) => Selection::Circuit(i, c),
                            (i, None) => Selection::Feeder(i),
                        };
                    }
                }
                Err(_) => crate::widgets::empty_state(
                    ui,
                    &p,
                    egui_phosphor::regular::WARNING_CIRCLE,
                    "No se pudo calcular el proyecto",
                    &self.doc.calc_error().unwrap_or_default(),
                ),
            }),
            Tab::Report => padded(ui, |ui| {
                self.export_report |= report::show(ui, self.doc, self.shared);
            }),
            Tab::Quick => self.shared.quick.ui(ui, &p),
        }
    }

    fn scroll_bars(&self, _tab: &Tab) -> [bool; 2] {
        [false, false]
    }
}
