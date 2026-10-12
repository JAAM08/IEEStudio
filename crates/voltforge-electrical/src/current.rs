//! Corriente nominal y corriente de diseño.

use voltforge_core::units::SQRT_3;
use voltforge_core::validation;
use voltforge_core::{CircuitKind, Result, System};
use voltforge_rules::Profile;

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
    validation::non_negative("la potencia", power_w)?;
    validation::power_factor(power_factor)?;
    let v = kind.reference_voltage(system)?;
    let k = match kind {
        CircuitKind::ThreePhase => SQRT_3,
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => 1.0,
    };
    Ok(power_w / (k * v * power_factor))
}

/// Corriente de diseño: 125 % de la parte continua más 100 % de la no continua.
pub fn design_current(profile: &Profile, continuous_a: f64, noncontinuous_a: f64) -> f64 {
    profile.parameters.continuous_load_factor * continuous_a + noncontinuous_a
}
