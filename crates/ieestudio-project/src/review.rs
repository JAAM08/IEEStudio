//! Revisiones del modelo que no son de la norma pero evitan errores de
//! captura: una carga que parece motor y se capturó como otra carga, y un
//! transformador intermedio que parece el de la acometida capturado otra vez.
//! Se informan en el resultado del proyecto para que la revisión general, la
//! memoria y la línea de comandos las vean, no solo un paso de la interfaz.

use serde::Serialize;

use crate::model::{LoadKind, Project};

/// Palabras que delatan un motor o un equipo de aire acondicionado en el
/// nombre de un circuito, sin acentos y en minúsculas.
const MOTOR_WORDS: [&str; 13] = [
    "motor",
    "bomba",
    "compresor de aire",
    "ventilador",
    "extractor",
    "elevador",
    "montacargas",
    "malacate",
    "polipasto",
    "transportador",
    "banda transportadora",
    "molino",
    "hidroneumatico",
];
const AIR_WORDS: [&str; 12] = [
    "aire acondicionado",
    "minisplit",
    "mini split",
    "mini-split",
    "clima",
    "a/a",
    "refrigera",
    "camara fria",
    "chiller",
    "condensadora",
    "enfriador",
    "manejadora",
];

fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|ch| match ch {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

/// Tipo de carga que sugiere el nombre de un circuito. Un compresor de
/// refrigeración es aire acondicionado; uno de aire, motor.
pub fn kind_from_name(name: &str) -> Option<LoadKind> {
    let name = fold(name);
    if AIR_WORDS.iter().any(|w| name.contains(w)) {
        Some(LoadKind::AirConditioning)
    } else if MOTOR_WORDS.iter().any(|w| name.contains(w)) {
        Some(LoadKind::Motor)
    } else {
        None
    }
}

/// Un circuito de «Otra carga» cuyo nombre parece de un motor o de un aire
/// acondicionado. Si lo es, su protección se calcula con otras reglas y su
/// aportación a la falla falta.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KindHint {
    /// Índices del tablero y del circuito en el proyecto.
    pub panel: usize,
    pub circuit: usize,
    /// `TG-1 · C-06 Bomba de agua`.
    pub label: String,
    pub suggested: LoadKind,
}

impl KindHint {
    pub fn text(&self) -> String {
        format!(
            "{} se capturó como «Otra carga» pero su nombre parece de {}: si lo es, cámbialo de tipo para calcular su protección y su aportación a la falla",
            self.label,
            match self.suggested {
                LoadKind::AirConditioning => "un equipo de aire acondicionado",
                _ => "un motor",
            }
        )
    }
}

pub fn kind_hints(project: &Project) -> Vec<KindHint> {
    let mut out = Vec::new();
    for (i, panel) in project.panels.iter().enumerate() {
        for (c, circuit) in panel.circuits.iter().enumerate() {
            if circuit.load.kind != LoadKind::General {
                continue;
            }
            if let Some(suggested) = kind_from_name(&circuit.name) {
                out.push(KindHint {
                    panel: i,
                    circuit: c,
                    label: format!("{} · {} {}", panel.id, circuit.id, circuit.name),
                    suggested,
                });
            }
        }
    }
    out
}

/// Transformadores intermedios que parecen una segunda captura del
/// transformador de la acometida: el secundario con la misma tensión que el
/// primario, o el primero de la instalación con la misma potencia que el de
/// la acometida. No se rechazan (un transformador de aislamiento existe),
/// pero se avisa.
pub fn transformer_warnings(project: &Project) -> Vec<String> {
    let mut out = Vec::new();
    for (i, panel) in project.panels.iter().enumerate() {
        let Some(t) = &panel.transformer else {
            continue;
        };
        let primary = crate::connection::feeder_system(project, i);
        let same_voltage = primary.line_to_line_v == t.secondary.line_to_line_v
            && primary.line_to_neutral_v == t.secondary.line_to_neutral_v;
        if same_voltage {
            out.push(format!(
                "El transformador intermedio de {} tiene la misma tensión en el primario y en el secundario. Elige la tensión de su secundario; si es el transformador de CFE o la subestación que alimenta toda la instalación, quítalo del tablero y captúralo una sola vez en la acometida del proyecto",
                panel.id
            ));
            continue;
        }
        let from_source = panel.parent.is_none();
        let same_rating = project
            .source
            .substation
            .as_ref()
            .and_then(|s| s.kva)
            .zip(t.kva)
            .is_some_and(|(a, b)| a == b);
        if from_source && same_rating {
            out.push(format!(
                "El transformador intermedio de {} tiene la misma potencia que el de la acometida y sale del punto de conexión. Si es el mismo equipo, quítalo del tablero: ya está capturado en la acometida",
                panel.id
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_suggest_the_load_kind() {
        assert_eq!(kind_from_name("Bomba de agua"), Some(LoadKind::Motor));
        assert_eq!(kind_from_name("EXTRACTOR baño"), Some(LoadKind::Motor));
        assert_eq!(kind_from_name("Hidroneumático"), Some(LoadKind::Motor));
        assert_eq!(
            kind_from_name("Minisplit recámara"),
            Some(LoadKind::AirConditioning)
        );
        assert_eq!(
            kind_from_name("Compresor de refrigeración"),
            Some(LoadKind::AirConditioning)
        );
        assert_eq!(
            kind_from_name("Cámara fría"),
            Some(LoadKind::AirConditioning)
        );
        assert_eq!(kind_from_name("Compresor de aire"), Some(LoadKind::Motor));
        assert_eq!(kind_from_name("Contactos sala"), None);
    }

    #[test]
    fn only_general_loads_are_questioned() {
        let mut project = crate::example::house();
        assert!(kind_hints(&project).is_empty());
        let c = project.panels[0]
            .circuits
            .iter()
            .position(|x| x.load.kind == LoadKind::Motor)
            .unwrap();
        project.panels[0].circuits[c].load.kind = LoadKind::General;
        let hints = kind_hints(&project);
        assert_eq!(hints.len(), 1);
        assert_eq!((hints[0].panel, hints[0].circuit), (0, c));
        assert_eq!(hints[0].suggested, LoadKind::Motor);
    }
}
