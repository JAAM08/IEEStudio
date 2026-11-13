//! Sistema eléctrico de suministro y tipo de circuito.

use crate::error::{Error, Result};

/// Topología declarada del sistema: número de fases, número de hilos y
/// conexión (decisión 0030). Nunca se deduce de las tensiones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Topology {
    /// Una fase, dos hilos: fase y neutro, o dos fases sin neutro (1F-2H).
    SinglePhaseTwoWire,
    /// Monofásico de tres hilos con derivación central (1F-3H).
    SinglePhaseThreeWire,
    /// Trifásico de tres hilos en delta (3F-3H).
    ThreePhaseDelta,
    /// Trifásico de cuatro hilos en estrella (3F-4H).
    ThreePhaseWye,
}

impl Topology {
    pub const ALL: [Topology; 4] = [
        Topology::SinglePhaseTwoWire,
        Topology::SinglePhaseThreeWire,
        Topology::ThreePhaseDelta,
        Topology::ThreePhaseWye,
    ];

    pub fn phases(self) -> u32 {
        match self {
            Topology::SinglePhaseTwoWire | Topology::SinglePhaseThreeWire => 1,
            Topology::ThreePhaseDelta | Topology::ThreePhaseWye => 3,
        }
    }

    pub fn wires(self) -> u32 {
        match self {
            Topology::SinglePhaseTwoWire => 2,
            Topology::SinglePhaseThreeWire | Topology::ThreePhaseDelta => 3,
            Topology::ThreePhaseWye => 4,
        }
    }

    /// `1F-3H`.
    pub fn code(self) -> &'static str {
        match self {
            Topology::SinglePhaseTwoWire => "1F-2H",
            Topology::SinglePhaseThreeWire => "1F-3H",
            Topology::ThreePhaseDelta => "3F-3H",
            Topology::ThreePhaseWye => "3F-4H",
        }
    }

    /// `1F-3H, monofásico de tres hilos`.
    pub fn label(self) -> &'static str {
        match self {
            Topology::SinglePhaseTwoWire => "1F-2H, una fase y dos hilos",
            Topology::SinglePhaseThreeWire => "1F-3H, monofásico de tres hilos",
            Topology::ThreePhaseDelta => "3F-3H, trifásico en delta",
            Topology::ThreePhaseWye => "3F-4H, trifásico en estrella",
        }
    }

    /// `true` si la topología lleva justo esas tensiones: entre fases
    /// (`ll`) y de fase a neutro (`ln`).
    pub fn takes(self, ll: bool, ln: bool) -> bool {
        match self {
            Topology::SinglePhaseTwoWire => ll != ln,
            Topology::SinglePhaseThreeWire | Topology::ThreePhaseWye => ll && ln,
            Topology::ThreePhaseDelta => ll && !ln,
        }
    }

    /// Relación entre la tensión entre fases y la de fase a neutro que pide
    /// la topología, si lleva las dos.
    pub fn voltage_ratio(self) -> Option<f64> {
        match self {
            Topology::SinglePhaseThreeWire => Some(2.0),
            Topology::ThreePhaseWye => Some(crate::units::SQRT_3),
            Topology::SinglePhaseTwoWire | Topology::ThreePhaseDelta => None,
        }
    }
}

/// Un sistema que no se puede usar tal como está declarado.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemIssue {
    /// `true` si los datos se contradicen («Dato inválido»); `false` si
    /// falta declarar la topología («Sin evaluar»).
    pub invalid: bool,
    pub reason: String,
}

/// Tensiones nominales del sistema, en volts, y su topología declarada.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct System {
    /// Tensión entre fases. `None` en un servicio de una fase y neutro.
    pub line_to_line_v: Option<f64>,
    /// Tensión de fase a neutro. `None` en un sistema sin neutro.
    pub line_to_neutral_v: Option<f64>,
    /// Topología declarada. `None` en un sistema de un archivo anterior cuyas
    /// tensiones no son de los que ofrece la interfaz: lo que dependa de ella
    /// queda sin evaluar.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub topology: Option<Topology>,
}

impl System {
    /// Una fase y neutro, 127 V.
    pub const SINGLE_127: System = System {
        line_to_line_v: None,
        line_to_neutral_v: Some(127.0),
        topology: Some(Topology::SinglePhaseTwoWire),
    };
    /// Monofásico de tres hilos, 240/120 V.
    pub const SPLIT_240_120: System = System {
        line_to_line_v: Some(240.0),
        line_to_neutral_v: Some(120.0),
        topology: Some(Topology::SinglePhaseThreeWire),
    };
    /// Trifásico en estrella, 220Y/127 V.
    pub const WYE_220_127: System = System {
        line_to_line_v: Some(220.0),
        line_to_neutral_v: Some(127.0),
        topology: Some(Topology::ThreePhaseWye),
    };
    /// Trifásico en estrella, 208Y/120 V.
    pub const WYE_208_120: System = System {
        line_to_line_v: Some(208.0),
        line_to_neutral_v: Some(120.0),
        topology: Some(Topology::ThreePhaseWye),
    };
    /// Trifásico en estrella, 480Y/277 V.
    pub const WYE_480_277: System = System {
        line_to_line_v: Some(480.0),
        line_to_neutral_v: Some(277.0),
        topology: Some(Topology::ThreePhaseWye),
    };
    /// Trifásico de tres hilos, 440 V.
    pub const DELTA_440: System = System {
        line_to_line_v: Some(440.0),
        line_to_neutral_v: None,
        topology: Some(Topology::ThreePhaseDelta),
    };

    /// Los sistemas que ofrece la interfaz, cada uno con la topología con
    /// que se muestra.
    pub const OFFERED: [System; 6] = [
        System::SINGLE_127,
        System::SPLIT_240_120,
        System::WYE_220_127,
        System::WYE_208_120,
        System::WYE_480_277,
        System::DELTA_440,
    ];

    /// Topología de un sistema de un archivo anterior, que no la guardaba:
    /// solo si sus tensiones son exactamente las de un sistema de la
    /// interfaz (decisión 0030, T-7). Cualquier otro queda sin declarar.
    pub fn offered_topology(&self) -> Option<Topology> {
        System::OFFERED
            .iter()
            .find(|s| {
                s.line_to_line_v == self.line_to_line_v
                    && s.line_to_neutral_v == self.line_to_neutral_v
            })
            .and_then(|s| s.topology)
    }

    /// Topologías que llevan las tensiones del sistema, para declararla. No
    /// revisa la relación: una que no corresponda sigue como dato inválido.
    pub fn candidate_topologies(&self) -> Vec<Topology> {
        let (ll, ln) = (
            self.line_to_line_v.is_some(),
            self.line_to_neutral_v.is_some(),
        );
        Topology::ALL
            .into_iter()
            .filter(|t| t.takes(ll, ln))
            .collect()
    }

    /// `226/120 V`, `127 V`.
    pub fn voltages_label(&self) -> String {
        match (self.line_to_line_v, self.line_to_neutral_v) {
            (Some(ll), Some(ln)) => format!("{ll}/{ln} V"),
            (Some(v), None) | (None, Some(v)) => format!("{v} V"),
            (None, None) => "sin tensión".into(),
        }
    }

    /// `true` si el sistema declara tres fases. Sin topología declarada no
    /// se supone.
    pub fn has_three_phases(&self) -> bool {
        self.topology.is_some_and(|t| t.phases() == 3)
    }

    /// `true` si el sistema declara cuatro hilos en estrella.
    pub fn is_wye(&self) -> bool {
        self.topology == Some(Topology::ThreePhaseWye)
    }

    /// Revisa que las tensiones correspondan a la topología declarada
    /// (decisión 0030, T-2 y T-3). `tolerance` es la del perfil, en por
    /// unidad. Devuelve la topología si se puede usar.
    pub fn check(&self, tolerance: f64) -> std::result::Result<Topology, SystemIssue> {
        let invalid = |reason: String| SystemIssue {
            invalid: true,
            reason,
        };
        let (ll, ln) = (self.line_to_line_v, self.line_to_neutral_v);
        if ll.is_some_and(|v| !(v.is_finite() && v > 0.0))
            || ln.is_some_and(|v| !(v.is_finite() && v > 0.0))
        {
            return Err(invalid("las tensiones deben ser mayores que cero".into()));
        }
        let fits = |ratio: f64, expected: f64| (ratio / expected - 1.0).abs() <= tolerance;
        let percent = tolerance * 100.0;
        let Some(topology) = self.topology else {
            if let (Some(ll), Some(ln)) = (ll, ln) {
                let ratio = ll / ln;
                if !fits(ratio, crate::units::SQRT_3) && !fits(ratio, 2.0) {
                    return Err(invalid(format!(
                        "la relación entre {ll} V y {ln} V es {ratio:.3}; no corresponde a un 3F-4H en estrella (√3 = 1.732) ni a un 1F-3H (2), con ±{percent} %"
                    )));
                }
            }
            return Err(SystemIssue {
                invalid: false,
                reason: "falta declarar la topología del sistema (fases, hilos y conexión)".into(),
            });
        };
        if !topology.takes(ll.is_some(), ln.is_some()) {
            let what = match topology {
                Topology::SinglePhaseTwoWire => "una sola tensión: de fase a neutro o entre fases",
                Topology::SinglePhaseThreeWire | Topology::ThreePhaseWye => {
                    "tensión entre fases y de fase a neutro"
                }
                Topology::ThreePhaseDelta => "solo tensión entre fases",
            };
            return Err(invalid(format!(
                "un sistema {} lleva {what}",
                topology.code()
            )));
        }
        if let (Some(expected), Some(ll), Some(ln)) = (topology.voltage_ratio(), ll, ln) {
            let ratio = ll / ln;
            if !fits(ratio, expected) {
                return Err(invalid(format!(
                    "la relación entre {ll} V y {ln} V es {ratio:.3}; un {} pide {expected:.3}, con ±{percent} %",
                    topology.code()
                )));
            }
        }
        Ok(topology)
    }

    /// `true` si el sistema admite un circuito con esa conexión: la tensión
    /// que pide y, para tres fases, tres fases declaradas.
    pub fn allows(&self, kind: CircuitKind) -> bool {
        match kind {
            CircuitKind::LineToNeutral => positive(self.line_to_neutral_v).is_some(),
            CircuitKind::LineToLine => positive(self.line_to_line_v).is_some(),
            CircuitKind::ThreePhase => {
                positive(self.line_to_line_v).is_some() && self.has_three_phases()
            }
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
            return Err(match system.topology {
                Some(t) => {
                    Error::InvalidInput(format!("un sistema {} no tiene tres fases", t.code()))
                }
                None => Error::MissingData(
                    "falta declarar la topología del sistema para saber si tiene tres fases".into(),
                ),
            });
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
        assert!(
            error.to_string().contains("1F-3H no tiene tres fases"),
            "{error}"
        );
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
    const TOLERANCE: f64 = 0.01;

    fn with(ll: Option<f64>, ln: Option<f64>, topology: Option<Topology>) -> System {
        System {
            line_to_line_v: ll,
            line_to_neutral_v: ln,
            topology,
        }
    }

    #[test]
    fn the_offered_systems_are_valid() {
        for system in System::OFFERED {
            assert_eq!(system.check(TOLERANCE), Ok(system.topology.unwrap()));
        }
    }

    #[test]
    fn the_ratio_limit_is_one_percent_of_the_expected() {
        // 1F-3H: 2 ± 1 % es de 1.98 a 2.02.
        let split = |ll| with(Some(ll), Some(100.0), Some(Topology::SinglePhaseThreeWire));
        assert!(split(201.9).check(TOLERANCE).is_ok());
        assert!(split(198.1).check(TOLERANCE).is_ok());
        assert!(split(202.1).check(TOLERANCE).unwrap_err().invalid);
        assert!(split(197.9).check(TOLERANCE).unwrap_err().invalid);
        // 3F-4H: √3 ± 1 % es de 1.7147 a 1.7494.
        let wye = |ll| with(Some(ll), Some(100.0), Some(Topology::ThreePhaseWye));
        assert!(wye(174.9).check(TOLERANCE).is_ok());
        assert!(wye(171.5).check(TOLERANCE).is_ok());
        assert!(wye(175.0).check(TOLERANCE).unwrap_err().invalid);
        assert!(wye(171.4).check(TOLERANCE).unwrap_err().invalid);
    }

    #[test]
    fn a_ratio_that_fits_no_topology_is_invalid_even_undeclared() {
        let system = with(Some(226.0), Some(120.0), None);
        let issue = system.check(TOLERANCE).unwrap_err();
        assert!(issue.invalid);
        assert_eq!(
            issue.reason,
            "la relación entre 226 V y 120 V es 1.883; no corresponde a un 3F-4H en estrella (√3 = 1.732) ni a un 1F-3H (2), con ±1 %"
        );
        // Declarado 1F-3H tampoco se acepta, y nunca se toma como 240/120.
        let declared = with(
            Some(226.0),
            Some(120.0),
            Some(Topology::SinglePhaseThreeWire),
        );
        assert_eq!(
            declared.check(TOLERANCE).unwrap_err().reason,
            "la relación entre 226 V y 120 V es 1.883; un 1F-3H pide 2.000, con ±1 %"
        );
    }

    #[test]
    fn an_undeclared_system_is_not_classified() {
        let system = with(Some(230.0), Some(132.8), None);
        let issue = system.check(TOLERANCE).unwrap_err();
        assert!(!issue.invalid);
        assert!(!system.has_three_phases());
        assert!(!system.is_wye());
        assert_eq!(system.kinds(), vec![LineToNeutral, LineToLine]);
        let error = ThreePhase.reference_voltage(&system).unwrap_err();
        assert!(error.to_string().contains("falta declarar la topología"));
    }

    #[test]
    fn declared_phases_allow_three_phase_circuits_even_with_a_bad_ratio() {
        let system = with(Some(226.0), Some(120.0), Some(Topology::ThreePhaseWye));
        assert!(system.check(TOLERANCE).unwrap_err().invalid);
        assert!(system.allows(ThreePhase));
    }

    #[test]
    fn the_topology_asks_for_its_voltages() {
        let delta_with_neutral = with(Some(440.0), Some(254.0), Some(Topology::ThreePhaseDelta));
        assert_eq!(
            delta_with_neutral.check(TOLERANCE).unwrap_err().reason,
            "un sistema 3F-3H lleva solo tensión entre fases"
        );
        let wye_without_neutral = with(Some(220.0), None, Some(Topology::ThreePhaseWye));
        assert!(wye_without_neutral.check(TOLERANCE).unwrap_err().invalid);
        // Una fase a 240 V entre fases, sin neutro: 1F-2H, no trifásico.
        let two_wire = with(Some(240.0), None, Some(Topology::SinglePhaseTwoWire));
        assert!(two_wire.check(TOLERANCE).is_ok());
        assert_eq!(two_wire.kinds(), vec![LineToLine]);
    }

    #[test]
    fn previous_files_take_the_topology_only_from_offered_systems() {
        assert_eq!(
            with(Some(220.0), Some(127.0), None).offered_topology(),
            Some(Topology::ThreePhaseWye)
        );
        assert_eq!(
            with(Some(440.0), None, None).offered_topology(),
            Some(Topology::ThreePhaseDelta)
        );
        assert_eq!(
            with(Some(230.0), Some(132.8), None).offered_topology(),
            None
        );
        assert_eq!(
            with(Some(226.0), Some(120.0), None).offered_topology(),
            None
        );
    }
}
