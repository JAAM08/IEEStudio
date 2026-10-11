//! Cálculo de proyectos completos. Los valores esperados se calcularon a
//! mano con las mismas tablas que usa el motor.

use voltforge_core::{CircuitKind, Size, System};
use voltforge_project::{
    calculate, example, Circuit, Feeder, Load, LoadKind, Overrides, Panel, Project, RunResult,
    Status,
};

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 0.01,
        "se esperaba {expected}, se obtuvo {actual}"
    );
}

fn circuit<'a>(items: &'a [RunResult], id: &str) -> &'a RunResult {
    items
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("falta el circuito {id}"))
}

#[test]
fn example_house_load_schedule() {
    let result = calculate(&example::house()).unwrap();
    assert_eq!(result.panels.len(), 1);
    let panel = &result.panels[0];
    close(panel.connected_w, 7500.0);
    close(panel.power_factor, 0.9);
    // Fases en VA: A = 1333.33 + 1333.33, B = 2000 + 666.67, C = 1333.33 + 1666.67.
    close(panel.phase_va[0], 2666.67);
    close(panel.phase_va[1], 2666.67);
    close(panel.phase_va[2], 3000.0);
    close(panel.imbalance_percent.unwrap(), 11.11);
    assert_eq!(result.warnings.len(), 1);

    // Alimentador: 7500 / (√3 · 220 · 0.9) = 21.87 A; 24 % continuo, diseño
    // 23.18 A, interruptor de 25 A, 10 AWG (30 A a 60 °C).
    let feeder = panel.feeder.run.as_ref().unwrap();
    close(feeder.nominal_a, 21.87);
    close(feeder.design_a, 23.18);
    close(feeder.breaker_a, 25.0);
    assert_eq!(feeder.size, Size::new("10"));
    assert_eq!(feeder.poles, 3);
    assert_eq!(feeder.current_carrying_conductors, 3);
    close(feeder.drop_percent, 0.74);
    assert_eq!(feeder.conduit.as_ref().unwrap().metric_designator, 16);
    assert_eq!(panel.feeder.status, Status::Complies);

    // C-01: 10.5 A continuos, 15 A, 14 AWG, 2.76 %; acumulada con el alimentador.
    let c01 = circuit(&panel.circuits, "C-01");
    let run = c01.run.as_ref().unwrap();
    close(run.breaker_a, 15.0);
    assert_eq!(run.size, Size::new("14"));
    close(run.drop_percent, 2.76);
    close(run.cumulative_drop_percent, 2.76 + 0.74);
    assert_eq!(c01.status, Status::Complies);

    // C-02: sube de 12 a 10 AWG por caída de tensión; la tierra sube con él.
    let c02 = circuit(&panel.circuits, "C-02").run.as_ref().unwrap();
    close(c02.breaker_a, 20.0);
    assert_eq!(c02.size, Size::new("10"));
    assert_eq!(c02.governing.as_deref(), Some("caída de tensión"));
    assert_eq!(c02.grounding_size, Size::new("10"));
    close(c02.drop_percent, 1.96);

    // C-03: se calcula, pero queda sin evaluar por ser climatización.
    let c03 = circuit(&panel.circuits, "C-03");
    assert_eq!(c03.status, Status::NotEvaluated);
    close(c03.run.as_ref().unwrap().nominal_a, 12.12);
    assert_eq!(c03.run.as_ref().unwrap().poles, 2);

    // C-04: calibre 12 fijado, 4.37 %; el programa recomienda 10 AWG.
    let c04 = circuit(&panel.circuits, "C-04");
    let run = c04.run.as_ref().unwrap();
    assert!(run.forced);
    assert_eq!(run.size, Size::new("12"));
    assert_eq!(run.recommended_size, Some(Size::new("10")));
    close(run.drop_percent, 4.37);
    assert_eq!(c04.status, Status::Fails);

    assert_eq!(
        (result.complies, result.fails, result.not_evaluated),
        (4, 1, 1)
    );
}

#[test]
fn project_round_trips_through_json() {
    let project = example::house();
    let json = project.to_json();
    assert_eq!(Project::from_json(&json).unwrap(), project);
    // Un archivo mínimo toma los criterios por defecto.
    let minimal = r#"{
        "format_version": 1, "name": "Mínimo", "profile": "nom-001-sede-2012",
        "source": {"name": "Acometida", "system": {"line_to_line_v": null, "line_to_neutral_v": 127.0}},
        "panels": []
    }"#;
    let p = Project::from_json(minimal).unwrap();
    close(p.settings.branch_drop_limit_percent, 3.0);
    close(p.settings.total_drop_limit_percent, 5.0);
    assert!(Project::from_json("{").is_err());
    assert!(
        Project::from_json(&json.replace("\"format_version\": 1", "\"format_version\": 99"))
            .is_err()
    );
}

fn industrial() -> Project {
    let mut project = example::house();
    project.name = "Nave".to_string();
    project.source.system = System::WYE_480_277;
    project.panels = vec![
        Panel {
            id: "TGN".to_string(),
            name: "Tablero general".to_string(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 40.0,
                demand_factor: 0.8,
                has_neutral: true,
                overrides: Overrides::default(),
            },
            circuits: vec![],
        },
        Panel {
            id: "TF-1".to_string(),
            name: "Tablero de fuerza".to_string(),
            parent: Some("TGN".to_string()),
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 60.0,
                demand_factor: 1.0,
                has_neutral: false,
                overrides: Overrides::default(),
            },
            circuits: vec![Circuit {
                id: "F-01".to_string(),
                name: "Horno".to_string(),
                kind: CircuitKind::ThreePhase,
                phases: vec![],
                load: Load {
                    power_w: 90_000.0,
                    power_factor: 1.0,
                    continuous: true,
                    kind: LoadKind::General,
                },
                length_m: 25.0,
                has_neutral: false,
                overrides: Overrides::default(),
            }],
        },
    ];
    project
}

#[test]
fn subpanels_add_load_and_voltage_drop_upstream() {
    let result = calculate(&industrial()).unwrap();
    let general = result.panels.iter().find(|p| p.id == "TGN").unwrap();
    let force = result.panels.iter().find(|p| p.id == "TF-1").unwrap();

    // El tablero general suma la carga de su subtablero y aplica su factor de demanda.
    close(general.connected_w, 90_000.0);
    close(general.demanded_w, 72_000.0);
    assert_eq!(general.imbalance_percent, None);

    // Horno: 90 kW a 480 V = 108.25 A continuos; diseño 135.32 A; interruptor 150 A.
    // A 75 °C: 1 AWG lleva 130 A y 1/0 AWG 150 A.
    let oven = circuit(&force.circuits, "F-01").run.as_ref().unwrap();
    close(oven.nominal_a, 108.25);
    close(oven.design_a, 135.32);
    close(oven.breaker_a, 150.0);
    assert_eq!(oven.size, Size::new("1/0"));

    // La caída se acumula: general, luego fuerza, luego el circuito.
    let g = general.feeder.run.as_ref().unwrap();
    let f = force.feeder.run.as_ref().unwrap();
    close(g.cumulative_drop_percent, g.drop_percent);
    close(f.cumulative_drop_percent, g.drop_percent + f.drop_percent);
    close(
        oven.cumulative_drop_percent,
        f.cumulative_drop_percent + oven.drop_percent,
    );
    assert!(oven.cumulative_drop_percent <= 5.0);
}

#[test]
fn structural_problems_are_errors() {
    let mut unknown_profile = example::house();
    unknown_profile.profile = "iec".to_string();
    assert!(calculate(&unknown_profile).is_err());

    let mut orphan = industrial();
    orphan.panels[1].parent = Some("NO-EXISTE".to_string());
    assert!(calculate(&orphan).unwrap_err().contains("no existe"));

    let mut cycle = industrial();
    cycle.panels[0].parent = Some("TF-1".to_string());
    assert!(calculate(&cycle).unwrap_err().contains("sí mismo"));

    let mut duplicate = example::house();
    let copy = duplicate.panels[0].circuits[0].clone();
    duplicate.panels[0].circuits.push(copy);
    assert!(calculate(&duplicate).unwrap_err().contains("repite"));
}

#[test]
fn a_case_the_tables_do_not_cover_is_not_evaluated() {
    // Un circuito trifásico en un servicio de una fase no se puede calcular.
    let mut project = example::house();
    project.source.system = System::SINGLE_127;
    let result = calculate(&project).unwrap();
    let panel = &result.panels[0];
    assert_eq!(panel.feeder.status, Status::NotEvaluated);
    assert!(panel.feeder.run.is_none());
    let c03 = circuit(&panel.circuits, "C-03");
    assert_eq!(c03.status, Status::NotEvaluated);
    // Los circuitos de una fase sí se calculan.
    assert!(circuit(&panel.circuits, "C-01").run.is_some());
}
