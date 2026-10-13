//! Cuadro de cargas: el área de trabajo principal.
//!
//! Escala a cientos de circuitos: solo se dibujan las filas visibles, hay
//! filtro de texto y de estado, las columnas se eligen y la tabla se
//! desplaza en horizontal cuando no cabe.

use eframe::egui::{self, Align, Layout, RichText, Sense, Vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use serde::{Deserialize, Serialize};
use voltforge_project::{RunResult, Status};

use super::Shared;
use crate::state::{Doc, Selection};
use crate::theme::Palette;
use crate::widgets::{label_of, muted, status_color, status_text, strong, worst, KINDS};

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
    Fault,
    Status,
}

impl Col {
    pub const ALL: [Col; 19] = [
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
        Col::Fault,
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
            Col::Fault => "Icc kA",
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
            Col::Fault => "Cortocircuito en las terminales del interruptor, kA",
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
                | Col::Fault
        )
    }

    fn width(self) -> f32 {
        match self {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Columns(pub Vec<Col>);

impl Default for Columns {
    fn default() -> Self {
        Columns(
            Col::ALL
                .into_iter()
                .filter(|c| {
                    !matches!(
                        c,
                        Col::Kind
                            | Col::Fault
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Fails,
    Pending,
}

struct Row<'a> {
    selection: Selection,
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
        Col::Load => format!("{:.0}", row.item.load_w),
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
            .map_or_else(dash, |c| format!("{} mm", c.metric_designator)),
        Col::Breaker => run.map_or_else(dash, |r| format!("{}P-{:.0}", r.poles, r.breaker_a)),
        Col::Ampacity => run.map_or_else(dash, |r| format!("{:.1}", r.allowable_ampacity_a)),
        Col::Drop => run.map_or_else(dash, |r| format!("{:.2}", r.drop_percent)),
        Col::Cumulative => run.map_or_else(dash, |r| format!("{:.2}", r.cumulative_drop_percent)),
        Col::Fault => run
            .and_then(|r| r.fault_ka)
            .map_or_else(dash, |k| format!("{k:.2}")),
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
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(strong(format!("{} · {}", panel.id, panel.name)).size(15.0));
                if doc.project.panels.len() > 1 {
                    let mut chosen = panel_index;
                    egui::ComboBox::from_id_salt("tablero-activo")
                        .selected_text("Cambiar tablero")
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
                }
            });
            ui.add_space(4.0);
            summary_strip(ui, &p, &result);
        });
    ui.separator();

    // ---------------------------------------------------------- herramientas
    let statuses: Vec<Status> = std::iter::once(result.feeder.status)
        .chain(result.circuits.iter().map(|c| c.status))
        .collect();
    let count = |s: Status| statuses.iter().filter(|&&x| x == s).count();
    let mut action = None;
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        if ui
            .button(format!("{} Circuito", icon::PLUS))
            .on_hover_text("Agregar un circuito a este tablero")
            .clicked()
        {
            action = Some(Action::Add);
        }
        let on_circuit = matches!(doc.selection, Selection::Circuit(..));
        if ui
            .add_enabled(
                on_circuit,
                egui::Button::new(format!("{} Duplicar", icon::COPY)),
            )
            .on_hover_text("Duplicar el circuito seleccionado (Ctrl+D)")
            .clicked()
        {
            action = Some(Action::Duplicate);
        }
        if ui
            .add_enabled(
                on_circuit,
                egui::Button::new(format!("{} Eliminar", icon::TRASH)),
            )
            .on_hover_text("Eliminar el circuito seleccionado (Supr)")
            .clicked()
        {
            action = Some(Action::Remove);
        }
        ui.separator();
        ui.add(
            egui::TextEdit::singleline(&mut shared.filter)
                .hint_text(format!(
                    "{} Filtrar por ID o descripción",
                    icon::MAGNIFYING_GLASS
                ))
                .desired_width(220.0),
        );
        ui.selectable_value(&mut shared.status_filter, StatusFilter::All, "Todos");
        ui.selectable_value(
            &mut shared.status_filter,
            StatusFilter::Fails,
            RichText::new(format!(
                "{} No cumple {}",
                icon::X_CIRCLE,
                count(Status::Fails)
            ))
            .color(p.bad),
        );
        ui.selectable_value(
            &mut shared.status_filter,
            StatusFilter::Pending,
            RichText::new(format!(
                "{} Sin evaluar {}",
                icon::CIRCLE_DASHED,
                count(Status::NotEvaluated)
            ))
            .color(p.warn),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(6.0);
            ui.menu_button(format!("{} Columnas", icon::COLUMNS), |ui| {
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
                    }
                }
                ui.separator();
                if ui.button("Restablecer columnas").clicked() {
                    shared.columns = Columns::default();
                }
            });
        });
    });
    ui.add_space(2.0);

    // ---------------------------------------------------------- filas
    let needle = shared.filter.trim().to_lowercase();
    let mut rows: Vec<Row> = Vec::with_capacity(result.circuits.len() + 1);
    rows.push(Row {
        selection: Selection::Feeder(panel_index),
        id: "ALIM".to_string(),
        name: format!("Alimentador de {}", panel.id),
        kind: "Alimentador",
        connection: label_of(&KINDS, panel.kind),
        phases: "—".to_string(),
        power_factor: Some(result.power_factor),
        forced: panel.feeder.overrides.size.is_some(),
        item: &result.feeder,
    });
    for (i, (circuit, item)) in panel.circuits.iter().zip(&result.circuits).enumerate() {
        rows.push(Row {
            selection: Selection::Circuit(panel_index, i),
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
        let status_ok = match shared.status_filter {
            StatusFilter::All => true,
            StatusFilter::Fails => r.item.status == Status::Fails,
            StatusFilter::Pending => r.item.status == Status::NotEvaluated,
        };
        text_ok && status_ok
    });

    let cols = shared.columns.0.clone();
    let mut clicked = None;
    // La descripción toma el espacio que sobra; si no cabe todo, la tabla se
    // desplaza en horizontal en lugar de aplastar columnas.
    const GAP: f32 = 10.0;
    let fixed: f32 = cols
        .iter()
        .filter(|c| **c != Col::Name)
        .map(|c| c.width() + GAP)
        .sum();
    // Margen a la izquierda de la primera columna, para que el texto no
    // quede pegado al borde ni a la barra de selección.
    const INDENT: f32 = 8.0;
    let name_width = (ui.available_width() - fixed - GAP - INDENT - 30.0).max(130.0);
    egui::ScrollArea::horizontal()
        .id_salt("cuadro-h")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = GAP;
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
                    Column::exact(col.width() + indent)
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
                        let feeder = matches!(row.selection, Selection::Feeder(_));
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
                                    ui.label(status_text(&p, row.item.status));
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
                                    ui.label(RichText::new(icon::LOCK_SIMPLE).size(11.0).color(p.ink3))
                                        .on_hover_text("Calibre fijado por ti: VoltForge lo verifica, no lo cambia.");
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
        Some(Action::Add) => doc.add_circuit(panel_index),
        Some(Action::Duplicate) => doc.duplicate_circuit(),
        Some(Action::Remove) => doc.remove_selected(),
        None => {}
    }
}

enum Action {
    Add,
    Duplicate,
    Remove,
}

/// Franja de resumen del tablero: densa, en una línea, sin tarjetas.
fn summary_strip(ui: &mut egui::Ui, p: &Palette, r: &voltforge_project::PanelResult) {
    ui.horizontal(|ui| {
        let figure = |ui: &mut egui::Ui, label: &str, value: String| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(10.5).color(p.ink3));
                ui.label(RichText::new(value).monospace().size(14.0));
            });
            ui.add_space(14.0);
        };
        figure(ui, "CONECTADA", format!("{:.0} W", r.connected_w));
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
        ui.vertical(|ui| {
            ui.label(RichText::new("CALCULADA").size(10.5).color(p.ink3));
            ui.label(
                RichText::new(format!("{:.0} VA", r.demanded_va))
                    .monospace()
                    .size(14.0),
            );
        })
        .response
        .on_hover_text(format!(
            "Carga del alimentador según el artículo 220 (factor resultante {:.2}):\n{detail}",
            r.demand_factor
        ));
        ui.add_space(14.0);
        if let Some(run) = &r.feeder.run {
            figure(ui, "CORRIENTE ALIM.", format!("{:.2} A", run.nominal_a));
        }
        figure(ui, "FP", format!("{:.2}", r.power_factor));

        // Balance de fases: tres barras, no una gráfica de dona.
        ui.vertical(|ui| {
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
            ui.horizontal(|ui| {
                for (name, va) in ["A", "B", "C"].iter().zip(r.phase_va) {
                    ui.label(RichText::new(*name).size(11.0).color(p.ink2));
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(54.0, 6.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, p.raise);
                    let mut fill = rect;
                    fill.set_width(rect.width() * (va / max) as f32);
                    ui.painter().rect_filled(fill, 2.0, p.ink2);
                    resp.on_hover_text(format!("Fase {name}: {va:.0} VA"));
                    ui.add_space(6.0);
                }
            });
        });
        ui.add_space(14.0);

        let all: Vec<Status> = std::iter::once(r.feeder.status)
            .chain(r.circuits.iter().map(|c| c.status))
            .collect();
        ui.vertical(|ui| {
            ui.label(RichText::new("ESTE TABLERO").size(10.5).color(p.ink3));
            ui.label(status_text(p, worst(all.iter().copied())));
        });
    });
}
