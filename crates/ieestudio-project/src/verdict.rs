//! Veredicto en tres niveles, que nunca se mezclan:
//!
//! 1. **Cálculo matemático**: si cada tramo se pudo calcular con los datos y
//!    las tablas disponibles.
//! 2. **Verificación normativa**: un encabezado, en este orden de prioridad
//!    (decisión 0029, D-6): «No conforme en lo evaluado», «Datos inválidos:
//!    requiere corrección», «Evaluación incompleta» y «Conforme en lo
//!    evaluado», con el conteo de lo demás debajo.
//! 3. **Aprobación profesional**: siempre pendiente. La otorga quien firma el
//!    proyecto; el programa nunca la emite.
//!
//! La conformidad es siempre **en lo evaluado**: hay verificaciones de la
//! norma que IEEStudio no hace y que se declaran fuera de su alcance, con su
//! referencia, para que nadie las dé por revisadas (decisión 0018).

use ieestudio_rules::{cite_all, Profile, Rule, RuleReference};
use serde::Serialize;

use crate::Status;

/// Verificación de la norma que IEEStudio no hace y que debe resolverse con
/// otro estudio.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutOfScope {
    /// Qué no se verifica.
    pub name: String,
    /// Con qué se resuelve.
    pub resolve_with: String,
    pub references: Vec<RuleReference>,
}

impl OutOfScope {
    /// `capacidad interruptiva (NOM-001-SEDE-2012: 110-9, 110-10)`.
    pub fn label(&self) -> String {
        match cite_all(&self.references) {
            Some(cite) => format!("{} ({cite})", self.name),
            None => self.name.clone(),
        }
    }
}

/// Verificaciones fuera del alcance de IEEStudio.
///
/// La capacidad interruptiva de los interruptores exige conocer la corriente
/// de cortocircuito en cada punto; ese estudio se hará en otro programa de la
/// suite (decisión 0018).
pub fn out_of_scope(profile: &Profile) -> Vec<OutOfScope> {
    vec![OutOfScope {
        name: "Capacidad interruptiva".into(),
        resolve_with: "requiere estudio de cortocircuito".into(),
        references: vec![profile.reference(Rule::InterruptingRating)],
    }]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Calculation {
    /// Cada tramo se calculó.
    Complete,
    /// Algún tramo no se pudo calcular, o no hay nada que calcular.
    Incomplete,
}

/// Resultado de una verificación para el dictamen: el estado de tres
/// valores más el dato inválido (decisión 0029, D-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Complies,
    Fails,
    /// Falta un dato o el resultado no es concluyente.
    Incomplete,
    /// Un dato inconsistente impide evaluar.
    Invalid,
}

impl From<Status> for Outcome {
    fn from(status: Status) -> Self {
        match status {
            Status::Complies => Outcome::Complies,
            Status::Fails => Outcome::Fails,
            Status::NotEvaluated => Outcome::Incomplete,
        }
    }
}

/// Encabezado del dictamen, en orden de prioridad (decisión 0029, D-6). La
/// conformidad siempre es en lo evaluado; «pendiente» queda solo para la
/// aprobación profesional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Compliance {
    /// Todo lo que el programa verifica cumple y nada quedó sin evaluar. Las
    /// verificaciones fuera de su alcance no cuentan.
    Conforms,
    /// Al menos una verificación no cumple.
    DoesNotConform,
    /// Nada incumple, pero hay datos inválidos que impiden evaluar.
    InvalidData,
    /// Nada incumple ni es inválido, pero faltan datos o hay resultados no
    /// concluyentes.
    Incomplete,
}

impl Compliance {
    /// El encabezado tal como lo muestran todas las salidas.
    pub fn heading(self) -> &'static str {
        match self {
            Compliance::DoesNotConform => "No conforme en lo evaluado",
            Compliance::InvalidData => "Datos inválidos: requiere corrección",
            Compliance::Incomplete => "Evaluación incompleta",
            Compliance::Conforms => "Conforme en lo evaluado",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Verdict {
    pub calculation: Calculation,
    pub compliance: Compliance,
    /// Siempre `false`: el programa no aprueba proyectos.
    pub professionally_approved: bool,
    /// Tramos sin calcular, incluidos los de dato inválido.
    pub uncalculated: usize,
    /// Verificaciones que no cumplen.
    pub failed_checks: usize,
    /// Verificaciones sin evaluar o no concluyentes.
    pub pending_checks: usize,
    /// Tramos y verificaciones que no se evaluaron por un dato inválido.
    pub invalid_data: usize,
    /// De los tramos sin calcular, los que son por un dato inválido.
    pub invalid_runs: usize,
    /// Verificaciones de la norma que el programa no hace.
    pub out_of_scope: Vec<OutOfScope>,
    /// La capacidad interruptiva se verificó con la corriente de falla
    /// estimada por bus infinito, un método conservador (decisión 0023).
    pub conservative_fault: bool,
}

impl Verdict {
    /// Veredicto a partir de cuántos tramos se calcularon (y cuántos de los
    /// que no, por un dato inválido) y del resultado de cada verificación.
    pub fn new(
        runs: usize,
        uncalculated: usize,
        invalid_runs: usize,
        checks: impl IntoIterator<Item = Outcome>,
    ) -> Self {
        let (mut failed, mut pending, mut invalid) = (0, 0, invalid_runs);
        for outcome in checks {
            match outcome {
                Outcome::Fails => failed += 1,
                Outcome::Incomplete => pending += 1,
                Outcome::Invalid => invalid += 1,
                Outcome::Complies => {}
            }
        }
        let calculation = if runs > 0 && uncalculated == 0 {
            Calculation::Complete
        } else {
            Calculation::Incomplete
        };
        let compliance = if failed > 0 {
            Compliance::DoesNotConform
        } else if invalid > 0 {
            Compliance::InvalidData
        } else if pending > 0 || calculation == Calculation::Incomplete {
            Compliance::Incomplete
        } else {
            Compliance::Conforms
        };
        Verdict {
            calculation,
            compliance,
            professionally_approved: false,
            uncalculated,
            failed_checks: failed,
            pending_checks: pending,
            invalid_data: invalid,
            invalid_runs,
            out_of_scope: Vec::new(),
            conservative_fault: false,
        }
    }

    /// Marca que la capacidad interruptiva se verificó por bus infinito.
    pub fn with_conservative_fault(mut self) -> Self {
        self.conservative_fault = true;
        self.out_of_scope.clear();
        self
    }

    /// La capacidad interruptiva queda fuera de alcance solo en los lugares
    /// cuya falla depende de una fuente sin datos.
    pub fn with_out_of_scope_at(mut self, profile: &Profile, places: &[String]) -> Self {
        let places = match places {
            [] => return self,
            [one] => one.clone(),
            [rest @ .., last] => format!("{} y {last}", rest.join(", ")),
        };
        self.out_of_scope = vec![OutOfScope {
            name: format!("Capacidad interruptiva en {places}"),
            resolve_with:
                "falta la corriente de falla de la suministradora o los datos del transformador de la acometida"
                    .into(),
            references: vec![profile.reference(Rule::InterruptingRating)],
        }];
        self
    }

    /// El usuario apagó el cálculo de cortocircuito del proyecto: la
    /// capacidad interruptiva queda fuera de alcance y se dice por qué.
    pub fn with_fault_study_off(mut self, profile: &Profile) -> Self {
        self.out_of_scope = vec![OutOfScope {
            name: "Capacidad interruptiva".into(),
            resolve_with:
                "no se revisó en este proyecto porque el cálculo de cortocircuito está apagado; requiere estudio de cortocircuito"
                    .into(),
            references: vec![profile.reference(Rule::InterruptingRating)],
        }];
        self
    }

    /// Agrega las verificaciones fuera del alcance del programa.
    pub fn with_out_of_scope(mut self, profile: &Profile) -> Self {
        self.out_of_scope = out_of_scope(profile);
        self
    }

    /// `Fuera de alcance: capacidad interruptiva (…), requiere estudio de
    /// cortocircuito.` `None` si no hay nada fuera de alcance.
    pub fn out_of_scope_text(&self) -> Option<String> {
        if self.out_of_scope.is_empty() {
            return None;
        }
        let items: Vec<String> = self
            .out_of_scope
            .iter()
            .map(|o| format!("{}, {}", lowercase_first(&o.label()), o.resolve_with))
            .collect();
        Some(format!("Fuera de alcance: {}.", items.join("; ")))
    }

    /// Las tres líneas del veredicto: nivel, estado y explicación.
    pub fn lines(&self, profile_code: &str) -> [(&'static str, Status, String); 3] {
        let calculation = match self.calculation {
            Calculation::Complete => (
                Status::Complies,
                "Completo: cada tramo se calculó con datos y tablas del perfil.".to_string(),
            ),
            Calculation::Incomplete if self.uncalculated == 0 => (
                Status::NotEvaluated,
                "Incompleto: el proyecto no tiene tramos que calcular.".to_string(),
            ),
            Calculation::Incomplete => (
                Status::NotEvaluated,
                format!(
                    "Incompleto: {} sin calcular por falta de datos o de tablas.",
                    count(self.uncalculated, "tramo", "tramos")
                ),
            ),
        };
        let heading = self.compliance.heading();
        let compliance = match self.compliance {
            Compliance::Conforms => (
                Status::Complies,
                match self.out_of_scope_text() {
                    Some(note) => format!(
                        "{heading} con {profile_code}: lo que el programa verifica cumple y nada quedó sin evaluar. {note}"
                    ),
                    None if self.conservative_fault => format!(
                        "{heading} con {profile_code}: lo que el programa verifica cumple y nada quedó sin evaluar. La capacidad interruptiva se verificó con un método conservador: la falla estimada es una cota superior."
                    ),
                    None => format!(
                        "{heading} con {profile_code}: lo que el programa verifica cumple y nada quedó sin evaluar."
                    ),
                },
            ),
            Compliance::DoesNotConform => (
                Status::Fails,
                format!(
                    "{heading} con {profile_code}: {} no {}{}.",
                    count(self.failed_checks, "verificación", "verificaciones"),
                    if self.failed_checks == 1 {
                        "cumple"
                    } else {
                        "cumplen"
                    },
                    self.also(true, true)
                ),
            ),
            Compliance::InvalidData => (
                Status::NotEvaluated,
                format!(
                    "{heading}: {} con dato inválido{}.",
                    count(self.invalid_data, "evaluación", "evaluaciones"),
                    self.also(false, true)
                ),
            ),
            Compliance::Incomplete if self.incomplete() == 0 => (
                Status::NotEvaluated,
                format!("{heading}: no hay nada calculado que verificar."),
            ),
            Compliance::Incomplete => (
                Status::NotEvaluated,
                format!(
                    "{heading}: {}; no se puede declarar conforme.",
                    join_list(&self.incomplete_parts())
                ),
            ),
        };
        [
            ("Cálculo matemático", calculation.0, calculation.1),
            ("Verificación normativa", compliance.0, compliance.1),
            (
                "Aprobación profesional",
                Status::NotEvaluated,
                "Pendiente: la otorga quien revisa y firma el proyecto; el programa no la emite."
                    .to_string(),
            ),
        ]
    }
}

impl Verdict {
    /// Encabezado y explicación de una línea de [`Verdict::lines`], para
    /// mostrarlos por separado. El encabezado del dictamen puede llevar dos
    /// puntos («Datos inválidos: requiere corrección») y se toma completo; la
    /// norma pasa al inicio de la explicación: «Con NOM-…: lo que…».
    pub fn split_line(&self, detail: &str) -> (String, String) {
        let heading = self.compliance.heading();
        let (head, rest) = match detail.strip_prefix(heading) {
            Some(rest) => (heading.to_string(), rest),
            None => match detail.split_once(": ") {
                Some((head, rest)) => return (head.to_string(), rest.to_string()),
                None => return (detail.to_string(), String::new()),
            },
        };
        let explanation = match rest.strip_prefix(" con ") {
            Some(with_code) => format!("Con {with_code}"),
            None => rest.trim_start_matches(':').trim_start().to_string(),
        };
        (head, explanation)
    }

    /// Tramos sin calcular y verificaciones sin evaluar, sin los de dato
    /// inválido.
    fn incomplete(&self) -> usize {
        self.uncalculated - self.invalid_runs + self.pending_checks
    }

    /// `2 tramos sin calcular`, `3 verificaciones sin evaluar o no concluyentes`.
    fn incomplete_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        let uncalculated = self.uncalculated - self.invalid_runs;
        if uncalculated > 0 {
            parts.push(format!(
                "{} sin calcular",
                count(uncalculated, "tramo", "tramos")
            ));
        }
        if self.pending_checks > 0 {
            parts.push(format!(
                "{} sin evaluar o no {}",
                count(self.pending_checks, "verificación", "verificaciones"),
                if self.pending_checks == 1 {
                    "concluyente"
                } else {
                    "concluyentes"
                }
            ));
        }
        parts
    }

    /// ` · además 2 evaluaciones con dato inválido y 1 tramo sin calcular`: lo que hay
    /// debajo del encabezado.
    fn also(&self, invalid: bool, incomplete: bool) -> String {
        let mut parts = Vec::new();
        if invalid && self.invalid_data > 0 {
            parts.push(format!(
                "{} con dato inválido",
                count(self.invalid_data, "evaluación", "evaluaciones")
            ));
        }
        if incomplete {
            parts.extend(self.incomplete_parts());
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(" · además {}", join_list(&parts))
        }
    }
}

/// `a`, `a y b`, `a, b y c`.
pub(crate) fn join_list(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} y {last}", rest.join(", ")),
    }
}

/// `Capacidad` → `capacidad`.
fn lowercase_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `1 tramo`, `3 tramos`.
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}
