//! Caída de tensión por impedancia (Tabla 9).

use crate::current::check_power_factor;
use crate::error::{Error, Result};
use crate::system::{CircuitKind, System};
use crate::tables::{Material, Profile, RacewayMaterial, Size};

/// Temperatura a la que la Tabla 9 da la resistencia, en °C.
pub const TABLE_9_TEMPERATURE_C: f64 = 75.0;

/// Coeficiente de temperatura de la resistencia a 75 °C (Tabla 8, nota 2).
pub fn temperature_coefficient(material: Material) -> f64 {
    match material {
        Material::Copper => 0.00323,
        Material::Aluminum => 0.00330,
    }
}

/// Datos de un cálculo de caída de tensión.
#[derive(Debug, Clone, PartialEq)]
pub struct VoltageDropInput {
    pub kind: CircuitKind,
    pub system: System,
    /// Corriente del circuito, en amperes.
    pub current_a: f64,
    /// Longitud de un sentido del tramo, en metros.
    pub length_m: f64,
    pub power_factor: f64,
    pub size: Size,
    pub material: Material,
    pub raceway: RacewayMaterial,
    /// Conductores por fase.
    pub conductors_per_phase: u32,
    /// Temperatura del conductor para ajustar la resistencia. `None` usa la
    /// resistencia de la Tabla 9 tal cual (75 °C).
    pub conductor_temperature_c: Option<f64>,
}

/// Resultado y valores intermedios, para poder mostrar el procedimiento.
#[derive(Debug, Clone, PartialEq)]
pub struct VoltageDropResult {
    /// Tensión contra la que se expresa el porcentaje.
    pub reference_voltage_v: f64,
    /// Factor de la fórmula: √3 o 2.
    pub factor: f64,
    /// Resistencia de la Tabla 9, en Ω/km.
    pub table_resistance_ohm_km: f64,
    /// Resistencia usada en el cálculo, tras el ajuste opcional.
    pub resistance_ohm_km: f64,
    /// Reactancia de la Tabla 9, en Ω/km.
    pub reactance_ohm_km: f64,
    /// `R·cos θ + X·sen θ`, en Ω/km, para un conductor.
    pub effective_impedance_ohm_km: f64,
    /// Caída de tensión con la fórmula de la Tabla 9.
    pub drop_v: f64,
    pub drop_percent: f64,
    /// Caída de tensión con la fórmula exacta (fasores), como comprobación.
    pub exact_drop_v: f64,
    pub exact_drop_percent: f64,
}

/// Calcula la caída de tensión de un tramo.
///
/// Fórmula oficial (nota 2 de la Tabla 9):
/// `ΔV = k · I · L · (R·cos θ + X·sen θ) / n`, con `k = √3` en circuitos
/// trifásicos y `2` en los demás, `L` en kilómetros y `n` conductores por fase.
pub fn voltage_drop(profile: &Profile, input: &VoltageDropInput) -> Result<VoltageDropResult> {
    check_power_factor(input.power_factor)?;
    if !(input.current_a.is_finite() && input.current_a >= 0.0) {
        return Err(Error::InvalidInput(
            "la corriente debe ser un número no negativo".into(),
        ));
    }
    if !(input.length_m.is_finite() && input.length_m >= 0.0) {
        return Err(Error::InvalidInput(
            "la longitud debe ser un número no negativo".into(),
        ));
    }
    if input.conductors_per_phase == 0 {
        return Err(Error::InvalidInput(
            "debe haber al menos un conductor por fase".into(),
        ));
    }
    let v_ref = input.kind.reference_voltage(&input.system)?;
    let (r_table, x) = profile.impedance(&input.size, input.material, input.raceway)?;
    let r = match input.conductor_temperature_c {
        Some(t) => {
            r_table * (1.0 + temperature_coefficient(input.material) * (t - TABLE_9_TEMPERATURE_C))
        }
        None => r_table,
    };

    let cos = input.power_factor;
    let sin = (1.0 - cos * cos).sqrt();
    let n = f64::from(input.conductors_per_phase);
    let length_km = input.length_m / 1000.0;
    let k = input.kind.drop_factor();
    let z_eff = r * cos + x * sin;
    let drop_v = k * input.current_a * length_km * z_eff / n;

    // Fórmula exacta. En trifásico se resuelve por fase (tensión al neutro,
    // impedancia de un conductor) y se pasa a tensión entre fases; en los
    // demás circuitos la corriente recorre dos conductores.
    let (v_load, loop_factor, scale) = match input.kind {
        CircuitKind::ThreePhase => (v_ref / 3f64.sqrt(), 1.0, 3f64.sqrt()),
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => (v_ref, 2.0, 1.0),
    };
    let ir = loop_factor * input.current_a * r * length_km / n;
    let ix = loop_factor * input.current_a * x * length_km / n;
    let v_source = ((v_load * cos + ir).powi(2) + (v_load * sin + ix).powi(2)).sqrt();
    let exact_drop_v = scale * (v_source - v_load);

    Ok(VoltageDropResult {
        reference_voltage_v: v_ref,
        factor: k,
        table_resistance_ohm_km: r_table,
        resistance_ohm_km: r,
        reactance_ohm_km: x,
        effective_impedance_ohm_km: z_eff,
        drop_v,
        drop_percent: drop_v / v_ref * 100.0,
        exact_drop_v,
        exact_drop_percent: exact_drop_v / v_ref * 100.0,
    })
}

/// Longitud máxima, en metros, con la que el tramo cumple un límite de caída.
pub fn max_length_m(
    profile: &Profile,
    input: &VoltageDropInput,
    limit_percent: f64,
) -> Result<f64> {
    let unit = voltage_drop(
        profile,
        &VoltageDropInput {
            length_m: 1.0,
            ..input.clone()
        },
    )?;
    if unit.drop_percent <= 0.0 {
        return Ok(f64::INFINITY);
    }
    Ok(limit_percent / unit.drop_percent)
}
