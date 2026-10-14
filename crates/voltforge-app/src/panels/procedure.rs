//! Procedimiento del elemento seleccionado, agrupado y contraíble.

use eframe::egui::{self, RichText};
use egui_phosphor::regular as icon;

use super::Shared;
use crate::state::{Doc, Selection};
use crate::widgets::{empty_state, muted, steps_ui, strong};

pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) {
    let p = shared.palette;
    let title = match doc.selection {
        Selection::Project => None,
        Selection::Feeder(i) => Some(format!("Alimentador de {}", doc.project.panels[i].id)),
        Selection::Secondary(i) => Some(format!(
            "Secundario del transformador de {}",
            doc.project.panels[i].id
        )),
        Selection::Circuit(i, c) => {
            let circuit = &doc.project.panels[i].circuits[c];
            Some(format!("{} · {}", circuit.id, circuit.name))
        }
    };
    let Some(title) = title else {
        empty_state(
            ui,
            &p,
            icon::LIST_BULLETS,
            "Selecciona un alimentador o un circuito",
            "Aquí aparece su procedimiento paso a paso.",
        );
        return;
    };
    let mut open = None;
    ui.horizontal(|ui| {
        ui.label(strong(title));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Contraer todo").clicked() {
                open = Some(false);
            }
            if ui.button("Expandir todo").clicked() {
                open = Some(true);
            }
        });
    });
    ui.separator();
    let Some(run) = doc.selected_run().and_then(|r| r.run.clone()) else {
        // Sin procedimiento: decir por qué, no solo que falta.
        let reasons: Vec<String> = match (doc.calc_error(), doc.selected_run()) {
            (Some(e), _) => vec![e],
            (None, Some(item)) => item
                .checks
                .iter()
                .filter(|c| c.status != voltforge_project::Status::Complies)
                .map(|c| format!("{}: {}", c.name, c.detail))
                .collect(),
            (None, None) => vec![],
        };
        ui.label(strong("Este tramo no se pudo calcular").color(p.warn));
        for r in reasons {
            ui.label(muted(&p, r));
        }
        return;
    };
    egui::ScrollArea::vertical()
        .id_salt("procedimiento")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            steps_ui(ui, &p, ("proc", doc.selection_key()), &run.steps, open);
            ui.label(
                RichText::new(
                    "Valores redondeados para presentar; el cálculo no redondea pasos intermedios.",
                )
                .size(11.0)
                .color(p.ink3),
            );
        });
}
