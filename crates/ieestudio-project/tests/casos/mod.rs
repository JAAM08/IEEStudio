//! Proyectos de los hallazgos de las auditorías del PR #11
//! (`docs/auditoria-fable-pr11.md`, H-01 a H-18, y
//! `docs/auditoria-modelo-y-falla.md`, pendientes 17 a 20).
//!
//! Cada proyecto se guarda también como `.iees` en
//! `fixtures/auditoria-pr11/`, para abrirlo en la aplicación o en la línea de
//! comandos y ver el hallazgo sin programar. Si se cambia un proyecto aquí,
//! se regeneran los archivos con
//! `IEES_REGENERAR=1 cargo test -p ieestudio-project --test auditoria_pr11`.
//!
//! Los datos de cada caso son los del informe de Fable; ninguno es un valor
//! normativo nuevo.

#![allow(dead_code)]

use ieestudio_core::{CircuitKind, System};
use ieestudio_project::{
    example, Area, Circuit, Connection, Demand, FaultLocation, Feeder, Load, LoadKind, Mounting,
    Overrides, Panel, Phase, Project, Source, Substation, Transformer,
};

/// Circuito de alumbrado: no aporta a la falla y siempre se calcula.
pub fn lighting(id: &str, kind: CircuitKind, phases: &[Phase], watts: f64) -> Circuit {
    Circuit {
        id: id.into(),
        name: format!("Alumbrado {id}"),
        kind,
        phases: phases.to_vec(),
        load: Load {
            power_w: watts,
            power_factor: 1.0,
            continuous: false,
            kind: LoadKind::Lighting,
            outlets: None,
            motor: None,
            compressor: None,
            confirmed_general: false,
        },
        length_m: 10.0,
        has_neutral: kind == CircuitKind::LineToNeutral,
        overrides: Overrides::default(),
    }
}

/// Circuito de motor trifásico con todos sus datos.
pub fn motor(id: &str, name: &str, hp: &str) -> Circuit {
    Circuit {
        id: id.into(),
        name: name.into(),
        kind: CircuitKind::ThreePhase,
        phases: vec![],
        load: Load {
            power_w: 0.0,
            power_factor: 0.85,
            continuous: false,
            kind: LoadKind::Motor,
            outlets: None,
            motor: Some(ieestudio_project::MotorLoad {
                hp: hp.into(),
                motor_type: Default::default(),
                nameplate_a: None,
                service_factor_marked: true,
            }),
            compressor: None,
            confirmed_general: false,
        },
        length_m: 15.0,
        has_neutral: false,
        overrides: Overrides::default(),
    }
}

/// Un tablero `TG` alimentado de la fuente, sin circuitos.
pub fn board(id: &str, parent: Option<&str>, kind: CircuitKind) -> Panel {
    Panel {
        board: Default::default(),
        interrupting_ka: None,
        id: id.into(),
        name: format!("Tablero {id}"),
        parent: parent.map(str::to_string),
        kind,
        feeder: Feeder {
            interrupting_ka: None,
            length_m: 15.0,
            demand: Demand::Full,
            demand_factor: None,
            has_neutral: true,
            overrides: Overrides::default(),
        },
        // Área para la carga mínima de alumbrado (220-12): sin ella la carga
        // del alimentador queda sin evaluar y el dictamen nunca es conforme.
        areas: vec![Area {
            description: "Área de prueba".into(),
            occupancy: None,
            m2: 10.0,
        }],
        transformer: None,
        circuits: vec![],
    }
}

/// Proyecto de un tablero con la fuente dada y dos circuitos de alumbrado.
pub fn single_board(system: System, kind: CircuitKind) -> Project {
    let mut project = example::house();
    project.name = "Caso de auditoría".into();
    project.info = Default::default();
    project.source = Source::new("Acometida", system);
    project.source.supplier = "CFE".into();
    let mut tg = board("TG", None, kind);
    let one = if system.line_to_neutral_v.is_some() {
        CircuitKind::LineToNeutral
    } else {
        CircuitKind::LineToLine
    };
    let phases = if one == CircuitKind::LineToNeutral {
        (vec![Phase::A], vec![Phase::B])
    } else {
        (vec![Phase::A, Phase::B], vec![Phase::A, Phase::B])
    };
    tg.circuits = vec![
        lighting("A-1", one, &phases.0, 800.0),
        lighting("A-2", one, &phases.1, 800.0),
    ];
    if kind == CircuitKind::ThreePhase && one == CircuitKind::LineToNeutral {
        tg.circuits.push(lighting("A-3", one, &[Phase::C], 800.0));
    }
    project.panels = vec![tg];
    project
}

/// Capacidad interruptiva de todos los interruptores de un tablero raíz.
pub fn breakers(project: &mut Project, panel: usize, ka: f64) {
    let p = &mut project.panels[panel];
    p.board.interrupting_ka = Some(ka);
    p.feeder.overrides.breaker.interrupting_ka = Some(ka);
    if let Some(t) = p.transformer.as_mut() {
        t.secondary_overrides.breaker.interrupting_ka = Some(ka);
    }
}

pub fn substation(kva: f64, z: Option<f64>, connection: Connection) -> Substation {
    Substation {
        kva: Some(kva),
        impedance_percent: z,
        impedance_from_table: z.is_none(),
        primary_kv: Some(13.2),
        connection: Some(connection),
        mounting: Some(Mounting::Pole),
        brand: String::new(),
    }
}

pub fn transformer(kva: Option<f64>, z: f64, secondary: System) -> Transformer {
    Transformer {
        kva,
        impedance_percent: Some(z),
        secondary,
        secondary_protection: true,
        secondary_length_m: Some(3.0),
        secondary_has_neutral: secondary.line_to_neutral_v.is_some(),
        secondary_overrides: Overrides::default(),
        confirmed_distinct: false,
        connection: None,
        mounting: None,
        brand: String::new(),
    }
}

// ---------------------------------------------------------------- casos

/// H-01: acometida 1F-3H 240/120 V con transformador de 25 kVA y 3 %,
/// interruptores de 5 kA.
pub fn h01_acometida_240_120() -> Project {
    let mut p = single_board(System::SPLIT_240_120, CircuitKind::LineToLine);
    p.source.substation = Some(substation(25.0, Some(3.0), Connection::SinglePhase));
    breakers(&mut p, 0, 5.0);
    p
}

/// H-01: intermedio seco 480 → 240/120 V de 25 kVA y 3 %. TGN con el dato
/// de la suministradora (25 kA, interruptores de 35 kA); TF-1 con 5 kA.
pub fn h01_intermedio_240_120() -> Project {
    let mut p = single_board(System::WYE_480_277, CircuitKind::ThreePhase);
    p.source.fault.available_ka = Some(25.0);
    p.source.fault.location = Some(FaultLocation::LowVoltage);
    breakers(&mut p, 0, 35.0);
    let mut tf = board("TF-1", Some("TG"), CircuitKind::LineToLine);
    tf.transformer = Some(transformer(Some(25.0), 3.0, System::SPLIT_240_120));
    tf.circuits = vec![
        lighting("B-1", CircuitKind::LineToNeutral, &[Phase::A], 800.0),
        lighting("B-2", CircuitKind::LineToNeutral, &[Phase::B], 800.0),
    ];
    p.panels.push(tf);
    breakers(&mut p, 1, 5.0);
    // El alimentador de TF-1 sale de las barras de TG (25 kA).
    p.panels[1].feeder.overrides.breaker.interrupting_ka = Some(35.0);
    p
}

/// H-02: subestación propia de 1000 kVA, 5.75 %, 13.2 kV a 480Y/277 V, con
/// un dato de la suministradora de `available_ka` en `location` (`None`:
/// sin decir dónde).
pub fn h02_subestacion_con_dato(
    available_ka: f64,
    location: Option<FaultLocation>,
    breaker_ka: f64,
) -> Project {
    let mut p = single_board(System::WYE_480_277, CircuitKind::ThreePhase);
    let mut sub = substation(1000.0, Some(5.75), Connection::DeltaWye);
    sub.mounting = Some(Mounting::Substation);
    p.source.substation = Some(sub);
    p.source.fault.available_ka = Some(available_ka);
    p.source.fault.location = location;
    breakers(&mut p, 0, breaker_ka);
    p
}

/// H-03: dos «Bomba 50 hp» a 220Y/127 V, con dato de 9.8 kA e interruptores
/// de 10 kA. `as_motor` las captura como motor; si no, como «Otra carga».
pub fn h03_bombas(as_motor: bool) -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    p.source.fault.available_ka = Some(9.8);
    p.source.fault.location = Some(FaultLocation::LowVoltage);
    for id in ["M-1", "M-2"] {
        let mut c = motor(id, "Bomba 50 hp", "50");
        if !as_motor {
            c.load = Load {
                power_w: 37_300.0,
                power_factor: 0.85,
                continuous: false,
                kind: LoadKind::General,
                outlets: None,
                motor: None,
                compressor: None,
                confirmed_general: false,
            };
        }
        p.panels[0].circuits.push(c);
    }
    breakers(&mut p, 0, 10.0);
    p
}

/// H-03: las mismas «Otras cargas» con «Es otra carga» confirmado.
pub fn h03_bombas_confirmadas() -> Project {
    let mut p = h03_bombas(false);
    for c in &mut p.panels[0].circuits {
        if c.id.starts_with("M-") {
            c.load.confirmed_general = true;
        }
    }
    p
}

/// H-04: sistema 226/120 V (relación 1.88), transformador de 25 kVA y 3 %.
pub fn h04_relacion_1_88() -> Project {
    let system = System {
        line_to_line_v: Some(226.0),
        line_to_neutral_v: Some(120.0),
    };
    let mut p = single_board(system, CircuitKind::LineToLine);
    p.source.substation = Some(substation(25.0, Some(3.0), Connection::SinglePhase));
    breakers(&mut p, 0, 5.0);
    p
}

/// H-05: transformador de la acometida de 75 kVA marcado «Monofásico» en un
/// sistema 220Y/127 V, con la impedancia mínima de la tabla.
pub fn h05_conexion_contradictoria() -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    p.source.substation = Some(substation(75.0, None, Connection::SinglePhase));
    breakers(&mut p, 0, 22.0);
    p
}

/// H-05: transformador de la acometida de 112.5 kVA con impedancia de 30 %
/// (capturada por 3.0).
pub fn h05_impedancia_30() -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    p.source.substation = Some(substation(112.5, Some(30.0), Connection::DeltaWye));
    breakers(&mut p, 0, 10.0);
    p
}

/// H-06: 300 kW en un transformador de acometida de 75 kVA.
pub fn h06_acometida_sobrecargada() -> Project {
    let mut p = single_board(System::WYE_480_277, CircuitKind::ThreePhase);
    p.source.substation = Some(substation(75.0, Some(3.0), Connection::DeltaWye));
    let mut big = lighting("G-1", CircuitKind::ThreePhase, &[], 300_000.0);
    big.name = "Carga de proceso".into();
    big.load.kind = LoadKind::General;
    p.panels[0].circuits.push(big);
    breakers(&mut p, 0, 22.0);
    p
}

/// H-07, H-09, H-10: acometida de CFE de 112.5 kVA, 3 %, 220Y/127 V, todo
/// cumple con interruptores de 22 kA. `from_table` usa la impedancia mínima
/// de la tabla en lugar de la de placa.
pub fn h07_todo_cumple(from_table: bool) -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    let z = if from_table { None } else { Some(3.0) };
    p.source.substation = Some(substation(112.5, z, Connection::DeltaWye));
    breakers(&mut p, 0, 22.0);
    p
}

/// H-09: el proyecto anterior con un transformador intermedio al que aún no
/// se le captura la potencia.
pub fn h09_transformador_sin_potencia() -> Project {
    let mut p = h07_todo_cumple(false);
    let mut tf = board("TF-1", Some("TG"), CircuitKind::ThreePhase);
    tf.transformer = Some(transformer(None, 3.0, System::WYE_480_277));
    tf.circuits = vec![lighting(
        "B-1",
        CircuitKind::LineToNeutral,
        &[Phase::A],
        800.0,
    )];
    p.panels.push(tf);
    p
}

/// H-10 y H-11: dato de la suministradora inválido (0 kA), sin transformador.
pub fn h11_dato_invalido() -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    p.source.fault.available_ka = Some(0.0);
    p.source.fault.location = Some(FaultLocation::LowVoltage);
    breakers(&mut p, 0, 10.0);
    p
}

/// H-12: motor con datos detrás de un transformador intermedio sin potencia.
pub fn h12_motor_detras_de_transformador_sin_datos() -> Project {
    let mut p = single_board(System::WYE_480_277, CircuitKind::ThreePhase);
    p.source.fault.available_ka = Some(25.0);
    p.source.fault.location = Some(FaultLocation::LowVoltage);
    breakers(&mut p, 0, 35.0);
    let mut tf = board("TF-1", Some("TG"), CircuitKind::ThreePhase);
    tf.transformer = Some(transformer(None, 3.0, System::WYE_220_127));
    tf.circuits = vec![motor("M-1", "Motor 10 hp", "10")];
    p.panels.push(tf);
    breakers(&mut p, 1, 10.0);
    p.panels[1].feeder.overrides.breaker.interrupting_ka = Some(35.0);
    p
}

/// H-16: transformador de aislamiento 127 → 127 V desde 220Y/127 V.
pub fn h16_aislamiento_127() -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    let mut tf = board("TA-1", Some("TG"), CircuitKind::LineToNeutral);
    tf.transformer = Some(transformer(Some(3.0), 3.0, System::SINGLE_127));
    tf.circuits = vec![lighting(
        "C-1",
        CircuitKind::LineToNeutral,
        &[Phase::A],
        300.0,
    )];
    p.panels.push(tf);
    breakers(&mut p, 0, 22.0);
    breakers(&mut p, 1, 10.0);
    p
}

/// Pendiente 17: transformador elevador de 127 V a 240/120 V. El tablero
/// se alimenta con una fase y neutro, pero su secundario tiene dos fases y
/// debería admitir un circuito de 240 V.
pub fn p17_secundario_con_otra_conexion() -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    let mut tf = board("TF-1", Some("TG"), CircuitKind::LineToNeutral);
    tf.transformer = Some(transformer(Some(5.0), 3.0, System::SPLIT_240_120));
    tf.circuits = vec![lighting(
        "D-1",
        CircuitKind::LineToLine,
        &[Phase::A, Phase::B],
        1200.0,
    )];
    p.panels.push(tf);
    breakers(&mut p, 0, 22.0);
    breakers(&mut p, 1, 10.0);
    p
}

/// Pendiente 19: un circuito con longitud de 0 m.
pub fn p19_longitud_cero() -> Project {
    let mut p = single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    p.panels[0].circuits[0].length_m = 0.0;
    p
}

/// Todos los casos con el nombre de su archivo en `fixtures/auditoria-pr11/`.
pub fn all() -> Vec<(&'static str, Project)> {
    vec![
        ("h01-acometida-240-120.iees", h01_acometida_240_120()),
        ("h01-intermedio-240-120.iees", h01_intermedio_240_120()),
        (
            "h02-dato-media-tension-5ka.iees",
            h02_subestacion_con_dato(5.0, None, 10.0),
        ),
        (
            "h02-dato-en-primario-5ka.iees",
            h02_subestacion_con_dato(5.0, Some(FaultLocation::Primary), 10.0),
        ),
        (
            "h02-dato-mayor-que-bus-infinito.iees",
            h02_subestacion_con_dato(50.0, Some(FaultLocation::LowVoltage), 65.0),
        ),
        ("h03-bombas-como-otra-carga.iees", h03_bombas(false)),
        ("h03-bombas-como-motor.iees", h03_bombas(true)),
        (
            "h03-bombas-confirmadas-otra-carga.iees",
            h03_bombas_confirmadas(),
        ),
        ("h04-relacion-1-88.iees", h04_relacion_1_88()),
        (
            "h05-conexion-contradictoria.iees",
            h05_conexion_contradictoria(),
        ),
        ("h05-impedancia-30.iees", h05_impedancia_30()),
        (
            "h06-acometida-sobrecargada.iees",
            h06_acometida_sobrecargada(),
        ),
        ("h07-todo-cumple.iees", h07_todo_cumple(false)),
        ("h07-todo-cumple-tabla.iees", h07_todo_cumple(true)),
        (
            "h09-transformador-sin-potencia.iees",
            h09_transformador_sin_potencia(),
        ),
        ("h11-dato-invalido.iees", h11_dato_invalido()),
        (
            "h12-motor-detras-de-transformador-sin-datos.iees",
            h12_motor_detras_de_transformador_sin_datos(),
        ),
        ("h16-aislamiento-127.iees", h16_aislamiento_127()),
        (
            "p17-secundario-con-otra-conexion.iees",
            p17_secundario_con_otra_conexion(),
        ),
        ("p19-longitud-cero.iees", p19_longitud_cero()),
    ]
}
