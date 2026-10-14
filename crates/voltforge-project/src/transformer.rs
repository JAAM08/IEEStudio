//! Transformadores de 600 V o menos dentro del proyecto (artículo 450).
//!
//! Un tablero puede alimentarse a través de un transformador: su alimentador
//! es el primario, a la tensión del tablero de donde sale, y del secundario
//! salen los conductores hasta el tablero, a la tensión nueva. Los dos lados
//! se dimensionan con la capacidad del transformador, no solo con la carga:
//! la corriente de diseño no baja del 125 % de su corriente nominal, que es la
//! protección de la Tabla 450-3(b), para poder usarlo completo.

use voltforge_core::{CircuitKind, Error, System};
use voltforge_electrical::current::{design_current, nominal_current};
use voltforge_rules::{Profile, Rule};

use crate::calc::{check, phase_multiplier, Check, Given, Status};
use crate::procedure::{current_step, plain, Step};

/// Corriente nominal de un lado del transformador.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Rating {
    pub rated_a: f64,
    pub volts: f64,
}

pub(crate) fn rating(kva: f64, kind: CircuitKind, system: &System) -> Result<Rating, Error> {
    if kva.is_nan() || kva <= 0.0 {
        return Err(Error::InvalidInput(
            "falta la potencia del transformador".into(),
        ));
    }
    let volts = kind.reference_voltage(system)?;
    Ok(Rating {
        rated_a: kva * 1000.0 / (phase_multiplier(kind) * volts),
        volts,
    })
}

/// Porciento de diseño de los conductores: el de la Tabla 450-3(b) para
/// corrientes de 9 A o más con protección solo en el primario (125 %).
fn design_percent(profile: &Profile) -> f64 {
    profile
        .transformer_protection_row(true, false, f64::INFINITY)
        .map_or(125.0, |r| r.percent)
}

/// Corriente de un lado del transformador: la de la carga, y la de diseño, que
/// no baja del 125 % de la nominal del transformador. `motors` es el diseño
/// con motores que ya calculó quien llama, si los hay.
#[allow(clippy::too_many_arguments)] // los datos de un lado, sin agrupar
pub(crate) fn side_given(
    profile: &Profile,
    kind: CircuitKind,
    system: &System,
    demanded_w: f64,
    power_factor: f64,
    continuous_fraction: f64,
    motors: Option<Given>,
    rating: Rating,
    side: &str,
    kva: f64,
) -> Result<Given, Error> {
    let nominal_a = nominal_current(demanded_w, power_factor, kind, system)?;
    let (load_design_a, mut steps) = match motors {
        Some(g) => (g.design_a, g.steps),
        None => {
            let continuous = nominal_a * continuous_fraction;
            (
                design_current(profile, continuous, nominal_a - continuous),
                vec![current_step(
                    kind,
                    Some(demanded_w),
                    power_factor,
                    rating.volts,
                    nominal_a,
                )],
            )
        }
    };
    let percent = design_percent(profile);
    let by_rating = rating.rated_a * percent / 100.0;
    let design_a = load_design_a.max(by_rating);
    let k = if phase_multiplier(kind) > 1.0 {
        "√3 · "
    } else {
        ""
    };
    steps.push(Step {
        title: format!("Corriente nominal del transformador, {side}"),
        formula: Some(format!("In = S / ({k}V)")),
        substitution: format!(
            "In = {} / ({k}{}) = {:.2} A",
            plain(kva * 1000.0),
            plain(rating.volts),
            rating.rated_a
        ),
        note: None,
        references: vec![],
    });
    steps.push(Step {
        title: format!("Corriente de diseño del {side}"),
        formula: Some(format!(
            "Id = máx(Id de la carga, {} % · In)",
            plain(percent)
        )),
        substitution: format!(
            "Id = máx({load_design_a:.2}, {by_rating:.2}) = {design_a:.2} A"
        ),
        note: Some(
            "Los conductores llevan la capacidad del transformador, no solo la carga de hoy: su protección es del 125 % de la corriente nominal."
                .into(),
        ),
        references: vec![profile.reference(Rule::TransformerProtection)],
    });
    Ok(Given {
        nominal_a,
        design_a,
        breaker_a: None,
        motor_protection: false,
        steps,
        ampacity_rules: vec![Rule::TransformerProtection, Rule::DesignCurrent],
        feeder_motor_limit_a: None,
    })
}

/// Protección máxima de un lado del transformador (Tabla 450-3(b)).
pub(crate) fn protection_check(
    profile: &Profile,
    breaker_a: f64,
    rated_a: f64,
    primary: bool,
    secondary_protected: bool,
    include_iec: bool,
) -> Check {
    let side = if primary { "primario" } else { "secundario" };
    let Some(row) = profile.transformer_protection_row(primary, secondary_protected, rated_a)
    else {
        return Check {
            name: format!("Protección del {side}"),
            status: Status::NotEvaluated,
            detail: "la Tabla 450-3(b) no cubre este caso".into(),
            references: vec![profile.reference(Rule::TransformerProtection)],
        };
    };
    let limit = rated_a * row.percent / 100.0;
    let allowed = if row.next_standard {
        crate::calc::standard_at_least(profile, limit, include_iec).unwrap_or(limit)
    } else {
        limit
    };
    let detail = format!(
        "interruptor de {breaker_a:.0} A; máximo {} % de {rated_a:.2} A = {limit:.2} A{}",
        plain(row.percent),
        if row.next_standard && (allowed - limit).abs() > 1e-9 {
            format!(", normalizado inmediato superior {allowed:.0} A (nota 1)")
        } else {
            String::new()
        }
    );
    let mut c = check(
        &format!("Protección del {side}"),
        breaker_a <= allowed + 1e-9,
        detail,
    );
    c.references = vec![profile.reference(Rule::TransformerProtection)];
    c
}

/// La carga calculada cabe en el transformador.
pub(crate) fn capacity_check(profile: &Profile, demanded_va: f64, kva: f64) -> Check {
    let mut c = check(
        "Capacidad del transformador",
        demanded_va <= kva * 1000.0 + 1e-6,
        format!(
            "{} kVA calculados en un transformador de {} kVA ({:.0} %)",
            plain((demanded_va / 1000.0 * 100.0).round() / 100.0),
            plain(kva),
            demanded_va / (kva * 10.0)
        ),
    );
    c.references = vec![profile.reference(Rule::FeederLoad)];
    c
}

/// Sin protección en el secundario, sus conductores dependen de las reglas
/// de derivación de 240-21(c), que el programa no verifica.
pub(crate) fn unprotected_secondary(profile: &Profile) -> Check {
    Check {
        name: "Protección del secundario".into(),
        status: Status::NotEvaluated,
        detail: "sin interruptor en el secundario: los conductores dependen de 240-21(c); revísalo a mano".into(),
        references: vec![
            profile.reference(Rule::TransformerSecondary),
            profile.reference(Rule::TransformerProtection),
        ],
    }
}
