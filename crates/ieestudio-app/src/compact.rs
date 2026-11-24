//! Marco de tableta y teléfono: barra inferior, listas de tableros y
//! circuitos, ficha a pantalla completa, hoja inferior del unifilar y cajones.
//!
//! Lo que se muestra sale del mismo `Doc` y del mismo cálculo que en
//! escritorio; aquí solo cambia cómo se acomoda.

use eframe::egui::{
    self, Align, Align2, Color32, FontFamily, FontId, Layout, RichText, Sense, Vec2,
};
use egui_phosphor::regular as icon;
use ieestudio_project::{RunResult, Status};

use crate::panels::review::problem_counts;
use crate::panels::{diagram, properties, Shared};
use crate::shell::CAPSULE;
use crate::state::{Doc, Selection};
use crate::theme::{self, semibold, Palette};
use crate::views::View;
use crate::widgets::{
    muted, primary_button, status_chip, status_color, status_icon, strong, thousands,
};

/// Alto de la barra inferior del teléfono.
pub const BOTTOM_BAR_H: f32 = 56.0;

/// Nombre corto de cada destino en la barra inferior.
pub fn short_title(view: View) -> &'static str {
    match view {
        View::Schedule => "Circuitos",
        other => other.title(),
    }
}

/// La barra inferior del teléfono: los cinco destinos, con una cápsula
/// detrás del ícono activo. Devuelve el destino tocado.
pub fn bottom_bar(
    ui: &mut egui::Ui,
    p: &Palette,
    current: Option<View>,
    doc: &Doc,
) -> Option<View> {
    let (fails, pending) = problem_counts(doc);
    let width = ui.available_width() / View::ALL.len() as f32;
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for view in View::ALL {
            let now = current == Some(view);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(width, BOTTOM_BAR_H), Sense::click());
            let painter = ui.painter();
            let glyph_at = egui::pos2(rect.center().x, rect.top() + theme::SP2 + theme::SP3);
            if now {
                painter.rect_filled(
                    egui::Rect::from_center_size(glyph_at, Vec2::new(56.0, theme::BAR_CONTROL_H)),
                    CAPSULE,
                    p.accent.gamma_multiply(if p.dark { 0.22 } else { 0.32 }),
                );
            }
            painter.text(
                glyph_at,
                Align2::CENTER_CENTER,
                view.icon(),
                FontId::new(theme::ICON_BAR, FontFamily::Proportional),
                if now { p.accent_text } else { p.ink2 },
            );
            painter.text(
                egui::pos2(rect.center().x, rect.bottom() - theme::SP2),
                Align2::CENTER_BOTTOM,
                short_title(view),
                FontId::new(
                    theme::FOOT,
                    if now {
                        semibold()
                    } else {
                        FontFamily::Proportional
                    },
                ),
                if now { p.ink } else { p.ink2 },
            );
            // Revisar avisa cuánto falta, como en la barra de escritorio.
            if view == View::Review && fails + pending > 0 {
                let color = if fails > 0 { p.bad } else { p.warn };
                let at = glyph_at + Vec2::new(theme::SP3, -theme::SP2);
                painter.circle_filled(at, theme::SP2, color);
                painter.text(
                    at,
                    Align2::CENTER_CENTER,
                    (fails + pending).min(99).to_string(),
                    FontId::new(theme::FOOT, semibold()),
                    Color32::WHITE,
                );
            }
            if response.clicked() && !now {
                chosen = Some(view);
            }
        }
    });
    chosen
}

// ------------------------------------------------------ tableros y circuitos

/// En qué nivel de «Tableros y circuitos» está el teléfono.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// La lista de tableros.
    Panels,
    /// Las tarjetas de circuito de un tablero.
    Circuits(usize),
    /// La ficha del elemento elegido, a pantalla completa.
    Ficha,
}

/// El nivel que corresponde a la selección.
pub fn level(selection: Selection, ficha_open: bool) -> Level {
    match selection {
        Selection::Project => Level::Panels,
        _ if ficha_open => Level::Ficha,
        Selection::Feeder(i) | Selection::Secondary(i) | Selection::Circuit(i, _) => {
            Level::Circuits(i)
        }
    }
}

/// A dónde lleva «atrás» desde la vista actual del teléfono y con qué
/// rótulo. `None` si no hay a dónde volver.
pub fn back(view: View, doc: &Doc, shared: &Shared) -> Option<String> {
    if view != View::Schedule {
        return None;
    }
    match level(doc.selection, shared.ficha_open) {
        Level::Panels => None,
        Level::Circuits(_) => Some("Tableros".to_string()),
        Level::Ficha => doc
            .panel_index()
            .and_then(|i| doc.project.panels.get(i))
            .map(|panel| panel.id.clone()),
    }
}

/// Vuelve un nivel atrás.
pub fn go_back(doc: &mut Doc, shared: &mut Shared) {
    match level(doc.selection, shared.ficha_open) {
        Level::Ficha => shared.ficha_open = false,
        Level::Circuits(_) => doc.selection = Selection::Project,
        Level::Panels => {}
    }
}

/// «Tableros y circuitos» en el teléfono: tableros, luego las tarjetas de
/// circuito, luego la ficha.
pub fn schedule(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) {
    let p = shared.palette;
    match level(doc.selection, shared.ficha_open) {
        Level::Ficha => {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(16, 12))
                .show(ui, |ui| properties::show(ui, doc, shared));
        }
        Level::Panels => page(ui, |ui| panel_list(ui, &p, doc)),
        Level::Circuits(i) => page(ui, |ui| circuit_list(ui, &p, doc, shared, i)),
    }
}

/// Una pantalla de lista: márgenes de 16 px y desplazamiento vertical.
fn page(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::ScrollArea::vertical()
        .id_salt("lista-telefono")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 16,
                    right: 16,
                    top: 12,
                    bottom: 24,
                })
                .show(ui, add);
        });
}

fn panel_list(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) {
    ui.label(strong("Tableros").size(theme::VIEW_TITLE));
    ui.label(muted(p, &doc.project.name));
    ui.add_space(theme::SP4);
    let rows = panel_rows(doc);
    if let Some(i) = list(ui, p, &rows) {
        doc.selection = Selection::Feeder(i);
    }
    if rows.is_empty() {
        ui.label(muted(p, "Sin tableros: agrégalos desde la vista Proyecto."));
    }
}

fn circuit_list(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc, shared: &mut Shared, i: usize) {
    let Some(panel) = doc.project.panels.get(i) else {
        doc.selection = Selection::Project;
        return;
    };
    ui.label(strong(format!("{} · {}", panel.id, panel.name)).size(theme::TITLE));
    if let Some(r) = doc.panel_result(i) {
        let current = r.feeder.run.as_ref().map_or_else(String::new, |run| {
            format!(" · {:.1} A en el alimentador", run.nominal_a)
        });
        ui.label(muted(
            p,
            match r.load_unknown {
                None => format!("{} VA calculados{current}", thousands(r.demanded_va)),
                Some(_) => "Carga sin conocer".to_string(),
            },
        ));
    }
    ui.add_space(theme::SP3);
    ui.horizontal_wrapped(|ui| {
        if primary_button(ui, p, &format!("{}  Agregar circuito", icon::PLUS)).clicked() {
            shared.choose_load = Some(i);
        }
        if ui
            .button(format!("{}  Ficha del tablero", icon::SLIDERS_HORIZONTAL))
            .clicked()
        {
            doc.selection = Selection::Feeder(i);
            shared.ficha_open = true;
        }
    });
    ui.add_space(theme::SP4);
    let Some(result) = doc.panel_result(i) else {
        ui.label(muted(p, doc.calc_error().unwrap_or_default()));
        return;
    };
    let rows: Vec<(String, String, Option<Status>)> = doc.project.panels[i]
        .circuits
        .iter()
        .enumerate()
        .map(|(c, circuit)| {
            let item = result.circuits.get(c);
            (
                format!("{} · {}", circuit.id, circuit.name),
                item.map_or_else(String::new, card_figures),
                item.map(|r| r.status),
            )
        })
        .collect();
    if let Some(c) = list(ui, p, &rows) {
        doc.selection = Selection::Circuit(i, c);
        shared.ficha_open = true;
    }
    if rows.is_empty() {
        ui.label(muted(p, "Este tablero no tiene circuitos todavía."));
    }
}

/// Un renglón por tablero: nombre, circuitos, carga y de dónde se alimenta,
/// con el peor estado de sus tramos.
pub fn panel_rows(doc: &Doc) -> Vec<(String, String, Option<Status>)> {
    doc.project
        .panels
        .iter()
        .enumerate()
        .map(|(i, panel)| {
            let result = doc.panel_result(i);
            let load = result.map_or_else(String::new, |r| match r.load_unknown {
                None => format!(" · {} VA", thousands(r.demanded_va)),
                Some(_) => " · carga sin conocer".to_string(),
            });
            let feeds = panel
                .parent
                .as_ref()
                .map_or_else(String::new, |p| format!(" · de {p}"));
            (
                format!("{} · {}", panel.id, panel.name),
                format!("{} circuitos{load}{feeds}", panel.circuits.len()),
                result.map(|r| worst(r.circuits.iter().chain(Some(&r.feeder)))),
            )
        })
        .collect()
}

/// Cifras de la tarjeta de un circuito: carga, cable, interruptor y caída.
pub fn card_figures(item: &RunResult) -> String {
    let load = item.load_w.map_or_else(
        || "sin carga".to_string(),
        |w| format!("{} W", thousands(w)),
    );
    match &item.run {
        Some(run) => format!(
            "{load} · {} AWG · {}P-{:.0} A · {:.2} %",
            run.size.0, run.poles, run.breaker_a, run.cumulative_drop_percent
        ),
        None => format!("{load} · sin tramo calculado"),
    }
}

/// El peor estado de un grupo de tramos: no cumple, luego sin evaluar.
fn worst<'a>(items: impl Iterator<Item = &'a RunResult>) -> Status {
    let mut out = Status::Complies;
    for item in items {
        match item.status {
            Status::Fails => return Status::Fails,
            Status::NotEvaluated => out = Status::NotEvaluated,
            _ => {}
        }
    }
    out
}

/// Lista de renglones táctiles (título, cifras y estado) en un grupo con
/// divisores finos. Devuelve el renglón tocado.
pub fn list(
    ui: &mut egui::Ui,
    p: &Palette,
    rows: &[(String, String, Option<Status>)],
) -> Option<usize> {
    let mut tapped = None;
    if rows.is_empty() {
        return None;
    }
    egui::Frame::new()
        .fill(p.raise)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(theme::RADIUS_CARD)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            for (k, (title, figures, status)) in rows.iter().enumerate() {
                if k > 0 {
                    let x = ui.max_rect().x_range();
                    ui.painter().hline(
                        (x.min + theme::SP3)..=x.max,
                        ui.cursor().top(),
                        egui::Stroke::new(1.0, p.hair),
                    );
                }
                let response = egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_min_height(theme::TOUCH_H - theme::SP4);
                        ui.horizontal(|ui| {
                            let title_w = (ui.available_width() - 132.0).max(96.0);
                            ui.allocate_ui_with_layout(
                                Vec2::new(title_w, theme::SP6),
                                Layout::left_to_right(Align::Center),
                                |ui| ui.add(egui::Label::new(strong(title.as_str())).truncate()),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(icon::CARET_RIGHT).color(p.ink3));
                                if let Some(status) = status {
                                    status_chip(ui, p, *status);
                                }
                            });
                        });
                        if !figures.is_empty() {
                            ui.add(
                                egui::Label::new(
                                    muted(p, figures.as_str())
                                        .monospace()
                                        .size(theme::SECONDARY),
                                )
                                .wrap(),
                            );
                        }
                    })
                    .response
                    .interact(Sense::click());
                if response.clicked() {
                    tapped = Some(k);
                }
            }
        });
    tapped
}

// ------------------------------------------------------------- unifilar

/// El unifilar en el teléfono: el dibujo completo y, al tocar un tablero o
/// un circuito, una hoja inferior con su resumen y «Abrir ficha». Devuelve
/// `true` si se pidió abrir la ficha.
pub fn diagram_view(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) -> bool {
    let p = shared.palette;
    let mut open_ficha = false;
    if let Some((i, c)) = shared.sheet {
        egui::Panel::bottom("hoja-unifilar")
            .frame(
                egui::Frame::new()
                    .fill(p.raise)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(egui::CornerRadius {
                        nw: theme::RADIUS_WINDOW,
                        ne: theme::RADIUS_WINDOW,
                        sw: 0,
                        se: 0,
                    })
                    .shadow(ui.style().visuals.popup_shadow)
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show_separator_line(false)
            .show(ui, |ui| {
                let (title, item) = match c {
                    Some(c) => (
                        doc.project
                            .panels
                            .get(i)
                            .and_then(|panel| panel.circuits.get(c))
                            .map(|circuit| format!("{} · {}", circuit.id, circuit.name)),
                        doc.panel_result(i).and_then(|r| r.circuits.get(c)).cloned(),
                    ),
                    None => (
                        doc.project
                            .panels
                            .get(i)
                            .map(|panel| format!("{} · {}", panel.id, panel.name)),
                        doc.panel_result(i).map(|r| r.feeder.clone()),
                    ),
                };
                let Some(title) = title else {
                    shared.sheet = None;
                    return;
                };
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(strong(title).size(theme::HEAD)).truncate());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new(icon::X).size(theme::ICON_BAR))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            shared.sheet = None;
                        }
                    });
                });
                if let Some(item) = &item {
                    ui.horizontal(|ui| {
                        status_chip(ui, &p, item.status);
                    });
                    ui.label(muted(&p, card_figures(item)).monospace());
                    for check in item.checks.iter().filter(|c| c.status != Status::Complies) {
                        ui.label(
                            RichText::new(format!("{} {}", status_icon(check.status), check.name))
                                .color(status_color(&p, check.status)),
                        );
                    }
                }
                ui.add_space(theme::SP2);
                if primary_button(ui, &p, &format!("{}  Abrir ficha", icon::ARROW_RIGHT)).clicked()
                {
                    open_ficha = true;
                }
            });
    }
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(ui, |ui| {
            let Some(drawing) = shared.diagram(doc, shared.diagram_mode) else {
                crate::widgets::empty_state(
                    ui,
                    &p,
                    icon::WARNING_CIRCLE,
                    "No se pudo calcular el proyecto",
                    &doc.calc_error().unwrap_or_default(),
                );
                return;
            };
            if shared.diagram_fit {
                shared.diagram_zoom = ((ui.available_width() - theme::SP4) / drawing.width as f32)
                    .clamp(diagram::ZOOM_RANGE.0, 1.3);
                shared.diagram_fit = false;
            }
            let selected = match doc.selection {
                Selection::Feeder(i) | Selection::Secondary(i) => Some((i, None)),
                Selection::Circuit(i, c) => Some((i, Some(c))),
                Selection::Project => None,
            };
            let before = shared.diagram_mode;
            if let Some(hit) = diagram::show(
                ui,
                &p,
                &drawing,
                &mut shared.diagram_zoom,
                &mut shared.diagram_mode,
                selected,
            ) {
                doc.selection = match hit {
                    (i, Some(c)) => Selection::Circuit(i, c),
                    (i, None) => Selection::Feeder(i),
                };
                shared.sheet = Some(hit);
            }
            if before != shared.diagram_mode {
                shared.diagram_fit = true;
            }
        });
    if open_ficha {
        shared.sheet = None;
        shared.ficha_open = true;
    }
    open_ficha
}

// ---------------------------------------------------------------- cajones

/// De qué lado sale un cajón.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// Cajón que se abre encima del contenido, con un velo detrás: tocar fuera
/// lo cierra. `area` es lo que puede cubrir (entre las barras de arriba y
/// abajo).
#[allow(clippy::too_many_arguments)]
pub fn drawer(
    ctx: &egui::Context,
    p: &Palette,
    id: &str,
    title: &str,
    side: Side,
    width: f32,
    area: egui::Rect,
    open: &mut bool,
    add: impl FnOnce(&mut egui::Ui),
) {
    let id = egui::Id::new(id);
    // Se desliza en 150 ms, lo más que permite el sistema visual.
    let t = ctx.animate_bool_with_time(id.with("abierto"), *open, 0.15);
    if t == 0.0 {
        return;
    }
    let width = crate::layout::drawer_width(area.width(), width);
    let veil = egui::Area::new(id.with("velo"))
        .order(egui::Order::Middle)
        .fixed_pos(area.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(area.size(), Sense::click());
            ui.painter().rect_filled(
                rect,
                0.0,
                Color32::from_black_alpha((if p.dark { 110.0 } else { 60.0 } * t) as u8),
            );
            response.clicked()
        });
    if veil.inner {
        *open = false;
    }
    let x = match side {
        Side::Left => area.left() - width * (1.0 - t),
        Side::Right => area.right() - width * t,
    };
    let panel = egui::Area::new(id)
        .order(egui::Order::Middle)
        .fixed_pos(egui::pos2(x, area.top()))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.panel)
                .stroke(egui::Stroke::new(1.0, p.line))
                .shadow(ui.style().visuals.popup_shadow)
                .show(ui, |ui| {
                    ui.set_width(width);
                    ui.set_height(area.height());
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(strong(title).size(theme::HEAD));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(icon::X).size(theme::ICON_BAR),
                                            )
                                            .frame(false),
                                        )
                                        .on_hover_text("Cerrar")
                                        .clicked()
                                    {
                                        *open = false;
                                    }
                                });
                            });
                        });
                    add(ui);
                });
        });
    // El cajón siempre encima de su velo.
    ctx.move_to_top(panel.response.layer_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phone_goes_from_panels_to_circuits_to_the_ficha_and_back() {
        let house = ieestudio_project::example::house();
        let mut doc = Doc::new(house, None);
        let mut shared = Shared::new(Palette::new(true));
        doc.selection = Selection::Project;
        assert_eq!(level(doc.selection, shared.ficha_open), Level::Panels);
        assert_eq!(back(View::Schedule, &doc, &shared), None);
        // Tocar un tablero lleva a sus circuitos.
        doc.selection = Selection::Feeder(0);
        assert_eq!(level(doc.selection, shared.ficha_open), Level::Circuits(0));
        assert_eq!(
            back(View::Schedule, &doc, &shared).as_deref(),
            Some("Tableros")
        );
        // Tocar un circuito abre su ficha; «atrás» dice a qué tablero vuelve.
        doc.selection = Selection::Circuit(0, 2);
        shared.ficha_open = true;
        assert_eq!(level(doc.selection, shared.ficha_open), Level::Ficha);
        assert_eq!(back(View::Schedule, &doc, &shared).as_deref(), Some("TG-1"));
        go_back(&mut doc, &mut shared);
        assert_eq!(level(doc.selection, shared.ficha_open), Level::Circuits(0));
        go_back(&mut doc, &mut shared);
        assert_eq!(level(doc.selection, shared.ficha_open), Level::Panels);
        // En otras vistas no hay «atrás».
        assert_eq!(back(View::Diagram, &doc, &shared), None);
    }

    #[test]
    fn the_circuit_card_shows_load_cable_breaker_and_drop() {
        let doc = Doc::new(ieestudio_project::example::house(), None);
        let result = doc.panel_result(0).unwrap();
        let figures = card_figures(&result.circuits[0]);
        assert!(figures.contains(" W · "), "{figures}");
        assert!(figures.contains("AWG"), "{figures}");
        assert!(figures.contains("P-"), "{figures}");
        assert!(figures.ends_with(" %"), "{figures}");
    }

    #[test]
    fn the_worst_status_wins() {
        let doc = Doc::new(ieestudio_project::example::house(), None);
        let result = doc.panel_result(0).unwrap();
        // La casa de ejemplo tiene dos circuitos que no cumplen.
        assert_eq!(worst(result.circuits.iter()), Status::Fails);
        assert_eq!(worst(std::iter::empty()), Status::Complies);
    }

    #[test]
    fn the_five_destinations_have_short_names() {
        for view in View::ALL {
            assert!(short_title(view).chars().count() <= 9, "{view:?}");
        }
        assert_eq!(short_title(View::Schedule), "Circuitos");
    }
}
