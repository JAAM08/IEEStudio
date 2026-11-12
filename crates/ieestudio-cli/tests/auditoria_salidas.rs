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

// -------------------------------------------------------------- H-01

/// H-01 (etapa 1A). Las cuatro salidas dicen lo mismo del caso de 25 kVA,
/// 240/120 V e interruptores de 5 kA: 6.44 kA de fase a neutro, no
/// concluyente, con el factor y su fuente.
#[test]
fn h01_las_salidas_dicen_fase_a_neutro() {
    let case = "h01-acometida-240-120.iees";

    let svg = plan_svg(case);
    assert!(
        svg.contains("Icc = 6.44 kA (BUS INFINITO, FASE A NEUTRO)"),
        "unifilar: {:?}",
        lines_with(&svg, "Icc")
    );
    assert!(
        svg.contains("FASE A NEUTRO es 1.67 veces la de fase a fase"),
        "nota del unifilar: {:?}",
        lines_with(&svg, "Icc:")
    );

    let source = report(case);
    assert!(source.contains("Icc F-N = 1.67 · 3.86 kA = 6.44 kA"));
    assert!(source.contains("estimada como 1.67 veces la de fase a fase"));
    assert!(source.contains("por bus infinito, de fase a neutro"));

    let out = cli(case);
    assert!(
        out.contains(
            "los 2 interruptores de las barras de TG no alcanzan los 6.44 kA estimados por bus infinito, de fase a neutro"
        ),
        "línea de comandos: {out}"
    );
    assert!(!out.contains("Conforme con"), "línea de comandos: {out}");
}

/// Un sistema trifásico no cambia: ni el unifilar ni la memoria hablan de
/// fase a neutro.
#[test]
fn h01_un_trifasico_no_menciona_fase_a_neutro() {
    let case = "h07-todo-cumple.iees";
    assert!(!plan_svg(case).contains("FASE A NEUTRO"));
    assert!(!report(case).contains("Icc F-N"));
}

// -------------------------------------------------------------- H-02

/// H-02 (etapa 1B). Con 5 kA en 13.2 kV y la subestación de 1000 kVA, las
/// salidas dicen lo mismo: el dato en la acometida a su tensión y 22.92 kA
/// en el secundario, con el método.
#[test]
fn h02_las_salidas_dicen_media_tension() {
    let case = "h02-dato-en-primario-5ka.iees";

    let svg = plan_svg(case);
    assert!(
        svg.contains("Icc = 5 kA EN 13.2 kV (DATO DE LA SUMINISTRADORA)"),
        "unifilar: {:?}",
        lines_with(&svg, "Icc")
    );
    assert!(
        svg.contains("Icc = 22.92 kA (DATO EN MEDIA TENSIÓN Y TRANSFORMADOR)"),
        "unifilar: {:?}",
        lines_with(&svg, "Icc")
    );
    assert!(svg.contains("combina en cuadratura"));

    let source = report(case);
    assert!(source.contains("Zred = 1000 kVA / (√3 · 13.2 kV · 5 kA) = 0.00875"));
    assert!(source.contains("en media tensión, la red se representa"));

    let out = cli(case);
    assert!(
        out.contains("los 22.92 kA estimados con el dato de la suministradora en media tensión y el transformador"),
        "línea de comandos: {out}"
    );
}

/// Un dato sin decir dónde está no da ningún número: las salidas piden
/// indicarlo.
#[test]
fn h02_sin_ubicacion_las_salidas_piden_indicarla() {
    let case = "h02-dato-media-tension-5ka.iees";
    let out = cli(case);
    assert!(out.contains("falta indicar dónde da la suministradora la corriente de falla"));
    assert!(!out.contains("kA estimados"), "línea de comandos: {out}");
    assert!(!plan_svg(case).contains("Icc = 5"));
}

// -------------------------------------------------------------- H-03

/// H-03 (etapa 1C). Sin confirmar: el aviso y la capacidad interruptiva no
/// concluyente en la línea de comandos y en la memoria.
#[test]
fn h03_sin_confirmar_las_salidas_no_concluyen() {
    let case = "h03-bombas-como-otra-carga.iees";
    let out = cli(case);
    assert!(
        out.contains("2 cargas que parecen motor o aire acondicionado están sin confirmar"),
        "línea de comandos: {out}"
    );
    assert!(out.contains(
        "Mientras no se resuelva, la capacidad interruptiva de su zona queda no concluyente"
    ));
    let source = report(case);
    assert!(source.contains("parece motor o aire acondicionado y está como «Otra carga»"));
}

/// Confirmadas: sin aviso, cumple, y las salidas dicen que se confirmaron.
#[test]
fn h03_confirmadas_las_salidas_lo_dicen() {
    let case = "h03-bombas-confirmadas-otra-carga.iees";
    let out = cli(case);
    assert!(!out.contains("Aviso: TG · M-1"), "línea de comandos: {out}");
    assert!(out.contains("Nota: TG · M-1 Bomba 50 hp se confirmó como otra carga"));
    // Ya no queda nada de capacidad interruptiva por resolver.
    assert!(!out.contains("Capacidad interruptiva ("), "{out}");
    assert!(out.contains("Verificación normativa: Conforme"), "{out}");
    let source = report(case);
    assert!(source.contains("se confirmó como otra carga aunque su nombre parece de un motor"));
}
