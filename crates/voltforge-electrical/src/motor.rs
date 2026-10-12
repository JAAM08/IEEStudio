//! Motores (artículo 430) y motocompresores de aire acondicionado y
//! refrigeración (artículo 440).
//!
//! Criterios (NOM-001-SEDE-2012):
//!
//! 1. Corriente del motor (430-6(a)(1)): de las tablas 430-248 (una fase) o
//!    430-250 (tres fases) por potencia y tensión, no la de la placa.
//! 2. Conductores de un motor (430-22): 125 % de esa corriente.
//! 3. Protección contra cortocircuito (430-52, Tabla 430-52): hasta el
//!    porciento de la tabla; si no es un valor normalizado, el inmediato
//!    superior (excepción 1). Si el motor no arranca, la excepción 2 permite
//!    subir hasta un límite.
//! 4. Sobrecarga (430-32(a)(1)): el relevador se ajusta con la corriente de
//!    la placa, a no más del 125 % o 115 %.
//! 5. Motocompresores (440-6(a), 440-22(a), 440-32): con la corriente de la
//!    placa; conductores al 125 % y protección hasta 175 % (225 % si no
//!    arranca), sin exigir menos de 15 A.
//! 6. Equipos con placa de ampacidad mínima y protección máxima (440-4(b),
//!    440-35, 440-22(c)): se usan esos valores.

use voltforge_core::{Error, MotorType, Result};
use voltforge_rules::{MotorCurrentRow, Profile, Rule};

use crate::sizing::{standard_rating_at_least, standard_rating_at_most};

/// Corriente a plena carga de un motor según su tabla.
#[derive(Debug, Clone, PartialEq)]
pub struct MotorCurrent {
    pub hp: String,
    pub kw: f64,
    pub three_phase: bool,
    /// Tensión nominal de la columna usada, por ejemplo 230 V para un sistema de 220 V.
    pub column_v: f64,
    pub full_load_a: f64,
}

/// Potencias de la tabla de motores, en orden creciente.
pub fn motor_ratings(profile: &Profile, three_phase: bool) -> Vec<(String, f64)> {
    let rows = if three_phase {
        &profile.three_phase_motors
    } else {
        &profile.single_phase_motors
    };
    rows.iter().map(|r| (r.hp.clone(), r.kw)).collect()
}

/// Corriente a plena carga de la Tabla 430-248 o 430-250 (430-6(a)(1)).
pub fn full_load_current(
    profile: &Profile,
    hp: &str,
    three_phase: bool,
    system_v: f64,
) -> Result<MotorCurrent> {
    let (rows, table): (&[MotorCurrentRow], &str) = if three_phase {
        (&profile.three_phase_motors, "430-250")
    } else {
        (&profile.single_phase_motors, "430-248")
    };
    let missing = |what: String| Error::NotInTable {
        table: profile.clause(Rule::MotorFullLoadCurrent),
        what,
    };
    let column = profile
        .motor_voltage_columns
        .iter()
        .find(|c| {
            c.table == table
                && system_v >= c.system_min_v - 1e-9
                && system_v <= c.system_max_v + 1e-9
        })
        .ok_or_else(|| {
            missing(format!(
                "un motor de {} a {system_v} V",
                if three_phase {
                    "tres fases"
                } else {
                    "una fase"
                }
            ))
        })?;
    let row = rows
        .iter()
        .find(|r| r.hp == hp)
        .ok_or_else(|| missing(format!("un motor de {hp} hp")))?;
    let full_load_a = row
        .currents
        .iter()
        .find(|(c, _)| c == &column.column)
        .and_then(|(_, a)| *a)
        .ok_or_else(|| missing(format!("un motor de {hp} hp a {system_v} V")))?;
    let column_v = column
        .column
        .trim_start_matches('v')
        .parse()
        .map_err(|_| missing(format!("la columna «{}»", column.column)))?;
    Ok(MotorCurrent {
        hp: row.hp.clone(),
        kw: row.kw,
        three_phase,
        column_v,
        full_load_a,
    })
}

/// Circuito derivado de un motor.
#[derive(Debug, Clone, PartialEq)]
pub struct MotorBranch {
    pub full_load_a: f64,
    /// Corriente mínima de los conductores (430-22).
    pub conductor_a: f64,
    /// Porciento de la Tabla 430-52 para interruptor de tiempo inverso.
    pub protection_percent: f64,
    /// Protección calculada: corriente × porciento.
    pub protection_limit_a: f64,
    /// Interruptor máximo: el normalizado igual o inmediato superior al
    /// calculado (430-52(c)(1), excepción 1).
    pub breaker_max_a: f64,
    /// Hasta dónde se permite subir si el motor no arranca (excepción 2(3)).
    pub starting_limit_a: f64,
}

/// Conductor y protección del circuito de un motor con interruptor de
/// tiempo inverso.
pub fn motor_branch(
    profile: &Profile,
    full_load_a: f64,
    motor_type: Option<MotorType>,
    include_iec: bool,
) -> Result<MotorBranch> {
    let p = &profile.parameters;
    // `None` es un motor de una fase.
    let key = motor_type.map_or("single_phase", MotorType::code);
    let row = profile
        .motor_protection
        .iter()
        .find(|r| r.motor_type == key)
        .ok_or_else(|| Error::NotInTable {
            table: profile.clause(Rule::MotorBranchProtection),
            what: format!("el tipo de motor «{key}»"),
        })?;
    let protection_limit_a = full_load_a * row.inverse_time_breaker / 100.0;
    let starting_factor = if full_load_a <= p.motor_starting_limit_a {
        p.motor_starting_factor_small
    } else {
        p.motor_starting_factor_large
    };
    Ok(MotorBranch {
        full_load_a,
        conductor_a: full_load_a * p.motor_conductor_factor,
        protection_percent: row.inverse_time_breaker,
        protection_limit_a,
        breaker_max_a: standard_rating_at_least(profile, protection_limit_a, include_iec)?,
        starting_limit_a: full_load_a * starting_factor,
    })
}

/// Ajuste máximo del relevador de sobrecarga (430-32(a)(1)).
///
/// `marked` es un motor con factor de servicio de 1.15 o más, o con
/// elevación de temperatura de 40 °C o menos.
pub fn overload_limit(profile: &Profile, nameplate_a: f64, marked: bool) -> (f64, f64) {
    let p = &profile.parameters;
    let factor = if marked {
        p.motor_overload_factor_marked
    } else {
        p.motor_overload_factor_other
    };
    (factor, nameplate_a * factor)
}

/// Circuito derivado de un motocompresor o de un equipo de aire acondicionado.
#[derive(Debug, Clone, PartialEq)]
pub struct CompressorBranch {
    /// Corriente de la placa con la que se calcula: la de carga nominal o la
    /// de selección del circuito derivado, la mayor (440-6(a)).
    pub basis_a: f64,
    /// Corriente mínima de los conductores (440-32), o la ampacidad mínima
    /// de la placa (440-35).
    pub conductor_a: f64,
    /// Protección calculada (440-22(a)), o la máxima de la placa (440-22(c)).
    pub protection_limit_a: f64,
    /// Interruptor máximo: el normalizado igual o inmediato inferior al
    /// límite, sin exigir menos del mínimo de la excepción.
    pub breaker_max_a: f64,
    /// Hasta dónde se permite subir si el compresor no arranca (440-22(a)).
    pub starting_limit_a: Option<f64>,
    /// Los valores salen de la placa del equipo y no de la corriente del compresor.
    pub from_equipment_nameplate: bool,
}

/// Datos de la placa de un motocompresor o de su equipo.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CompressorNameplate {
    /// Corriente de carga nominal (RLA).
    pub rated_load_a: Option<f64>,
    /// Corriente de selección del circuito derivado (BCSC).
    pub branch_selection_a: Option<f64>,
    /// Ampacidad mínima del circuito marcada en el equipo (MCA).
    pub minimum_circuit_a: Option<f64>,
    /// Protección máxima marcada en el equipo (MOP).
    pub maximum_protection_a: Option<f64>,
}

pub fn compressor_branch(
    profile: &Profile,
    plate: &CompressorNameplate,
    include_iec: bool,
) -> Result<CompressorBranch> {
    let p = &profile.parameters;
    let positive = |name: &str, v: Option<f64>| -> Result<Option<f64>> {
        match v {
            Some(x) if !(x.is_finite() && x > 0.0) => Err(Error::InvalidInput(format!(
                "la {name} debe ser mayor que cero"
            ))),
            other => Ok(other),
        }
    };
    let rla = positive("corriente de carga nominal", plate.rated_load_a)?;
    let bcsc = positive("corriente de selección", plate.branch_selection_a)?;
    let mca = positive("ampacidad mínima", plate.minimum_circuit_a)?;
    let mop = positive("protección máxima", plate.maximum_protection_a)?;
    let basis = match (rla, bcsc) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    let minimum = p.compressor_minimum_protection_a;
    // Si el límite queda por debajo del mínimo, la excepción permite el mínimo.
    let round_down = |limit: f64| -> Result<f64> {
        if limit < minimum {
            Ok(minimum)
        } else {
            Ok(standard_rating_at_most(profile, limit, include_iec)?.max(minimum))
        }
    };
    match (mca, mop, basis) {
        (Some(mca), Some(mop), _) => Ok(CompressorBranch {
            basis_a: basis.unwrap_or(mca / p.compressor_conductor_factor),
            conductor_a: mca,
            protection_limit_a: mop,
            breaker_max_a: standard_rating_at_most(profile, mop, include_iec)?,
            starting_limit_a: None,
            from_equipment_nameplate: true,
        }),
        (_, _, Some(basis)) => {
            let limit = basis * p.compressor_protection_factor;
            Ok(CompressorBranch {
                basis_a: basis,
                conductor_a: mca.unwrap_or(basis * p.compressor_conductor_factor),
                protection_limit_a: mop.map_or(limit, |m| m.min(limit)),
                breaker_max_a: round_down(mop.map_or(limit, |m| m.min(limit)))?,
                starting_limit_a: Some(basis * p.compressor_protection_max_factor),
                from_equipment_nameplate: mca.is_some() || mop.is_some(),
            })
        }
        _ => Err(Error::InvalidInput(
            "falta la corriente de carga nominal del compresor o la ampacidad mínima y la protección máxima de la placa"
                .into(),
        )),
    }
}
