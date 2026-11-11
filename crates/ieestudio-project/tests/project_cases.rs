//! Cálculo de proyectos completos. Los valores esperados se calcularon a
//! mano con las mismas tablas que usa el motor.

use ieestudio_core::{CircuitKind, Size, System};
use ieestudio_project::{
    calculate, example, Area, Circuit, Demand, Feeder, Load, LoadKind, Overrides, Panel, Project,
    RunResult, Status,
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
    // El minisplit (C-03) cuenta 12 A · 220 V = 2640 VA (su ampacidad mínima
    // de 15 A entre 1.25) y la bomba (C-06) 8.9 A · 127 V = 1130.3 VA, con
    // sus factores de potencia de 0.9 y 0.8.
    close(
        panel.connected_w,
        1200.0 + 1800.0 + 2376.0 + 1500.0 + 600.0 + 904.24,
    );
    close(panel.power_factor, 8380.24 / 9436.97);
    // Fases en VA: A = 1333.33 + 1320 + 1130.3, B = 2000 + 666.67, C = 1320 + 1666.67.
    close(panel.phase_va[0], 3783.63);
    close(panel.phase_va[1], 2666.67);
    close(panel.phase_va[2], 2986.67);
    close(panel.imbalance_percent.unwrap(), 29.52);
    assert_eq!(result.warnings.len(), 1);

    // Carga del alimentador (artículo 220, vivienda). Alumbrado y contactos:
    // 1333.33 + 666.67 + 2000 + 1666.67 = 5666.67 VA, más que la mínima de
    // 33 VA/m² × 120 m² = 3960 VA. Tabla 220-42: 3000 al 100 % y 2666.67 al
    // 35 % = 3933.33 VA. Motores y compresor al 100 %: 2640 + 1130.3 VA.
    close(panel.demanded_va, 3933.33 + 3770.3);
    assert_eq!(panel.demand.len(), 2);
    // 7703.63 VA / (√3 · 220) = 20.22 A. Continuo: el alumbrado, 2000 de
    // 5666.67 VA del grupo: 3933.33 · 0.3529 = 1388.24 VA = 3.64 A. El
    // compresor es el motor mayor: 2640 / 381.05 = 6.93 A.
    // Id = 1.25 · 3.64 + 16.57 + 0.25 · 6.93 = 22.86 A → 25 A, 10 AWG
    // (12 AWG lleva 20 A a 60 °C).
    let feeder = panel.feeder.run.as_ref().unwrap();
    close(feeder.nominal_a, 20.22);
    close(feeder.design_a, 22.86);
    close(feeder.breaker_a, 25.0);
    assert_eq!(feeder.size, Size::new("10"));
    assert_eq!(feeder.poles, 3);
    assert_eq!(feeder.current_carrying_conductors, 3);
    close(feeder.drop_percent, 0.68);
    assert_eq!(feeder.conduit.as_ref().unwrap().metric_designator, 16);
    let load = panel
        .feeder
        .checks
        .iter()
        .find(|c| c.name == "Carga del alimentador")
        .unwrap();
    assert_eq!(load.status, Status::Complies);
    assert_eq!(panel.feeder.status, Status::Complies);

    // C-01: 10.5 A continuos, 15 A, 14 AWG, 2.76 %; acumulada con el alimentador.
    let c01 = circuit(&panel.circuits, "C-01");
    let run = c01.run.as_ref().unwrap();
    close(run.breaker_a, 15.0);
    assert_eq!(run.size, Size::new("14"));
    close(run.drop_percent, 2.76);
    close(run.cumulative_drop_percent, 2.76 + 0.68);
    assert_eq!(c01.status, Status::Complies);

    // C-02: sube de 12 a 10 AWG por caída de tensión; la tierra sube con él.
    let c02 = circuit(&panel.circuits, "C-02").run.as_ref().unwrap();
    close(c02.breaker_a, 20.0);
    assert_eq!(c02.size, Size::new("10"));
    assert_eq!(c02.governing.as_deref(), Some("caída de tensión"));
    assert_eq!(c02.grounding_size, Size::new("10"));
    close(c02.drop_percent, 1.96);

    // C-03: minisplit con placa de 15 A mínimos y 25 A máximos (440-35,
    // 440-22(c)): 14 AWG (15 A a 60 °C) e interruptor de 25 A. La tierra de
    // tabla para 25 A (10 AWG) se limita al calibre de fase (250-122(a)).
    let c03 = circuit(&panel.circuits, "C-03");
    assert_eq!(c03.status, Status::Complies);
    let run = c03.run.as_ref().unwrap();
    close(run.nominal_a, 12.0);
    close(run.design_a, 15.0);
    close(run.breaker_a, 25.0);
    assert_eq!(run.size, Size::new("14"));
    assert_eq!(run.grounding_size, Size::new("14"));
    assert_eq!(run.poles, 2);

    // C-06: bomba de 1/2 hp a 127 V: 8.9 A de la Tabla 430-248, diseño de
    // 11.13 A, 14 AWG, interruptor de 25 A (250 % = 22.25 A, inmediato
    // superior). Sin la corriente de placa, la sobrecarga queda sin evaluar.
    let c06 = circuit(&panel.circuits, "C-06");
    let run = c06.run.as_ref().unwrap();
    close(run.nominal_a, 8.9);
    close(run.design_a, 11.125);
    close(run.breaker_a, 25.0);
    assert_eq!(run.size, Size::new("14"));
    assert_eq!(c06.status, Status::NotEvaluated);
    let overload = c06.checks.iter().find(|c| c.name == "Sobrecarga").unwrap();
    assert_eq!(overload.status, Status::NotEvaluated);
    let protection = c06.checks.iter().find(|c| c.name == "Protección").unwrap();
    assert_eq!(protection.status, Status::Complies);

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
        (5, 1, 1)
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
        Project::from_json(&json.replace("\"format_version\": 3", "\"format_version\": 99"))
            .is_err()
    );
}

fn industrial() -> Project {
    let mut project = example::house();
    project.name = "Nave".to_string();
    project.occupancy = Some("industrial_commercial".to_string());
    project.source.system = System::WYE_480_277;
    project.panels = vec![
        Panel {
            board: Default::default(),
            interrupting_ka: Default::default(),
            id: "TGN".to_string(),
            name: "Tablero general".to_string(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                interrupting_ka: Default::default(),
                length_m: 40.0,
                demand: Demand::Custom(0.8),
                demand_factor: None,
                has_neutral: true,
                overrides: Overrides::default(),
            },
            transformer: None,
            areas: vec![],
            circuits: vec![],
        },
        Panel {
            board: Default::default(),
            interrupting_ka: Default::default(),
            id: "TF-1".to_string(),
            name: "Tablero de fuerza".to_string(),
            parent: Some("TGN".to_string()),
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                interrupting_ka: Default::default(),
                length_m: 60.0,
                demand: Demand::Article220,
                demand_factor: None,
                has_neutral: false,
                overrides: Overrides::default(),
            },
            transformer: None,
            areas: vec![],
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
                    outlets: None,
                    motor: None,
                    compressor: None,
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

/// Cada tramo calculado lleva su procedimiento: corriente, diseño, interruptor,
/// ampacidad, verificación, tierra, tubería, cuatro pasos de caída y la acumulada.
#[test]
fn every_run_carries_its_procedure() {
    let result = calculate(&example::house()).unwrap();
    let panel = &result.panels[0];
    for item in std::iter::once(&panel.feeder).chain(&panel.circuits) {
        let run = item.run.as_ref().unwrap();
        // El alimentador empieza con su carga según el artículo 220.
        let steps: Vec<_> = run
            .steps
            .iter()
            .filter(|s| !s.title.starts_with("Carga"))
            .collect();
        assert_eq!(steps.len(), 12, "{}", item.id);
        assert!(
            [
                "Corriente nominal",
                "Corriente del motor",
                "Corriente del compresor"
            ]
            .contains(&steps[0].title.as_str()),
            "{}",
            item.id
        );
        assert_eq!(steps[11].title, "Caída acumulada desde la fuente");
    }
    // C-02: 1800 W a 127 V con fp 0.90: I = 1800 / (127 · 0.9) = 15.75 A.
    let c02 = panel.circuits[1].run.as_ref().unwrap();
    assert_eq!(
        c02.steps[0].substitution,
        "I = 1,800 / (127 · 0.90) = 15.75 A"
    );
}

/// El veredicto separa cálculo, verificación y aprobación profesional, y no
/// declara conforme lo que tiene verificaciones pendientes.
#[test]
fn verdict_keeps_three_levels_apart() {
    use ieestudio_project::verdict::{Calculation, Compliance};

    // La casa: todo se calcula, C-04 no cumple y la sobrecarga de la bomba
    // (C-06) queda sin evaluar.
    let house = calculate(&example::house()).unwrap().verdict;
    assert_eq!(house.calculation, Calculation::Complete);
    assert_eq!(house.compliance, Compliance::DoesNotConform);
    assert!(!house.professionally_approved);

    // Sin C-04 nada incumple, pero la bomba sigue sin evaluar: el proyecto
    // no puede declararse conforme.
    let mut project = example::house();
    project.panels[0].circuits.retain(|c| c.id != "C-04");
    let pending = calculate(&project).unwrap().verdict;
    assert_eq!(pending.compliance, Compliance::Pending);
    assert_eq!(pending.pending_checks, 1);

    // Con la corriente de placa de la bomba todo cumple, pero la aprobación
    // sigue pendiente.
    project.panels[0]
        .circuits
        .iter_mut()
        .find(|c| c.id == "C-06")
        .and_then(|c| c.load.motor.as_mut())
        .unwrap()
        .nameplate_a = Some(7.6);
    let conforms = calculate(&project).unwrap().verdict;
    assert_eq!(conforms.compliance, Compliance::Conforms);
    assert!(!conforms.professionally_approved);
    // Es conforme en lo evaluado: la capacidad interruptiva queda fuera del
    // alcance del programa, con su referencia (decisión 0018).
    let [_, (_, _, compliance), _] = conforms.lines("NOM-001-SEDE-2012");
    assert!(
        compliance.starts_with("Conforme en lo evaluado"),
        "{compliance}"
    );
    assert!(compliance.contains("Fuera de alcance: capacidad interruptiva"));
    assert!(compliance.contains("110-9"), "{compliance}");
    assert!(compliance.contains("requiere estudio de cortocircuito"));

    // Un proyecto sin circuitos no tiene nada calculado.
    project.panels[0].circuits.clear();
    let empty = calculate(&project).unwrap().verdict;
    assert_eq!(empty.calculation, Calculation::Incomplete);
    assert_ne!(empty.compliance, Compliance::Conforms);
}

fn motor_circuit(id: &str, hp: &str, length_m: f64) -> Circuit {
    Circuit {
        id: id.to_string(),
        name: format!("Motor de {hp} hp"),
        kind: CircuitKind::ThreePhase,
        phases: vec![],
        load: Load {
            power_w: 0.0,
            power_factor: 0.85,
            continuous: false,
            kind: LoadKind::Motor,
            outlets: None,
            motor: Some(ieestudio_project::MotorLoad {
                hp: hp.to_string(),
                motor_type: Default::default(),
                nameplate_a: Some(26.0),
                service_factor_marked: true,
            }),
            compressor: None,
        },
        length_m,
        has_neutral: false,
        overrides: Overrides::default(),
    }
}

/// Tablero de motores a 220 V: 10 hp (28 A), 15 hp (42 A) y 3000 W de
/// alumbrado continuo trifásico (7.87 A).
fn motor_panel() -> Project {
    let mut project = example::house();
    // 22 VA/m² × 100 m² = 2200 VA de alumbrado mínimo: manda el capturado.
    project.occupancy = Some("industrial_commercial".to_string());
    let lighting = Circuit {
        id: "A-01".to_string(),
        name: "Alumbrado".to_string(),
        kind: CircuitKind::ThreePhase,
        phases: vec![],
        load: Load {
            power_w: 3000.0,
            power_factor: 1.0,
            continuous: true,
            kind: LoadKind::Lighting,
            outlets: None,
            motor: None,
            compressor: None,
        },
        length_m: 10.0,
        has_neutral: false,
        overrides: Overrides::default(),
    };
    project.panels = vec![Panel {
        board: Default::default(),
        interrupting_ka: Default::default(),
        id: "TM".to_string(),
        name: "Tablero de motores".to_string(),
        parent: None,
        kind: CircuitKind::ThreePhase,
        feeder: Feeder {
            interrupting_ka: Default::default(),
            length_m: 10.0,
            demand: Demand::Article220,
            demand_factor: None,
            has_neutral: false,
            overrides: Overrides::default(),
        },
        transformer: None,
        areas: vec![Area {
            description: "Nave".to_string(),
            occupancy: None,
            m2: 100.0,
        }],
        circuits: vec![
            motor_circuit("M-10", "10", 10.0),
            motor_circuit("M-15", "15", 10.0),
            lighting,
        ],
    }];
    project
}

/// 430-24: el alimentador lleva el 125 % del motor mayor más los demás
/// motores y las otras cargas; 430-62 permite un interruptor mayor que la
/// ampacidad del conductor, hasta el del motor mayor más el resto.
#[test]
fn motor_feeder_follows_430_24_and_430_62() {
    let result = calculate(&motor_panel()).unwrap();
    let panel = &result.panels[0];
    // 28 + 42 + 3000 / (√3 · 220) = 77.87 A.
    let feeder = panel.feeder.run.as_ref().unwrap();
    close(feeder.nominal_a, 77.87);
    // Id = 1.25 · 7.87 + 70 + 0.25 · 42 = 90.34 A → 100 A; 2 AWG (95 A a 60 °C).
    close(feeder.design_a, 90.34);
    close(feeder.breaker_a, 100.0);
    assert_eq!(feeder.size, Size::new("2"));
    assert_eq!(panel.feeder.status, Status::Complies);
    // El motor de 15 hp lleva 110 A: el alimentador de 100 A puede dispararse.
    assert!(result.warnings.iter().any(|w| w.contains("motor mayor")));

    // M-15: 42 A, 52.5 A de diseño, 6 AWG e interruptor de 110 A.
    let m15 = circuit(&panel.circuits, "M-15");
    let run = m15.run.as_ref().unwrap();
    close(run.design_a, 52.5);
    close(run.breaker_a, 110.0);
    assert_eq!(run.size, Size::new("6"));
    assert_eq!(m15.status, Status::Complies);
    // La sobrecarga se ajusta con la corriente de placa: 1.25 · 26 = 32.5 A.
    let overload = m15.checks.iter().find(|c| c.name == "Sobrecarga").unwrap();
    assert!(overload.detail.contains("32.50"), "{}", overload.detail);

    // Alimentador con interruptor de 125 A fijado: 240-4 no lo admite para
    // 2 AWG, pero 430-62 sí, hasta 77.87 + (110 − 42) = 145.87 A.
    let mut project = motor_panel();
    project.panels[0].feeder.overrides.breaker_a = Some(125.0);
    let result = calculate(&project).unwrap();
    let feeder = &result.panels[0].feeder;
    assert_eq!(feeder.run.as_ref().unwrap().breaker_a, 125.0);
    assert_eq!(feeder.status, Status::Complies);
    // Con 150 A el programa ya no deja el 2 AWG: propone uno que el
    // interruptor protege. Si el usuario fija el 2 AWG, no cumple.
    project.panels[0].feeder.overrides.breaker_a = Some(150.0);
    let result = calculate(&project).unwrap();
    assert_ne!(
        result.panels[0].feeder.run.as_ref().unwrap().size,
        Size::new("2")
    );
    assert_eq!(result.panels[0].feeder.status, Status::Complies);
    project.panels[0].feeder.overrides.size = Some(Size::new("2"));
    let result = calculate(&project).unwrap();
    assert_eq!(result.panels[0].feeder.status, Status::Fails);
}

/// 430-52: un interruptor de motor mayor que el de la tabla solo se admite
/// si el motor no arranca (excepción 2); arriba de su límite no cumple.
#[test]
fn motor_breaker_above_the_table_needs_confirmation() {
    let status_with = |breaker: f64| {
        let mut project = motor_panel();
        project.panels[0].circuits[1].overrides.breaker_a = Some(breaker);
        let result = calculate(&project).unwrap();
        let m15 = circuit(&result.panels[0].circuits, "M-15").clone();
        m15.checks
            .iter()
            .find(|c| c.name == "Protección")
            .unwrap()
            .status
    };
    assert_eq!(status_with(100.0), Status::Complies);
    assert_eq!(status_with(150.0), Status::NotEvaluated); // ≤ 400 % de 42 A = 168 A
    assert_eq!(status_with(175.0), Status::Fails);
}

/// 210-20(a): un interruptor fijado menor que la corriente de diseño no cumple.
#[test]
fn a_fixed_breaker_must_cover_the_design_current() {
    let mut project = example::house();
    // C-01: 10.5 A continuos, 13.13 A de diseño.
    project.panels[0].circuits[0].overrides.breaker_a = Some(10.0);
    let result = calculate(&project).unwrap();
    let c01 = circuit(&result.panels[0].circuits, "C-01");
    let protection = c01.checks.iter().find(|c| c.name == "Protección").unwrap();
    assert_eq!(protection.status, Status::Fails);
    assert!(c01.run.as_ref().unwrap().breaker_forced);
}

/// Un motor sin potencia o en una tensión que la tabla no cubre se calcula
/// como carga general y queda sin evaluar, con el motivo.
#[test]
fn a_motor_without_data_is_not_evaluated() {
    let mut project = motor_panel();
    project.panels[0].circuits[0].load.motor = None;
    project.panels[0].circuits[0].load.power_w = 7460.0;
    let result = calculate(&project).unwrap();
    let m10 = circuit(&result.panels[0].circuits, "M-10");
    assert_eq!(m10.status, Status::NotEvaluated);
    assert!(m10.run.is_some());
    assert!(m10
        .checks
        .iter()
        .any(|c| c.detail.contains("potencia del motor")));

    let mut project = motor_panel();
    project.source.system = System {
        line_to_line_v: Some(380.0),
        line_to_neutral_v: Some(220.0),
    };
    let result = calculate(&project).unwrap();
    let m10 = circuit(&result.panels[0].circuits, "M-10");
    assert_eq!(m10.status, Status::NotEvaluated);
}

/// Sin datos de la fuente la capacidad interruptiva queda fuera del alcance
/// del programa: ningún tramo la verifica, y el veredicto la declara aparte
/// (decisiones 0018 y 0023).
#[test]
fn interrupting_rating_is_out_of_scope() {
    for project in [example::house(), industrial(), transformer_project()] {
        let result = calculate(&project).unwrap();
        for panel in &result.panels {
            for run in panel.runs() {
                assert!(
                    run.all_checks().all(|c| c.name != "Capacidad interruptiva"),
                    "{}: {}",
                    panel.id,
                    run.id
                );
                if let Some(r) = &run.run {
                    assert!(r.steps.iter().all(|s| !s.title.contains("Cortocircuito")));
                }
            }
        }
        let out = &result.verdict.out_of_scope;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "Capacidad interruptiva");
        assert_eq!(
            out[0].references[0].clause.as_deref(),
            Some("110-9, 110-10")
        );
    }
}

/// 250-66: el conductor al electrodo sale del mayor conductor de la acometida.
#[test]
fn grounding_electrode_follows_the_service_conductor() {
    let result = calculate(&example::house()).unwrap();
    // Acometida de 10 AWG: 2 AWG o menor, conductor al electrodo de 8 AWG.
    let gec = result.grounding_electrode.as_ref().unwrap();
    assert_eq!(gec.size, Size::new("8"));
}

/// Un alimentador de 446.13 A: un solo conductor sería de 1500 kcmil. El
/// programa propone dos por fase, un juego por tubo; cada juego lleva tres
/// portadores y no se ajusta por agrupamiento. Con 557.67 A de diseño y
/// 600 A de interruptor, dos de 300 kcmil (285 A a 75 °C) dan 570 A.
#[test]
fn large_feeders_go_parallel() {
    let mut project = motor_panel();
    let circuit = &mut project.panels[0].circuits[2];
    circuit.load.power_w = 170_000.0;
    circuit.load.kind = LoadKind::General;
    circuit.load.continuous = true;
    project.panels[0].circuits.truncate(3);
    project.panels[0].circuits.drain(0..2);
    project.panels[0].feeder.length_m = 5.0;
    project.panels[0].areas.clear();
    let result = calculate(&project).unwrap();
    let panel = &result.panels[0];
    let feeder = panel.feeder.run.as_ref().unwrap();
    close(feeder.design_a, 557.67);
    close(feeder.breaker_a, 600.0);
    assert_eq!(feeder.conductors_per_phase, 2);
    assert!(feeder.parallel_auto);
    assert_eq!(feeder.size, Size::new("300"));
    assert_eq!(feeder.current_carrying_conductors, 3);
    assert_eq!(feeder.conduit.as_ref().unwrap().count, 2);
    assert!(feeder
        .steps
        .iter()
        .any(|s| s.title == "Conductores en paralelo"));
    // Tabla 250-66: dos de 300 kcmil equivalen a 304 mm² (600 kcmil): 1/0 AWG.
    let gec = result.grounding_electrode.as_ref().unwrap();
    assert_eq!(gec.size, Size::new("1/0"));
    assert_eq!(gec.aluminum_size, Size::new("3/0"));
    assert_eq!(gec.service, "2 × 300 kcmil de cobre");
    // El circuito de 446 A también va en paralelo.
    let c = panel.circuits[0].run.as_ref().unwrap();
    assert!(c.conductors_per_phase >= 2);

    // Todos los juegos en un solo tubo: seis portadores, factor de 0.80.
    project.panels[0].feeder.overrides.single_raceway = true;
    project.panels[0].feeder.overrides.conductors_per_phase = Some(2);
    let result = calculate(&project).unwrap();
    let feeder = result.panels[0].feeder.run.as_ref().unwrap();
    assert_eq!(feeder.current_carrying_conductors, 6);
    assert_eq!(feeder.conduit.as_ref().unwrap().count, 1);
    assert!(!feeder.parallel_auto);
}

/// Un tablero de 220Y/127 V alimentado desde 480Y/277 V por un transformador
/// de 45 kVA y 3 %. Primario: In = 45 000 / (√3 · 480) = 54.13 A; diseño al
/// 125 %, 67.66 A: interruptor de 70 A y 4 AWG (70 A a 60 °C). Secundario:
/// In = 118.09 A; 147.62 A de diseño: interruptor de 150 A (el normalizado
/// inmediato superior al 125 %, nota 1) y 1/0 AWG (150 A a 75 °C).
fn transformer_project() -> Project {
    let mut project = industrial();
    let force = &mut project.panels[1];
    force.name = "Tablero de servicios".into();
    force.circuits[0].load.power_w = 20_000.0;
    force.circuits[0].load.continuous = false;
    force.feeder.length_m = 20.0;
    force.transformer = Some(ieestudio_project::Transformer {
        brand: Default::default(),
        connection: Default::default(),
        mounting: Default::default(),
        kva: 45.0,
        impedance_percent: 3.0,
        secondary: System::WYE_220_127,
        secondary_protection: true,
        secondary_length_m: 3.0,
        secondary_has_neutral: true,
        secondary_overrides: Overrides::default(),
    });
    project
}

#[test]
fn transformer_sizes_both_sides_by_its_rating() {
    let result = calculate(&transformer_project()).unwrap();
    let panel = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    let t = panel.transformer.as_ref().unwrap();
    close(t.primary_a.unwrap(), 54.13);
    close(t.secondary_a.unwrap(), 118.09);
    close(t.load_percent, 44.44);

    let primary = panel.feeder.run.as_ref().unwrap();
    close(primary.nominal_a, 24.06);
    close(primary.design_a, 67.66);
    close(primary.breaker_a, 70.0);
    assert_eq!(primary.size, Size::new("4"));
    for name in ["Protección del primario", "Capacidad del transformador"] {
        let c = panel.feeder.checks.iter().find(|c| c.name == name).unwrap();
        assert_eq!(c.status, Status::Complies, "{name}: {}", c.detail);
    }

    let secondary = panel.secondary.as_ref().unwrap();
    let run = secondary.run.as_ref().unwrap();
    close(run.nominal_a, 52.49);
    close(run.design_a, 147.62);
    close(run.breaker_a, 150.0);
    assert_eq!(run.size, Size::new("1/0"));
    // Sistema derivado: la caída se cuenta desde el secundario.
    close(run.cumulative_drop_percent, run.drop_percent);
    let c = secondary
        .checks
        .iter()
        .find(|c| c.name == "Protección del secundario")
        .unwrap();
    assert_eq!(c.status, Status::Complies, "{}", c.detail);

    // El circuito de 127/220 V se calcula con la tensión del secundario.
    let oven = circuit(&panel.circuits, "F-01").run.as_ref().unwrap();
    close(oven.nominal_a, 52.49);
    close(
        oven.cumulative_drop_percent,
        run.drop_percent + oven.drop_percent,
    );
}

/// Un transformador chico para la carga no cumple, y sin protección en el
/// secundario se exige revisar 240-21(c).
#[test]
fn an_overloaded_transformer_fails() {
    let mut project = transformer_project();
    let t = project.panels[1].transformer.as_mut().unwrap();
    t.kva = 15.0;
    t.secondary_protection = false;
    let result = calculate(&project).unwrap();
    let panel = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    let capacity = panel
        .feeder
        .checks
        .iter()
        .find(|c| c.name == "Capacidad del transformador")
        .unwrap();
    assert_eq!(capacity.status, Status::Fails);
    let secondary = panel.secondary.as_ref().unwrap();
    let protection = secondary
        .checks
        .iter()
        .find(|c| c.name == "Protección del secundario")
        .unwrap();
    assert_eq!(protection.status, Status::NotEvaluated);
}

fn checks_named<'a>(run: &'a RunResult, name: &str) -> Vec<&'a ieestudio_project::Check> {
    run.all_checks().filter(|c| c.name == name).collect()
}

/// Con el transformador de la acometida, la falla por bus infinito en el
/// punto de conexión (curso: 112.5 kVA, 220 V, Z = 3 %, 9.84 kA) con la
/// impedancia reducida 10 %, más 4 veces la corriente de los motores del
/// proyecto: la bomba (8.9 A) y el minisplit (12 A).
#[test]
fn fault_by_infinite_bus_with_motors() {
    let mut project = example::house();
    project.source.substation = Some(ieestudio_project::Substation {
        impedance_from_table: Default::default(),
        kva: 112.5,
        impedance_percent: 3.0,
        primary_kv: Some(13.2),
        connection: None,
        mounting: None,
        brand: String::new(),
    });
    let result = calculate(&project).unwrap();
    let study = result.fault.as_ref().expect("con subestación hay estudio");
    close(study.source.base_ka, 9.8412 / 0.9);
    let motors = study.motors.as_ref().expect("el ejemplo tiene motores");
    assert_eq!(motors.motors.len(), 2);
    assert_eq!(motors.multiplier, 4.0);
    close(motors.total_a, 4.0 * (8.9 + 12.0));
    close(study.source.total_ka, 9.8412 / 0.9 + 0.0836);
    // El tablero sin transformador ve la misma falla que el punto de conexión.
    let panel = &result.panels[0];
    let bus = panel.fault.as_ref().unwrap();
    close(bus.total_ka, study.source.total_ka);
    // El procedimiento muestra la aportación de motores.
    assert!(bus
        .steps
        .iter()
        .any(|s| s.title == "Aportación de los motores"));
    // Sin capacidad interruptiva capturada, cada interruptor queda sin evaluar
    // y la capacidad interruptiva ya no está fuera de alcance.
    for run in panel.runs() {
        let checks = checks_named(run, "Capacidad interruptiva");
        assert_eq!(checks.len(), 1, "{}", run.id);
        assert_eq!(checks[0].status, Status::NotEvaluated);
        assert!(checks[0]
            .detail
            .starts_with("falta la capacidad interruptiva"));
    }
    assert!(result.verdict.out_of_scope.is_empty());
    assert!(result.verdict.conservative_fault);

    // 22 kA en el tablero y en el principal: todo cumple con el método conservador.
    project.panels[0].board.interrupting_ka = Some(22.0);
    project.panels[0].feeder.overrides.breaker.interrupting_ka = Some(22.0);
    let result = calculate(&project).unwrap();
    for run in result.panels[0].runs() {
        let c = checks_named(run, "Capacidad interruptiva")[0];
        assert_eq!(c.status, Status::Complies, "{}: {}", run.id, c.detail);
        assert!(c.detail.contains("método conservador"));
        let clauses: Vec<_> = c
            .references
            .iter()
            .filter_map(|r| r.clause.clone())
            .collect();
        assert!(clauses.iter().any(|x| x == "110-9, 110-10"));
    }

    // 10 kA no alcanza los 11 kA estimados: no concluyente, nunca «no cumple».
    project.panels[0].board.interrupting_ka = Some(10.0);
    let result = calculate(&project).unwrap();
    let c02 = circuit(&result.panels[0].circuits, "C-02");
    let c = checks_named(c02, "Capacidad interruptiva")[0];
    assert_eq!(c.status, Status::NotEvaluated);
    assert!(c.detail.starts_with("no concluyente"));
    // El interruptor de un circuito puede tener la suya.
    project.panels[0].circuits[1]
        .overrides
        .breaker
        .interrupting_ka = Some(25.0);
    let result = calculate(&project).unwrap();
    let c02 = circuit(&result.panels[0].circuits, "C-02");
    assert_eq!(
        checks_named(c02, "Capacidad interruptiva")[0].status,
        Status::Complies
    );
}

/// El dato de la suministradora manda sobre la estimación con la subestación.
#[test]
fn supplier_fault_current_takes_precedence() {
    let mut project = example::house();
    project.source.substation = Some(ieestudio_project::Substation {
        impedance_from_table: Default::default(),
        kva: 112.5,
        impedance_percent: 3.0,
        primary_kv: None,
        connection: None,
        mounting: None,
        brand: String::new(),
    });
    project.source.fault.available_ka = Some(5.0);
    let result = calculate(&project).unwrap();
    let study = result.fault.unwrap();
    assert_eq!(study.source.base_ka, 5.0);
    assert_eq!(
        study.source.origin,
        ieestudio_project::fault::FaultOrigin::Supplier
    );
}

/// Un tablero alimentado por un transformador del proyecto toma la falla de
/// ese transformador (45 kVA, 220 V, Z = 3 % → 4.37 kA con la tolerancia);
/// su alimentador se compara con la falla de donde sale.
#[test]
fn a_project_transformer_sets_its_own_fault() {
    let mut project = transformer_project();
    project.source.fault.available_ka = Some(25.0);
    project.panels[0].board.interrupting_ka = Some(25.0);
    project.panels[1].board.interrupting_ka = Some(10.0);
    project.panels[0].feeder.overrides.breaker.interrupting_ka = Some(25.0);
    let result = calculate(&project).unwrap();
    let tf = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    let bus = tf.fault.as_ref().unwrap();
    close(bus.base_ka, 4.3738);
    assert_eq!(
        bus.origin,
        ieestudio_project::fault::FaultOrigin::Transformer {
            panel: "TF-1".into()
        }
    );
    // El primario sale de las barras de TGN (25 kA, cumple con 25 kA del tablero).
    let feeder = checks_named(&tf.feeder, "Capacidad interruptiva")[0];
    assert_eq!(feeder.status, Status::Complies, "{}", feeder.detail);
    assert!(feeder.detail.contains("TGN"));
    // El secundario y los circuitos, con los 4.37 kA del transformador.
    for run in tf.secondary.iter().chain(&tf.circuits) {
        let c = checks_named(run, "Capacidad interruptiva")[0];
        assert_eq!(c.status, Status::Complies, "{}: {}", run.id, c.detail);
    }
}

/// Sin dato de placa, la falla se calcula con la impedancia mínima de la
/// Tabla 11 de la NMX-J-116-ANCE, sin reducirla otra vez: tipo poste trifásico
/// de 112.5 kVA a 13.2 kV, Z = 2 % → 295.24 A / 0.02 = 14.76 kA.
#[test]
fn fault_with_the_minimum_impedance_of_the_standard() {
    let mut project = example::house();
    project.source.substation = Some(ieestudio_project::Substation {
        kva: 112.5,
        impedance_percent: 3.0,
        impedance_from_table: true,
        primary_kv: Some(13.2),
        connection: None,
        mounting: Some(ieestudio_project::Mounting::Pole),
        brand: String::new(),
    });
    let result = calculate(&project).unwrap();
    let study = result
        .fault
        .as_ref()
        .expect("la tabla cubre este transformador");
    close(study.source.base_ka, 14.7618);
    let step = study
        .source
        .steps
        .iter()
        .find(|s| s.title == "Impedancia para la falla")
        .unwrap();
    assert!(step.substitution.contains("2 %"));
    assert!(step.references[0]
        .clause
        .as_deref()
        .unwrap()
        .contains("NMX-J-116"));

    // Tipo pedestal no está en la tabla: no se supone un valor y la capacidad
    // interruptiva vuelve a quedar fuera de alcance.
    let sub = project.source.substation.as_mut().unwrap();
    sub.mounting = Some(ieestudio_project::Mounting::PadMounted);
    let profile = ieestudio_rules::Profile::nom_001_sede_2012();
    assert!(ieestudio_project::fault::substation_impedance(profile, &project.source).is_err());
    let result = calculate(&project).unwrap();
    assert!(result.fault.is_none());
    assert!(!result.verdict.out_of_scope.is_empty());
}

/// Un proyecto guardado con una conexión que su sistema no tiene abre y se
/// calcula, pero ese tramo queda sin evaluar con el motivo; nunca un número.
#[test]
fn an_impossible_connection_is_not_evaluated() {
    use ieestudio_project::connection::{circuit_kinds, feeder_kinds, mismatches};

    let mut project = example::house();
    project.source.system = System::SPLIT_240_120;
    // La casa de ejemplo es trifásica: su alimentador ya no existe.
    assert_eq!(project.panels[0].kind, CircuitKind::ThreePhase);
    let json = project.to_json();
    let project = Project::from_json(&json).unwrap();
    let result = calculate(&project).unwrap();
    let feeder = &result.panels[0].feeder;
    assert!(feeder.run.is_none());
    assert_eq!(feeder.status, Status::NotEvaluated);
    assert!(feeder.checks[0].detail.contains("no tiene tres fases"));

    assert_eq!(
        feeder_kinds(&project, 0),
        vec![CircuitKind::LineToNeutral, CircuitKind::LineToLine]
    );
    let found = mismatches(&project);
    assert_eq!(found[0].circuit, None);
    assert_eq!(found[0].nearest, Some(CircuitKind::LineToLine));

    // Con el alimentador a dos fases, los circuitos de una y dos se calculan.
    let mut fixed = project.clone();
    fixed.panels[0].kind = CircuitKind::LineToLine;
    assert_eq!(
        circuit_kinds(&fixed, 0),
        vec![CircuitKind::LineToNeutral, CircuitKind::LineToLine]
    );
    let result = calculate(&fixed).unwrap();
    assert!(result.panels[0].feeder.run.is_some());
    assert!(mismatches(&fixed).iter().all(|m| m.circuit.is_some()));
}

/// Un circuito no puede tener más fases que su tablero, ni un subtablero más
/// que el tablero que lo alimenta.
#[test]
fn a_run_cannot_have_more_phases_than_its_panel() {
    let mut project = example::house();
    project.panels[0].kind = CircuitKind::LineToLine;
    project.panels[0].circuits[0].kind = CircuitKind::ThreePhase;
    let result = calculate(&project).unwrap();
    let run = &result.panels[0].circuits[0];
    assert!(run.run.is_none());
    assert!(
        run.checks[0].detail.contains("es de dos fases"),
        "{:?}",
        run.checks[0]
    );
}

/// La capacidad interruptiva es una verificación del tablero, no de cada
/// circuito: un circuito bien calculado cumple aunque el interruptor no
/// alcance la falla estimada, y el pendiente se cuenta una vez por lugar.
#[test]
fn interrupting_capacity_is_one_check_per_place() {
    let mut project = example::house();
    project.source.substation = Some(ieestudio_project::Substation {
        impedance_from_table: Default::default(),
        kva: 112.5,
        impedance_percent: 3.0,
        primary_kv: Some(13.2),
        connection: None,
        mounting: None,
        brand: String::new(),
    });
    let before = calculate(&example::house()).unwrap();
    project.panels[0].board.interrupting_ka = Some(10.0);
    let result = calculate(&project).unwrap();
    // El estado de cada tramo es el mismo que sin estudio de falla.
    for (with, without) in result.panels[0].runs().zip(before.panels[0].runs()) {
        assert_eq!(with.status, without.status, "{}", with.id);
        assert!(with.interrupting.is_some());
    }
    let study = result.fault.as_ref().unwrap();
    assert_eq!(study.groups.len(), 2);
    let source = &study.groups[0];
    assert_eq!(source.panel, None);
    assert_eq!((source.breakers.len(), source.missing()), (1, 1));
    assert!(source
        .summary()
        .starts_with("falta la capacidad interruptiva de 1 interruptor"));
    let bars = &study.groups[1];
    assert_eq!(bars.panel.as_deref(), Some("TG-1"));
    assert_eq!(bars.status, Status::NotEvaluated);
    assert_eq!(bars.short(), bars.breakers.len());
    assert!(
        bars.summary().starts_with("no concluyente: los "),
        "{}",
        bars.summary()
    );
    // Dos pendientes por la capacidad interruptiva, no uno por interruptor.
    assert_eq!(
        result.verdict.pending_checks,
        before.verdict.pending_checks + 2
    );

    project.panels[0].board.interrupting_ka = Some(22.0);
    project.panels[0].feeder.overrides.breaker.interrupting_ka = Some(22.0);
    let result = calculate(&project).unwrap();
    let study = result.fault.as_ref().unwrap();
    assert!(study.groups.iter().all(|g| g.status == Status::Complies));
    assert!(study.groups[1]
        .summary()
        .contains("cumple con un método conservador"));
    assert_eq!(result.verdict.pending_checks, before.verdict.pending_checks);
}
