//! Proyecto de ejemplo: una casa habitación con un tablero y seis circuitos.

use voltforge_core::{CircuitKind, Size, System};

use crate::model::{
    Area, Circuit, CompressorLoad, Demand, Feeder, Load, LoadKind, MotorLoad, Overrides, Panel,
    Phase, Project, ProjectInfo, Settings, Source, SourceFault, FORMAT_VERSION,
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
            outlets: None,
            motor: None,
            compressor: None,
        },
        length_m,
        has_neutral: false,
        overrides: Overrides::default(),
    }
}

/// Casa habitación con acometida 3F-4H 220Y/127 V.
pub fn house() -> Project {
    use CircuitKind::{LineToLine, LineToNeutral};
    use LoadKind::{AirConditioning, Lighting, Motor, Receptacles};
    let mut living_room = circuit(
        "C-04",
        "Contactos sala",
        LineToNeutral,
        &[Phase::C],
        1500.0,
        false,
        Receptacles,
        35.0,
    );
    // Calibre fijado por el usuario, para mostrar un circuito que no cumple.
    living_room.overrides.size = Some(Size::new("12"));
    // Minisplit con la ampacidad mínima y la protección máxima de su placa.
    let mut air = circuit(
        "C-03",
        "Aire acondicionado",
        LineToLine,
        &[Phase::A, Phase::C],
        2400.0,
        false,
        AirConditioning,
        15.0,
    );
    air.load.compressor = Some(CompressorLoad {
        minimum_circuit_a: Some(15.0),
        maximum_protection_a: Some(25.0),
        ..CompressorLoad::default()
    });
    // Bomba de agua sin la corriente de placa: el relevador de sobrecarga
    // queda sin evaluar hasta capturarla.
    let mut pump = circuit(
        "C-06",
        "Bomba de agua",
        LineToNeutral,
        &[Phase::A],
        373.0,
        false,
        Motor,
        12.0,
    );
    pump.load.power_factor = 0.8;
    pump.load.motor = Some(MotorLoad {
        hp: "1/2".to_string(),
        motor_type: Default::default(),
        nameplate_a: None,
        service_factor_marked: false,
    });
    Project {
        format_version: FORMAT_VERSION,
        name: "Casa habitación (ejemplo)".to_string(),
        profile: "nom-001-sede-2012".to_string(),
        info: ProjectInfo {
            client: "Familia Pérez".to_string(),
            location: "Salamanca, Guanajuato".to_string(),
            revision: "A".to_string(),
            ..ProjectInfo::default()
        },
        occupancy: Some("dwellings".to_string()),
        settings: Settings::default(),
        source: Source {
            name: "Acometida".to_string(),
            system: System::WYE_220_127,
            // Transformador de la compañía de 75 kVA con 3.5 % de impedancia.
            fault: SourceFault {
                available_ka: None,
                transformer_kva: Some(75.0),
                transformer_impedance_percent: Some(3.5),
            },
        },
        panels: vec![Panel {
            id: "TG-1".to_string(),
            name: "Tablero TG-1".to_string(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 12.0,
                demand: Demand::Article220,
                demand_factor: None,
                has_neutral: true,
                interrupting_ka: Some(10.0),
                overrides: Overrides::default(),
            },
            interrupting_ka: Some(10.0),
            areas: vec![Area {
                description: "Planta baja y planta alta".to_string(),
                occupancy: None,
                m2: 120.0,
            }],
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
                    Receptacles,
                    22.0,
                ),
                air,
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
                pump,
            ],
        }],
    }
}
