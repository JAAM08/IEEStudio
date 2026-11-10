//! Capacidad interruptiva con la corriente de falla estimada por bus
//! infinito (decisión 0023).
//!
//! La corriente de falla en el punto de conexión es el dato de la
//! suministradora o, si no se tiene, la del transformador que alimenta la
//! instalación con la red sin impedancia. Un tablero alimentado por un
//! transformador del proyecto toma la de ese transformador. A todas se les
//! suma la aportación de todos los motores y compresores del proyecto. No se
//! descuenta la impedancia de los alimentadores. Con eso cada valor es una
//! cota superior: si un interruptor la soporta, cumple; si no, el resultado
//! no es concluyente y hace falta un estudio de cortocircuito.

use ieestudio_core::System;
use ieestudio_electrical::fault::{
    effective_impedance_pu, infinite_bus_current, motor_contribution, transformer_full_load_current,
};
use ieestudio_rules::{Profile, Rule};
use serde::Serialize;

use crate::calc::{Check, PanelResult, RunResult, Status};
use crate::model::{LoadKind, Project};
use crate::procedure::Step;

/// De dónde sale la corriente de falla de unas barras.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FaultOrigin {
    /// Dato de la suministradora en el punto de conexión.
    Supplier,
    /// Bus infinito del transformador que alimenta la instalación.
    Substation,
    /// Bus infinito del transformador del proyecto que alimenta el tablero.
    Transformer { panel: String },
}

/// Corriente de falla estimada en unas barras.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BusFault {
    /// Sin la aportación de motores, en kA.
    pub base_ka: f64,
    /// Aportación de los motores, en kA.
    pub motor_ka: f64,
    /// Lo que se compara con la capacidad interruptiva, en kA.
    pub total_ka: f64,
    pub origin: FaultOrigin,
    /// Procedimiento, para la interfaz y la memoria.
    pub steps: Vec<Step>,
}

/// Un motor o compresor que aporta a la falla.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MotorShare {
    /// `TG-1 · C-06 Bomba de agua`.
    pub label: String,
    /// Corriente a plena carga, en amperes.
    pub full_load_a: f64,
}

/// Aportación de los motores a la falla: la misma en todos los tableros.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MotorContribution {
    pub multiplier: f64,
    pub motors: Vec<MotorShare>,
    /// `multiplier · Σ corriente`, en amperes.
    pub total_a: f64,
}

/// Resultado del estudio para todo el proyecto.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FaultStudy {
    /// En el punto de conexión, donde sale el alimentador principal.
    pub source: BusFault,
    /// `None` si el proyecto no tiene motores ni compresores calculados.
    pub motors: Option<MotorContribution>,
    /// Fracción en que se redujo la impedancia de placa de los
    /// transformadores.
    pub impedance_tolerance: f64,
}

/// Si el sistema es trifásico y la tensión con la que se calcula la
/// corriente del transformador.
fn transformer_voltage(system: &System) -> Option<(f64, bool)> {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        // Monofásico de tres hilos: el transformador es de una fase a la
        // tensión entre fases.
        (Some(ll), Some(ln)) if (ll / ln - 2.0).abs() < 0.1 => Some((ll, false)),
        (Some(ll), _) => Some((ll, true)),
        (None, Some(ln)) => Some((ln, false)),
        (None, None) => None,
    }
}

fn steps_for_transformer(
    profile: &Profile,
    kva: f64,
    impedance_percent: f64,
    system: &System,
    what: &str,
    from_table: bool,
) -> Result<(f64, Vec<Step>), String> {
    let (voltage, three_phase) =
        transformer_voltage(system).ok_or_else(|| "el sistema no tiene tensión".to_string())?;
    // El mínimo de la tabla de la norma ya incluye la tolerancia.
    let tolerance = if from_table {
        0.0
    } else {
        profile.parameters.fault_impedance_tolerance
    };
    let full_load =
        transformer_full_load_current(kva, voltage, three_phase).map_err(|e| e.to_string())?;
    let z = effective_impedance_pu(impedance_percent, tolerance).map_err(|e| e.to_string())?;
    let icc = infinite_bus_current(full_load, z).map_err(|e| e.to_string())?;
    let refs = vec![profile.reference(Rule::AvailableFault)];
    let steps = vec![
        Step {
            title: format!("Corriente nominal del {what}"),
            formula: Some(if three_phase {
                "In = S / (√3 · V)".into()
            } else {
                "In = S / V".into()
            }),
            substitution: if three_phase {
                format!("In = {kva} kVA / (√3 · {voltage:.0} V) = {full_load:.2} A")
            } else {
                format!("In = {kva} kVA / {voltage:.0} V = {full_load:.2} A")
            },
            note: None,
            references: vec![],
        },
        if from_table {
            Step {
                title: "Impedancia para la falla".into(),
                formula: None,
                substitution: format!("Z = {impedance_percent} %, la mínima de la tabla"),
                note: Some(
                    "Sin dato de placa se toma la impedancia mínima de la norma de fabricación para su tipo, potencia y tensión, que ya incluye la tolerancia: da la falla mayor."
                        .into(),
                ),
                references: vec![profile.reference(Rule::TransformerImpedanceLimits)],
            }
        } else {
            Step {
                title: "Impedancia para la falla".into(),
                formula: Some("Z = Z placa · (1 − tolerancia)".into()),
                substitution: format!(
                    "Z = {impedance_percent} % · (1 − {tolerance:.2}) = {:.3} %",
                    z * 100.0
                ),
                note: Some(
                    "La impedancia real puede salir menor que la de placa; con la reducida la falla sale mayor."
                        .into(),
                ),
                references: refs.clone(),
            }
        },
        Step {
            title: "Falla por bus infinito".into(),
            formula: Some("Icc = In / Z".into()),
            substitution: format!(
                "Icc = {full_load:.2} A / {z:.5} = {icc:.0} A = {:.2} kA",
                icc / 1000.0
            ),
            note: Some(
                "La red se toma sin impedancia: es la corriente mayor posible en el secundario."
                    .into(),
            ),
            references: refs,
        },
    ];
    Ok((icc / 1000.0, steps))
}

/// Motores y compresores calculados del proyecto, con su corriente.
fn motor_shares(project: &Project, panels: &[PanelResult]) -> Vec<MotorShare> {
    let mut out = Vec::new();
    for panel in &project.panels {
        let Some(result) = panels.iter().find(|r| r.id == panel.id) else {
            continue;
        };
        for (circuit, item) in panel.circuits.iter().zip(&result.circuits) {
            if !matches!(
                circuit.load.kind,
                LoadKind::Motor | LoadKind::AirConditioning
            ) {
                continue;
            }
            if let Some(run) = &item.run {
                out.push(MotorShare {
                    label: format!("{} · {} {}", panel.id, circuit.id, circuit.name),
                    full_load_a: run.nominal_a,
                });
            }
        }
    }
    out
}

fn with_motors(
    profile: &Profile,
    base_ka: f64,
    origin: FaultOrigin,
    mut steps: Vec<Step>,
    motors: &Option<MotorContribution>,
) -> BusFault {
    let motor_ka = motors.as_ref().map_or(0.0, |m| m.total_a / 1000.0);
    if let Some(m) = motors {
        let currents: Vec<String> = m
            .motors
            .iter()
            .map(|s| format!("{:.2}", s.full_load_a))
            .collect();
        steps.push(Step {
            title: "Aportación de los motores".into(),
            formula: Some("Im = k · Σ Ipc".into()),
            substitution: format!(
                "Im = {} · ({}) A = {:.1} A = {motor_ka:.3} kA",
                m.multiplier,
                currents.join(" + "),
                m.total_a
            ),
            note: Some(format!(
                "{} en marcha alimentan la falla. Se suman todos en cada tablero, aunque un motor lejano aporte menos.",
                if m.motors.len() == 1 {
                    "Un motor".to_string()
                } else {
                    format!("{} motores", m.motors.len())
                }
            )),
            references: vec![profile.reference(Rule::MotorFaultContribution)],
        });
        steps.push(Step {
            title: "Corriente de falla con motores".into(),
            formula: Some("Icc total = Icc + Im".into()),
            substitution: format!(
                "Icc total = {base_ka:.2} + {motor_ka:.3} = {:.2} kA",
                base_ka + motor_ka
            ),
            note: None,
            references: vec![],
        });
    }
    BusFault {
        base_ka,
        motor_ka,
        total_ka: base_ka + motor_ka,
        origin,
        steps,
    }
}

/// Barras en el punto de conexión. `None` si no hay dato de la suministradora
/// ni de la subestación.
fn source_fault(
    profile: &Profile,
    project: &Project,
    motors: &Option<MotorContribution>,
) -> Option<BusFault> {
    let source = &project.source;
    if let Some(ka) = source.fault.available_ka.filter(|ka| *ka > 0.0) {
        let step = Step {
            title: "Corriente de falla de la suministradora".into(),
            formula: None,
            substitution: format!("Icc = {ka} kA en el punto de conexión"),
            note: Some("Dato de la suministradora; manda sobre cualquier estimación.".into()),
            references: vec![profile.reference(Rule::InterruptingRating)],
        };
        return Some(with_motors(
            profile,
            ka,
            FaultOrigin::Supplier,
            vec![step],
            motors,
        ));
    }
    let substation = source.substation.as_ref()?;
    let impedance = substation_impedance(profile, source).ok()?;
    let (ka, steps) = steps_for_transformer(
        profile,
        substation.kva,
        impedance,
        &source.system,
        "transformador de la acometida",
        substation.impedance_from_table,
    )
    .ok()?;
    Some(with_motors(
        profile,
        ka,
        FaultOrigin::Substation,
        steps,
        motors,
    ))
}

/// Impedancia con la que se calcula la falla de la subestación: la de placa
/// o, si se pidió, la mínima de la tabla de la norma de fabricación. El error
/// dice qué falta para encontrarla en la tabla.
pub fn substation_impedance(
    profile: &Profile,
    source: &crate::model::Source,
) -> Result<f64, String> {
    use crate::model::{Connection, Mounting};
    let Some(sub) = &source.substation else {
        return Err("no hay subestación".into());
    };
    if !sub.impedance_from_table {
        return Ok(sub.impedance_percent);
    }
    let three_phase = transformer_voltage(&source.system).is_some_and(|(_, t)| t)
        && sub.connection != Some(Connection::SinglePhase);
    let kind = match (three_phase, sub.mounting) {
        (false, _) => "single_phase",
        (true, Some(Mounting::Pole)) => "pole_three_phase",
        (true, Some(Mounting::Substation)) => "substation",
        (true, Some(_)) => {
            return Err(
                "la tabla solo cubre transformadores tipo poste y tipo subestación; captura la impedancia de placa".into(),
            )
        }
        (true, None) => return Err("falta el montaje del transformador".into()),
    };
    let kv = sub
        .primary_kv
        .ok_or_else(|| "falta la tensión del primario".to_string())?;
    profile
        .impedance_limits(kind, sub.kva, kv)
        .map(|(min, _)| min)
        .ok_or_else(|| {
            format!(
                "la tabla no tiene un transformador de {} kVA a {kv} kV de este tipo; captura la impedancia de placa",
                sub.kva
            )
        })
}

fn worse(a: Status, b: Status) -> Status {
    match (a, b) {
        (Status::Fails, _) | (_, Status::Fails) => Status::Fails,
        (Status::NotEvaluated, _) | (_, Status::NotEvaluated) => Status::NotEvaluated,
        _ => Status::Complies,
    }
}

/// Compara el interruptor de un tramo con la falla de las barras de donde sale.
fn check_breaker(
    profile: &Profile,
    item: &mut RunResult,
    breaker_ka: Option<f64>,
    bus: &BusFault,
    bus_name: &str,
) {
    if item.run.is_none() {
        return;
    }
    let mut rules = vec![Rule::InterruptingRating];
    match bus.origin {
        FaultOrigin::Supplier => {}
        _ => rules.push(Rule::AvailableFault),
    }
    if bus.motor_ka > 0.0 {
        rules.push(Rule::MotorFaultContribution);
    }
    let icc = bus.total_ka;
    let (status, detail) = match breaker_ka {
        Some(ka) if ka >= icc => (
            Status::Complies,
            format!(
                "{ka} kA ≥ {icc:.2} kA de falla en {bus_name}; cumple con un método conservador (bus infinito)"
            ),
        ),
        Some(ka) => (
            Status::NotEvaluated,
            format!(
                "no concluyente: {ka} kA es menor que los {icc:.2} kA estimados en {bus_name} por bus infinito, que es una cota superior; requiere estudio de cortocircuito"
            ),
        ),
        None => (
            Status::NotEvaluated,
            format!(
                "falta la capacidad interruptiva del interruptor o la del tablero; la falla estimada en {bus_name} es de {icc:.2} kA"
            ),
        ),
    };
    let references = rules.iter().map(|r| profile.reference(*r)).collect();
    item.checks.push(Check {
        name: "Capacidad interruptiva".into(),
        status,
        detail,
        references,
    });
    item.status = worse(item.status, status);
}

/// Hace el estudio y agrega la verificación de capacidad interruptiva a cada
/// tramo. `None` si no hay datos de la fuente: entonces la capacidad
/// interruptiva queda fuera de alcance, como en la decisión 0018.
pub(crate) fn study(
    profile: &Profile,
    project: &Project,
    panels: &mut [PanelResult],
) -> Option<FaultStudy> {
    let shares = motor_shares(project, panels);
    let motors = (!shares.is_empty()).then(|| {
        let multiplier = profile.parameters.fault_motor_multiplier;
        let currents: Vec<f64> = shares.iter().map(|s| s.full_load_a).collect();
        MotorContribution {
            multiplier,
            total_a: motor_contribution(&currents, multiplier),
            motors: shares,
        }
    });
    let source = source_fault(profile, project, &motors)?;

    // Falla en las barras de cada tablero: la del transformador más cercano
    // aguas arriba, o la del punto de conexión.
    let n = project.panels.len();
    let mut bus: Vec<Option<BusFault>> = vec![None; n];
    let index_of = |id: &str| project.panels.iter().position(|p| p.id == id);
    // Los tableros se resuelven de arriba hacia abajo.
    let mut pending: Vec<usize> = (0..n).collect();
    for _ in 0..=n {
        pending.retain(|&i| {
            let panel = &project.panels[i];
            let upstream = match &panel.parent {
                None => Some(source.clone()),
                Some(parent) => index_of(parent).and_then(|k| bus[k].clone()),
            };
            let Some(upstream) = upstream else {
                return true;
            };
            bus[i] = Some(match &panel.transformer {
                Some(t) => match steps_for_transformer(
                    profile,
                    t.kva,
                    t.impedance_percent,
                    &t.secondary,
                    &format!("transformador de {}", panel.id),
                    false,
                ) {
                    Ok((ka, steps)) => with_motors(
                        profile,
                        ka,
                        FaultOrigin::Transformer {
                            panel: panel.id.clone(),
                        },
                        steps,
                        &motors,
                    ),
                    // Sin datos válidos del transformador se queda la de arriba.
                    Err(_) => upstream,
                },
                None => upstream,
            });
            false
        });
        if pending.is_empty() {
            break;
        }
    }

    for (i, panel) in project.panels.iter().enumerate() {
        let Some(own) = bus[i].clone() else {
            continue;
        };
        let parent = panel.parent.as_deref().and_then(index_of);
        let (upstream, upstream_name, upstream_board_ka) = match parent {
            Some(k) => (
                bus[k].clone(),
                format!("las barras de {}", project.panels[k].id),
                project.panels[k].board.interrupting_ka,
            ),
            None => (
                Some(source.clone()),
                "el punto de conexión".to_string(),
                None,
            ),
        };
        let Some(result) = panels.iter_mut().find(|r| r.id == panel.id) else {
            continue;
        };
        // El alimentador sale de las barras de arriba.
        if let Some(up) = &upstream {
            let ka = panel
                .feeder
                .overrides
                .breaker
                .interrupting_ka
                .or(upstream_board_ka);
            check_breaker(profile, &mut result.feeder, ka, up, &upstream_name);
        }
        let own_name = format!("las barras de {}", panel.id);
        let board_ka = panel.board.interrupting_ka;
        if let (Some(t), Some(secondary)) = (&panel.transformer, result.secondary.as_mut()) {
            if t.secondary_protection {
                let ka = t.secondary_overrides.breaker.interrupting_ka.or(board_ka);
                check_breaker(profile, secondary, ka, &own, &own_name);
            }
        }
        for (circuit, item) in panel.circuits.iter().zip(result.circuits.iter_mut()) {
            let ka = circuit.overrides.breaker.interrupting_ka.or(board_ka);
            check_breaker(profile, item, ka, &own, &own_name);
        }
        result.fault = Some(own);
    }
    Some(FaultStudy {
        source,
        motors,
        impedance_tolerance: profile.parameters.fault_impedance_tolerance,
    })
}
