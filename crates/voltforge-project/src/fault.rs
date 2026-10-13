//! Corriente de cortocircuito en cada tablero y capacidad interruptiva de
//! los interruptores (110-9, 110-10).
//!
//! La corriente de la fuente sale del dato de la compañía suministradora o del
//! transformador; de ahí se suma la impedancia de cada alimentador hasta cada
//! tablero (ver `voltforge_electrical::short_circuit`). Cada interruptor se
//! compara con la corriente disponible en sus terminales de línea: los de los
//! circuitos, con la del tablero donde están; el de un alimentador, con la del
//! tablero o la fuente de donde sale.

use voltforge_core::{Error, System};
use voltforge_electrical::short_circuit::{
    fault_basis, fault_current_a, source_impedance, transformer_fault_a, FaultBasis, Impedance,
};
use voltforge_rules::{Profile, Rule};

use crate::calc::{Check, Status};
use crate::model::Project;
use crate::procedure::{plain, Step};

/// Datos de falla de la fuente.
#[derive(Debug, Clone)]
pub(crate) struct SourceFault {
    pub basis: FaultBasis,
    pub available_a: f64,
    pub impedance: Impedance,
    /// Aportación de los motores, sumada en cada punto.
    pub motor_a: f64,
    pub motor_multiplier: f64,
    pub steps: Vec<Step>,
}

/// Qué se sabe de la falla aguas arriba de un alimentador.
#[derive(Debug, Clone)]
pub(crate) enum Upstream {
    /// El proyecto no tiene los datos de cortocircuito de la fuente.
    NoData,
    /// No se pudo calcular; la razón.
    Unknown(String),
    Known {
        /// Impedancia desde la fuente hasta el punto, ya con la ida y la
        /// vuelta en sistemas de una fase.
        impedance: Impedance,
        /// Corriente de falla en el punto, en amperes.
        fault_a: f64,
        /// Capacidad interruptiva de los interruptores de ese punto.
        rating_ka: Option<f64>,
        /// Nombre del punto: la fuente o un tablero.
        place: String,
    },
}

fn refs(profile: &Profile, rules: &[Rule]) -> Vec<voltforge_rules::RuleReference> {
    rules.iter().map(|&r| profile.reference(r)).collect()
}

/// Falla en la fuente, o `None` si el proyecto no tiene los datos.
pub(crate) fn source_fault(
    project: &Project,
    motors_va: f64,
) -> Option<Result<SourceFault, String>> {
    let data = &project.source.fault;
    let system: &System = &project.source.system;
    let (available_a, first) = match (
        data.available_ka,
        data.transformer_kva,
        data.transformer_impedance_percent,
    ) {
        (Some(ka), _, _) => (
            ka * 1000.0,
            Step {
                title: "Cortocircuito de la fuente".into(),
                formula: None,
                substitution: format!("Icc fuente = {} kA", plain(ka)),
                note: Some(
                    "Corriente de cortocircuito simétrica disponible en el punto de conexión, dato de la compañía suministradora."
                        .into(),
                ),
                references: vec![],
            },
        ),
        (None, Some(kva), Some(z)) => match transformer_fault_a(system, kva, z) {
            Ok(a) => {
                let three = fault_basis(system).map(|b| b.three_phase).unwrap_or(true);
                let v = system
                    .line_to_line_v
                    .or(system.line_to_neutral_v)
                    .unwrap_or_default();
                (
                    a,
                    Step {
                        title: "Cortocircuito de la fuente".into(),
                        formula: Some(if three {
                            "Icc = S / (√3 · V · Z)".into()
                        } else {
                            "Icc = S / (V · Z)".into()
                        }),
                        substitution: if three {
                            format!(
                                "Icc = {} / (1.732 · {} · {}) = {:.2} kA",
                                plain(kva * 1000.0),
                                plain(v),
                                plain(z / 100.0),
                                a / 1000.0
                            )
                        } else {
                            format!(
                                "Icc = {} / ({} · {}) = {:.2} kA",
                                plain(kva * 1000.0),
                                plain(v),
                                plain(z / 100.0),
                                a / 1000.0
                            )
                        },
                        note: Some(format!(
                            "Transformador de {} kVA con impedancia de {} %, con el primario como fuente infinita: es la corriente mayor posible en el secundario.",
                            plain(kva),
                            plain(z)
                        )),
                        references: vec![],
                    },
                )
            }
            Err(e) => return Some(Err(e.to_string())),
        },
        (None, Some(_), None) | (None, None, Some(_)) => {
            return Some(Err(
                "faltan la potencia o la impedancia del transformador".into()
            ))
        }
        (None, None, None) => return None,
    };
    let basis = match fault_basis(system) {
        Ok(b) => b,
        Err(e) => return Some(Err(e.to_string())),
    };
    let impedance = match source_impedance(&basis, available_a) {
        Ok(z) => z,
        Err(e) => return Some(Err(e.to_string())),
    };
    let multiplier = project.settings.motor_fault_multiplier;
    let motor_a = if basis.three_phase {
        multiplier * motors_va / (3.0 * basis.voltage_v)
    } else {
        multiplier * motors_va / basis.voltage_v
    };
    let steps = vec![
        first,
        Step {
            title: "Cortocircuito: impedancia de la fuente".into(),
            formula: Some("Zf = Vf / Icc fuente".into()),
            substitution: format!(
                "Zf = {} / {} = j{:.6} Ω",
                plain(basis.voltage_v),
                plain(available_a),
                impedance.x_ohm
            ),
            note: Some(if basis.three_phase {
                "Vf es la tensión de fase a neutro. La fuente se toma como reactancia pura: con la resistencia de los conductores en cuadratura da la corriente mayor.".into()
            } else {
                "Sistema de una fase: la falla recorre la ida y la vuelta de cada tramo. La fuente se toma como reactancia pura.".into()
            }),
            references: vec![],
        },
    ];
    Some(Ok(SourceFault {
        basis,
        available_a,
        impedance,
        motor_a,
        motor_multiplier: multiplier,
        steps,
    }))
}

/// Paso del cortocircuito en el tablero al final de un alimentador.
pub(crate) fn bus_step(
    profile: &Profile,
    source: &SourceFault,
    conductor: Impedance,
    total: Impedance,
    fault_a: f64,
    description: &str,
) -> Step {
    let symmetric = fault_current_a(&source.basis, total);
    let loop_text = if source.basis.loop_factor > 1.0 {
        "2 · "
    } else {
        ""
    };
    let mut note = format!(
        "Alimentador: {description}, {loop_text}({:.6} + j{:.6}) Ω con la resistencia a 25 °C.",
        conductor.r_ohm, conductor.x_ohm
    );
    if source.motor_a > 0.0 {
        note.push_str(&format!(
            " Se suma la aportación de todos los motores del proyecto, {} veces su corriente: {:.0} A, del lado seguro.",
            plain(source.motor_multiplier),
            source.motor_a
        ));
    }
    Step {
        title: "Cortocircuito en el tablero".into(),
        formula: Some(if source.motor_a > 0.0 {
            "Icc = Vf / |Z total| + I motores".into()
        } else {
            "Icc = Vf / |Z total|".into()
        }),
        substitution: if source.motor_a > 0.0 {
            format!(
                "Icc = {} / |{:.6} + j{:.6}| + {:.0} = {:.0} + {:.0} = {:.2} kA",
                plain(source.basis.voltage_v),
                total.r_ohm,
                total.x_ohm,
                source.motor_a,
                symmetric,
                source.motor_a,
                fault_a / 1000.0
            )
        } else {
            format!(
                "Icc = {} / |{:.6} + j{:.6}| = {:.2} kA",
                plain(source.basis.voltage_v),
                total.r_ohm,
                total.x_ohm,
                fault_a / 1000.0
            )
        },
        note: Some(note),
        references: refs(profile, &[Rule::Impedance, Rule::ResistanceTemperature]),
    }
}

/// Capacidad interruptiva de un interruptor frente a la corriente disponible
/// en sus terminales de línea.
pub(crate) fn interrupting_check(
    profile: &Profile,
    available: Result<(f64, &str), String>,
    rating_ka: Option<f64>,
) -> Check {
    let references = refs(profile, &[Rule::InterruptingRating]);
    let (status, detail) = match (available, rating_ka) {
        (Err(reason), _) => (Status::NotEvaluated, reason),
        (Ok((a, place)), None) => (
            Status::NotEvaluated,
            format!(
                "falta la capacidad interruptiva del interruptor; hay {:.2} kA disponibles en {place}",
                a / 1000.0
            ),
        ),
        (Ok((a, place)), Some(ka)) => (
            if ka * 1000.0 + 1e-6 >= a {
                Status::Complies
            } else {
                Status::Fails
            },
            format!(
                "interruptor de {} kA para {:.2} kA disponibles en {place}",
                plain(ka),
                a / 1000.0
            ),
        ),
    };
    Check {
        name: "Capacidad interruptiva".into(),
        status,
        detail,
        references,
    }
}

/// Mensaje cuando el proyecto no tiene los datos de la fuente.
pub(crate) fn missing_source(profile: &Profile) -> Check {
    Check {
        name: "Capacidad interruptiva".into(),
        status: Status::NotEvaluated,
        detail: "falta la corriente de cortocircuito de la fuente (dato de la compañía suministradora) o la potencia e impedancia del transformador".into(),
        references: refs(profile, &[Rule::InterruptingRating]),
    }
}

pub(crate) fn unknown(e: &Error) -> Upstream {
    Upstream::Unknown(format!("no se pudo calcular el cortocircuito: {e}"))
}
