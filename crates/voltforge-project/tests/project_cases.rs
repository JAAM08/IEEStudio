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

    // Alimentador (430-24): 9436.97 VA / (√3 · 220) = 24.77 A, de los que
    // 2000 VA / 381.05 = 5.25 A son continuos. El motor mayor es el
    // minisplit: 2640 / 381.05 = 6.93 A, que suma su 25 %.
    // Id = 1.25 · 5.25 + 19.52 + 0.25 · 6.93 = 27.81 A → 30 A, 10 AWG.
    let feeder = panel.feeder.run.as_ref().unwrap();
    close(feeder.nominal_a, 24.77);
    close(feeder.design_a, 27.81);
    close(feeder.breaker_a, 30.0);
    assert_eq!(feeder.size, Size::new("10"));
    assert_eq!(feeder.poles, 3);
    assert_eq!(feeder.current_carrying_conductors, 3);
    close(feeder.drop_percent, 0.83);
    assert_eq!(feeder.conduit.as_ref().unwrap().metric_designator, 16);
    assert_eq!(panel.feeder.status, Status::Complies);

    // C-01: 10.5 A continuos, 15 A, 14 AWG, 2.76 %; acumulada con el alimentador.
    let c01 = circuit(&panel.circuits, "C-01");
    let run = c01.run.as_ref().unwrap();
    close(run.breaker_a, 15.0);
    assert_eq!(run.size, Size::new("14"));
    close(run.drop_percent, 2.76);
    close(run.cumulative_drop_percent, 2.76 + 0.83);
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
        Project::from_json(&json.replace("\"format_version\": 2", "\"format_version\": 99"))
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
        assert_eq!(run.steps.len(), 12, "{}", item.id);
        assert!(
            [
                "Corriente nominal",
                "Corriente del motor",
                "Corriente del compresor"
            ]
            .contains(&run.steps[0].title.as_str()),
            "{}",
            item.id
        );
        assert_eq!(run.steps[11].title, "Caída acumulada desde la fuente");
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
            demand_factor: 1.0,
            has_neutral: false,
            overrides: Overrides::default(),
        },
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
