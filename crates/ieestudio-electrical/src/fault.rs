//! Corriente de falla por bus infinito.
//!
//! La red de la suministradora se toma sin impedancia: solo la del
//! transformador limita la falla trifásica en sus terminales del secundario.
//! Es la corriente mayor que puede haber ahí, así que un interruptor que la
//! soporta tiene capacidad interruptiva suficiente; uno que no, necesita un
//! estudio de cortocircuito que tome en cuenta la red y los conductores.

use ieestudio_core::{Error, Result};

/// Mayor que cero y finito; un NaN no pasa.
fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.0
}

/// Corriente nominal del secundario de un transformador, en amperes.
/// Trifásico: `I = S / (√3 · V)`; de una fase: `I = S / V`.
pub fn transformer_full_load_current(kva: f64, voltage_v: f64, three_phase: bool) -> Result<f64> {
    if !positive(kva) || !positive(voltage_v) {
        return Err(Error::InvalidInput(
            "la potencia y la tensión del transformador deben ser mayores que cero".into(),
        ));
    }
    let va = kva * 1000.0;
    Ok(if three_phase {
        va / (3f64.sqrt() * voltage_v)
    } else {
        va / voltage_v
    })
}

/// Impedancia con la que se calcula la falla, en por unidad: la de placa
/// reducida por la tolerancia de fabricación.
pub fn effective_impedance_pu(impedance_percent: f64, tolerance: f64) -> Result<f64> {
    if !positive(impedance_percent) {
        return Err(Error::InvalidInput(
            "la impedancia del transformador debe ser mayor que cero".into(),
        ));
    }
    if !(0.0..1.0).contains(&tolerance) {
        return Err(Error::InvalidInput(
            "la tolerancia de la impedancia debe estar entre 0 y 1".into(),
        ));
    }
    Ok(impedance_percent / 100.0 * (1.0 - tolerance))
}

/// Corriente de falla por bus infinito, en amperes: `Icc = In / Z`.
pub fn infinite_bus_current(full_load_a: f64, impedance_pu: f64) -> Result<f64> {
    if !positive(impedance_pu) || full_load_a < 0.0 {
        return Err(Error::InvalidInput(
            "la impedancia debe ser mayor que cero y la corriente no negativa".into(),
        ));
    }
    Ok(full_load_a / impedance_pu)
}

/// Aportación de los motores a la falla, en amperes: un múltiplo de la suma
/// de sus corrientes a plena carga.
pub fn motor_contribution(full_load_currents: &[f64], multiplier: f64) -> f64 {
    multiplier * full_load_currents.iter().sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_example() {
        // 112.5 kVA, 220 V, Z = 3 %: In = 295.23 A e Icc = 9 841 A.
        let i = transformer_full_load_current(112.5, 220.0, true).unwrap();
        assert!((i - 295.236).abs() < 0.01, "{i}");
        let z = effective_impedance_pu(3.0, 0.0).unwrap();
        let icc = infinite_bus_current(i, z).unwrap();
        assert!((icc - 9841.2).abs() < 0.5, "{icc}");
    }

    #[test]
    fn tolerance_raises_the_fault() {
        let i = transformer_full_load_current(75.0, 220.0, true).unwrap();
        let nominal = infinite_bus_current(i, effective_impedance_pu(2.5, 0.0).unwrap()).unwrap();
        let reduced = infinite_bus_current(i, effective_impedance_pu(2.5, 0.1).unwrap()).unwrap();
        assert!((reduced - nominal / 0.9).abs() < 1e-9);
    }

    #[test]
    fn single_phase_uses_the_full_voltage() {
        let i = transformer_full_load_current(25.0, 240.0, false).unwrap();
        assert!((i - 104.1667).abs() < 0.001);
    }

    #[test]
    fn invalid_data_is_rejected() {
        assert!(transformer_full_load_current(0.0, 220.0, true).is_err());
        assert!(transformer_full_load_current(75.0, 0.0, true).is_err());
        assert!(effective_impedance_pu(0.0, 0.1).is_err());
        assert!(effective_impedance_pu(3.0, 1.0).is_err());
        assert!(infinite_bus_current(100.0, 0.0).is_err());
    }

    #[test]
    fn motors_add_a_multiple_of_their_current() {
        assert_eq!(motor_contribution(&[10.0, 20.0], 4.0), 120.0);
        assert_eq!(motor_contribution(&[], 4.0), 0.0);
    }
}
