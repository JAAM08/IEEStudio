//! El proyecto abierto: la única fuente de verdad de todas las vistas.

use std::path::PathBuf;

use voltforge_core::{CircuitKind, System};
use voltforge_project::{
    calculate, Circuit, Feeder, Load, LoadKind, Overrides, Panel, PanelResult, Phase, Project,
    ProjectResult, RunResult, Settings, Source, FORMAT_VERSION,
};

/// Lo que está seleccionado. Todas las vistas leen y cambian esta selección.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Project,
    /// El tablero y su alimentador.
    Feeder(usize),
    /// Tablero y circuito.
    Circuit(usize, usize),
}

pub struct Doc {
    pub project: Project,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub result: Result<ProjectResult, String>,
    pub selection: Selection,
    /// Cambia cada vez que el resultado cambia; la vista previa lo compara.
    pub revision: u64,
}

pub fn empty_project() -> Project {
    Project {
        format_version: FORMAT_VERSION,
        name: "Proyecto nuevo".to_string(),
        profile: "nom-001-sede-2012".to_string(),
        settings: Settings::default(),
        source: Source {
            name: "Acometida".to_string(),
            system: System::WYE_220_127,
        },
        panels: vec![new_panel("TG-1", "Tablero general", None)],
    }
}

fn new_panel(id: &str, name: &str, parent: Option<String>) -> Panel {
    Panel {
        id: id.to_string(),
        name: name.to_string(),
        parent,
        kind: CircuitKind::ThreePhase,
        feeder: Feeder {
            length_m: 10.0,
            demand_factor: 1.0,
            has_neutral: true,
            overrides: Overrides::default(),
        },
        circuits: vec![],
    }
}

impl Doc {
    pub fn new(project: Project, path: Option<PathBuf>) -> Self {
        let mut doc = Doc {
            project,
            path,
            dirty: false,
            result: Err(String::new()),
            selection: Selection::Project,
            revision: 0,
        };
        doc.recalculate();
        if !doc.project.panels.is_empty() {
            doc.selection = Selection::Feeder(0);
        }
        doc
    }

    pub fn recalculate(&mut self) {
        self.result = calculate(&self.project);
        self.revision += 1;
        self.clamp_selection();
    }

    /// Marca un cambio del usuario: recalcula y deja el proyecto sin guardar.
    pub fn edited(&mut self) {
        self.dirty = true;
        self.recalculate();
    }

    fn clamp_selection(&mut self) {
        let panels = &self.project.panels;
        self.selection = match self.selection {
            Selection::Circuit(p, c) if p < panels.len() && c < panels[p].circuits.len() => {
                Selection::Circuit(p, c)
            }
            Selection::Circuit(p, _) | Selection::Feeder(p) if p < panels.len() => {
                Selection::Feeder(p)
            }
            _ if panels.is_empty() => Selection::Project,
            Selection::Project => Selection::Project,
            _ => Selection::Feeder(0),
        };
    }

    /// Tablero activo: el de la selección, o el primero.
    pub fn panel_index(&self) -> Option<usize> {
        match self.selection {
            Selection::Feeder(p) | Selection::Circuit(p, _) => Some(p),
            Selection::Project => (!self.project.panels.is_empty()).then_some(0),
        }
    }

    pub fn panel_result(&self, index: usize) -> Option<&PanelResult> {
        let id = &self.project.panels.get(index)?.id;
        self.result
            .as_ref()
            .ok()?
            .panels
            .iter()
            .find(|p| &p.id == id)
    }

    /// Resultado del alimentador o del circuito seleccionado.
    pub fn selected_run(&self) -> Option<&RunResult> {
        match self.selection {
            Selection::Feeder(p) => Some(&self.panel_result(p)?.feeder),
            Selection::Circuit(p, c) => self.panel_result(p)?.circuits.get(c),
            Selection::Project => None,
        }
    }

    /// Clave estable de la selección, para recordar qué grupos se abrieron.
    pub fn selection_key(&self) -> (u8, usize, usize) {
        match self.selection {
            Selection::Project => (0, 0, 0),
            Selection::Feeder(p) => (1, p, 0),
            Selection::Circuit(p, c) => (2, p, c),
        }
    }

    /// Texto de la selección para la barra de estado: `Proyecto › TG-1 › C-02`.
    pub fn breadcrumb(&self) -> Vec<String> {
        let mut parts = vec![self.project.name.clone()];
        match self.selection {
            Selection::Project => {}
            Selection::Feeder(p) => parts.push(self.project.panels[p].id.clone()),
            Selection::Circuit(p, c) => {
                parts.push(self.project.panels[p].id.clone());
                parts.push(self.project.panels[p].circuits[c].id.clone());
            }
        }
        parts
    }

    pub fn add_circuit(&mut self, panel: usize) {
        let circuits = &mut self.project.panels[panel].circuits;
        let id = (1..)
            .map(|n| format!("C-{n:02}"))
            .find(|id| circuits.iter().all(|c| &c.id != id))
            .unwrap_or_default();
        circuits.push(Circuit {
            id,
            name: "Circuito nuevo".to_string(),
            kind: CircuitKind::LineToNeutral,
            phases: vec![Phase::A],
            load: Load {
                power_w: 1000.0,
                power_factor: 0.9,
                continuous: false,
                kind: LoadKind::General,
            },
            length_m: 10.0,
            has_neutral: false,
            overrides: Overrides::default(),
        });
        self.selection = Selection::Circuit(panel, circuits.len() - 1);
        self.edited();
    }

    pub fn duplicate_circuit(&mut self) {
        let Selection::Circuit(p, c) = self.selection else {
            return;
        };
        let circuits = &mut self.project.panels[p].circuits;
        let mut copy = circuits[c].clone();
        copy.id = (1..)
            .map(|n| format!("C-{n:02}"))
            .find(|id| circuits.iter().all(|x| &x.id != id))
            .unwrap_or_default();
        circuits.insert(c + 1, copy);
        self.selection = Selection::Circuit(p, c + 1);
        self.edited();
    }

    pub fn remove_selected(&mut self) {
        match self.selection {
            Selection::Circuit(p, c) => {
                self.project.panels[p].circuits.remove(c);
                let n = self.project.panels[p].circuits.len();
                self.selection = if n == 0 {
                    Selection::Feeder(p)
                } else {
                    Selection::Circuit(p, c.min(n - 1))
                };
            }
            Selection::Feeder(p) => {
                // Los subtableros pasan a colgar del tablero padre del que se quita.
                let removed = self.project.panels.remove(p);
                for panel in &mut self.project.panels {
                    if panel.parent.as_deref() == Some(removed.id.as_str()) {
                        panel.parent = removed.parent.clone();
                    }
                }
                self.selection = Selection::Project;
            }
            Selection::Project => return,
        }
        self.edited();
    }

    /// Agrega un tablero; si hay uno seleccionado, como subtablero suyo.
    pub fn add_panel(&mut self) {
        let parent = self
            .panel_index()
            .filter(|_| self.selection != Selection::Project)
            .map(|p| self.project.panels[p].id.clone());
        let id = (1..)
            .map(|n| format!("T-{n}"))
            .find(|id| self.project.panels.iter().all(|p| &p.id != id))
            .unwrap_or_default();
        let name = format!("Tablero {id}");
        self.project.panels.push(new_panel(&id, &name, parent));
        self.selection = Selection::Feeder(self.project.panels.len() - 1);
        self.edited();
    }

    /// Mueve la selección a la fila anterior o siguiente del cuadro de cargas.
    pub fn step_selection(&mut self, delta: i32) {
        let Some(p) = self.panel_index() else {
            return;
        };
        let n = self.project.panels[p].circuits.len() as i32;
        // Fila 0 es el alimentador; las siguientes, los circuitos.
        let row = match self.selection {
            Selection::Circuit(_, c) => c as i32 + 1,
            _ => 0,
        };
        let next = (row + delta).clamp(0, n);
        self.selection = if next == 0 {
            Selection::Feeder(p)
        } else {
            Selection::Circuit(p, next as usize - 1)
        };
    }

    /// Motivo por el que no se pudo calcular el proyecto, como oración.
    pub fn calc_error(&self) -> Option<String> {
        let e = self.result.as_ref().err()?;
        let mut chars = e.chars();
        Some(match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => "No se pudo calcular el proyecto.".to_string(),
        })
    }

    /// Después de cambiar el ID de un tablero, sus subtableros lo siguen.
    /// No se propaga si el ID anterior estaba repetido: no se sabría de cuál
    /// de los dos colgaban.
    pub fn panel_renamed(&mut self, index: usize, old: &str) {
        let new = self.project.panels[index].id.clone();
        let others_with_old = self
            .project
            .panels
            .iter()
            .enumerate()
            .any(|(k, p)| k != index && p.id == old);
        if new == old || others_with_old {
            return;
        }
        for panel in &mut self.project.panels {
            if panel.parent.as_deref() == Some(old) {
                panel.parent = Some(new.clone());
            }
        }
    }

    /// Tableros de los que puede alimentarse el tablero `index`: todos menos
    /// él mismo y los que cuelgan de él, para que no se formen ciclos.
    pub fn feeder_candidates(&self, index: usize) -> Vec<(String, String)> {
        let panels = &self.project.panels;
        let mut excluded = vec![panels[index].id.clone()];
        // Recorre hacia abajo hasta que no aparezcan descendientes nuevos.
        loop {
            let before = excluded.len();
            for p in panels {
                if let Some(parent) = &p.parent {
                    if excluded.contains(parent) && !excluded.contains(&p.id) {
                        excluded.push(p.id.clone());
                    }
                }
            }
            if excluded.len() == before {
                break;
            }
        }
        panels
            .iter()
            .filter(|p| !excluded.contains(&p.id))
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect()
    }

    /// Aviso si el ID del tablero `index` está vacío o repetido.
    pub fn panel_id_problem(&self, index: usize) -> Option<&'static str> {
        let id = self.project.panels[index].id.trim();
        if id.is_empty() {
            return Some("El tablero necesita un ID.");
        }
        let repeated = self
            .project
            .panels
            .iter()
            .enumerate()
            .any(|(k, p)| k != index && p.id.trim() == id);
        repeated.then_some("Ya hay otro tablero con este ID.")
    }

    /// Aviso si el ID del circuito está vacío o repetido en su tablero.
    pub fn circuit_id_problem(&self, panel: usize, index: usize) -> Option<&'static str> {
        let circuits = &self.project.panels[panel].circuits;
        let id = circuits[index].id.trim();
        if id.is_empty() {
            return Some("El circuito necesita un ID.");
        }
        let repeated = circuits
            .iter()
            .enumerate()
            .any(|(k, c)| k != index && c.id.trim() == id);
        repeated.then_some("Ya hay otro circuito con este ID en el tablero.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_levels() -> Doc {
        // TG ← T-1 ← T-2
        let mut doc = Doc::new(empty_project(), None);
        doc.selection = Selection::Feeder(0);
        doc.add_panel();
        doc.add_panel();
        doc
    }

    #[test]
    fn subpanels_follow_a_renamed_panel() {
        let mut doc = three_levels();
        let old = doc.project.panels[1].id.clone();
        doc.project.panels[1].id = "TS-A".to_string();
        doc.panel_renamed(1, &old);
        assert_eq!(doc.project.panels[2].parent.as_deref(), Some("TS-A"));
        doc.recalculate();
        assert!(doc.result.is_ok(), "{:?}", doc.result.err());
    }

    #[test]
    fn a_panel_cannot_be_fed_from_itself_or_below() {
        let doc = three_levels();
        let ids = |i: usize| -> Vec<String> {
            doc.feeder_candidates(i)
                .into_iter()
                .map(|(id, _)| id)
                .collect()
        };
        let top = doc.project.panels[0].id.clone();
        assert!(
            ids(0).is_empty(),
            "el principal no puede colgar de sus subtableros"
        );
        assert_eq!(ids(1), vec![top.clone()]);
        assert_eq!(ids(2), vec![top, doc.project.panels[1].id.clone()]);
    }

    #[test]
    fn repeated_ids_are_reported() {
        let mut doc = three_levels();
        assert_eq!(doc.panel_id_problem(1), None);
        doc.project.panels[1].id = doc.project.panels[0].id.clone();
        assert!(doc.panel_id_problem(1).is_some());
        doc.project.panels[1].id = "  ".to_string();
        assert!(doc.panel_id_problem(1).is_some());
    }
}
