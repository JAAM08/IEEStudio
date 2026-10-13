//! Panel de propiedades del elemento seleccionado, en tres pestañas que no se
//! mezclan: lo que capturó el usuario, lo que calculó VoltForge y lo que se
//! verificó contra la norma.

use eframe::egui::{self, RichText};
use egui_phosphor::regular as icon;
use voltforge_core::{CircuitKind, Size};
use voltforge_project::{
    size_label, Area, CompressorLoad, Demand, LoadKind, MotorLoad, Overrides, Phase, RunResult,
    Status,
};
use voltforge_rules::{cite_all, Profile, Rule};

use super::Shared;
use crate::state::{Doc, Selection};
use crate::theme::Palette;
use crate::widgets::{
    combo, combo_override, load_kind_hint, motor_hp_combo, muted, number, optional_number,
    property, section, source_badge, status_text, strong, Source, CONDUITS, INSULATIONS, KINDS,
    LOAD_KINDS, MATERIALS, MOTOR_TYPES, SYSTEMS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropsTab {
    #[default]
    Inputs,
    Results,
    Normative,
}

pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) {
    let p = shared.palette;
    let (title, status) = match doc.selection {
        Selection::Project => (doc.project.name.clone(), None),
        Selection::Feeder(i) => (
            format!("Alimentador · {}", doc.project.panels[i].id),
            doc.selected_run().map(|r| r.status),
        ),
        Selection::Circuit(i, c) => {
            let circuit = &doc.project.panels[i].circuits[c];
            (
                format!("{} · {}", circuit.id, circuit.name),
                doc.selected_run().map(|r| r.status),
            )
        }
    };
    ui.horizontal(|ui| {
        ui.add(egui::Label::new(strong(title).size(14.0)).truncate());
    });
    if let Some(s) = status {
        ui.label(status_text(&p, s));
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for (tab, label, tip) in [
            (PropsTab::Inputs, "Datos", "Lo que capturaste"),
            (PropsTab::Results, "Resultados", "Lo que calculó VoltForge"),
            (
                PropsTab::Normative,
                "Normativa",
                "Lo que se verificó y dónde lo dice la norma",
            ),
        ] {
            ui.selectable_value(&mut shared.props_tab, tab, label)
                .on_hover_text(tip);
        }
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("propiedades")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Sin cálculo, Resultados y Normativa dicen por qué en vez de
            // quedarse en blanco; Datos sigue editable para corregirlo.
            if shared.props_tab != PropsTab::Inputs {
                if let Some(e) = doc.calc_error() {
                    ui.label(
                        RichText::new(format!("{} No se pudo calcular", icon::WARNING_CIRCLE))
                            .color(p.bad),
                    );
                    ui.label(e);
                    ui.label(muted(&p, "Corrige el dato en la pestaña Datos."));
                    return;
                }
            }
            let changed = match (shared.props_tab, doc.selection) {
                (PropsTab::Inputs, Selection::Project) => project_inputs(ui, doc, &p),
                (PropsTab::Inputs, Selection::Feeder(i)) => feeder_inputs(ui, doc, &p, i),
                (PropsTab::Inputs, Selection::Circuit(i, c)) => circuit_inputs(ui, doc, &p, i, c),
                (PropsTab::Results, Selection::Project) => {
                    project_results(ui, doc, &p);
                    false
                }
                (PropsTab::Results, _) => run_results(ui, doc, &p),
                (PropsTab::Normative, Selection::Project) => {
                    profile_references(ui, doc, &p);
                    false
                }
                (PropsTab::Normative, _) => {
                    if let Some(run) = doc.selected_run().cloned() {
                        checks(ui, &p, &run);
                    }
                    false
                }
            };
            if changed {
                doc.edited();
            }
        });
}

// ---------------------------------------------------------------- datos

fn project_inputs(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette) -> bool {
    let mut changed = false;
    let project = &mut doc.project;
    section(ui, p, "Proyecto");
    property(ui, p, "Nombre", |ui| {
        changed |= ui.text_edit_singleline(&mut project.name).changed();
    });
    property(ui, p, "Perfil normativo", |ui| {
        ui.label("NOM-001-SEDE-2012");
    });
    section(ui, p, "Inmueble");
    property(ui, p, "Tipo de inmueble", |ui| {
        changed |= occupancy_combo(ui, "inmueble", &mut project.occupancy, false, "Sin elegir");
    })
    .on_hover_text(
        "Tabla 220-12. Decide la carga mínima de alumbrado por m² y los factores de demanda de los alimentadores (artículo 220).",
    );
    if project.occupancy.is_none() {
        ui.label(
            muted(
                p,
                "Sin el tipo de inmueble, los alimentadores con alumbrado o contactos quedan sin evaluar.",
            )
            .size(11.0),
        );
    }
    section(ui, p, "Suministro");
    property(ui, p, "Nombre", |ui| {
        changed |= ui.text_edit_singleline(&mut project.source.name).changed();
    });
    property(ui, p, "Sistema", |ui| {
        changed |= combo(ui, "sistema", &mut project.source.system, &SYSTEMS);
    });

    let s = &mut project.settings;
    section(ui, p, "Criterios de diseño");
    property(ui, p, "Caída en derivado", |ui| {
        changed |= number(ui, &mut s.branch_drop_limit_percent, 0.1, 0.5, 10.0, " %");
    });
    property(ui, p, "Caída en alimentador", |ui| {
        changed |= number(ui, &mut s.feeder_drop_limit_percent, 0.1, 0.5, 10.0, " %");
    });
    property(ui, p, "Caída acumulada", |ui| {
        changed |= number(ui, &mut s.total_drop_limit_percent, 0.1, 0.5, 15.0, " %");
    });
    property(ui, p, "Temp. ambiente", |ui| {
        changed |= number(ui, &mut s.ambient_c, 1.0, 0.0, 85.0, " °C");
    });
    property(ui, p, "Conductor", |ui| {
        changed |= combo(ui, "material", &mut s.material, &MATERIALS);
    });
    property(ui, p, "Aislamiento", |ui| {
        changed |= combo(ui, "aislamiento", &mut s.insulation, &INSULATIONS);
    });
    property(ui, p, "Canalización", |ui| {
        changed |= combo(ui, "tubo", &mut s.conduit, &CONDUITS);
    });
    changed |= ui
        .checkbox(&mut s.bare_grounding, "Tierra desnuda")
        .changed();
    changed |= ui.checkbox(&mut s.wet_location, "Lugar mojado").changed();
    changed |= ui
        .checkbox(
            &mut s.include_iec_ratings,
            "Admitir interruptores de 16, 32 y 63 A",
        )
        .changed();
    let mut adjust = s.resistance_temperature_c.is_some();
    if ui
        .checkbox(&mut adjust, "Ajustar resistencia por temperatura")
        .changed()
    {
        s.resistance_temperature_c = adjust.then_some(60.0);
        changed = true;
    }
    if let Some(t) = &mut s.resistance_temperature_c {
        property(ui, p, "Temp. del conductor", |ui| {
            changed |= number(ui, t, 1.0, 20.0, 90.0, " °C");
        });
    }
    changed
}

fn feeder_inputs(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette, i: usize) -> bool {
    let mut changed = false;
    let others = doc.feeder_candidates(i);
    let source_name = doc.project.source.name.clone();
    let settings = doc.project.settings.clone();
    let id_problem = doc.panel_id_problem(i);
    let old_id = doc.project.panels[i].id.clone();
    let building = doc
        .project
        .occupancy
        .as_deref()
        .and_then(|id| Profile::nom_001_sede_2012().occupancy(id))
        .map_or("Del inmueble".to_string(), |o| {
            format!("{} · proyecto", o.label)
        });
    let mut renamed = false;
    section(ui, p, "Tablero");
    property(ui, p, "ID", |ui| {
        renamed = ui
            .add(egui::TextEdit::singleline(&mut doc.project.panels[i].id).desired_width(80.0))
            .changed();
    });
    if renamed {
        doc.panel_renamed(i, &old_id);
        changed = true;
    }
    if let Some(problem) = id_problem {
        ui.label(RichText::new(format!("{} {problem}", icon::WARNING)).color(p.bad));
    }
    let panel = &mut doc.project.panels[i];
    property(ui, p, "Nombre", |ui| {
        changed |= ui.text_edit_singleline(&mut panel.name).changed();
    });
    property(ui, p, "Se alimenta de", |ui| {
        let current = panel.parent.clone().unwrap_or_else(|| source_name.clone());
        egui::ComboBox::from_id_salt("padre")
            .selected_text(current)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut panel.parent, None, &source_name)
                    .changed();
                for (id, name) in &others {
                    changed |= ui
                        .selectable_value(
                            &mut panel.parent,
                            Some(id.clone()),
                            format!("{id} · {name}"),
                        )
                        .changed();
                }
            });
    });
    section(ui, p, "Alimentador");
    property(ui, p, "Conexión", |ui| {
        changed |= combo(ui, "conexion-tablero", &mut panel.kind, &KINDS);
    });
    property(ui, p, "Longitud", |ui| {
        changed |= number(ui, &mut panel.feeder.length_m, 0.5, 0.0, 5000.0, " m");
    });
    changed |= ui
        .checkbox(&mut panel.feeder.has_neutral, "Lleva neutro")
        .changed();

    section(ui, p, "Carga del alimentador");
    property(ui, p, "Factor de demanda", |ui| {
        let text = match panel.feeder.demand {
            Demand::Article220 => "Artículo 220",
            Demand::Full => "Sin factores (100 %)",
            Demand::Custom(_) => "Factor propio",
        };
        egui::ComboBox::from_id_salt("demanda")
            .selected_text(text)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut panel.feeder.demand, Demand::Article220, "Artículo 220")
                    .on_hover_text(
                        "Factores de las tablas 220-42, 220-44 y 220-56 y de 220-53, según el tipo de inmueble.",
                    )
                    .changed();
                changed |= ui
                    .selectable_value(&mut panel.feeder.demand, Demand::Full, "Sin factores (100 %)")
                    .on_hover_text(
                        "Toda la carga al 100 %. Por ejemplo, áreas de hospital u hotel donde se usa todo el alumbrado a la vez.",
                    )
                    .changed();
                let custom = matches!(panel.feeder.demand, Demand::Custom(_));
                if ui.selectable_label(custom, "Factor propio").clicked() && !custom {
                    panel.feeder.demand = Demand::Custom(1.0);
                    changed = true;
                }
            });
    });
    if let Demand::Custom(f) = &mut panel.feeder.demand {
        property(ui, p, "Factor", |ui| {
            changed |= number(ui, f, 0.01, 0.05, 1.0, "");
        });
        ui.label(
            muted(
                p,
                "Un factor propio menor que 1 deja el alimentador sin evaluar: el artículo 220 no lo respalda por sí mismo.",
            )
            .size(11.0),
        );
    }

    section(ui, p, "Áreas que atiende");
    ui.label(
        muted(
            p,
            "Superficie de piso que alimentan los circuitos de este tablero, para la carga mínima de alumbrado (220-12). Las de sus subtableros se suman solas.",
        )
        .size(11.0),
    );
    let mut remove = None;
    for (k, area) in panel.areas.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut area.description)
                        .hint_text("Descripción")
                        .desired_width(120.0),
                )
                .changed();
            changed |= number(ui, &mut area.m2, 1.0, 0.0, 1.0e6, " m²");
            if ui
                .small_button(icon::TRASH)
                .on_hover_text("Quitar el área")
                .clicked()
            {
                remove = Some(k);
            }
        });
        property(ui, p, "Tipo", |ui| {
            changed |= occupancy_combo(ui, ("area", k), &mut area.occupancy, true, &building);
        });
        ui.add_space(4.0);
    }
    if let Some(k) = remove {
        panel.areas.remove(k);
        changed = true;
    }
    if ui.button(format!("{} Agregar área", icon::PLUS)).clicked() {
        panel.areas.push(Area {
            description: String::new(),
            occupancy: None,
            m2: 100.0,
        });
        changed = true;
    }
    changed |= decisions(ui, p, &mut panel.feeder.overrides, "alim", &settings);
    changed
}

/// Tipo de inmueble o de área de la Tabla 220-12. `none` es el texto de la
/// opción vacía: «Sin elegir» o el tipo del proyecto.
fn occupancy_combo(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    value: &mut Option<String>,
    areas: bool,
    none: &str,
) -> bool {
    let profile = Profile::nom_001_sede_2012();
    let mut changed = false;
    let text = value
        .as_deref()
        .and_then(|v| profile.occupancy(v))
        .map_or(none.to_string(), |o| o.label.clone());
    egui::ComboBox::from_id_salt(id)
        .selected_text(text)
        .width((ui.available_width() - 8.0).clamp(120.0, 190.0))
        .truncate()
        .height(360.0)
        .show_ui(ui, |ui| {
            changed |= ui.selectable_value(value, None, none).changed();
            for o in profile.occupancies.iter().filter(|o| areas || !o.area_only) {
                changed |= ui
                    .selectable_value(
                        value,
                        Some(o.id.clone()),
                        format!("{} · {} VA/m²", o.label, o.va_per_m2),
                    )
                    .changed();
            }
        });
    changed
}

fn circuit_inputs(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette, i: usize, c: usize) -> bool {
    let mut changed = false;
    let settings = doc.project.settings.clone();
    let id_problem = doc.circuit_id_problem(i, c);
    // Potencia calculada de un motor o compresor, para mostrarla.
    let computed_w = doc
        .panel_result(i)
        .and_then(|r| r.circuits.get(c))
        .map(|r| r.load_w);
    let circuit = &mut doc.project.panels[i].circuits[c];
    section(ui, p, "Circuito");
    property(ui, p, "ID", |ui| {
        changed |= ui
            .add(egui::TextEdit::singleline(&mut circuit.id).desired_width(80.0))
            .changed();
    });
    if let Some(problem) = id_problem {
        ui.label(RichText::new(format!("{} {problem}", icon::WARNING)).color(p.warn));
    }
    property(ui, p, "Descripción", |ui| {
        changed |= ui.text_edit_singleline(&mut circuit.name).changed();
    });
    section(ui, p, "Carga");
    property(ui, p, "Tipo de carga", |ui| {
        if combo(ui, ("tipo-carga", c), &mut circuit.load.kind, &LOAD_KINDS) {
            changed = true;
            // Al elegir motor o aire acondicionado se piden sus datos.
            match circuit.load.kind {
                LoadKind::Motor if circuit.load.motor.is_none() => {
                    circuit.load.motor = Some(MotorLoad {
                        hp: "1".to_string(),
                        motor_type: Default::default(),
                        nameplate_a: None,
                        service_factor_marked: false,
                    });
                }
                LoadKind::AirConditioning if circuit.load.compressor.is_none() => {
                    circuit.load.compressor = Some(CompressorLoad::default());
                }
                _ => {}
            }
        }
    })
    .on_hover_text(load_kind_hint(circuit.load.kind));
    if circuit.load.kind == LoadKind::Receptacles {
        let unit = Profile::nom_001_sede_2012().parameters.receptacle_outlet_va;
        property(ui, p, "Salidas", |ui| {
            let mut n = circuit.load.outlets.unwrap_or(0);
            if ui
                .add(egui::DragValue::new(&mut n).range(0..=500).suffix(" contactos"))
                .changed()
            {
                circuit.load.outlets = (n > 0).then_some(n);
                if n > 0 {
                    circuit.load.power_w = unit * f64::from(n) * circuit.load.power_factor;
                }
                changed = true;
            }
        })
        .on_hover_text(format!(
            "Cada salida se calcula con {unit:.0} VA (220-14(i)). Al cambiar el número se recalcula la potencia; puedes capturar una mayor."
        ));
    }
    let equipment = matches!(
        (
            circuit.load.kind,
            &circuit.load.motor,
            &circuit.load.compressor
        ),
        (LoadKind::Motor, Some(_), _) | (LoadKind::AirConditioning, _, Some(_))
    );
    if equipment {
        property(ui, p, "Potencia", |ui| {
            ui.label(
                RichText::new(computed_w.map_or("—".to_string(), |w| format!("{w:.0} W")))
                    .monospace(),
            )
            .on_hover_text(
                "Calculada con la corriente del equipo: la de tabla del motor o la de placa del compresor.",
            );
        });
    } else {
        property(ui, p, "Potencia", |ui| {
            changed |= number(ui, &mut circuit.load.power_w, 10.0, 0.0, 5.0e6, " W");
        });
    }
    property(ui, p, "Factor de potencia", |ui| {
        changed |= ui
            .add(
                egui::DragValue::new(&mut circuit.load.power_factor)
                    .speed(0.005)
                    .range(0.1..=1.0)
                    .fixed_decimals(2),
            )
            .changed();
    });
    if !equipment {
        changed |= ui
            .checkbox(&mut circuit.load.continuous, "Carga continua (3 h o más)")
            .changed();
    }
    let three_phase = circuit.kind == CircuitKind::ThreePhase;
    if let (LoadKind::Motor, Some(motor)) = (circuit.load.kind, circuit.load.motor.as_mut()) {
        changed |= motor_inputs(ui, p, motor, three_phase, c);
    }
    if let (LoadKind::AirConditioning, Some(plate)) =
        (circuit.load.kind, circuit.load.compressor.as_mut())
    {
        changed |= compressor_inputs(ui, p, plate);
    }
    section(ui, p, "Conexión");
    property(ui, p, "Conexión", |ui| {
        if combo(ui, ("conexion", c), &mut circuit.kind, &KINDS) {
            circuit.phases.clear();
            changed = true;
        }
    });
    property(ui, p, "Fases", |ui| {
        for (label, phase) in [("A", Phase::A), ("B", Phase::B), ("C", Phase::C)] {
            let mut on = circuit.phases.contains(&phase);
            if ui.checkbox(&mut on, label).changed() {
                if on {
                    circuit.phases.push(phase);
                } else {
                    circuit.phases.retain(|x| *x != phase);
                }
                changed = true;
            }
        }
    });
    if circuit.kind != voltforge_core::CircuitKind::LineToNeutral {
        changed |= ui
            .checkbox(&mut circuit.has_neutral, "Lleva neutro")
            .changed();
    }
    section(ui, p, "Instalación");
    property(ui, p, "Longitud", |ui| {
        changed |= number(ui, &mut circuit.length_m, 0.5, 0.0, 5000.0, " m");
    });
    changed |= decisions(ui, p, &mut circuit.overrides, &format!("c{c}"), &settings);
    changed
}

/// Datos del motor: potencia de la tabla, tipo y placa.
fn motor_inputs(
    ui: &mut egui::Ui,
    p: &Palette,
    motor: &mut MotorLoad,
    three_phase: bool,
    c: usize,
) -> bool {
    let mut changed = false;
    section(ui, p, "Motor");
    property(ui, p, "Potencia", |ui| {
        changed |= motor_hp_combo(ui, ("hp", c), &mut motor.hp, three_phase);
    })
    .on_hover_text(if three_phase {
        "Tabla 430-250: motores de tres fases"
    } else {
        "Tabla 430-248: motores de una fase"
    });
    if three_phase {
        property(ui, p, "Tipo", |ui| {
            changed |= combo(ui, ("tipo-motor", c), &mut motor.motor_type, &MOTOR_TYPES);
        });
    }
    property(ui, p, "Corriente de placa", |ui| {
        changed |= optional_number(ui, &mut motor.nameplate_a, 10.0, " A");
    })
    .on_hover_text("Para ajustar el relevador de sobrecarga. Conductores y protección usan la corriente de la tabla.");
    changed |= ui
        .checkbox(
            &mut motor.service_factor_marked,
            "Factor de servicio 1.15 o más, o elevación de 40 °C",
        )
        .on_hover_text("El relevador de sobrecarga puede ajustarse al 125 % en lugar del 115 %.")
        .changed();
    changed
}

/// Datos de placa de un equipo de aire acondicionado o refrigeración.
fn compressor_inputs(ui: &mut egui::Ui, p: &Palette, plate: &mut CompressorLoad) -> bool {
    let mut changed = false;
    section(ui, p, "Placa del equipo");
    ui.label(
        muted(
            p,
            "Basta la corriente de carga nominal, o la ampacidad mínima y la protección máxima.",
        )
        .size(11.0),
    );
    for (label, value, default, tip) in [
        (
            "Nominal (RLA)",
            &mut plate.rated_load_a,
            10.0,
            "Corriente de carga nominal del motocompresor",
        ),
        (
            "Selección (BCSC)",
            &mut plate.branch_selection_a,
            10.0,
            "Corriente de selección del circuito derivado, si la placa la trae",
        ),
        (
            "Mínima (MCA)",
            &mut plate.minimum_circuit_a,
            15.0,
            "Ampacidad mínima del circuito marcada en el equipo",
        ),
        (
            "Máxima (MOP)",
            &mut plate.maximum_protection_a,
            20.0,
            "Protección máxima marcada en el equipo",
        ),
    ] {
        property(ui, p, label, |ui| {
            changed |= optional_number(ui, value, default, " A");
        })
        .on_hover_text(tip);
    }
    changed
}

/// Decisiones opcionales que sustituyen lo que VoltForge propondría.
fn decisions(
    ui: &mut egui::Ui,
    p: &Palette,
    ov: &mut Overrides,
    id: &str,
    project: &voltforge_project::Settings,
) -> bool {
    let mut changed = false;
    section(ui, p, "Decisiones de diseño (opcional)");
    ui.label(
        muted(
            p,
            "Si no fijas nada, VoltForge propone el menor valor que cumple.",
        )
        .size(11.0),
    );
    property(ui, p, "Calibre", |ui| {
        let text = ov
            .size
            .as_ref()
            .map_or("Lo propone VoltForge".to_string(), size_label);
        egui::ComboBox::from_id_salt(("calibre", id))
            .selected_text(text)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut ov.size, None, "Lo propone VoltForge")
                    .changed();
                for c in Profile::nom_001_sede_2012()
                    .sizes
                    .iter()
                    .skip_while(|c| c.size != "14")
                {
                    let size = Size::new(&c.size);
                    let label = size_label(&size);
                    changed |= ui
                        .selectable_value(&mut ov.size, Some(size), label)
                        .changed();
                }
            });
    });
    property(ui, p, "Interruptor", |ui| {
        let text = ov
            .breaker_a
            .map_or("Lo propone VoltForge".to_string(), |b| format!("{b:.0} A"));
        egui::ComboBox::from_id_salt(("interruptor", id))
            .selected_text(text)
            .height(320.0)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut ov.breaker_a, None, "Lo propone VoltForge")
                    .changed();
                for r in Profile::nom_001_sede_2012()
                    .standard_ratings
                    .iter()
                    .filter(|r| r.common || project.include_iec_ratings)
                {
                    changed |= ui
                        .selectable_value(
                            &mut ov.breaker_a,
                            Some(r.rating_a),
                            format!("{:.0} A", r.rating_a),
                        )
                        .changed();
                }
            });
    })
    .on_hover_text("VoltForge verifica el interruptor que fijes; no lo cambia.");
    property(ui, p, "Conductores por fase", |ui| {
        let mut n = ov.conductors_per_phase.unwrap_or(1);
        if ui.add(egui::DragValue::new(&mut n).range(1..=12)).changed() {
            ov.conductors_per_phase = (n > 1).then_some(n);
            changed = true;
        }
    });
    property(ui, p, "Material", |ui| {
        changed |= combo_override(
            ui,
            ("mat", id),
            &mut ov.material,
            &MATERIALS,
            crate::widgets::label_of(&MATERIALS, project.material),
        );
    });
    property(ui, p, "Aislamiento", |ui| {
        changed |= combo_override(
            ui,
            ("ais", id),
            &mut ov.insulation,
            &INSULATIONS,
            crate::widgets::label_of(&INSULATIONS, project.insulation),
        );
    });
    property(ui, p, "Canalización", |ui| {
        changed |= combo_override(
            ui,
            ("tub", id),
            &mut ov.conduit,
            &CONDUITS,
            crate::widgets::label_of(&CONDUITS, project.conduit),
        );
    });
    changed |= ui
        .checkbox(&mut ov.nipple, "Tramo de 60 cm o menos (niple)")
        .changed();
    changed |= ui
        .checkbox(&mut ov.nonlinear, "Carga mayormente no lineal")
        .on_hover_text("El neutro cuenta como conductor portador de corriente.")
        .changed();
    property(ui, p, "Otros portadores", |ui| {
        changed |= ui
            .add(egui::DragValue::new(&mut ov.other_current_carrying_conductors).range(0..=40))
            .on_hover_text("Conductores portadores de otros circuitos en la misma tubería")
            .changed();
    });
    changed
}

// ---------------------------------------------------------------- resultados

fn result_row(ui: &mut egui::Ui, p: &Palette, label: &str, value: String, source: Source) {
    ui.horizontal(|ui| {
        crate::widgets::fixed_label(ui, p, label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Solo se marca lo que no es un cálculo directo: lo propuesto y lo fijado.
            if source != Source::Calculated {
                source_badge(ui, p, source);
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(RichText::new(value).monospace()).truncate());
            });
        });
    });
}

fn run_results(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette) -> bool {
    let Some(item) = doc.selected_run().cloned() else {
        return false;
    };
    ui.label(
        muted(
            p,
            format!(
                "{} Calculado por VoltForge con NOM-001-SEDE-2012. Solo lectura.",
                icon::LOCK_SIMPLE
            ),
        )
        .size(11.0),
    );
    if let Selection::Feeder(i) = doc.selection {
        if let Some(panel) = doc.panel_result(i).cloned() {
            demand_results(ui, p, &panel);
        }
    }
    let Some(run) = &item.run else {
        section(ui, p, "Sin resultado");
        for check in &item.checks {
            ui.label(muted(p, &check.detail));
        }
        return false;
    };
    let calc = Source::Calculated;
    section(ui, p, "Corriente");
    result_row(ui, p, "Nominal", format!("{:.2} A", run.nominal_a), calc);
    result_row(ui, p, "De diseño", format!("{:.2} A", run.design_a), calc);
    section(ui, p, "Protección");
    result_row(
        ui,
        p,
        "Interruptor",
        format!("{}P-{:.0} A", run.poles, run.breaker_a),
        calc,
    );
    section(ui, p, "Conductor");
    let size = if run.conductors_per_phase > 1 {
        format!(
            "{} × {} por fase",
            run.conductors_per_phase,
            size_label(&run.size)
        )
    } else {
        size_label(&run.size)
    };
    result_row(
        ui,
        p,
        "Calibre",
        size,
        if run.forced {
            Source::Fixed
        } else {
            Source::Proposed
        },
    );
    result_row(
        ui,
        p,
        "Ampacidad permitida",
        format!("{:.2} A", run.allowable_ampacity_a),
        calc,
    );
    result_row(
        ui,
        p,
        "Portadores",
        format!("{}", run.current_carrying_conductors),
        calc,
    );
    result_row(
        ui,
        p,
        "Puesta a tierra",
        size_label(&run.grounding_size),
        calc,
    );

    // Propuesta frente a decisión: siempre explícito, nunca se aplica solo.
    let mut set_size: Option<Option<Size>> = None;
    egui::Frame::new()
        .fill(p.raise)
        .corner_radius(4)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            match (run.forced, &run.recommended_size, &run.governing) {
                (true, Some(rec), Some(gov)) if rec != &run.size => {
                    ui.label(format!(
                        "Fijaste {}. VoltForge propondría {} (lo determina {gov}).",
                        size_label(&run.size),
                        size_label(rec)
                    ));
                    if ui.button("Usar el propuesto").clicked() {
                        set_size = Some(None);
                    }
                }
                (true, _, _) => {
                    ui.label(format!(
                        "Fijaste {}; coincide con lo que propone VoltForge.",
                        size_label(&run.size)
                    ));
                    if ui.button("Dejar que lo proponga VoltForge").clicked() {
                        set_size = Some(None);
                    }
                }
                (false, _, Some(gov)) => {
                    ui.label(format!(
                        "Propuesto: el menor calibre que cumple. Lo determina {gov}."
                    ));
                    if ui
                        .button(format!(
                            "{} Fijar {}",
                            icon::LOCK_SIMPLE,
                            size_label(&run.size)
                        ))
                        .on_hover_text("Conservar este calibre aunque cambien los datos")
                        .clicked()
                    {
                        set_size = Some(Some(run.size.clone()));
                    }
                }
                _ => {
                    ui.label(muted(p, "Ningún calibre cumple con estos datos."));
                }
            }
        });

    section(ui, p, "Tubería");
    let conduit = run.conduit.as_ref().map_or("—".to_string(), |c| {
        format!(
            "{} {} mm ({}\")",
            c.conduit_type, c.metric_designator, c.trade_size
        )
    });
    result_row(ui, p, "Tubo", conduit, calc);
    if let Some(c) = &run.conduit {
        result_row(ui, p, "Ocupación", format!("{:.1} %", c.fill_percent), calc);
    }
    section(ui, p, "Caída de tensión");
    result_row(
        ui,
        p,
        "Del tramo",
        format!("{:.2} V · {:.2} %", run.drop_v, run.drop_percent),
        calc,
    );
    result_row(
        ui,
        p,
        "Acumulada",
        format!("{:.2} %", run.cumulative_drop_percent),
        calc,
    );

    if let Some(size) = set_size {
        match doc.selection {
            Selection::Feeder(i) => doc.project.panels[i].feeder.overrides.size = size,
            Selection::Circuit(i, c) => doc.project.panels[i].circuits[c].overrides.size = size,
            Selection::Project => {}
        }
        return true;
    }
    false
}

/// Carga del alimentador por grupo: de lo conectado a lo calculado.
fn demand_results(ui: &mut egui::Ui, p: &Palette, panel: &voltforge_project::PanelResult) {
    if panel.demand.is_empty() {
        return;
    }
    section(ui, p, "Carga (artículo 220)");
    for line in &panel.demand {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new(strong(&line.label)).truncate());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{:.0} VA", line.demanded_va)).monospace());
            });
        });
        ui.label(muted(p, &line.detail).size(11.0));
        ui.add_space(2.0);
    }
    result_row(
        ui,
        p,
        "Conectada",
        format!("{:.0} W", panel.connected_w),
        Source::Calculated,
    );
    result_row(
        ui,
        p,
        "Calculada",
        format!("{:.0} VA · {:.0} W", panel.demanded_va, panel.demanded_w),
        Source::Calculated,
    );
    result_row(
        ui,
        p,
        "Factor resultante",
        format!("{:.2}", panel.demand_factor),
        Source::Calculated,
    );
}

fn project_results(ui: &mut egui::Ui, doc: &Doc, p: &Palette) {
    let Ok(result) = &doc.result else {
        ui.label(
            RichText::new(doc.result.as_ref().err().cloned().unwrap_or_default()).color(p.bad),
        );
        return;
    };
    section(ui, p, "Veredicto");
    for (level, status, detail) in result.verdict.lines("NOM-001-SEDE-2012") {
        ui.label(strong(level));
        let color = match status {
            Status::Complies => p.ok,
            Status::Fails => p.bad,
            Status::NotEvaluated => p.warn,
        };
        ui.label(RichText::new(detail).color(color));
        ui.add_space(4.0);
    }
    section(ui, p, "Tableros");
    for panel in &result.panels {
        ui.horizontal(|ui| {
            ui.label(&panel.name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{:.0} W", panel.demanded_w)).monospace());
            });
        });
    }
    if !result.warnings.is_empty() {
        section(ui, p, "Avisos");
        for w in &result.warnings {
            ui.label(RichText::new(format!("{} {w}", icon::WARNING_CIRCLE)).color(p.warn));
        }
    }
}

// ---------------------------------------------------------------- normativa

fn checks(ui: &mut egui::Ui, p: &Palette, item: &RunResult) {
    section(ui, p, "Verificaciones");
    for check in &item.checks {
        ui.horizontal(|ui| {
            ui.label(strong(&check.name));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(status_text(p, check.status));
            });
        });
        ui.label(muted(p, &check.detail));
        if let Some(cite) = cite_all(&check.references) {
            ui.label(
                RichText::new(format!("{} {cite}", icon::BOOK_OPEN))
                    .size(11.0)
                    .color(p.ink3),
            );
        }
        ui.add_space(6.0);
    }
    ui.separator();
    ui.label(
        muted(
            p,
            "Cumple significa que pasó la verificación del programa. La aprobación profesional corresponde a quien firma el proyecto.",
        )
        .size(11.0),
    );
}

fn profile_references(ui: &mut egui::Ui, doc: &Doc, p: &Palette) {
    let profile = Profile::by_id(&doc.project.profile).unwrap_or(Profile::nom_001_sede_2012());
    let d = &profile.document;
    section(ui, p, "Norma del perfil");
    ui.label(strong(&d.code));
    ui.label(format!("{} · edición {}", d.title, d.edition));
    if let Some(url) = &d.source_url {
        ui.hyperlink_to("Publicación oficial", url);
    }
    section(ui, p, "Reglas que aplica VoltForge");
    for rule in Rule::ALL {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(profile.clause(rule))
                    .monospace()
                    .color(p.accent_text),
            );
            if let Some(desc) = profile.rule_description(rule) {
                ui.label(muted(p, desc));
            }
        });
    }
}
