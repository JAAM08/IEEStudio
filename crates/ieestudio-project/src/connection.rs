//! Qué conexiones puede tener cada tramo. El sistema de suministro y el
//! tablero de donde sale un tramo ya dicen cuántas fases hay: aquí se deduce,
//! para que la interfaz no lo vuelva a preguntar y el cálculo rechace la
//! combinación imposible en vez de dar un número que parece válido.

use ieestudio_core::{CircuitKind, Error, System};

use crate::model::{Phase, Project};

/// Sistema de las barras de un tablero: el de la fuente o el del secundario
/// del transformador más cercano aguas arriba.
pub fn bus_system(project: &Project, panel: usize) -> System {
    let panels = &project.panels;
    let mut at = panel;
    // Cada paso sube un tablero; el límite evita quedarse aquí si el
    // proyecto tuviera un ciclo.
    for _ in 0..=panels.len() {
        if let Some(t) = &panels[at].transformer {
            return t.secondary;
        }
        match parent_of(project, at) {
            Some(parent) => at = parent,
            None => break,
        }
    }
    project.source.system
}

/// Sistema del que sale el alimentador de un tablero: las barras del tablero
/// que lo alimenta o la fuente.
pub fn feeder_system(project: &Project, panel: usize) -> System {
    match parent_of(project, panel) {
        Some(parent) => bus_system(project, parent),
        None => project.source.system,
    }
}

fn parent_of(project: &Project, panel: usize) -> Option<usize> {
    let parent = project.panels[panel].parent.as_deref()?;
    project.panels.iter().position(|p| p.id == parent)
}

/// Conexiones posibles para el alimentador de un tablero: las del sistema
/// del que sale, sin más fases que el tablero que lo alimenta y, si lleva
/// transformador, que también existan en su secundario.
pub fn feeder_kinds(project: &Project, panel: usize) -> Vec<CircuitKind> {
    let upstream = parent_of(project, panel).map(|parent| project.panels[parent].kind);
    let secondary = project.panels[panel]
        .transformer
        .as_ref()
        .map(|t| t.secondary);
    feeder_system(project, panel)
        .kinds()
        .into_iter()
        .filter(|kind| upstream.map_or(true, |up| kind.poles() <= up.poles()))
        .filter(|kind| secondary.map_or(true, |s| s.allows(*kind)))
        .collect()
}

/// Conexiones posibles para un circuito de un tablero: las del sistema de
/// sus barras, sin más fases que el tablero.
pub fn circuit_kinds(project: &Project, panel: usize) -> Vec<CircuitKind> {
    let own = project.panels[panel].kind;
    bus_system(project, panel)
        .kinds()
        .into_iter()
        .filter(|kind| kind.poles() <= own.poles())
        .collect()
}

/// Fases que tiene un tablero según la conexión de su alimentador.
pub fn panel_phases(kind: CircuitKind) -> &'static [Phase] {
    match kind {
        CircuitKind::LineToNeutral => &[Phase::A],
        CircuitKind::LineToLine => &[Phase::A, Phase::B],
        CircuitKind::ThreePhase => &[Phase::A, Phase::B, Phase::C],
    }
}

/// Formas de conectar un circuito a las fases de su tablero: cada una es una
/// elección válida. Si solo hay una, no hay nada que preguntar.
pub fn phase_choices(panel: CircuitKind, circuit: CircuitKind) -> Vec<Vec<Phase>> {
    let phases = panel_phases(panel);
    match circuit {
        CircuitKind::LineToNeutral => phases.iter().map(|p| vec![*p]).collect(),
        CircuitKind::LineToLine => match phases {
            [a, b] => vec![vec![*a, *b]],
            [a, b, c] => vec![vec![*a, *b], vec![*b, *c], vec![*a, *c]],
            _ => vec![],
        },
        CircuitKind::ThreePhase => match phases {
            [_, _, _] => vec![phases.to_vec()],
            _ => vec![],
        },
    }
}

/// Nombre de la conexión dentro de una frase: «una fase y neutro».
pub fn kind_phrase(kind: CircuitKind) -> &'static str {
    match kind {
        CircuitKind::LineToNeutral => "una fase y neutro",
        CircuitKind::LineToLine => "dos fases",
        CircuitKind::ThreePhase => "tres fases",
    }
}

/// Error si un tramo pide más fases que el tablero del que sale.
pub(crate) fn fits_upstream(
    kind: CircuitKind,
    upstream: CircuitKind,
    upstream_id: &str,
) -> Result<(), Error> {
    if kind.poles() <= upstream.poles() {
        return Ok(());
    }
    Err(Error::InvalidInput(format!(
        "el tablero «{upstream_id}» es de {}; no puede alimentar un tramo de {}",
        kind_phrase(upstream),
        kind_phrase(kind)
    )))
}

/// Tramos del proyecto cuya conexión no existe en su sistema o en su
/// tablero, con la conexión válida más cercana para corregirlos.
#[derive(Debug, Clone, PartialEq)]
pub struct Mismatch {
    pub panel: usize,
    /// `None` si es el alimentador del tablero.
    pub circuit: Option<usize>,
    pub kind: CircuitKind,
    /// La conexión posible con más fases sin pasar de la pedida; si no hay,
    /// la menor de las posibles. `None` si el tramo no tiene ninguna.
    pub nearest: Option<CircuitKind>,
}

/// La conexión permitida más cercana a `kind`.
pub fn nearest_kind(allowed: &[CircuitKind], kind: CircuitKind) -> Option<CircuitKind> {
    allowed
        .iter()
        .rev()
        .find(|k| k.poles() <= kind.poles())
        .or(allowed.first())
        .copied()
}

pub fn mismatches(project: &Project) -> Vec<Mismatch> {
    let mut found = Vec::new();
    for (i, panel) in project.panels.iter().enumerate() {
        let allowed = feeder_kinds(project, i);
        if !allowed.contains(&panel.kind) {
            found.push(Mismatch {
                panel: i,
                circuit: None,
                kind: panel.kind,
                nearest: nearest_kind(&allowed, panel.kind),
            });
        }
        let allowed = circuit_kinds(project, i);
        for (k, circuit) in panel.circuits.iter().enumerate() {
            if !allowed.contains(&circuit.kind) {
                found.push(Mismatch {
                    panel: i,
                    circuit: Some(k),
                    kind: circuit.kind,
                    nearest: nearest_kind(&allowed, circuit.kind),
                });
            }
        }
    }
    found
}

/// Pasa cada tramo con una conexión imposible a la más cercana que sí
/// existe y acomoda sus fases. Devuelve cuántos tramos cambió. Corregir un
/// tablero puede dejar fuera a sus circuitos, así que se repite hasta que no
/// quede ninguno.
pub fn fix(project: &mut Project) -> usize {
    let mut fixed = 0;
    // Cada vuelta corrige al menos un nivel; no hay más niveles que tableros.
    for _ in 0..=project.panels.len() {
        let pending: Vec<Mismatch> = mismatches(project)
            .into_iter()
            .filter(|m| m.nearest.is_some())
            .collect();
        if pending.is_empty() {
            break;
        }
        for m in pending {
            let Some(kind) = m.nearest else { continue };
            match m.circuit {
                None => project.panels[m.panel].kind = kind,
                Some(c) => project.panels[m.panel].circuits[c].kind = kind,
            }
            fixed += 1;
        }
    }
    // Fases: las que ya no corresponden a la conexión pasan a la primera
    // forma válida; balancear las reparte después.
    for panel in &mut project.panels {
        let own = panel.kind;
        for circuit in &mut panel.circuits {
            let choices = phase_choices(own, circuit.kind);
            let valid = choices.iter().any(|choice| {
                choice.len() == circuit.phases.len()
                    && choice.iter().all(|p| circuit.phases.contains(p))
            });
            if !valid {
                if let Some(first) = choices.first() {
                    circuit.phases = first.clone();
                }
            }
        }
    }
    fixed
}

#[cfg(test)]
mod tests {
    use super::*;
    use CircuitKind::{LineToLine, LineToNeutral, ThreePhase};

    #[test]
    fn phases_are_only_a_choice_when_there_is_more_than_one_way() {
        assert_eq!(phase_choices(ThreePhase, ThreePhase).len(), 1);
        assert_eq!(phase_choices(LineToLine, LineToLine).len(), 1);
        assert_eq!(phase_choices(LineToNeutral, LineToNeutral).len(), 1);
        assert_eq!(phase_choices(ThreePhase, LineToNeutral).len(), 3);
        assert_eq!(phase_choices(ThreePhase, LineToLine).len(), 3);
        assert_eq!(phase_choices(LineToLine, LineToNeutral).len(), 2);
        assert!(phase_choices(LineToLine, ThreePhase).is_empty());
    }

    #[test]
    fn nearest_keeps_as_many_phases_as_possible() {
        assert_eq!(
            nearest_kind(&[LineToNeutral, LineToLine], ThreePhase),
            Some(LineToLine)
        );
        assert_eq!(
            nearest_kind(&[LineToLine, ThreePhase], LineToNeutral),
            Some(LineToLine)
        );
        assert_eq!(nearest_kind(&[], ThreePhase), None);
    }

    #[test]
    fn fixing_a_project_leaves_no_impossible_connection() {
        let mut project = crate::example::house();
        project.source.system = System::SPLIT_240_120;
        assert!(!mismatches(&project).is_empty());
        let fixed = fix(&mut project);
        assert!(fixed >= 1);
        assert!(mismatches(&project).is_empty());
        assert_eq!(project.panels[0].kind, LineToLine);
        for circuit in &project.panels[0].circuits {
            assert!(!circuit.phases.contains(&Phase::C), "{}", circuit.id);
        }
        let result = crate::calculate(&project).unwrap();
        assert!(result.panels[0].feeder.run.is_some());
    }
}
