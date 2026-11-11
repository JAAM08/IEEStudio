//! Vista guiada: el proyecto en cinco pasos, Proyecto → Tableros →
//! Circuitos → Revisar → Memoria, en lugar de paneles sueltos.
//!
//! Cada paso muestra solo lo que toca en ese momento y reutiliza los mismos
//! formularios, el mismo cálculo y la misma memoria que la vista avanzada
//! (decisión 0020). Los pasos no obligan un orden: se salta a cualquiera.

use eframe::egui::{self, Align, Layout, RichText, Sense, Vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use ieestudio_project::{size_label, Check, LoadKind, RunResult, Status};
use ieestudio_rules::{cite_all, Profile, Rule};
use serde::{Deserialize, Serialize};

use crate::panels::{properties, report, Shared};
use crate::state::{set_load_kind, Doc, Selection};
use crate::symbols::{self, Symbol};
use crate::theme::{semibold, Palette};
use crate::widgets::{
    label_of, muted, status_color, status_icon, status_text, steps_ui, strong, worst, KINDS,
    SYSTEMS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Step {
    Project,
    Panels,
    #[default]
    Circuits,
    Review,
    Report,
}

impl Step {
    pub const ALL: [Step; 5] = [
        Step::Project,
        Step::Panels,
        Step::Circuits,
        Step::Review,
        Step::Report,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Step::Project => "Proyecto",
            Step::Panels => "Tableros",
            Step::Circuits => "Circuitos",
            Step::Review => "Revisar",
            Step::Report => "Memoria",
        }
    }

    fn index(self) -> usize {
        Step::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    fn previous(self) -> Option<Step> {
        self.index().checked_sub(1).map(|i| Step::ALL[i])
    }

    fn next(self) -> Option<Step> {
        Step::ALL.get(self.index() + 1).copied()
    }
}

/// Archivo que un paso pide exportar a la ventana principal.
pub enum Export {
    Pdf,
    Excel,
    Dxf,
}

// ------------------------------------------------------------------ estado

/// Verificaciones que no cumplen y que quedaron sin evaluar en todo el
/// proyecto, contando las de los tramos que no se pudieron calcular.
fn problem_counts(doc: &Doc) -> (usize, usize) {
    let Ok(result) = &doc.result else {
        return (0, 0);
    };
    let v = &result.verdict;
    (v.failed_checks, v.pending_checks + v.uncalculated)
}

/// Si el paso ya tiene lo mínimo, para marcarlo con ✓.
fn step_done(step: Step, doc: &Doc) -> bool {
    let project = &doc.project;
    match step {
        Step::Project => !project.name.trim().is_empty() && project.occupancy.is_some(),
        Step::Panels => !project.panels.is_empty(),
        Step::Circuits => project.panels.iter().any(|p| !p.circuits.is_empty()),
        Step::Review => {
            doc.result.is_ok() && problem_counts(doc) == (0, 0) && step_done(Step::Circuits, doc)
        }
        Step::Report => false,
    }
}

/// Barra de pasos para la barra superior. Devuelve el paso elegido.
pub fn step_bar(ui: &mut egui::Ui, p: &Palette, current: Step, doc: &Doc) -> Option<Step> {
    let mut chosen = None;
    let (fails, pending) = problem_counts(doc);
    for (k, step) in Step::ALL.into_iter().enumerate() {
        let now = step == current;
        let done = !now && step_done(step, doc);
        let (bubble_fill, bubble_ink, bubble_text) = if now {
            (p.accent, p.on_accent, format!("{}", k + 1))
        } else if done {
            (p.raise, p.ok, icon::CHECK.to_string())
        } else {
            (p.bg, p.ink2, format!("{}", k + 1))
        };
        let text_color = if now { p.ink } else { p.ink2 };
        let response = egui::Frame::new()
            .fill(if now {
                p.raise
            } else {
                egui::Color32::TRANSPARENT
            })
            .corner_radius(14)
            .inner_margin(egui::Margin {
                left: 6,
                right: 12,
                top: 3,
                bottom: 3,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 7.0;
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
                    ui.painter().circle(
                        rect.center(),
                        10.0,
                        bubble_fill,
                        egui::Stroke::new(1.0, if now { p.accent } else { p.line }),
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        bubble_text,
                        egui::FontId::proportional(11.5),
                        bubble_ink,
                    );
                    let label = RichText::new(step.title()).color(text_color);
                    ui.label(if now { label.family(semibold()) } else { label });
                    // Revisar avisa cuánto falta, desde cualquier paso.
                    if step == Step::Review && fails + pending > 0 {
                        let (n, color) = if fails > 0 {
                            (fails, p.bad)
                        } else {
                            (pending, p.warn)
                        };
                        ui.label(RichText::new(format!("· {n}")).color(color).size(12.0));
                    }
                });
            })
            .response
            .interact(Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() && !now {
            chosen = Some(step);
        }
    }
    chosen
}

// ------------------------------------------------------------------ piezas

fn page_title(ui: &mut egui::Ui, p: &Palette, title: &str, subtitle: &str) {
    ui.label(strong(title).size(24.0));
    ui.add_space(2.0);
    ui.label(muted(p, subtitle));
    ui.add_space(14.0);
}

fn label_caps(ui: &mut egui::Ui, p: &Palette, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(11.0)
            .color(p.ink2)
            .family(semibold()),
    );
}

/// Botón principal, amarillo.
fn primary(ui: &mut egui::Ui, p: &Palette, text: &str) -> bool {
    ui.add(
        egui::Button::new(RichText::new(text).color(p.on_accent).family(semibold()))
            .fill(p.accent)
            .min_size(Vec2::new(0.0, 30.0)),
    )
    .clicked()
}

/// Tarjeta seleccionable: título y una línea debajo.
fn choice(ui: &mut egui::Ui, p: &Palette, on: bool, title: &str, sub: &str, width: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 62.0), Sense::click());
    let hovered = response.hovered();
    ui.painter().rect(
        rect,
        7.0,
        if on || hovered { p.raise } else { p.panel },
        egui::Stroke::new(
            if on { 1.5 } else { 1.0 },
            if on {
                p.accent
            } else if hovered {
                p.ink3
            } else {
                p.line
            },
        ),
        egui::StrokeKind::Inside,
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(14.0, 11.0)))
            .layout(Layout::top_down(Align::Min)),
    );
    child.label(strong(title));
    child.label(muted(p, sub).size(12.0));
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

/// Columna centrada de ancho máximo, para formularios.
fn centered(ui: &mut egui::Ui, max_width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width().min(max_width);
    let left = ((ui.available_width() - width) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(left);
        ui.vertical(|ui| {
            ui.set_width(width);
            add(ui);
        });
    });
}

fn text_field(ui: &mut egui::Ui, p: &Palette, label: &str, value: &mut String, hint: &str) -> bool {
    let mut changed = false;
    ui.vertical(|ui| {
        label_caps(ui, p, label);
        changed = ui
            .add(
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY)
                    .margin(egui::Margin::symmetric(8, 6)),
            )
            .changed();
    });
    changed
}

/// `1,800`: miles con coma, sin decimales.
fn thousands(value: f64) -> String {
    let digits = format!("{:.0}", value.abs());
    let mut out = String::new();
    for (k, ch) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if value < 0.0 {
        format!("-{out}")
    } else {
        out
    }
}

/// Primera verificación que impide que el tramo cumpla: la que no cumple, o
/// si no hay, la que quedó sin evaluar.
fn main_problem(item: &RunResult) -> Option<&Check> {
    item.checks
        .iter()
        .find(|c| c.status == Status::Fails)
        .or_else(|| {
            item.checks
                .iter()
                .find(|c| c.status == Status::NotEvaluated)
        })
}

/// El calibre que propone IEEStudio cuando el usuario fijó otro.
fn proposed_instead(item: &RunResult) -> Option<ieestudio_core::Size> {
    let run = item.run.as_ref()?;
    match &run.recommended_size {
        Some(rec) if run.forced && rec != &run.size => Some(rec.clone()),
        _ => None,
    }
}

fn sentence(text: &str) -> String {
    crate::widgets::capitalize(text.trim_end_matches('.')) + "."
}

// ------------------------------------------------------------------ pasos

/// Dibuja el paso y su pie. Devuelve el paso al que se quiere ir y lo que se
/// pide a la ventana principal.
pub fn show(
    ui: &mut egui::Ui,
    step: Step,
    doc: &mut Doc,
    shared: &mut Shared,
) -> (Option<Step>, Option<Export>) {
    let p = shared.palette;
    let mut go = None;
    let mut request = None;

    egui::Panel::bottom("pie-guiado")
        .frame(
            egui::Frame::new()
                .fill(p.bg)
                .inner_margin(egui::Margin::symmetric(16, 8)),
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(prev) = step.previous() {
                    if ui
                        .add(
                            egui::Button::new(format!("{}  {}", icon::ARROW_LEFT, prev.title()))
                                .frame(false),
                        )
                        .clicked()
                    {
                        go = Some(prev);
                    }
                }
                ui.add_space(8.0);
                footer_summary(ui, &p, step, doc);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    match step.next() {
                        Some(next) => {
                            if primary(
                                ui,
                                &p,
                                &format!("Siguiente: {}  {}", next.title(), icon::ARROW_RIGHT),
                            ) {
                                go = Some(next);
                            }
                        }
                        None => {
                            if primary(
                                ui,
                                &p,
                                &format!("{}  Exportar memoria en PDF", icon::FILE_PDF),
                            ) {
                                request = Some(Export::Pdf);
                            }
                        }
                    }
                });
            });
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(p.bg))
        .show(ui, |ui| match step {
            Step::Project => project_step(ui, &p, doc),
            Step::Panels => {
                if let Some(s) = panels_step(ui, &p, doc) {
                    go = Some(s);
                }
            }
            Step::Circuits => {
                if let Some(s) = circuits_step(ui, &p, doc, shared) {
                    go = Some(s);
                }
            }
            Step::Review => {
                if let Some(s) = review_step(ui, &p, doc, shared) {
                    go = Some(s);
                }
            }
            Step::Report => {
                if let Some(r) = report_step(ui, &p, doc, shared) {
                    request = Some(r);
                }
            }
        });
    (go, request)
}

fn footer_summary(ui: &mut egui::Ui, p: &Palette, step: Step, doc: &Doc) {
    let k = step.index() + 1;
    match step {
        Step::Circuits => {
            let Some(i) = doc.panel_index() else {
                ui.label(muted(p, "Paso 3 de 5"));
                return;
            };
            let Some(result) = doc.panel_result(i) else {
                ui.label(muted(p, "Paso 3 de 5"));
                return;
            };
            let count = |s: Status| result.circuits.iter().filter(|c| c.status == s).count();
            ui.label(muted(p, format!("{} ·", doc.project.panels[i].id)));
            let ok = count(Status::Complies);
            ui.label(
                RichText::new(format!(
                    "{ok} {}",
                    if ok == 1 { "cumple" } else { "cumplen" }
                ))
                .color(p.ok),
            );
            let fails = count(Status::Fails);
            if fails > 0 {
                let verb = if fails == 1 {
                    "no cumple"
                } else {
                    "no cumplen"
                };
                ui.label(RichText::new(format!("· {fails} {verb}")).color(p.bad));
            }
            let pending = count(Status::NotEvaluated);
            if pending > 0 {
                ui.label(RichText::new(format!("· {pending} sin evaluar")).color(p.warn));
            }
        }
        Step::Review => {
            ui.label(muted(
                p,
                "Puedes generar la memoria aunque falten puntos: saldrán anotados como pendientes.",
            ));
        }
        Step::Panels => {
            let n = doc.project.panels.len();
            ui.label(muted(
                p,
                format!(
                    "Paso 2 de 5 · {n} {}",
                    if n == 1 { "tablero" } else { "tableros" }
                ),
            ));
        }
        _ => {
            ui.label(muted(p, format!("Paso {k} de 5")));
        }
    }
}

// ------------------------------------------------------------- 1 · proyecto

/// Tipos de inmueble que se ofrecen como tarjetas; los demás, en la lista.
const COMMON_OCCUPANCIES: [&str; 3] = ["dwellings", "offices", "stores"];

fn project_step(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) {
    let mut changed = false;
    let study = doc.result.as_ref().ok().and_then(|r| r.fault.clone());
    egui::ScrollArea::vertical()
        .id_salt("paso-proyecto")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(28.0);
            centered(ui, 920.0, |ui| {
                page_title(
                    ui,
                    p,
                    "¿Qué vas a calcular?",
                    "Datos generales de la obra. Lo que no sepas todavía lo puedes llenar después.",
                );
                let project = &mut doc.project;
                let half = (ui.available_width() - 16.0) / 2.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.vertical(|ui| {
                        ui.set_width(half);
                        changed |= text_field(ui, p, "Nombre del proyecto", &mut project.name, "Casa, local, nave…");
                    });
                    ui.vertical(|ui| {
                        ui.set_width(half);
                        changed |= text_field(ui, p, "Cliente", &mut project.info.client, "Propietario o razón social");
                    });
                });
                ui.add_space(10.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.vertical(|ui| {
                        ui.set_width(half);
                        changed |= text_field(ui, p, "Ubicación", &mut project.info.location, "Domicilio o municipio");
                    });
                    ui.vertical(|ui| {
                        ui.set_width(half);
                        changed |= text_field(ui, p, "Quién lo elabora", &mut project.info.prepared_by, "Tu nombre");
                    });
                });

                ui.add_space(24.0);
                label_caps(ui, p, "Tipo de inmueble");
                ui.label(muted(p, "Decide la carga mínima por m² y los factores de demanda (Tabla 220-12).").size(12.5));
                ui.add_space(6.0);
                let profile = Profile::nom_001_sede_2012();
                let card = (ui.available_width() - 3.0 * 12.0) / 4.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    for id in COMMON_OCCUPANCIES {
                        if let Some(o) = profile.occupancy(id) {
                            let on = project.occupancy.as_deref() == Some(id);
                            if choice(ui, p, on, &o.label, &format!("{} VA/m²", o.va_per_m2), card) && !on {
                                project.occupancy = Some(id.to_string());
                                changed = true;
                            }
                        }
                    }
                    ui.vertical(|ui| {
                        ui.set_width(card);
                        ui.add_space(4.0);
                        ui.label(muted(p, "Otro tipo").size(12.0));
                        let other = project
                            .occupancy
                            .as_deref()
                            .is_some_and(|o| !COMMON_OCCUPANCIES.contains(&o));
                        let mut value = if other { project.occupancy.clone() } else { None };
                        if properties::occupancy_combo(ui, "inmueble-otro", &mut value, false, "Elegir de la tabla") {
                            if value.is_some() {
                                project.occupancy = value;
                            }
                            changed = true;
                        }
                    });
                });
                if project.occupancy.is_none() {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "{} Sin el tipo de inmueble, los alimentadores con alumbrado o contactos quedan sin evaluar.",
                            icon::INFO
                        ))
                        .color(p.warn)
                        .size(12.5),
                    );
                }

                ui.add_space(24.0);
                label_caps(ui, p, "Suministro");
                ui.add_space(6.0);
                let card = (ui.available_width() - 2.0 * 12.0) / 3.0;
                for row in SYSTEMS.chunks(3) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
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
                    ui.add_space(10.0);
                }
                // Cambiar el suministro con tableros ya capturados puede
                // dejar conexiones que el sistema nuevo no tiene.
                let stale = ieestudio_project::connection::mismatches(project).len();
                if stale > 0 {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} {} con fases que este suministro no tiene: no se calculan hasta ajustarlos.",
                                icon::WARNING,
                                if stale == 1 { "Hay 1 tramo capturado".to_string() } else { format!("Hay {stale} tramos capturados") }
                            ))
                            .color(p.bad)
                            .size(12.5),
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
                    ui.add_space(10.0);
                }

                egui::Frame::new()
                    .fill(p.panel)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(8)
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(strong("Acometida y cortocircuito"));
                        let source = &project.source;
                        let mut summary = Vec::new();
                        if !source.supplier.trim().is_empty() {
                            summary.push(source.supplier.trim().to_string());
                        }
                        if let Some(sub) = &source.substation {
                            summary.push(format!("subestación de {} kVA, Z = {} %", sub.kva, sub.impedance_percent));
                        }
                        match &study {
                            Some(f) => summary.push(format!("falla de {:.2} kA en el punto de conexión", f.source.total_ka)),
                            None => summary.push("sin datos de falla: la capacidad interruptiva queda fuera de alcance".into()),
                        }
                        ui.label(muted(p, crate::widgets::capitalize(&summary.join(" · "))).size(12.5));
                        egui::CollapsingHeader::new("Datos de la acometida, la subestación y la falla")
                            .id_salt("acometida-guiado")
                            .show(ui, |ui| {
                                changed |= properties::source_inputs(ui, p, &mut project.source, study.as_ref());
                            });
                    });

                ui.add_space(14.0);
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
                egui::Frame::new()
                    .fill(p.panel)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(8)
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(strong("Criterios de diseño"));
                        ui.label(muted(p, summary).size(12.5));
                        egui::CollapsingHeader::new("Cambiar criterios")
                            .id_salt("criterios-guiado")
                            .show(ui, |ui| {
                                changed |= properties::criteria_inputs(ui, p, &mut project.settings);
                            });
                    });
                ui.add_space(30.0);
            });
        });
    if changed {
        doc.edited();
    }
}

// ------------------------------------------------------------- 2 · tableros

fn panels_step(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc) -> Option<Step> {
    let mut go = None;
    let selected_panel = match doc.selection {
        Selection::Project => None,
        _ => doc.panel_index(),
    };
    egui::Panel::right("ficha-tablero")
        .resizable(false)
        .exact_size(420.0)
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(20, 18)),
        )
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("ficha-tablero")
                .auto_shrink([false, false])
                .show(ui, |ui| match selected_panel {
                    Some(i) => {
                        label_caps(ui, p, "Ficha del tablero");
                        let panel = &doc.project.panels[i];
                        ui.label(strong(format!("{} · {}", panel.id, panel.name)).size(18.0));
                        ui.horizontal(|ui| {
                            if ui.button(format!("{} Ver sus circuitos", icon::LIST_BULLETS)).clicked() {
                                go = Some(Step::Circuits);
                            }
                            if ui
                                .button(RichText::new(format!("{} Eliminar tablero", icon::TRASH)).color(p.bad))
                                .on_hover_text("Sus subtableros pasan al tablero de donde se alimentaba. Se puede deshacer con Ctrl+Z.")
                                .clicked()
                            {
                                doc.selection = Selection::Feeder(i);
                                doc.remove_selected();
                            }
                        });
                        if i < doc.project.panels.len() && properties::feeder_inputs(ui, doc, p, i) {
                            doc.edited();
                        }
                    }
                    None => {
                        ui.add_space(40.0);
                        ui.label(muted(p, "Elige un tablero del árbol para ver su ficha."));
                    }
                });
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(36, 28)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("arbol-tableros")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    page_title(
                        ui,
                        p,
                        "¿Cómo se reparte la energía?",
                        "De dónde sale cada tablero. Los circuitos se agregan en el siguiente paso.",
                    );
                    let source = &doc.project.source;
                    node(
                        ui,
                        p,
                        false,
                        "Suministro",
                        &source.name,
                        label_of(&SYSTEMS, source.system),
                        None,
                    );
                    let mut action = None;
                    let roots: Vec<usize> = (0..doc.project.panels.len())
                        .filter(|&i| doc.project.panels[i].parent.is_none())
                        .collect();
                    for i in roots {
                        tree(ui, p, doc, i, 1, selected_panel, &mut action);
                    }
                    ui.add_space(10.0);
                    if ui
                        .button(format!("{} Otro tablero desde {}", icon::PLUS, doc.project.source.name))
                        .clicked()
                    {
                        action = Some(TreeAction::Add(None));
                    }
                    ui.add_space(16.0);
                    ui.label(
                        muted(
                            p,
                            "Un tablero alimentado por un transformador se marca en su ficha; el programa calcula primario y secundario con la tensión de cada lado.",
                        )
                        .size(12.5),
                    );
                    match action {
                        Some(TreeAction::Select(i)) => doc.selection = Selection::Feeder(i),
                        Some(TreeAction::Add(parent)) => {
                            doc.selection = match parent {
                                Some(i) => Selection::Feeder(i),
                                None => Selection::Project,
                            };
                            doc.add_panel();
                        }
                        None => {}
                    }
                });
        });
    go
}

enum TreeAction {
    Select(usize),
    /// Agregar un tablero colgado de otro, o de la fuente.
    Add(Option<usize>),
}

/// Recuadro del árbol de tableros.
fn node(
    ui: &mut egui::Ui,
    p: &Palette,
    on: bool,
    kicker: &str,
    title: &str,
    sub: &str,
    status: Option<(String, egui::Color32)>,
) -> egui::Response {
    egui::Frame::new()
        .fill(if on { p.raise } else { p.panel })
        .stroke(egui::Stroke::new(
            if on { 1.5 } else { 1.0 },
            if on { p.accent } else { p.line },
        ))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_min_width(280.0);
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.label(
                    RichText::new(kicker.to_uppercase())
                        .size(10.5)
                        .color(p.ink2)
                        .family(semibold()),
                );
                ui.label(strong(title));
                ui.label(muted(p, sub).size(12.5));
                if let Some((text, color)) = status {
                    ui.label(RichText::new(text).color(color).size(12.5));
                }
            });
        })
        .response
}

fn tree(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &Doc,
    i: usize,
    depth: usize,
    selected: Option<usize>,
    action: &mut Option<TreeAction>,
) {
    let panel = &doc.project.panels[i];
    let indent = 28.0 * depth as f32;
    // Línea que baja del tablero de arriba.
    ui.horizontal(|ui| {
        ui.add_space(indent - 4.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(2.0, 18.0), Sense::hover());
        ui.painter().rect_filled(rect, 0.0, p.ink3);
    });
    let area: f64 = panel.areas.iter().map(|a| a.m2).sum();
    let mut sub = format!(
        "Alimentador de {} m · {}",
        panel.feeder.length_m,
        label_of(&KINDS, panel.kind)
    );
    if area > 0.0 {
        sub.push_str(&format!(" · {area} m²"));
    }
    if let Some(t) = &panel.transformer {
        sub.push_str(&format!(" · transformador de {} kVA", t.kva));
    }
    let status = doc.panel_result(i).map(|r| {
        let statuses: Vec<Status> = r.runs().map(|x| x.status).collect();
        let fails = statuses.iter().filter(|s| **s == Status::Fails).count();
        let n = panel.circuits.len();
        let circuits = format!("{n} {}", if n == 1 { "circuito" } else { "circuitos" });
        match worst(statuses.iter().copied()) {
            Status::Fails => (format!("{circuits} · {fails} no cumple"), p.bad),
            Status::NotEvaluated => (format!("{circuits} · algo sin evaluar"), p.warn),
            Status::Complies => (format!("{circuits} · todo cumple"), p.ok),
        }
    });
    let kicker = match &panel.parent {
        None => "Tablero",
        Some(_) => "Subtablero",
    };
    ui.horizontal(|ui| {
        ui.add_space(indent - 14.0);
        let response = node(
            ui,
            p,
            selected == Some(i),
            kicker,
            &format!("{} · {}", panel.id, panel.name),
            &sub,
            status,
        )
        .interact(Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            *action = Some(TreeAction::Select(i));
        }
        if ui
            .add(
                egui::Button::new(
                    RichText::new(format!("{} Subtablero", icon::PLUS)).color(p.ink2),
                )
                .frame(false),
            )
            .on_hover_text(format!(
                "Agregar un tablero que se alimenta de {}",
                panel.id
            ))
            .clicked()
        {
            *action = Some(TreeAction::Add(Some(i)));
        }
    });
    let children: Vec<usize> = (0..doc.project.panels.len())
        .filter(|&c| doc.project.panels[c].parent.as_deref() == Some(panel.id.as_str()))
        .collect();
    for c in children {
        tree(ui, p, doc, c, depth + 1, selected, action);
    }
}

// ------------------------------------------------------------ 3 · circuitos

fn circuits_step(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &mut Doc,
    shared: &mut Shared,
) -> Option<Step> {
    let mut go = None;
    let Some(panel_index) = doc.panel_index() else {
        crate::widgets::empty_state(
            ui,
            p,
            icon::SQUARES_FOUR,
            "Todavía no hay tableros",
            "Agrega uno en el paso Tableros.",
        );
        return None;
    };
    let selected = match doc.selection {
        Selection::Circuit(i, c) if i == panel_index => Some(c),
        _ => None,
    };

    egui::Panel::right("ficha-circuito")
        .resizable(false)
        .exact_size(390.0)
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(20, 18)),
        )
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("ficha-circuito")
                .auto_shrink([false, false])
                .show(ui, |ui| match selected {
                    Some(c) => circuit_card(ui, p, doc, panel_index, c),
                    None => {
                        ui.add_space(40.0);
                        ui.label(muted(
                            p,
                            "Elige un circuito de la tabla para ver su ficha, o agrega uno nuevo.",
                        ));
                    }
                });
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(24, 18)))
        .show(ui, |ui| {
            // Tableros como pestañas.
            ui.horizontal(|ui| {
                for (i, panel) in doc.project.panels.iter().enumerate() {
                    if ui.selectable_label(i == panel_index, &panel.id).clicked() && i != panel_index {
                        doc.selection = Selection::Feeder(i);
                    }
                }
            });
            ui.add_space(10.0);
            let panel = &doc.project.panels[panel_index];
            let result = doc.panel_result(panel_index).cloned();
            let mut add = false;
            let mut balance = false;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(strong(format!("Circuitos de {}", panel.id)).size(22.0));
                    if let Some(r) = &result {
                        ui.horizontal(|ui| {
                            let mut line = format!("{} VA calculados", thousands(r.demanded_va));
                            if let Some(run) = &r.feeder.run {
                                line.push_str(&format!(" · {:.1} A en el alimentador", run.nominal_a));
                            }
                            if let Some(i) = r.imbalance_percent {
                                line.push_str(&format!(" · desbalance {i:.1} %"));
                            }
                            ui.label(muted(p, line).size(12.5));
                            if r.imbalance_percent.is_some()
                                && ui
                                    .add(egui::Button::new(RichText::new("Balancear").color(p.accent_text).size(12.5)).frame(false))
                                    .on_hover_text("Reparte las fases para dejar el tablero lo más balanceado posible. Se puede deshacer con Ctrl+Z.")
                                    .clicked()
                            {
                                balance = true;
                            }
                        });
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if primary(ui, p, &format!("{}  Agregar circuito", icon::PLUS)) {
                        add = true;
                    }
                });
            });
            ui.add_space(12.0);
            match &result {
                Some(r) => {
                    if let Some(c) = circuit_table(ui, p, doc, panel_index, r, selected) {
                        doc.selection = Selection::Circuit(panel_index, c);
                    }
                }
                None => {
                    ui.label(RichText::new(doc.calc_error().unwrap_or_default()).color(p.bad));
                }
            }
            if let Some(action) = subpanel_list(ui, p, doc, panel_index) {
                match action {
                    SubpanelAction::Circuits(k) => doc.selection = Selection::Feeder(k),
                    SubpanelAction::Card(k) => {
                        doc.selection = Selection::Feeder(k);
                        go = Some(Step::Panels);
                    }
                }
            }
            if add {
                shared.choose_load = Some(panel_index);
            }
            if balance {
                shared.done = doc.balance_phases(panel_index).map(|(before, after)| {
                    format!("Fases balanceadas: desbalance de {before:.1} % a {after:.1} %")
                });
            }
        });
    go
}

enum SubpanelAction {
    /// Ver los circuitos del subtablero.
    Circuits(usize),
    /// Abrir su ficha en el paso Tableros, donde se captura su alimentador.
    Card(usize),
}

/// Subtableros que se alimentan del tablero, debajo de sus circuitos. Se
/// leen de la estructura del proyecto: son los mismos del paso Tableros.
fn subpanel_list(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &Doc,
    panel_index: usize,
) -> Option<SubpanelAction> {
    let children = doc.subpanels(panel_index);
    if children.is_empty() {
        return None;
    }
    let mut action = None;
    ui.add_space(16.0);
    label_caps(
        ui,
        p,
        &format!(
            "Subtableros que alimenta {} · {}",
            doc.project.panels[panel_index].id,
            children.len()
        ),
    );
    ui.add_space(4.0);
    for k in children {
        let panel = &doc.project.panels[k];
        let status = doc.panel_result(k).map(|r| r.feeder.status);
        egui::Frame::new()
            .fill(p.panel)
            .stroke(egui::Stroke::new(1.0, p.line))
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width() - 2.0);
                ui.horizontal(|ui| {
                    symbols::show(ui, Symbol::Subpanel, 22.0, p.ink);
                    ui.add_space(4.0);
                    ui.label(strong(&panel.id));
                    ui.add(egui::Label::new(muted(p, &panel.name)).truncate());
                    let n = panel.circuits.len();
                    ui.label(muted(
                        p,
                        format!(
                            "· alimentador de {} m · {n} {}",
                            panel.feeder.length_m,
                            if n == 1 { "circuito" } else { "circuitos" }
                        ),
                    ));
                    if let Some(status) = status {
                        ui.label(status_text(p, status));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(format!("{} Ficha del tablero", icon::SQUARES_FOUR))
                            .on_hover_text("Su alimentador, carga y transformador se capturan en el paso Tableros.")
                            .clicked()
                        {
                            action = Some(SubpanelAction::Card(k));
                        }
                        if ui
                            .button(format!("{} Ver sus circuitos", icon::LIST_BULLETS))
                            .clicked()
                        {
                            action = Some(SubpanelAction::Circuits(k));
                        }
                    });
                });
            });
        ui.add_space(4.0);
    }
    action
}

/// Tabla de circuitos en dos grupos: lo que se captura y lo que resuelve
/// IEEStudio. Devuelve el circuito en el que se hizo clic.
fn circuit_table(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &Doc,
    panel_index: usize,
    result: &ieestudio_project::PanelResult,
    selected: Option<usize>,
) -> Option<usize> {
    let panel = &doc.project.panels[panel_index];
    let input = [
        ("Tipo", 120.0),
        ("Carga", 78.0),
        ("Fase", 42.0),
        ("Longitud", 66.0),
    ];
    let output = [
        ("Cable", 92.0),
        ("Interruptor", 80.0),
        ("Caída total", 74.0),
        ("Estado", 112.0),
    ];
    const GAP: f32 = 12.0;
    let fixed: f32 = input.iter().chain(&output).map(|(_, w)| w + GAP).sum();
    let name_width = (ui.available_width() - fixed - 40.0).max(150.0);
    let capture_width = name_width + input.iter().map(|(_, w)| w + GAP).sum::<f32>();

    // Encabezado de grupos.
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(8.0);
        let (r1, _) = ui.allocate_exact_size(Vec2::new(capture_width, 22.0), Sense::hover());
        ui.painter().text(
            r1.left_center(),
            egui::Align2::LEFT_CENTER,
            "LO QUE CAPTURAS",
            egui::FontId::new(11.0, semibold()),
            p.ink2,
        );
        let (r2, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), Sense::hover());
        ui.painter().line_segment(
            [r2.left_top(), r2.left_bottom()],
            egui::Stroke::new(1.0, p.line),
        );
        ui.painter().text(
            r2.left_center() + Vec2::new(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "LO QUE RESUELVE IEESTUDIO",
            egui::FontId::new(11.0, semibold()),
            p.accent_text,
        );
    });

    let mut clicked = None;
    egui::ScrollArea::horizontal()
        .id_salt("circuitos-guiado")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = GAP;
            let mut table = TableBuilder::new(ui)
                .id_salt(("circuitos-guiado", panel_index))
                .sense(Sense::click())
                .cell_layout(Layout::left_to_right(Align::Center))
                .resizable(false)
                .auto_shrink([false, true])
                .column(Column::exact(name_width).clip(true));
            for (_, w) in input.iter().chain(&output) {
                table = table.column(Column::exact(*w));
            }
            table
                .header(26.0, |mut header| {
                    header.col(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Circuito").size(11.5).color(p.ink2));
                    });
                    for (title, _) in input.iter().chain(&output) {
                        header.col(|ui| {
                            ui.label(RichText::new(*title).size(11.5).color(p.ink2));
                        });
                    }
                })
                .body(|body| {
                    body.rows(34.0, panel.circuits.len(), |mut row| {
                        let c = row.index();
                        let circuit = &panel.circuits[c];
                        let Some(item) = result.circuits.get(c) else {
                            return;
                        };
                        let run = item.run.as_ref();
                        let is_selected = selected == Some(c);
                        row.set_selected(is_selected);
                        row.col(|ui| {
                            if is_selected {
                                let r = ui.max_rect();
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(
                                        r.left_top(),
                                        Vec2::new(3.0, r.height()),
                                    ),
                                    0.0,
                                    p.accent,
                                );
                            }
                            ui.add_space(8.0);
                            ui.label(strong(&circuit.id));
                            ui.add(egui::Label::new(muted(p, &circuit.name)).truncate());
                        });
                        row.col(|ui| {
                            symbols::show(ui, Symbol::Load(circuit.load.kind), 20.0, p.ink2);
                            ui.add(egui::Label::new(circuit.load.kind.label()).truncate());
                        });
                        row.col(|ui| {
                            let load = match (&circuit.load.motor, circuit.load.kind) {
                                (Some(m), LoadKind::Motor) => {
                                    format!("{} hp", ieestudio_project::hp_label(&m.hp))
                                }
                                _ => format!("{} W", thousands(item.load_w)),
                            };
                            ui.label(RichText::new(load).monospace());
                        });
                        row.col(|ui| {
                            let phases: String =
                                circuit.phases.iter().map(|ph| format!("{ph:?}")).collect();
                            ui.label(if phases.is_empty() {
                                "—".to_string()
                            } else {
                                phases
                            });
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(format!("{} m", circuit.length_m)).monospace());
                        });
                        row.col(|ui| match run {
                            Some(r) => {
                                let size = if r.conductors_per_phase > 1 {
                                    format!("{}×{}", r.conductors_per_phase, size_label(&r.size))
                                } else {
                                    size_label(&r.size)
                                };
                                ui.label(RichText::new(size).monospace());
                                if r.forced {
                                    ui.label(
                                        RichText::new(icon::LOCK_SIMPLE).size(11.0).color(p.ink2),
                                    )
                                    .on_hover_text(
                                        "Fijado por ti: IEEStudio lo verifica, no lo cambia.",
                                    );
                                }
                            }
                            None => {
                                ui.label(muted(p, "—"));
                            }
                        });
                        row.col(|ui| {
                            let text = run.map_or("—".to_string(), |r| {
                                format!("{}P-{:.0} A", r.poles, r.breaker_a)
                            });
                            ui.label(RichText::new(text).monospace());
                        });
                        row.col(|ui| {
                            let fails = item
                                .checks
                                .iter()
                                .any(|k| k.name.starts_with("Caída") && k.status == Status::Fails);
                            let text = run.map_or("—".to_string(), |r| {
                                format!("{:.1} %", r.cumulative_drop_percent)
                            });
                            ui.label(RichText::new(text).monospace().color(if fails {
                                p.bad
                            } else {
                                p.ink
                            }));
                        });
                        row.col(|ui| {
                            ui.label(status_text(p, item.status));
                        });
                        if row.response().clicked() {
                            clicked = Some(c);
                        }
                    });
                });
        });
    if panel.circuits.is_empty() {
        ui.add_space(18.0);
        ui.label(muted(
            p,
            "Este tablero no tiene circuitos. Agrega el primero con el botón de arriba.",
        ));
    }
    clicked
}

/// Ficha del circuito: qué le pasa en palabras simples, cómo arreglarlo, sus
/// resultados, sus datos esenciales y cómo se calculó.
fn circuit_card(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc, i: usize, c: usize) {
    let item = doc.panel_result(i).and_then(|r| r.circuits.get(c)).cloned();
    let circuit = &doc.project.panels[i].circuits[c];
    let title = format!("{} · {}", circuit.id, circuit.name);
    let load_kind = circuit.load.kind;
    let mut duplicate = false;
    let mut remove = false;
    let mut use_proposed = false;
    let procedure_id = egui::Id::new(("procedimiento-guiado", i, c));
    let mut open_procedure: bool = ui.ctx().data(|d| d.get_temp(procedure_id)).unwrap_or(false);

    ui.horizontal(|ui| {
        label_caps(ui, p, "Ficha del circuito");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("{} Eliminar", icon::TRASH)).color(p.bad),
                    )
                    .frame(false),
                )
                .on_hover_text("Se puede deshacer con Ctrl+Z")
                .clicked()
            {
                remove = true;
            }
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("{} Duplicar", icon::COPY)).color(p.ink2),
                    )
                    .frame(false),
                )
                .clicked()
            {
                duplicate = true;
            }
        });
    });
    ui.horizontal(|ui| {
        symbols::show(ui, Symbol::Load(load_kind), 30.0, p.accent_text);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(strong(&title).size(18.0));
            ui.label(muted(p, load_kind.label()).size(12.0));
        });
    });
    ui.add_space(8.0);

    if let Some(item) = &item {
        match main_problem(item) {
            Some(check) => {
                let (color, head) = match check.status {
                    Status::Fails => (p.bad, format!("No cumple: {}", check.name.to_lowercase())),
                    _ => (
                        p.warn,
                        format!("Sin evaluar: {}", check.name.to_lowercase()),
                    ),
                };
                egui::Frame::new()
                    .fill(color.gamma_multiply(0.10))
                    .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.55)))
                    .corner_radius(8)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(head).color(color).family(semibold()));
                        ui.label(sentence(&check.detail));
                        if let Some(cite) = cite_all(&check.references) {
                            ui.label(muted(p, cite).size(11.5));
                        }
                        let others = item
                            .checks
                            .iter()
                            .filter(|k| k.status != Status::Complies)
                            .count();
                        if others > 1 {
                            ui.label(
                                muted(p, format!("Y {} más en Revisar.", others - 1)).size(11.5),
                            );
                        }
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            if let Some(rec) = proposed_instead(item) {
                                if primary(ui, p, &format!("Usar {} (propuesto)", size_label(&rec)))
                                {
                                    use_proposed = true;
                                }
                            }
                            if ui.button("Ver cómo se calculó").clicked() {
                                open_procedure = true;
                            }
                        });
                    });
            }
            None => {
                ui.label(
                    RichText::new(format!(
                        "{} Cumple todo lo que se verificó",
                        icon::CHECK_CIRCLE
                    ))
                    .color(p.ok),
                );
            }
        }
        if let Some(run) = &item.run {
            ui.add_space(10.0);
            egui::Grid::new(("resumen-circuito", i, c))
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    let size = if run.conductors_per_phase > 1 {
                        format!(
                            "{} × {} por fase",
                            run.conductors_per_phase,
                            size_label(&run.size)
                        )
                    } else {
                        size_label(&run.size)
                    };
                    for (label, value) in [
                        ("Cable", size),
                        (
                            "Interruptor",
                            format!("{}P-{:.0} A", run.poles, run.breaker_a),
                        ),
                        ("Tierra", size_label(&run.grounding_size)),
                        (
                            "Tubo",
                            run.conduit.as_ref().map_or("—".to_string(), |t| t.label()),
                        ),
                        (
                            "Caída total",
                            format!("{:.2} %", run.cumulative_drop_percent),
                        ),
                    ] {
                        ui.label(muted(p, label));
                        ui.label(RichText::new(value).monospace());
                        ui.end_row();
                    }
                });
        }
    }

    if let Some(item) = &item {
        equipment_rules(ui, p, load_kind, item);
    }

    ui.add_space(6.0);
    ui.separator();
    if properties::circuit_form(ui, doc, p, i, c, true) {
        doc.edited();
    }

    if let Some(run) = item.as_ref().and_then(|x| x.run.as_ref()) {
        ui.add_space(6.0);
        let header = egui::CollapsingHeader::new(RichText::new("Cómo se calculó").color(p.ink2))
            .id_salt(("como-se-calculo", i, c))
            .open(open_procedure.then_some(true))
            .show(ui, |ui| {
                steps_ui(ui, p, ("pasos-guiado", i, c), &run.steps, None);
            });
        open_procedure = header.openness > 0.5;
    }
    ui.ctx()
        .data_mut(|d| d.insert_temp(procedure_id, open_procedure));

    if use_proposed {
        if let Some(ov) = doc.selected_overrides_mut() {
            ov.size = None;
            doc.edited();
        }
    }
    if duplicate {
        doc.duplicate_circuit();
    }
    if remove {
        doc.remove_selected();
    }
}

/// Para motores y aire acondicionado, las reglas propias del equipo con su
/// resultado: corriente, corriente de diseño e interruptor. Son los primeros
/// pasos del procedimiento, así que dicen lo mismo que «Cómo se calculó».
fn equipment_rules(ui: &mut egui::Ui, p: &Palette, kind: LoadKind, item: &RunResult) {
    let title = match kind {
        LoadKind::Motor => "Por ser motor",
        LoadKind::AirConditioning => "Por ser aire acondicionado",
        _ => return,
    };
    ui.add_space(10.0);
    let frame = egui::Frame::new()
        .fill(p.bg)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(6)
        .inner_margin(egui::Margin {
            left: 14,
            right: 10,
            top: 8,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            label_caps(ui, p, title);
            ui.add_space(2.0);
            match &item.run {
                Some(run) => {
                    for step in run.steps.iter().take(3) {
                        ui.label(RichText::new(&step.title).size(12.5));
                        ui.add(
                            egui::Label::new(RichText::new(&step.substitution).monospace().size(12.0))
                                .wrap(),
                        );
                        if let Some(cite) = cite_all(&step.references) {
                            ui.label(muted(p, cite).size(11.0));
                        }
                        ui.add_space(3.0);
                    }
                }
                None => {
                    let need = match kind {
                        LoadKind::Motor => "Elige la potencia del motor para calcularlo.",
                        _ => "Captura los datos de la placa de la unidad para calcularlo: la corriente nominal (RLA), o la ampacidad mínima (MCA) y la protección máxima (MOP).",
                    };
                    ui.add(egui::Label::new(muted(p, need).size(12.5)).wrap());
                }
            }
        });
    // Filete de acento a la izquierda del recuadro.
    let r = frame.response.rect;
    ui.painter().rect_filled(
        egui::Rect::from_min_size(r.left_top(), Vec2::new(3.0, r.height())),
        egui::CornerRadius {
            nw: 6,
            sw: 6,
            ne: 0,
            se: 0,
        },
        p.accent,
    );
}

// ------------------------------------------------------------- 4 · revisar

/// Un pendiente del proyecto: el tramo, la verificación y a dónde ir.
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

fn review_step(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc, shared: &mut Shared) -> Option<Step> {
    let mut go = None;
    egui::Panel::right("revisar-unifilar")
        .resizable(false)
        .exact_size(440.0)
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(20, 18)),
        )
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("revisar-lateral")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    label_caps(ui, p, "Unifilar");
                    ui.label(muted(p, "Lo que no cumple también se marca aquí. Haz clic en un circuito para ir a él.").size(12.0));
                    ui.add_space(6.0);
                    if let Ok(result) = &doc.result {
                        let diagram = ieestudio_diagrams::single_line(&doc.project, result);
                        // Ajustado al ancho del panel, sin barra de zoom: para
                        // verlo en grande está la vista avanzada.
                        let zoom = ((ui.available_width() - 4.0) / diagram.width as f32).clamp(0.2, 1.0);
                        egui::ScrollArea::vertical()
                            .id_salt("unifilar-guiado")
                            .max_height(360.0)
                            .show(ui, |ui| {
                                if let Some(hit) = crate::panels::diagram::paint(ui, p, &diagram, zoom, false, None) {
                                    doc.selection = match hit {
                                        (i, Some(c)) => Selection::Circuit(i, c),
                                        (i, None) => Selection::Feeder(i),
                                    };
                                    go = Some(match hit.1 {
                                        Some(_) => Step::Circuits,
                                        None => Step::Panels,
                                    });
                                }
                            });
                        ui.add_space(14.0);
                        for panel in &result.panels {
                            label_caps(ui, p, &format!("Tablero {}", panel.id));
                            egui::Grid::new(("resumen-tablero", &panel.id))
                                .num_columns(2)
                                .spacing([16.0, 5.0])
                                .show(ui, |ui| {
                                    ui.label(muted(p, "Carga calculada"));
                                    ui.label(RichText::new(format!("{} VA", thousands(panel.demanded_va))).monospace());
                                    ui.end_row();
                                    if let Some(run) = &panel.feeder.run {
                                        ui.label(muted(p, "Corriente"));
                                        ui.label(RichText::new(format!("{:.2} A", run.nominal_a)).monospace());
                                        ui.end_row();
                                    }
                                    let max = panel
                                        .circuits
                                        .iter()
                                        .filter_map(|c| c.run.as_ref())
                                        .map(|r| r.cumulative_drop_percent)
                                        .fold(0.0, f64::max);
                                    ui.label(muted(p, "Caída máxima"));
                                    ui.label(RichText::new(format!("{max:.2} %")).monospace());
                                    ui.end_row();
                                    if let Some(f) = &panel.fault {
                                        ui.label(muted(p, "Falla (bus infinito)"));
                                        ui.label(RichText::new(format!("{:.2} kA", f.total_ka)).monospace())
                                            .on_hover_text(if f.motor_ka > 0.0 {
                                                format!("{:.2} kA más {:.3} kA de aportación de motores", f.base_ka, f.motor_ka)
                                            } else {
                                                "Sin motores en el proyecto".to_string()
                                            });
                                        ui.end_row();
                                    }
                                });
                            ui.add_space(10.0);
                        }
                        if let Some(g) = &result.grounding_electrode {
                            ui.label(muted(p, format!("Conductor al electrodo: {} de cobre (Tabla 250-66)", size_label(&g.size))).size(12.0));
                        }
                    }
                });
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(32, 24)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("revisar")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    page_title(
                        ui,
                        p,
                        "¿Está listo para entregar?",
                        "Todo lo que falta, en un solo lugar. Cada punto te lleva a donde se arregla.",
                    );
                    let Ok(result) = doc.result.clone() else {
                        ui.label(RichText::new(doc.calc_error().unwrap_or_default()).color(p.bad));
                        return;
                    };
                    // Los tres niveles del veredicto.
                    let lines = result.verdict.lines("NOM-001-SEDE-2012");
                    let width = (ui.available_width() - 2.0 * 12.0) / 3.0 - 4.0;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        for (k, (level, status, detail)) in lines.iter().enumerate() {
                            let color = if k == 2 { p.ink2 } else { status_color(p, *status) };
                            let (head, rest) = detail.split_once(": ").unwrap_or((detail.as_str(), ""));
                            egui::Frame::new()
                                .fill(p.panel)
                                .stroke(egui::Stroke::new(1.0, if *status == Status::Fails { p.bad.gamma_multiply(0.6) } else { p.line }))
                                .corner_radius(8)
                                .inner_margin(egui::Margin::symmetric(14, 12))
                                .show(ui, |ui| {
                                    ui.set_width(width - 30.0);
                                    ui.vertical(|ui| {
                                        label_caps(ui, p, level);
                                        let glyph = if k == 2 { icon::SEAL_QUESTION } else { status_icon(*status) };
                                        ui.add(egui::Label::new(RichText::new(format!("{glyph} {head}")).color(color).size(16.0).family(semibold())).wrap());
                                        ui.add(egui::Label::new(muted(p, sentence(rest)).size(12.0)).wrap());
                                    });
                                });
                        }
                    });

                    // Antes que nada, lo que no se puede calcular: conexiones
                    // que el sistema no tiene.
                    if connection_review(ui, p, doc) {
                        ieestudio_project::connection::fix(&mut doc.project);
                        doc.edited();
                    }

                    // Primero el tipo de carga: cambia todo lo demás del circuito.
                    if let Some((selection, kind)) = kind_review(ui, p, doc) {
                        if let Selection::Circuit(i, c) = selection {
                            set_load_kind(&mut doc.project.panels[i].circuits[c].load, kind);
                            doc.selection = selection;
                            doc.edited();
                            go = Some(Step::Circuits);
                        }
                    }

                    let items = review_items(doc);
                    ui.add_space(20.0);
                    label_caps(ui, p, &format!("Por resolver · {}", items.len()));
                    ui.add_space(4.0);
                    if items.is_empty() {
                        ui.label(RichText::new(format!("{} Nada pendiente: todo lo que verifica IEEStudio cumple.", icon::CHECK_CIRCLE)).color(p.ok));
                    }
                    let mut action: Option<(Selection, bool)> = None;
                    for (k, item) in items.iter().enumerate() {
                        let color = status_color(p, item.check.status);
                        egui::Frame::new()
                            .fill(p.panel)
                            .stroke(egui::Stroke::new(1.0, if item.check.status == Status::Fails { p.bad.gamma_multiply(0.5) } else { p.line }))
                            .corner_radius(8)
                            .inner_margin(egui::Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width() - 2.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(status_icon(item.check.status)).color(color).size(16.0));
                                    ui.add(egui::Label::new(strong(format!("{}: {}", item.title, item.check.name.to_lowercase()))).wrap());
                                });
                                let mut detail = sentence(&item.check.detail);
                                if let Some(cite) = cite_all(&item.check.references) {
                                    detail.push_str(&format!(" · {cite}"));
                                }
                                ui.add(egui::Label::new(muted(p, detail).size(12.5)).wrap());
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    if let Some(rec) = &item.proposed {
                                        if item.check.status == Status::Fails
                                            && ui
                                                .add(egui::Button::new(RichText::new(format!("Usar {}", size_label(rec))).color(p.on_accent)).fill(p.accent))
                                                .on_hover_text("Que IEEStudio vuelva a proponer el calibre")
                                                .clicked()
                                        {
                                            action = Some((item.selection, true));
                                        }
                                    }
                                    let go_label = match item.selection {
                                        Selection::Circuit(..) => "Ir al circuito",
                                        _ => "Ir al tablero",
                                    };
                                    if ui.button(go_label).clicked() {
                                        action = Some((item.selection, false));
                                    }
                                });
                            });
                        if k + 1 < items.len() {
                            ui.add_space(6.0);
                        }
                    }
                    if let Some((selection, fix)) = action {
                        doc.selection = selection;
                        if fix {
                            if let Some(ov) = doc.selected_overrides_mut() {
                                ov.size = None;
                                doc.edited();
                            }
                        } else {
                            go = Some(match selection {
                                Selection::Circuit(..) => Step::Circuits,
                                _ => Step::Panels,
                            });
                        }
                    }

                    if !result.verdict.out_of_scope.is_empty() {
                        ui.add_space(20.0);
                        label_caps(ui, p, "Fuera de alcance · se resuelve con otro estudio");
                        ui.add_space(4.0);
                        for o in &result.verdict.out_of_scope {
                            ui.label(RichText::new(format!("{} {}", icon::INFO, o.label())));
                            ui.label(muted(p, sentence(&o.resolve_with) + " Se anota en la memoria.").size(12.5));
                        }
                    }

                    if !result.warnings.is_empty() {
                        ui.add_space(20.0);
                        label_caps(ui, p, "Avisos");
                        ui.add_space(4.0);
                        for warning in &result.warnings {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{} {warning}", icon::WARNING)).color(p.warn));
                                // El aviso de desbalance se arregla aquí mismo.
                                if warning.contains("desbalance") {
                                    let name = warning.split(':').next().unwrap_or_default();
                                    if let Some(i) = doc.project.panels.iter().position(|x| x.name == name) {
                                        if ui.button("Balancear fases").clicked() {
                                            shared.done = doc.balance_phases(i).map(|(before, after)| {
                                                format!("Fases balanceadas: desbalance de {before:.1} % a {after:.1} %")
                                            });
                                        }
                                    }
                                }
                            });
                        }
                    }
                    ui.add_space(24.0);
                });
        });
    go
}

/// Palabras que delatan un motor o un equipo de aire acondicionado en el
/// nombre de un circuito, sin acentos y en minúsculas.
const MOTOR_WORDS: [&str; 13] = [
    "motor",
    "bomba",
    "compresor de aire",
    "ventilador",
    "extractor",
    "elevador",
    "montacargas",
    "malacate",
    "polipasto",
    "transportador",
    "banda transportadora",
    "molino",
    "hidroneumatico",
];
const AIR_WORDS: [&str; 12] = [
    "aire acondicionado",
    "minisplit",
    "mini split",
    "mini-split",
    "clima",
    "a/a",
    "refrigera",
    "camara fria",
    "chiller",
    "condensadora",
    "enfriador",
    "manejadora",
];

fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|ch| match ch {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

/// Tipo de carga que sugiere el nombre de un circuito capturado como «Otra
/// carga». Un compresor de refrigeración es aire acondicionado; uno de aire,
/// motor.
fn kind_from_name(name: &str) -> Option<LoadKind> {
    let name = fold(name);
    if AIR_WORDS.iter().any(|w| name.contains(w)) {
        Some(LoadKind::AirConditioning)
    } else if MOTOR_WORDS.iter().any(|w| name.contains(w)) {
        Some(LoadKind::Motor)
    } else {
        None
    }
}

/// Circuitos de «Otra carga» cuyo nombre parece de un motor o de un aire
/// acondicionado, con el tipo que sugiere.
fn suspicious_kinds(doc: &Doc) -> Vec<(usize, usize, LoadKind)> {
    let mut out = Vec::new();
    for (i, panel) in doc.project.panels.iter().enumerate() {
        for (c, circuit) in panel.circuits.iter().enumerate() {
            if circuit.load.kind != LoadKind::General {
                continue;
            }
            if let Some(kind) = kind_from_name(&circuit.name) {
                out.push((i, c, kind));
            }
        }
    }
    out
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
    ui.add_space(20.0);
    label_caps(
        ui,
        p,
        &format!("Conexiones que no existen · {}", items.len()),
    );
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(p.panel)
        .stroke(egui::Stroke::new(1.0, p.bad.gamma_multiply(0.6)))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width() - 2.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon::X_CIRCLE).color(p.bad).size(16.0));
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
                    .size(12.5),
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
                    .size(12.5),
                );
            }
            if items.len() > 8 {
                ui.label(muted(p, format!("… y {} más", items.len() - 8)).size(12.5));
            }
            ui.add_space(4.0);
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
/// se capturó como «Otra carga». Devuelve el circuito y el tipo al que se
/// pidió cambiarlo. «Es otra carga» lo oculta mientras la app esté abierta.
fn kind_review(ui: &mut egui::Ui, p: &Palette, doc: &Doc) -> Option<(Selection, LoadKind)> {
    let hidden_id = egui::Id::new("tipo-de-carga-revisado");
    let mut hidden: Vec<(String, String, String)> =
        ui.ctx().data(|d| d.get_temp(hidden_id)).unwrap_or_default();
    let key = |i: usize, c: usize| {
        let panel = &doc.project.panels[i];
        let circuit = &panel.circuits[c];
        (panel.id.clone(), circuit.id.clone(), circuit.name.clone())
    };
    let items: Vec<_> = suspicious_kinds(doc)
        .into_iter()
        .filter(|(i, c, _)| !hidden.contains(&key(*i, *c)))
        .collect();
    if items.is_empty() {
        return None;
    }
    let profile = Profile::by_id(&doc.project.profile).unwrap_or(Profile::nom_001_sede_2012());
    let mut action = None;
    ui.add_space(20.0);
    label_caps(
        ui,
        p,
        &format!("¿Tipo de carga correcto? · {}", items.len()),
    );
    ui.add_space(4.0);
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
            .corner_radius(8)
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width() - 2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon::QUESTION).color(p.warn).size(16.0));
                    ui.add(
                        egui::Label::new(strong(format!(
                            "{} «{}» está como «Otra carga» ({} W)",
                            circuit.id,
                            circuit.name,
                            thousands(circuit.load.power_w)
                        )))
                        .wrap(),
                    );
                });
                ui.add(
                    egui::Label::new(
                        muted(
                            p,
                            format!(
                                "Su nombre parece de {}. Como carga general no se le aplican {missing}. NOM-001-SEDE-2012, {cite}.",
                                if kind == LoadKind::Motor { "un motor" } else { "un equipo de aire acondicionado" },
                            ),
                        )
                        .size(12.5),
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
                        action = Some((Selection::Circuit(i, c), kind));
                    }
                    if ui.button("Es otra carga").clicked() {
                        hidden.push(key(i, c));
                    }
                });
            });
        ui.add_space(6.0);
    }
    ui.ctx().data_mut(|d| d.insert_temp(hidden_id, hidden));
    action
}

// ------------------------------------------------------------- 5 · memoria

fn report_step(
    ui: &mut egui::Ui,
    p: &Palette,
    doc: &mut Doc,
    shared: &mut Shared,
) -> Option<Export> {
    let mut request = None;
    egui::Panel::left("memoria-datos")
        .resizable(false)
        .exact_size(440.0)
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(24, 22)),
        )
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("memoria-datos")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    page_title(
                        ui,
                        p,
                        "Memoria de cálculo",
                        "Los datos de firma y lo que entregas. Lo demás ya salió de los pasos anteriores.",
                    );
                    let mut changed = false;
                    let info = &mut doc.project.info;
                    changed |= text_field(ui, p, "Responsable que firma", &mut info.responsible, "Nombre del ingeniero");
                    ui.add_space(8.0);
                    changed |= text_field(ui, p, "Cédula profesional", &mut info.license, "Número de cédula");
                    ui.add_space(8.0);
                    ui.horizontal_top(|ui| {
                        let half = (ui.available_width() - 12.0) / 2.0;
                        ui.vertical(|ui| {
                            ui.set_width(half);
                            changed |= text_field(ui, p, "Revisión", &mut info.revision, "A, 0, 1…");
                        });
                        ui.vertical(|ui| {
                            ui.set_width(half);
                            changed |= text_field(ui, p, "Fecha", &mut info.date, "Vacía: la del día");
                        });
                    });
                    if changed {
                        doc.edited();
                    }

                    ui.add_space(20.0);
                    label_caps(ui, p, "Antes de entregar");
                    ui.add_space(4.0);
                    let (fails, pending) = problem_counts(doc);
                    let complete = doc
                        .result
                        .as_ref()
                        .is_ok_and(|r| r.verdict.calculation == ieestudio_project::verdict::Calculation::Complete);
                    let info = &doc.project.info;
                    let checklist = [
                        (complete, "Todos los tramos calculados".to_string()),
                        (fails == 0, match fails {
                            0 => "Nada incumple".to_string(),
                            1 => "1 verificación no cumple".to_string(),
                            n => format!("{n} verificaciones no cumplen"),
                        }),
                        (pending == 0, match pending {
                            0 => "Nada quedó sin evaluar".to_string(),
                            1 => "1 verificación sin evaluar".to_string(),
                            n => format!("{n} verificaciones sin evaluar"),
                        }),
                        (
                            !info.responsible.trim().is_empty() && !info.license.trim().is_empty(),
                            "Responsable y cédula capturados".to_string(),
                        ),
                    ];
                    for (ok, text) in checklist {
                        let (glyph, color) = if ok { (icon::CHECK_CIRCLE, p.ok) } else { (icon::CIRCLE_DASHED, p.warn) };
                        ui.label(RichText::new(format!("{glyph} {text}")).color(if ok { p.ink } else { color }));
                    }
                    if let Ok(r) = &doc.result {
                        for o in &r.verdict.out_of_scope {
                            ui.label(muted(p, format!("{} {}: fuera de alcance, se anota en la memoria", icon::INFO, o.name)).size(12.5));
                        }
                    }

                    ui.add_space(24.0);
                    label_caps(ui, p, "Exportar");
                    ui.add_space(4.0);
                    if primary(ui, p, &format!("{}  Memoria en PDF", icon::FILE_PDF)) {
                        request = Some(Export::Pdf);
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button(format!("{}  Cuadro de cargas en Excel", icon::FILE_XLS)).clicked() {
                            request = Some(Export::Excel);
                        }
                        if ui.button(format!("{}  Unifilar en DXF", icon::FLOW_ARROW)).clicked() {
                            request = Some(Export::Dxf);
                        }
                    });
                });
        });
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(p.bg)
                .inner_margin(egui::Margin::symmetric(16, 12)),
        )
        .show(ui, |ui| {
            if report::show(ui, doc, shared) {
                request = Some(Export::Pdf);
            }
        });
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_groups_digits() {
        assert_eq!(thousands(0.0), "0");
        assert_eq!(thousands(1800.0), "1,800");
        assert_eq!(thousands(7704.4), "7,704");
        assert_eq!(thousands(1_234_567.0), "1,234,567");
    }

    #[test]
    fn names_that_look_like_equipment() {
        assert_eq!(kind_from_name("Bomba de agua"), Some(LoadKind::Motor));
        assert_eq!(kind_from_name("EXTRACTOR baño"), Some(LoadKind::Motor));
        assert_eq!(kind_from_name("Hidroneumático"), Some(LoadKind::Motor));
        assert_eq!(
            kind_from_name("Minisplit recámara"),
            Some(LoadKind::AirConditioning)
        );
        assert_eq!(
            kind_from_name("Cámara fría"),
            Some(LoadKind::AirConditioning)
        );
        // Un compresor de refrigeración es aire acondicionado, no motor.
        assert_eq!(
            kind_from_name("Compresor de refrigeración"),
            Some(LoadKind::AirConditioning)
        );
        assert_eq!(kind_from_name("Compresor de aire"), Some(LoadKind::Motor));
        assert_eq!(kind_from_name("Contactos sala"), None);
        assert_eq!(kind_from_name("Alumbrado exterior"), None);
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
    }

    #[test]
    fn steps_go_back_and_forth() {
        assert_eq!(Step::Project.previous(), None);
        assert_eq!(Step::Project.next(), Some(Step::Panels));
        assert_eq!(Step::Report.next(), None);
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
}
