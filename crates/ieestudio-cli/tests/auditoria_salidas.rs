//! Casos de las auditorías del PR #11 que se ven en las salidas: unifilar
//! (H-10), memoria y línea de comandos (H-11). Usan los proyectos de
//! `fixtures/auditoria-pr11/`, los mismos que la aplicación puede abrir.
//!
//! Como en `crates/ieestudio-project/tests/auditoria_pr11.rs`, cada prueba con
//! `#[ignore]` dice lo que debe pasar cuando el hallazgo se corrija y hoy
//! falla con lo que el programa muestra:
//!
//! ```text
//! cargo test -p ieestudio-cli --test auditoria_salidas -- --ignored
//! ```

use std::path::PathBuf;
use std::process::Command;

use ieestudio_project::{calculate, Project};

fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/auditoria-pr11")
        .join(name)
}

fn load(name: &str) -> Project {
    let text = std::fs::read_to_string(path(name)).expect("existe el caso");
    Project::from_json(&text).expect("el caso se lee")
}

/// Texto del unifilar en modo plano (el de la memoria, el PDF y el DXF).
fn plan_svg(name: &str) -> String {
    let project = load(name);
    let result = calculate(&project).unwrap();
    ieestudio_diagrams::plan(&project, &result).to_svg()
}

/// Fuente Typst de la memoria.
fn report(name: &str) -> String {
    let project = load(name);
    let result = calculate(&project).unwrap();
    ieestudio_reports::project_source(&project, &result)
}

/// Lo que imprime `ieestudio calcular`.
fn cli(name: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ieestudio"))
        .arg("calcular")
        .arg(path(name))
        .output()
        .expect("corre la línea de comandos");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Renglones que contienen el texto, para el mensaje de error.
fn lines_with<'a>(text: &'a str, needle: &str) -> Vec<&'a str> {
    text.lines().filter(|l| l.contains(needle)).collect()
}

// -------------------------------------------------------------- H-10

/// H-10. Con el dato de la suministradora inválido (0 kA) no hay ninguna
/// falla estimada, pero la nota del plano dice «Icc: corriente de falla con
/// el dato de la suministradora…».
#[test]
#[ignore = "H-10 abierto"]
fn h10_sin_falla_no_hay_nota_de_suministradora() {
    let svg = plan_svg("h11-dato-invalido.iees");
    let wrong = lines_with(&svg, "con el dato de la suministradora");
    assert!(wrong.is_empty(), "H-10: hoy {wrong:?}");
}

/// H-10. Con la impedancia mínima de la tabla, la nota dice que se redujo
/// la de placa un 10 %, cosa que no se hizo.
#[test]
#[ignore = "H-10 abierto"]
fn h10_con_impedancia_de_tabla_no_dice_reducida() {
    let svg = plan_svg("h07-todo-cumple-tabla.iees");
    let wrong = lines_with(&svg, "reducida");
    assert!(wrong.is_empty(), "H-10: hoy {wrong:?}");
}

/// H-10. Un tablero sin falla estimada no lleva nada en el plano: debe
/// decir «Icc SIN ESTIMAR» (con su causa). TF-1 no tiene potencia.
#[test]
#[ignore = "H-10 abierto"]
fn h10_un_tablero_sin_falla_dice_sin_estimar() {
    for case in [
        "h09-transformador-sin-potencia.iees",
        "h11-dato-invalido.iees",
    ] {
        let svg = plan_svg(case);
        assert!(
            svg.contains("SIN ESTIMAR"),
            "H-10 ({case}): hoy {:?}",
            lines_with(&svg, "Icc")
        );
    }
}

// -------------------------------------------------------------- H-11

/// H-11. En la memoria, el dato inválido aparece con su estado pero la
/// frase dice «sin evaluar en…», y el renglón «Barras de…» solo se escribe
/// para datos faltantes.
#[test]
#[ignore = "H-11 abierto"]
fn h11_la_memoria_no_llama_sin_evaluar_a_un_dato_invalido() {
    let source = report("h11-dato-invalido.iees");
    let mixed: Vec<&str> = source
        .lines()
        .filter(|l| l.contains("Dato inválido") && l.to_lowercase().contains("sin evaluar"))
        .collect();
    assert!(mixed.is_empty(), "H-11: hoy {mixed:?}");
}

/// H-11. La línea de comandos imprime «Capacidad interruptiva (dato
/// inválido): sin evaluar en…».
#[test]
#[ignore = "H-11 abierto"]
fn h11_la_linea_de_comandos_no_llama_sin_evaluar_a_un_dato_invalido() {
    let out = cli("h11-dato-invalido.iees");
    assert!(out.contains("dato inválido"), "el estado ya se imprime");
    let mixed: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("dato inválido") && l.contains("sin evaluar"))
        .collect();
    assert!(mixed.is_empty(), "H-11: hoy {mixed:?}");
}

/// Lo que ya está bien: las tres salidas reciben el mismo caso y la línea
/// de comandos lo calcula sin error.
#[test]
fn las_salidas_abren_todos_los_casos() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/auditoria-pr11");
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_none_or(|e| e != "iees") {
            continue;
        }
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        let project = load(&name);
        let result = calculate(&project).unwrap();
        assert!(!ieestudio_diagrams::plan(&project, &result)
            .to_svg()
            .is_empty());
        assert!(!ieestudio_reports::project_source(&project, &result).is_empty());
        assert!(cli(&name).contains("Verificación normativa"), "{name}");
    }
}
