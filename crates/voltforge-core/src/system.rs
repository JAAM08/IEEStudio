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
    /// Tensión a la que se conecta la carga y contra la que se expresa el
    /// porcentaje de caída de tensión.
    pub fn reference_voltage(self, system: &System) -> Result<f64> {
        let v = match self {
            CircuitKind::LineToNeutral => system.line_to_neutral_v,
            CircuitKind::LineToLine | CircuitKind::ThreePhase => system.line_to_line_v,
        };
        match v {
            Some(v) if v > 0.0 => Ok(v),
            _ => Err(Error::InvalidInput(format!(
                "el sistema no tiene la tensión que requiere un circuito {self:?}"
            ))),
        }
    }

    /// Factor de la fórmula de caída de tensión: √3 en circuitos trifásicos y
    /// 2 (ida y vuelta) en los demás.
    pub fn drop_factor(self) -> f64 {
        match self {
            CircuitKind::ThreePhase => 3f64.sqrt(),
            CircuitKind::LineToNeutral | CircuitKind::LineToLine => 2.0,
        }
    }
}
