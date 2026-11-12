//! Capacidad interruptiva con la corriente de falla estimada (decisiones 0023
//! y 0028).
//!
//! La instalación se divide en **zonas de tensión**: la de la fuente y la del
//! secundario de cada transformador intermedio. Todas las barras de una zona
//! ven la misma falla, porque no se descuenta la impedancia de los
//! alimentadores.
//!
//! - **Base.** En la zona de la fuente, el dato de la suministradora o, si no
//!   se tiene, el bus infinito del transformador de la acometida. En la zona
//!   de un transformador intermedio, su bus infinito: no depende de la fuente,
//!   así que se evalúa aunque la fuente no tenga datos.
//! - **Motores.** Los de la misma zona aportan completos: `k · Ipc`. Los de
//!   una zona aguas abajo llegan a través de la impedancia de su
//!   transformador, referidos a la tensión de estas barras. Los de una zona
//!   aguas arriba no aportan a un secundario calculado por bus infinito: la
//!   red infinita ya fija el primario.
//!
//! - **Fase a neutro.** En el secundario de un transformador monofásico con
//!   derivación central (1F-3H) la falla de fase a neutro supera a la de fase
//!   a fase: por bus infinito se multiplica por el factor del perfil
//!   (decisión 0029). El dato de la suministradora no se multiplica: ya es la
//!   corriente máxima disponible en el punto de conexión.
//!
//! Cada valor es una cota superior: un interruptor que lo alcanza cumple; uno
//! que no, deja el resultado no concluyente. Un dato que falta nunca se
//! reemplaza: la zona queda sin evaluar con su causa o, si es la fuente sin
//! ningún dato, fuera de alcance (decisión 0018).

use std::collections::HashMap;

use ieestudio_core::{Error, System};
use ieestudio_electrical::fault::{
    effective_impedance_pu, infinite_bus_current, motor_contribution, transformer_full_load_current,
};
use ieestudio_rules::{Profile, Rule};
use serde::Serialize;

use crate::calc::{Check, PanelResult, RunResult, Status};
use crate::evaluation::{Datum, Evaluation, Trace};
use crate::model::Project;
use crate::procedure::Step;
use crate::transformer::Rating;

/// De dónde sale la corriente de falla de unas barras.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FaultOrigin {
    /// Dato de la suministradora en baja tensión, en el punto de conexión.
    Supplier,
    /// Dato de la suministradora en media tensión, combinado con la
    /// impedancia del transformador de la acometida (decisión 0029, D-2).
    SupplierPrimary,
    /// Bus infinito del transformador de la acometida.
    Substation,
    /// Bus infinito del transformador intermedio que alimenta el tablero.
    Transformer { panel: String },
}

impl FaultOrigin {
    /// Se estimó con la red sin impedancia.
    pub fn infinite_bus(&self) -> bool {
        matches!(
            self,
            FaultOrigin::Substation | FaultOrigin::Transformer { .. }
        )
    }

    /// `por bus infinito` o `con el dato de la suministradora`: la hipótesis
    /// con que se obtuvo, para no presentar dos métodos como si fueran uno.
    pub fn basis(&self) -> &'static str {
        match self {
            FaultOrigin::Supplier => "con el dato de la suministradora",
            FaultOrigin::SupplierPrimary => {
                "con el dato de la suministradora en media tensión y el transformador"
            }
            _ => "por bus infinito",
        }
    }

    /// Es una cota superior y no el dato tal cual de la suministradora.
    pub fn upper_bound(&self) -> bool {
        !matches!(self, FaultOrigin::Supplier)
    }
}

/// Corriente de falla estimada en unas barras.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BusFault {
    /// Sin la aportación de motores, en kA.
    pub base_ka: f64,
    /// Aportación de los motores, en kA.
    pub motor_ka: f64,
    /// Lo que se compara con la capacidad interruptiva, en kA.
    pub total_ka: f64,
    pub origin: FaultOrigin,
    /// Procedimiento, para la interfaz y la memoria.
    pub steps: Vec<Step>,
    /// Motores que aportan a estas barras. `None` si ninguno.
    pub motors: Option<MotorContribution>,
    /// Motores que aportarían aquí pero no tienen datos: la falla está
    /// incompleta y nada que se compare con ella puede cumplir.
    pub incomplete: Vec<String>,
    /// «Otras cargas» que parecen motor o aire acondicionado y aportarían
    /// aquí si lo son: mientras no se confirmen, nada que se compare con esta
    /// falla puede cumplir (decisión 0029, D-3).
    pub suspected: Vec<String>,
    /// En un secundario 1F-3H por bus infinito, `base_ka` es la falla de
    /// fase a neutro; aquí queda la de fase a fase y el factor.
    pub line_to_neutral: Option<LineToNeutral>,
}

/// Falla de fase a neutro en el secundario de un transformador monofásico
/// con derivación central (decisión 0029).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LineToNeutral {
    /// Veces la falla de fase a fase (perfil).
    pub factor: f64,
    /// Falla de fase a fase en las terminales, en kA.
    pub line_to_line_ka: f64,
}

impl BusFault {
    /// `por bus infinito, de fase a neutro`: la hipótesis con que se obtuvo.
    pub fn basis(&self) -> String {
        match self.line_to_neutral {
            Some(_) => format!("{}, de fase a neutro", self.origin.basis()),
            None => self.origin.basis().to_string(),
        }
    }

    /// Válida, o no concluyente si le faltan motores.
    pub fn evaluation(&self) -> Evaluation {
        if self.incomplete.is_empty() && self.suspected.is_empty() {
            Evaluation::Valid
        } else {
            Evaluation::Inconclusive
        }
    }

    /// Por qué no puede cumplir aunque un interruptor la alcance: `la falla
    /// no incluye 1 motor sin datos`, `1 carga que parece motor sin
    /// confirmar`. `None` si la falla está completa.
    pub fn open_items(&self) -> Option<String> {
        let mut parts = Vec::new();
        if !self.incomplete.is_empty() {
            parts.push(format!(
                "la falla no incluye {} sin datos",
                motors_phrase(self.incomplete.len())
            ));
        }
        if !self.suspected.is_empty() {
            parts.push(suspected_phrase(self.suspected.len()));
        }
        (!parts.is_empty()).then(|| parts.join(" y "))
    }
}

/// `1 carga que parece motor sin confirmar`, `2 cargas que parecen…`.
fn suspected_phrase(n: usize) -> String {
    if n == 1 {
        "1 carga que parece motor o aire acondicionado está sin confirmar".to_string()
    } else {
        format!("{n} cargas que parecen motor o aire acondicionado están sin confirmar")
    }
}

/// Un motor o compresor de la misma zona de tensión.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MotorShare {
    /// `TG-1 · C-06 Bomba de agua`.
    pub label: String,
    /// Corriente a plena carga, en amperes.
    pub full_load_a: f64,
}

/// Aportación de los motores de otra zona que llega a través de un
/// transformador intermedio.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransferShare {
    /// Tablero que alimenta el transformador.
    pub panel: String,
    /// Aportación de los motores en su secundario, en amperes a esa tensión.
    pub downstream_a: f64,
    /// Lo que llega a estas barras, en amperes a su tensión.
    pub contribution_a: f64,
}

/// Aportación de los motores a la falla de unas barras.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MotorContribution {
    pub multiplier: f64,
    /// Motores de la misma zona: aportan `multiplier · Ipc`.
    pub motors: Vec<MotorShare>,
    /// Motores de zonas aguas abajo, a través de cada transformador.
    pub through: Vec<TransferShare>,
    /// Total, en amperes.
    pub total_a: f64,
}

impl MotorContribution {
    /// `2 motores`, `1 motor y los de T-2`.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        match self.motors.len() {
            0 => {}
            1 => parts.push("1 motor".to_string()),
            n => parts.push(format!("{n} motores")),
        }
        for t in &self.through {
            parts.push(format!(
                "los motores detrás del transformador de {}",
                t.panel
            ));
        }
        parts.join(" y ")
    }
}

/// Motor o compresor sin los datos para saber su corriente.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IncompleteMotor {
    pub panel: String,
    pub circuit: String,
    /// `TG-1 · C-06 Bomba de agua`.
    pub label: String,
    /// Qué le falta.
    pub missing: String,
}

/// Por qué no hay corriente de falla en unas barras.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum IssueKind {
    /// La fuente no tiene ningún dato: la capacidad interruptiva de lo que
    /// depende de ella queda fuera de alcance (decisión 0018).
    OutOfScope,
    /// Se capturó un transformador o la acometida, pero falta un dato:
    /// queda sin evaluar.
    Incomplete,
    /// Un dato capturado es inconsistente (potencia o impedancia que no es
    /// mayor que cero, conexión que el sistema no tiene).
    Invalid,
}

impl IssueKind {
    pub fn evaluation(self) -> Evaluation {
        match self {
            IssueKind::Invalid => Evaluation::Invalid,
            _ => Evaluation::NotEvaluated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FaultIssue {
    pub kind: IssueKind,
    /// `falta la potencia del transformador de T-2`.
    pub reason: String,
}

/// Resultado del estudio para todo el proyecto.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FaultStudy {
    /// En el punto de conexión, donde sale el alimentador principal. `None`
    /// si la fuente no tiene datos o están incompletos: ver `source_issue`.
    pub source: Option<BusFault>,
    pub source_issue: Option<FaultIssue>,
    /// Cómo se obtuvo la falla del punto de conexión, o por qué no.
    pub source_trace: Trace,
    /// Fracción en que se redujo la impedancia de placa de los
    /// transformadores.
    pub impedance_tolerance: f64,
    /// Veces la corriente a plena carga con que aporta un motor (perfil).
    pub multiplier: f64,
    /// Veces la falla de fase a fase con que se estima la de fase a neutro en
    /// un secundario 1F-3H (perfil, decisión 0029).
    pub line_to_neutral_factor: f64,
    /// Motores sin datos de todo el proyecto.
    pub incomplete_motors: Vec<IncompleteMotor>,
    /// Lugares cuya falla depende de una fuente sin datos: su capacidad
    /// interruptiva queda fuera de alcance.
    pub out_of_scope: Vec<String>,
    /// Capacidad interruptiva resumida por lugar: el punto de conexión y
    /// las barras de cada tablero con interruptores.
    pub groups: Vec<InterruptingGroup>,
}

/// Nombre de la verificación de capacidad interruptiva.
pub const INTERRUPTING: &str = "Capacidad interruptiva";

/// Un interruptor comparado con la falla del lugar donde está instalado.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BreakerVerdict {
    /// Tablero al que pertenece el tramo.
    pub panel: String,
    /// `None` si es el alimentador o el secundario de ese tablero.
    pub circuit: Option<String>,
    /// `C-02 · Contactos cocina`, `Alimentador de TG-1`.
    pub label: String,
    /// Capacidad interruptiva que se le tomó, propia o la del tablero.
    pub ka: Option<f64>,
    pub status: Status,
}

/// La capacidad interruptiva de los interruptores instalados en un mismo
/// lugar, que ven la misma corriente de falla. Es una sola verificación:
/// cumple si todos la alcanzan.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterruptingGroup {
    /// Tablero en cuyas barras están los interruptores. `None` en el punto
    /// de conexión, de donde sale el alimentador principal.
    pub panel: Option<String>,
    /// `las barras de TG-1`, `el punto de conexión`.
    pub place: String,
    /// Corriente de falla del lugar, en kA. `None` si no se pudo estimar.
    pub fault_ka: Option<f64>,
    /// Se estimó por bus infinito (cota superior) y no con el dato de la
    /// suministradora.
    pub infinite_bus: bool,
    /// Es la falla de fase a neutro de un secundario 1F-3H (decisión 0029).
    pub line_to_neutral: bool,
    /// De dónde sale la falla. `None` si no se pudo estimar.
    pub origin: Option<FaultOrigin>,
    /// Por qué no hay falla estimada, si no la hay.
    pub issue: Option<String>,
    /// Motores sin datos que aportarían aquí.
    pub incomplete_motors: usize,
    /// Cargas sin confirmar que parecen motor y aportarían aquí.
    pub suspected_loads: usize,
    /// Capacidad interruptiva capturada para el tablero, en kA.
    pub board_ka: Option<f64>,
    pub status: Status,
    /// Qué tan completa es la verificación: la de la falla y la de la
    /// comparación con los interruptores.
    pub evaluation: Evaluation,
    /// Cómo se obtuvo la falla del lugar y qué falta.
    pub trace: Trace,
    pub breakers: Vec<BreakerVerdict>,
    pub references: Vec<ieestudio_rules::RuleReference>,
}

impl InterruptingGroup {
    /// «Cumple»; «No concluyente» si hay falla estimada y un interruptor no la
    /// alcanza o la falla no incluye motores sin datos; «Sin evaluar» si
    /// falta un dato para comparar.
    pub fn label(&self) -> &'static str {
        match (self.evaluation, self.status) {
            (Evaluation::Valid, Status::Complies) => "Cumple",
            (Evaluation::Valid, Status::Fails) => "No cumple",
            (Evaluation::Valid, _) => "Sin evaluar",
            (other, _) => other.label(),
        }
    }

    /// Interruptores sin dato de capacidad interruptiva.
    pub fn missing(&self) -> usize {
        self.breakers.iter().filter(|b| b.ka.is_none()).count()
    }

    /// Interruptores con dato que no alcanza la falla estimada.
    pub fn short(&self) -> usize {
        match self.fault_ka {
            Some(fault) => self
                .breakers
                .iter()
                .filter(|b| b.ka.is_some_and(|ka| ka < fault))
                .count(),
            None => 0,
        }
    }

    /// Una frase con lo que falta: para Revisar y la memoria.
    pub fn summary(&self) -> String {
        let Some(fault_ka) = self.fault_ka else {
            return format!(
                "sin evaluar en {}: no se pudo estimar la falla porque {}",
                self.place,
                self.issue.as_deref().unwrap_or("faltan datos")
            );
        };
        let n = self.breakers.len();
        let (missing, short) = (self.missing(), self.short());
        let count = |k: usize| {
            if k == n && n > 1 {
                format!("los {n} interruptores")
            } else if k == 1 {
                "1 interruptor".to_string()
            } else {
                format!("{k} interruptores")
            }
        };
        let fault = format!("{fault_ka:.2} kA");
        let (basis, method) = match (&self.origin, self.line_to_neutral) {
            (Some(FaultOrigin::Supplier), _) | (None, _) => (
                "estimados con el dato de la suministradora, sin descontar los alimentadores",
                "un método conservador (dato de la suministradora)",
            ),
            (Some(FaultOrigin::SupplierPrimary), _) => (
                "estimados con el dato de la suministradora en media tensión y el transformador, que es una cota superior",
                "un método conservador (dato en media tensión y transformador)",
            ),
            (_, true) => (
                "estimados por bus infinito, de fase a neutro, que es una cota superior",
                "un método conservador (bus infinito, de fase a neutro)",
            ),
            (_, false) => (
                "estimados por bus infinito, que es una cota superior",
                "un método conservador (bus infinito)",
            ),
        };
        // «del punto de conexión», no «de el punto de conexión».
        let of_place = match self.place.strip_prefix("el ") {
            Some(rest) => format!("del {rest}"),
            None => format!("de {}", self.place),
        };
        match (missing, short) {
            (0, 0) if self.incomplete_motors > 0 || self.suspected_loads > 0 => {
                let mut open = Vec::new();
                if self.incomplete_motors > 0 {
                    open.push(format!(
                        "la falla no incluye {} sin datos",
                        motors_phrase(self.incomplete_motors)
                    ));
                }
                if self.suspected_loads > 0 {
                    open.push(suspected_phrase(self.suspected_loads));
                }
                format!(
                    "no concluyente: {} {of_place} {} los {fault}, pero {}",
                    count(n),
                    if n == 1 { "alcanza" } else { "alcanzan" },
                    open.join(" y ")
                )
            }
            (0, 0) => format!(
                "{} {of_place} {} los {fault} de falla estimada; cumple con {method}",
                capitalize(&count(n)),
                if n == 1 { "alcanza" } else { "alcanzan" }
            ),
            (_, 0) => format!(
                "falta la capacidad interruptiva de {} en {}; la falla estimada es de {fault}",
                count(missing),
                self.place
            ),
            (0, _) => format!(
                "no concluyente: {} {of_place} no {} los {fault} {basis}; requiere estudio de cortocircuito",
                count(short),
                if short == 1 { "alcanza" } else { "alcanzan" }
            ),
            _ => format!(
                "no concluyente: en {}, {} no {} los {fault} {basis} y a {} le falta el dato; requiere estudio de cortocircuito",
                self.place,
                count(short),
                if short == 1 { "alcanza" } else { "alcanzan" },
                count(missing)
            ),
        }
    }
}

fn motors_phrase(n: usize) -> String {
    if n == 1 {
        "la aportación de 1 motor".into()
    } else {
        format!("la aportación de {n} motores")
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Texto de un error sin el prefijo «dato inválido».
fn reason(e: &Error) -> String {
    match e {
        Error::InvalidInput(msg) => msg.clone(),
        other => other.to_string(),
    }
}

/// Si el sistema es trifásico y la tensión con la que se calcula la
/// corriente del transformador de la acometida.
fn transformer_voltage(system: &System) -> Option<(f64, bool)> {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        // Monofásico de tres hilos: el transformador es de una fase a la
        // tensión entre fases.
        (Some(ll), Some(ln)) if (ll / ln - 2.0).abs() < 0.1 => Some((ll, false)),
        (Some(ll), _) => Some((ll, true)),
        (None, Some(ln)) => Some((ln, false)),
        (None, None) => None,
    }
}

/// Secundario de un transformador monofásico con derivación central: tres
/// hilos, con tensión entre fases y al neutro, y una sola fase. Usa el mismo
/// criterio que la corriente del transformador.
fn center_tapped(system: &System) -> bool {
    system.line_to_line_v.is_some()
        && system.line_to_neutral_v.is_some()
        && transformer_voltage(system).is_some_and(|(_, three_phase)| !three_phase)
}

/// Paso de la falla de fase a neutro de un secundario 1F-3H.
fn line_to_neutral_step(profile: &Profile, line_to_line_ka: f64, factor: f64) -> Step {
    Step {
        title: "Falla de fase a neutro (1F-3H)".into(),
        formula: Some("Icc F-N = k · Icc F-F".into()),
        substitution: format!(
            "Icc F-N = {factor} · {line_to_line_ka:.2} kA = {:.2} kA",
            factor * line_to_line_ka
        ),
        note: Some(
            "En un transformador monofásico con derivación central la falla de fase a neutro solo recorre media bobina y supera a la de fase a fase: en las terminales es de 1.33 a 1.67 veces. Se toma el máximo y se usa en toda la zona, porque los conductores solo la reducen."
                .into(),
        ),
        references: vec![profile.reference(Rule::LineToNeutralFault)],
    }
}

/// Pasos y corriente de falla por bus infinito en el secundario de un
/// transformador, con su corriente nominal ya calculada.
fn infinite_bus_steps(
    profile: &Profile,
    rating: &Rating,
    impedance_percent: f64,
    what: &str,
    from_table: bool,
) -> Result<(f64, Vec<Step>), String> {
    // El mínimo de la tabla de la norma ya incluye la tolerancia.
    let tolerance = if from_table {
        0.0
    } else {
        profile.parameters.fault_impedance_tolerance
    };
    let z = effective_impedance_pu(impedance_percent, tolerance).map_err(|e| reason(&e))?;
    let icc = infinite_bus_current(rating.rated_a, z).map_err(|e| reason(&e))?;
    let (kva, voltage, full_load) = (rating.kva, rating.volts, rating.rated_a);
    let refs = vec![profile.reference(Rule::AvailableFault)];
    let steps = vec![
        Step {
            title: format!("Corriente nominal del {what}"),
            formula: Some(if rating.three_phase {
                "In = S / (√3 · V)".into()
            } else {
                "In = S / V".into()
            }),
            substitution: if rating.three_phase {
                format!("In = {kva} kVA / (√3 · {voltage:.0} V) = {full_load:.2} A")
            } else {
                format!("In = {kva} kVA / {voltage:.0} V = {full_load:.2} A")
            },
            note: None,
            references: vec![],
        },
        if from_table {
            Step {
                title: "Impedancia para la falla".into(),
                formula: None,
                substitution: format!("Z = {impedance_percent} %, la mínima de la tabla"),
                note: Some(
                    "Sin dato de placa se toma la impedancia mínima de la norma de fabricación para su tipo, potencia y tensión, que ya incluye la tolerancia: da la falla mayor."
                        .into(),
                ),
                references: vec![profile.reference(Rule::TransformerImpedanceLimits)],
            }
        } else {
            Step {
                title: "Impedancia para la falla".into(),
                formula: Some("Z = Z placa · (1 − tolerancia)".into()),
                substitution: format!(
                    "Z = {impedance_percent} % · (1 − {tolerance:.2}) = {:.3} %",
                    z * 100.0
                ),
                note: Some(
                    "La impedancia real puede salir menor que la de placa; con la reducida la falla sale mayor."
                        .into(),
                ),
                references: refs.clone(),
            }
        },
        Step {
            title: "Falla por bus infinito".into(),
            formula: Some("Icc = In / Z".into()),
            substitution: format!(
                "Icc = {full_load:.2} A / {z:.5} = {icc:.0} A = {:.2} kA",
                icc / 1000.0
            ),
            note: Some(
                "La red aguas arriba se toma sin impedancia: es la corriente mayor posible en el secundario."
                    .into(),
            ),
            references: refs,
        },
    ];
    Ok((icc / 1000.0, steps))
}

/// Inicio del motivo cuando el dato de la suministradora no dice dónde está.
/// La interfaz lo usa para ofrecer los dos lugares.
pub const MISSING_LOCATION: &str = "falta indicar dónde da la suministradora la corriente de falla";

/// El transformador de la acometida para la falla: su corriente nominal, su
/// impedancia reducida y su bus infinito, con los pasos.
struct SubstationFault {
    rating: Rating,
    /// Impedancia con la que se calcula, en por unidad.
    z_pu: f64,
    infinite_ka: f64,
    /// Corriente nominal, impedancia y bus infinito.
    steps: Vec<Step>,
}

/// Bus infinito del transformador de la acometida. Fuera de alcance si no
/// hay transformador; sin evaluar o inválido si le falta o tiene mal un dato.
fn substation_fault(profile: &Profile, project: &Project) -> Result<SubstationFault, FaultIssue> {
    let source = &project.source;
    let Some(substation) = source.substation.as_ref() else {
        return Err(FaultIssue {
            kind: IssueKind::OutOfScope,
            reason: "no hay corriente de falla de la suministradora ni datos del transformador de la acometida".into(),
        });
    };
    let what = |r: String| FaultIssue {
        kind: IssueKind::Incomplete,
        reason: format!("en el transformador de la acometida {r}"),
    };
    let invalid = |r: &str| FaultIssue {
        kind: IssueKind::Invalid,
        reason: format!("en el transformador de la acometida {r}"),
    };
    if substation.kva.is_some_and(|k| !(k.is_finite() && k > 0.0)) {
        return Err(invalid("la potencia debe ser mayor que cero"));
    }
    if !substation.impedance_from_table
        && substation
            .impedance_percent
            .is_some_and(|z| !(z.is_finite() && z > 0.0))
    {
        return Err(invalid("la impedancia debe ser mayor que cero"));
    }
    let kva = substation
        .kva
        .ok_or_else(|| what("falta la potencia".into()))?;
    let impedance = substation_impedance(profile, source).map_err(what)?;
    let (voltage, three_phase) = transformer_voltage(&source.system)
        .ok_or_else(|| what("el sistema de suministro no tiene tensión".into()))?;
    let rated_a =
        transformer_full_load_current(kva, voltage, three_phase).map_err(|e| what(reason(&e)))?;
    let rating = Rating {
        rated_a,
        volts: voltage,
        kva,
        three_phase,
    };
    let (infinite_ka, steps) = infinite_bus_steps(
        profile,
        &rating,
        impedance,
        "transformador de la acometida",
        substation.impedance_from_table,
    )
    .map_err(what)?;
    // El mínimo de la tabla ya incluye la tolerancia (decisión 0029, D-8).
    let tolerance = if substation.impedance_from_table {
        0.0
    } else {
        profile.parameters.fault_impedance_tolerance
    };
    let z_pu = effective_impedance_pu(impedance, tolerance).map_err(|e| what(reason(&e)))?;
    Ok(SubstationFault {
        rating,
        z_pu,
        infinite_ka,
        steps,
    })
}

/// Falla base en el punto de conexión, sin motores (decisión 0029, D-2):
///
/// - Sin dato de la suministradora, el bus infinito del transformador de la
///   acometida.
/// - Con el dato en baja tensión, el dato tal cual; no puede pasar del bus
///   infinito del transformador, si se conoce.
/// - Con el dato en media tensión, la red como impedancia combinada en
///   cuadratura con la del transformador. En un secundario 1F-3H no reduce
///   el bus infinito.
/// - Con el dato sin decir dónde, sin evaluar: nunca se supone.
fn source_base(
    profile: &Profile,
    project: &Project,
) -> Result<(f64, FaultOrigin, Vec<Step>), FaultIssue> {
    use crate::model::FaultLocation;
    let source = &project.source;
    let Some(ka) = source.fault.available_ka else {
        let s = substation_fault(profile, project)?;
        return Ok((s.infinite_ka, FaultOrigin::Substation, s.steps));
    };
    if !(ka.is_finite() && ka > 0.0) {
        return Err(FaultIssue {
            kind: IssueKind::Invalid,
            reason: "la corriente de falla de la suministradora debe ser mayor que cero".into(),
        });
    }
    let datum = |place: String| Step {
        title: "Corriente de falla de la suministradora".into(),
        formula: None,
        substitution: format!("Icc = {ka} kA {place}"),
        note: Some("Dato de la suministradora; manda sobre cualquier estimación.".into()),
        references: vec![profile.reference(Rule::InterruptingRating)],
    };
    match source.fault.location {
        None => Err(FaultIssue {
            kind: IssueKind::Incomplete,
            reason: format!(
                "{MISSING_LOCATION}: en baja tensión, en el punto de conexión, o en media tensión, en el primario del transformador de la acometida"
            ),
        }),
        Some(FaultLocation::LowVoltage) => {
            // Ninguna falla en el secundario pasa del bus infinito de su
            // transformador: un dato mayor es imposible.
            if let Ok(s) = substation_fault(profile, project) {
                let (bound, what) = if center_tapped(&source.system) {
                    (
                        s.infinite_ka * profile.parameters.fault_line_to_neutral_factor,
                        "por bus infinito, de fase a neutro",
                    )
                } else {
                    (s.infinite_ka, "por bus infinito")
                };
                if ka > bound {
                    return Err(FaultIssue {
                        kind: IssueKind::Invalid,
                        reason: format!(
                            "la corriente de falla de la suministradora en baja tensión ({ka} kA) es mayor que la mayor posible con el transformador de la acometida ({bound:.2} kA {what}); revisa el dato o si es de media tensión"
                        ),
                    });
                }
            }
            Ok((
                ka,
                FaultOrigin::Supplier,
                vec![datum("en baja tensión, en el punto de conexión".into())],
            ))
        }
        Some(FaultLocation::Primary) => {
            let s = substation_fault(profile, project).map_err(|issue| match issue.kind {
                IssueKind::OutOfScope => FaultIssue {
                    kind: IssueKind::Incomplete,
                    reason: "con el dato de la suministradora en media tensión falta el transformador de la acometida: potencia, impedancia y tensión del primario".into(),
                },
                _ => issue,
            })?;
            let primary_kv = source.substation.as_ref().and_then(|t| t.primary_kv);
            let kv = match primary_kv {
                Some(kv) if kv.is_finite() && kv > 0.0 => kv,
                Some(_) => {
                    return Err(FaultIssue {
                        kind: IssueKind::Invalid,
                        reason: "en el transformador de la acometida la tensión del primario debe ser mayor que cero".into(),
                    })
                }
                None => {
                    return Err(FaultIssue {
                        kind: IssueKind::Incomplete,
                        reason: "con el dato de la suministradora en media tensión, en el transformador de la acometida falta la tensión del primario".into(),
                    })
                }
            };
            let mut steps = vec![datum(format!(
                "en {kv} kV, en el primario del transformador de la acometida"
            ))];
            if center_tapped(&source.system) {
                steps.extend(s.steps);
                steps.push(Step {
                    title: "Dato de media tensión en un secundario 1F-3H".into(),
                    formula: None,
                    substitution: format!(
                        "Se usa el bus infinito del transformador: {:.2} kA",
                        s.infinite_ka
                    ),
                    note: Some(
                        "Con media bobina no está definida la combinación con la red; el bus infinito es la cota superior y el dato de media tensión no la reduce."
                            .into(),
                    ),
                    references: vec![profile.reference(Rule::LineToNeutralFault)],
                });
                return Ok((s.infinite_ka, FaultOrigin::Substation, steps));
            }
            let z_net = s.rating.kva / (3f64.sqrt() * kv * ka * 1000.0);
            let z = (z_net * z_net + s.z_pu * s.z_pu).sqrt();
            let icc = s.rating.rated_a / z;
            // Corriente nominal e impedancia del transformador; su bus
            // infinito no se usa.
            steps.extend(s.steps.into_iter().take(2));
            let network = vec![profile.reference(Rule::NetworkFaultImpedance)];
            steps.push(Step {
                title: "Impedancia de la red".into(),
                formula: Some("Zred = S / (√3 · V · Icc)".into()),
                substitution: format!(
                    "Zred = {} kVA / (√3 · {kv} kV · {ka} kA) = {z_net:.5}",
                    s.rating.kva
                ),
                note: Some(if s.rating.three_phase {
                    "La red se representa con la impedancia que da esa corriente, en por unidad de la potencia del transformador.".into()
                } else {
                    "La red se representa con la impedancia que da esa corriente, en por unidad de la potencia del transformador. En un transformador monofásico se usa la misma expresión: da la impedancia de red menor y la falla mayor.".into()
                }),
                references: network.clone(),
            });
            steps.push(Step {
                title: "Falla en el secundario".into(),
                formula: Some("Z = √(Zred² + Ztr²); Icc = In / Z".into()),
                substitution: format!(
                    "Z = √({z_net:.5}² + {:.5}²) = {z:.5}; Icc = {:.2} A / {z:.5} = {icc:.0} A = {:.2} kA",
                    s.z_pu,
                    s.rating.rated_a,
                    icc / 1000.0
                ),
                note: Some(
                    "Sin la relación X/R de la red y del transformador se combinan en cuadratura: nunca da una impedancia mayor que la real, así que la falla es una cota superior."
                        .into(),
                ),
                references: network,
            });
            Ok((icc / 1000.0, FaultOrigin::SupplierPrimary, steps))
        }
    }
}

/// Impedancia con la que se calcula la falla del transformador de la
/// acometida: la de placa o, si se pidió, la mínima de la tabla de la norma
/// de fabricación. El error dice qué falta para encontrarla.
pub fn substation_impedance(
    profile: &Profile,
    source: &crate::model::Source,
) -> Result<f64, String> {
    use crate::model::{Connection, Mounting};
    let Some(sub) = &source.substation else {
        return Err("no hay transformador de la acometida".into());
    };
    if !sub.impedance_from_table {
        return match sub.impedance_percent {
            Some(z) => Ok(z),
            None => Err("falta la impedancia de placa".into()),
        };
    }
    let three_phase = transformer_voltage(&source.system).is_some_and(|(_, t)| t)
        && sub.connection != Some(Connection::SinglePhase);
    let kind = match (three_phase, sub.mounting) {
        (false, _) => "single_phase",
        (true, Some(Mounting::Pole)) => "pole_three_phase",
        (true, Some(Mounting::Substation)) => "substation",
        (true, Some(_)) => {
            return Err(
                "la tabla solo cubre transformadores tipo poste y tipo subestación; captura la impedancia de placa".into(),
            )
        }
        (true, None) => return Err("falta el montaje del transformador".into()),
    };
    let kva = sub.kva.ok_or_else(|| "falta la potencia".to_string())?;
    let kv = sub
        .primary_kv
        .ok_or_else(|| "falta la tensión del primario".to_string())?;
    profile
        .impedance_limits(kind, kva, kv)
        .map(|(min, _)| min)
        .ok_or_else(|| {
            format!(
                "la tabla no tiene un transformador de {kva} kVA a {kv} kV de este tipo; captura la impedancia de placa"
            )
        })
}

fn worse(a: Status, b: Status) -> Status {
    match (a, b) {
        (Status::Fails, _) | (_, Status::Fails) => Status::Fails,
        (Status::NotEvaluated, _) | (_, Status::NotEvaluated) => Status::NotEvaluated,
        _ => Status::Complies,
    }
}

/// Zona de tensión: `None` la de la fuente; `Some(i)` la del secundario del
/// transformador del tablero `i`.
type Zone = Option<usize>;

/// Corriente de un motor o compresor del proyecto, o lo que le falta para
/// conocerla. Por tablero y circuito; `None` si el circuito no es de motor.
pub(crate) type MotorData = Vec<Vec<Option<Result<f64, String>>>>;

/// Datos de un transformador intermedio para la falla: sus dos corrientes
/// nominales y su impedancia reducida.
struct TransformerData {
    primary: Rating,
    secondary: Rating,
    /// Impedancia de placa, en porciento.
    impedance_percent: f64,
    /// Reducida por la tolerancia, en por unidad.
    z_pu: f64,
}

struct Topology<'a> {
    project: &'a Project,
    profile: &'a Profile,
    parent: Vec<Option<usize>>,
    zone: Vec<Zone>,
    /// Transformador de cada tablero con transformador.
    transformers: HashMap<usize, Result<TransformerData, String>>,
    motors: &'a MotorData,
    /// «Otras cargas» sin confirmar que parecen motor: tablero, circuito.
    suspects: Vec<(usize, usize)>,
}

impl<'a> Topology<'a> {
    fn new(profile: &'a Profile, project: &'a Project, motors: &'a MotorData) -> Self {
        let n = project.panels.len();
        let parent: Vec<Option<usize>> = (0..n)
            .map(|i| {
                let id = project.panels[i].parent.as_deref()?;
                project.panels.iter().position(|p| p.id == id)
            })
            .collect();
        let zone = (0..n)
            .map(|i| {
                let mut at = i;
                // El proyecto ya se validó sin ciclos; el límite es por si acaso.
                for _ in 0..=n {
                    if project.panels[at].transformer.is_some() {
                        return Some(at);
                    }
                    match parent[at] {
                        Some(p) => at = p,
                        None => return None,
                    }
                }
                None
            })
            .collect();
        let tolerance = profile.parameters.fault_impedance_tolerance;
        let transformers = (0..n)
            .filter_map(|i| {
                let panel = &project.panels[i];
                let t = panel.transformer.as_ref()?;
                let data = (|| {
                    let primary = crate::transformer::rating(
                        t.kva,
                        panel.kind,
                        &crate::connection::feeder_system(project, i),
                    )
                    .map_err(|e| reason(&e))?;
                    let secondary = crate::transformer::rating(
                        t.kva,
                        panel.kind,
                        &crate::connection::bus_system(project, i),
                    )
                    .map_err(|e| reason(&e))?;
                    let impedance_percent = t
                        .impedance_percent
                        .ok_or_else(|| "falta la impedancia".to_string())?;
                    let z_pu = effective_impedance_pu(impedance_percent, tolerance)
                        .map_err(|e| reason(&e))?;
                    Ok(TransformerData {
                        primary,
                        secondary,
                        impedance_percent,
                        z_pu,
                    })
                })();
                Some((i, data))
            })
            .collect();
        let suspects = crate::review::kind_hints(project)
            .into_iter()
            .map(|h| (h.panel, h.circuit))
            .collect();
        Topology {
            project,
            profile,
            parent,
            zone,
            transformers,
            motors,
            suspects,
        }
    }

    /// Cargas sin confirmar que parecen motor y aportarían a una zona: las
    /// suyas y las de las zonas que cuelgan de ella por un transformador.
    fn suspects_at(&self, zone: Zone, depth: usize) -> Vec<String> {
        let mut out: Vec<String> = self
            .suspects
            .iter()
            .filter(|(i, _)| self.zone[*i] == zone)
            .map(|&(i, c)| self.label(i, c))
            .collect();
        if depth <= self.project.panels.len() {
            for t in self.children(zone) {
                out.extend(self.suspects_at(Some(t), depth + 1));
            }
        }
        out
    }

    /// «Otras cargas» de una zona que el usuario confirmó que no son motor
    /// aunque su nombre lo parezca, para la trazabilidad.
    fn confirmed_at(&self, zone: Zone) -> Vec<String> {
        let mut out = Vec::new();
        for (i, panel) in self.project.panels.iter().enumerate() {
            if self.zone[i] != zone {
                continue;
            }
            for (c, circuit) in panel.circuits.iter().enumerate() {
                if circuit.load.kind == crate::model::LoadKind::General
                    && circuit.load.confirmed_general
                    && crate::review::kind_from_name(&circuit.name).is_some()
                {
                    out.push(self.label(i, c));
                }
            }
        }
        out
    }

    /// Zona de la que sale el alimentador de un tablero.
    fn upstream_zone(&self, panel: usize) -> Zone {
        self.parent[panel].and_then(|p| self.zone[p])
    }

    fn label(&self, panel: usize, circuit: usize) -> String {
        let p = &self.project.panels[panel];
        let c = &p.circuits[circuit];
        format!("{} · {} {}", p.id, c.id, c.name)
    }

    /// Transformadores cuyo primario está en la zona `zone`.
    fn children(&self, zone: Zone) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .transformers
            .keys()
            .copied()
            .filter(|&t| self.upstream_zone(t) == zone)
            .collect();
        out.sort_unstable();
        out
    }

    /// Aportación de los motores a las barras de una zona, en amperes a su
    /// tensión, y los motores sin datos que la dejan incompleta.
    fn motors_at(&self, zone: Zone, depth: usize) -> (Option<MotorContribution>, Vec<String>) {
        let multiplier = self.profile.parameters.fault_motor_multiplier;
        let mut direct = Vec::new();
        let mut incomplete = Vec::new();
        for (i, row) in self.motors.iter().enumerate() {
            if self.zone[i] != zone {
                continue;
            }
            for (k, datum) in row.iter().enumerate() {
                match datum {
                    Some(Ok(a)) => direct.push(MotorShare {
                        label: self.label(i, k),
                        full_load_a: *a,
                    }),
                    Some(Err(_)) => incomplete.push(self.label(i, k)),
                    None => {}
                }
            }
        }
        let currents: Vec<f64> = direct.iter().map(|m| m.full_load_a).collect();
        let mut total_a = motor_contribution(&currents, multiplier);
        let mut through = Vec::new();
        // Una zona cuelga de otra a través de un transformador; el límite
        // evita recorrer un proyecto con ciclos.
        if depth <= self.project.panels.len() {
            for t in self.children(zone) {
                let (down, down_incomplete) = self.motors_at(Some(t), depth + 1);
                incomplete.extend(down_incomplete);
                let Some(down) = down else { continue };
                match &self.transformers[&t] {
                    Ok(data) => {
                        let contribution_a = transfer(data, down.total_a);
                        total_a += contribution_a;
                        through.push(TransferShare {
                            panel: self.project.panels[t].id.clone(),
                            downstream_a: down.total_a,
                            contribution_a,
                        });
                    }
                    Err(why) => incomplete.push(format!(
                        "los motores detrás del transformador de {} ({why})",
                        self.project.panels[t].id
                    )),
                }
            }
        }
        let contribution =
            (!direct.is_empty() || !through.is_empty()).then_some(MotorContribution {
                multiplier,
                motors: direct,
                through,
                total_a,
            });
        (contribution, incomplete)
    }

    /// Falla base de una zona, sin motores.
    fn base(&self, zone: Zone) -> Result<(f64, FaultOrigin, Vec<Step>), FaultIssue> {
        let Some(t) = zone else {
            return source_base(self.profile, self.project);
        };
        let id = &self.project.panels[t].id;
        let incomplete = |why: String| FaultIssue {
            kind: IssueKind::Incomplete,
            reason: format!("en el transformador de {id} {why}"),
        };
        let invalid = |why: String| FaultIssue {
            kind: IssueKind::Invalid,
            reason: format!("en el transformador de {id} {why}"),
        };
        let panel = &self.project.panels[t];
        let transformer = panel
            .transformer
            .as_ref()
            .expect("zona de un transformador");
        let positive = |v: f64| v.is_finite() && v > 0.0;
        if transformer.kva.is_some_and(|k| !positive(k)) {
            return Err(invalid("la potencia debe ser mayor que cero".into()));
        }
        if transformer.impedance_percent.is_some_and(|z| !positive(z)) {
            return Err(invalid("la impedancia debe ser mayor que cero".into()));
        }
        // Una conexión que el secundario no tiene es inconsistente, no un
        // dato que falta.
        let secondary_system = crate::connection::bus_system(self.project, t);
        if transformer.kva.is_some() && panel.kind.reference_voltage(&secondary_system).is_err() {
            return Err(invalid(format!(
                "el secundario no admite la conexión del tablero ({})",
                crate::connection::kind_phrase(panel.kind)
            )));
        }
        // El secundario no depende del primario: basta su corriente nominal.
        let secondary = crate::transformer::rating(
            transformer.kva,
            panel.kind,
            &crate::connection::bus_system(self.project, t),
        )
        .map_err(|e| incomplete(reason(&e)))?;
        let impedance = transformer
            .impedance_percent
            .ok_or_else(|| incomplete("falta la impedancia".into()))?;
        let (ka, steps) = infinite_bus_steps(
            self.profile,
            &secondary,
            impedance,
            &format!("transformador de {id}"),
            false,
        )
        .map_err(incomplete)?;
        Ok((ka, FaultOrigin::Transformer { panel: id.clone() }, steps))
    }

    /// Sistema de las barras de una zona.
    fn system(&self, zone: Zone) -> System {
        match zone {
            None => self.project.source.system,
            Some(t) => crate::connection::bus_system(self.project, t),
        }
    }

    fn bus(&self, zone: Zone) -> Result<BusFault, FaultIssue> {
        let (mut base_ka, origin, mut steps) = self.base(zone)?;
        // Por bus infinito en un secundario 1F-3H manda la falla de fase a
        // neutro. El dato de la suministradora ya es la máxima disponible.
        let line_to_neutral =
            (origin.infinite_bus() && center_tapped(&self.system(zone))).then(|| {
                let factor = self.profile.parameters.fault_line_to_neutral_factor;
                steps.push(line_to_neutral_step(self.profile, base_ka, factor));
                let line_to_line_ka = base_ka;
                base_ka *= factor;
                LineToNeutral {
                    factor,
                    line_to_line_ka,
                }
            });
        let (motors, incomplete) = self.motors_at(zone, 0);
        let motor_ka = motors.as_ref().map_or(0.0, |m| m.total_a / 1000.0);
        if let Some(m) = &motors {
            steps.extend(self.motor_steps(m, base_ka, motor_ka));
        }
        Ok(BusFault {
            base_ka,
            motor_ka,
            total_ka: base_ka + motor_ka,
            origin,
            steps,
            motors,
            incomplete,
            suspected: self.suspects_at(zone, 0),
            line_to_neutral,
        })
    }

    /// Trazabilidad de la falla de una zona vista desde `place`.
    fn trace(&self, zone: Zone, result: &Result<BusFault, FaultIssue>, place: &str) -> Trace {
        let profile = self.profile;
        let source = &self.project.source;
        let shown = |v: Option<f64>, unit: &str| {
            v.map_or("sin dato".to_string(), |x| format!("{x} {unit}"))
        };
        let system = |s: &System| match (s.line_to_line_v, s.line_to_neutral_v) {
            (Some(ll), Some(ln)) => format!("{ll}/{ln} V"),
            (Some(ll), None) => format!("{ll} V"),
            (None, Some(ln)) => format!("{ln} V"),
            (None, None) => "sin tensión".to_string(),
        };
        let mut inputs = Vec::new();
        let mut assumptions = Vec::new();
        let mut contributors = Vec::new();
        match zone {
            None => {
                inputs.push(Datum::new("Sistema de suministro", system(&source.system)));
                if let Some(ka) = source.fault.available_ka {
                    use crate::model::FaultLocation;
                    let place = match source.fault.location {
                        Some(FaultLocation::LowVoltage) => {
                            "en baja tensión, en el punto de conexión".to_string()
                        }
                        Some(FaultLocation::Primary) => format!(
                            "en media tensión{}, en el primario del transformador",
                            source
                                .substation
                                .as_ref()
                                .and_then(|s| s.primary_kv)
                                .map_or(String::new(), |kv| format!(" ({kv} kV)"))
                        ),
                        None => "sin decir dónde".to_string(),
                    };
                    inputs.push(Datum::new(
                        "Falla de la suministradora",
                        format!("{ka} kA {place}"),
                    ));
                }
                if let Some(sub) = &source.substation {
                    inputs.push(Datum::new(
                        "Transformador de la acometida",
                        shown(sub.kva, "kVA"),
                    ));
                    inputs.push(Datum::new(
                        "Impedancia",
                        if sub.impedance_from_table {
                            "mínima de la tabla de la norma de fabricación".to_string()
                        } else {
                            shown(sub.impedance_percent, "%")
                        },
                    ));
                    inputs.push(Datum::new("Primario", shown(sub.primary_kv, "kV")));
                }
            }
            Some(t) => {
                let panel = &self.project.panels[t];
                let tr = panel
                    .transformer
                    .as_ref()
                    .expect("zona de un transformador");
                inputs.push(Datum::new(
                    format!("Transformador intermedio de {}", panel.id),
                    shown(tr.kva, "kVA"),
                ));
                inputs.push(Datum::new("Impedancia", shown(tr.impedance_percent, "%")));
                inputs.push(Datum::new(
                    "Primario",
                    system(&crate::connection::feeder_system(self.project, t)),
                ));
                inputs.push(Datum::new("Secundario", system(&tr.secondary)));
                inputs.push(Datum::new(
                    "Conexión del tablero",
                    crate::connection::kind_phrase(panel.kind),
                ));
            }
        }
        let mut missing = Vec::new();
        let mut problems = Vec::new();
        let evaluation = match result {
            Ok(bus) => {
                let base = match &bus.line_to_neutral {
                    Some(ln) => format!(
                        "{:.2} kA de fase a fase, {:.2} kA de fase a neutro",
                        ln.line_to_line_ka, bus.base_ka
                    ),
                    None => format!("{:.2} kA", bus.base_ka),
                };
                match &bus.origin {
                    FaultOrigin::SupplierPrimary => {
                        assumptions.push("Dato de la suministradora en media tensión: la red se representa con la impedancia que da esa corriente y se combina en cuadratura con la del transformador, sin la relación X/R (decisión 0029).".into());
                        contributors.push(format!("Red y transformador de la acometida: {base}"));
                    }
                    FaultOrigin::Supplier => {
                        assumptions.push(
                            "Dato de la suministradora en baja tensión, en el punto de conexión."
                                .into(),
                        );
                        if center_tapped(&self.system(zone)) {
                            assumptions.push("Servicio 1F-3H: el dato de la suministradora se toma como la corriente máxima disponible en el punto de conexión y no se multiplica por el factor de fase a neutro.".into());
                        }
                        contributors.push(format!("Suministradora: {base}"));
                    }
                    FaultOrigin::Substation => {
                        assumptions.push(
                            "Bus infinito: la red de media tensión se toma sin impedancia.".into(),
                        );
                        contributors.push(format!("Transformador de la acometida: {base}"));
                    }
                    FaultOrigin::Transformer { panel } => {
                        assumptions.push(format!("Bus infinito: la red aguas arriba del transformador de {panel} se toma sin impedancia; ya incluye la aportación de los motores de arriba."));
                        contributors.push(format!("Transformador de {panel}: {base}"));
                    }
                }
                if let Some(ln) = &bus.line_to_neutral {
                    assumptions.push(format!(
                        "Secundario monofásico con derivación central (1F-3H): la falla de fase a neutro se toma como {} veces la de fase a fase, el máximo teórico del método punto a punto de Eaton Bussmann (decisión 0029). Es la de las terminales y se usa en toda la zona.",
                        ln.factor
                    ));
                }
                if bus.origin.upper_bound() {
                    let from_table = zone.is_none()
                        && source
                            .substation
                            .as_ref()
                            .is_some_and(|s| s.impedance_from_table);
                    assumptions.push(if from_table {
                        "Impedancia mínima de la tabla de la norma de fabricación, que ya incluye la tolerancia.".into()
                    } else {
                        format!(
                            "Impedancia de placa reducida {:.0} % por la tolerancia de fabricación.",
                            profile.parameters.fault_impedance_tolerance * 100.0
                        )
                    });
                }
                assumptions.push(
                    "Sin descontar la impedancia de los alimentadores: es una cota superior."
                        .into(),
                );
                if let Some(m) = &bus.motors {
                    assumptions.push(format!(
                        "Motores de esta tensión: {} veces su corriente a plena carga; los de otra tensión, a través de la impedancia de su transformador (decisión 0028).",
                        m.multiplier
                    ));
                    contributors.extend(
                        m.motors
                            .iter()
                            .map(|s| format!("{}: {:.2} A a plena carga", s.label, s.full_load_a)),
                    );
                    contributors.extend(m.through.iter().map(|t| {
                        format!(
                            "Motores detrás del transformador de {}: {:.1} A en estas barras",
                            t.panel, t.contribution_a
                        )
                    }));
                }
                missing.extend(
                    bus.incomplete
                        .iter()
                        .map(|m| format!("{m}: sin datos, su aportación no está incluida")),
                );
                missing.extend(bus.suspected.iter().map(|m| {
                    format!("{m}: parece motor o aire acondicionado y está como «Otra carga»; confirma que es otra carga o cámbiala de tipo")
                }));
                contributors.extend(
                    self.confirmed_at(zone)
                        .into_iter()
                        .map(|m| format!("{m}: confirmada como otra carga, no aporta")),
                );
                bus.evaluation()
            }
            Err(issue) => {
                match issue.kind {
                    IssueKind::Invalid => problems.push(capitalize(&issue.reason)),
                    _ => missing.push(capitalize(&issue.reason)),
                }
                issue.kind.evaluation()
            }
        };
        Trace {
            evaluation,
            what: "Corriente de falla estimada".into(),
            place: place.to_string(),
            inputs,
            assumptions,
            contributors,
            missing,
            problems,
        }
    }

    fn motor_steps(&self, m: &MotorContribution, base_ka: f64, motor_ka: f64) -> Vec<Step> {
        let profile = self.profile;
        let mut steps = Vec::new();
        if !m.motors.is_empty() {
            let currents: Vec<String> = m
                .motors
                .iter()
                .map(|s| format!("{:.2}", s.full_load_a))
                .collect();
            let direct_a = motor_contribution(
                &m.motors.iter().map(|s| s.full_load_a).collect::<Vec<_>>(),
                m.multiplier,
            );
            steps.push(Step {
                title: "Aportación de los motores de esta tensión".into(),
                formula: Some("Im = k · Σ Ipc".into()),
                substitution: format!(
                    "Im = {} · ({}) A = {direct_a:.1} A",
                    m.multiplier,
                    currents.join(" + "),
                ),
                note: Some(
                    "Los motores en marcha alimentan la falla como fuentes detrás de su impedancia subtransitoria. Se suman todos los de la misma zona de tensión, aunque un motor lejano aporte menos."
                        .into(),
                ),
                references: vec![profile.reference(Rule::MotorFaultContribution)],
            });
        }
        for t in &m.through {
            let index = self
                .project
                .panels
                .iter()
                .position(|p| p.id == t.panel)
                .expect("transformador del proyecto");
            let Some(Ok(data)) = self.transformers.get(&index) else {
                continue;
            };
            steps.push(Step {
                title: format!("Aportación a través del transformador de {}", t.panel),
                formula: Some("Im = Ib₁ / (Zt + Ib₂ / Im₂)".into()),
                substitution: format!(
                    "Im = {:.2} A / ({:.5} + {:.2} A / {:.1} A) = {:.1} A",
                    data.primary.rated_a,
                    data.z_pu,
                    data.secondary.rated_a,
                    t.downstream_a,
                    t.contribution_a
                ),
                note: Some(format!(
                    "Los motores del secundario ({:.1} A a {:.0} V) llegan a estas barras a través de la impedancia del transformador ({} % reducida por la tolerancia), referidos a {:.0} V. Ib₁ e Ib₂ son sus corrientes nominales de cada lado.",
                    t.downstream_a, data.secondary.volts, data.impedance_percent, data.primary.volts
                )),
                references: vec![
                    profile.reference(Rule::MotorFaultContribution),
                    profile.reference(Rule::AvailableFault),
                ],
            });
        }
        steps.push(Step {
            title: "Corriente de falla con motores".into(),
            formula: Some("Icc total = Icc + Im".into()),
            substitution: format!(
                "Icc total = {base_ka:.2} + {motor_ka:.3} = {:.2} kA",
                base_ka + motor_ka
            ),
            note: None,
            references: vec![],
        });
        steps
    }
}

/// Aportación de los motores de un secundario que llega al primario, en
/// amperes a la tensión del primario: los motores son una fuente detrás de
/// la impedancia `Ib₂ / Im₂` (en por unidad de la base del transformador) y
/// la del transformador queda en serie. Sin la impedancia del transformador
/// sería `Im₂ · V₂ / V₁`; con una aportación infinita, su bus infinito.
fn transfer(data: &TransformerData, downstream_a: f64) -> f64 {
    if downstream_a <= 0.0 {
        return 0.0;
    }
    data.primary.rated_a / (data.z_pu + data.secondary.rated_a / downstream_a)
}

/// Compara el interruptor de un tramo con la falla de las barras de donde
/// sale. `None` si el tramo no se calculó o si la falla de ese lugar está
/// fuera de alcance.
fn check_breaker(
    profile: &Profile,
    item: &mut RunResult,
    breaker_ka: Option<f64>,
    bus: &Result<BusFault, FaultIssue>,
    bus_name: &str,
) -> Option<(Option<f64>, Status)> {
    item.run.as_ref()?;
    let bus = match bus {
        Ok(bus) => bus,
        Err(issue) if issue.kind == IssueKind::OutOfScope => return None,
        Err(issue) => {
            item.interrupting = Some(Check {
                name: INTERRUPTING.into(),
                status: Status::NotEvaluated,
                detail: format!(
                    "{}: no se pudo estimar la falla en {bus_name} porque {}",
                    if issue.kind == IssueKind::Invalid {
                        "dato inválido"
                    } else {
                        "sin evaluar"
                    },
                    issue.reason
                ),
                references: vec![profile.reference(Rule::InterruptingRating)],
            });
            return Some((breaker_ka, Status::NotEvaluated));
        }
    };
    let icc = bus.total_ka;
    let (status, detail) = match breaker_ka {
        Some(ka) if ka >= icc && bus.open_items().is_some() => (
            Status::NotEvaluated,
            format!(
                "no concluyente: {ka} kA ≥ {icc:.2} kA en {bus_name}, pero {}",
                bus.open_items().unwrap_or_default()
            ),
        ),
        Some(ka) if ka >= icc => (
            Status::Complies,
            format!(
                "{ka} kA ≥ {icc:.2} kA de falla en {bus_name}, estimada {}; cumple con un método conservador",
                bus.basis()
            ),
        ),
        Some(ka) => (
            Status::NotEvaluated,
            if bus.origin.upper_bound() {
                format!(
                    "no concluyente: {ka} kA es menor que los {icc:.2} kA estimados en {bus_name} {}, que es una cota superior; requiere estudio de cortocircuito",
                    bus.basis()
                )
            } else {
                format!(
                    "no concluyente: {ka} kA es menor que los {icc:.2} kA estimados en {bus_name} con el dato de la suministradora, sin descontar los alimentadores; requiere estudio de cortocircuito"
                )
            },
        ),
        None => (
            Status::NotEvaluated,
            format!(
                "falta la capacidad interruptiva del interruptor o la del tablero; la falla estimada en {bus_name} es de {icc:.2} kA"
            ),
        ),
    };
    let references = group_rules(bus)
        .iter()
        .map(|r| profile.reference(*r))
        .collect();
    item.interrupting = Some(Check {
        name: INTERRUPTING.into(),
        status,
        detail,
        references,
    });
    Some((breaker_ka, status))
}

fn group_rules(bus: &BusFault) -> Vec<Rule> {
    let mut rules = vec![Rule::InterruptingRating];
    if bus.origin.infinite_bus() {
        rules.push(Rule::AvailableFault);
    }
    if bus.origin == FaultOrigin::SupplierPrimary {
        rules.push(Rule::NetworkFaultImpedance);
    }
    if bus.line_to_neutral.is_some() {
        rules.push(Rule::LineToNeutralFault);
    }
    if bus.motor_ka > 0.0 {
        rules.push(Rule::MotorFaultContribution);
    }
    rules
}

/// Hace el estudio y agrega la verificación de capacidad interruptiva a cada
/// tramo. `None` si nada se puede evaluar porque la fuente no tiene ningún
/// dato y no hay transformadores intermedios: la capacidad interruptiva queda
/// fuera de alcance, como en la decisión 0018.
pub(crate) fn study(
    profile: &Profile,
    project: &Project,
    panels: &mut [PanelResult],
    motors: &MotorData,
) -> Option<FaultStudy> {
    let topology = Topology::new(profile, project, motors);
    let n = project.panels.len();

    // Falla de cada zona: la de la fuente y la de cada transformador.
    let mut zones: HashMap<Zone, Result<BusFault, FaultIssue>> = HashMap::new();
    zones.insert(None, topology.bus(None));
    for &t in topology.transformers.keys() {
        zones.insert(Some(t), topology.bus(Some(t)));
    }
    let source = &zones[&None];

    // Interruptores por lugar: índice 0 el punto de conexión, después las
    // barras de cada tablero.
    let mut found: Vec<Vec<BreakerVerdict>> = vec![Vec::new(); n + 1];
    let mut outside: Vec<bool> = vec![false; n + 1];
    let mut record = |place: usize,
                      verdict: Option<(Option<f64>, Status)>,
                      item: &RunResult,
                      panel: &str,
                      circuit: Option<&str>,
                      label: String| {
        match verdict {
            Some((ka, status)) => found[place].push(BreakerVerdict {
                panel: panel.to_string(),
                circuit: circuit.map(str::to_string),
                label,
                ka,
                status,
            }),
            None if item.run.is_some() => outside[place] = true,
            None => {}
        }
    };
    for (i, panel) in project.panels.iter().enumerate() {
        let own = &zones[&topology.zone[i]];
        let parent = topology.parent[i];
        let upstream = &zones[&topology.upstream_zone(i)];
        let (upstream_name, upstream_board_ka) = match parent {
            Some(k) => (
                format!("las barras de {}", project.panels[k].id),
                project.panels[k].board.interrupting_ka,
            ),
            None => ("el punto de conexión".to_string(), None),
        };
        let Some(result) = panels.iter_mut().find(|r| r.id == panel.id) else {
            continue;
        };
        // El alimentador sale de las barras de arriba.
        let ka = panel
            .feeder
            .overrides
            .breaker
            .interrupting_ka
            .or(upstream_board_ka);
        let verdict = check_breaker(profile, &mut result.feeder, ka, upstream, &upstream_name);
        record(
            parent.map_or(0, |k| k + 1),
            verdict,
            &result.feeder,
            &panel.id,
            None,
            format!("Alimentador de {}", panel.id),
        );
        let own_name = format!("las barras de {}", panel.id);
        let board_ka = panel.board.interrupting_ka;
        if let (Some(t), Some(secondary)) = (&panel.transformer, result.secondary.as_mut()) {
            if t.secondary_protection {
                let ka = t.secondary_overrides.breaker.interrupting_ka.or(board_ka);
                let verdict = check_breaker(profile, secondary, ka, own, &own_name);
                record(
                    i + 1,
                    verdict,
                    secondary,
                    &panel.id,
                    None,
                    format!("Secundario de {}", panel.id),
                );
            }
        }
        for (circuit, item) in panel.circuits.iter().zip(result.circuits.iter_mut()) {
            let ka = circuit.overrides.breaker.interrupting_ka.or(board_ka);
            let verdict = check_breaker(profile, item, ka, own, &own_name);
            record(
                i + 1,
                verdict,
                item,
                &panel.id,
                Some(&circuit.id),
                format!("{} · {}", circuit.id, circuit.name),
            );
        }
        match own {
            Ok(bus) => result.fault = Some(bus.clone()),
            Err(issue) => result.fault_issue = Some(issue.clone()),
        }
        let own_out_of_scope = matches!(own, Err(i) if i.kind == IssueKind::OutOfScope);
        if !own_out_of_scope {
            result.fault_trace = Some(topology.trace(topology.zone[i], own, &own_name));
        }
    }

    let mut groups = Vec::new();
    let mut out_of_scope = Vec::new();
    for (k, breakers) in found.into_iter().enumerate() {
        let (panel, place, zone, board_ka) = match k.checked_sub(1) {
            None => (None, "el punto de conexión".to_string(), None, None),
            Some(i) => (
                Some(project.panels[i].id.clone()),
                format!("las barras de {}", project.panels[i].id),
                topology.zone[i],
                project.panels[i].board.interrupting_ka,
            ),
        };
        if outside[k] {
            out_of_scope.push(place.clone());
        }
        if breakers.is_empty() {
            continue;
        }
        let status = breakers
            .iter()
            .fold(Status::Complies, |all, b| worse(all, b.status));
        let (fault_ka, infinite_bus, line_to_neutral, origin, issue, incomplete_motors, references) =
            match &zones[&zone] {
                Ok(bus) => (
                    Some(bus.total_ka),
                    bus.origin.infinite_bus(),
                    bus.line_to_neutral.is_some(),
                    Some(bus.origin.clone()),
                    None,
                    bus.incomplete.len(),
                    group_rules(bus),
                ),
                Err(issue) => (
                    None,
                    false,
                    false,
                    None,
                    Some(issue.reason.clone()),
                    0,
                    vec![Rule::InterruptingRating],
                ),
            };
        let suspected_loads = zones[&zone].as_ref().map_or(0, |b| b.suspected.len());
        let mut trace = topology.trace(zone, &zones[&zone], &place);
        trace.what = "Capacidad interruptiva frente a la falla estimada".into();
        let missing_ka = breakers.iter().filter(|b| b.ka.is_none()).count();
        if missing_ka > 0 {
            trace.missing.push(if missing_ka == 1 {
                "Capacidad interruptiva de 1 interruptor o la del tablero".to_string()
            } else {
                format!("Capacidad interruptiva de {missing_ka} interruptores o la del tablero")
            });
        }
        let comparison = match fault_ka {
            Some(fault) => {
                let short = breakers
                    .iter()
                    .filter(|b| b.ka.is_some_and(|ka| ka < fault))
                    .count();
                if short > 0 {
                    trace.problems.push(if short == 1 {
                        "1 interruptor no alcanza la cota superior: requiere estudio de cortocircuito".to_string()
                    } else {
                        format!("{short} interruptores no alcanzan la cota superior: requiere estudio de cortocircuito")
                    });
                }
                if missing_ka > 0 {
                    Evaluation::NotEvaluated
                } else if short > 0 {
                    Evaluation::Inconclusive
                } else {
                    Evaluation::Valid
                }
            }
            None => Evaluation::NotEvaluated,
        };
        let evaluation = trace.evaluation.worst(comparison);
        trace.evaluation = evaluation;
        groups.push(InterruptingGroup {
            panel,
            place,
            fault_ka,
            infinite_bus,
            line_to_neutral,
            origin,
            issue,
            incomplete_motors,
            suspected_loads,
            board_ka,
            status,
            evaluation,
            trace,
            breakers,
            references: references.iter().map(|r| profile.reference(*r)).collect(),
        });
    }

    let (source_bus, source_issue) = match source {
        Ok(bus) => (Some(bus.clone()), None),
        Err(issue) => (None, Some(issue.clone())),
    };
    // Sin ningún dato de la fuente y sin nada evaluado, el estudio no existe:
    // la capacidad interruptiva queda fuera de alcance como antes.
    if groups.is_empty()
        && source_issue
            .as_ref()
            .is_some_and(|i| i.kind == IssueKind::OutOfScope)
    {
        return None;
    }
    let mut incomplete_motors = Vec::new();
    for (i, row) in motors.iter().enumerate() {
        for (k, datum) in row.iter().enumerate() {
            if let Some(Err(missing)) = datum {
                incomplete_motors.push(IncompleteMotor {
                    panel: project.panels[i].id.clone(),
                    circuit: project.panels[i].circuits[k].id.clone(),
                    label: topology.label(i, k),
                    missing: missing.clone(),
                });
            }
        }
    }
    let source_trace = topology.trace(None, source, "el punto de conexión");
    Some(FaultStudy {
        source: source_bus,
        source_issue,
        source_trace,
        impedance_tolerance: profile.parameters.fault_impedance_tolerance,
        multiplier: profile.parameters.fault_motor_multiplier,
        line_to_neutral_factor: profile.parameters.fault_line_to_neutral_factor,
        incomplete_motors,
        out_of_scope,
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(kva: f64, v1: f64, v2: f64, z_pu: f64) -> TransformerData {
        let rating = |v: f64| Rating {
            rated_a: kva * 1000.0 / (3f64.sqrt() * v),
            volts: v,
            kva,
            three_phase: true,
        };
        TransformerData {
            primary: rating(v1),
            secondary: rating(v2),
            impedance_percent: z_pu * 100.0,
            z_pu,
        }
    }

    /// Sin impedancia del transformador, la aportación se refiere solo con
    /// la relación de tensiones (cuaderno técnico 158).
    #[test]
    fn transfer_without_impedance_is_the_voltage_ratio() {
        let d = data(45.0, 480.0, 220.0, 1e-12);
        let up = transfer(&d, 100.0);
        assert!((up - 100.0 * 220.0 / 480.0).abs() < 1e-6, "{up}");
    }

    /// Con aportación muy grande, lo que pasa es el bus infinito del
    /// transformador visto desde el primario.
    #[test]
    fn transfer_is_limited_by_the_transformer() {
        let d = data(45.0, 480.0, 220.0, 0.027);
        let up = transfer(&d, 1e12);
        assert!((up - d.primary.rated_a / 0.027).abs() < 1e-3, "{up}");
    }

    /// 45 kVA, 480/220 V, Z = 3 % reducida 10 %: Ib₁ = 54.13 A,
    /// Ib₂ = 118.10 A. Con 4 × 30 A = 120 A de motores en el secundario:
    /// Im = 54.13 / (0.027 + 118.10 / 120) = 53.53 A en 480 V, menos que los
    /// 55 A de referirlos solo con la relación.
    #[test]
    fn transfer_example() {
        let d = data(45.0, 480.0, 220.0, 0.027);
        let up = transfer(&d, 120.0);
        assert!((up - 53.534).abs() < 0.01, "{up}");
        assert!(up < 120.0 * 220.0 / 480.0);
        assert_eq!(transfer(&d, 0.0), 0.0);
    }
}
