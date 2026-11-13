//! El proyecto abierto: la única fuente de verdad de todas las vistas.

use std::path::PathBuf;

use ieestudio_core::{CircuitKind, System};
use ieestudio_project::{
    calculate, Circuit, CompressorLoad, Feeder, Load, LoadKind, MotorLoad, Overrides, Panel,
    PanelResult, Phase, Project, ProjectResult, RunResult, Settings, Source, FORMAT_VERSION,
};

/// Lo que está seleccionado. Todas las vistas leen y cambian esta selección.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Project,
    /// El tablero y su alimentador.
    Feeder(usize),
    /// Los conductores del secundario del transformador de un tablero.
    Secondary(usize),
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
    /// Estados anteriores para deshacer, el más reciente al final.
    undo: Vec<Project>,
    /// Estados deshechos para rehacer.
    redo: Vec<Project>,
    /// Último estado registrado en el historial.
    committed: Project,
}

/// Pasos que guarda el historial.
const HISTORY: usize = 100;

pub fn empty_project() -> Project {
    Project {
        format_version: FORMAT_VERSION,
        name: "Proyecto nuevo".to_string(),
        profile: "nom-001-sede-2012".to_string(),
        info: Default::default(),
        occupancy: None,
        settings: Settings::default(),
        source: Source::new("Acometida", System::WYE_220_127),
        panels: vec![new_panel("TG-1", "Tablero general", None)],
    }
}

/// Cambia el tipo de carga y crea los datos que el tipo nuevo pide: la
/// potencia del motor o la placa del equipo de aire acondicionado. Los datos
/// del tipo anterior se conservan por si se regresa a él.
pub fn set_load_kind(load: &mut Load, kind: LoadKind) {
    load.kind = kind;
    match kind {
        // La potencia empieza sin elegir: un motor sin ella queda sin
        // calcular y se avisa que su aportación a la falla falta.
        LoadKind::Motor if load.motor.is_none() => {
            load.motor = Some(MotorLoad {
                hp: String::new(),
                motor_type: Default::default(),
                nameplate_a: None,
                service_factor_marked: false,
            });
        }
        LoadKind::AirConditioning if load.compressor.is_none() => {
            load.compressor = Some(CompressorLoad::default());
        }
        _ => {}
    }
}

fn new_panel(id: &str, name: &str, parent: Option<String>) -> Panel {
    Panel {
        board: Default::default(),
        interrupting_ka: Default::default(),
        id: id.to_string(),
        name: name.to_string(),
        parent,
        kind: CircuitKind::ThreePhase,
        feeder: Feeder {
            interrupting_ka: Default::default(),
            // Nace sin longitud: nunca se supone (pendiente 19).
            length_m: None,
            demand: Default::default(),
            demand_factor: None,
            has_neutral: true,
            overrides: Overrides::default(),
        },
        transformer: None,
        areas: vec![],
        circuits: vec![],
    }
}

impl Doc {
    pub fn new(project: Project, path: Option<PathBuf>) -> Self {
        let mut doc = Doc {
            committed: project.clone(),
            project,
            path,
            dirty: false,
            result: Err(String::new()),
            selection: Selection::Project,
            revision: 0,
            undo: Vec::new(),
            redo: Vec::new(),
        };
        doc.recalculate();
        if !doc.project.panels.is_empty() {
            doc.selection = Selection::Feeder(0);
        }
        doc
    }

    pub fn recalculate(&mut self) {
        self.result = calculate(&self.project);
        // Único entre documentos: lo que se guarda por revisión (la vista
        // previa, el unifilar) no debe confundir un proyecto con otro.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        self.revision = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.clamp_selection();
    }

    /// Registra en el historial el cambio pendiente, si lo hay. Se llama
    /// cuando la interacción terminó (sin botón presionado ni campo con el
    /// cursor), para que arrastrar un número o escribir un nombre sea un solo
    /// paso.
    pub fn commit(&mut self) {
        if self.project != self.committed {
            let previous = std::mem::replace(&mut self.committed, self.project.clone());
            self.undo.push(previous);
            if self.undo.len() > HISTORY {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.project != self.committed
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        self.commit();
        if let Some(previous) = self.undo.pop() {
            self.redo
                .push(std::mem::replace(&mut self.project, previous));
            self.committed = self.project.clone();
            self.edited();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.project, next));
            self.committed = self.project.clone();
            self.edited();
        }
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
            Selection::Secondary(p) if p < panels.len() && panels[p].transformer.is_some() => {
                Selection::Secondary(p)
            }
            Selection::Circuit(p, _) | Selection::Feeder(p) | Selection::Secondary(p)
                if p < panels.len() =>
            {
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
            Selection::Feeder(p) | Selection::Secondary(p) | Selection::Circuit(p, _) => Some(p),
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
            Selection::Secondary(p) => self.panel_result(p)?.secondary.as_ref(),
            Selection::Circuit(p, c) => self.panel_result(p)?.circuits.get(c),
            Selection::Project => None,
        }
    }

    /// Decisiones de diseño del tramo seleccionado, para cambiarlas.
    pub fn selected_overrides_mut(&mut self) -> Option<&mut Overrides> {
        match self.selection {
            Selection::Feeder(p) => Some(&mut self.project.panels[p].feeder.overrides),
            Selection::Secondary(p) => self.project.panels[p]
                .transformer
                .as_mut()
                .map(|t| &mut t.secondary_overrides),
            Selection::Circuit(p, c) => Some(&mut self.project.panels[p].circuits[c].overrides),
            Selection::Project => None,
        }
    }

    /// Sistema de las barras de un tablero: el de la fuente o el del
    /// secundario del transformador más cercano aguas arriba.
    pub fn bus_system(&self, panel: usize) -> System {
        ieestudio_project::connection::bus_system(&self.project, panel)
    }

    /// Clave estable de la selección, para recordar qué grupos se abrieron.
    pub fn selection_key(&self) -> (u8, usize, usize) {
        match self.selection {
            Selection::Project => (0, 0, 0),
            Selection::Feeder(p) => (1, p, 0),
            Selection::Secondary(p) => (3, p, 0),
            Selection::Circuit(p, c) => (2, p, c),
        }
    }

    /// Texto de la selección para la barra de estado: `Proyecto › TG-1 › C-02`.
    pub fn breadcrumb(&self) -> Vec<String> {
        let mut parts = vec![self.project.name.clone()];
        match self.selection {
            Selection::Project => {}
            Selection::Feeder(p) => parts.push(self.project.panels[p].id.clone()),
            Selection::Secondary(p) => {
                parts.push(self.project.panels[p].id.clone());
                parts.push("Secundario".into());
            }
            Selection::Circuit(p, c) => {
                parts.push(self.project.panels[p].id.clone());
                parts.push(self.project.panels[p].circuits[c].id.clone());
            }
        }
        parts
    }

    /// Reparte las fases de los circuitos del tablero para balancearlo.
    /// Dice el desbalance antes y después, o por qué no se balanceó: sin la
    /// carga de todos los circuitos no se reparte nada (pendiente 19).
    pub fn balance_phases(&mut self, panel: usize) -> String {
        let Some(result) = self.panel_result(panel).cloned() else {
            return "No se balanceó: el tablero no se pudo calcular".to_string();
        };
        let p = &self.project.panels[panel];
        let missing: Vec<&str> = p
            .circuits
            .iter()
            .zip(&result.circuits)
            .filter(|(_, r)| r.load_w.is_none())
            .map(|(c, _)| c.id.as_str())
            .collect();
        if !missing.is_empty() {
            return format!(
                "No se balanceó: falta la potencia de {}",
                missing.join(", ")
            );
        }
        let before = result.imbalance_percent.unwrap_or(100.0);
        let va: Vec<f64> = p
            .circuits
            .iter()
            .zip(&result.circuits)
            .map(|(c, r)| {
                if c.load.power_factor > 0.0 {
                    r.load_w.unwrap_or(0.0) / c.load.power_factor
                } else {
                    0.0
                }
            })
            .collect();
        let phases = ieestudio_project::balance::balance(p, &va);
        for (c, ph) in self.project.panels[panel].circuits.iter_mut().zip(phases) {
            c.phases = ph;
        }
        self.edited();
        match self.panel_result(panel).and_then(|r| r.imbalance_percent) {
            Some(after) => {
                format!("Fases balanceadas: desbalance de {before:.1} % a {after:.1} %")
            }
            None => "Fases asignadas; el tablero no se pudo calcular".to_string(),
        }
    }

    /// Primer identificador `C-NN` libre en un tablero.
    pub fn free_circuit_id(&self, panel: usize) -> String {
        let circuits = &self.project.panels[panel].circuits;
        (1..)
            .map(|n| format!("C-{n:02}"))
            .find(|id| circuits.iter().all(|c| &c.id != id))
            .unwrap_or_default()
    }

    /// Circuito de carga general; la interfaz pregunta antes el tipo.
    #[cfg(test)]
    pub fn add_circuit(&mut self, panel: usize) {
        self.add_circuit_of(panel, LoadKind::General);
    }

    /// Agrega un circuito del tipo de carga elegido, con los datos que ese
    /// tipo necesita y en las fases menos cargadas del tablero.
    pub fn add_circuit_of(&mut self, panel: usize, kind: LoadKind) {
        let id = self.free_circuit_id(panel);
        // El aire acondicionado suele ir entre fases cuando el tablero las tiene.
        let two_phases = ieestudio_project::connection::circuit_kinds(&self.project, panel)
            .contains(&CircuitKind::LineToLine);
        let allowed = ieestudio_project::connection::circuit_kinds(&self.project, panel);
        let circuit_kind = match kind {
            LoadKind::AirConditioning if two_phases => CircuitKind::LineToLine,
            // En un sistema sin neutro no hay circuitos de una fase.
            _ => allowed
                .first()
                .copied()
                .unwrap_or(CircuitKind::LineToNeutral),
        };
        let phases = self.default_phases(panel, circuit_kind);
        let name = match kind {
            LoadKind::General => "Circuito nuevo",
            other => other.label(),
        };
        let mut load = Load {
            // Nace sin potencia: nunca se supone (pendiente 19).
            power_w: None,
            power_factor: 0.9,
            continuous: false,
            kind: LoadKind::General,
            outlets: None,
            motor: None,
            compressor: None,
            confirmed_general: false,
        };
        set_load_kind(&mut load, kind);
        let circuits = &mut self.project.panels[panel].circuits;
        circuits.push(Circuit {
            id,
            name: name.to_string(),
            kind: circuit_kind,
            phases,
            load,
            length_m: None,
            has_neutral: false,
            overrides: Overrides::default(),
        });
        self.selection = Selection::Circuit(panel, circuits.len() - 1);
        self.edited();
    }

    /// Fases para un circuito nuevo del tipo de conexión dado: las menos
    /// cargadas de las que tiene el tablero, en orden A, B, C.
    pub fn default_phases(&self, panel: usize, kind: CircuitKind) -> Vec<Phase> {
        // Las fases de las barras: con transformador, las de su secundario.
        let bars = ieestudio_project::connection::bars_kind(&self.project, panel)
            .unwrap_or(self.project.panels[panel].kind);
        let available: &[Phase] = match bars {
            CircuitKind::LineToNeutral => &[Phase::A],
            CircuitKind::LineToLine => &[Phase::A, Phase::B],
            CircuitKind::ThreePhase => &[Phase::A, Phase::B, Phase::C],
        };
        let needed = match kind {
            CircuitKind::LineToNeutral => 1,
            CircuitKind::LineToLine => 2,
            CircuitKind::ThreePhase => 3,
        };
        let load = self.panel_result(panel).map_or([0.0; 3], |r| r.phase_va);
        let index = |ph: Phase| match ph {
            Phase::A => 0,
            Phase::B => 1,
            Phase::C => 2,
        };
        let mut by_load = available.to_vec();
        // Orden estable: con la misma carga se queda el orden A, B, C.
        by_load.sort_by(|a, b| load[index(*a)].total_cmp(&load[index(*b)]));
        let mut chosen: Vec<Phase> = by_load.into_iter().take(needed).collect();
        chosen.sort_by_key(|ph| index(*ph));
        chosen
    }

    /// Agrega un subtablero alimentado desde el tablero `panel`. Es el mismo
    /// tablero que se agrega en el paso Tableros: no hay una copia aparte.
    pub fn add_subpanel(&mut self, panel: usize) -> usize {
        self.selection = Selection::Feeder(panel);
        self.add_panel();
        self.project.panels.len() - 1
    }

    /// Tableros que se alimentan del tablero `panel`.
    pub fn subpanels(&self, panel: usize) -> Vec<usize> {
        let id = &self.project.panels[panel].id;
        (0..self.project.panels.len())
            .filter(|&k| self.project.panels[k].parent.as_ref() == Some(id))
            .collect()
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
            Selection::Secondary(p) => {
                self.project.panels[p].transformer = None;
                self.selection = Selection::Feeder(p);
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
        // La conexión sale del sistema y del tablero de donde se alimenta:
        // la de más fases que existe ahí (decisión 0024).
        let index = self.project.panels.len() - 1;
        if let Some(kind) = ieestudio_project::connection::feeder_kinds(&self.project, index)
            .into_iter()
            .max_by_key(|k| k.poles())
        {
            self.project.panels[index].kind = kind;
        }
        self.selection = Selection::Feeder(index);
        self.edited();
    }

    /// Mueve la selección a la fila anterior o siguiente del cuadro de cargas.
    pub fn step_selection(&mut self, delta: i32) {
        let Some(p) = self.panel_index() else {
            return;
        };
        // Filas: el alimentador, el secundario si hay transformador y los
        // circuitos.
        let mut rows = vec![Selection::Feeder(p)];
        if self.project.panels[p].transformer.is_some() {
            rows.push(Selection::Secondary(p));
        }
        rows.extend((0..self.project.panels[p].circuits.len()).map(|c| Selection::Circuit(p, c)));
        let row = rows.iter().position(|s| *s == self.selection).unwrap_or(0) as i32;
        let next = (row + delta).clamp(0, rows.len() as i32 - 1);
        self.selection = rows[next as usize];
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

    #[test]
    fn undo_and_redo_whole_steps() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        let original = doc.project.clone();
        assert!(!doc.can_undo());
        // Varios cambios antes de registrar son un solo paso.
        doc.project.name = "A".into();
        doc.project.name = "AB".into();
        doc.commit();
        doc.add_circuit(0);
        doc.commit();
        let with_circuit = doc.project.clone();
        doc.undo();
        assert_eq!(doc.project.name, "AB");
        assert_eq!(
            doc.project.panels[0].circuits.len(),
            original.panels[0].circuits.len()
        );
        doc.undo();
        assert_eq!(doc.project, original);
        assert!(!doc.can_undo());
        doc.redo();
        doc.redo();
        assert_eq!(doc.project, with_circuit);
        assert!(!doc.can_redo());
        // Un cambio nuevo borra lo que se podía rehacer.
        doc.undo();
        doc.project.name = "C".into();
        doc.commit();
        assert!(!doc.can_redo());
        // Un cambio sin registrar también se deshace.
        doc.project.name = "D".into();
        assert!(doc.can_undo());
        doc.undo();
        assert_eq!(doc.project.name, "C");
    }

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
    fn each_load_kind_is_created_with_its_data() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        let n = doc.project.panels[0].circuits.len();
        doc.add_circuit_of(0, LoadKind::Motor);
        doc.add_circuit_of(0, LoadKind::AirConditioning);
        doc.add_circuit_of(0, LoadKind::Lighting);
        let circuits = &doc.project.panels[0].circuits;
        assert_eq!(circuits.len(), n + 3);
        let motor = &circuits[n];
        assert_eq!(motor.load.kind, LoadKind::Motor);
        // Sin potencia supuesta: queda por elegir.
        assert_eq!(motor.load.motor.as_ref().unwrap().hp, "");
        assert_eq!(motor.name, "Motor");
        // El tablero del ejemplo es de tres fases: el aire va entre dos.
        let ac = &circuits[n + 1];
        assert_eq!(ac.load.kind, LoadKind::AirConditioning);
        assert!(ac.load.compressor.is_some());
        assert_eq!(ac.kind, CircuitKind::LineToLine);
        assert_eq!(ac.phases.len(), 2);
        assert_eq!(circuits[n + 2].load.kind, LoadKind::Lighting);
        // El circuito nuevo queda seleccionado y los ID no se repiten.
        assert_eq!(doc.selection, Selection::Circuit(0, n + 2));
        for c in 0..circuits.len() {
            assert_eq!(doc.circuit_id_problem(0, c), None);
        }
        doc.recalculate();
        assert!(doc.result.is_ok(), "{:?}", doc.result.err());
    }

    /// Pendiente 19: lo nuevo nace sin potencia ni longitud y queda «Sin
    /// evaluar» con lo que falta; nunca con valores supuestos.
    #[test]
    fn new_circuits_and_feeders_start_blank() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        doc.add_circuit_of(0, LoadKind::General);
        let added = doc.project.panels[0].circuits.last().unwrap();
        assert_eq!(added.load.power_w, None);
        assert_eq!(added.length_m, None);
        let id = added.id.clone();
        let k = doc.add_subpanel(0);
        assert_eq!(doc.project.panels[k].feeder.length_m, None);
        doc.recalculate();
        let result = doc.panel_result(0).unwrap();
        let circuit = result.circuits.iter().find(|c| c.id == id).unwrap();
        assert_eq!(circuit.status, ieestudio_project::Status::NotEvaluated);
        assert!(circuit.checks[0].detail.contains("falta la potencia"));
        assert!(result.load_unknown.is_some());
        // Sin la carga de todos los circuitos no se balancea nada.
        let phases: Vec<_> = doc.project.panels[0]
            .circuits
            .iter()
            .map(|c| c.phases.clone())
            .collect();
        assert!(doc
            .balance_phases(0)
            .starts_with("No se balanceó: falta la potencia de"));
        let after: Vec<_> = doc.project.panels[0]
            .circuits
            .iter()
            .map(|c| c.phases.clone())
            .collect();
        assert_eq!(phases, after);
    }

    /// Pendiente 17: un tablero detrás de un elevador de una fase a 240/120 V
    /// tiene barras de dos fases; un circuito nuevo puede ir entre fases.
    #[test]
    fn a_new_circuit_uses_the_phases_of_the_secondary() {
        let project = ieestudio_project::Project::from_json(include_str!(
            "../../../fixtures/auditoria-pr11/p17-secundario-con-otra-conexion.iees"
        ))
        .unwrap();
        let tf = project.panels.iter().position(|p| p.id == "TF-1").unwrap();
        assert_eq!(project.panels[tf].kind, CircuitKind::LineToNeutral);
        let doc = Doc::new(project, None);
        assert_eq!(
            doc.default_phases(tf, CircuitKind::LineToLine),
            vec![Phase::A, Phase::B]
        );
    }

    #[test]
    fn a_new_circuit_goes_to_the_least_loaded_phase() {
        let mut doc = Doc::new(ieestudio_project::example::house(), None);
        let load = doc.panel_result(0).unwrap().phase_va;
        let lightest = (0..3).min_by(|&a, &b| load[a].total_cmp(&load[b])).unwrap();
        doc.add_circuit_of(0, LoadKind::Lighting);
        let added = doc.project.panels[0].circuits.last().unwrap();
        assert_eq!(added.phases, vec![[Phase::A, Phase::B, Phase::C][lightest]]);
    }

    #[test]
    fn a_single_phase_panel_only_offers_phase_a() {
        let mut doc = Doc::new(empty_project(), None);
        doc.project.panels[0].kind = CircuitKind::LineToNeutral;
        doc.project.source.system = System::SINGLE_127;
        doc.add_circuit_of(0, LoadKind::AirConditioning);
        let added = &doc.project.panels[0].circuits[0];
        assert_eq!(added.kind, CircuitKind::LineToNeutral);
        assert_eq!(added.phases, vec![Phase::A]);
    }

    #[test]
    fn a_subpanel_from_circuits_is_the_same_as_from_panels() {
        // Por los dos caminos sale el mismo tablero, colgado del de origen.
        let mut a = Doc::new(empty_project(), None);
        let k = a.add_subpanel(0);
        let mut b = Doc::new(empty_project(), None);
        b.selection = Selection::Feeder(0);
        b.add_panel();
        assert_eq!(a.project, b.project);
        assert_eq!(a.selection, Selection::Feeder(k));
        assert_eq!(a.project.panels[k].parent.as_deref(), Some("TG-1"));
        assert_eq!(a.subpanels(0), vec![k]);
        assert!(a.subpanels(k).is_empty());
        // Dos subtableros no repiten ID.
        a.add_subpanel(0);
        assert_eq!(a.panel_id_problem(1), None);
        assert_eq!(a.panel_id_problem(2), None);
        assert_eq!(a.subpanels(0).len(), 2);
    }

    /// Un tablero nuevo toma la conexión de más fases que existe donde se
    /// alimenta, no siempre tres fases.
    #[test]
    fn a_new_panel_takes_a_connection_its_system_has() {
        let mut project = ieestudio_project::example::house();
        project.source.system = ieestudio_core::System::SPLIT_240_120;
        let mut doc = Doc::new(project, None);
        doc.selection = Selection::Project;
        doc.add_panel();
        let panel = doc.project.panels.last().unwrap();
        assert_eq!(panel.kind, CircuitKind::LineToLine);
    }

    #[test]
    fn changing_the_load_kind_keeps_the_other_data() {
        let mut load = ieestudio_project::example::house().panels[0].circuits[0]
            .load
            .clone();
        set_load_kind(&mut load, LoadKind::Motor);
        load.motor.as_mut().unwrap().hp = "2".to_string();
        set_load_kind(&mut load, LoadKind::General);
        set_load_kind(&mut load, LoadKind::Motor);
        assert_eq!(load.motor.unwrap().hp, "2");
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
