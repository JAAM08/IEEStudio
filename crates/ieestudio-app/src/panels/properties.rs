//! Panel de propiedades del elemento seleccionado, en tres pestañas que no se
//! mezclan: lo que capturó el usuario, lo que calculó IEEStudio y lo que se
//! verificó contra la norma.

use eframe::egui::{self, RichText};
use egui_phosphor::regular as icon;
use ieestudio_core::{CircuitKind, Size};
use ieestudio_project::{
    size_label, Area, CompressorLoad, Demand, LoadKind, MotorLoad, Overrides, Phase, RunResult,
    Status,
};
use ieestudio_rules::{cite_all, Profile, Rule};

use super::Shared;
use crate::state::{set_load_kind, Doc, Selection};
use crate::theme::Palette;
use crate::widgets::{
    combo, combo_override, load_kind_hint, motor_hp_combo, muted, number, optional_number,
    property, proposed_text, reset_button, section, source_badge, status_text, strong, Source,
    CONDUITS, INSULATIONS, KINDS, LOAD_KINDS, MATERIALS, MOTOR_TYPES, SYSTEMS,
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
        Selection::Secondary(i) => (
            format!("Secundario · {}", doc.project.panels[i].id),
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
            (PropsTab::Results, "Resultados", "Lo que calculó IEEStudio"),
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
                (PropsTab::Inputs, Selection::Secondary(i)) => secondary_inputs(ui, doc, &p, i),
                (PropsTab::Inputs, Selection::Circuit(i, c)) => circuit_inputs(ui, doc, &p, i, c),
                (PropsTab::Results, Selection::Project) => {
                    project_results(ui, doc, &p);
                    false
                }
                (PropsTab::Results, _) => run_results(ui, doc, &p, &mut shared.props_tab),
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
    let study = doc.result.as_ref().ok().and_then(|r| r.fault.clone());
    let project = &mut doc.project;
    section(ui, p, "Proyecto");
    property(ui, p, "Nombre", |ui| {
        changed |= ui.text_edit_singleline(&mut project.name).changed();
    });
    property(ui, p, "Perfil normativo", |ui| {
        ui.label("NOM-001-SEDE-2012");
    });
    section(ui, p, "Datos de la obra");
    ui.label(muted(p, "Aparecen en la memoria; no intervienen en el cálculo.").size(11.0));
    let info = &mut project.info;
    for (label, value, hint) in [
        ("Cliente", &mut info.client, "Propietario o razón social"),
        ("Ubicación", &mut info.location, "Domicilio o municipio"),
        ("Elaboró", &mut info.prepared_by, "Quien hizo el cálculo"),
        ("Responsable", &mut info.responsible, "Quien revisa y firma"),
        ("Cédula profesional", &mut info.license, "Del responsable"),
        ("Revisión", &mut info.revision, "A, 0, 1…"),
        ("Fecha", &mut info.date, "Vacía: la del día"),
    ] {
        property(ui, p, label, |ui| {
            changed |= ui
                .add(egui::TextEdit::singleline(value).hint_text(hint))
                .changed();
        });
    }
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
    section(ui, p, "Acometida y cortocircuito");
    changed |= source_inputs(ui, p, &mut project.source, study.as_ref());

    section(ui, p, "Criterios de diseño");
    changed |= criteria_inputs(ui, p, &mut project.settings);
    changed
}

/// Criterios de diseño del proyecto: límites de caída, ambiente, conductor,
/// canalización y tablas opcionales.
pub(crate) fn criteria_inputs(
    ui: &mut egui::Ui,
    p: &Palette,
    s: &mut ieestudio_project::Settings,
) -> bool {
    let mut changed = false;
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
    property(ui, p, "Calibre máximo", |ui| {
        egui::ComboBox::from_id_salt("calibre-maximo")
            .selected_text(size_label(&s.largest_single_size))
            .height(300.0)
            .show_ui(ui, |ui| {
                for c in Profile::nom_001_sede_2012()
                    .sizes
                    .iter()
                    .skip_while(|c| c.size != "1/0")
                {
                    let size = Size::new(&c.size);
                    let label = size_label(&size);
                    changed |= ui
                        .selectable_value(&mut s.largest_single_size, size, label)
                        .changed();
                }
            });
    })
    .on_hover_text("Calibre mayor que IEEStudio propone con un conductor por fase; arriba de él propone conductores en paralelo.");
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

pub(crate) fn feeder_inputs(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette, i: usize) -> bool {
    let mut changed = false;
    let fault = doc.panel_result(i).and_then(|r| r.fault.clone());
    let motors = doc
        .result
        .as_ref()
        .ok()
        .and_then(|r| r.fault.as_ref())
        .and_then(|f| f.motors.clone());
    // El interruptor del alimentador está en el tablero de donde sale.
    let upstream_ka = doc.project.panels[i]
        .parent
        .as_ref()
        .and_then(|id| doc.project.panels.iter().find(|x| &x.id == id))
        .and_then(|x| x.board.interrupting_ka);
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
    egui::CollapsingHeader::new(
        RichText::new("Interruptor del alimentador (catálogo)").color(p.ink2),
    )
    .id_salt(("interruptor-alimentador", i))
    .show(ui, |ui| {
        changed |= breaker_inputs(
            ui,
            p,
            &format!("alim{i}"),
            &mut panel.feeder.overrides,
            upstream_ka,
        );
    });

    section(ui, p, "Tablero (catálogo)");
    changed |= board_inputs(ui, p, &mut panel.board);
    if let Some(bus) = &fault {
        ui.add_space(4.0);
        fault_box(
            ui,
            p,
            &format!("Corriente de falla en las barras de {}", panel.id),
            bus,
            motors.as_ref(),
        );
    }

    section(ui, p, "Transformador");
    let mut has_transformer = panel.transformer.is_some();
    if ui
        .checkbox(&mut has_transformer, "Se alimenta por un transformador")
        .on_hover_text("El alimentador llega al primario; el tablero y sus circuitos quedan con la tensión del secundario (artículo 450).")
        .changed()
    {
        panel.transformer = has_transformer.then(|| ieestudio_project::Transformer {
            brand: Default::default(),
            connection: Default::default(),
            mounting: Default::default(),
            kva: 45.0,
            impedance_percent: 3.0,
            secondary: ieestudio_core::System::WYE_220_127,
            secondary_protection: true,
            secondary_length_m: 3.0,
            secondary_has_neutral: true,
            secondary_overrides: Overrides::default(),
        });
        changed = true;
    }
    let mut show_secondary = false;
    if let Some(t) = panel.transformer.as_mut() {
        property(ui, p, "Potencia", |ui| {
            changed |= number(ui, &mut t.kva, 1.0, 0.5, 10_000.0, " kVA");
        });
        property(ui, p, "Impedancia", |ui| {
            changed |= number(ui, &mut t.impedance_percent, 0.1, 0.5, 20.0, " %");
        });
        property(ui, p, "Secundario", |ui| {
            changed |= combo(ui, "sistema-secundario", &mut t.secondary, &SYSTEMS);
        });
        property(ui, p, "Conexión", |ui| {
            changed |= optional_combo(
                ui,
                ("conexion-transformador", i),
                &mut t.connection,
                &ieestudio_project::Connection::ALL,
                ieestudio_project::Connection::label,
            );
        });
        property(ui, p, "Montaje", |ui| {
            changed |= optional_combo(
                ui,
                ("montaje-transformador", i),
                &mut t.mounting,
                &ieestudio_project::Mounting::ALL,
                ieestudio_project::Mounting::label,
            );
        });
        changed |= text_property(ui, p, "Marca", &mut t.brand, "Opcional");
        changed |= ui
            .checkbox(&mut t.secondary_protection, "Interruptor en el secundario")
            .on_hover_text("Con protección en el secundario (máximo 125 %) el primario puede protegerse hasta el 250 % (Tabla 450-3(b)).")
            .changed();
        if t.secondary_protection {
            let board_ka = panel.board.interrupting_ka;
            egui::CollapsingHeader::new(
                RichText::new("Interruptor del secundario (catálogo)").color(p.ink2),
            )
            .id_salt(("interruptor-secundario", i))
            .show(ui, |ui| {
                changed |= breaker_inputs(
                    ui,
                    p,
                    &format!("sec{i}"),
                    &mut t.secondary_overrides,
                    board_ka,
                );
            });
        }
        if ui
            .button(format!("{} Conductores del secundario", icon::ARROW_RIGHT))
            .clicked()
        {
            show_secondary = true;
        }
    }

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
    if show_secondary {
        doc.selection = Selection::Secondary(i);
    }
    changed
}

/// Conductores del secundario de un transformador hasta su tablero.
fn secondary_inputs(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette, i: usize) -> bool {
    let mut changed = false;
    let settings = doc.project.settings.clone();
    let Some(t) = doc.project.panels[i].transformer.as_mut() else {
        return false;
    };
    section(ui, p, "Secundario del transformador");
    ui.label(
        muted(
            p,
            "Del transformador al tablero. Su interruptor protege estos conductores; la caída de tensión del tablero se cuenta desde aquí.",
        )
        .size(11.0),
    );
    property(ui, p, "Longitud", |ui| {
        changed |= number(ui, &mut t.secondary_length_m, 0.5, 0.0, 5000.0, " m");
    });
    changed |= ui
        .checkbox(&mut t.secondary_has_neutral, "Lleva neutro")
        .changed();
    changed |= decisions(ui, p, &mut t.secondary_overrides, "secundario", &settings);
    changed
}

/// Tipo de inmueble o de área de la Tabla 220-12. `none` es el texto de la
/// opción vacía: «Sin elegir» o el tipo del proyecto.
pub(crate) fn occupancy_combo(
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
    circuit_form(ui, doc, p, i, c, false)
}

/// Datos de un circuito. En la vista guiada solo quedan a la vista los
/// esenciales; el identificador, el factor de potencia, la carga continua,
/// el neutro y las decisiones de diseño van en «Opciones avanzadas».
pub(crate) fn circuit_form(
    ui: &mut egui::Ui,
    doc: &mut Doc,
    p: &Palette,
    i: usize,
    c: usize,
    guided: bool,
) -> bool {
    let mut changed = false;
    let settings = doc.project.settings.clone();
    let id_problem = doc.circuit_id_problem(i, c);
    // Potencia calculada de un motor o compresor, para mostrarla.
    let computed_w = doc
        .panel_result(i)
        .and_then(|r| r.circuits.get(c))
        .map(|r| r.load_w);
    // Lo que se necesita para la conexión, antes de tomar el circuito.
    let system = doc.bus_system(i);
    let panel_kind = doc.project.panels[i].kind;
    let phases_for = |kind| doc.default_phases(i, kind);
    let suggested = [
        phases_for(CircuitKind::LineToNeutral),
        phases_for(CircuitKind::LineToLine),
        phases_for(CircuitKind::ThreePhase),
    ];
    let focus = take_focus_description(ui.ctx());
    let board_ka = doc.project.panels[i].board.interrupting_ka;
    let circuit = &mut doc.project.panels[i].circuits[c];
    if !guided {
        section(ui, p, "Circuito");
        changed |= id_field(ui, p, &mut circuit.id, id_problem);
    } else if let Some(problem) = &id_problem {
        ui.label(RichText::new(format!("{} {problem}", icon::WARNING)).color(p.warn));
    }
    property(ui, p, "Descripción", |ui| {
        let response = ui.text_edit_singleline(&mut circuit.name);
        if focus {
            response.request_focus();
            // Se selecciona todo para escribir encima del nombre propuesto.
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                let all = egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(circuit.name.chars().count()),
                );
                state.cursor.set_char_range(Some(all));
                state.store(ui.ctx(), response.id);
            }
        }
        changed |= response.changed();
    });
    section(ui, p, "Carga");
    property(ui, p, "Tipo de carga", |ui| {
        let mut kind = circuit.load.kind;
        if combo(ui, ("tipo-carga", c), &mut kind, &LOAD_KINDS) {
            // Al elegir motor o aire acondicionado se piden sus datos.
            set_load_kind(&mut circuit.load, kind);
            changed = true;
        }
    })
    .on_hover_text(load_kind_hint(circuit.load.kind));
    // Primero los datos del equipo, que es lo que el tipo pide.
    let three_phase = circuit.kind == CircuitKind::ThreePhase;
    if let (LoadKind::Motor, Some(motor)) = (circuit.load.kind, circuit.load.motor.as_mut()) {
        changed |= motor_inputs(ui, p, motor, three_phase, c);
    }
    if let (LoadKind::AirConditioning, Some(plate)) =
        (circuit.load.kind, circuit.load.compressor.as_mut())
    {
        changed |= compressor_inputs(ui, p, plate);
    }
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
        property(ui, p, "Potencia equivalente", |ui| {
            ui.label(
                RichText::new(computed_w.map_or("—".to_string(), |w| format!("{w:.0} W")))
                    .monospace(),
            )
            .on_hover_text(
                "No se captura: sale de la corriente del equipo, la de tabla del motor o la de placa del compresor.",
            );
        });
    } else {
        property(ui, p, "Potencia", |ui| {
            changed |= number(ui, &mut circuit.load.power_w, 10.0, 0.0, 5.0e6, " W");
        });
    }
    if !guided {
        changed |= load_details(ui, p, &mut circuit.load, equipment);
    }
    section(ui, p, "Conexión");
    property(ui, p, "Conexión", |ui| {
        if connection(ui, &mut circuit.kind, system, panel_kind) {
            // Las fases menos cargadas del tablero para la conexión nueva.
            circuit.phases = match circuit.kind {
                CircuitKind::LineToNeutral => suggested[0].clone(),
                CircuitKind::LineToLine => suggested[1].clone(),
                CircuitKind::ThreePhase => suggested[2].clone(),
            };
            changed = true;
        }
    })
    .on_hover_text("Fases y tensión a las que se conecta la carga. Solo se pueden elegir las que tiene el tablero.");
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
    if !guided && circuit.kind != ieestudio_core::CircuitKind::LineToNeutral {
        changed |= ui
            .checkbox(&mut circuit.has_neutral, "Lleva neutro")
            .changed();
    }
    section(ui, p, "Instalación");
    property(ui, p, "Longitud", |ui| {
        changed |= number(ui, &mut circuit.length_m, 0.5, 0.0, 5000.0, " m");
    });
    if !guided {
        section(ui, p, "Interruptor (catálogo)");
        changed |= breaker_inputs(ui, p, &format!("c{c}"), &mut circuit.overrides, board_ka);
        changed |= decisions(ui, p, &mut circuit.overrides, &format!("c{c}"), &settings);
        return changed;
    }
    ui.add_space(6.0);
    egui::CollapsingHeader::new(
        RichText::new("Opciones avanzadas").color(p.ink2),
    )
    .id_salt(("avanzadas", i, c))
    .show(ui, |ui| {
        ui.label(
            muted(
                p,
                "Identificador, factor de potencia, carga continua, neutro, cable o interruptor fijados, material y tubería.",
            )
            .size(11.0),
        );
        changed |= id_field(ui, p, &mut circuit.id, None);
        changed |= load_details(ui, p, &mut circuit.load, equipment);
        if circuit.kind != ieestudio_core::CircuitKind::LineToNeutral {
            changed |= ui
                .checkbox(&mut circuit.has_neutral, "Lleva neutro")
                .changed();
        }
        changed |= decisions(ui, p, &mut circuit.overrides, &format!("c{c}"), &settings);
    });
    egui::CollapsingHeader::new(RichText::new("Interruptor (catálogo)").color(p.ink2))
        .id_salt(("catalogo", i, c))
        .show(ui, |ui| {
            ui.label(
                muted(
                    p,
                    "Marco, capacidad interruptiva, modelo, marca y posición: aparecen en el unifilar.",
                )
                .size(11.0),
            );
            changed |= breaker_inputs(ui, p, &format!("c{c}"), &mut circuit.overrides, board_ka);
        });
    changed
}

/// Tensión de una conexión en el sistema del tablero, si la tiene.
fn connection_voltage(kind: CircuitKind, system: ieestudio_core::System) -> Option<f64> {
    match kind {
        CircuitKind::LineToNeutral => system.line_to_neutral_v,
        CircuitKind::LineToLine | CircuitKind::ThreePhase => system.line_to_line_v,
    }
}

/// Conexión como botones con su tensión: `1F · 127 V`, `2F · 220 V`,
/// `3F · 220 V`. Las que el tablero no tiene quedan desactivadas.
fn connection(
    ui: &mut egui::Ui,
    kind: &mut CircuitKind,
    system: ieestudio_core::System,
    panel: CircuitKind,
) -> bool {
    let mut changed = false;
    let rank = |k: CircuitKind| match k {
        CircuitKind::LineToNeutral => 1,
        CircuitKind::LineToLine => 2,
        CircuitKind::ThreePhase => 3,
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for option in [
            CircuitKind::LineToNeutral,
            CircuitKind::LineToLine,
            CircuitKind::ThreePhase,
        ] {
            let voltage = connection_voltage(option, system);
            let text = match voltage {
                Some(v) => format!("{}F · {v:.0} V", rank(option)),
                None => format!("{}F", rank(option)),
            };
            let possible = voltage.is_some() && rank(option) <= rank(panel);
            let on = *kind == option;
            let response = ui
                .add_enabled(possible || on, egui::Button::selectable(on, text))
                .on_disabled_hover_text("El tablero no tiene esta conexión.");
            if response.clicked() && !on {
                *kind = option;
                changed = true;
            }
        }
    });
    changed
}

/// Pide que la próxima ficha de circuito ponga el cursor en la descripción,
/// como después de agregar un circuito.
pub fn focus_description(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("enfocar-descripcion"), true));
}

fn take_focus_description(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(egui::Id::new("enfocar-descripcion")))
        .unwrap_or(false)
}

/// Lista de opciones con «Sin dato», para un valor opcional.
fn optional_combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    value: &mut Option<T>,
    options: &[T],
    label: impl Fn(T) -> &'static str,
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.map_or("Sin dato", &label))
        .show_ui(ui, |ui| {
            changed |= ui.selectable_value(value, None, "Sin dato").changed();
            for &option in options {
                changed |= ui
                    .selectable_value(value, Some(option), label(option))
                    .changed();
            }
        });
    changed
}

fn text_property(
    ui: &mut egui::Ui,
    p: &Palette,
    label: &str,
    value: &mut String,
    hint: &str,
) -> bool {
    let mut changed = false;
    property(ui, p, label, |ui| {
        changed |= ui
            .add(
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .desired_width(150.0),
            )
            .changed();
    });
    changed
}

/// Acometida: suministradora, frecuencia, medidor, subestación y corriente
/// de falla (requisitos I.1 e I.2 del unifilar y decisión 0023).
pub(crate) fn source_inputs(
    ui: &mut egui::Ui,
    p: &Palette,
    source: &mut ieestudio_project::Source,
    study: Option<&ieestudio_project::fault::FaultStudy>,
) -> bool {
    let mut changed = false;
    changed |= text_property(ui, p, "Suministradora", &mut source.supplier, "CFE");
    property(ui, p, "Frecuencia", |ui| {
        changed |= optional_number(ui, &mut source.frequency_hz, 60.0, " Hz");
    });
    changed |= ui
        .checkbox(&mut source.meter, "Medidor en el punto de conexión")
        .changed();
    property(ui, p, "Falla disponible", |ui| {
        changed |= optional_number(ui, &mut source.fault.available_ka, 10.0, " kA");
    })
    .on_hover_text("Corriente de cortocircuito en el punto de conexión, si la da la suministradora. Manda sobre la que se estima con la subestación.");
    let mut has_substation = source.substation.is_some();
    if ui
        .checkbox(&mut has_substation, "Se alimenta de una subestación (media tensión)")
        .on_hover_text("El transformador que alimenta la instalación, propio o de la suministradora. Con su potencia e impedancia se estima la corriente de falla por bus infinito.")
        .changed()
    {
        source.substation = has_substation.then(|| ieestudio_project::Substation {
            impedance_from_table: Default::default(),
            kva: 112.5,
            impedance_percent: 3.0,
            primary_kv: Some(13.2),
            connection: None,
            mounting: None,
            brand: String::new(),
        });
        changed = true;
    }
    if let Some(sub) = source.substation.as_mut() {
        property(ui, p, "Potencia", |ui| {
            changed |= number(ui, &mut sub.kva, 1.0, 1.0, 100_000.0, " kVA");
        });
        if !sub.impedance_from_table {
            property(ui, p, "Impedancia", |ui| {
                changed |= number(ui, &mut sub.impedance_percent, 0.1, 0.5, 20.0, " %");
            })
            .on_hover_text("La de placa. Se reduce por la tolerancia de fabricación antes de calcular la falla.");
        }
        changed |= ui
            .checkbox(&mut sub.impedance_from_table, "Sin dato de placa: usar la mínima de la norma")
            .on_hover_text("Toma la impedancia mínima de la Tabla 11 de la NMX-J-116-ANCE para el tipo, la potencia y la tensión del transformador. Ya incluye la tolerancia y da la falla mayor.")
            .changed();
        property(ui, p, "Primario", |ui| {
            changed |= optional_number(ui, &mut sub.primary_kv, 13.2, " kV");
        });
        property(ui, p, "Conexión", |ui| {
            changed |= optional_combo(
                ui,
                "conexion-subestacion",
                &mut sub.connection,
                &ieestudio_project::Connection::ALL,
                ieestudio_project::Connection::label,
            );
        });
        property(ui, p, "Montaje", |ui| {
            changed |= optional_combo(
                ui,
                "montaje-subestacion",
                &mut sub.mounting,
                &ieestudio_project::Mounting::ALL,
                ieestudio_project::Mounting::label,
            );
        });
        changed |= text_property(ui, p, "Marca", &mut sub.brand, "Opcional");
    }
    if source
        .substation
        .as_ref()
        .is_some_and(|s| s.impedance_from_table)
    {
        let profile = Profile::nom_001_sede_2012();
        match ieestudio_project::fault::substation_impedance(profile, source) {
            Ok(z) => {
                ui.label(
                    muted(
                        p,
                        format!(
                            "Impedancia de la tabla: {z} % ({}).",
                            profile.clause(ieestudio_rules::Rule::TransformerImpedanceLimits)
                        ),
                    )
                    .size(11.5),
                );
            }
            Err(why) => {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{} Sin impedancia: {why}.", icon::WARNING))
                            .color(p.warn)
                            .size(12.0),
                    )
                    .wrap(),
                );
            }
        }
    }
    if let Some(study) = study {
        ui.add_space(6.0);
        fault_box(
            ui,
            p,
            "Corriente de falla en el punto de conexión",
            &study.source,
            study.motors.as_ref(),
        );
    } else {
        ui.label(
            muted(
                p,
                "Sin la falla de la suministradora ni la subestación, la capacidad interruptiva queda fuera de alcance.",
            )
            .size(11.0),
        );
    }
    changed
}

/// Recuadro con la corriente de falla de unas barras: de dónde sale, la
/// aportación de los motores a la vista y el procedimiento.
pub(crate) fn fault_box(
    ui: &mut egui::Ui,
    p: &Palette,
    title: &str,
    bus: &ieestudio_project::fault::BusFault,
    motors: Option<&ieestudio_project::fault::MotorContribution>,
) {
    use ieestudio_project::fault::FaultOrigin;
    egui::Frame::new()
        .fill(p.bg)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(11.5).color(p.ink2));
            ui.label(
                RichText::new(format!("{} {:.2} kA", icon::LIGHTNING, bus.total_ka))
                    .size(16.0)
                    .strong(),
            );
            let origin = match &bus.origin {
                FaultOrigin::Supplier => "Dato de la suministradora.".to_string(),
                FaultOrigin::Substation => {
                    "Bus infinito del transformador de la acometida: la red se toma sin impedancia, así que es la corriente mayor posible.".to_string()
                }
                FaultOrigin::Transformer { panel } => format!(
                    "Bus infinito del transformador de {panel}: es la corriente mayor posible en su secundario."
                ),
            };
            ui.add(egui::Label::new(muted(p, origin).size(11.5)).wrap());
            if let Some(m) = motors.filter(|_| bus.motor_ka > 0.0) {
                // La aportación de los motores, a la vista.
                let response = egui::Frame::new()
                    .fill(p.accent.gamma_multiply(0.10))
                    .stroke(egui::Stroke::new(1.0, p.accent.gamma_multiply(0.5)))
                    .corner_radius(5)
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new(format!(
                                "{} Incluye la aportación de {}",
                                icon::ENGINE,
                                if m.motors.len() == 1 {
                                    "1 motor".to_string()
                                } else {
                                    format!("{} motores", m.motors.len())
                                }
                            ))
                            .color(p.accent_text)
                            .strong(),
                        );
                        let currents: Vec<String> =
                            m.motors.iter().map(|s| format!("{:.2}", s.full_load_a)).collect();
                        ui.add(
                            egui::Label::new(
                                RichText::new(format!(
                                    "{} × ({}) A = {:.1} A → {:.2} + {:.3} = {:.2} kA",
                                    m.multiplier,
                                    currents.join(" + "),
                                    m.total_a,
                                    bus.base_ka,
                                    bus.motor_ka,
                                    bus.total_ka
                                ))
                                .monospace()
                                .size(11.5),
                            )
                            .wrap(),
                        );
                        ui.label(
                            muted(
                                p,
                                "Los motores en marcha también alimentan la falla. Se suma su corriente a plena carga multiplicada, en todos los tableros.",
                            )
                            .size(11.0),
                        );
                    })
                    .response;
                let list: Vec<String> = m
                    .motors
                    .iter()
                    .map(|s| format!("• {}: {:.2} A", s.label, s.full_load_a))
                    .collect();
                response.on_hover_text(format!(
                    "Motores y compresores que aportan:\n{}\n\nMultiplicador {} (perfil). Es una cota superior: un motor lejano aporta menos.",
                    list.join("\n"),
                    m.multiplier
                ));
            }
            egui::CollapsingHeader::new(RichText::new("Cómo se calculó").size(12.0).color(p.ink2))
                .id_salt(("falla", title))
                .show(ui, |ui| {
                    crate::widgets::steps_ui(ui, p, ("pasos-falla", bus.total_ka.to_bits()), &bus.steps, None);
                });
        });
}

/// Datos de catálogo de un interruptor y su posición en el tablero.
pub(crate) fn breaker_inputs(
    ui: &mut egui::Ui,
    p: &Palette,
    id: &str,
    ov: &mut Overrides,
    board_ka: Option<f64>,
) -> bool {
    let mut changed = false;
    let data = &mut ov.breaker;
    property(ui, p, "Marco", |ui| {
        changed |= optional_number(ui, &mut data.frame_a, 100.0, " A");
    })
    .on_hover_text("Amperes de marco (AM). Los de disparo (AD) son los del interruptor que propone o verifica IEEStudio.");
    property(ui, p, "Cap. interruptiva", |ui| {
        changed |= optional_number(ui, &mut data.interrupting_ka, 10.0, " kA");
        if data.interrupting_ka.is_none() {
            if let Some(ka) = board_ka {
                ui.label(muted(p, format!("{ka} kA del tablero")).size(11.0));
            }
        }
    })
    .on_hover_text("Se compara con la corriente de falla de las barras de donde sale. Sin dato toma la del tablero.");
    changed |= text_property(ui, p, "Modelo", &mut data.model, "QOB120");
    changed |= text_property(ui, p, "Marca", &mut data.brand, "Opcional");
    changed |= text_property(
        ui,
        p,
        "Ajuste",
        &mut data.trip_range,
        "Electrónicos: 0.4–1.0 In",
    );
    let mut position = ov.position.clone().unwrap_or_default();
    property(ui, p, "Posición", |ui| {
        if ui
            .add(
                egui::TextEdit::singleline(&mut position)
                    .hint_text("La propone IEEStudio")
                    .desired_width(110.0)
                    .id_salt(("posicion", id)),
            )
            .changed()
        {
            ov.position = (!position.trim().is_empty()).then(|| position.trim().to_string());
            changed = true;
        }
    })
    .on_hover_text("Espacios del tablero como en el unifilar: 1, 2,4 o 13,15,17. Vacía, IEEStudio la propone según las fases.");
    changed
}

/// Datos de catálogo del tablero: barras, capacidad interruptiva, modelo y marca.
fn board_inputs(ui: &mut egui::Ui, p: &Palette, board: &mut ieestudio_project::Board) -> bool {
    let mut changed = false;
    property(ui, p, "Barras", |ui| {
        changed |= optional_number(ui, &mut board.bus_a, 225.0, " A");
    });
    property(ui, p, "Cap. interruptiva", |ui| {
        changed |= optional_number(ui, &mut board.interrupting_ka, 10.0, " kA");
    })
    .on_hover_text(
        "La de sus interruptores. La toma cada interruptor del tablero que no tiene la suya.",
    );
    changed |= text_property(ui, p, "Modelo", &mut board.model, "NQ304AB225S");
    changed |= text_property(ui, p, "Marca", &mut board.brand, "Opcional");
    changed
}

/// Identificador de un circuito, con el aviso si se repite.
fn id_field(ui: &mut egui::Ui, p: &Palette, id: &mut String, problem: Option<&str>) -> bool {
    let mut changed = false;
    property(ui, p, "ID", |ui| {
        changed |= ui
            .add(egui::TextEdit::singleline(id).desired_width(80.0))
            .changed();
    });
    if let Some(problem) = problem {
        ui.label(RichText::new(format!("{} {problem}", icon::WARNING)).color(p.warn));
    }
    changed
}

/// Factor de potencia y carga continua.
fn load_details(
    ui: &mut egui::Ui,
    p: &Palette,
    load: &mut ieestudio_project::Load,
    equipment: bool,
) -> bool {
    let mut changed = false;
    property(ui, p, "Factor de potencia", |ui| {
        changed |= ui
            .add(
                egui::DragValue::new(&mut load.power_factor)
                    .speed(0.005)
                    .range(0.1..=1.0)
                    .fixed_decimals(2),
            )
            .changed();
    });
    if !equipment {
        changed |= ui
            .checkbox(&mut load.continuous, "Carga continua (3 h o más)")
            .changed();
    }
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

/// Decisiones opcionales que sustituyen lo que IEEStudio propondría.
fn decisions(
    ui: &mut egui::Ui,
    p: &Palette,
    ov: &mut Overrides,
    id: &str,
    project: &ieestudio_project::Settings,
) -> bool {
    let mut changed = false;
    section(ui, p, "Decisiones de diseño (opcional)");
    ui.label(
        muted(
            p,
            "Si no fijas nada, IEEStudio propone el menor valor que cumple.",
        )
        .size(11.0),
    );
    property(ui, p, "Calibre", |ui| {
        let text = ov
            .size
            .as_ref()
            .map_or_else(|| proposed_text(ui), |s| size_label(s).into());
        egui::ComboBox::from_id_salt(("calibre", id))
            .selected_text(text)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut ov.size, None, "Lo propone IEEStudio")
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
        if ov.size.is_some() {
            changed |= reset_button(ui, "Que lo proponga IEEStudio", || ov.size = None);
        }
    });
    property(ui, p, "Interruptor", |ui| {
        let text = ov
            .breaker_a
            .map_or_else(|| proposed_text(ui), |b| format!("{b:.0} A").into());
        egui::ComboBox::from_id_salt(("interruptor", id))
            .selected_text(text)
            .height(320.0)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut ov.breaker_a, None, "Lo propone IEEStudio")
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
        if ov.breaker_a.is_some() {
            changed |= reset_button(ui, "Que lo proponga IEEStudio", || ov.breaker_a = None);
        }
    })
    .on_hover_text("IEEStudio verifica el interruptor que fijes; no lo cambia.");
    property(ui, p, "Conductores por fase", |ui| {
        let text = ov
            .conductors_per_phase
            .map_or_else(|| proposed_text(ui), |n| n.to_string().into());
        egui::ComboBox::from_id_salt(("por-fase", id))
            .selected_text(text)
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut ov.conductors_per_phase, None, "Lo propone IEEStudio")
                    .changed();
                for n in 1..=8u32 {
                    changed |= ui
                        .selectable_value(&mut ov.conductors_per_phase, Some(n), n.to_string())
                        .changed();
                }
            });
        if ov.conductors_per_phase.is_some() {
            changed |= reset_button(ui, "Que lo proponga IEEStudio", || {
                ov.conductors_per_phase = None
            });
        }
    })
    .on_hover_text(
        "IEEStudio propone conductores en paralelo cuando uno solo no cumple, no cabe en un tubo o rebasa el calibre máximo de los criterios.",
    );
    changed |= ui
        .checkbox(
            &mut ov.single_raceway,
            "Paralelos en un solo tubo",
        )
        .on_hover_text("Por omisión cada juego de conductores en paralelo lleva su propio tubo. En un solo tubo se aplica el ajuste por agrupamiento.")
        .changed();
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

/// Verificaciones que respaldan cada sección de Resultados, por su nombre.
const TRACE: [(&str, &[&str]); 4] = [
    (
        "Protección",
        &[
            "Protección",
            "Protección del primario",
            "Protección del secundario",
        ],
    ),
    ("Conductor", &["Ampacidad"]),
    ("Tubería", &["Tubería"]),
    ("Caída de tensión", &["Caída de tensión", "Caída acumulada"]),
];

/// Encabezado de sección con su trazabilidad a la derecha: el estado de las
/// verificaciones que la respaldan y los artículos que las piden. Al tocar
/// los artículos se abre la pestaña Normativa con el detalle.
fn traced_section(
    ui: &mut egui::Ui,
    p: &Palette,
    title: &str,
    item: &RunResult,
    tab: &mut PropsTab,
) {
    let names = TRACE
        .iter()
        .find(|(t, _)| *t == title)
        .map_or(&[][..], |(_, n)| *n);
    let backing: Vec<&ieestudio_project::Check> = item
        .checks
        .iter()
        .filter(|c| names.contains(&c.name.as_str()))
        .collect();
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title.to_uppercase())
                .size(10.5)
                .color(p.ink3)
                .family(crate::theme::semibold()),
        );
        if backing.is_empty() {
            return;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let status = crate::widgets::worst(backing.iter().map(|c| c.status));
            let mut clauses: Vec<String> = Vec::new();
            for c in &backing {
                for r in &c.references {
                    if let Some(clause) = &r.clause {
                        if !clauses.contains(clause) {
                            clauses.push(clause.clone());
                        }
                    }
                }
            }
            let cite = if clauses.is_empty() {
                "ver normativa".to_string()
            } else {
                clauses.join("; ")
            };
            let detail = backing
                .iter()
                .map(|c| format!("{} · {}: {}", c.name, c.status.label(), c.detail))
                .collect::<Vec<_>>()
                .join("\n");
            let link = ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("{} {}", crate::widgets::status_icon(status), cite))
                            .size(11.0)
                            .color(crate::widgets::status_color(p, status)),
                    )
                    .frame(false)
                    .truncate(),
                )
                .on_hover_text(format!(
                    "{detail}\n\nClic para ver la verificación completa en Normativa."
                ));
            if link.clicked() {
                *tab = PropsTab::Normative;
            }
        });
    });
    ui.add_space(1.0);
}

fn run_results(ui: &mut egui::Ui, doc: &mut Doc, p: &Palette, tab: &mut PropsTab) -> bool {
    let Some(item) = doc.selected_run().cloned() else {
        return false;
    };
    ui.label(
        muted(
            p,
            format!(
                "{} Calculado por IEEStudio con NOM-001-SEDE-2012. Solo lectura.",
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
    traced_section(ui, p, "Protección", &item, tab);
    result_row(
        ui,
        p,
        "Interruptor",
        format!("{}P-{:.0} A", run.poles, run.breaker_a),
        calc,
    );
    traced_section(ui, p, "Conductor", &item, tab);
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
                        "Fijaste {}. IEEStudio propondría {} (lo determina {gov}).",
                        size_label(&run.size),
                        size_label(rec)
                    ));
                    if ui.button("Usar el propuesto").clicked() {
                        set_size = Some(None);
                    }
                }
                (true, _, _) => {
                    ui.label(format!(
                        "Fijaste {}; coincide con lo que propone IEEStudio.",
                        size_label(&run.size)
                    ));
                    if ui.button("Dejar que lo proponga IEEStudio").clicked() {
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

    traced_section(ui, p, "Tubería", &item, tab);
    let conduit = run.conduit.as_ref().map_or("—".to_string(), |c| {
        format!("{} ({}\")", c.label(), c.trade_size)
    });
    result_row(ui, p, "Tubo", conduit, calc);
    if let Some(c) = &run.conduit {
        result_row(ui, p, "Ocupación", format!("{:.1} %", c.fill_percent), calc);
    }
    traced_section(ui, p, "Caída de tensión", &item, tab);
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

    // Lo demás que se verificó (carga, transformador, datos del equipo…),
    // también con su estado y su referencia.
    let traced: Vec<&str> = TRACE.iter().flat_map(|(_, n)| n.iter().copied()).collect();
    let others: Vec<&ieestudio_project::Check> = item
        .checks
        .iter()
        .filter(|c| !traced.contains(&c.name.as_str()))
        .collect();
    if !others.is_empty() {
        section(ui, p, "Otras verificaciones");
        for c in others {
            ui.horizontal(|ui| {
                crate::widgets::fixed_label(ui, p, &c.name);
                ui.label(status_text(p, c.status).size(12.0))
                    .on_hover_text(&c.detail);
            });
            if let Some(cite) = cite_all(&c.references) {
                ui.label(
                    RichText::new(format!("{} {cite}", icon::BOOK_OPEN))
                        .size(10.5)
                        .color(p.ink3),
                );
            }
        }
    }

    if let Some(size) = set_size {
        match doc.selection {
            Selection::Feeder(i) => doc.project.panels[i].feeder.overrides.size = size,
            Selection::Secondary(i) => {
                if let Some(t) = doc.project.panels[i].transformer.as_mut() {
                    t.secondary_overrides.size = size;
                }
            }
            Selection::Circuit(i, c) => doc.project.panels[i].circuits[c].overrides.size = size,
            Selection::Project => {}
        }
        return true;
    }
    false
}

/// Carga del alimentador por grupo: de lo conectado a lo calculado.
fn demand_results(ui: &mut egui::Ui, p: &Palette, panel: &ieestudio_project::PanelResult) {
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
    if !result.verdict.out_of_scope.is_empty() {
        section(ui, p, "Fuera de alcance");
        ui.label(
            muted(
                p,
                "La norma lo pide, pero IEEStudio no lo verifica. Resuélvelo con otro estudio antes de firmar.",
            )
            .size(11.0),
        );
        for item in &result.verdict.out_of_scope {
            ui.label(RichText::new(format!("{} {}", icon::INFO, item.label())).color(p.ink2));
            ui.label(muted(p, crate::widgets::capitalize(&item.resolve_with)).size(11.0));
            ui.add_space(4.0);
        }
    }
    section(ui, p, "Acometida");
    match &result.grounding_electrode {
        Some(gec) => {
            result_row(
                ui,
                p,
                "Al electrodo",
                format!("{} de cobre", size_label(&gec.size)),
                Source::Calculated,
            );
            ui.label(
                muted(
                    p,
                    format!(
                        "Tabla 250-66, para {}; de aluminio sería {}.",
                        gec.service,
                        size_label(&gec.aluminum_size)
                    ),
                )
                .size(11.0),
            );
        }
        None => {
            ui.label(muted(p, "Sin alimentador calculado desde la fuente.").size(11.0));
        }
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
    section(ui, p, "Reglas que aplica IEEStudio");
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
