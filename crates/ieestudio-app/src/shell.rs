//! El marco único: pestañas de las vistas y la disposición de las regiones
//! (navegador, inspector, procedimiento) alrededor de la vista activa.

use eframe::egui::{self, Align, Layout, RichText, Sense};
use egui_phosphor::regular as icon;

use crate::guided::{self, Export, Step};
use crate::layout::{self, Regions};
use crate::panels::{procedure, Shared, Viewer};
use crate::state::Doc;
use crate::theme::{semibold, Palette};
use crate::views::{Panel, View};
use crate::widgets::strong;

/// Las cinco vistas como pestañas de la barra superior. `current` es `None`
/// en la pantalla de inicio. Con `compact`, las vistas que no son la actual
/// muestran solo su ícono, para que la barra quepa en ventanas angostas.
/// Devuelve la vista elegida.
pub fn view_tabs(
    ui: &mut egui::Ui,
    p: &Palette,
    current: Option<View>,
    doc: &Doc,
    compact: bool,
) -> Option<View> {
    let mut chosen = None;
    let (fails, pending) = guided::problem_counts(doc);
    // Una sola cápsula: la vista activa va elevada sobre ella. Sin subrayado.
    egui::Frame::new()
        .fill(p.selected)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(CAPSULE)
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.horizontal(|ui| {
                chosen = capsule_items(ui, p, current, compact, fails, pending);
            });
        });
    chosen
}

/// Radio de una cápsula: el mayor que admite egui, así cualquier alto queda
/// redondo en los extremos.
pub const CAPSULE: u8 = u8::MAX;

fn capsule_items(
    ui: &mut egui::Ui,
    p: &Palette,
    current: Option<View>,
    compact: bool,
    fails: usize,
    pending: usize,
) -> Option<View> {
    let mut chosen = None;
    for view in View::ALL {
        let now = current == Some(view);
        let ink = if now { p.ink } else { p.ink2 };
        let response = egui::Frame::new()
            .fill(if now {
                p.raise
            } else {
                egui::Color32::TRANSPARENT
            })
            .shadow(if now {
                egui::Shadow {
                    offset: [0, 1],
                    blur: 2,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(if p.dark { 90 } else { 36 }),
                }
            } else {
                egui::Shadow::NONE
            })
            .corner_radius(CAPSULE)
            .inner_margin(egui::Margin::symmetric(12, 2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.label(
                        RichText::new(view.icon())
                            .size(crate::theme::ICON)
                            .color(if now { p.accent } else { p.ink3 }),
                    );
                    if now || !compact {
                        let label = RichText::new(view.title()).color(ink);
                        ui.label(if now { label.family(semibold()) } else { label });
                    }
                    // Revisar avisa cuánto falta desde cualquier vista: el
                    // mismo total que lista esa vista.
                    if view == View::Review && fails + pending > 0 {
                        let color = if fails > 0 { p.bad } else { p.warn };
                        crate::widgets::count_badge(ui, color, fails + pending);
                    }
                });
            })
            .response
            .interact(Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        let tip = if view == View::Review && fails + pending > 0 {
            format!(
                "{} ({}): {} por resolver ({fails} no {}, {pending} sin evaluar)",
                view.title(),
                view.shortcut(),
                fails + pending,
                if fails == 1 { "cumple" } else { "cumplen" }
            )
        } else {
            format!("{} ({})", view.title(), view.shortcut())
        };
        let response = response.on_hover_text(tip);
        if response.clicked() && !now {
            chosen = Some(view);
        }
    }
    chosen
}

/// A qué vista, y a qué contenido de «Tableros y circuitos» en ventanas sin
/// inspector, lleva un paso que pidió una vista (por ejemplo, un renglón de
/// Revisar que se arregla en el calibre de un circuito).
pub fn target(step: Step) -> (View, Option<Step>) {
    match step {
        Step::Panels | Step::Circuits => (View::Schedule, Some(step)),
        Step::Review => (View::Review, None),
        Step::Report => (View::Report, None),
    }
}

/// Lo que una vista pide al terminar el cuadro.
pub struct Outcome {
    pub go: Option<Step>,
    pub request: Option<Export>,
}

/// Estado de la interfaz que decide cómo se arma la vista activa.
pub struct Arrangement {
    pub view: View,
    /// Contenido de «Tableros y circuitos» en ventanas sin inspector.
    pub sub: Step,
    pub regions: Regions,
    pub procedure: bool,
}

/// Dibuja la vista activa dentro de `ui`, con sus regiones.
pub fn show_view(
    ui: &mut egui::Ui,
    arr: &mut Arrangement,
    doc: &mut Doc,
    shared: &mut Shared,
) -> Outcome {
    let (go, request) = match arr.view {
        View::Project => {
            let mut go = None;
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(shared.palette.panel)
                        .inner_margin(egui::Margin::symmetric(16, 0)),
                )
                .show(ui, |ui| {
                    if let Some(i) = crate::panels::project::show(ui, doc, shared) {
                        doc.selection = crate::state::Selection::Feeder(i);
                        go = Some(Step::Panels);
                    }
                });
            (go, None)
        }
        View::Review => guided::view(ui, Step::Review, doc, shared),
        View::Report => guided::view(ui, Step::Report, doc, shared),
        View::Schedule | View::Diagram => workbench(ui, arr, doc, shared),
    };
    Outcome { go, request }
}

/// Las vistas de trabajo: navegador a la izquierda, inspector a la derecha,
/// procedimiento abajo. En ventanas sin regiones laterales cada vista se
/// muestra sola, con el contenido que ya tenía en esa anchura.
fn workbench(
    ui: &mut egui::Ui,
    arr: &mut Arrangement,
    doc: &mut Doc,
    shared: &mut Shared,
) -> (Option<Step>, Option<Export>) {
    let p = shared.palette;
    let regions = arr.regions;
    let frame = |margin: egui::Margin| egui::Frame::new().fill(p.panel).inner_margin(margin);

    if arr.procedure {
        let (default, min, max) = layout::bottom_panel(ui.available_height());
        egui::Panel::bottom("procedimiento")
            .resizable(true)
            .default_size(default.min(260.0))
            .size_range(min..=max)
            .frame(frame(egui::Margin::symmetric(12, 8)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon::LIST_BULLETS).color(p.accent));
                    ui.label(strong("Procedimiento"));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new(icon::X).frame(false))
                            .on_hover_text("Cerrar el procedimiento (F7)")
                            .clicked()
                        {
                            arr.procedure = false;
                        }
                    });
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("procedimiento-inferior")
                    .auto_shrink([false, false])
                    .show(ui, |ui| procedure::show(ui, doc, shared));
            });
    }
    if regions.navigator {
        egui::Panel::left("navegador")
            .resizable(true)
            .default_size(layout::NAVIGATOR_W)
            .size_range(200.0..=360.0)
            // El navegador es parte del cromo: un tono abajo del contenido.
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| {
                Viewer { doc, shared }.panel(ui, Panel::Browser);
            });
    }
    if regions.inspector {
        egui::Panel::right("inspector")
            .resizable(true)
            .default_size(layout::INSPECTOR_W)
            .size_range(280.0..=520.0)
            .frame(frame(egui::Margin::ZERO))
            .show(ui, |ui| {
                Viewer { doc, shared }.panel(ui, Panel::Properties);
            });
    }
    if regions.split {
        // Dos vistas a la vez: el cuadro a la izquierda y el unifilar a la
        // derecha, sincronizados por la misma selección.
        let width = ui.available_width() / 2.0;
        egui::Panel::right("segunda-vista")
            .resizable(false)
            .exact_size(width)
            .frame(frame(egui::Margin::ZERO))
            .show(ui, |ui| {
                Viewer { doc, shared }.panel(ui, Panel::Diagram);
            });
    }

    let mut go = None;
    let mut request = None;
    let narrow = !regions.inspector;
    egui::CentralPanel::default()
        .frame(frame(egui::Margin::ZERO))
        .show(ui, |ui| match arr.view {
            View::Diagram if !regions.split => {
                (go, request) = guided::diagram_view(ui, doc, shared, narrow);
            }
            _ if narrow => {
                // Sin inspector, «Tableros y circuitos» conserva sus dos
                // contenidos de antes, uno a la vez.
                egui::Panel::top("tableros-o-circuitos")
                    .frame(
                        egui::Frame::new()
                            .fill(p.bg)
                            .inner_margin(egui::Margin::symmetric(16, 8)),
                    )
                    .show_separator_line(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (step, text) in
                                [(Step::Panels, "Tableros"), (Step::Circuits, "Circuitos")]
                            {
                                if ui.selectable_label(arr.sub == step, text).clicked() {
                                    arr.sub = step;
                                }
                            }
                        });
                    });
                (go, request) = guided::view(ui, arr.sub, doc, shared);
            }
            _ => {
                Viewer { doc, shared }.panel(ui, Panel::Schedule);
            }
        });
    (go, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_lands_on_one_of_the_five_views() {
        for step in [Step::Panels, Step::Circuits, Step::Review, Step::Report] {
            let (view, sub) = target(step);
            assert!(View::ALL.contains(&view));
            // Solo «Tableros y circuitos» recuerda a cuál de sus dos contenidos
            // se quiere ir.
            assert_eq!(sub.is_some(), view == View::Schedule);
        }
        assert_eq!(target(Step::Review), (View::Review, None));
        assert_eq!(target(Step::Panels), (View::Schedule, Some(Step::Panels)));
        assert_eq!(
            target(Step::Circuits),
            (View::Schedule, Some(Step::Circuits))
        );
    }
}
