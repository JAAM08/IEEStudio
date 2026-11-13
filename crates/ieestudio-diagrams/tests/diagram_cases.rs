//! El unifilar cubre todos los tramos del proyecto.

use ieestudio_core::{CircuitKind, System};
use ieestudio_diagrams::{plan, single_line, Diagram, Element, Ink};
use ieestudio_project::{
    calculate, example, Circuit, Feeder, Load, LoadKind, Overrides, Panel, Project,
};

fn industrial() -> Project {
    let mut project = example::house();
    project.name = "Nave".to_string();
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
                length_m: Some(40.0),
                demand: ieestudio_project::Demand::Custom(0.8),
                demand_factor: None,
                has_neutral: true,
                overrides: Overrides {
                    // Dato declarado del caso: carga lineal (decisión 0030, T-6).
                    nonlinear: Some(false),
                    ..Overrides::default()
                },
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
                length_m: Some(60.0),
                demand: ieestudio_project::Demand::Article220,
                demand_factor: None,
                has_neutral: false,
                overrides: Overrides {
                    // Dato declarado del caso: carga lineal (decisión 0030, T-6).
                    nonlinear: Some(false),
                    ..Overrides::default()
                },
            },
            transformer: None,
            areas: vec![],
            circuits: vec![Circuit {
                id: "F-01".to_string(),
                name: "Horno".to_string(),
                kind: CircuitKind::ThreePhase,
                phases: vec![],
                load: Load {
                    power_w: Some(90_000.0),
                    power_factor: 1.0,
                    continuous: true,
                    kind: LoadKind::General,
                    outlets: None,
                    motor: None,
                    compressor: None,
                    confirmed_general: false,
                },
                length_m: Some(25.0),
                has_neutral: false,
                overrides: Overrides {
                    // Dato declarado del caso: carga lineal (decisión 0030, T-6).
                    nonlinear: Some(false),
                    ..Overrides::default()
                },
            }],
        },
    ];
    project
}

fn texts(d: &Diagram) -> Vec<String> {
    d.elements
        .iter()
        .filter_map(|e| match e {
            Element::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// El unifilar dibuja un tramo por circuito y uno por alimentador, con su
/// interruptor, sus conductores separados por función (requisito I.3), su
/// tubería, corriente y caída; y los subtableros cuelgan de la barra de su
/// tablero.
#[test]
fn single_line_diagram_covers_every_run() {
    let house = example::house();
    let diagram = single_line(&house, &calculate(&house).unwrap());
    let t = texts(&diagram);
    for id in ["C-01", "C-02", "C-03", "C-04", "C-05", "C-06"] {
        assert!(t.iter().any(|s| s == id), "falta {id}");
    }
    assert!(t.iter().any(|s| s.starts_with("ALIM. TG-1 · L = ")));
    // C-02: interruptor de 20 A, fase y neutro de 10 AWG y tierra desnuda de 10.
    assert!(t.iter().any(|s| s == "1x20 A"));
    assert!(t.iter().any(|s| s == "1-10 AWG 1F Cu THW"));
    assert!(t.iter().any(|s| s == "1-10 AWG 1N Cu THW"));
    assert!(t.iter().any(|s| s == "1-10 AWG CPT Cu desnudo"));
    // El alimentador trifásico con neutro: tres fases y un neutro.
    assert!(t.iter().any(|s| s == "3-10 AWG 3F Cu THW"));
    assert!(t.iter().any(|s| s.starts_with("In: ")));
    assert!(t.iter().any(|s| s.starts_with("TUBO EMT 16 (½")));
    // Encabezado del tablero con carga instalada y demandada.
    assert!(t.iter().any(|s| s.ends_with("INSTALADOS")));
    assert!(t.iter().any(|s| s.contains("DEMANDADOS, F.D.")));
    assert!(t.iter().any(|s| s == "220/127 V, 3F-4H, 60 Hz"));
    assert!(t.iter().any(|s| s == "ACOMETIDA EN BAJA TENSIÓN · CFE"));
    // Una zona por circuito y una por tablero, para seleccionar desde el diagrama.
    assert_eq!(diagram.hotspots.len(), 7);
    assert!(diagram.hotspots.iter().any(|h| h.circuit == Some(4)));
    // El tablero va en un recuadro punteado.
    assert!(diagram
        .elements
        .iter()
        .any(|e| matches!(e, Element::Rect { dashed: true, .. })));
    let svg = diagram.to_svg();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("stroke-dasharray"));

    let nave = industrial();
    let diagram = single_line(&nave, &calculate(&nave).unwrap());
    let t = texts(&diagram);
    assert!(t.iter().any(|s| s.starts_with("ALIM. TGN · ")));
    // El subtablero se alimenta desde una columna del tablero general.
    assert!(t.iter().any(|s| s.starts_with("ALIM. TF-1 · ")));
    assert!(t.iter().any(|s| s == "F-01"));
    assert!(t.iter().any(|s| s == "480/277 V, 3F-4H, 60 Hz"));
    // El subtablero se selecciona desde su columna y desde su propio dibujo.
    let tf = nave.panels.iter().position(|p| p.id == "TF-1").unwrap();
    assert!(
        diagram
            .hotspots
            .iter()
            .filter(|h| h.panel == tf && h.circuit.is_none())
            .count()
            >= 2
    );
    if let Ok(dir) = std::env::var("IEESTUDIO_SVG_OUT") {
        std::fs::write(
            format!("{dir}/nave.svg"),
            plan(&nave, &calculate(&nave).unwrap()).to_svg(),
        )
        .unwrap();
        std::fs::write(
            format!("{dir}/casa.svg"),
            plan(&house, &calculate(&house).unwrap()).to_svg(),
        )
        .unwrap();
        std::fs::write(
            format!("{dir}/casa-revision.svg"),
            single_line(&house, &calculate(&house).unwrap()).to_svg(),
        )
        .unwrap();
        std::fs::write(format!("{dir}/nave.json"), nave.to_json()).unwrap();
    }
}

/// El plano va en blanco y negro, con la simbología de lo que aparece y el
/// cajetín con los datos de la obra.
#[test]
fn plan_is_black_and_white_with_its_sheet() {
    let mut house = example::house();
    house.info.client = "Familia Pérez".to_string();
    let result = calculate(&house).unwrap();
    let diagram = plan(&house, &result);
    for e in &diagram.elements {
        if let Element::Text { ink, .. } = e {
            assert!(
                matches!(ink, Ink::Text | Ink::Muted | Ink::Accent),
                "el plano no lleva colores de estado"
            );
        }
    }
    let t = texts(&diagram);
    for expected in [
        "SIMBOLOGÍA (NMX-J-136-ANCE)",
        "INTERRUPTOR TERMOMAGNÉTICO (AUTOMÁTICO)",
        "EQUIPO DE MEDICIÓN",
        "MOTOR ELÉCTRICO",
        "RECEPTÁCULO (CONTACTOS)",
        "SALIDA ESPECIAL: AIRE ACONDICIONADO",
        "DIAGRAMA UNIFILAR",
        "Familia Pérez",
        "NOTAS",
    ] {
        assert!(t.iter().any(|s| s == expected), "falta {expected}");
    }
    // Sin subtableros ni equipo de cocina, sus símbolos no se listan.
    assert!(!t.iter().any(|s| s == "SALIDA ESPECIAL: EQUIPO DE COCINA"));
    // Los nombres van en mayúsculas, como en un plano.
    assert!(t.iter().any(|s| s == "BOMBA DE AGUA"));
    // La revisión sí colorea según el estado.
    let review = single_line(&house, &result);
    assert!(review.elements.iter().any(|e| matches!(
        e,
        Element::Text {
            ink: Ink::Complies | Ink::Fails | Ink::NotEvaluated,
            ..
        }
    )));
    assert!(diagram.width > review.width);
}

/// Con muchos circuitos la barra sigue en otra fila en lugar de crecer sin
/// fin a lo ancho.
#[test]
fn many_circuits_wrap_into_rows() {
    let mut house = example::house();
    let base = house.panels[0].circuits[0].clone();
    for n in 0..30 {
        let mut c = base.clone();
        c.id = format!("X-{n:02}");
        house.panels[0].circuits.push(c);
    }
    let result = calculate(&house).unwrap();
    let wide = single_line(&house, &result);
    let few = single_line(&example::house(), &calculate(&example::house()).unwrap());
    assert!(wide.width < few.width * 3.0, "ancho {}", wide.width);
    assert!(wide.height > few.height * 1.6);
    assert_eq!(wide.hotspots.len(), 1 + 36);
}

/// El DXF lleva las mismas líneas y textos que el diagrama, con el eje Y
/// hacia arriba, en Windows-1252.
#[test]
fn dxf_has_every_line_and_text() {
    let project = ieestudio_project::example::house();
    let result = ieestudio_project::calculate(&project).unwrap();
    let diagram = ieestudio_diagrams::plan(&project, &result);
    let dxf = diagram.to_dxf();
    // El límite de los tableros va con línea discontinua.
    let text_all: String = dxf.iter().map(|&b| char::from(b)).collect();
    assert!(text_all.contains("\n  6\nDASHED\n"));
    // Windows-1252: «á» es un byte, no dos.
    assert!(dxf.contains(&0xE1) || dxf.contains(&0xF3) || dxf.iter().all(|b| *b < 0x80));
    let text: String = dxf.iter().map(|&b| char::from(b)).collect();
    assert!(text.starts_with("  0\nSECTION\n  2\nHEADER"));
    assert!(text.trim_end().ends_with("EOF"));
    let texts = diagram
        .elements
        .iter()
        .filter(|e| matches!(e, ieestudio_diagrams::Element::Text { .. }))
        .count();
    assert_eq!(text.matches("\n  0\nTEXT\n").count(), texts);
    let segments: usize = diagram
        .elements
        .iter()
        .map(|e| match e {
            ieestudio_diagrams::Element::Path { points, .. } => points.len() - 1,
            ieestudio_diagrams::Element::Rect { .. } => 4,
            ieestudio_diagrams::Element::Text { .. } => 0,
        })
        .sum();
    assert_eq!(text.matches("\n  0\nLINE\n").count(), segments);
    // Los caracteres fuera de Windows-1252 van escapados.
    assert!(!text.contains('Σ'));
}

/// Con los datos de catálogo y de la subestación, el unifilar lleva la
/// subestación (I.2), los interruptores con marco, disparo, modelo, kA y
/// marca (I.4), el tablero con sus barras, la corriente de falla y la
/// posición de cada interruptor.
#[test]
fn catalog_data_and_fault_current_are_drawn() {
    let mut house = example::house();
    house.source.substation = Some(ieestudio_project::Substation {
        impedance_from_table: Default::default(),
        kva: Some(112.5),
        impedance_percent: Some(3.0),
        primary_kv: Some(13.2),
        connection: Some(ieestudio_project::Connection::DeltaWye),
        mounting: Some(ieestudio_project::Mounting::Pole),
        brand: "Imem".into(),
    });
    let panel = &mut house.panels[0];
    panel.board.bus_a = Some(225.0);
    panel.board.interrupting_ka = Some(22.0);
    panel.board.model = "NQ304AB225S".into();
    panel.board.brand = "Square D".into();
    let breaker = &mut panel.circuits[1].overrides.breaker;
    breaker.frame_a = Some(100.0);
    breaker.model = "QOB120".into();
    breaker.brand = "Square D".into();
    let result = calculate(&house).unwrap();
    let diagram = plan(&house, &result);
    let t = texts(&diagram);
    for expected in [
        "TRANSFORMADOR DE 112.5 kVA, Z = 3 %",
        "13.2 kV – 220/127 V, 3F-4H, 60 Hz",
        "DELTA – ESTRELLA · TIPO POSTE · MARCA IMEM",
        "ACOMETIDA EN MEDIA TENSIÓN, 13.2 kV · CFE",
        "BARRAS 225 A · 22 kA · MOD. NQ304AB225S · SQUARE D",
        "1x100 A AM",
        "1x20 A AD",
        "QOB120",
        "22 kA · SQUARE D",
    ] {
        assert!(t.iter().any(|s| s == expected), "falta {expected}");
    }
    assert!(t
        .iter()
        .any(|s| s.starts_with("Icc = ") && s.ends_with("kA (BUS INFINITO)")));
    assert!(t
        .iter()
        .any(|s| s.contains("AM: amperes de marco · AD: amperes de disparo")));
    // Posiciones de los interruptores en el tablero.
    assert!(t.iter().any(|s| s == "1"));
    // Sin datos de la fuente, la nota dice que no se verificó.
    let plain = plan(&example::house(), &calculate(&example::house()).unwrap());
    assert!(texts(&plain)
        .iter()
        .any(|s| s.contains("Capacidad interruptiva no verificada")));
    if let Ok(dir) = std::env::var("IEESTUDIO_SVG_OUT") {
        std::fs::write(format!("{dir}/casa-catalogo.svg"), diagram.to_svg()).unwrap();
    }
}
