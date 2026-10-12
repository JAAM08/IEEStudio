//! Paneles acoplables y espacios de trabajo.
//!
//! Cada panel es una pestaña que se puede arrastrar, dividir o cerrar. Un
//! espacio de trabajo es un acomodo de paneles pensado para una tarea; el que
//! el usuario deje al cerrar se guarda.

use egui_dock::{DockState, NodeIndex};
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
