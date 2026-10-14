//! Circuitos de motores y de motocompresores dentro del proyecto: su
//! corriente, su corriente de diseño, su interruptor y sus verificaciones
//! propias (artículos 430 y 440).

use ieestudio_core::units::SQRT_3;
use ieestudio_core::{CircuitKind, Error, MotorType, System};
use ieestudio_electrical::motor::{
    compressor_branch, full_load_current, motor_branch, overload_limit, CompressorNameplate,
};
use ieestudio_rules::{Profile, Rule};

use crate::calc::{Check, Status};
use crate::model::{Circuit, LoadKind, MotorLoad, Settings};
use crate::procedure::{plain, Step};

/// Lo que un motor o un motocompresor aporta a su circuito y a su tablero.
#[derive(Debug, Clone)]
pub(crate) struct MotorCircuit {
    /// Corriente con la que se calculan conductores y caída: la de tabla del
    /// motor o la de placa del compresor.
    pub current_a: f64,
    /// Corriente que deben cubrir los conductores (430-22, 440-32 o 440-35).
    pub design_a: f64,
    /// Interruptor propuesto: el máximo permitido.
    pub breaker_max_a: f64,
    /// Potencia aparente, para el cuadro de cargas.
    pub va: f64,
    /// Pasos del procedimiento: corriente, diseño e interruptor.
    pub steps: Vec<Step>,
    /// Verificaciones propias además de las del conductor.
    pub checks: Vec<Check>,
    /// Reglas que citan la ampacidad del circuito.
    pub ampacity_rules: Vec<Rule>,
    /// Descripción corta: «motor de 10 hp», «compresor de 16.7 A».
    pub label: String,
}

fn multiplier(kind: CircuitKind) -> f64 {
    match kind {
        CircuitKind::ThreePhase => SQRT_3,
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => 1.0,
    }
}

fn check(name: &str, status: Status, detail: String, profile: &Profile, rules: &[Rule]) -> Check {
    Check {
        name: name.to_string(),
        status,
        detail,
        references: rules.iter().map(|&r| profile.reference(r)).collect(),
    }
}

/// Potencia en hp para mostrar: `7-1/2` → `7½`.
pub fn hp_label(hp: &str) -> String {
    let fraction = |f: &str| match f {
        "1/6" => Some("⅙"),
        "1/4" => Some("¼"),
        "1/3" => Some("⅓"),
        "1/2" => Some("½"),
        "3/4" => Some("¾"),
        _ => None,
    };
    if let Some(f) = fraction(hp) {
        return f.to_string();
    }
    match hp.split_once('-') {
        Some((whole, part)) => match fraction(part) {
            Some(f) => format!("{whole}{f}"),
            None => hp.to_string(),
        },
        None => hp.to_string(),
    }
}

/// Datos de motor o de compresor del circuito, ya resueltos. `None` si el
/// circuito no es de motor ni de aire acondicionado. `Some(Err)` si lo es
/// pero falta un dato o la tabla no cubre el caso: entonces se calcula como
/// carga general y queda «no evaluado».
pub(crate) fn motor_circuit(
    profile: &Profile,
    settings: &Settings,
    system: &System,
    circuit: &Circuit,
) -> Option<Result<MotorCircuit, Error>> {
    match circuit.load.kind {
        LoadKind::Motor => Some(match &circuit.load.motor {
            Some(_) => motor(profile, settings, system, circuit),
            None => Err(Error::InvalidInput(
                "falta la potencia del motor en hp".into(),
            )),
        }),
        LoadKind::AirConditioning => Some(match &circuit.load.compressor {
            Some(_) => compressor(profile, settings, system, circuit),
            None => Err(Error::InvalidInput(
                "faltan los datos de placa del equipo: corriente de carga nominal, o ampacidad mínima y protección máxima".into(),
            )),
        }),
        _ => None,
    }
}

fn motor(
    profile: &Profile,
    settings: &Settings,
    system: &System,
    circuit: &Circuit,
) -> Result<MotorCircuit, Error> {
    let data = circuit.load.motor.as_ref().expect("se revisó antes");
    motor_parts(
        profile,
        settings.include_iec_ratings,
        circuit.kind,
        system,
        data,
        circuit.overrides.breaker_a,
    )
}

/// Corriente, diseño, interruptor y sobrecarga de un motor conectado como
/// `kind` en `system`. `forced_breaker` es el interruptor que fijó el usuario.
pub(crate) fn motor_parts(
    profile: &Profile,
    include_iec: bool,
    kind: CircuitKind,
    system: &System,
    data: &MotorLoad,
    forced_breaker: Option<f64>,
) -> Result<MotorCircuit, Error> {
    let volts = kind.reference_voltage(system)?;
    let three_phase = kind == CircuitKind::ThreePhase;
    let current = full_load_current(profile, &data.hp, three_phase, volts)?;
    let motor_type: Option<MotorType> = three_phase.then_some(data.motor_type);
    let branch = motor_branch(profile, current.full_load_a, motor_type, include_iec)?;
    let flc = current.full_load_a;
    let p = &profile.parameters;
    let r = |rules: &[Rule]| rules.iter().map(|&x| profile.reference(x)).collect();
    let hp = hp_label(&current.hp);
    let table = if three_phase { "430-250" } else { "430-248" };

    let mut steps = vec![
        Step {
            title: "Corriente del motor".into(),
            formula: None,
            substitution: format!(
                "Ipc = {} A ({hp} hp, {} kW, {} V)",
                plain(flc),
                plain(current.kw),
                plain(current.column_v)
            ),
            note: Some(format!(
                "Tabla {table}, columna de {} V para un sistema de {} V. Para conductores y protección se usa la corriente de la tabla, no la de la placa.",
                plain(current.column_v),
                plain(volts)
            )),
            references: r(&[Rule::MotorFullLoadCurrent]),
        },
        Step {
            title: "Corriente de diseño".into(),
            formula: Some("Id = 1.25 · Ipc".into()),
            substitution: format!(
                "Id = {} · {} = {:.2} A",
                plain(p.motor_conductor_factor),
                plain(flc),
                branch.conductor_a
            ),
            note: Some(
                "La ampacidad del conductor, ya corregida por temperatura y agrupamiento, debe cubrirla."
                    .into(),
            ),
            references: r(&[Rule::MotorBranchConductor]),
        },
    ];

    let breaker = forced_breaker.unwrap_or(branch.breaker_max_a);
    let poles = match kind {
        CircuitKind::LineToNeutral => 1,
        CircuitKind::LineToLine => 2,
        CircuitKind::ThreePhase => 3,
    };
    steps.push(Step {
        title: "Interruptor del motor".into(),
        formula: Some("Imax = % de la Tabla 430-52 · Ipc".into()),
        substitution: format!(
            "Imax = {} % · {} = {:.2} A → {poles}P-{:.0} A",
            plain(branch.protection_percent),
            plain(flc),
            branch.protection_limit_a,
            breaker
        ),
        note: Some(format!(
            "Interruptor de tiempo inverso. Si {:.2} A no es un valor normalizado, se permite el inmediato superior ({:.0} A). Si el motor no arranca, se permite subir hasta {:.0} A. El relevador de sobrecarga protege al conductor contra sobrecarga; por eso el interruptor puede ser mayor que su ampacidad.",
            branch.protection_limit_a, branch.breaker_max_a, branch.starting_limit_a
        )),
        references: r(&[
            Rule::MotorBranchProtection,
            Rule::MotorStarting,
            Rule::MotorConductorProtection,
        ]),
    });

    let mut checks = vec![protection_check(
        profile,
        breaker,
        branch.breaker_max_a,
        Some(branch.starting_limit_a),
        &[
            Rule::MotorBranchProtection,
            Rule::MotorStarting,
            Rule::MotorConductorProtection,
        ],
        "la Tabla 430-52",
    )];

    match data.nameplate_a {
        Some(plate) if plate.is_finite() && plate > 0.0 => {
            let (factor, limit) = overload_limit(profile, plate, data.service_factor_marked);
            steps.push(Step {
                title: "Relevador de sobrecarga".into(),
                formula: Some("Ajuste máximo = factor · I de placa".into()),
                substitution: format!(
                    "{} · {} = {:.2} A",
                    plain(factor),
                    plain(plate),
                    limit
                ),
                note: Some(if data.service_factor_marked {
                    "Motor con factor de servicio de 1.15 o más, o elevación de temperatura de 40 °C o menos: 125 %.".into()
                } else {
                    "Motor sin factor de servicio de 1.15 ni elevación de 40 °C marcados: 115 %.".into()
                }),
                references: r(&[Rule::MotorOverload]),
            });
            checks.push(check(
                "Sobrecarga",
                Status::Complies,
                format!("relevador ajustado a no más de {limit:.2} A"),
                profile,
                &[Rule::MotorOverload],
            ));
        }
        _ => checks.push(check(
            "Sobrecarga",
            Status::NotEvaluated,
            "falta la corriente de placa del motor para ajustar el relevador de sobrecarga".into(),
            profile,
            &[Rule::MotorOverload],
        )),
    }

    Ok(MotorCircuit {
        current_a: flc,
        design_a: branch.conductor_a,
        breaker_max_a: branch.breaker_max_a,
        va: flc * volts * multiplier(kind),
        steps,
        checks,
        ampacity_rules: vec![Rule::MotorFullLoadCurrent, Rule::MotorBranchConductor],
        label: format!("motor de {hp} hp"),
    })
}

/// Verificación del interruptor de un motor o compresor contra su máximo.
fn protection_check(
    profile: &Profile,
    breaker: f64,
    max: f64,
    starting: Option<f64>,
    rules: &[Rule],
    source: &str,
) -> Check {
    let (status, detail) = if breaker <= max + 1e-9 {
        (
            Status::Complies,
            format!("interruptor de {breaker:.0} A; {source} permite hasta {max:.0} A"),
        )
    } else if starting.is_some_and(|s| breaker <= s + 1e-9) {
        (
            Status::NotEvaluated,
            format!(
                "interruptor de {breaker:.0} A, mayor que los {max:.0} A de {source}; solo se permite si el equipo no arranca con {max:.0} A. Confírmalo"
            ),
        )
    } else {
        (
            Status::Fails,
            format!("interruptor de {breaker:.0} A; {source} permite hasta {max:.0} A"),
        )
    };
    check("Protección", status, detail, profile, rules)
}

fn compressor(
    profile: &Profile,
    settings: &Settings,
    system: &System,
    circuit: &Circuit,
) -> Result<MotorCircuit, Error> {
    let data = circuit.load.compressor.expect("se revisó antes");
    let volts = circuit.kind.reference_voltage(system)?;
    let plate = CompressorNameplate {
        rated_load_a: data.rated_load_a,
        branch_selection_a: data.branch_selection_a,
        minimum_circuit_a: data.minimum_circuit_a,
        maximum_protection_a: data.maximum_protection_a,
    };
    let branch = compressor_branch(profile, &plate, settings.include_iec_ratings)?;
    let p = &profile.parameters;
    let r = |rules: &[Rule]| rules.iter().map(|&x| profile.reference(x)).collect();
    let breaker = circuit.overrides.breaker_a.unwrap_or(branch.breaker_max_a);
    let poles = match circuit.kind {
        CircuitKind::LineToNeutral => 1,
        CircuitKind::LineToLine => 2,
        CircuitKind::ThreePhase => 3,
    };

    let mut steps = Vec::new();
    let rla_known = data.rated_load_a.is_some() || data.branch_selection_a.is_some();
    steps.push(Step {
        title: "Corriente del compresor".into(),
        formula: None,
        substitution: if rla_known {
            format!("I = {} A", plain(branch.basis_a))
        } else {
            format!(
                "I ≈ {} / {} = {:.2} A",
                plain(branch.conductor_a),
                plain(p.compressor_conductor_factor),
                branch.basis_a
            )
        },
        note: Some(if rla_known {
            "Corriente de carga nominal o de selección del circuito derivado de la placa, la mayor."
                .into()
        } else {
            "Sin la corriente de carga nominal se estima desde la ampacidad mínima de la placa; solo se usa para la caída de tensión y el cuadro de cargas.".into()
        }),
        references: r(&[Rule::CompressorRating]),
    });
    if branch.from_equipment_nameplate && data.minimum_circuit_a.is_some() {
        steps.push(Step {
            title: "Corriente de diseño".into(),
            formula: None,
            substitution: format!(
                "Id = {} A (ampacidad mínima de la placa)",
                plain(branch.conductor_a)
            ),
            note: Some("El equipo ya marca la ampacidad mínima de sus conductores.".into()),
            references: r(&[Rule::EquipmentNameplate]),
        });
    } else {
        steps.push(Step {
            title: "Corriente de diseño".into(),
            formula: Some("Id = 1.25 · I".into()),
            substitution: format!(
                "Id = {} · {} = {:.2} A",
                plain(p.compressor_conductor_factor),
                plain(branch.basis_a),
                branch.conductor_a
            ),
            note: Some(
                "La ampacidad del conductor, ya corregida por temperatura y agrupamiento, debe cubrirla."
                    .into(),
            ),
            references: r(&[Rule::CompressorConductor]),
        });
    }
    let source = if data.maximum_protection_a.is_some() {
        "la placa del equipo"
    } else {
        "440-22(a)"
    };
    steps.push(Step {
        title: "Interruptor del compresor".into(),
        formula: if data.maximum_protection_a.is_some() {
            None
        } else {
            Some("Imax = 1.75 · I".into())
        },
        substitution: format!(
            "Imax = {:.2} A → {poles}P-{:.0} A",
            branch.protection_limit_a, breaker
        ),
        note: Some(match (data.maximum_protection_a, branch.starting_limit_a) {
            (Some(_), _) => "Protección máxima marcada en la placa del equipo; no se redondea hacia arriba.".into(),
            (None, Some(start)) => format!(
                "Valor normalizado igual o inmediato inferior, sin exigir menos de {} A. Si el compresor no arranca, se permite hasta {start:.2} A.",
                plain(p.compressor_minimum_protection_a)
            ),
            (None, None) => String::new(),
        }),
        references: r(&[Rule::CompressorProtection, Rule::EquipmentNameplate]),
    });

    let checks = vec![protection_check(
        profile,
        breaker,
        branch.breaker_max_a,
        if data.maximum_protection_a.is_some() {
            None
        } else {
            branch.starting_limit_a
        },
        &[
            Rule::CompressorProtection,
            Rule::EquipmentNameplate,
            Rule::MotorConductorProtection,
        ],
        source,
    )];

    Ok(MotorCircuit {
        current_a: branch.basis_a,
        design_a: branch.conductor_a,
        breaker_max_a: branch.breaker_max_a,
        va: branch.basis_a * volts * multiplier(circuit.kind),
        steps,
        checks,
        ampacity_rules: vec![
            Rule::CompressorRating,
            Rule::CompressorConductor,
            Rule::EquipmentNameplate,
        ],
        label: format!("compresor de {} A", plain(branch.basis_a)),
    })
}
