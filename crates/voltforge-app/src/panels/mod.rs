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
use voltforge_diagrams::single_line;

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
                    let d = single_line(&self.doc.project, result);
                    let selected = match self.doc.selection {
                        Selection::Feeder(i) => Some((i, None)),
                        Selection::Circuit(i, c) => Some((i, Some(c))),
                        Selection::Project => None,
                    };
                    if let Some(hit) =
                        diagram::show(ui, &p, &d, &mut self.shared.diagram_zoom, selected)
                    {
                        self.doc.selection = match hit {
                            (i, Some(c)) => Selection::Circuit(i, c),
                            (i, None) => Selection::Feeder(i),
                        };
                    }
                }
                Err(e) => {
                    ui.label(RichText::new(e).color(p.bad));
                }
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
