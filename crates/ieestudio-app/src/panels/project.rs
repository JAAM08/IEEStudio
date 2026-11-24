//! Vista Proyecto: el resumen de la instalación, la tabla de tableros y los
//! cuatro bloques de captura de la obra (datos de la obra, tipo de inmueble,
//! suministro y acometida, y criterios de diseño).

use eframe::egui::{self, RichText};
use egui_phosphor::regular as icon;
use ieestudio_project::fault::IssueKind;
use ieestudio_rules::Profile;

use super::{properties, Shared};
use crate::layout;
use crate::state::Doc;
use crate::theme::{Palette, RADIUS_CARD};
use crate::widgets::{
    capitalize, centered, choice, label_caps, label_of, muted, page_title, strong, text_field,
    thousands, worst, SYSTEMS,
};

/// Tipos de inmueble que se ofrecen como tarjetas; los demás, en la lista.
const COMMON_OCCUPANCIES: [&str; 3] = ["dwellings", "offices", "stores"];

/// Ancho máximo del contenido: se lee como un formulario, no a todo lo ancho.
const MAX_WIDTH: f32 = 980.0;

/// Dibuja la vista. Devuelve el tablero que se pidió abrir en «Tableros y
/// circuitos».
pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) -> Option<usize> {
    let p = shared.palette;
    let mut open = None;
    let mut changed = false;
    egui::ScrollArea::vertical()
        .id_salt("vista-proyecto")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(crate::theme::SP6);
            centered(ui, MAX_WIDTH, |ui| {
                page_title(
                    ui,
                    &p,
                    "Proyecto",
                    "Cómo va la instalación y los datos generales de la obra. Lo que no sepas todavía lo puedes llenar después.",
                );
                summary(ui, &p, doc);
                ui.add_space(crate::theme::SP4);
                open = panels_table(ui, &p, doc);
                ui.add_space(crate::theme::SP6);
                changed |= work_block(ui, &p, doc);
                ui.add_space(crate::theme::SP4);
                changed |= occupancy_block(ui, &p, doc);
                ui.add_space(crate::theme::SP4);
                changed |= supply_block(ui, &p, doc);
                ui.add_space(crate::theme::SP4);
                changed |= criteria_block(ui, &p, doc);
                ui.add_space(crate::theme::SP8);
            });
        });
    if changed {
        doc.edited();
    }
    open
}

/// Recuadro de un bloque de captura, con su título y una línea que dice para
/// qué sirve.
fn block<R>(
    ui: &mut egui::Ui,
    p: &Palette,
    title: &str,
    hint: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    // Sin recuadro propio: un divisor discreto arriba y el título. Los campos
    // de adentro ya van en grupos; otro recuadro sería un contenedor de más.
    ui.separator();
    ui.add_space(crate::theme::SP2);
    ui.label(strong(title).size(crate::theme::HEAD));
    if !hint.is_empty() {
        ui.label(muted(p, hint).size(crate::theme::SECONDARY));
    }
    ui.add_space(crate::theme::SP2);
    let inner = add(ui);
    crate::widgets::end_group(ui);
    inner
}

// ------------------------------------------------------------------ resumen

/// Cuatro cifras: la verificación, los tramos calculados, la carga de la
/// instalación y la falla en el punto de conexión.
fn summary(ui: &mut egui::Ui, p: &Palette, doc: &Doc) {
    // Un solo recuadro con las cuatro cifras separadas por divisores, no
    // cuatro tarjetas.
    let tiles = summary_tiles(p, doc);
    let pad = crate::theme::SP3;
    egui::Frame::new()
        .fill(p.raise)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(RADIUS_CARD)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            let per_row = layout::cards_per_row(ui.available_width(), tiles.len(), 0.0);
            let width = layout::column_width(ui.available_width(), per_row, 0.0);
            for (r, row) in tiles.chunks(per_row).enumerate() {
                if r > 0 {
                    let x = ui.max_rect().x_range();
                    ui.painter()
                        .hline(x, ui.cursor().top(), egui::Stroke::new(1.0, p.hair));
                }
                ui.horizontal_top(|ui| {
                    for (k, tile) in row.iter().enumerate() {
                        let cell = ui.vertical(|ui| {
                            ui.set_width(width);
                            egui::Frame::new()
                                .inner_margin(egui::Margin::symmetric(16, 12))
                                .show(ui, |ui| {
                                    ui.set_width(layout::remaining(width, 2.0 * pad + 8.0));
                                    tile_body(ui, p, tile);
                                });
                        });
                        if k > 0 {
                            let rect = cell.response.rect;
                            ui.painter().vline(
                                rect.left(),
                                rect.y_range(),
                                egui::Stroke::new(1.0, p.hair),
                            );
                        }
                    }
                });
            }
        });
    ui.add_space(crate::theme::SP4);
}

fn tile_body(ui: &mut egui::Ui, p: &Palette, tile: &Tile) {
    ui.vertical(|ui| {
        ui.label(muted(p, &tile.label).size(crate::theme::FOOT));
        ui.add(
            egui::Label::new(
                RichText::new(&tile.value)
                    .size(crate::theme::TITLE)
                    .color(tile.color),
            )
            .wrap(),
        );
        ui.add(egui::Label::new(muted(p, &tile.detail).size(crate::theme::SECONDARY)).wrap());
    });
}

struct Tile {
    label: String,
    value: String,
    detail: String,
    color: egui::Color32,
}

fn summary_tiles(p: &Palette, doc: &Doc) -> Vec<Tile> {
    let Ok(r) = &doc.result else {
        return vec![Tile {
            label: "Cálculo".into(),
            value: "No se pudo calcular".into(),
            detail: doc.calc_error().unwrap_or_default(),
            color: p.bad,
        }];
    };
    use ieestudio_project::verdict::Compliance;
    let v = &r.verdict;
    let (verdict, color) = match v.compliance {
        Compliance::Conforms => ("Conforme en lo evaluado", p.ok),
        Compliance::DoesNotConform => ("No conforme en lo evaluado", p.bad),
        Compliance::InvalidData => ("Datos inválidos", p.bad),
        Compliance::Incomplete => ("Evaluación incompleta", p.warn),
    };
    let runs: usize = r.panels.iter().map(|pr| pr.runs().count()).sum();
    let calculated = runs.saturating_sub(v.uncalculated);
    // La carga de la instalación: la de los tableros que salen de la
    // acometida. Si alguno no tiene carga conocida, la suma tampoco (0033).
    let roots: Vec<&ieestudio_project::PanelResult> = doc
        .project
        .panels
        .iter()
        .zip(&r.panels)
        .filter(|(panel, _)| panel.parent.is_none())
        .map(|(_, pr)| pr)
        .collect();
    let unknown = roots.iter().find_map(|pr| pr.load_unknown.clone());
    let load = match &unknown {
        None => format!(
            "{} VA",
            thousands(roots.iter().map(|pr| pr.demanded_va).sum())
        ),
        Some(_) => "Sin conocer".to_string(),
    };
    let load_detail = match unknown {
        None => format!(
            "Calculada según el artículo 220, de {} {}",
            roots.len(),
            if roots.len() == 1 {
                "tablero que sale de la acometida"
            } else {
                "tableros que salen de la acometida"
            }
        ),
        Some(why) => capitalize(&why),
    };
    let (fault, fault_detail, fault_color) = if !doc.project.settings.fault_study {
        (
            "Fuera de alcance".to_string(),
            "La verificación de capacidad interruptiva está apagada".to_string(),
            p.ink2,
        )
    } else {
        match r.fault.as_ref().map(|f| (&f.source, &f.source_issue)) {
            Some((Some(bus), _)) => (
                format!("{:.2} kA", bus.total_ka),
                "Falla estimada en el punto de conexión".to_string(),
                p.ink,
            ),
            Some((None, Some(issue))) if issue.kind == IssueKind::Invalid => (
                "Dato inválido".to_string(),
                capitalize(&issue.reason),
                p.bad,
            ),
            Some((None, Some(issue))) => {
                ("Sin estimar".to_string(), capitalize(&issue.reason), p.warn)
            }
            _ => (
                "Sin datos".to_string(),
                "Sin datos de la acometida: lo que depende de ella queda fuera de alcance"
                    .to_string(),
                p.warn,
            ),
        }
    };
    vec![
        Tile {
            label: "Verificación".into(),
            value: verdict.into(),
            detail: format!(
                "{} no cumplen · {} sin evaluar",
                v.failed_checks, v.pending_checks
            ),
            color,
        },
        Tile {
            label: "Tramos calculados".into(),
            value: format!("{calculated} de {runs}"),
            detail: if v.uncalculated == 0 {
                "Todos los tramos tienen resultado".into()
            } else {
                format!("{} sin calcular: falta un dato", v.uncalculated)
            },
            color: if v.uncalculated == 0 { p.ink } else { p.warn },
        },
        Tile {
            label: "Carga de la instalación".into(),
            value: load,
            detail: load_detail,
            color: p.ink,
        },
        Tile {
            label: "Falla en el punto de conexión".into(),
            value: fault,
            detail: fault_detail,
            color: fault_color,
        },
    ]
}

// ------------------------------------------------------------ tableros

/// Tabla de tableros con lo esencial de cada uno. Devuelve el que se pidió
/// abrir.
fn panels_table(ui: &mut egui::Ui, p: &Palette, doc: &Doc) -> Option<usize> {
    let mut open = None;
    label_caps(ui, p, "Tableros");
    ui.add_space(crate::theme::SP1);
    if doc.project.panels.is_empty() {
        ui.label(muted(
            p,
            "Todavía no hay tableros. Agrégalos en «Tableros y circuitos».",
        ));
        return None;
    }
    // En el teléfono los tableros son una lista que se toca, no una tabla.
    if ui.available_width() < crate::layout::PHONE {
        return crate::compact::list(ui, p, &crate::compact::panel_rows(doc));
    }
    egui::Frame::new()
        .fill(p.panel)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(RADIUS_CARD)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::horizontal()
                .id_salt("tabla-tableros")
                .show(ui, |ui| {
                    egui::Grid::new("tableros-proyecto")
                        .num_columns(7)
                        .spacing([18.0, 8.0])
                        .striped(false)
                        .show(ui, |ui| {
                            for title in [
                                "Tablero",
                                "Se alimenta de",
                                "Circuitos",
                                "Carga calculada",
                                "Alimentador",
                                "Estado",
                                "",
                            ] {
                                ui.label(
                                    RichText::new(title).size(crate::theme::FOOT).color(p.ink3),
                                );
                            }
                            ui.end_row();
                            for (i, panel) in doc.project.panels.iter().enumerate() {
                                let result = doc.panel_result(i);
                                ui.label(strong(format!("{} · {}", panel.id, panel.name)));
                                ui.label(muted(
                                    p,
                                    panel
                                        .parent
                                        .clone()
                                        .unwrap_or_else(|| doc.project.source.name.clone()),
                                ));
                                ui.label(panel.circuits.len().to_string());
                                ui.label(
                                    RichText::new(match result {
                                        Some(r) if r.load_unknown.is_none() => {
                                            format!("{} VA", thousands(r.demanded_va))
                                        }
                                        Some(_) => "sin conocer".to_string(),
                                        None => "—".to_string(),
                                    })
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(
                                        result.and_then(|r| r.feeder.run.as_ref()).map_or(
                                            "—".to_string(),
                                            |run| {
                                                format!(
                                                    "{:.1} A · {}",
                                                    run.nominal_a,
                                                    ieestudio_project::size_label(&run.size)
                                                )
                                            },
                                        ),
                                    )
                                    .monospace(),
                                );
                                match result {
                                    Some(r) => {
                                        let status = worst(r.runs().map(|x| x.status));
                                        crate::widgets::status_chip(ui, p, status);
                                    }
                                    None => {
                                        ui.label(muted(p, "—"));
                                    }
                                }
                                if ui
                                    .button(format!("Abrir {}", icon::ARROW_RIGHT))
                                    .on_hover_text("Ver su cuadro de cargas y su ficha")
                                    .clicked()
                                {
                                    open = Some(i);
                                }
                                ui.end_row();
                            }
                        });
                });
        });
    open
}

// ------------------------------------------------------------- bloques

fn work_block(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) -> bool {
    let mut changed = false;
    let project = &mut doc.project;
    block(
        ui,
        p,
        "Datos de la obra",
        "Aparecen en la portada de la memoria.",
        |ui| {
            let gap = 16.0;
            let two = layout::cards_per_row(ui.available_width(), 2, gap) == 2
                && ui.available_width() >= 2.0 * 220.0 + gap;
            let half = if two {
                layout::column_width(ui.available_width(), 2, gap)
            } else {
                ui.available_width()
            };
            let fields: [(&str, &mut String, &str); 4] = [
                (
                    "Nombre del proyecto",
                    &mut project.name,
                    "Casa, local, nave…",
                ),
                (
                    "Cliente",
                    &mut project.info.client,
                    "Propietario o razón social",
                ),
                (
                    "Ubicación",
                    &mut project.info.location,
                    "Domicilio o municipio",
                ),
                (
                    "Quién lo elabora",
                    &mut project.info.prepared_by,
                    "Tu nombre",
                ),
            ];
            let mut fields = fields.into_iter();
            loop {
                let first = fields.next();
                let second = if two { fields.next() } else { None };
                let Some((label, value, hint)) = first else {
                    break;
                };
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.vertical(|ui| {
                        ui.set_width(half);
                        changed |= text_field(ui, p, label, value, hint);
                    });
                    if let Some((label, value, hint)) = second {
                        ui.vertical(|ui| {
                            ui.set_width(half);
                            changed |= text_field(ui, p, label, value, hint);
                        });
                    }
                });
                ui.add_space(crate::theme::SP2);
            }
        },
    );
    changed
}

fn occupancy_block(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) -> bool {
    let mut changed = false;
    let project = &mut doc.project;
    block(
        ui,
        p,
        "Tipo de inmueble",
        "Decide la carga mínima por m² y los factores de demanda (Tabla 220-12).",
        |ui| {
            let profile = Profile::nom_001_sede_2012();
            let gap = 12.0;
            let per_row = layout::cards_per_row(ui.available_width(), 4, gap);
            let card = layout::column_width(ui.available_width(), per_row, gap);
            let current = project.occupancy.clone();
            let mut picks: Vec<Option<String>> = Vec::new();
            let mut other_value = None;
            let mut other_changed = false;
            // Las tarjetas y «Otro tipo» se reparten en filas según el ancho.
            let ids: Vec<&str> = COMMON_OCCUPANCIES
                .iter()
                .copied()
                .filter(|id| profile.occupancy(id).is_some())
                .collect();
            let mut items: Vec<Option<&str>> = ids.iter().map(|id| Some(*id)).collect();
            items.push(None);
            for row in items.chunks(per_row) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for item in row {
                        match item {
                            Some(id) => {
                                let Some(o) = profile.occupancy(id) else {
                                    continue;
                                };
                                let on = current.as_deref() == Some(*id);
                                if choice(
                                    ui,
                                    p,
                                    on,
                                    &o.label,
                                    &format!("{} VA/m²", o.va_per_m2),
                                    card,
                                ) && !on
                                {
                                    picks.push(Some(id.to_string()));
                                }
                            }
                            None => {
                                ui.vertical(|ui| {
                                    ui.set_width(card);
                                    ui.add_space(crate::theme::SP1);
                                    ui.label(muted(p, "Otro tipo").size(crate::theme::SECONDARY));
                                    let other = current
                                        .as_deref()
                                        .is_some_and(|o| !COMMON_OCCUPANCIES.contains(&o));
                                    let mut value = if other { current.clone() } else { None };
                                    if properties::occupancy_combo(
                                        ui,
                                        "inmueble-otro",
                                        &mut value,
                                        false,
                                        "Elegir de la tabla",
                                    ) {
                                        other_changed = true;
                                        other_value = value;
                                    }
                                });
                            }
                        }
                    }
                });
                ui.add_space(gap);
            }
            if let Some(Some(id)) = picks.pop() {
                project.occupancy = Some(id);
                changed = true;
            }
            if other_changed {
                if other_value.is_some() {
                    project.occupancy = other_value;
                }
                changed = true;
            }
            if project.occupancy.is_none() {
                ui.label(
                    RichText::new(format!(
                        "{} Sin el tipo de inmueble, los alimentadores con alumbrado o contactos quedan sin evaluar.",
                        icon::INFO
                    ))
                    .color(p.warn)
                    .size(crate::theme::ICON),
                );
            }
        },
    );
    changed
}

fn supply_block(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) -> bool {
    let mut changed = false;
    let study = doc.result.as_ref().ok().and_then(|r| r.fault.clone());
    let project = &mut doc.project;
    block(
        ui,
        p,
        "Suministro y acometida",
        "Sistema de la compañía suministradora, datos de la acometida y de su transformador.",
        |ui| {
            let gap = 12.0;
            let per_row = layout::cards_per_row(ui.available_width(), 3, gap);
            let card = layout::column_width(ui.available_width(), per_row, gap);
            for row in SYSTEMS.chunks(per_row) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for (label, system) in row {
                        let (phases, volts) = label.split_once(' ').unwrap_or((label, ""));
                        let on = project.source.system == *system;
                        let plain = match phases {
                            "1F-2H" => "una fase y neutro",
                            "1F-3H" => "monofásico de tres hilos",
                            "3F-4H" => "trifásico con neutro",
                            _ => "trifásico sin neutro",
                        };
                        let detail = format!("{phases} · {plain}");
                        if choice(ui, p, on, volts, &detail, card) && !on {
                            project.source.system = *system;
                            changed = true;
                        }
                    }
                });
                ui.add_space(gap);
            }
            // Un sistema que no es de la lista (archivo anterior o
            // importado) declara aquí su topología (decisión 0030).
            if !SYSTEMS.iter().any(|(_, s)| *s == project.source.system) {
                let tolerance = properties::ratio_tolerance(&project.profile);
                ui.label(strong(format!(
                    "Sistema del archivo: {}",
                    project.source.system.voltages_label()
                )));
                changed |= crate::widgets::topology_row(
                    ui,
                    p,
                    "topologia-proyecto",
                    &mut project.source.system,
                    tolerance,
                );
                ui.add_space(crate::theme::SP3);
            }
            // Cambiar el suministro con tableros ya capturados puede dejar
            // conexiones que el sistema nuevo no tiene.
            let stale = ieestudio_project::connection::mismatches(project).len();
            if stale > 0 {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} {} con fases que este suministro no tiene: no se calculan hasta ajustarlos.",
                            icon::WARNING,
                            if stale == 1 {
                                "Hay 1 tramo capturado".to_string()
                            } else {
                                format!("Hay {stale} tramos capturados")
                            }
                        ))
                        .color(p.bad)
                        .size(crate::theme::SECONDARY),
                    );
                    if ui
                        .button("Ajustarlos")
                        .on_hover_text("Pasa cada tramo a la conexión más cercana que sí existe. Se puede deshacer con Ctrl+Z.")
                        .clicked()
                    {
                        ieestudio_project::connection::fix(project);
                        changed = true;
                    }
                });
                ui.add_space(crate::theme::SP3);
            }

            ui.separator();
            changed |= properties::fault_study_switch(ui, p, &mut project.settings.fault_study);
            ui.add_space(crate::theme::SP2);
            // Con algo por resolver en la falla, la sección se abre sola: ahí
            // está el dato que falta.
            let attention = project.settings.fault_study
                && matches!(
                    study.as_ref().map(|f| (&f.source, &f.source_issue)),
                    Some((None, Some(_)))
                );
            let mut summary = Vec::new();
            let source = &project.source;
            if !source.supplier.trim().is_empty() {
                summary.push(source.supplier.trim().to_string());
            }
            if let Some(sub) = &source.substation {
                summary.push(match sub.kva {
                    Some(kva) => format!("transformador de la acometida de {kva} kVA"),
                    None => "transformador de la acometida sin potencia".to_string(),
                });
            }
            if !summary.is_empty() {
                ui.label(muted(p, capitalize(&summary.join(" · "))).size(crate::theme::SECONDARY));
            }
            egui::CollapsingHeader::new("Datos de la acometida, su transformador y la falla")
                .id_salt(("acometida-proyecto", attention))
                .default_open(attention)
                .show(ui, |ui| {
                    changed |=
                        properties::source_inputs(ui, p, &mut project.source, study.as_ref());
                });
        },
    );
    changed
}

fn criteria_block(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) -> bool {
    let mut changed = false;
    let project = &mut doc.project;
    let s = &project.settings;
    let summary = format!(
        "Caída {} % en derivados, {} % en alimentadores y {} % acumulada · {} °C · {} {} · {}",
        s.branch_drop_limit_percent,
        s.feeder_drop_limit_percent,
        s.total_drop_limit_percent,
        s.ambient_c,
        label_of(&crate::widgets::MATERIALS, s.material).to_lowercase(),
        label_of(&crate::widgets::INSULATIONS, s.insulation),
        label_of(&crate::widgets::CONDUITS, s.conduit),
    );
    block(ui, p, "Criterios de diseño", &summary, |ui| {
        egui::CollapsingHeader::new("Cambiar criterios")
            .id_salt("criterios-proyecto")
            .show(ui, |ui| {
                changed |= properties::criteria_inputs(ui, p, &mut project.settings);
            });
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiles(doc: &Doc) -> Vec<(String, String)> {
        summary_tiles(&Palette::new(true), doc)
            .into_iter()
            .map(|t| (t.label, t.value))
            .collect()
    }

    #[test]
    fn the_summary_says_how_the_installation_is_doing() {
        let doc = Doc::new(ieestudio_project::example::house(), None);
        let r = doc.result.as_ref().unwrap();
        let t = tiles(&doc);
        let labels: Vec<&str> = t.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Verificación",
                "Tramos calculados",
                "Carga de la instalación",
                "Falla en el punto de conexión"
            ]
        );
        let runs: usize = r.panels.iter().map(|p| p.runs().count()).sum();
        assert_eq!(t[1].1, format!("{runs} de {runs}"));
        // La carga es la de los tableros que salen de la acometida.
        let roots: f64 = doc
            .project
            .panels
            .iter()
            .zip(&r.panels)
            .filter(|(p, _)| p.parent.is_none())
            .map(|(_, pr)| pr.demanded_va)
            .sum();
        assert_eq!(t[2].1, format!("{} VA", thousands(roots)));
    }

    // Una suma parcial no es la carga de la instalación (decisión 0033).
    #[test]
    fn a_missing_power_leaves_the_installation_load_unknown() {
        let mut project = ieestudio_project::example::house();
        project.panels[0].circuits[0].load.power_w = None;
        let doc = Doc::new(project, None);
        assert_eq!(tiles(&doc)[2].1, "Sin conocer");
    }

    #[test]
    fn with_the_check_turned_off_the_fault_is_out_of_scope() {
        let mut project = ieestudio_project::example::house();
        project.settings.fault_study = false;
        let doc = Doc::new(project, None);
        assert_eq!(tiles(&doc)[3].1, "Fuera de alcance");
    }
}
