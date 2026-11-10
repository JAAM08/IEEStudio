//! Veredicto en tres niveles, que nunca se mezclan:
//!
//! 1. **Cálculo matemático**: si cada tramo se pudo calcular con los datos y
//!    las tablas disponibles.
//! 2. **Verificación normativa**: si todo lo calculado cumple el perfil. Solo
//!    es «conforme» cuando todo cumple y no queda nada sin evaluar.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Compliance {
    /// Todas las verificaciones que hace el programa cumplen y ninguna quedó
    /// sin evaluar. Las que están fuera de su alcance no cuentan.
    Conforms,
    /// Al menos una verificación no cumple.
    DoesNotConform,
    /// Nada incumple, pero hay verificaciones sin evaluar.
    Pending,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Verdict {
    pub calculation: Calculation,
    pub compliance: Compliance,
    /// Siempre `false`: el programa no aprueba proyectos.
    pub professionally_approved: bool,
    /// Tramos sin calcular.
    pub uncalculated: usize,
    /// Verificaciones que no cumplen.
    pub failed_checks: usize,
    /// Verificaciones sin evaluar.
    pub pending_checks: usize,
    /// Verificaciones de la norma que el programa no hace.
    pub out_of_scope: Vec<OutOfScope>,
    /// La capacidad interruptiva se verificó con la corriente de falla
    /// estimada por bus infinito, un método conservador (decisión 0023).
    pub conservative_fault: bool,
}

impl Verdict {
    /// Veredicto a partir de cuántos tramos se calcularon y del estado de
    /// cada verificación.
    pub fn new(runs: usize, uncalculated: usize, checks: impl IntoIterator<Item = Status>) -> Self {
        let (mut failed, mut pending) = (0, 0);
        for status in checks {
            match status {
                Status::Fails => failed += 1,
                Status::NotEvaluated => pending += 1,
                Status::Complies => {}
            }
        }
        let calculation = if runs > 0 && uncalculated == 0 {
            Calculation::Complete
        } else {
            Calculation::Incomplete
        };
        let compliance = if failed > 0 {
            Compliance::DoesNotConform
        } else if pending > 0 || calculation == Calculation::Incomplete {
            Compliance::Pending
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
        let compliance = match self.compliance {
            Compliance::Conforms => (
                Status::Complies,
                match self.out_of_scope_text() {
                    Some(note) => format!(
                        "Conforme en lo evaluado con {profile_code}: lo que el programa verifica cumple y nada quedó sin evaluar. {note}"
                    ),
                    None if self.conservative_fault => format!(
                        "Conforme con {profile_code}: todo cumple y nada quedó sin evaluar. La capacidad interruptiva se verificó con un método conservador (bus infinito)."
                    ),
                    None => format!(
                        "Conforme con {profile_code}: todo cumple y nada quedó sin evaluar."
                    ),
                },
            ),
            Compliance::DoesNotConform => (
                Status::Fails,
                format!(
                    "No conforme: {} no {}.",
                    count(self.failed_checks, "verificación", "verificaciones"),
                    if self.failed_checks == 1 {
                        "cumple"
                    } else {
                        "cumplen"
                    }
                ),
            ),
            Compliance::Pending if self.pending_checks == 0 && self.uncalculated == 0 => (
                Status::NotEvaluated,
                "Pendiente: no hay nada calculado que verificar.".to_string(),
            ),
            Compliance::Pending => {
                let mut parts = Vec::new();
                if self.uncalculated > 0 {
                    parts.push(format!(
                        "{} sin calcular",
                        count(self.uncalculated, "tramo", "tramos")
                    ));
                }
                if self.pending_checks > 0 {
                    parts.push(format!(
                        "{} sin evaluar",
                        count(self.pending_checks, "verificación", "verificaciones")
                    ));
                }
                (
                    Status::NotEvaluated,
                    format!(
                        "Pendiente: {}; no se puede declarar conforme.",
                        parts.join(" y ")
                    ),
                )
            }
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
