//! Las vistas de la aplicación y los paneles que se dibujan dentro de ellas.
//!
//! IEEStudio tiene un solo marco: una barra superior con cinco vistas
//! (Proyecto, Tableros y circuitos, Unifilar, Revisar y Memoria), un
//! navegador y un inspector fijos a los lados, un panel de procedimiento
//! abajo y la barra de estado. No hay modos ni paneles acoplables.

use egui_phosphor::regular as icon;
use serde::{Deserialize, Serialize};

/// Cada una de las cinco vistas de la barra superior, en el orden de la ruta
/// de trabajo: de los datos de la obra a la memoria que se entrega.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum View {
    Project,
    #[default]
    Schedule,
    Diagram,
    Review,
    Report,
}

impl View {
    pub const ALL: [View; 5] = [
        View::Project,
        View::Schedule,
        View::Diagram,
        View::Review,
        View::Report,
    ];

    pub fn title(self) -> &'static str {
        match self {
            View::Project => "Proyecto",
            View::Schedule => "Tableros y circuitos",
            View::Diagram => "Unifilar",
            View::Review => "Revisar",
            View::Report => "Memoria",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            View::Project => icon::FOLDER_OPEN,
            View::Schedule => icon::TABLE,
            View::Diagram => icon::FLOW_ARROW,
            View::Review => icon::LIST_CHECKS,
            View::Report => icon::FILE_PDF,
        }
    }

    /// Atajo de teclado para ir a la vista.
    pub fn shortcut(self) -> &'static str {
        match self {
            View::Project => "F2",
            View::Schedule => "F3",
            View::Diagram => "F4",
            View::Review => "F5",
            View::Report => "F6",
        }
    }

    fn index(self) -> usize {
        View::ALL.iter().position(|v| *v == self).unwrap_or(0)
    }

    /// La vista que sigue en la ruta de trabajo; la barra de estado la ofrece
    /// como «Siguiente», sin obligar el orden.
    pub fn next(self) -> Option<View> {
        View::ALL.get(self.index() + 1).copied()
    }

    /// Si la vista muestra el navegador, el inspector y el procedimiento
    /// alrededor del contenido.
    pub fn has_regions(self) -> bool {
        matches!(self, View::Schedule | View::Diagram)
    }
}

/// Los paneles que se dibujan en las regiones del marco.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    Browser,
    Schedule,
    Diagram,
    Properties,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_route_goes_from_the_project_to_the_report() {
        assert_eq!(View::ALL[0], View::Project);
        assert_eq!(View::Project.next(), Some(View::Schedule));
        assert_eq!(View::Review.next(), Some(View::Report));
        assert_eq!(View::Report.next(), None);
    }

    #[test]
    fn every_view_has_its_own_title_and_shortcut() {
        for (i, a) in View::ALL.iter().enumerate() {
            for b in &View::ALL[i + 1..] {
                assert_ne!(a.title(), b.title());
                assert_ne!(a.shortcut(), b.shortcut());
            }
        }
    }

    #[test]
    fn only_the_working_views_have_regions() {
        assert!(View::Schedule.has_regions());
        assert!(View::Diagram.has_regions());
        assert!(!View::Project.has_regions());
        assert!(!View::Review.has_regions());
        assert!(!View::Report.has_regions());
    }

    // Las preferencias guardadas por versiones anteriores no traen la vista;
    // la de omisión es la de trabajo.
    #[test]
    fn the_default_view_is_the_working_one() {
        assert_eq!(View::default(), View::Schedule);
    }
}
