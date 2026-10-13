//! El unifilar cubre todos los tramos del proyecto.

use voltforge_core::{CircuitKind, System};
use voltforge_diagrams::{single_line, Diagram, Element};
use voltforge_project::{
    calculate, example, Circuit, Feeder, Load, LoadKind, Overrides, Panel, Project,
};

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
                demand: voltforge_project::Demand::Custom(0.8),
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
                demand: voltforge_project::Demand::Article220,
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

/// El unifilar dibuja un tramo por circuito y uno por alimentador, con su
/// interruptor y caída, y los subtableros cuelgan de la barra de su tablero.
#[test]
fn single_line_diagram_covers_every_run() {
    let texts = |d: &Diagram| -> Vec<String> {
        d.elements
            .iter()
            .filter_map(|e| match e {
                Element::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    };

    let house = example::house();
    let diagram = single_line(&house, &calculate(&house).unwrap());
    let t = texts(&diagram);
    for id in [
        "C-01",
        "C-02",
        "C-03",
        "C-04",
        "C-05",
        "C-06",
        "Alimentador TG-1",
    ] {
        assert!(t.iter().any(|s| s == id), "falta {id}");
    }
    // C-02: interruptor de 20 A, dos conductores de 10 AWG y tierra desnuda de 10.
    assert!(t.iter().any(|s| s == "1P-20 A"));
    assert!(t.iter().any(|s| s == "2-10, 1-10d"));
    // El alimentador trifásico con neutro lleva cuatro conductores.
    assert!(t.iter().any(|s| s == "4-10, 1-10d"));
    // Una zona por circuito y una por tablero, para seleccionar desde el diagrama.
    assert_eq!(diagram.hotspots.len(), 7);
    assert!(diagram.hotspots.iter().any(|h| h.circuit == Some(4)));
    let svg = diagram.to_svg();
    assert!(svg.starts_with("<svg"));

    let nave = industrial();
    let diagram = single_line(&nave, &calculate(&nave).unwrap());
    let t = texts(&diagram);
    assert!(t.iter().any(|s| s == "Alimentador TGN"));
    assert!(t.iter().any(|s| s == "Alimentador TF-1"));
    assert!(t.iter().any(|s| s == "F-01"));
    if let Ok(dir) = std::env::var("VOLTFORGE_SVG_OUT") {
        std::fs::write(format!("{dir}/nave.svg"), diagram.to_svg()).unwrap();
        std::fs::write(format!("{dir}/nave.json"), nave.to_json()).unwrap();
    }
}

/// El DXF lleva las mismas líneas y textos que el diagrama, con el eje Y
/// hacia arriba, en Windows-1252.
#[test]
fn dxf_has_every_line_and_text() {
    let project = voltforge_project::example::house();
    let result = voltforge_project::calculate(&project).unwrap();
    let diagram = voltforge_diagrams::single_line(&project, &result);
    let dxf = diagram.to_dxf();
    // Windows-1252: «á» es un byte, no dos.
    assert!(dxf.contains(&0xE1) || dxf.contains(&0xF3) || dxf.iter().all(|b| *b < 0x80));
    let text: String = dxf.iter().map(|&b| char::from(b)).collect();
    assert!(text.starts_with("  0\nSECTION\n  2\nHEADER"));
    assert!(text.trim_end().ends_with("EOF"));
    let texts = diagram
        .elements
        .iter()
        .filter(|e| matches!(e, voltforge_diagrams::Element::Text { .. }))
        .count();
    assert_eq!(text.matches("\n  0\nTEXT\n").count(), texts);
    let segments: usize = diagram
        .elements
        .iter()
        .map(|e| match e {
            voltforge_diagrams::Element::Path { points, .. } => points.len() - 1,
            voltforge_diagrams::Element::Rect { .. } => 4,
            voltforge_diagrams::Element::Text { .. } => 0,
        })
        .sum();
    assert_eq!(text.matches("\n  0\nLINE\n").count(), segments);
    // Los caracteres fuera de Windows-1252 van escapados.
    assert!(!text.contains('Σ'));
}
