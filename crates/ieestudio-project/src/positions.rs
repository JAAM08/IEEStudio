//! Posiciones de los interruptores en un tablero, para el unifilar.
//!
//! En un tablero de distribución los espacios se numeran con los nones a la
//! izquierda (1, 3, 5…) y los pares a la derecha (2, 4, 6…); cada renglón
//! toma una fase, en el orden A, B, C, A… Un circuito de una fase va en un
//! espacio de su fase; uno de dos o tres fases, en espacios seguidos del
//! mismo lado. Las posiciones que fijó el usuario se respetan y las demás se
//! acomodan alrededor. Es una propuesta: no interviene en ningún cálculo.

use ieestudio_core::CircuitKind;

use crate::model::{Phase, Project};

fn phase_of_row(row: usize) -> Phase {
    match row % 3 {
        0 => Phase::A,
        1 => Phase::B,
        _ => Phase::C,
    }
}

/// Espacio de un lado y renglón: `1` es izquierda del primero, `2` derecha.
fn number(side: usize, row: usize) -> u32 {
    (2 * row + side + 1) as u32
}

struct Slots {
    /// Ocupación por lado (0 izquierda, 1 derecha) y renglón.
    taken: [Vec<bool>; 2],
    /// En un tablero de una fase todos los espacios son de la misma fase.
    single_phase: bool,
}

impl Slots {
    fn is_free(&self, side: usize, row: usize) -> bool {
        !self.taken[side].get(row).copied().unwrap_or(false)
    }

    fn take(&mut self, side: usize, row: usize) {
        let list = &mut self.taken[side];
        if list.len() <= row {
            list.resize(row + 1, false);
        }
        list[row] = true;
    }

    /// Primer lugar con `count` espacios seguidos libres que empiecen en la
    /// fase `first` (o en cualquiera si no se pide).
    fn place(&mut self, count: usize, first: Option<Phase>) -> Vec<u32> {
        for row in 0.. {
            for side in 0..2 {
                let fits = (row..row + count).all(|r| self.is_free(side, r));
                let phase_ok =
                    self.single_phase || first.map_or(true, |ph| phase_of_row(row) == ph);
                if fits && phase_ok {
                    for r in row..row + count {
                        self.take(side, r);
                    }
                    return (row..row + count).map(|r| number(side, r)).collect();
                }
            }
        }
        unreachable!("siempre hay renglones libres más abajo")
    }

    /// Marca como ocupados los espacios que escribió el usuario.
    fn reserve(&mut self, text: &str) {
        for n in text
            .split([',', ' ', '-'])
            .filter_map(|p| p.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
        {
            let side = (n - 1) % 2;
            let row = (n - 1) / 2;
            self.take(side, row);
        }
    }
}

/// Posiciones de lo que cuelga de un tablero: primero sus circuitos, en
/// orden, y después los alimentadores de sus subtableros, en el orden del
/// proyecto. Cada una como se escribe en el unifilar: `1`, `2,4`, `13,15,17`.
pub fn panel_positions(project: &Project, panel: usize) -> (Vec<String>, Vec<String>) {
    let p = &project.panels[panel];
    let children: Vec<usize> = (0..project.panels.len())
        .filter(|&k| project.panels[k].parent.as_ref() == Some(&p.id))
        .collect();
    let mut slots = Slots {
        taken: [Vec::new(), Vec::new()],
        single_phase: p.kind == CircuitKind::LineToNeutral,
    };
    // Primero lo que fijó el usuario, para no encimarse.
    for c in &p.circuits {
        if let Some(text) = &c.overrides.position {
            slots.reserve(text);
        }
    }
    for &k in &children {
        if let Some(text) = &project.panels[k].feeder.overrides.position {
            slots.reserve(text);
        }
    }
    let poles = |kind: CircuitKind| match kind {
        CircuitKind::LineToNeutral => 1,
        CircuitKind::LineToLine => 2,
        CircuitKind::ThreePhase => 3,
    };
    let join = |numbers: Vec<u32>| {
        numbers
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    let circuits = p
        .circuits
        .iter()
        .map(|c| match &c.overrides.position {
            Some(text) => text.clone(),
            None => {
                // La primera fase asignada, en el orden A, B, C.
                let first = [Phase::A, Phase::B, Phase::C]
                    .into_iter()
                    .find(|ph| c.phases.contains(ph));
                let count = poles(c.kind);
                // Dos fases B y C… o C y A: empieza en la que va primero en
                // el renglón.
                let first = match (count, first, c.phases.contains(&Phase::C)) {
                    (2, Some(Phase::A), true) if !c.phases.contains(&Phase::B) => Some(Phase::C),
                    (3, _, _) => None,
                    (_, f, _) => f,
                };
                join(slots.place(count, first))
            }
        })
        .collect();
    let subpanels = children
        .iter()
        .map(|&k| {
            let child = &project.panels[k];
            match &child.feeder.overrides.position {
                Some(text) => text.clone(),
                None => join(slots.place(poles(child.kind), None)),
            }
        })
        .collect();
    (circuits, subpanels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_house_positions_follow_the_phases() {
        let project = crate::example::house();
        let (circuits, subpanels) = panel_positions(&project, 0);
        assert!(subpanels.is_empty());
        assert_eq!(circuits.len(), project.panels[0].circuits.len());
        for (c, pos) in project.panels[0].circuits.iter().zip(&circuits) {
            let numbers: Vec<usize> = pos.split(',').map(|n| n.parse().unwrap()).collect();
            assert_eq!(
                numbers.len(),
                match c.kind {
                    CircuitKind::LineToNeutral => 1,
                    CircuitKind::LineToLine => 2,
                    CircuitKind::ThreePhase => 3,
                }
            );
            // Cada espacio es de una de las fases del circuito.
            for n in &numbers {
                let row = (n - 1) / 2;
                assert!(c.phases.contains(&phase_of_row(row)), "{} en {n}", c.id);
            }
        }
        // Ninguna posición se repite.
        let mut all: Vec<&str> = circuits.iter().flat_map(|p| p.split(',')).collect();
        let n = all.len();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), n);
    }

    #[test]
    fn single_phase_circuits_fill_by_phase() {
        let mut project = crate::example::house();
        let base = project.panels[0].circuits[0].clone();
        project.panels[0].circuits.clear();
        for (k, ph) in [Phase::A, Phase::B, Phase::C, Phase::A]
            .into_iter()
            .enumerate()
        {
            let mut c = base.clone();
            c.id = format!("C-{k}");
            c.kind = CircuitKind::LineToNeutral;
            c.phases = vec![ph];
            project.panels[0].circuits.push(c);
        }
        let (circuits, _) = panel_positions(&project, 0);
        assert_eq!(circuits, vec!["1", "3", "5", "2"]);
    }

    #[test]
    fn a_three_phase_breaker_takes_three_rows_and_fixed_ones_are_kept() {
        let mut project = crate::example::house();
        let base = project.panels[0].circuits[0].clone();
        project.panels[0].circuits.clear();
        let mut fixed = base.clone();
        fixed.id = "F".into();
        fixed.overrides.position = Some("1".into());
        let mut three = base.clone();
        three.id = "T".into();
        three.kind = CircuitKind::ThreePhase;
        three.phases = vec![Phase::A, Phase::B, Phase::C];
        project.panels[0].circuits.push(fixed);
        project.panels[0].circuits.push(three);
        let (circuits, _) = panel_positions(&project, 0);
        assert_eq!(circuits[0], "1");
        assert_eq!(circuits[1], "2,4,6");
    }
}
