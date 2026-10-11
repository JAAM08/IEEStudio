//! Corriente nominal y corriente de diseño.

use crate::error::{Error, Result};
use crate::system::{CircuitKind, System};

/// Fracción adicional que se aplica a la carga continua: 125 %.
pub const CONTINUOUS_LOAD_FACTOR: f64 = 1.25;

/// Corriente nominal de una carga a partir de su potencia activa.
///
/// - Una fase: `I = P / (Vfn · fp)`
/// - Dos fases: `I = P / (Vff · fp)`
/// - Tres fases: `I = P / (√3 · Vff · fp)`
pub fn nominal_current(
    power_w: f64,
    power_factor: f64,
    kind: CircuitKind,
    system: &System,
) -> Result<f64> {
    if !(power_w.is_finite() && power_w >= 0.0) {
        return Err(Error::InvalidInput(
            "la potencia debe ser un número no negativo".into(),
        ));
    }
    check_power_factor(power_factor)?;
    let v = kind.reference_voltage(system)?;
    let k = match kind {
        CircuitKind::ThreePhase => 3f64.sqrt(),
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => 1.0,
    };
    Ok(power_w / (k * v * power_factor))
}

/// Corriente de diseño: 125 % de la parte continua más 100 % de la no continua.
pub fn design_current(continuous_a: f64, noncontinuous_a: f64) -> f64 {
    CONTINUOUS_LOAD_FACTOR * continuous_a + noncontinuous_a
}

pub(crate) fn check_power_factor(power_factor: f64) -> Result<()> {
    if power_factor > 0.0 && power_factor <= 1.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput(
            "el factor de potencia debe ser mayor que 0 y hasta 1".into(),
        ))
    }
}
