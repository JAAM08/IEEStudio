//! Cálculo de proyectos completos. Los valores esperados se calcularon a
//! mano con las mismas tablas que usa el motor.

use voltforge_core::{CircuitKind, Size, System};
use voltforge_project::{
    calculate, example, Area, Circuit, Demand, Feeder, Load, LoadKind, Overrides, Panel, Project,
    RunResult, SourceFault, Status,
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
            id: "TGN".to_string(),
            name: "Tablero general".to_string(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 40.0,
                demand: Demand::Custom(0.8),
                demand_factor: None,
                has_neutral: true,
                interrupting_ka: None,
                overrides: Overrides::default(),
            },
            interrupting_ka: None,
            areas: vec![],
            circuits: vec![],
        },
        Panel {
            id: "TF-1".to_string(),
            name: "Tablero de fuerza".to_string(),
            parent: Some("TGN".to_string()),
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 60.0,
                demand: Demand::Article220,
                demand_factor: None,
                has_neutral: false,
                interrupting_ka: None,
                overrides: Overrides::default(),
            },
            interrupting_ka: None,
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
    // Sin datos de cortocircuito: aquí solo interesa el caso que no cubren
    // las tablas.
    project.source.fault = Default::default();
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
        // El alimentador empieza con su carga según el artículo 220 y
        // termina con el cortocircuito.
        let steps: Vec<_> = run
            .steps
            .iter()
            .filter(|s| !s.title.starts_with("Carga") && !s.title.starts_with("Cortocircuito"))
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
    use voltforge_project::verdict::{Calculation, Compliance};

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
            motor: Some(voltforge_project::MotorLoad {
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
        id: "TM".to_string(),
        name: "Tablero de motores".to_string(),
        parent: None,
        kind: CircuitKind::ThreePhase,
        feeder: Feeder {
            length_m: 10.0,
            demand: Demand::Article220,
            demand_factor: None,
            has_neutral: false,
            interrupting_ka: Some(10.0),
            overrides: Overrides::default(),
        },
        interrupting_ka: Some(10.0),
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

/// 110-9: cada interruptor se compara con la corriente de falla en sus
/// terminales de línea: el del alimentador, con la del punto de donde sale; los
/// de los circuitos, con la de su tablero.
#[test]
fn interrupting_rating_follows_the_fault_downstream() {
    let mut project = industrial();
    project.source.fault = SourceFault {
        available_ka: Some(25.0),
        ..SourceFault::default()
    };
    project.panels[0].feeder.interrupting_ka = Some(35.0);
    project.panels[0].interrupting_ka = Some(22.0);
    let result = calculate(&project).unwrap();
    close(result.source_fault_ka.unwrap(), 25.0);
    let general = result.panels.iter().find(|p| p.id == "TGN").unwrap();
    let force = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    let g = general.fault_ka.unwrap();
    let f = force.fault_ka.unwrap();
    assert!(g < 25.0 && f < g, "{g} {f}");
    let rating = |item: &RunResult| {
        item.checks
            .iter()
            .find(|c| c.name == "Capacidad interruptiva")
            .unwrap()
            .clone()
    };
    // El general sale de la fuente con 25 kA: 35 kA cumple.
    assert_eq!(rating(&general.feeder).status, Status::Complies);
    close(general.feeder.run.as_ref().unwrap().fault_ka.unwrap(), 25.0);
    // El de fuerza sale del general: toma sus 22 kA.
    assert_eq!(rating(&force.feeder).status, Status::Complies);
    close(force.feeder.run.as_ref().unwrap().fault_ka.unwrap(), g);
    // Los circuitos del tablero de fuerza no tienen capacidad capturada.
    let oven = circuit(&force.circuits, "F-01");
    assert_eq!(rating(oven).status, Status::NotEvaluated);
    assert!(rating(oven).detail.contains("falta la capacidad"));

    // Con interruptores de menos capacidad que la falla del tablero general,
    // el alimentador de fuerza no cumple.
    project.panels[0].interrupting_ka = Some(g / 2.0);
    project.panels[1].interrupting_ka = Some(65.0);
    let result = calculate(&project).unwrap();
    let force = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    assert_eq!(rating(&force.feeder).status, Status::Fails);
    assert_eq!(
        rating(circuit(&force.circuits, "F-01")).status,
        Status::Complies
    );

    // Sin datos de la fuente solo el alimentador principal lo señala.
    project.source.fault = SourceFault::default();
    let result = calculate(&project).unwrap();
    let general = result.panels.iter().find(|p| p.id == "TGN").unwrap();
    let force = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    assert_eq!(rating(&general.feeder).status, Status::NotEvaluated);
    assert!(force
        .feeder
        .checks
        .iter()
        .all(|c| c.name != "Capacidad interruptiva"));
    assert!(result.source_fault_ka.is_none());
}

/// La corriente en el tablero sale de la impedancia de la fuente más la del
/// alimentador, con la resistencia a 25 °C, más la aportación de motores.
#[test]
fn bus_fault_adds_the_feeder_impedance() {
    let result = calculate(&example::house()).unwrap();
    let panel = &result.panels[0];
    // Fuente: 75 kVA, 3.5 %, 220 V: 5623.54 A; Zf = j 127.017 / 5623.54.
    // Alimentador: 12 m de 10 AWG en acero, R = 3.9 Ω/km a 75 °C
    // (3.2702 a 25 °C), X = 0.207 Ω/km: 0.039242 + j 0.002484 Ω.
    // Icc = 127.017 / |0.039242 + j 0.025071| = 2727.5 A, más 4 veces la
    // corriente de los motores: 4 · 3770.3 / (3 · 127.017) = 39.6 A.
    close(result.source_fault_ka.unwrap(), 5.66314);
    close(panel.fault_ka.unwrap(), 2.76722);
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
