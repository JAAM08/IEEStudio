//! Casos de ingeniería del proyecto completo, en
//! `fixtures/engineering-cases/<perfil>/`. Un ingeniero los revisa y amplía
//! en una hoja de cálculo; esta prueba los ejecuta todos.

use voltforge_core::CircuitKind;
use voltforge_project::{
    calculate, example, Area, Circuit, Demand, Feeder, Load, LoadKind, Overrides, Panel,
};
use voltforge_rules::csv;

const FEEDER_LOAD: &str =
    include_str!("../../../fixtures/engineering-cases/nom-001-sede-2012/feeder_load.csv");

fn circuit(id: String, kind: LoadKind, va: f64) -> Circuit {
    Circuit {
        id: id.clone(),
        name: id,
        kind: CircuitKind::ThreePhase,
        phases: vec![],
        load: Load {
            power_w: va,
            power_factor: 1.0,
            continuous: false,
            kind,
            outlets: None,
            motor: None,
            compressor: None,
        },
        length_m: 10.0,
        has_neutral: false,
        overrides: Overrides::default(),
    }
}

/// Carga del alimentador según el artículo 220.
#[test]
fn feeder_load_cases() {
    let cases = csv::parse("feeder_load.csv", FEEDER_LOAD, |r| {
        let list = |column: &str| -> Vec<f64> {
            r.opt_text(column)
                .map(|t| t.split(';').map(|v| v.trim().parse().unwrap()).collect())
                .unwrap_or_default()
        };
        (
            r.text("id"),
            r.text("occupancy"),
            r.opt::<f64>("area_m2"),
            [
                (LoadKind::Lighting, list("lighting_va")),
                (LoadKind::Receptacles, list("receptacles_va")),
                (LoadKind::Appliance, list("appliances_va")),
                (LoadKind::KitchenEquipment, list("kitchen_va")),
                (LoadKind::General, list("other_va")),
            ],
            r.get::<f64>("expected_demanded_va"),
        )
    });
    assert!(cases.len() >= 10);
    for (id, occupancy, area, groups, expected) in cases {
        let mut project = example::house();
        project.occupancy = Some(occupancy);
        let mut circuits = Vec::new();
        for (kind, loads) in groups {
            for va in loads {
                circuits.push(circuit(format!("C-{}", circuits.len() + 1), kind, va));
            }
        }
        project.panels = vec![Panel {
            id: "T".into(),
            name: "Tablero".into(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 10.0,
                demand: Demand::Article220,
                demand_factor: None,
                has_neutral: true,
                interrupting_ka: None,
                overrides: Overrides::default(),
            },
            interrupting_ka: None,
            transformer: None,
            areas: area
                .map(|m2| {
                    vec![Area {
                        description: String::new(),
                        occupancy: None,
                        m2,
                    }]
                })
                .unwrap_or_default(),
            circuits,
        }];
        let result = calculate(&project).unwrap();
        let panel = &result.panels[0];
        assert!(
            (panel.demanded_va - expected).abs() < 0.01,
            "{id}: se esperaban {expected} VA, se obtuvieron {}",
            panel.demanded_va
        );
        let check = panel
            .feeder
            .checks
            .iter()
            .find(|c| c.name == "Carga del alimentador")
            .unwrap();
        assert_eq!(
            check.status,
            voltforge_project::Status::Complies,
            "{id}: {}",
            check.detail
        );
    }
}
