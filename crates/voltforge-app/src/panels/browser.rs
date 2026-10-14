//! Navegador del proyecto: el árbol de la instalación, como el Project
//! Browser de Revit o el panel Proyecto de Premiere.

use eframe::egui::{self, RichText, Sense, Vec2};
use egui_phosphor::regular as icon;
use voltforge_project::Status;

use super::Shared;
use crate::state::{Doc, Selection};
use crate::theme::Palette;
use crate::widgets::{muted, status_glyph, strong, worst};

pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) {
    let p = shared.palette;
    ui.horizontal(|ui| {
        if ui
            .button(format!("{} Tablero", icon::PLUS))
            .on_hover_text(
                "Agregar un tablero. Si hay uno seleccionado, queda como subtablero suyo.",
            )
            .clicked()
        {
            doc.add_panel();
        }
        if let Some(panel) = doc.panel_index() {
            if ui
                .button(format!("{} Circuito", icon::PLUS))
                .on_hover_text("Agregar un circuito al tablero activo")
                .clicked()
            {
                doc.add_circuit(panel);
            }
        }
    });
    ui.add_space(4.0);

    let mut select = None;
    // Proyecto y fuente.
    let project_selected = doc.selection == Selection::Project;
    if item(
        ui,
        &p,
        0,
        icon::FOLDER_OPEN,
        &doc.project.name,
        "",
        project_selected,
        doc.result.as_ref().ok().map(|r| {
            worst(r.panels.iter().flat_map(|pr| {
                std::iter::once(pr.feeder.status).chain(pr.circuits.iter().map(|c| c.status))
            }))
        }),
    )
    .clicked()
    {
        select = Some(Selection::Project);
    }
    let system = &doc.project.source.system;
    let volts = match (system.line_to_line_v, system.line_to_neutral_v) {
        (Some(ll), Some(ln)) => format!("{ll:.0}/{ln:.0} V"),
        (Some(ll), None) => format!("{ll:.0} V"),
        (None, Some(ln)) => format!("{ln:.0} V"),
        (None, None) => String::new(),
    };
    item(
        ui,
        &p,
        1,
        icon::PLUG,
        &doc.project.source.name,
        &volts,
        false,
        None,
    );

    // Tableros en árbol: primero los que cuelgan de la fuente.
    let roots: Vec<usize> = doc
        .project
        .panels
        .iter()
        .enumerate()
        .filter(|(_, pn)| pn.parent.is_none())
        .map(|(i, _)| i)
        .collect();
    let mut shown = vec![false; doc.project.panels.len()];
    for root in roots {
        panel_tree(ui, doc, &p, root, 1, &mut select, &mut shown);
    }
    // Tableros que no cuelgan de nada que exista (o que forman un ciclo): se
    // muestran aparte para que el usuario pueda corregirlos.
    let lost: Vec<usize> = (0..shown.len()).filter(|&i| !shown[i]).collect();
    if !lost.is_empty() {
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("{} Sin alimentación válida", icon::WARNING))
                .size(11.0)
                .color(p.warn),
        )
        .on_hover_text("Revisa «Se alimenta de» en Propiedades.");
        for i in lost {
            if !shown[i] {
                panel_tree(ui, doc, &p, i, 1, &mut select, &mut shown);
            }
        }
    }

    if let Some(sel) = select {
        doc.selection = sel;
    }

    ui.add_space(10.0);
    ui.label(
        muted(
            &p,
            "Clic para seleccionar; las demás vistas siguen la selección.",
        )
        .size(11.0),
    );
}

#[allow(clippy::too_many_arguments)]
fn panel_tree(
    ui: &mut egui::Ui,
    doc: &Doc,
    p: &Palette,
    index: usize,
    depth: usize,
    select: &mut Option<Selection>,
    shown: &mut [bool],
) {
    // Cada tablero se dibuja una sola vez, aunque el archivo traiga un ciclo.
    if shown[index] {
        return;
    }
    shown[index] = true;
    let panel = &doc.project.panels[index];
    let result = doc.panel_result(index);
    let status = result.map(|r| worst(r.runs().map(|c| c.status)));
    let selected = doc.selection == Selection::Feeder(index);
    let id = ui.make_persistent_id(("tablero", index));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
    let open = state.is_open();
    let response = ui
        .horizontal(|ui| {
            ui.add_space(depth as f32 * 14.0 - 14.0);
            let caret = if open {
                icon::CARET_DOWN
            } else {
                icon::CARET_RIGHT
            };
            if ui
                .add(egui::Label::new(RichText::new(caret).color(p.ink3)).sense(Sense::click()))
                .clicked()
            {
                state.toggle(ui);
            }
            item_inline(
                ui,
                p,
                icon::SQUARES_FOUR,
                &format!("{} {}", panel.id, panel.name),
                &format!("{} circ.", panel.circuits.len()),
                selected,
                status,
            )
        })
        .inner;
    if response.clicked() {
        *select = Some(Selection::Feeder(index));
    }
    state.store(ui.ctx());
    if !open {
        return;
    }
    if let Some(t) = &panel.transformer {
        let st = result.and_then(|r| r.secondary.as_ref()).map(|r| r.status);
        if item(
            ui,
            p,
            depth + 1,
            icon::SWAP,
            &format!("Transformador {} kVA", t.kva),
            "secundario",
            doc.selection == Selection::Secondary(index),
            st,
        )
        .clicked()
        {
            *select = Some(Selection::Secondary(index));
        }
    }
    for (c, circuit) in panel.circuits.iter().enumerate() {
        let st = result.and_then(|r| r.circuits.get(c)).map(|r| r.status);
        if item(
            ui,
            p,
            depth + 1,
            icon::LINE_SEGMENTS,
            &format!("{} {}", circuit.id, circuit.name),
            "",
            doc.selection == Selection::Circuit(index, c),
            st,
        )
        .clicked()
        {
            *select = Some(Selection::Circuit(index, c));
        }
    }
    let children: Vec<usize> = doc
        .project
        .panels
        .iter()
        .enumerate()
        .filter(|(_, pn)| pn.parent.as_deref() == Some(panel.id.as_str()))
        .map(|(i, _)| i)
        .collect();
    for child in children {
        panel_tree(ui, doc, p, child, depth + 1, select, shown);
    }
}

/// Fila del árbol con sangría.
#[allow(clippy::too_many_arguments)]
fn item(
    ui: &mut egui::Ui,
    p: &Palette,
    depth: usize,
    glyph: &str,
    text: &str,
    detail: &str,
    selected: bool,
    status: Option<Status>,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 14.0 + 4.0);
        item_inline(ui, p, glyph, text, detail, selected, status)
    })
    .inner
}

fn item_inline(
    ui: &mut egui::Ui,
    p: &Palette,
    glyph: &str,
    text: &str,
    detail: &str,
    selected: bool,
    status: Option<Status>,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::click());
    let painter = ui.painter_at(rect);
    if selected {
        painter.rect_filled(rect, 3.0, p.selected);
        painter.rect_filled(
            egui::Rect::from_min_size(rect.left_top(), Vec2::new(2.0, rect.height())),
            0.0,
            p.accent,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, 3.0, p.raise);
    }
    // Primero lo que va a la derecha (estado y detalle); el nombre ocupa el
    // resto y se recorta si no cabe.
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(6.0, 0.0)))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    if let Some(s) = status {
        child.label(status_glyph(p, s));
    }
    if !detail.is_empty() {
        child.label(RichText::new(detail).size(11.0).color(p.ink3));
    }
    child.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.label(RichText::new(glyph).color(p.ink2));
        ui.add(
            egui::Label::new(if selected {
                strong(text)
            } else {
                RichText::new(text)
            })
            .truncate()
            .selectable(false),
        );
    });
    response
}
