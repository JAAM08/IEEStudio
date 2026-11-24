//! Vista Revisar: el dictamen del proyecto y todo lo que falta, en un solo
//! lugar, con el arreglo a la mano.
//!
//! En escritorio la ficha del elemento elegido va a la derecha, sin salir de
//! la vista; en tableta, en su cajón; en el teléfono, a pantalla completa en
//! «Circuitos». Las incidencias se arman una vez por cambio del proyecto.

use std::rc::Rc;

use eframe::egui::{self, Align, Layout, RichText};
use egui_phosphor::regular as icon;
use ieestudio_project::{size_label, Check, LoadKind, RunResult, Status};
use ieestudio_rules::{cite_all, Profile, Rule};

use super::{Shared, Viewer};
use crate::layout::{self, Class};
use crate::state::{set_load_kind, Doc, Selection};
use crate::theme::{self, semibold, Palette};
use crate::views::{Panel, View};
use crate::widgets::{
    label_caps, label_of, muted, page_title, proposed_instead, sentence, status_color, status_icon,
    strong, thousands, KINDS,
};

/// Incidencias que se muestran de entrada; las demás, con «Mostrar más».
const FIRST: usize = 40;

/// Verificaciones que no cumplen y que quedaron sin evaluar en todo el
/// proyecto, contando las de los tramos que no se pudieron calcular.
pub fn problem_counts(doc: &Doc) -> (usize, usize) {
    let Ok(result) = &doc.result else {
        return (0, 0);
    };
    let v = &result.verdict;
    // Los datos inválidos también quedan por resolver; un tramo inválido ya
    // está entre los sin calcular.
    (
        v.failed_checks,
        v.pending_checks + v.uncalculated + v.invalid_data - v.invalid_runs,
    )
}

/// Lo que Revisar arma una vez por revisión del proyecto.
#[derive(Default)]
pub struct Cache {
    revision: Option<u64>,
    items: Rc<Vec<Pending>>,
    /// Incidencias a la vista; crece con «Mostrar más».
    shown: usize,
}

impl Cache {
    fn items(&mut self, doc: &Doc) -> Rc<Vec<Pending>> {
        if self.revision != Some(doc.revision) {
            self.revision = Some(doc.revision);
            self.items = Rc::new(review_items(doc));
        }
        self.items.clone()
    }
}

/// Tramos calculados y tramos en total (alimentadores, secundarios y
/// circuitos).
fn runs_calculated(doc: &Doc) -> (usize, usize) {
    let Ok(result) = &doc.result else {
        return (0, 0);
    };
    let runs = result.panels.iter().flat_map(|panel| {
        std::iter::once(&panel.feeder)
            .chain(panel.secondary.iter())
            .chain(panel.circuits.iter())
    });
    runs.fold((0, 0), |(done, all), r| {
        (done + usize::from(r.run.is_some()), all + 1)
    })
}

/// La vista Revisar. Devuelve la vista a la que se pidió ir.
pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared, class: Class) -> Option<View> {
    let p = shared.palette;
    let mut go = None;
    if class == Class::Desktop {
        egui::Panel::right("revisar-ficha")
            .resizable(true)
            .default_size(layout::INSPECTOR_W)
            .size_range(280.0..=520.0)
            .frame(egui::Frame::new().fill(p.panel))
            .show(ui, |ui| Viewer { doc, shared }.panel(ui, Panel::Properties));
    }
    let margin = if class == Class::Phone { 16 } else { 24 };
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(margin, 16)),
        )
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("revisar")
                .auto_shrink([false, false])
                .show(ui, |ui| go = content(ui, &p, doc, shared, class));
        });
    go
}

/// Lo que pide elegir un elemento desde Revisar: en el teléfono se va a su
/// ficha; en los demás tamaños se queda aquí y la ficha se ve al lado.
fn pick(doc: &mut Doc, selection: Selection, class: Class) -> Option<View> {
    doc.selection = selection;
    (class == Class::Phone).then_some(View::Schedule)
}

fn content(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &mut Doc,
    shared: &mut Shared,
    class: Class,
) -> Option<View> {
    let mut go = None;
    page_title(
        ui,
        p,
        "¿Está listo para entregar?",
        "Todo lo que falta, en un solo lugar, con su arreglo.",
    );
    let Ok(result) = doc.result.clone() else {
        ui.label(RichText::new(doc.calc_error().unwrap_or_default()).color(p.bad));
        return None;
    };
    verdict_cards(ui, p, &result);
    ui.add_space(theme::SP3);
    counters(ui, p, doc, &result, class);

    // Antes que nada, lo que no se puede calcular: conexiones que el
    // sistema no tiene.
    if connection_review(ui, p, doc) {
        ieestudio_project::connection::fix(&mut doc.project);
        doc.edited();
    }
    // Sistemas sin topología o que no corresponden a la declarada (decisión
    // 0030): se declara aquí mismo.
    if let Some((at, topology)) = system_review(ui, p, doc) {
        match at {
            None => doc.project.source.system.topology = Some(topology),
            Some(i) => {
                if let Some(t) = &mut doc.project.panels[i].transformer {
                    t.secondary.topology = Some(topology);
                }
            }
        }
        doc.edited();
    }
    // Primero el tipo de carga: cambia todo lo demás del circuito.
    match kind_review(ui, p, doc) {
        Some(KindAction::Change(i, c, kind)) => {
            set_load_kind(&mut doc.project.panels[i].circuits[c].load, kind);
            doc.edited();
            go = pick(doc, Selection::Circuit(i, c), class);
        }
        // Se guarda en el proyecto: deja de avisar en Revisar, la memoria y
        // la línea de comandos (decisión 0029, D-3).
        Some(KindAction::Confirm(i, c)) => {
            doc.project.panels[i].circuits[c].load.confirmed_general = true;
            doc.edited();
        }
        None => {}
    }
    // Transformadores que pueden ser el de la acometida otra vez.
    match duplicate_review(ui, p, doc) {
        Some(CardAction::Changed) => doc.edited(),
        Some(CardAction::Go(i)) => go = pick(doc, Selection::Feeder(i), class),
        None => {}
    }

    let items = shared.review.items(doc);
    let groups: Vec<_> = result
        .fault
        .iter()
        .flat_map(|f| &f.groups)
        .filter(|g| g.status != Status::Complies)
        .cloned()
        .collect();
    crate::widgets::section(
        ui,
        p,
        &format!("Por resolver · {}", items.len() + groups.len()),
    );
    if items.is_empty() && groups.is_empty() {
        ui.label(
            RichText::new(format!(
                "{} Nada pendiente: todo lo que verifica IEEStudio cumple.",
                icon::CHECK_CIRCLE
            ))
            .color(p.ok),
        );
    }
    if shared.review.shown == 0 {
        shared.review.shown = FIRST;
    }
    let shown = shared.review.shown.min(items.len());
    let mut action: Option<(Selection, bool)> = None;
    let mut nonlinear: Option<(Selection, bool)> = None;
    for item in items.iter().take(shown) {
        let picked = doc.selection == item.selection;
        if let Some(a) = incident(ui, p, item, picked, class, &mut nonlinear) {
            action = Some(a);
        }
        ui.add_space(theme::SP2);
    }
    if shown < items.len() {
        let more = (items.len() - shown).min(FIRST);
        if ui
            .button(format!(
                "Mostrar {more} más (quedan {})",
                items.len() - shown
            ))
            .clicked()
        {
            shared.review.shown += FIRST;
        }
        ui.add_space(theme::SP2);
    }
    // La capacidad interruptiva: una tarjeta por lugar, con el dato que la
    // resuelve ahí mismo.
    for group in &groups {
        match interrupting_card(ui, p, doc, group) {
            Some(CardAction::Changed) => doc.edited(),
            Some(CardAction::Go(i)) => go = pick(doc, Selection::Feeder(i), class),
            None => {}
        }
        ui.add_space(theme::SP2);
    }
    if let Some((selection, value)) = nonlinear {
        doc.selection = selection;
        if let Some(ov) = doc.selected_overrides_mut() {
            ov.nonlinear = Some(value);
            doc.edited();
        }
    }
    if let Some((selection, fix)) = action {
        if fix {
            doc.selection = selection;
            if let Some(ov) = doc.selected_overrides_mut() {
                ov.size = None;
                doc.edited();
            }
        } else {
            go = pick(doc, selection, class);
        }
    }

    if !result.verdict.out_of_scope.is_empty() {
        crate::widgets::section(ui, p, "Fuera de alcance · se resuelve con otro estudio");
        for o in &result.verdict.out_of_scope {
            ui.label(RichText::new(format!("{} {}", icon::INFO, o.label())));
            ui.label(
                muted(p, sentence(&o.resolve_with) + " Se anota en la memoria.")
                    .size(theme::SECONDARY),
            );
        }
    }
    // Los del sistema eléctrico ya tienen su tarjeta arriba.
    let warnings: Vec<&String> = result
        .warnings
        .iter()
        .filter(|w| !is_system_warning(w))
        .collect();
    if !warnings.is_empty() {
        crate::widgets::section(ui, p, "Avisos");
        for warning in warnings {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{} {warning}", icon::WARNING)).color(p.warn),
                    )
                    .wrap(),
                );
                // El aviso de desbalance se arregla aquí mismo.
                if warning.contains("desbalance") {
                    let name = warning.split(':').next().unwrap_or_default();
                    if let Some(i) = doc.project.panels.iter().position(|x| x.name == name) {
                        if ui.button("Balancear fases").clicked() {
                            shared.done = Some(doc.balance_phases(i));
                        }
                    }
                }
            });
        }
    }
    ui.add_space(theme::SP4);
    panel_summary(ui, p, &result);
    ui.add_space(theme::SP6);
    go
}

/// Los tres niveles del dictamen, en tarjetas: en un renglón si caben; si
/// no, una debajo de otra.
fn verdict_cards(ui: &mut egui::Ui, p: &Palette, result: &ieestudio_project::ProjectResult) {
    let lines = result.verdict.lines("NOM-001-SEDE-2012");
    let gap = theme::SP3;
    let per_row = layout::cards_per_row(ui.available_width(), lines.len(), gap);
    // El ancho interior descuenta el margen (12 + 12) y el borde (1 + 1), y
    // nunca es negativo.
    let inner = layout::remaining(
        layout::column_width(ui.available_width(), per_row, gap),
        26.0,
    );
    let card = |ui: &mut egui::Ui, k: usize, level: &str, status: Status, detail: &str| {
        let color = if k == 2 {
            p.ink2
        } else {
            status_color(p, status)
        };
        let (head, rest) = result.verdict.split_line(detail);
        egui::Frame::new()
            .fill(p.raise)
            .stroke(egui::Stroke::new(
                1.0,
                if status == Status::Fails {
                    p.bad.gamma_multiply(0.6)
                } else {
                    p.line
                },
            ))
            .corner_radius(theme::RADIUS_CARD)
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(inner);
                ui.vertical(|ui| {
                    label_caps(ui, p, level);
                    // La aprobación profesional nunca la da el programa: anillo
                    // vacío, sin color de estado.
                    let glyph = if k == 2 {
                        icon::SEAL_QUESTION
                    } else {
                        status_icon(status)
                    };
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("{glyph} {head}"))
                                .color(color)
                                .size(theme::HEAD)
                                .family(semibold()),
                        )
                        .wrap(),
                    );
                    ui.add(
                        egui::Label::new(muted(p, sentence(&rest)).size(theme::SECONDARY)).wrap(),
                    );
                });
            });
    };
    if per_row > 1 {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (k, (level, status, detail)) in lines.iter().enumerate() {
                card(ui, k, level, *status, detail);
            }
        });
    } else {
        for (k, (level, status, detail)) in lines.iter().enumerate() {
            if k > 0 {
                ui.add_space(gap);
            }
            card(ui, k, level, *status, detail);
        }
    }
}

/// Cuatro contadores en un solo recuadro con divisores: en un renglón, o en
/// dos columnas en el teléfono.
fn counters(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &Doc,
    result: &ieestudio_project::ProjectResult,
    class: Class,
) {
    let (fails, pending) = problem_counts(doc);
    let (done, all) = runs_calculated(doc);
    let tiles = [
        (
            "No cumplen",
            fails.to_string(),
            if fails > 0 { p.bad } else { p.ok },
        ),
        (
            "Sin evaluar",
            pending.to_string(),
            if pending > 0 { p.warn } else { p.ok },
        ),
        (
            "Tramos calculados",
            format!("{done} de {all}"),
            if done == all { p.ink } else { p.warn },
        ),
        (
            "Fuera de alcance",
            result.verdict.out_of_scope.len().to_string(),
            p.ink,
        ),
    ];
    let columns = if class == Class::Phone { 2 } else { 4 };
    egui::Frame::new()
        .fill(p.raise)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(theme::RADIUS_CARD)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let width = layout::column_width(ui.available_width(), columns, 0.0);
            for (r, row) in tiles.chunks(columns).enumerate() {
                if r > 0 {
                    let x = ui.max_rect().x_range();
                    ui.painter()
                        .hline(x, ui.cursor().top(), egui::Stroke::new(1.0, p.hair));
                }
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for (k, (label, figure, color)) in row.iter().enumerate() {
                        let response = ui
                            .allocate_ui_with_layout(
                                egui::vec2(width, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    egui::Frame::new()
                                        .inner_margin(egui::Margin::symmetric(12, 8))
                                        .show(ui, |ui| {
                                            ui.set_width((width - theme::SP6).max(0.0));
                                            ui.label(muted(p, *label).size(theme::FOOT));
                                            ui.label(
                                                RichText::new(figure.as_str())
                                                    .monospace()
                                                    .size(if class == Class::Phone {
                                                        theme::TOUCH_FIGURE
                                                    } else {
                                                        theme::TITLE
                                                    })
                                                    .color(*color),
                                            );
                                        });
                                },
                            )
                            .response;
                        if k > 0 {
                            ui.painter().vline(
                                response.rect.left(),
                                response.rect.y_range(),
                                egui::Stroke::new(1.0, p.hair),
                            );
                        }
                    }
                });
            }
        });
}

/// Una incidencia: el tramo, la verificación, su detalle con la norma y lo
/// que la arregla. Devuelve `(selección, arreglar)` si se pidió algo.
fn incident(
    ui: &mut egui::Ui,
    p: &Palette,
    item: &Pending,
    picked: bool,
    class: Class,
    nonlinear: &mut Option<(Selection, bool)>,
) -> Option<(Selection, bool)> {
    let mut action = None;
    let color = status_color(p, item.check.status);
    let stroke = if picked {
        p.accent
    } else if item.check.status == Status::Fails {
        p.bad.gamma_multiply(0.5)
    } else {
        p.line
    };
    egui::Frame::new()
        .fill(p.raise)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(theme::RADIUS_CARD)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(layout::remaining(ui.available_width(), 2.0));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(status_icon(item.check.status))
                        .color(color)
                        .size(theme::HEAD),
                );
                ui.add(
                    egui::Label::new(strong(format!(
                        "{}: {}",
                        item.title,
                        item.check.name.to_lowercase()
                    )))
                    .wrap(),
                );
            });
            let mut detail = sentence(&item.check.detail);
            if let Some(cite) = cite_all(&item.check.references) {
                detail.push_str(&format!(" · {cite}"));
            }
            ui.add(egui::Label::new(muted(p, detail).size(theme::SECONDARY)).wrap());
            ui.add_space(theme::SP1);
            ui.horizontal_wrapped(|ui| {
                if let Some(rec) = &item.proposed {
                    // Uno de tantos arreglos de la lista: sin amarillo, que es
                    // de la acción principal de cada vista.
                    if item.check.status == Status::Fails
                        && ui
                            .button(format!("Usar {}", size_label(rec)))
                            .on_hover_text("Que IEEStudio vuelva a proponer el calibre")
                            .clicked()
                    {
                        action = Some((item.selection, true));
                    }
                }
                // 310-15(b)(5)(3): el dato se contesta aquí.
                if item
                    .check
                    .detail
                    .starts_with(ieestudio_project::MISSING_NONLINEAR)
                {
                    let hover = "Se guarda en el tramo y se puede deshacer con Ctrl+Z.";
                    if ui.button("Carga lineal").on_hover_text(hover).clicked() {
                        *nonlinear = Some((item.selection, false));
                    }
                    if ui
                        .button("Mayormente no lineal")
                        .on_hover_text(hover)
                        .clicked()
                    {
                        *nonlinear = Some((item.selection, true));
                    }
                }
                let label = match (class, picked) {
                    (Class::Phone, _) => "Abrir ficha",
                    (_, true) => "En la ficha",
                    _ => "Ver ficha",
                };
                if ui
                    .add_enabled(class == Class::Phone || !picked, egui::Button::new(label))
                    .clicked()
                {
                    action = Some((item.selection, false));
                }
            });
        });
    action
}

/// Lo que se calculó en cada tablero, plegado: carga, corriente, caída
/// máxima y falla.
fn panel_summary(ui: &mut egui::Ui, p: &Palette, result: &ieestudio_project::ProjectResult) {
    egui::CollapsingHeader::new(strong("Resumen por tablero"))
        .id_salt("revisar-resumen")
        .show(ui, |ui| {
            for panel in &result.panels {
                crate::widgets::section(ui, p, &format!("Tablero {}", panel.id));
                let mut row = |label: &str, value: String, color, hover: Option<String>| {
                    let response = crate::widgets::property(ui, p, label, |ui| {
                        ui.label(RichText::new(value).monospace().color(color));
                    });
                    if let Some(hover) = hover {
                        response.on_hover_text(hover);
                    }
                };
                row(
                    "Carga calculada",
                    match panel.load_unknown {
                        None => format!("{} VA", thousands(panel.demanded_va)),
                        Some(_) => "sin conocer".to_string(),
                    },
                    p.ink,
                    None,
                );
                if let Some(run) = &panel.feeder.run {
                    row("Corriente", format!("{:.2} A", run.nominal_a), p.ink, None);
                }
                let max = panel
                    .circuits
                    .iter()
                    .filter_map(|c| c.run.as_ref())
                    .map(|r| r.cumulative_drop_percent)
                    .fold(0.0, f64::max);
                row("Caída máxima", format!("{max:.2} %"), p.ink, None);
                if let Some(f) = &panel.fault {
                    // Con los alimentadores descontados manda el método punto
                    // a punto (decisión 0031).
                    let label = match (&f.origin, f.line_to_neutral.is_some(), f.feeders.is_some())
                    {
                        (_, true, true) => "Falla (punto a punto, F-N)",
                        (_, false, true) => "Falla (punto a punto)",
                        (ieestudio_project::fault::FaultOrigin::Supplier, _, _) => {
                            "Falla (suministradora)"
                        }
                        (ieestudio_project::fault::FaultOrigin::SupplierPrimary, _, _) => {
                            "Falla (dato en MT)"
                        }
                        (_, true, false) => "Falla (bus infinito, F-N)",
                        (_, false, false) => "Falla (bus infinito)",
                    };
                    let mut hover = if f.motor_ka > 0.0 {
                        format!(
                            "{:.2} kA {} más {:.3} kA de aportación de motores",
                            f.base_ka,
                            f.basis(),
                            f.motor_ka
                        )
                    } else {
                        format!(
                            "{:.2} kA {}; ningún motor aporta a estas barras",
                            f.base_ka,
                            f.basis()
                        )
                    };
                    if !f.incomplete.is_empty() {
                        hover.push_str(&format!("\nNo incluye: {}", f.incomplete.join("; ")));
                    }
                    let incomplete = !f.incomplete.is_empty();
                    row(
                        label,
                        if incomplete {
                            format!("{:.2} kA · incompleta", f.total_ka)
                        } else {
                            format!("{:.2} kA", f.total_ka)
                        },
                        if incomplete { p.warn } else { p.ink },
                        Some(hover),
                    );
                }
                crate::widgets::end_group(ui);
            }
            if let Some(g) = &result.grounding_electrode {
                ui.add_space(theme::SP2);
                ui.label(
                    muted(
                        p,
                        format!(
                            "Conductor al electrodo: {} de cobre (Tabla 250-66)",
                            size_label(&g.size)
                        ),
                    )
                    .size(theme::SECONDARY),
                );
            }
        });
}

/// «No concluyente» si hay interruptores con dato que no alcanzan la falla
/// estimada; «Sin evaluar» si solo faltan datos.
fn interrupting_label(group: &ieestudio_project::fault::InterruptingGroup) -> &'static str {
    group.label()
}

enum CardAction {
    Changed,
    Go(usize),
}

/// Aviso en Revisar: un transformador intermedio que puede ser el de la
/// acometida capturado dos veces. No bloquea nada: «Son equipos distintos» lo
/// confirma y el aviso ya no sale.
fn duplicate_review(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) -> Option<CardAction> {
    let flags = doc
        .result
        .as_ref()
        .map(|r| r.duplicate_flags.clone())
        .unwrap_or_default();
    if flags.is_empty() {
        return None;
    }
    let mut action = None;
    ui.add_space(crate::theme::SP6);
    label_caps(
        ui,
        p,
        &format!("¿Transformador capturado dos veces? · {}", flags.len()),
    );
    for flag in &flags {
        egui::Frame::new()
            .fill(p.warn.gamma_multiply(0.06))
            .stroke(egui::Stroke::new(1.0, p.warn.gamma_multiply(0.6)))
            .corner_radius(crate::theme::RADIUS_CARD)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(layout::remaining(ui.available_width(), 2.0));
                ui.add(egui::Label::new(RichText::new(format!("{} {}.", icon::WARNING, flag.text())).size(crate::theme::ICON)).wrap());
                ui.horizontal(|ui| {
                    if ui
                        .button("Son equipos distintos")
                        .on_hover_text("El aviso deja de salir; se guarda con el proyecto y se puede quitar en la ficha del tablero.")
                        .clicked()
                    {
                        if let Some(t) = doc.project.panels[flag.panel].transformer.as_mut() {
                            t.confirmed_distinct = true;
                            action = Some(CardAction::Changed);
                        }
                    }
                    if ui.button("Ir al tablero").clicked() {
                        action = Some(CardAction::Go(flag.panel));
                    }
                });
            });
        ui.add_space(crate::theme::SP2);
    }
    action
}

/// Tarjeta de Revisar para la capacidad interruptiva de un lugar: qué pasa,
/// el dato que la resuelve y los interruptores que abarca.
fn interrupting_card(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &mut Doc,
    group: &ieestudio_project::fault::InterruptingGroup,
) -> Option<CardAction> {
    let mut action = None;
    let index_of = |doc: &Doc, id: &str| doc.project.panels.iter().position(|x| x.id == id);
    egui::Frame::new()
        .fill(p.panel)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(crate::theme::RADIUS_CARD)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(layout::remaining(ui.available_width(), 2.0));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icon::CIRCLE_DASHED)
                        .color(p.warn)
                        .size(crate::theme::ICON_BAR),
                );
                ui.add(
                    egui::Label::new(strong(format!(
                        "Capacidad interruptiva en {}: {}",
                        group.place,
                        interrupting_label(group).to_lowercase()
                    )))
                    .wrap(),
                );
            });
            // El título ya dice «no concluyente»; el detalle no lo repite.
            let summary = group.summary();
            let mut detail = sentence(summary.strip_prefix("no concluyente: ").unwrap_or(&summary));
            if let Some(cite) = cite_all(&group.references) {
                detail.push_str(&format!(" · {cite}"));
            }
            ui.add(egui::Label::new(muted(p, detail).size(crate::theme::SECONDARY)).wrap());
            ui.add_space(crate::theme::SP1);
            // El dato de la suministradora sin decir dónde: se resuelve aquí
            // con un clic, sin suponer nada (decisión 0029, D-2).
            let missing_location = group
                .issue
                .as_deref()
                .is_some_and(|i| i.starts_with(ieestudio_project::fault::MISSING_LOCATION));
            if missing_location {
                let ka = doc.project.source.fault.available_ka.unwrap_or_default();
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("¿Dónde da la suministradora los {ka} kA?"));
                    for location in ieestudio_project::FaultLocation::ALL {
                        if ui.button(location.label()).clicked() {
                            doc.project.source.fault.location = Some(location);
                            action = Some(CardAction::Changed);
                        }
                    }
                });
                ui.add_space(2.0);
            }
            match group.panel.as_deref().and_then(|id| index_of(doc, id)) {
                // En las barras de un tablero: el dato del tablero lo toman
                // todos sus interruptores sin dato propio.
                Some(i) => {
                    ui.horizontal(|ui| {
                        ui.label("Capacidad interruptiva del tablero");
                        if crate::widgets::optional_number(
                            ui,
                            &mut doc.project.panels[i].board.interrupting_ka,
                            10.0,
                            " kA",
                        ) {
                            action = Some(CardAction::Changed);
                        }
                        if ui.button("Ir al tablero").clicked() {
                            action = Some(CardAction::Go(i));
                        }
                    });
                }
                // En el punto de conexión: el interruptor principal de cada
                // tablero que sale de la acometida.
                None => {
                    for breaker in &group.breakers {
                        let Some(i) = index_of(doc, &breaker.panel) else {
                            continue;
                        };
                        ui.horizontal(|ui| {
                            ui.label(format!("Interruptor principal de {}", breaker.panel));
                            if crate::widgets::optional_number(
                                ui,
                                &mut doc.project.panels[i]
                                    .feeder
                                    .overrides
                                    .breaker
                                    .interrupting_ka,
                                10.0,
                                " kA",
                            ) {
                                action = Some(CardAction::Changed);
                            }
                            if ui.button("Ir al tablero").clicked() {
                                action = Some(CardAction::Go(i));
                            }
                        });
                    }
                }
            }
            crate::widgets::trace_ui(ui, p, &group.place, &group.trace);
            if group.breakers.len() > 1 {
                egui::CollapsingHeader::new(
                    muted(p, format!("Los {} interruptores", group.breakers.len()))
                        .size(crate::theme::SECONDARY),
                )
                .id_salt(("interruptores", &group.place))
                .show(ui, |ui| {
                    for breaker in &group.breakers {
                        let ka = breaker
                            .ka
                            .map_or("sin dato".to_string(), |ka| format!("{ka} kA"));
                        ui.horizontal(|ui| {
                            ui.label(crate::widgets::status_glyph(p, breaker.status));
                            ui.label(
                                RichText::new(format!("{} — {ka}", breaker.label))
                                    .size(crate::theme::ICON),
                            );
                        });
                    }
                });
            }
        });
    action
}

/// Un pendiente del proyecto: el tramo, la verificación y a dónde ir.
#[derive(Clone)]
struct Pending {
    selection: Selection,
    title: String,
    check: Check,
    proposed: Option<ieestudio_core::Size>,
}

fn review_items(doc: &Doc) -> Vec<Pending> {
    let mut items = Vec::new();
    let Ok(result) = &doc.result else {
        return items;
    };
    for (i, panel) in doc.project.panels.iter().enumerate() {
        let Some(r) = result.panels.iter().find(|x| x.id == panel.id) else {
            continue;
        };
        let mut runs: Vec<(Selection, String, &RunResult)> = vec![(
            Selection::Feeder(i),
            format!("Alimentador de {}", panel.id),
            &r.feeder,
        )];
        if let Some(s) = &r.secondary {
            runs.push((
                Selection::Secondary(i),
                format!("Secundario de {}", panel.id),
                s,
            ));
        }
        for (c, item) in r.circuits.iter().enumerate() {
            runs.push((
                Selection::Circuit(i, c),
                format!("{} · {}", panel.circuits[c].id, panel.circuits[c].name),
                item,
            ));
        }
        for (selection, title, item) in runs {
            let proposed = proposed_instead(item);
            // Primero lo que no cumple, después lo que falta.
            let mut checks: Vec<&Check> = item
                .checks
                .iter()
                .filter(|k| k.status != Status::Complies)
                .collect();
            checks.sort_by_key(|k| k.status != Status::Fails);
            // Un tramo sin calcular es un solo pendiente: su motivo.
            if item.run.is_none() {
                checks.truncate(1);
            }
            for check in checks {
                items.push(Pending {
                    selection,
                    title: title.clone(),
                    check: check.clone(),
                    proposed: proposed.clone(),
                });
            }
        }
    }
    items.sort_by_key(|x| x.check.status != Status::Fails);
    items
}

/// Circuitos de «Otra carga» cuyo nombre parece de un motor o de un aire
/// acondicionado, con el tipo que sugiere (lo detecta el modelo).
fn suspicious_kinds(doc: &Doc) -> Vec<(usize, usize, LoadKind)> {
    ieestudio_project::review::kind_hints(&doc.project)
        .into_iter()
        .map(|h| (h.panel, h.circuit, h.suggested))
        .collect()
}

/// Aviso en Revisar: tramos con una conexión que su sistema o su tablero no
/// tienen (por ejemplo, tres fases en un servicio monofásico de tres hilos).
/// Devuelve `true` si se pidió corregirlos.
fn connection_review(ui: &mut egui::Ui, p: &Palette, doc: &Doc) -> bool {
    use ieestudio_project::connection::{bus_system, feeder_system, mismatches};
    let items = mismatches(&doc.project);
    if items.is_empty() {
        return false;
    }
    let mut fix = false;
    ui.add_space(crate::theme::SP6);
    label_caps(
        ui,
        p,
        &format!("Conexiones que no existen · {}", items.len()),
    );
    ui.add_space(crate::theme::SP1);
    egui::Frame::new()
        .fill(p.panel)
        .stroke(egui::Stroke::new(1.0, p.bad.gamma_multiply(0.6)))
        .corner_radius(crate::theme::RADIUS_CARD)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(layout::remaining(ui.available_width(), 2.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon::X_CIRCLE).color(p.bad).size(crate::theme::ICON_BAR));
                ui.add(
                    egui::Label::new(strong(if items.len() == 1 {
                        "Un tramo pide una conexión que aquí no hay".to_string()
                    } else {
                        format!("{} tramos piden una conexión que aquí no hay", items.len())
                    }))
                    .wrap(),
                );
            });
            ui.add(
                egui::Label::new(
                    muted(
                        p,
                        "El suministro o el tablero de donde salen no tiene esas fases, así que no se calculan. Pasa cuando se cambia el suministro o la conexión de un tablero después de capturar.",
                    )
                    .size(crate::theme::SECONDARY),
                )
                .wrap(),
            );
            ui.add_space(2.0);
            for m in items.iter().take(8) {
                let panel = &doc.project.panels[m.panel];
                let (what, system) = match m.circuit {
                    None => (
                        format!("Alimentador de {}", panel.id),
                        feeder_system(&doc.project, m.panel),
                    ),
                    Some(c) => (
                        format!("{} · {}", panel.circuits[c].id, panel.circuits[c].name),
                        bus_system(&doc.project, m.panel),
                    ),
                };
                let to = m.nearest.map_or("sin conexión posible".to_string(), |k| {
                    format!("pasaría a {}", crate::widgets::connection_label(k, system))
                });
                ui.label(
                    RichText::new(format!(
                        "{what}: {} → {to}",
                        label_of(&KINDS, m.kind)
                    ))
                    .size(crate::theme::SECONDARY),
                );
            }
            if items.len() > 8 {
                ui.label(muted(p, format!("… y {} más", items.len() - 8)).size(crate::theme::SECONDARY));
            }
            ui.add_space(crate::theme::SP1);
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Corregir todas").color(p.on_accent),
                    )
                    .fill(p.accent),
                )
                .on_hover_text("Pasa cada tramo a la conexión más cercana que sí existe y acomoda sus fases. Se puede deshacer con Ctrl+Z.")
                .clicked()
            {
                fix = true;
            }
        });
    fix
}

/// Aviso en Revisar: un circuito que parece motor o aire acondicionado pero
/// se capturó como «Otra carga». Devuelve lo que se pidió: cambiarlo de tipo
/// o confirmar que es otra carga, que se guarda en el proyecto.
/// Aviso de un sistema que no se puede usar: lo muestra `system_review`.
fn is_system_warning(warning: &str) -> bool {
    warning.starts_with("Sistema de suministro: ")
        || warning.contains(", secundario del transformador: ")
}

/// Sistemas que no se pueden usar tal como están (decisión 0030): el de
/// suministro (`None`) y el secundario de cada transformador (`Some(i)`).
/// Devuelve la topología que se declaró con un botón.
fn system_review(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &Doc,
) -> Option<(Option<usize>, ieestudio_core::Topology)> {
    let tolerance = crate::panels::properties::ratio_tolerance(&doc.project.profile);
    let mut systems = vec![(
        None,
        "Sistema de suministro".to_string(),
        doc.project.source.system,
    )];
    for (i, panel) in doc.project.panels.iter().enumerate() {
        if let Some(t) = &panel.transformer {
            systems.push((
                Some(i),
                format!("Secundario del transformador de {}", panel.id),
                t.secondary,
            ));
        }
    }
    let items: Vec<_> = systems
        .into_iter()
        .filter_map(|(at, place, system)| Some((at, place, system, system.check(tolerance).err()?)))
        .collect();
    if items.is_empty() {
        return None;
    }
    let mut action = None;
    ui.add_space(crate::theme::SP6);
    label_caps(ui, p, &format!("Sistema eléctrico · {}", items.len()));
    ui.add_space(crate::theme::SP1);
    for (at, place, system, issue) in items {
        let color = if issue.invalid { p.bad } else { p.warn };
        egui::Frame::new()
            .fill(p.panel)
            .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.5)))
            .corner_radius(crate::theme::RADIUS_CARD)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(layout::remaining(ui.available_width(), 2.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new(if issue.invalid { icon::WARNING } else { icon::QUESTION }).color(color).size(crate::theme::ICON_BAR));
                    ui.add(
                        egui::Label::new(strong(format!(
                            "{place} · {}: {}",
                            system.voltages_label(),
                            if issue.invalid { "dato inválido" } else { "topología sin declarar" }
                        )))
                        .wrap(),
                    );
                });
                let consequence = if issue.invalid {
                    "La capacidad interruptiva que se calcula con un transformador de este sistema no se concluye hasta corregir las tensiones o la topología."
                } else {
                    "Sin ella no se sabe cuántas fases tiene ni si el neutro es portador: los circuitos de tres fases, el neutro de los de dos fases y la capacidad interruptiva quedan sin evaluar. No se deduce de las tensiones."
                };
                ui.add(
                    egui::Label::new(
                        muted(p, format!("{} {consequence}", sentence(&issue.reason))).size(crate::theme::SECONDARY),
                    )
                    .wrap(),
                );
                if system.topology.is_none() {
                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        for topology in system.candidate_topologies() {
                            if ui
                                .button(topology.label())
                                .on_hover_text("Se guarda en el proyecto y se puede deshacer con Ctrl+Z. Las tensiones no cambian: si no corresponden a esta topología, queda como dato inválido.")
                                .clicked()
                            {
                                action = Some((at, topology));
                            }
                        }
                    });
                }
            });
        ui.add_space(crate::theme::SP2);
    }
    action
}

enum KindAction {
    Change(usize, usize, LoadKind),
    Confirm(usize, usize),
}

fn kind_review(ui: &mut egui::Ui, p: &Palette, doc: &Doc) -> Option<KindAction> {
    let items = suspicious_kinds(doc);
    if items.is_empty() {
        return None;
    }
    let profile = Profile::by_id(&doc.project.profile).unwrap_or(Profile::nom_001_sede_2012());
    let mut action = None;
    ui.add_space(crate::theme::SP6);
    label_caps(
        ui,
        p,
        &format!("¿Tipo de carga correcto? · {}", items.len()),
    );
    ui.add_space(crate::theme::SP1);
    for (i, c, kind) in items {
        let circuit = &doc.project.panels[i].circuits[c];
        let (what, rules, missing) = match kind {
            LoadKind::Motor => (
                "motor",
                [
                    Rule::MotorFullLoadCurrent,
                    Rule::MotorBranchConductor,
                    Rule::MotorBranchProtection,
                ],
                "la corriente de tabla, el conductor al 125 % ni la protección del motor",
            ),
            _ => (
                "aire acondicionado",
                [
                    Rule::CompressorRating,
                    Rule::CompressorConductor,
                    Rule::EquipmentNameplate,
                ],
                "los datos de placa, el conductor al 125 % ni la protección máxima del equipo",
            ),
        };
        let cite = rules
            .iter()
            .map(|r| profile.clause(*r))
            .collect::<Vec<_>>()
            .join(" · ");
        egui::Frame::new()
            .fill(p.panel)
            .stroke(egui::Stroke::new(1.0, p.warn.gamma_multiply(0.5)))
            .corner_radius(crate::theme::RADIUS_CARD)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(layout::remaining(ui.available_width(), 2.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon::QUESTION).color(p.warn).size(crate::theme::ICON_BAR));
                    ui.add(
                        egui::Label::new(strong(format!(
                            "{} «{}» está como «Otra carga» ({})",
                            circuit.id,
                            circuit.name,
                            circuit
                                .load
                                .power_w
                                .map_or_else(|| "sin potencia".to_string(), |w| format!("{} W", thousands(w)))
                        )))
                        .wrap(),
                    );
                });
                ui.add(
                    egui::Label::new(
                        muted(
                            p,
                            format!(
                                "Su nombre parece de {}. Como carga general no se le aplican {missing}, y mientras no se confirme, la capacidad interruptiva de su zona queda no concluyente porque faltaría su aportación a la falla. NOM-001-SEDE-2012, {cite}.",
                                if kind == LoadKind::Motor { "un motor" } else { "un equipo de aire acondicionado" },
                            ),
                        )
                        .size(crate::theme::SECONDARY),
                    )
                    .wrap(),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("Cambiar a {what}")).color(p.on_accent),
                            )
                            .fill(p.accent),
                        )
                        .on_hover_text("Cambia el tipo y te lleva a su ficha para capturar los datos del equipo. Se puede deshacer con Ctrl+Z.")
                        .clicked()
                    {
                        action = Some(KindAction::Change(i, c, kind));
                    }
                    if ui
                        .button("Es otra carga")
                        .on_hover_text("Se guarda en el proyecto: no aporta a la falla y deja de avisar. Se puede deshacer con Ctrl+Z o en la ficha del circuito.")
                        .clicked()
                    {
                        action = Some(KindAction::Confirm(i, c));
                    }
                });
            });
        ui.add_space(crate::theme::SP2);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El número de pendientes es uno solo: lo que marca la barra de pasos y
    /// la memoria (del dictamen) es lo que lista Revisar.
    #[test]
    fn review_lists_exactly_what_the_step_bar_counts() {
        let listed = |doc: &Doc| {
            let groups = doc
                .result
                .as_ref()
                .unwrap()
                .fault
                .iter()
                .flat_map(|f| &f.groups)
                .filter(|g| g.status != Status::Complies)
                .count();
            review_items(doc).len() + groups
        };
        let counted = |doc: &Doc| {
            let (fails, pending) = problem_counts(doc);
            fails + pending
        };
        // La casa de ejemplo, tal cual.
        let house = ieestudio_project::example::house();
        let doc = Doc::new(house.clone(), None);
        assert_eq!(listed(&doc), counted(&doc));
        // Con estudio de falla y kA que no alcanzan: un pendiente por lugar.
        let mut project = house.clone();
        project.source.substation = Some(ieestudio_project::Substation {
            impedance_from_table: Default::default(),
            kva: Some(112.5),
            impedance_percent: Some(3.0),
            primary_kv: Some(13.2),
            connection: None,
            mounting: None,
            brand: String::new(),
        });
        // 1 kA no alcanza ni lo que llega a las barras con el alimentador
        // descontado (decisión 0031).
        project.panels[0].board.interrupting_ka = Some(1.0);
        let with_fault = Doc::new(project, None);
        assert_eq!(listed(&with_fault), counted(&with_fault));
        assert_eq!(counted(&with_fault), counted(&doc) + 2);
        // Con un dato inválido (0 kA de la suministradora): también cuenta
        // (decisión 0029, D-6).
        let mut project = house.clone();
        project.source.fault.available_ka = Some(0.0);
        project.source.fault.location = Some(ieestudio_project::FaultLocation::LowVoltage);
        let invalid = Doc::new(project, None);
        assert_eq!(listed(&invalid), counted(&invalid));
        assert!(counted(&invalid) > counted(&doc));
        // Con tramos que no se pudieron calcular: cada uno cuenta una vez.
        let mut project = house;
        project.source.system = ieestudio_core::System::SPLIT_240_120;
        let broken = Doc::new(project, None);
        assert_eq!(listed(&broken), counted(&broken));
    }

    #[test]
    fn thousands_groups_digits() {
        assert_eq!(thousands(0.0), "0");
        assert_eq!(thousands(1800.0), "1,800");
        assert_eq!(thousands(7704.4), "7,704");
        assert_eq!(thousands(1_234_567.0), "1,234,567");
    }

    #[test]
    fn only_general_loads_are_questioned() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        // El ejemplo ya tiene la bomba como motor y el aire con su placa.
        assert!(suspicious_kinds(&doc).is_empty());
        let c = doc.project.panels[0]
            .circuits
            .iter()
            .position(|x| x.load.kind == LoadKind::Motor)
            .unwrap();
        doc.project.panels[0].circuits[c].load.kind = LoadKind::General;
        assert_eq!(suspicious_kinds(&doc), vec![(0, c, LoadKind::Motor)]);
        // «Es otra carga» se guarda en el proyecto y deja de preguntar.
        doc.project.panels[0].circuits[c].load.confirmed_general = true;
        assert!(suspicious_kinds(&doc).is_empty());
    }

    /// En la casa de ejemplo, Revisar lista primero lo que no cumple (C-04)
    /// y ofrece volver al calibre propuesto.
    #[test]
    fn review_lists_failures_first_with_their_fix() {
        let doc = Doc::new(ieestudio_project::example::house(), None);
        let items = review_items(&doc);
        assert!(!items.is_empty());
        assert_eq!(items[0].check.status, Status::Fails);
        assert!(items[0].title.starts_with("C-04"), "{}", items[0].title);
        assert!(items[0].proposed.is_some());
        assert!(items.iter().any(|x| x.check.status == Status::NotEvaluated));
    }

    /// Las incidencias se arman una vez por revisión del proyecto: mientras
    /// no cambie, cada cuadro reutiliza la misma lista.
    #[test]
    fn incidents_are_built_once_per_change() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        let mut cache = Cache::default();
        let first = cache.items(&doc);
        assert!(Rc::ptr_eq(&first, &cache.items(&doc)));
        doc.project.panels[0].circuits[0].length_m = Some(90.0);
        doc.edited();
        let after = cache.items(&doc);
        assert!(!Rc::ptr_eq(&first, &after));
        assert!(after.len() >= first.len());
    }

    /// En el teléfono, elegir una incidencia lleva a su ficha en «Circuitos»;
    /// en escritorio y tableta se queda en Revisar con la ficha al lado.
    #[test]
    fn picking_an_incident_depends_on_the_size() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        let target = Selection::Circuit(0, 3);
        assert_eq!(pick(&mut doc, target, Class::Desktop), None);
        assert_eq!(doc.selection, target);
        assert_eq!(pick(&mut doc, Selection::Feeder(0), Class::Tablet), None);
        assert_eq!(pick(&mut doc, target, Class::Phone), Some(View::Schedule));
        assert_eq!(doc.selection, target);
    }

    #[test]
    fn every_run_of_the_example_is_calculated() {
        let doc = Doc::new(ieestudio_project::example::house(), None);
        // Un alimentador y seis circuitos.
        assert_eq!(runs_calculated(&doc), (7, 7));
    }
}
