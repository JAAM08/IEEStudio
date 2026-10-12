//! Veredicto en tres niveles, que nunca se mezclan:
//!
//! 1. **Cálculo matemático**: si cada tramo se pudo calcular con los datos y
//!    las tablas disponibles.
//! 2. **Verificación normativa**: si todo lo calculado cumple el perfil. Solo
//!    es «conforme» cuando todo cumple y no queda nada sin evaluar.
//! 3. **Aprobación profesional**: siempre pendiente. La otorga quien firma el
//!    proyecto; el programa nunca la emite.

use serde::Serialize;

use crate::Status;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Calculation {
    /// Cada tramo se calculó.
    Complete,
    /// Algún tramo no se pudo calcular, o no hay nada que calcular.
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Compliance {
    /// Todas las verificaciones cumplen y ninguna quedó sin evaluar.
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
            pending_checks: pending + uncalculated,
        }
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
                    "Incompleto: {} tramo(s) sin calcular por falta de datos o de tablas.",
                    self.uncalculated
                ),
            ),
        };
        let compliance = match self.compliance {
            Compliance::Conforms => (
                Status::Complies,
                format!("Conforme con {profile_code}: todo cumple y nada quedó sin evaluar."),
            ),
            Compliance::DoesNotConform => (
                Status::Fails,
                format!(
                    "No conforme: {} verificación(es) no cumplen.",
                    self.failed_checks
                ),
            ),
            Compliance::Pending => (
                Status::NotEvaluated,
                format!(
                    "Pendiente: {} verificación(es) sin evaluar; no se puede declarar conforme.",
                    self.pending_checks
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
