//! Paneles acoplables y espacios de trabajo.
//!
//! Cada panel es una pestaña que se puede arrastrar, dividir o cerrar. Un
//! espacio de trabajo es un acomodo de paneles pensado para una tarea; el que
//! el usuario deje al cerrar se guarda.

use eframe::egui;
use egui_dock::{DockState, NodeIndex, SurfaceIndex, Tree};
use egui_phosphor::regular as icon;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tab {
    Browser,
    Schedule,
    Diagram,
    Report,
    Procedure,
    Properties,
    Quick,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Browser,
        Tab::Schedule,
        Tab::Diagram,
        Tab::Report,
        Tab::Procedure,
        Tab::Properties,
        Tab::Quick,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Browser => "Proyecto",
            Tab::Schedule => "Cuadro de cargas",
            Tab::Diagram => "Unifilar",
            Tab::Report => "Memoria",
            Tab::Procedure => "Procedimiento",
            Tab::Properties => "Propiedades",
            Tab::Quick => "Cálculo rápido",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Tab::Browser => icon::TREE_STRUCTURE,
            Tab::Schedule => icon::TABLE,
            Tab::Diagram => icon::FLOW_ARROW,
            Tab::Report => icon::FILE_PDF,
            Tab::Procedure => icon::LIST_BULLETS,
            Tab::Properties => icon::SLIDERS,
            Tab::Quick => icon::CALCULATOR,
        }
    }

    /// Atajo del menú Ventana.
    pub fn shortcut(self) -> &'static str {
        match self {
            Tab::Browser => "F2",
            Tab::Schedule => "F3",
            Tab::Diagram => "F4",
            Tab::Report => "F5",
            Tab::Procedure => "F6",
            Tab::Properties => "F7",
            Tab::Quick => "F8",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Workspace {
    /// Captura y revisión del cuadro de cargas.
    Calculation,
    /// Revisión del unifilar.
    Diagram,
    /// Revisión de la memoria antes de entregarla.
    Documentation,
}

impl Workspace {
    pub const ALL: [Workspace; 3] = [
        Workspace::Calculation,
        Workspace::Diagram,
        Workspace::Documentation,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Workspace::Calculation => "Cálculo",
            Workspace::Diagram => "Diagrama",
            Workspace::Documentation => "Documentación",
        }
    }

    /// Acomodo inicial del espacio de trabajo.
    pub fn layout(self) -> DockState<Tab> {
        match self {
            Workspace::Calculation => {
                let mut state = DockState::new(vec![Tab::Schedule, Tab::Diagram, Tab::Report]);
                let tree = state.main_surface_mut();
                let [center, _] = tree.split_left(NodeIndex::root(), 0.16, vec![Tab::Browser]);
                let [center, _] = tree.split_right(center, 0.74, vec![Tab::Properties]);
                tree.split_below(center, 0.64, vec![Tab::Procedure]);
                state
            }
            Workspace::Diagram => {
                let mut state = DockState::new(vec![Tab::Diagram, Tab::Schedule]);
                let tree = state.main_surface_mut();
                let [center, _] = tree.split_left(NodeIndex::root(), 0.16, vec![Tab::Browser]);
                tree.split_right(center, 0.76, vec![Tab::Properties]);
                state
            }
            Workspace::Documentation => {
                let mut state = DockState::new(vec![Tab::Report, Tab::Schedule, Tab::Diagram]);
                let tree = state.main_surface_mut();
                tree.split_left(NodeIndex::root(), 0.16, vec![Tab::Browser]);
                state
            }
        }
    }
}

/// ¿La zona principal se quedó sin paneles? Las ventanas flotantes no cuentan.
pub fn main_is_empty(dock: &DockState<Tab>) -> bool {
    dock.main_surface().num_tabs() == 0
}

/// ¿El panel está abierto, acoplado o en una ventana flotante?
pub fn is_open(dock: &DockState<Tab>, tab: Tab) -> bool {
    dock.find_tab(&tab).is_some()
}

/// Cierra un panel si está abierto.
pub fn close(dock: &mut DockState<Tab>, tab: Tab) {
    if let Some(path) = dock.find_tab(&tab) {
        dock.remove_tab(path);
    }
}

/// Abre un panel donde le corresponde y lo deja activo. Nunca falla, aunque
/// el usuario haya cerrado todos los paneles.
pub fn open(dock: &mut DockState<Tab>, tab: Tab) {
    if !is_open(dock, tab) {
        if main_is_empty(dock) {
            // Sin ningún panel no hay dónde dividir: el primero llena la ventana.
            *dock.main_surface_mut() = Tree::new(vec![tab]);
        } else if tab == Tab::Quick {
            // El cálculo rápido no es del proyecto: abre en ventana flotante.
            let surface = dock.add_window(vec![tab]);
            if let Some(window) = dock.get_window_state_mut(surface) {
                window
                    .set_position(egui::pos2(180.0, 70.0))
                    .set_size(egui::vec2(1120.0, 720.0));
            }
        } else {
            let tree = dock.main_surface_mut();
            match tab {
                Tab::Browser => {
                    tree.split_left(NodeIndex::root(), 0.16, vec![tab]);
                }
                Tab::Properties => {
                    tree.split_right(NodeIndex::root(), 0.76, vec![tab]);
                }
                Tab::Procedure => {
                    tree.split_below(NodeIndex::root(), 0.68, vec![tab]);
                }
                _ => {
                    // Los documentos van junto a los otros documentos.
                    let center = [Tab::Schedule, Tab::Diagram, Tab::Report]
                        .iter()
                        .filter_map(|t| dock.find_tab(t))
                        .find(|path| path.surface == SurfaceIndex::main());
                    match center {
                        Some(path) => {
                            dock.set_focused_node_and_surface(path.node_path());
                            dock.push_to_focused_leaf(tab);
                        }
                        None => dock.push_to_first_leaf(tab),
                    }
                }
            }
        }
    }
    if let Some(path) = dock.find_tab(&tab) {
        let _ = dock.set_active_tab(path);
    }
}

/// Abre el panel si está cerrado; lo cierra si está abierto.
pub fn toggle(dock: &mut DockState<Tab>, tab: Tab) {
    if is_open(dock, tab) {
        close(dock, tab);
    } else {
        open(dock, tab);
    }
}

/// Un acomodo guardado que quedó vacío o dañado se reemplaza por el del
/// espacio de trabajo.
pub fn sanitize(dock: DockState<Tab>, workspace: Workspace) -> DockState<Tab> {
    if main_is_empty(&dock) {
        workspace.layout()
    } else {
        dock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close_all(dock: &mut DockState<Tab>) {
        for tab in Tab::ALL {
            close(dock, tab);
        }
    }

    #[test]
    fn every_workspace_has_its_documents() {
        for ws in Workspace::ALL {
            let dock = ws.layout();
            assert!(is_open(&dock, Tab::Browser), "{ws:?}");
            assert!(is_open(&dock, Tab::Schedule), "{ws:?}");
            assert!(!is_open(&dock, Tab::Quick), "{ws:?}");
        }
    }

    /// Cerrar todos los paneles y volver a abrirlos no debe cerrar el programa.
    #[test]
    fn panels_reopen_after_closing_everything() {
        for ws in Workspace::ALL {
            for first in Tab::ALL {
                let mut dock = ws.layout();
                close_all(&mut dock);
                assert!(main_is_empty(&dock));
                open(&mut dock, first);
                for tab in Tab::ALL {
                    open(&mut dock, tab);
                    assert!(is_open(&dock, tab), "{ws:?}: {first:?} y luego {tab:?}");
                }
            }
        }
    }

    #[test]
    fn toggling_twice_restores_the_state() {
        for tab in Tab::ALL {
            let mut dock = Workspace::Calculation.layout();
            let before = is_open(&dock, tab);
            toggle(&mut dock, tab);
            assert_ne!(is_open(&dock, tab), before);
            toggle(&mut dock, tab);
            assert_eq!(is_open(&dock, tab), before);
        }
    }

    #[test]
    fn an_empty_saved_layout_is_replaced() {
        let mut dock = Workspace::Calculation.layout();
        close_all(&mut dock);
        let fixed = sanitize(dock, Workspace::Diagram);
        assert!(is_open(&fixed, Tab::Diagram));
    }
}
