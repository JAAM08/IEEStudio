//! Corriente de cortocircuito por el método de impedancias.
//!
//! Se calcula la corriente simétrica de una falla franca en cada punto de la
//! instalación para compararla con la capacidad interruptiva de los
//! interruptores (110-9). Todo se resuelve del lado de la corriente mayor:
//!
//! - La fuente se toma como reactancia pura. Al sumarle la resistencia de los
//!   conductores en cuadratura, la impedancia total sale menor que si tuviera
//!   parte resistiva, y la corriente mayor.
//! - La resistencia de los conductores se lleva de 75 °C (Tabla 9) a 25 °C con
//!   el coeficiente de la Tabla 8, nota 2: un conductor frío deja pasar más
//!   corriente de falla.
//! - En un sistema trifásico se calcula la falla trifásica con la impedancia
//!   de un conductor, que en la red es la mayor; en uno de una fase, la falla
//!   recorre la ida y la vuelta.

use ieestudio_core::units::{m_to_km, SQRT_3};
use ieestudio_core::{Error, Material, RacewayMaterial, Result, Size, System};
use ieestudio_rules::Profile;

use crate::voltage_drop::{temperature_coefficient, TABLE_9_TEMPERATURE_C};

/// Temperatura del conductor para la corriente de falla máxima, en °C.
pub const SHORT_CIRCUIT_TEMPERATURE_C: f64 = 25.0;

/// Impedancia en ohms, parte resistiva y reactiva.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Impedance {
    pub r_ohm: f64,
    pub x_ohm: f64,
}

impl Impedance {
    pub fn magnitude(self) -> f64 {
        self.r_ohm.hypot(self.x_ohm)
    }

    pub fn plus(self, other: Impedance) -> Impedance {
        Impedance {
            r_ohm: self.r_ohm + other.r_ohm,
            x_ohm: self.x_ohm + other.x_ohm,
        }
    }

    pub fn scaled(self, factor: f64) -> Impedance {
        Impedance {
            r_ohm: self.r_ohm * factor,
            x_ohm: self.x_ohm * factor,
        }
    }
}

/// Tensión y recorrido de la falla según el sistema.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FaultBasis {
    pub three_phase: bool,
    /// Tensión que impulsa la falla: de fase a neutro en trifásico; la mayor
    /// del sistema en uno de una fase.
    pub voltage_v: f64,
    /// Veces que la falla recorre cada conductor: 1 en trifásico, 2 en una fase.
    pub loop_factor: f64,
}

/// Un sistema es trifásico si tiene tensión entre fases y su tensión al
/// neutro es la de una estrella, o si no tiene neutro.
pub fn is_three_phase(system: &System) -> bool {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        (Some(_), None) => true,
        (Some(ll), Some(ln)) => (ll / ln - SQRT_3).abs() < 0.05,
        _ => false,
    }
}

pub fn fault_basis(system: &System) -> Result<FaultBasis> {
    let missing = || Error::InvalidInput("el sistema no tiene tensiones".into());
    if is_three_phase(system) {
        let ll = system.line_to_line_v.ok_or_else(missing)?;
        Ok(FaultBasis {
            three_phase: true,
            voltage_v: ll / SQRT_3,
            loop_factor: 1.0,
        })
    } else {
        let v = system
            .line_to_line_v
            .or(system.line_to_neutral_v)
            .ok_or_else(missing)?;
        Ok(FaultBasis {
            three_phase: false,
            voltage_v: v,
            loop_factor: 2.0,
        })
    }
}

/// Corriente de falla en las terminales del secundario de un transformador,
/// con el primario como fuente infinita: `I = S / (√3 · V · Z)` en trifásico
/// y `I = S / (V · Z)` en una fase.
pub fn transformer_fault_a(system: &System, kva: f64, impedance_percent: f64) -> Result<f64> {
    if kva.is_nan() || kva <= 0.0 || impedance_percent.is_nan() || impedance_percent <= 0.0 {
        return Err(Error::InvalidInput(
            "el transformador necesita potencia e impedancia mayores que cero".into(),
        ));
    }
    let basis = fault_basis(system)?;
    let z = impedance_percent / 100.0;
    let va = kva * 1000.0;
    Ok(if basis.three_phase {
        va / (3.0 * basis.voltage_v * z)
    } else {
        va / (basis.voltage_v * z)
    })
}

/// Impedancia de la fuente a partir de su corriente de falla, toda reactiva.
pub fn source_impedance(basis: &FaultBasis, available_a: f64) -> Result<Impedance> {
    if available_a.is_nan() || available_a <= 0.0 {
        return Err(Error::InvalidInput(
            "la corriente de cortocircuito de la fuente debe ser mayor que cero".into(),
        ));
    }
    Ok(Impedance {
        r_ohm: 0.0,
        x_ohm: basis.voltage_v / available_a,
    })
}

/// Impedancia de un conductor de fase del tramo, con su resistencia a 25 °C.
pub fn conductor_impedance(
    profile: &Profile,
    size: &Size,
    material: Material,
    raceway: RacewayMaterial,
    length_m: f64,
    conductors_per_phase: u32,
) -> Result<Impedance> {
    let (r75, x) = profile.impedance(size, material, raceway)?;
    let r = r75
        * (1.0
            + temperature_coefficient(material)
                * (SHORT_CIRCUIT_TEMPERATURE_C - TABLE_9_TEMPERATURE_C));
    let km = m_to_km(length_m) / f64::from(conductors_per_phase.max(1));
    Ok(Impedance {
        r_ohm: r * km,
        x_ohm: x * km,
    })
}

/// Corriente de falla con la impedancia total hasta el punto, que ya incluye
/// la ida y la vuelta si el sistema es de una fase.
pub fn fault_current_a(basis: &FaultBasis, total: Impedance) -> f64 {
    basis.voltage_v / total.magnitude()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueba matemática: transformador de 75 kVA, 3.5 %, 220Y/127 V.
    /// I = 75 000 / (√3 · 220 · 0.035) = 5623.5 A.
    #[test]
    fn transformer_secondary_fault() {
        let i = transformer_fault_a(&System::WYE_220_127, 75.0, 3.5).unwrap();
        assert!((i - 5623.54).abs() < 0.01, "{i}");
        // Una fase, 240/120 V, 25 kVA, 2 %: 25 000 / (240 · 0.02) = 5208.3 A.
        let i = transformer_fault_a(&System::SPLIT_240_120, 25.0, 2.0).unwrap();
        assert!((i - 5208.33).abs() < 0.01, "{i}");
        assert!(transformer_fault_a(&System::WYE_220_127, 0.0, 3.5).is_err());
    }

    /// Prueba de referencia, calculada a mano: 10 kA a 220Y/127 V y 30 m de
    /// 3/0 AWG de cobre en tubo de acero. Tabla 9: R = 0.259 Ω/km a 75 °C,
    /// 0.2172 a 25 °C; X = 0.171 Ω/km.
    /// Zf = j 127.017 / 10 000 = j 0.012702 Ω.
    /// Zc = 0.030 · (0.21717 + j 0.171) = 0.006515 + j 0.005130 Ω.
    /// I = 127.017 / |0.006515 + j 0.017832| = 6690 A.
    #[test]
    fn fault_through_a_feeder() {
        let profile = Profile::nom_001_sede_2012();
        let basis = fault_basis(&System::WYE_220_127).unwrap();
        let zs = source_impedance(&basis, 10_000.0).unwrap();
        assert!((zs.x_ohm - 0.012_702).abs() < 1e-6);
        let zc = conductor_impedance(
            profile,
            &Size::new("3/0"),
            Material::Copper,
            RacewayMaterial::Steel,
            30.0,
            1,
        )
        .unwrap();
        assert!((zc.r_ohm - 0.006_515).abs() < 1e-6, "{}", zc.r_ohm);
        assert!((zc.x_ohm - 0.005_130).abs() < 1e-6, "{}", zc.x_ohm);
        let i = fault_current_a(&basis, zs.plus(zc));
        assert!((i - 6690.0).abs() < 1.0, "{i}");
    }

    /// En un sistema de una fase la falla recorre dos conductores.
    #[test]
    fn single_phase_systems_use_the_loop() {
        let basis = fault_basis(&System::SINGLE_127).unwrap();
        assert!(!basis.three_phase);
        assert_eq!(basis.loop_factor, 2.0);
        assert_eq!(basis.voltage_v, 127.0);
        let basis = fault_basis(&System::SPLIT_240_120).unwrap();
        assert_eq!(basis.voltage_v, 240.0);
        assert!(fault_basis(&System::DELTA_440).unwrap().three_phase);
    }
}
