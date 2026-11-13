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
    /// El proyecto revisa la capacidad interruptiva: la carga sin confirmar
    /// la deja no concluyente. Con el cálculo de cortocircuito apagado solo
    /// importa para su protección.
    pub affects_fault: bool,
}

impl KindHint {
    pub fn text(&self) -> String {
        let what = match self.suggested {
            LoadKind::AirConditioning => "un equipo de aire acondicionado",
            _ => "un motor",
        };
        if self.affects_fault {
            format!(
                "{} se capturó como «Otra carga» pero su nombre parece de {what}: si lo es, cámbialo de tipo para calcular su protección y su aportación a la falla; si no, confirma que es otra carga. Mientras no se resuelva, la capacidad interruptiva de su zona queda no concluyente",
                self.label
            )
        } else {
            format!(
                "{} se capturó como «Otra carga» pero su nombre parece de {what}: si lo es, cámbialo de tipo para calcular su protección; si no, confirma que es otra carga",
                self.label
            )
        }
    }
}

/// «Otras cargas» con nombre de motor o de aire acondicionado que no se han
/// confirmado (decisión 0029, D-3).
pub fn kind_hints(project: &Project) -> Vec<KindHint> {
    let mut out = Vec::new();
    for (i, panel) in project.panels.iter().enumerate() {
        for (c, circuit) in panel.circuits.iter().enumerate() {
            if circuit.load.kind != LoadKind::General || circuit.load.confirmed_general {
                continue;
            }
            if let Some(suggested) = kind_from_name(&circuit.name) {
                out.push(KindHint {
                    panel: i,
                    circuit: c,
                    label: format!("{} · {} {}", panel.id, circuit.id, circuit.name),
                    suggested,
                    affects_fault: project.settings.fault_study,
                });
            }
        }
    }
    out
}

/// «Otras cargas» con nombre de motor o de aire acondicionado que el usuario
/// confirmó que no lo son, para la trazabilidad de la memoria y la línea de
/// comandos: `TG-1 · C-06 Bomba de agua`.
pub fn confirmed_general(project: &Project) -> Vec<String> {
    let mut out = Vec::new();
    for panel in &project.panels {
        for circuit in &panel.circuits {
            if circuit.load.kind == LoadKind::General
                && circuit.load.confirmed_general
                && kind_from_name(&circuit.name).is_some()
            {
                out.push(format!("{} · {} {}", panel.id, circuit.id, circuit.name));
            }
        }
    }
    out
}

/// Un transformador intermedio que puede ser el de la acometida capturado
/// otra vez. Es una advertencia: no se bloquea nada y el usuario puede
/// confirmar que son equipos distintos (`Transformer::confirmed_distinct`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DuplicateFlag {
    /// Índice del tablero en el proyecto.
    pub panel: usize,
    pub panel_id: String,
    /// Por qué parece el mismo equipo, y qué hacer.
    pub reason: String,
}

/// Transformadores intermedios que parecen una segunda captura del de la
/// acometida, según su lugar en la instalación y sus datos:
///
/// - **Relación 1:1**: el secundario tiene la tensión de su primario. Puede
///   ser legítimo (aislamiento), así que solo se avisa.
/// - **Junto a la acometida y con su misma potencia**: sale directo del
///   punto de conexión, la acometida tiene transformador y los dos tienen la
///   misma potencia. Dos transformadores de igual potencia en otros lugares
///   de la instalación no se marcan.
pub fn duplicate_flags(project: &Project) -> Vec<DuplicateFlag> {
    let mut out = Vec::new();
    for (i, panel) in project.panels.iter().enumerate() {
        let Some(t) = &panel.transformer else {
            continue;
        };
        if t.confirmed_distinct {
            continue;
        }
        let primary = crate::connection::feeder_system(project, i);
        let one_to_one = primary.line_to_line_v == t.secondary.line_to_line_v
            && primary.line_to_neutral_v == t.secondary.line_to_neutral_v;
        let service = project.source.substation.as_ref();
        let beside_service = panel.parent.is_none() && service.is_some();
        let same_rating = service
            .and_then(|s| s.kva)
            .zip(t.kva)
            .is_some_and(|(a, b)| a == b);
        let reason = if one_to_one && beside_service {
            "tiene relación 1:1 y sale directo de la acometida, que ya tiene transformador: parece el mismo equipo capturado dos veces. Si lo es, quítalo del tablero; si es un transformador de aislamiento, confirma que es otro equipo".to_string()
        } else if one_to_one {
            "tiene la misma tensión en el primario y en el secundario (relación 1:1). Si es un transformador de aislamiento, confirma que es otro equipo; si no, elige la tensión de su secundario".to_string()
        } else if beside_service && same_rating {
            "sale directo de la acometida y tiene la misma potencia que su transformador: puede ser el mismo equipo capturado dos veces. Si lo es, quítalo del tablero; si no, confirma que es otro equipo".to_string()
        } else {
            continue;
        };
        out.push(DuplicateFlag {
            panel: i,
            panel_id: panel.id.clone(),
            reason,
        });
    }
    out
}

impl DuplicateFlag {
    pub fn text(&self) -> String {
        format!(
            "El transformador intermedio de {} {}",
            self.panel_id, self.reason
        )
    }
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
