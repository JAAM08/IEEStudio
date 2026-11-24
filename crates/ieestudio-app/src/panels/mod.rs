//! Los paneles de las regiones del marco y lo que comparten.

pub mod browser;
pub mod diagram;
pub mod procedure;
pub mod project;
pub mod properties;
pub mod quick;
pub mod report;
pub mod review;
pub mod schedule;

use eframe::egui;
use ieestudio_diagrams::{single_line_with, Mode, Options};

use crate::state::{Doc, Selection};
use crate::theme::Palette;
use crate::views::Panel;

/// Estado de la interfaz que no es del proyecto.
pub struct Shared {
    pub palette: Palette,
    pub columns: schedule::Columns,
    /// Juego de columnas del cuadro: «Esencial» o «Completo».
    pub column_set: schedule::ColumnSet,
    pub filter: String,
    pub status_filter: schedule::StatusFilter,
    pub props_tab: properties::PropsTab,
    pub diagram_zoom: f32,
    pub quick: quick::QuickView,
    pub preview: report::Preview,
    /// Aviso de una acción terminada, para la barra de estado.
    pub done: Option<String>,
    /// Otro panel pidió abrir el procedimiento, por ejemplo desde la barra
    /// contextual del cuadro de cargas.
    pub open_procedure: bool,
    /// Tablero al que se va a agregar un circuito: mientras tenga valor está
    /// abierta la ventana «¿Qué vas a conectar?».
    pub choose_load: Option<usize>,
    /// Unifilar como revisión (colores) o como plano (hoja para imprimir).
    pub diagram_mode: ieestudio_diagrams::Mode,
    /// Un panel pidió ver el unifilar completo: el marco pasa a esa vista.
    pub diagram_full: bool,
    /// Pendiente de ajustar el unifilar a la pantalla al abrirlo.
    pub diagram_fit: bool,
    /// Lo que Revisar arma una vez por revisión del proyecto.
    pub review: review::Cache,
    /// Teléfono: la ficha del elemento elegido ocupa la pantalla.
    pub ficha_open: bool,
    /// Teléfono: hoja inferior del unifilar con el tablero o circuito tocado.
    pub sheet: Option<(usize, Option<usize>)>,
    /// Tableta: cajón del navegador abierto.
    pub navigator_drawer: bool,
    /// Tableta: cajón de la ficha abierto.
    pub inspector_drawer: bool,
    /// Último unifilar dibujado, con la revisión del proyecto y el modo: se
    /// reconstruye solo cuando cambian, no en cada cuadro.
    diagram_cache: Option<(u64, Mode, std::rc::Rc<ieestudio_diagrams::Diagram>)>,
}

impl Shared {
    pub fn new(palette: Palette) -> Self {
        Shared {
            palette,
            columns: schedule::Columns::default(),
            column_set: schedule::ColumnSet::Essential,
            filter: String::new(),
            status_filter: schedule::StatusFilter::All,
            props_tab: properties::PropsTab::Inputs,
            diagram_zoom: 1.3,
            quick: quick::QuickView::default(),
            done: None,
            open_procedure: false,
            choose_load: None,
            diagram_mode: ieestudio_diagrams::Mode::Review,
            diagram_full: false,
            diagram_fit: false,
            diagram_cache: None,
            review: review::Cache::default(),
            ficha_open: false,
            sheet: None,
            navigator_drawer: false,
            inspector_drawer: false,
            preview: report::Preview::default(),
        }
    }
}

impl Shared {
    /// El unifilar del proyecto en el modo pedido. `None` si el proyecto no
    /// se pudo calcular.
    pub fn diagram(
        &mut self,
        doc: &Doc,
        mode: Mode,
    ) -> Option<std::rc::Rc<ieestudio_diagrams::Diagram>> {
        let result = doc.result.as_ref().ok()?;
        if let Some((revision, cached, diagram)) = &self.diagram_cache {
            if *revision == doc.revision && *cached == mode {
                return Some(diagram.clone());
            }
        }
        let options = match mode {
            Mode::Review => Options::REVIEW,
            Mode::Plan => Options::PLAN,
        };
        let diagram = std::rc::Rc::new(single_line_with(&doc.project, result, options));
        self.diagram_cache = Some((doc.revision, mode, diagram.clone()));
        Some(diagram)
    }
}

/// Dibuja cada panel con acceso al proyecto y al estado compartido.
pub struct Viewer<'a> {
    pub doc: &'a mut Doc,
    pub shared: &'a mut Shared,
}

fn padded(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, add);
}

impl Viewer<'_> {
    pub fn panel(&mut self, ui: &mut egui::Ui, panel: Panel) {
        let p = self.shared.palette;
        match panel {
            Panel::Browser => padded(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("navegador")
                    .auto_shrink([false, false])
                    .show(ui, |ui| browser::show(ui, self.doc, self.shared));
            }),
            Panel::Schedule => schedule::show(ui, self.doc, self.shared),
            Panel::Properties => padded(ui, |ui| properties::show(ui, self.doc, self.shared)),
            Panel::Diagram => padded(ui, |ui| {
                match self.shared.diagram(self.doc, self.shared.diagram_mode) {
                    Some(d) => {
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
                    None => crate::widgets::empty_state(
                        ui,
                        &p,
                        egui_phosphor::regular::WARNING_CIRCLE,
                        "No se pudo calcular el proyecto",
                        &self.doc.calc_error().unwrap_or_default(),
                    ),
                }
            }),
        }
    }
}
