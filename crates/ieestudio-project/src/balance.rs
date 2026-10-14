//! Asignación de fases para balancear un tablero.
//!
//! Los circuitos se reparten de mayor a menor potencia aparente: cada uno va
//! a la fase (o al par de fases) que deja la mayor carga por fase más baja.
//! Es el método de «la carga mayor primero»; no garantiza el óptimo, pero
//! queda muy cerca y es fácil de seguir a mano.

use ieestudio_core::CircuitKind;

use crate::model::{Panel, Phase};

const ALL: [Phase; 3] = [Phase::A, Phase::B, Phase::C];

/// Hasta este número de circuitos de una y dos fases se prueban todas las
/// combinaciones (3¹⁰ = 59 049).
const EXHAUSTIVE_LIMIT: usize = 10;

fn slot(phase: Phase) -> usize {
    match phase {
        Phase::A => 0,
        Phase::B => 1,
        Phase::C => 2,
    }
}

/// Fases disponibles en un tablero según la conexión de su alimentador.
fn available(panel: &Panel) -> &'static [Phase] {
    match panel.kind {
        CircuitKind::LineToNeutral => &ALL[..1],
        CircuitKind::LineToLine => &ALL[..2],
        CircuitKind::ThreePhase => &ALL,
    }
}

/// Desbalance en porciento entre la fase más cargada y la menos cargada de
/// las disponibles.
pub fn imbalance_percent(loads: [f64; 3], phases: usize) -> f64 {
    let used = &loads[..phases];
    let max = used.iter().cloned().fold(0.0, f64::max);
    let min = used.iter().cloned().fold(f64::INFINITY, f64::min);
    if max > 0.0 {
        (max - min) / max * 100.0
    } else {
        0.0
    }
}

/// Fases propuestas para cada circuito del tablero, en el mismo orden.
/// `loads_va` es la potencia aparente de cada circuito. Un circuito que pide
/// más fases de las que tiene el tablero conserva las suyas.
pub fn balance(panel: &Panel, loads_va: &[f64]) -> Vec<Vec<Phase>> {
    let phases = available(panel);
    let mut result: Vec<Vec<Phase>> = panel.circuits.iter().map(|c| c.phases.clone()).collect();
    let mut totals = [0.0f64; 3];
    // Los circuitos que ocupan todas las fases cargan a todas por igual.
    let mut order: Vec<usize> = Vec::new();
    for (i, c) in panel.circuits.iter().enumerate() {
        let poles = match c.kind {
            CircuitKind::LineToNeutral => 1,
            CircuitKind::LineToLine => 2,
            CircuitKind::ThreePhase => 3,
        };
        let va = loads_va.get(i).copied().unwrap_or(0.0);
        if poles > phases.len() {
            continue;
        }
        if poles == phases.len() {
            result[i] = phases.to_vec();
            for p in phases {
                totals[slot(*p)] += va / poles as f64;
            }
        } else {
            order.push(i);
        }
    }
    order.sort_by(|&a, &b| {
        let va = |i: usize| loads_va.get(i).copied().unwrap_or(0.0);
        va(b).total_cmp(&va(a)).then(a.cmp(&b))
    });
    let options = |kind: CircuitKind| -> Vec<Vec<Phase>> {
        match kind {
            CircuitKind::LineToNeutral => phases.iter().map(|p| vec![*p]).collect(),
            CircuitKind::LineToLine => {
                let mut pairs = Vec::new();
                for (k, a) in phases.iter().enumerate() {
                    for b in &phases[k + 1..] {
                        pairs.push(vec![*a, *b]);
                    }
                }
                pairs
            }
            CircuitKind::ThreePhase => vec![],
        }
    };
    let va = |i: usize| loads_va.get(i).copied().unwrap_or(0.0);
    let add = |t: &mut [f64; 3], opt: &[Phase], amount: f64| {
        for p in opt {
            t[slot(*p)] += amount / opt.len() as f64;
        }
    };
    // Primero, la carga mayor a donde menos pesa.
    for &i in &order {
        let best = options(panel.circuits[i].kind)
            .into_iter()
            .min_by(|x, y| {
                let mut tx = totals;
                add(&mut tx, x, va(i));
                let mut ty = totals;
                add(&mut ty, y, va(i));
                score(&tx, phases.len()).total_cmp(&score(&ty, phases.len()))
            })
            .expect("siempre hay al menos una opción");
        add(&mut totals, &best, va(i));
        result[i] = best;
    }
    // Con pocos circuitos se prueban todas las combinaciones: el resultado es
    // el mejor posible.
    if order.len() <= EXHAUSTIVE_LIMIT {
        let choices: Vec<Vec<Vec<Phase>>> = order
            .iter()
            .map(|&i| options(panel.circuits[i].kind))
            .collect();
        // Lo que no se mueve: los circuitos que ocupan todas las fases.
        let mut base = [0.0f64; 3];
        for (i, assigned) in result.iter().enumerate() {
            if !order.contains(&i) && assigned.len() == phases.len() {
                add(&mut base, assigned, va(i));
            }
        }
        let mut best_score = score(&totals, phases.len());
        let mut pick = vec![0usize; order.len()];
        let mut best_pick: Option<Vec<usize>> = None;
        loop {
            let mut t = base;
            for (k, &i) in order.iter().enumerate() {
                add(&mut t, &choices[k][pick[k]], va(i));
            }
            let sc = score(&t, phases.len());
            if sc < best_score - 1e-9 {
                best_score = sc;
                best_pick = Some(pick.clone());
            }
            // Siguiente combinación.
            let mut k = 0;
            loop {
                if k == pick.len() {
                    break;
                }
                pick[k] += 1;
                if pick[k] < choices[k].len() {
                    break;
                }
                pick[k] = 0;
                k += 1;
            }
            if k == pick.len() {
                break;
            }
        }
        if let Some(best) = best_pick {
            for (k, &i) in order.iter().enumerate() {
                result[i] = choices[k][best[k]].clone();
            }
        }
        return result;
    }
    // Con muchos, se mueve un circuito o se intercambian dos mientras mejore.
    for _ in 0..200 {
        let current = score(&totals, phases.len());
        let mut improved = false;
        'search: for &i in &order {
            for opt in options(panel.circuits[i].kind) {
                if opt == result[i] {
                    continue;
                }
                let mut t = totals;
                add(&mut t, &result[i], -va(i));
                add(&mut t, &opt, va(i));
                if score(&t, phases.len()) < current - 1e-9 {
                    totals = t;
                    result[i] = opt;
                    improved = true;
                    break 'search;
                }
            }
            for &j in &order {
                if j <= i
                    || panel.circuits[i].kind != panel.circuits[j].kind
                    || result[i] == result[j]
                {
                    continue;
                }
                let mut t = totals;
                add(&mut t, &result[i], -va(i));
                add(&mut t, &result[j], -va(j));
                add(&mut t, &result[j], va(i));
                add(&mut t, &result[i], va(j));
                if score(&t, phases.len()) < current - 1e-9 {
                    totals = t;
                    result.swap(i, j);
                    improved = true;
                    break 'search;
                }
            }
        }
        if !improved {
            break;
        }
    }
    result
}

/// Qué tan desbalanceado está: la diferencia entre la fase más cargada y la
/// menos cargada, y como desempate la suma de cuadrados.
fn score(t: &[f64; 3], phases: usize) -> f64 {
    let used = &t[..phases];
    let max = used.iter().cloned().fold(f64::MIN, f64::max);
    let min = used.iter().cloned().fold(f64::MAX, f64::min);
    let sum_sq: f64 = used.iter().map(|v| v * v).sum();
    (max - min) * 1e6 + sum_sq * 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::example;

    fn loads(panel: &Panel, phases: &[Vec<Phase>], va: &[f64]) -> [f64; 3] {
        let mut t = [0.0; 3];
        for (k, ph) in phases.iter().enumerate() {
            for p in ph {
                t[slot(*p)] += va[k] / ph.len() as f64;
            }
        }
        let _ = panel;
        t
    }

    /// Prueba matemática: seis cargas de una fase de 3, 3, 2, 2, 1 y 1 kVA
    /// quedan en 4 kVA por fase.
    #[test]
    fn equal_split_of_single_phase_loads() {
        let mut panel = example::house().panels[0].clone();
        for c in &mut panel.circuits {
            c.kind = CircuitKind::LineToNeutral;
        }
        let va = [3000.0, 1000.0, 2000.0, 3000.0, 1000.0, 2000.0];
        let phases = balance(&panel, &va);
        let t = loads(&panel, &phases, &va);
        assert_eq!(t, [4000.0, 4000.0, 4000.0]);
        assert_eq!(imbalance_percent(t, 3), 0.0);
    }

    /// El ejemplo de la casa baja de 29.5 % de desbalance a 9.64 %, el mejor
    /// posible con esas cargas (se comprobó probando todas las combinaciones).
    #[test]
    fn the_example_house_gets_better() {
        let project = example::house();
        let result = crate::calculate(&project).unwrap();
        let panel = &project.panels[0];
        let va: Vec<f64> = panel
            .circuits
            .iter()
            .zip(&result.panels[0].circuits)
            .map(|(c, r)| r.load_w / c.load.power_factor)
            .collect();
        let phases = balance(panel, &va);
        let mut balanced = project.clone();
        for (c, p) in balanced.panels[0].circuits.iter_mut().zip(phases) {
            assert_eq!(p.len(), crate::calc::poles(c.kind) as usize);
            c.phases = p;
        }
        let after = crate::calculate(&balanced).unwrap().panels[0]
            .imbalance_percent
            .unwrap();
        assert!((after - 9.64).abs() < 0.01, "{after}");
        assert!(after < result.panels[0].imbalance_percent.unwrap());
    }

    /// Un tablero grande (30 circuitos) queda casi balanceado por la búsqueda
    /// local.
    #[test]
    fn a_large_panel_is_balanced() {
        let mut panel = example::house().panels[0].clone();
        let template = panel.circuits[0].clone();
        panel.circuits = (0..30)
            .map(|k| {
                let mut c = template.clone();
                c.id = format!("C-{k}");
                c.kind = if k % 5 == 0 {
                    CircuitKind::LineToLine
                } else {
                    CircuitKind::LineToNeutral
                };
                c
            })
            .collect();
        let va: Vec<f64> = (0..30).map(|k| 500.0 + f64::from(k * 137 % 1900)).collect();
        let phases = balance(&panel, &va);
        let t = loads(&panel, &phases, &va);
        assert!(imbalance_percent(t, 3) < 2.0, "{t:?}");
    }

    /// En un tablero de dos fases no se usa la C, y uno trifásico no se toca.
    #[test]
    fn two_phase_panels_use_a_and_b() {
        let mut panel = example::house().panels[0].clone();
        panel.kind = CircuitKind::LineToLine;
        panel.circuits[0].kind = CircuitKind::ThreePhase;
        panel.circuits[0].phases = vec![];
        let va = [1000.0; 6];
        let phases = balance(&panel, &va);
        assert!(phases.iter().flatten().all(|p| *p != Phase::C));
        assert!(phases[0].is_empty());
    }
}
