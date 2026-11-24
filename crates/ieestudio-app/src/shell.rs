//! El marco único: pestañas de las vistas y la disposición de las regiones
//! (navegador, inspector, procedimiento) alrededor de la vista activa.

use eframe::egui::{self, Align, Layout, RichText, Sense};
use egui_phosphor::regular as icon;

use crate::compact;
use crate::layout::{self, Class, Regions};
use crate::panels::{procedure, Shared, Viewer};
use crate::state::Doc;
use crate::theme::{semibold, Palette};
use crate::views::{Export, Panel, View};
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
    let (fails, pending) = crate::panels::review::problem_counts(doc);
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
    // En tableta todas van solo con su ícono, también la activa: así caben
    // las cinco junto a los botones de los cajones.
    let icons_only = compact && ui.ctx().content_rect().width() < layout::WIDE;
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
                    if (now && !icons_only) || !compact {
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

/// Lo que una vista pide al terminar el cuadro.
pub struct Outcome {
    /// Vista a la que se pidió ir, por ejemplo desde una incidencia de
    /// Revisar en el teléfono.
    pub go: Option<View>,
    pub request: Option<Export>,
}

/// Estado de la interfaz que decide cómo se arma la vista activa.
pub struct Arrangement {
    pub view: View,
    pub regions: Regions,
    pub procedure: bool,
    /// Escritorio, tableta o teléfono.
    pub class: Class,
}

/// Dibuja la vista activa dentro de `ui`, con sus regiones.
pub fn show_view(
    ui: &mut egui::Ui,
    arr: &mut Arrangement,
    doc: &mut Doc,
    shared: &mut Shared,
) -> Outcome {
    let area = ui.max_rect();
    let before = doc.selection;
    let phone = arr.class == Class::Phone;
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
                        go = Some(View::Schedule);
                    }
                });
            (go, None)
        }
        View::Review => (
            crate::panels::review::show(ui, doc, shared, arr.class),
            None,
        ),
        View::Report => (None, crate::panels::report::view(ui, doc, shared)),
        View::Schedule if phone => {
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(shared.palette.panel))
                .show(ui, |ui| compact::schedule(ui, doc, shared));
            (None, None)
        }
        View::Diagram if phone => {
            let open = compact::diagram_view(ui, doc, shared);
            (open.then_some(View::Schedule), None)
        }
        View::Schedule | View::Diagram => workbench(ui, arr, doc, shared),
    };
    if arr.class == Class::Tablet && arr.view.has_inspector() {
        // Elegir un tablero o circuito abre su ficha en el cajón de la
        // derecha; en el navegador, además, cierra el cajón del navegador.
        if doc.selection != before && doc.selection != crate::state::Selection::Project {
            shared.inspector_drawer = true;
            shared.navigator_drawer = false;
        }
        tablet_drawers(ui.ctx(), area, doc, shared);
    }
    Outcome { go, request }
}

/// Los cajones de la tableta: navegador a la izquierda y ficha a la derecha.
fn tablet_drawers(ctx: &egui::Context, area: egui::Rect, doc: &mut Doc, shared: &mut Shared) {
    let p = shared.palette;
    let mut open = shared.navigator_drawer;
    compact::drawer(
        ctx,
        &p,
        "cajon-navegador",
        "Navegador",
        compact::Side::Left,
        layout::NAVIGATOR_W + 40.0,
        area,
        &mut open,
        |ui| Viewer { doc, shared }.panel(ui, Panel::Browser),
    );
    shared.navigator_drawer = open;
    let mut open = shared.inspector_drawer;
    compact::drawer(
        ctx,
        &p,
        "cajon-ficha",
        "Ficha",
        compact::Side::Right,
        layout::INSPECTOR_W + 32.0,
        area,
        &mut open,
        |ui| Viewer { doc, shared }.panel(ui, Panel::Properties),
    );
    shared.inspector_drawer = open;
}

/// Las vistas de trabajo: navegador a la izquierda, inspector a la derecha,
/// procedimiento abajo. En ventanas sin regiones laterales cada vista se
/// muestra sola, con el contenido que ya tenía en esa anchura.
fn workbench(
    ui: &mut egui::Ui,
    arr: &mut Arrangement,
    doc: &mut Doc,
    shared: &mut Shared,
) -> (Option<View>, Option<Export>) {
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

    let mut request = None;
    egui::CentralPanel::default()
        .frame(frame(egui::Margin::ZERO))
        .show(ui, |ui| match arr.view {
            View::Diagram if !regions.split => {
                // Elegir en el dibujo muestra la ficha en el inspector o, en
                // tableta, en su cajón, sin salir del unifilar.
                request = crate::panels::diagram::view(ui, doc, shared);
            }
            _ => {
                Viewer { doc, shared }.panel(ui, Panel::Schedule);
            }
        });
    (None, request)
}
