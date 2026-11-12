//! Estado de evaluación de un resultado y su trazabilidad (decisión 0027).
//!
//! El estado dice **qué tan completo** es un resultado, no si cumple: un
//! resultado válido puede cumplir o no cumplir. El cálculo lo decide una vez
//! aquí y la interfaz, la memoria y la línea de comandos lo muestran tal cual,
//! sin repetir la lógica.

use serde::Serialize;

/// Qué tan completo es un resultado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Evaluation {
    /// Calculado con información suficiente y bajo hipótesis identificadas.
    Valid,
    /// Hay un resultado, pero faltan datos que pueden modificarlo, o el
    /// método conservador no alcanza para concluir.
    Inconclusive,
    /// No hay condiciones suficientes para calcular: falta un dato.
    NotEvaluated,
    /// Los datos son inconsistentes y no permiten calcular: una potencia o
    /// una impedancia que no es mayor que cero, una conexión que el sistema
    /// no tiene.
    Invalid,
}

impl Evaluation {
    pub fn label(self) -> &'static str {
        match self {
            Evaluation::Valid => "Válido",
            Evaluation::Inconclusive => "No concluyente",
            Evaluation::NotEvaluated => "Sin evaluar",
            Evaluation::Invalid => "Dato inválido",
        }
    }

    fn rank(self) -> u8 {
        match self {
            Evaluation::Valid => 0,
            Evaluation::Inconclusive => 1,
            Evaluation::NotEvaluated => 2,
            Evaluation::Invalid => 3,
        }
    }

    /// El menos completo de los dos: un resultado vale lo que su parte más
    /// débil.
    pub fn worst(self, other: Evaluation) -> Evaluation {
        if other.rank() > self.rank() {
            other
        } else {
            self
        }
    }
}

/// Un dato que se usó, con su valor como se muestra.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Datum {
    pub name: String,
    pub value: String,
}

impl Datum {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Datum {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Lo que un resultado puede contar de sí mismo: qué se calculó, dónde, con
/// qué datos e hipótesis, qué elementos contribuyeron, qué falta y qué
/// problemas tiene.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Trace {
    pub evaluation: Evaluation,
    /// `Corriente de falla estimada`.
    pub what: String,
    /// `las barras de TF-1`.
    pub place: String,
    pub inputs: Vec<Datum>,
    pub assumptions: Vec<String>,
    /// Fuentes y motores que aportan, con su valor.
    pub contributors: Vec<String>,
    /// Datos que faltan y qué cambiarían.
    pub missing: Vec<String>,
    /// Errores de datos y advertencias.
    pub problems: Vec<String>,
}

impl Trace {
    /// Las secciones con contenido, en orden, para mostrarlas igual en todas
    /// partes: `("Datos usados", […])`.
    pub fn sections(&self) -> Vec<(&'static str, Vec<String>)> {
        let inputs: Vec<String> = self
            .inputs
            .iter()
            .map(|d| format!("{}: {}", d.name, d.value))
            .collect();
        [
            ("Datos usados", inputs),
            ("Hipótesis", self.assumptions.clone()),
            ("Contribuyen", self.contributors.clone()),
            ("Falta", self.missing.clone()),
            ("Problemas", self.problems.clone()),
        ]
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weakest_part_decides() {
        use Evaluation::*;
        assert_eq!(Valid.worst(Inconclusive), Inconclusive);
        assert_eq!(Inconclusive.worst(Valid), Inconclusive);
        assert_eq!(Inconclusive.worst(NotEvaluated), NotEvaluated);
        assert_eq!(Invalid.worst(NotEvaluated), Invalid);
        assert_eq!(Valid.worst(Valid), Valid);
    }
}
