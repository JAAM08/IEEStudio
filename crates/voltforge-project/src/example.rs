//! Proyecto de ejemplo: una casa habitación con un tablero y cinco circuitos.

use voltforge_core::{CircuitKind, Size, System};

use crate::model::{
    Circuit, Feeder, Load, LoadKind, Overrides, Panel, Phase, Project, Settings, Source,
    FORMAT_VERSION,
};

#[allow(clippy::too_many_arguments)] // constructor interno del ejemplo
fn circuit(
    id: &str,
    name: &str,
    kind: CircuitKind,
    phases: &[Phase],
    power_w: f64,
    continuous: bool,
    load_kind: LoadKind,
    length_m: f64,
) -> Circuit {
    Circuit {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        phases: phases.to_vec(),
        load: Load {
            power_w,
            power_factor: 0.9,
            continuous,
            kind: load_kind,
        },
        length_m,
        has_neutral: false,
        overrides: Overrides::default(),
    }
}

/// Casa habitación con acometida 3F-4H 220Y/127 V.
pub fn house() -> Project {
    use CircuitKind::{LineToLine, LineToNeutral};
    use LoadKind::{AirConditioning, General, Lighting};
    let mut living_room = circuit(
        "C-04",
        "Contactos sala",
        LineToNeutral,
        &[Phase::C],
        1500.0,
        false,
        General,
        35.0,
    );
    // Calibre fijado por el usuario, para mostrar un circuito que no cumple.
    living_room.overrides.size = Some(Size::new("12"));
    Project {
        format_version: FORMAT_VERSION,
        name: "Casa habitación (ejemplo)".to_string(),
        profile: "nom-001-sede-2012".to_string(),
        settings: Settings::default(),
        source: Source {
            name: "Acometida".to_string(),
            system: System::WYE_220_127,
        },
        panels: vec![Panel {
            id: "TG-1".to_string(),
            name: "Tablero TG-1".to_string(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 12.0,
                demand_factor: 1.0,
                has_neutral: true,
                overrides: Overrides::default(),
            },
            circuits: vec![
                circuit(
                    "C-01",
                    "Alumbrado planta baja",
                    LineToNeutral,
                    &[Phase::A],
                    1200.0,
                    true,
                    Lighting,
                    18.0,
                ),
                circuit(
                    "C-02",
                    "Contactos cocina",
                    LineToNeutral,
                    &[Phase::B],
                    1800.0,
                    false,
                    General,
                    22.0,
                ),
                circuit(
                    "C-03",
                    "Aire acondicionado",
                    LineToLine,
                    &[Phase::A, Phase::C],
                    2400.0,
                    false,
                    AirConditioning,
                    15.0,
                ),
                living_room,
                circuit(
                    "C-05",
                    "Alumbrado exterior",
                    LineToNeutral,
                    &[Phase::B],
                    600.0,
                    true,
                    Lighting,
                    40.0,
                ),
            ],
        }],
    }
}
