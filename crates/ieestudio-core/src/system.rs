//! Sistema eléctrico de suministro y tipo de circuito.

use crate::error::{Error, Result};

/// Tensiones nominales del sistema, en volts.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct System {
    /// Tensión entre fases. `None` en un servicio de una fase y neutro.
    pub line_to_line_v: Option<f64>,
    /// Tensión de fase a neutro. `None` en un sistema sin neutro.
    pub line_to_neutral_v: Option<f64>,
}

impl System {
    /// Una fase y neutro, 127 V.
    pub const SINGLE_127: System = System {
        line_to_line_v: None,
        line_to_neutral_v: Some(127.0),
    };
    /// Monofásico de tres hilos, 240/120 V.
    pub const SPLIT_240_120: System = System {
        line_to_line_v: Some(240.0),
        line_to_neutral_v: Some(120.0),
    };
    /// Trifásico en estrella, 220Y/127 V.
    pub const WYE_220_127: System = System {
        line_to_line_v: Some(220.0),
        line_to_neutral_v: Some(127.0),
    };
    /// Trifásico en estrella, 208Y/120 V.
    pub const WYE_208_120: System = System {
        line_to_line_v: Some(208.0),
        line_to_neutral_v: Some(120.0),
    };
    /// Trifásico en estrella, 480Y/277 V.
    pub const WYE_480_277: System = System {
        line_to_line_v: Some(480.0),
        line_to_neutral_v: Some(277.0),
    };
    /// Trifásico de tres hilos, 440 V.
    pub const DELTA_440: System = System {
        line_to_line_v: Some(440.0),
        line_to_neutral_v: None,
    };

    /// `true` si el sistema tiene tres fases. Un servicio de una fase y
    /// neutro no las tiene, y tampoco el monofásico de tres hilos, que se
    /// reconoce porque su tensión entre líneas es el doble de la de línea a
    /// neutro; en un trifásico en estrella es √3 veces. Un sistema sin
    /// neutro se toma como trifásico de tres hilos.
    pub fn has_three_phases(&self) -> bool {
        match (
            positive(self.line_to_line_v),
            positive(self.line_to_neutral_v),
        ) {
            (None, _) => false,
            (Some(_), None) => true,
            // Punto medio entre √3 y 2: de qué relación está más cerca.
            (Some(ll), Some(ln)) => ll / ln < (crate::units::SQRT_3 + 2.0) / 2.0,
        }
    }

    /// `true` si el sistema admite un circuito con esa conexión.
    pub fn allows(&self, kind: CircuitKind) -> bool {
        match kind {
            CircuitKind::LineToNeutral => positive(self.line_to_neutral_v).is_some(),
            CircuitKind::LineToLine => positive(self.line_to_line_v).is_some(),
            CircuitKind::ThreePhase => self.has_three_phases(),
        }
    }

    /// Conexiones que el sistema admite, de menos a más fases.
    pub fn kinds(&self) -> Vec<CircuitKind> {
        CircuitKind::ALL
            .into_iter()
            .filter(|&kind| self.allows(kind))
            .collect()
    }

    /// Número de fases del sistema: 1, 2 o 3.
    pub fn phase_count(&self) -> u32 {
        self.kinds().last().map_or(1, |kind| kind.poles())
    }
}

fn positive(v: Option<f64>) -> Option<f64> {
    v.filter(|v| *v > 0.0)
}

/// Conexión del circuito al sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CircuitKind {
    /// Una fase y neutro (1F-2H).
    LineToNeutral,
    /// Dos fases, con o sin neutro (2F).
    LineToLine,
    /// Tres fases, con o sin neutro (3F).
    ThreePhase,
}

impl CircuitKind {
    /// Todas las conexiones, de menos a más fases.
    pub const ALL: [CircuitKind; 3] = [
        CircuitKind::LineToNeutral,
        CircuitKind::LineToLine,
        CircuitKind::ThreePhase,
    ];

    /// Conductores de fase del circuito.
    pub fn poles(self) -> u32 {
        match self {
            CircuitKind::LineToNeutral => 1,
            CircuitKind::LineToLine => 2,
            CircuitKind::ThreePhase => 3,
        }
    }

    /// Tensión a la que se conecta la carga y contra la que se expresa el
    /// porcentaje de caída de tensión.
    ///
    /// Una conexión que el sistema no tiene devuelve un error: no se calcula
    /// con una tensión que no le corresponde.
    pub fn reference_voltage(self, system: &System) -> Result<f64> {
        if self == CircuitKind::ThreePhase
            && system.line_to_line_v.is_some_and(|v| v > 0.0)
            && !system.has_three_phases()
        {
            return Err(Error::InvalidInput(
                "un sistema monofásico de tres hilos no tiene tres fases".to_string(),
            ));
        }
        let v = match self {
            CircuitKind::LineToNeutral => system.line_to_neutral_v,
            CircuitKind::LineToLine | CircuitKind::ThreePhase => system.line_to_line_v,
        };
        match v {
            Some(v) if v > 0.0 => Ok(v),
            _ => Err(Error::InvalidInput(format!(
                "el sistema elegido no tiene la tensión que requiere un circuito de {}",
                match self {
                    CircuitKind::LineToNeutral => "una fase y neutro",
                    CircuitKind::LineToLine => "dos fases",
                    CircuitKind::ThreePhase => "tres fases",
                }
            ))),
        }
    }

    /// Factor de la fórmula de caída de tensión: √3 en circuitos trifásicos y
    /// 2 (ida y vuelta) en los demás.
    pub fn drop_factor(self) -> f64 {
        match self {
            CircuitKind::ThreePhase => crate::units::SQRT_3,
            CircuitKind::LineToNeutral | CircuitKind::LineToLine => 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CircuitKind::{LineToLine, LineToNeutral, ThreePhase};

    #[test]
    fn single_phase_two_wire_only_allows_line_to_neutral() {
        assert_eq!(System::SINGLE_127.kinds(), vec![LineToNeutral]);
        assert_eq!(System::SINGLE_127.phase_count(), 1);
    }

    #[test]
    fn split_phase_never_allows_three_phases() {
        let system = System::SPLIT_240_120;
        assert_eq!(system.kinds(), vec![LineToNeutral, LineToLine]);
        assert_eq!(system.phase_count(), 2);
        assert_eq!(LineToNeutral.reference_voltage(&system).unwrap(), 120.0);
        assert_eq!(LineToLine.reference_voltage(&system).unwrap(), 240.0);
        let error = ThreePhase.reference_voltage(&system).unwrap_err();
        assert!(error.to_string().contains("no tiene tres fases"), "{error}");
    }

    #[test]
    fn wye_systems_allow_every_connection() {
        for system in [
            System::WYE_220_127,
            System::WYE_208_120,
            System::WYE_480_277,
        ] {
            assert_eq!(system.kinds(), CircuitKind::ALL.to_vec());
            assert_eq!(system.phase_count(), 3);
            assert!(ThreePhase.reference_voltage(&system).is_ok());
        }
    }

    #[test]
    fn three_wire_system_has_no_line_to_neutral() {
        let system = System::DELTA_440;
        assert_eq!(system.kinds(), vec![LineToLine, ThreePhase]);
        assert!(LineToNeutral.reference_voltage(&system).is_err());
        assert_eq!(ThreePhase.reference_voltage(&system).unwrap(), 440.0);
    }
}
