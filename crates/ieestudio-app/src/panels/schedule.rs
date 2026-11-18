//! Cuadro de cargas: el área de trabajo principal.
//!
//! Escala a cientos de circuitos: solo se dibujan las filas visibles, hay
//! filtro de texto y de pendientes, dos juegos de columnas («Esencial» y
//! «Completo», este último configurable) y la tabla se desplaza en horizontal
//! cuando no cabe.

use eframe::egui::{self, Align, Layout, RichText, Sense, Vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use ieestudio_project::{LoadKind, RunResult, Status};
use serde::{Deserialize, Serialize};

use super::Shared;
use crate::state::{Doc, Selection};
use crate::symbols::{self, Symbol};
use crate::theme::Palette;
use crate::widgets::{
    label_of, muted, primary_button, segmented, status_color, status_text, strong, worst, KINDS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Col {
    Id,
    Name,
    Kind,
    Connection,
    Phases,
    Load,
    PowerFactor,
    Current,
    Design,
    Length,
    Size,
    Ground,
    Conduit,
    Breaker,
    Ampacity,
    Drop,
    Cumulative,
    Status,
}

impl Col {
    pub const ALL: [Col; 18] = [
        Col::Id,
        Col::Name,
        Col::Kind,
        Col::Connection,
        Col::Phases,
        Col::Load,
        Col::PowerFactor,
        Col::Current,
        Col::Design,
        Col::Length,
        Col::Size,
        Col::Ground,
        Col::Conduit,
        Col::Breaker,
        Col::Ampacity,
        Col::Drop,
        Col::Cumulative,
        Col::Status,
    ];

    fn header(self) -> &'static str {
        match self {
            Col::Id => "ID",
            Col::Name => "Descripción",
            Col::Kind => "Tipo",
            Col::Connection => "Conexión",
            Col::Phases => "Fases",
            Col::Load => "Carga W",
            Col::PowerFactor => "fp",
            Col::Current => "I A",
            Col::Design => "Id A",
            Col::Length => "L m",
            Col::Size => "Calibre",
            Col::Ground => "Tierra",
            Col::Conduit => "Tubo",
            Col::Breaker => "ITM",
            Col::Ampacity => "Amp. A",
            Col::Drop => "e %",
            Col::Cumulative => "Σe %",
            Col::Status => "Estado",
        }
    }

    /// Nombre completo para el menú de columnas.
    fn description(self) -> &'static str {
        match self {
            Col::Id => "Identificador",
            Col::Name => "Descripción",
            Col::Kind => "Tipo de carga",
            Col::Connection => "Conexión",
            Col::Phases => "Fases asignadas",
            Col::Load => "Carga, W",
            Col::PowerFactor => "Factor de potencia",
            Col::Current => "Corriente nominal, A",
            Col::Design => "Corriente de diseño, A",
            Col::Length => "Longitud, m",
            Col::Size => "Calibre (candado: fijado por ti)",
            Col::Ground => "Puesta a tierra",
            Col::Conduit => "Tubería",
            Col::Breaker => "Interruptor",
            Col::Ampacity => "Ampacidad permitida, A",
            Col::Drop => "Caída del tramo, %",
            Col::Cumulative => "Caída acumulada, %",
            Col::Status => "Estado",
        }
    }

    fn numeric(self) -> bool {
        matches!(
            self,
            Col::Load
                | Col::PowerFactor
                | Col::Current
                | Col::Design
                | Col::Length
                | Col::Ampacity
                | Col::Drop
                | Col::Cumulative
        )
    }

    fn width(self, set: ColumnSet) -> f32 {
        match self {
            // En «Esencial» el tipo es su símbolo, como en la tabla guiada.
            Col::Kind if set == ColumnSet::Essential => 30.0,
            Col::Id => 44.0,
            Col::Name => 150.0,
            Col::Status => 84.0,
            Col::Kind => 74.0,
            Col::Connection | Col::Conduit | Col::Breaker => 58.0,
            Col::Size => 56.0,
            Col::Load | Col::Ampacity => 52.0,
            Col::Phases | Col::PowerFactor | Col::Ground => 36.0,
            _ => 46.0,
        }
    }
}

/// Columnas visibles. Se guarda entre sesiones.
#[derive(Debug, Clone, Serialize)]
pub struct Columns(pub Vec<Col>);

/// Columnas como se guardaron, incluidas las que ya no existen: así unas
/// preferencias de una versión anterior se siguen leyendo y solo se descarta
/// la columna retirada, no todo lo guardado.
#[derive(Deserialize)]
enum SavedCol {
    Id,
    Name,
    Kind,
    Connection,
    Phases,
    Load,
    PowerFactor,
    Current,
    Design,
    Length,
    Size,
    Ground,
    Conduit,
    Breaker,
    Ampacity,
    Drop,
    Cumulative,
    /// «Icc kA», retirada con el cortocircuito (decisión 0018).
    Fault,
    Status,
}

impl SavedCol {
    fn current(self) -> Option<Col> {
        Some(match self {
            SavedCol::Id => Col::Id,
            SavedCol::Name => Col::Name,
            SavedCol::Kind => Col::Kind,
            SavedCol::Connection => Col::Connection,
            SavedCol::Phases => Col::Phases,
            SavedCol::Load => Col::Load,
            SavedCol::PowerFactor => Col::PowerFactor,
            SavedCol::Current => Col::Current,
            SavedCol::Design => Col::Design,
            SavedCol::Length => Col::Length,
            SavedCol::Size => Col::Size,
            SavedCol::Ground => Col::Ground,
            SavedCol::Conduit => Col::Conduit,
            SavedCol::Breaker => Col::Breaker,
            SavedCol::Ampacity => Col::Ampacity,
            SavedCol::Drop => Col::Drop,
            SavedCol::Cumulative => Col::Cumulative,
            SavedCol::Fault => return None,
            SavedCol::Status => Col::Status,
        })
    }
}

impl<'de> Deserialize<'de> for Columns {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let saved = Vec::<SavedCol>::deserialize(deserializer)?;
        Ok(Columns(
            saved.into_iter().filter_map(SavedCol::current).collect(),
        ))
    }
}

impl Default for Columns {
    fn default() -> Self {
        Columns(
            Col::ALL
                .into_iter()
                .filter(|c| {
                    !matches!(
                        c,
                        Col::Kind
                            | Col::Connection
                            | Col::PowerFactor
                            | Col::Design
                            | Col::Ampacity
                    )
                })
                .collect(),
        )
    }
}

/// Qué renglones se ven: todos, o solo los que no cumplen o están sin
/// evaluar (lo que falta resolver).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Pending,
}

impl StatusFilter {
    fn keeps(self, status: Status) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::Pending => status != Status::Complies,
        }
    }
}

/// Juego de columnas del cuadro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColumnSet {
    /// Lo que se captura y lo que resuelve IEEStudio, como la tabla de la
    /// vista guiada: para capturar y revisar de un vistazo.
    #[default]
    Essential,
    /// Las columnas que elija el usuario (≡ › Ver › Personalizar columnas).
    Complete,
}

/// Columnas del juego «Esencial»: de `Kind` a `Length` es lo que se captura;
/// de `Size` en adelante, lo que resuelve IEEStudio.
pub const ESSENTIAL: [Col; 10] = [
    Col::Id,
    Col::Name,
    Col::Kind,
    Col::Load,
    Col::Phases,
    Col::Length,
    Col::Size,
    Col::Breaker,
    Col::Cumulative,
    Col::Status,
];

/// Las columnas visibles con el juego elegido.
pub fn visible(set: ColumnSet, custom: &Columns) -> Vec<Col> {
    match set {
        ColumnSet::Essential => ESSENTIAL.to_vec(),
        ColumnSet::Complete => custom.0.clone(),
    }
}

struct Row<'a> {
    selection: Selection,
    /// Tipo de carga de un circuito; `None` en el alimentador y el secundario.
    load: Option<LoadKind>,
    id: String,
    name: String,
    kind: &'static str,
    connection: &'static str,
    phases: String,
    power_factor: Option<f64>,
    forced: bool,
    item: &'a RunResult,
}

fn cell(col: Col, row: &Row) -> String {
    let run = row.item.run.as_ref();
    let dash = || "—".to_string();
    match col {
        Col::Id => row.id.clone(),
        Col::Name => row.name.clone(),
        Col::Kind => row.kind.to_string(),
        Col::Connection => row.connection.to_string(),
        Col::Phases => row.phases.clone(),
        Col::Load => row.item.load_w.map_or_else(dash, |w| format!("{w:.0}")),
        Col::PowerFactor => row.power_factor.map_or_else(dash, |f| format!("{f:.2}")),
        Col::Current => run.map_or_else(dash, |r| format!("{:.2}", r.nominal_a)),
        Col::Design => run.map_or_else(dash, |r| format!("{:.2}", r.design_a)),
        Col::Length => run.map_or_else(dash, |r| format!("{:.1}", r.length_m)),
        Col::Size => run.map_or_else(dash, |r| {
            if r.conductors_per_phase > 1 {
                format!("{}×{}", r.conductors_per_phase, r.size.0)
            } else {
                r.size.0.clone()
            }
        }),
        Col::Ground => run.map_or_else(dash, |r| r.grounding_size.0.clone()),
        Col::Conduit => run
            .and_then(|r| r.conduit.as_ref())
            .map_or_else(dash, |c| c.short_label()),
        Col::Breaker => run.map_or_else(dash, |r| format!("{}P-{:.0}", r.poles, r.breaker_a)),
        Col::Ampacity => run.map_or_else(dash, |r| format!("{:.1}", r.allowable_ampacity_a)),
        Col::Drop => run.map_or_else(dash, |r| format!("{:.2}", r.drop_percent)),
        Col::Cumulative => run.map_or_else(dash, |r| format!("{:.2}", r.cumulative_drop_percent)),
        Col::Status => String::new(),
    }
}

/// Estado de una columna concreta, para colorear solo la celda que falla.
fn cell_status(col: Col, item: &RunResult) -> Option<Status> {
    let name = match col {
        Col::Drop => "Caída de tensión",
        Col::Cumulative => "Caída acumulada",
        Col::Breaker => "Protección",
        Col::Size | Col::Ampacity => "Ampacidad",
        Col::Conduit => "Tubería",
        _ => return None,
    };
    item.checks
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.status)
        .filter(|s| *s != Status::Complies)
}

pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) {
    let p = shared.palette;
    let Some(panel_index) = doc.panel_index() else {
        crate::widgets::empty_state(
            ui,
            &p,
            icon::TABLE,
            "Sin tableros",
            "Agrega un tablero desde el panel Proyecto.",
        );
        return;
    };
    let Some(result) = doc.panel_result(panel_index).cloned() else {
        // Sin cálculo no hay tabla, pero el motivo debe quedar a la vista y
        // los datos siguen editables en Propiedades.
        let reason = doc.calc_error().unwrap_or_default();
        crate::widgets::empty_state(
            ui,
            &p,
            icon::WARNING_CIRCLE,
            "No se pudo calcular el proyecto",
            &format!("{reason}\nCorrige el dato en Propiedades y el cálculo se repite solo."),
        );
        return;
    };
    let panel = &doc.project.panels[panel_index];

    // ---------------------------------------------------------- encabezado
    let statuses: Vec<Status> = std::iter::once(result.feeder.status)
        .chain(result.secondary.iter().map(|s| s.status))
        .chain(result.circuits.iter().map(|c| c.status))
        .collect();
    let count = |s: Status| statuses.iter().filter(|&&x| x == s).count();
    let system = doc.bus_system(panel_index);
    let feeds_from = panel
        .parent
        .as_ref()
        .map_or_else(|| doc.project.source.name.clone(), |id| id.clone());
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 12,
            right: 12,
            top: 10,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(strong(format!("{} · {}", panel.id, panel.name)).size(17.0));
                ui.add_space(6.0);
                // Lo que quepa junto a los conteos de la derecha; nunca
                // encimado con ellos.
                let room = ui.available_width() - 450.0;
                if room > 60.0 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(room, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.add(
                                egui::Label::new(
                                    muted(
                                        &p,
                                        format!(
                                            "{} · se alimenta de {feeds_from}",
                                            label_of(&crate::widgets::SYSTEMS, system)
                                        ),
                                    )
                                    .size(11.5),
                                )
                                .truncate(),
                            );
                        },
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if doc.project.panels.len() > 1 {
                        let mut chosen = panel_index;
                        egui::ComboBox::from_id_salt("tablero-activo")
                            .selected_text(format!("{} Cambiar tablero", icon::SWAP))
                            .show_ui(ui, |ui| {
                                for (i, other) in doc.project.panels.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut chosen,
                                        i,
                                        format!("{} · {}", other.id, other.name),
                                    );
                                }
                            });
                        if chosen != panel_index {
                            doc.selection = Selection::Feeder(chosen);
                        }
                        ui.add_space(8.0);
                    }
                    // Conteo por estado: lo primero que se busca al revisar.
                    let tally = [
                        (
                            Status::NotEvaluated,
                            count(Status::NotEvaluated),
                            "sin evaluar",
                        ),
                        (Status::Fails, count(Status::Fails), "no cumplen"),
                        (Status::Complies, count(Status::Complies), "cumplen"),
                    ];
                    for (status, n, label) in tally {
                        if n == 0 && status != Status::Complies {
                            continue;
                        }
                        ui.label(
                            RichText::new(format!(
                                "{} {n} {label}",
                                crate::widgets::status_icon(status)
                            ))
                            .color(status_color(&p, status))
                            .size(12.0),
                        );
                        ui.add_space(4.0);
                    }
                });
            });
            ui.add_space(8.0);
            summary_strip(ui, &p, &result, worst(statuses.iter().copied()));
        });
    ui.separator();

    // ---------------------------------------------------------- herramientas
    let mut action = None;
    let pending = statuses.iter().filter(|s| **s != Status::Complies).count();
    ui.horizontal_wrapped(|ui| {
        ui.add_space(8.0);
        if primary_button(ui, &p, &format!("{}  Agregar circuito", icon::PLUS))
            .on_hover_text("¿Qué vas a conectar? Elige el tipo de carga o un subtablero")
            .clicked()
        {
            action = Some(Action::Add);
        }
        ui.add_space(6.0);
        segmented(
            ui,
            &p,
            &mut shared.column_set,
            &[
                (
                    ColumnSet::Essential,
                    "Esencial",
                    "Lo que capturas y lo que resuelve IEEStudio",
                ),
                (
                    ColumnSet::Complete,
                    "Completo",
                    "Todas las columnas que elegiste (≡ › Ver › Personalizar columnas)",
                ),
            ],
        );
        let pending_label = format!("Pendientes {pending}");
        segmented(
            ui,
            &p,
            &mut shared.status_filter,
            &[
                (StatusFilter::All, "Todos", ""),
                (
                    StatusFilter::Pending,
                    &pending_label,
                    "Solo lo que no cumple o quedó sin evaluar",
                ),
            ],
        );
        ui.add(
            egui::TextEdit::singleline(&mut shared.filter)
                .hint_text(format!(
                    "{} Filtrar por ID o descripción",
                    icon::MAGNIFYING_GLASS
                ))
                .desired_width(200.0),
        );
        ui.menu_button(format!("{} Tablero {}", icon::SQUARES_FOUR, icon::CARET_DOWN), |ui| {
            if ui
                .button(format!("{}  Balancear fases", icon::SCALES))
                .on_hover_text(
                    "Reparte las fases de los circuitos de una y dos fases de este tablero para que quede lo más balanceado posible. No reparte si falta la potencia de algún circuito. Se puede deshacer con Ctrl+Z.",
                )
                .clicked()
            {
                ui.close();
                action = Some(Action::Balance);
            }
            if ui
                .button(format!("{}  Agregar subtablero", icon::PLUS))
                .on_hover_text(format!("Un tablero que se alimenta de {}", panel.id))
                .clicked()
            {
                ui.close();
                action = Some(Action::Subpanel);
            }
            ui.separator();
            if ui
                .button(RichText::new(format!("{}  Eliminar tablero", icon::TRASH)).color(p.bad))
                .on_hover_text("Sus subtableros pasan al tablero de donde se alimentaba. Se puede deshacer con Ctrl+Z.")
                .clicked()
            {
                ui.close();
                action = Some(Action::RemovePanel);
            }
        });
    });
    ui.add_space(6.0);

    // ---------------------------------------------------------- filas
    let needle = shared.filter.trim().to_lowercase();
    let mut rows: Vec<Row> = Vec::with_capacity(result.circuits.len() + 1);
    rows.push(Row {
        selection: Selection::Feeder(panel_index),
        load: None,
        id: "ALIM".to_string(),
        name: format!("Alimentador de {}", panel.id),
        kind: "Alimentador",
        connection: label_of(&KINDS, panel.kind),
        phases: "—".to_string(),
        power_factor: Some(result.power_factor),
        forced: panel.feeder.overrides.size.is_some(),
        item: &result.feeder,
    });
    if let (Some(t), Some(item)) = (&panel.transformer, &result.secondary) {
        rows.push(Row {
            selection: Selection::Secondary(panel_index),
            load: None,
            id: "SEC".to_string(),
            name: match t.kva {
                Some(kva) => format!("Secundario, transformador de {kva} kVA"),
                None => "Secundario del transformador".to_string(),
            },
            kind: "Transformador",
            // El secundario tiene la conexión de su topología (pendiente 17).
            connection: label_of(
                &KINDS,
                ieestudio_project::connection::secondary_kind(t, panel.kind).unwrap_or(panel.kind),
            ),
            phases: "—".to_string(),
            power_factor: Some(result.power_factor),
            forced: t.secondary_overrides.size.is_some(),
            item,
        });
    }
    for (i, (circuit, item)) in panel.circuits.iter().zip(&result.circuits).enumerate() {
        rows.push(Row {
            selection: Selection::Circuit(panel_index, i),
            load: Some(circuit.load.kind),
            id: circuit.id.clone(),
            name: circuit.name.clone(),
            kind: circuit.load.kind.label(),
            connection: label_of(&KINDS, circuit.kind),
            phases: circuit
                .phases
                .iter()
                .map(|ph| format!("{ph:?}"))
                .collect::<Vec<_>>()
                .join(""),
            power_factor: Some(circuit.load.power_factor),
            forced: circuit.overrides.size.is_some(),
            item,
        });
    }
    rows.retain(|r| {
        let text_ok = needle.is_empty()
            || r.id.to_lowercase().contains(&needle)
            || r.name.to_lowercase().contains(&needle);
        text_ok && shared.status_filter.keeps(r.item.status)
    });

    let set = shared.column_set;
    let cols = visible(set, &shared.columns);
    let mut clicked = None;
    // La descripción toma el espacio que sobra; si no cabe todo, la tabla se
    // desplaza en horizontal en lugar de aplastar columnas.
    const GAP: f32 = 10.0;
    let fixed: f32 = cols
        .iter()
        .filter(|c| **c != Col::Name)
        .map(|c| c.width(set) + GAP)
        .sum();
    // Margen a la izquierda de la primera columna, para que el texto no
    // quede pegado al borde ni a la barra de selección.
    const INDENT: f32 = 8.0;
    let name_width = (ui.available_width() - fixed - GAP - INDENT - 30.0).max(130.0);
    egui::ScrollArea::horizontal()
        .id_salt("cuadro-h")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = GAP;
            if set == ColumnSet::Essential {
                group_header(ui, &p, &cols, name_width, GAP, INDENT);
            }
            let mut table = TableBuilder::new(ui)
                .id_salt(("cuadro", panel_index))
                .striped(false)
                .sense(Sense::click())
                .cell_layout(Layout::left_to_right(Align::Center))
                .resizable(false)
                .auto_shrink([false, false])
                .min_scrolled_height(0.0);
            for (k, col) in cols.iter().enumerate() {
                let indent = if k == 0 { INDENT } else { 0.0 };
                table = table.column(if *col == Col::Name {
                    Column::exact(name_width + indent).clip(true)
                } else {
                    Column::exact(col.width(set) + indent)
                });
            }
            table
                .header(26.0, |mut header| {
                    for (k, col) in cols.iter().enumerate() {
                        header.col(|ui| {
                            if k == 0 {
                                ui.add_space(INDENT);
                            }
                            let text = RichText::new(col.header()).size(11.0).color(p.ink3);
                            if col.numeric() {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(text);
                                });
                            } else {
                                ui.label(text);
                            }
                        });
                    }
                })
                .body(|body| {
                    body.rows(24.0, rows.len(), |mut table_row| {
                        let row = &rows[table_row.index()];
                        let selected = doc.selection == row.selection;
                        table_row.set_selected(selected);
                        let feeder = matches!(row.selection, Selection::Feeder(_) | Selection::Secondary(_));
                        for (k, col) in cols.iter().enumerate() {
                            table_row.col(|ui| {
                                if k == 0 && selected {
                                    let r = ui.max_rect();
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(r.left_top(), Vec2::new(3.0, r.height())),
                                        0.0,
                                        p.accent,
                                    );
                                }
                                if k == 0 {
                                    ui.add_space(INDENT);
                                }
                                if *col == Col::Status {
                                    crate::widgets::status_chip(ui, &p, row.item.status);
                                    return;
                                }
                                if *col == Col::Kind && set == ColumnSet::Essential {
                                    match row.load {
                                        Some(kind) => {
                                            symbols::show(ui, Symbol::Load(kind), 18.0, p.ink2)
                                                .on_hover_text(kind.label());
                                        }
                                        None => {
                                            ui.label(RichText::new(icon::SQUARES_FOUR).color(p.ink3))
                                                .on_hover_text(row.kind);
                                        }
                                    }
                                    return;
                                }
                                let text = cell(*col, row);
                                let mut rich = if col.numeric() || matches!(col, Col::Size | Col::Ground | Col::Breaker | Col::Conduit) {
                                    RichText::new(text).monospace()
                                } else {
                                    RichText::new(text)
                                };
                                if feeder && matches!(col, Col::Id | Col::Name) {
                                    rich = rich.family(crate::theme::semibold());
                                }
                                if *col == Col::Id {
                                    rich = rich.color(p.ink2);
                                }
                                if let Some(s) = cell_status(*col, row.item) {
                                    rich = rich.color(status_color(&p, s));
                                }
                                if col.numeric() {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(rich);
                                    });
                                } else if *col == Col::Size && row.forced {
                                    ui.label(rich);
                                    ui.label(RichText::new(icon::LOCK_SIMPLE).size(11.0).color(p.lav_text))
                                        .on_hover_text("Calibre fijado por ti: IEEStudio lo verifica, no lo cambia.");
                                } else {
                                    ui.label(rich);
                                }
                            });
                        }
                        if table_row.response().clicked() {
                            clicked = Some(row.selection);
                        }
                    });
                });
        });
    if rows.is_empty() {
        ui.label(muted(&p, "Ningún circuito coincide con el filtro."));
    }
    if let Some(sel) = clicked {
        doc.selection = sel;
    }

    // Teclado: flechas para recorrer las filas con el cursor sobre la tabla.
    let typing = ui.ctx().memory(|m| m.focused().is_some());
    if !typing && ui.ui_contains_pointer() {
        let (up, down) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
            )
        });
        if up {
            doc.step_selection(-1);
        }
        if down {
            doc.step_selection(1);
        }
    }

    match action {
        Some(Action::Add) => shared.choose_load = Some(panel_index),
        Some(Action::Balance) => {
            shared.done = Some(doc.balance_phases(panel_index));
        }
        Some(Action::Subpanel) => {
            let k = doc.add_subpanel(panel_index);
            let panels = &doc.project.panels;
            shared.done = Some(format!(
                "Subtablero {} agregado desde {}: captura su alimentador en la ficha.",
                panels[k].id, panels[panel_index].id
            ));
        }
        Some(Action::RemovePanel) => {
            doc.selection = Selection::Feeder(panel_index);
            doc.remove_selected();
        }
        None => {}
    }
}

enum Action {
    Add,
    Balance,
    Subpanel,
    RemovePanel,
}

/// Encabezado de grupos del juego «Esencial»: lo que se captura y lo que
/// resuelve IEEStudio, alineado con las columnas de la tabla.
fn group_header(
    ui: &mut egui::Ui,
    p: &Palette,
    cols: &[Col],
    name_width: f32,
    gap: f32,
    indent: f32,
) {
    let width = |c: &Col| {
        if *c == Col::Name {
            name_width
        } else {
            c.width(ColumnSet::Essential)
        }
    };
    let split = cols
        .iter()
        .position(|c| *c == Col::Size)
        .unwrap_or(cols.len());
    let capture: f32 = indent + cols[..split].iter().map(|c| width(c) + gap).sum::<f32>();
    let resolve: f32 = cols[split..].iter().map(|c| width(c) + gap).sum::<f32>();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let (r1, _) = ui.allocate_exact_size(Vec2::new(capture, 20.0), Sense::hover());
        ui.painter().text(
            r1.left_center() + Vec2::new(indent, 0.0),
            egui::Align2::LEFT_CENTER,
            "LO QUE CAPTURAS",
            egui::FontId::new(10.5, crate::theme::semibold()),
            p.ink2,
        );
        let (r2, _) = ui.allocate_exact_size(Vec2::new(resolve, 20.0), Sense::hover());
        ui.painter().line_segment(
            [r2.left_top(), r2.left_bottom()],
            egui::Stroke::new(1.0, p.line),
        );
        ui.painter().text(
            r2.left_center() + Vec2::new(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "LO QUE RESUELVE IEESTUDIO",
            egui::FontId::new(10.5, crate::theme::semibold()),
            p.accent_text,
        );
    });
}

/// Ventana «Personalizar columnas» (≡ › Ver): elige las columnas del juego
/// «Completo» y lo deja activo.
pub fn columns_window(ctx: &egui::Context, shared: &mut Shared, open: &mut bool) {
    egui::Window::new("Personalizar columnas")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .default_pos([420.0, 120.0])
        .show(ctx, |ui| {
            ui.label(muted(
                &shared.palette,
                "Columnas del juego «Completo» del cuadro de cargas.",
            ));
            ui.add_space(4.0);
            for col in Col::ALL {
                if matches!(col, Col::Id | Col::Status) {
                    continue;
                }
                let mut on = shared.columns.0.contains(&col);
                if ui.checkbox(&mut on, col.description()).changed() {
                    if on {
                        shared.columns.0.push(col);
                        shared
                            .columns
                            .0
                            .sort_by_key(|c| Col::ALL.iter().position(|x| x == c));
                    } else {
                        shared.columns.0.retain(|c| *c != col);
                    }
                    shared.column_set = ColumnSet::Complete;
                }
            }
            ui.separator();
            if ui.button("Restablecer columnas").clicked() {
                shared.columns = Columns::default();
                shared.column_set = ColumnSet::Complete;
            }
        });
}

/// Franja de resumen del tablero: una línea de cifras grandes separadas por
/// líneas finas, sin tarjetas. Cada cifra explica de dónde sale al pasar el
/// cursor.
fn summary_strip(
    ui: &mut egui::Ui,
    p: &Palette,
    r: &ieestudio_project::PanelResult,
    status: Status,
) {
    // Con poco ancho (dos vistas a la vez, 1024 px) las cifras pasan al
    // renglón de abajo en lugar de apretarse contra el borde.
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.spacing_mut().item_spacing.y = 8.0;
        // Línea entre cifras; si la cifra que sigue (de ancho `next`) no cabe,
        // pasa al renglón de abajo.
        let divider = |ui: &mut egui::Ui, next: f32| {
            if ui.available_size_before_wrap().x < 33.0 + next {
                ui.end_row();
                return;
            }
            ui.add_space(16.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 34.0), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, p.line);
            ui.add_space(16.0);
        };
        let figure = |ui: &mut egui::Ui, label: &str, value: String, unit: &str, color| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(RichText::new(label).size(10.5).color(p.ink3));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    ui.label(RichText::new(value).monospace().size(19.0).color(color));
                    ui.label(RichText::new(unit).size(11.0).color(p.ink2));
                });
            })
            .response
        };
        // Una suma parcial no es la carga del tablero (pendiente 19).
        let known = |value: f64| match r.load_unknown {
            None => format!("{value:.0}"),
            Some(_) => "—".to_string(),
        };
        figure(ui, "CONECTADA", known(r.connected_w), "W", p.ink).on_hover_text(
            match &r.load_unknown {
                None => "Suma de las cargas de este tablero y sus subtableros".to_string(),
                Some(why) => format!("Sin conocer: {why}"),
            },
        );
        divider(ui, 90.0);
        let detail = r
            .demand
            .iter()
            .map(|l| {
                format!(
                    "{}: {:.0} → {:.0} VA",
                    l.label, l.connected_va, l.demanded_va
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        figure(ui, "CALCULADA", known(r.demanded_va), "VA", p.ink).on_hover_text(format!(
            "Carga del alimentador según el artículo 220 (factor resultante {:.2}):\n{detail}",
            r.demand_factor
        ));
        divider(ui, 70.0);
        let factor = match r.load_unknown {
            None => format!("{:.2}", r.demand_factor),
            Some(_) => "—".to_string(),
        };
        figure(ui, "F.D.", factor, "", p.ink)
            .on_hover_text("Factor de demanda resultante: carga calculada entre carga conectada");
        divider(ui, 90.0);
        if let Some(run) = &r.feeder.run {
            figure(ui, "CORRIENTE", format!("{:.2}", run.nominal_a), "A", p.ink)
                .on_hover_text("Corriente nominal del alimentador");
            divider(ui, 70.0);
        }
        figure(ui, "FP", format!("{:.2}", r.power_factor), "", p.ink)
            .on_hover_text("Factor de potencia de la carga conectada");
        divider(ui, 110.0);
        // La caída mayor del tablero: la que manda al revisar.
        let worst_drop = r
            .circuits
            .iter()
            .filter_map(|c| c.run.as_ref().map(|run| (c, run.cumulative_drop_percent)))
            .max_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((c, drop)) = worst_drop {
            let fails = c
                .checks
                .iter()
                .any(|k| k.name == "Caída acumulada" && k.status == Status::Fails);
            figure(
                ui,
                "CAÍDA MÁXIMA",
                format!("{drop:.2}"),
                "%",
                if fails { p.bad } else { p.ink },
            )
            .on_hover_text(format!(
                "Caída acumulada mayor del tablero: {} · {}",
                c.id, c.name
            ));
            divider(ui, 210.0);
        }

        // Balance de fases: tres barras con su carga, no una gráfica de dona.
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            let title = match r.imbalance_percent {
                Some(i) => format!("FASES · DESBALANCE {i:.1} %"),
                None => "FASES · SIN ASIGNAR".to_string(),
            };
            let warn = r.imbalance_percent.is_some_and(|i| i > 5.0);
            ui.label(
                RichText::new(title)
                    .size(10.5)
                    .color(if warn { p.warn } else { p.ink3 }),
            );
            let max = r.phase_va.iter().cloned().fold(1.0, f64::max);
            for (idx, (name, va)) in ["A", "B", "C"].iter().zip(r.phase_va).enumerate() {
                let phase_color = p.phases[idx];
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    ui.label(RichText::new(*name).size(10.5).color(phase_color));
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(90.0, 5.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, p.selected);
                    let mut fill = rect;
                    fill.set_width(rect.width() * (va / max) as f32);
                    ui.painter().rect_filled(fill, 2.0, phase_color);
                    ui.label(
                        RichText::new(format!("{:.1} kVA", va / 1000.0))
                            .monospace()
                            .size(10.5)
                            .color(p.ink2),
                    );
                    resp.on_hover_text(format!("Fase {name}: {va:.0} VA"));
                });
            }
        });
        divider(ui, 110.0);
        ui.vertical(|ui| {
            // Con ancho mínimo, si no cabe pasa al renglón de abajo en vez de
            // escribirse letra por letra.
            ui.set_min_width(110.0);
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.label(RichText::new("ESTE TABLERO").size(10.5).color(p.ink3));
            ui.label(status_text(p, status).size(15.0));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unas preferencias con la columna de cortocircuito se siguen leyendo;
    /// solo esa columna se descarta.
    #[test]
    fn saved_columns_drop_the_retired_fault_column() {
        let saved = r#"["Id","Name","Fault","Status"]"#;
        let columns: Columns = serde_json::from_str(saved).unwrap();
        assert_eq!(columns.0, vec![Col::Id, Col::Name, Col::Status]);
        let again: Columns =
            serde_json::from_str(&serde_json::to_string(&columns).unwrap()).unwrap();
        assert_eq!(again.0, columns.0);
    }

    #[test]
    fn the_pending_filter_keeps_what_is_left_to_solve() {
        assert!(StatusFilter::All.keeps(Status::Complies));
        assert!(!StatusFilter::Pending.keeps(Status::Complies));
        assert!(StatusFilter::Pending.keeps(Status::Fails));
        assert!(StatusFilter::Pending.keeps(Status::NotEvaluated));
    }

    #[test]
    fn the_essential_set_shows_what_is_captured_then_what_is_solved() {
        let cols = visible(ColumnSet::Essential, &Columns::default());
        assert_eq!(cols, ESSENTIAL.to_vec());
        let split = cols.iter().position(|c| *c == Col::Size).unwrap();
        // Lo que se captura: tipo, carga, fases y longitud.
        for c in [Col::Kind, Col::Load, Col::Phases, Col::Length] {
            assert!(cols[..split].contains(&c));
        }
        // Lo que resuelve IEEStudio, y el estado al final.
        for c in [Col::Size, Col::Breaker, Col::Cumulative] {
            assert!(cols[split..].contains(&c));
        }
        assert_eq!(cols.last(), Some(&Col::Status));
        // «Completo» son las columnas elegidas.
        let custom = Columns(vec![Col::Id, Col::Ampacity, Col::Status]);
        assert_eq!(visible(ColumnSet::Complete, &custom), custom.0);
    }

    /// El mismo proyecto da los mismos resultados en el cuadro de la
    /// interfaz, en la memoria y en la línea de comandos: los tres salen del
    /// mismo cálculo y se escriben con el mismo formato.
    #[test]
    fn the_schedule_the_report_and_the_command_line_agree() {
        let mut files = Vec::new();
        let mut dirs = vec![std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures"
        ))];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "iees") {
                    files.push(path);
                }
            }
        }
        assert!(files.len() >= 10, "se esperaban los casos de fixtures/");
        let mut checked = 0;
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            let Ok(project) = ieestudio_project::Project::from_json(&text) else {
                continue;
            };
            // La línea de comandos: `calcular --json` lee el archivo y calcula.
            let cli = ieestudio_project::calculate(&project);
            let doc = Doc::new(project.clone(), None);
            let (Ok(cli), Ok(app)) = (cli, &doc.result) else {
                assert_eq!(
                    doc.result.is_err(),
                    ieestudio_project::calculate(&project).is_err()
                );
                continue;
            };
            assert_eq!(
                serde_json::to_value(&cli).unwrap(),
                serde_json::to_value(app).unwrap(),
                "{}",
                path.display()
            );
            let source = ieestudio_reports::project_source(&doc.project, app);
            for (i, panel) in doc.project.panels.iter().enumerate() {
                let result = &app.panels[i];
                for (circuit, item) in panel.circuits.iter().zip(&result.circuits) {
                    let row = Row {
                        selection: Selection::Circuit(i, 0),
                        load: Some(circuit.load.kind),
                        id: circuit.id.clone(),
                        name: circuit.name.clone(),
                        kind: circuit.load.kind.label(),
                        connection: "",
                        phases: String::new(),
                        power_factor: None,
                        forced: false,
                        item,
                    };
                    let Some(run) = &item.run else {
                        assert_eq!(cell(Col::Size, &row), "—");
                        continue;
                    };
                    assert_eq!(
                        cell(Col::Breaker, &row),
                        format!("{}P-{:.0}", run.poles, run.breaker_a)
                    );
                    assert_eq!(
                        cell(Col::Cumulative, &row),
                        format!("{:.2}", run.cumulative_drop_percent)
                    );
                    // La memoria escribe las mismas celdas que el cuadro.
                    for col in [Col::Size, Col::Breaker, Col::Drop, Col::Cumulative] {
                        let text = format!("\"{}\"", cell(col, &row));
                        assert!(
                            source.contains(&text),
                            "{}: {} {:?} no está en la memoria",
                            path.display(),
                            circuit.id,
                            col
                        );
                    }
                    checked += 1;
                }
            }
        }
        assert!(checked > 20, "se revisaron {checked} circuitos");
    }
}
