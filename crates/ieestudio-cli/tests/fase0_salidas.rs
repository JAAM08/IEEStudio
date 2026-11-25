//! Fase 0.1 en las salidas: la tubería compartida (C-1), el conductor de
//! puesta a tierra del cálculo rápido (C-2) y el tablero vacío se ven igual
//! en la línea de comandos, el unifilar, la memoria y el servidor MCP. Usan
//! los proyectos de `fixtures/fase-0/`.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use ieestudio_core::{Material, Size};
use ieestudio_project::quick::{quick_circuit, QuickInput};
use ieestudio_project::{calculate, Project};
use ieestudio_rules::Profile;

fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/fase-0")
        .join(name)
}

fn load(name: &str) -> Project {
    let text = std::fs::read_to_string(path(name)).expect("existe el caso");
    Project::from_json(&text).expect("el caso se lee")
}

fn cli(name: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ieestudio"))
        .arg("calcular")
        .arg(path(name))
        .output()
        .expect("corre la línea de comandos");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// El caso C-2 de la auditoría: aluminio 4 AWG, 12 kW, 3F, 20 m.
fn c2_aluminio() -> QuickInput {
    QuickInput {
        material: Material::Aluminum,
        size: Size::new("4"),
        power_w: 12_000.0,
        length_m: 20.0,
        ..QuickInput::default()
    }
}

// -------------------------------------------------------------------- C-1

/// C-1: ninguna salida publica la tubería compartida ni dice «Conforme».
#[test]
fn c1_las_salidas_no_publican_la_tuberia_compartida() {
    let name = "c1-tuberia-compartida.iees";
    let project = load(name);
    let result = calculate(&project).unwrap();

    let text = cli(name);
    let a1 = text
        .lines()
        .find(|l| l.trim_start().starts_with("A-1"))
        .unwrap_or_else(|| panic!("no está A-1:\n{text}"));
    assert!(!a1.contains("16 mm"), "{a1}");
    assert!(a1.contains("Sin evaluar"), "{a1}");
    assert!(text.contains("Evaluación incompleta"), "{text}");
    assert!(!text.contains("Conforme en lo evaluado"), "{text}");

    let svg = ieestudio_diagrams::plan(&project, &result).to_svg();
    assert!(svg.contains("TUBO —"), "el unifilar no dice TUBO —");

    let report = ieestudio_reports::project_source(&project, &result);
    assert!(report.contains("la tubería es compartida"));
    assert!(report.contains("Evaluación incompleta"));
    // En el cuadro de la memoria la tubería de A-1 es «—»; el mínimo propio
    // solo aparece en la verificación y en el procedimiento, con su aviso.
    let row = report
        .lines()
        .find(|l| l.contains("\"A-1\", \"Alumbrado A-1\""))
        .expect("A-1 en el cuadro de la memoria");
    assert!(row.contains("mono(\"—\")"), "{row}");
    assert!(!row.contains("16 mm"), "{row}");
    assert!(report.contains("no es la tubería a instalar"));
}

// -------------------------------------------------------------------- C-2

/// C-2: la memoria del cálculo rápido dice que falta la tierra y no da
/// conformidad.
#[test]
fn c2_la_memoria_del_calculo_rapido_lo_dice() {
    let input = c2_aluminio();
    let result = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    let source = ieestudio_reports::quick_source(&input, &result);
    assert!(source.contains("Puesta a tierra"), "{source}");
    assert!(source.contains("Sin evaluar: Tabla 250-122"), "{source}");
    assert!(
        source.contains("no se calculó porque falta el conductor de puesta a tierra"),
        "{source}"
    );
    assert!(source.contains("Evaluación incompleta"), "{source}");
    assert!(!source.contains("Conforme en lo evaluado"), "{source}");
}

/// C-2: el servidor MCP entrega la causa y un veredicto incompleto.
#[test]
fn c2_el_servidor_mcp_lo_dice() {
    let request = |id: u32, method: &str, params: serde_json::Value| {
        serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
            .to_string()
    };
    let input = format!(
        "{}\n{}\n",
        request(1, "initialize", serde_json::json!({})),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "calculo_rapido",
                "arguments": {"entrada": {
                    "material": "Aluminum", "size": "4", "power_w": 12000,
                    "length_m": 20, "kind": "ThreePhase"
                }}
            })
        )
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_ieestudio"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("corre el servidor");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let last = String::from_utf8_lossy(&out.stdout)
        .lines()
        .last()
        .unwrap()
        .to_string();
    let reply: serde_json::Value = serde_json::from_str(&last).unwrap();
    let text = reply["result"]["content"][0]["text"].as_str().unwrap();
    let quick: serde_json::Value = serde_json::from_str(text).unwrap();
    let protection = &quick["resultado"]["protection"]["Ok"];
    assert!(protection["grounding_size"].is_null(), "{protection}");
    assert!(protection["grounding_error"]
        .as_str()
        .unwrap()
        .contains("250-122"));
    assert!(protection["conduit"].is_null(), "{protection}");
    assert!(protection["conduit_error"].is_string(), "{protection}");
    assert_eq!(quick["veredicto"]["compliance"], "Incomplete");
    assert_eq!(quick["veredicto"]["calculation"], "Incomplete");
}

// ----------------------------------------------------------- tablero vacío

/// Tablero vacío: la línea de comandos y la memoria dicen que falta
/// capturar sus circuitos, no que hay un dato inválido.
#[test]
fn v_las_salidas_dicen_que_faltan_circuitos() {
    let name = "v-tablero-vacio.iees";
    let text = cli(name);
    assert!(text.contains("Evaluación incompleta"), "{text}");
    assert!(!text.contains("Datos inválidos"), "{text}");
    let project = load(name);
    let result = calculate(&project).unwrap();
    let report = ieestudio_reports::project_source(&project, &result);
    assert!(report.contains("falta capturar los circuitos del tablero TA-1"));
    assert!(!report.contains("Datos inválidos: requiere corrección"));
}
