//! Casos reproducibles de las auditorías del PR #11 (etapa 0 del plan 1.0).
//!
//! Cada prueba marcada `#[ignore]` describe **lo que debe pasar cuando el
//! hallazgo quede corregido** y hoy falla: su mensaje de error es la
//! reproducción del hallazgo, con el valor que da el programa. Se corren con
//!
//! ```text
//! cargo test -p ieestudio-project --test auditoria_pr11 -- --ignored
//! ```
//!
//! Al corregir un hallazgo se quita su `#[ignore]` en el mismo PR, y la
//! prueba queda como regresión. Las que dicen «depende de D-x» fijan solo lo
//! que vale con cualquiera de las alternativas de esa decisión; el valor
//! exacto se agrega cuando Antonio la apruebe.
//!
//! Las pruebas sin `#[ignore]` protegen lo que ya está bien y no debe cambiar
//! al corregir lo demás.
//!
//! `informe` imprime el estado actual de todos los casos:
//! `cargo test -p ieestudio-project --test auditoria_pr11 informe -- --ignored --nocapture`.

mod casos;

use ieestudio_project::evaluation::Evaluation;
use ieestudio_project::fault::InterruptingGroup;
use ieestudio_project::verdict::Compliance;
use ieestudio_project::{calculate, Project, ProjectResult, Status};

fn calc(project: &Project) -> ProjectResult {
    calculate(project).expect("el caso se calcula")
}

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "se esperaba {expected}, se obtuvo {actual}"
    );
}

/// Verificación de capacidad interruptiva de las barras de un tablero, o del
/// punto de conexión con `None`.
fn group<'a>(result: &'a ProjectResult, panel: Option<&str>) -> &'a InterruptingGroup {
    result
        .fault
        .as_ref()
        .expect("hay estudio de falla")
        .groups
        .iter()
        .find(|g| g.panel.as_deref() == panel)
        .unwrap_or_else(|| panic!("falta el lugar {panel:?}"))
}

/// `Cumple con 3.86 kA (Valid)`: lo que hoy dice el programa de un lugar.
fn state(g: &InterruptingGroup) -> String {
    format!(
        "{} con {} ({:?})",
        g.label(),
        g.fault_ka
            .map_or("sin falla estimada".to_string(), |f| format!("{f:.3} kA")),
        g.evaluation
    )
}

fn compliance_text(result: &ProjectResult) -> String {
    result.verdict.lines("NOM-001-SEDE-2012")[1].2.clone()
}

/// Algún aviso del lugar o del proyecto que contenga alguna de las palabras.
fn warned(result: &ProjectResult, g: &InterruptingGroup, words: &[&str]) -> bool {
    let hit = |t: &str| {
        let t = t.to_lowercase();
        words.iter().any(|w| t.contains(w))
    };
    g.trace.problems.iter().any(|p| hit(p))
        || result.warnings.iter().any(|w| hit(w))
        || result
            .fault
            .as_ref()
            .is_some_and(|f| f.source_trace.problems.iter().any(|p| hit(p)))
}

// ---------------------------------------------------------------- archivos

/// Los `.iees` de `fixtures/auditoria-pr11/` son los mismos proyectos que
/// arman estas pruebas. Con `IEES_REGENERAR=1` se vuelven a escribir.
#[test]
fn los_archivos_de_casos_estan_al_dia() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/auditoria-pr11");
    let regenerate = std::env::var_os("IEES_REGENERAR").is_some();
    for (name, project) in casos::all() {
        let path = dir.join(name);
        let json = project.to_json();
        if regenerate {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&path, &json).unwrap();
            continue;
        }
        let saved = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("falta {name}; regenera con IEES_REGENERAR=1"));
        assert_eq!(
            Project::from_json(&saved).unwrap(),
            project,
            "{name} no corresponde a su caso; regenera con IEES_REGENERAR=1"
        );
    }
}

/// Los archivos escritos por la versión anterior al PR #11 (`ef59295`) se
/// siguen leyendo igual. Ver `fixtures/compatibilidad/README.md`.
#[test]
fn los_archivos_anteriores_al_pr11_se_leen() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/compatibilidad");
    let mut read = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "iees")
            && path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("ef59295-"))
        {
            let text = std::fs::read_to_string(&path).unwrap();
            let project =
                Project::from_json(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            calculate(&project).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            read += 1;
        }
    }
    assert!(read > 0, "no hay archivos anteriores en {}", dir.display());
}

// ------------------------------------------------- H-01 fase a neutro 1F-3H

/// Lo que ya está bien: la falla entre líneas de 25 kVA, 3 % (reducida al
/// 2.7 %) a 240 V es `S / V / Z = 104.17 A / 0.027 = 3.858 kA`.
#[test]
fn h01_la_falla_entre_lineas_se_calcula_bien() {
    let result = calc(&casos::h01_acometida_240_120());
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    close(bus.base_ka, 25_000.0 / 240.0 / 0.027 / 1000.0, 1e-6);
}

/// H-01 (crítico). En un secundario 240/120 V la falla de fase a neutro puede
/// ser mayor que la de fase a fase. Con 5 kA no se puede afirmar «Cumple» ni
/// «Conforme» con una falla entre líneas de 3.86 kA (plan 1.0, 1A). Depende
/// de D-1 solo en el número: con el factor sería «No concluyente» por
/// 5.79 kA; sin él, «No concluyente» por no evaluar la fase a neutro.
#[test]
#[ignore = "H-01 abierto (D-1)"]
fn h01_acometida_240_120_no_cumple_con_la_falla_entre_lineas() {
    let result = calc(&casos::h01_acometida_240_120());
    let g = group(&result, Some("TG"));
    assert_ne!(g.label(), "Cumple", "H-01: hoy {}", state(g));
    assert_ne!(
        result.verdict.compliance,
        Compliance::Conforms,
        "H-01: hoy «{}»",
        compliance_text(&result)
    );
}

/// H-01 en un transformador intermedio seco 480 → 240/120 V.
#[test]
#[ignore = "H-01 abierto (D-1)"]
fn h01_intermedio_240_120_no_cumple_con_la_falla_entre_lineas() {
    let result = calc(&casos::h01_intermedio_240_120());
    let g = group(&result, Some("TF-1"));
    assert_ne!(g.label(), "Cumple", "H-01: hoy {}", state(g));
}

// --------------------------------------------- H-02 tensión del dato de CFE

/// H-02 (alto). Un dato de la suministradora mayor que el bus infinito de
/// su propio transformador es imposible: 1000 kVA, 5.75 % (5.175 %) a
/// 480 V dan 23.24 kA como máximo y el dato dice 50 kA. Debe avisarse con
/// cualquier alternativa de D-2.
#[test]
#[ignore = "H-02 abierto (D-2)"]
fn h02_un_dato_mayor_que_el_bus_infinito_se_avisa() {
    let result = calc(&casos::h02_subestacion_con_dato(50.0, 65.0));
    let g = group(&result, Some("TG"));
    assert!(
        warned(&result, g, &["bus infinito", "contradic", "mayor que"]),
        "H-02: sin aviso; hoy {}",
        state(g)
    );
}

/// H-02 (alto). 5 kA sin decir si son de 13.2 kV o de 480 V, con una
/// subestación propia de 1000 kVA. Si fueran de media tensión, el
/// secundario tendría unos 19.9 kA; hoy se usan 5 kA en 480 V y los
/// interruptores de 10 kA «cumplen». El plan (1B) pide no suponer que el
/// dato es de baja tensión. Depende de D-2 en cómo se corrige.
#[test]
#[ignore = "H-02 abierto (D-2)"]
fn h02_un_dato_sin_tension_no_da_cumple() {
    let result = calc(&casos::h02_subestacion_con_dato(5.0, 10.0));
    let g = group(&result, Some("TG"));
    assert_ne!(g.label(), "Cumple", "H-02: hoy {}", state(g));
}

// ----------------------------------------------- H-03 cargas que parecen motor

/// Lo que ya está bien: capturadas como motor, las dos bombas de 50 hp
/// (130 A cada una en la columna de 230 V) aportan `4 · 260 A` y la falla
/// sube de 9.8 a 10.84 kA, arriba de los interruptores de 10 kA.
#[test]
fn h03_capturadas_como_motor_no_concluyente() {
    let result = calc(&casos::h03_bombas(true));
    let g = group(&result, Some("TG"));
    close(g.fault_ka.unwrap(), 9.8 + 4.0 * 2.0 * 130.0 / 1000.0, 1e-6);
    assert_eq!(g.label(), "No concluyente");
}

/// H-03 (alto). Capturadas como «Otra carga», el programa avisa que parecen
/// motor y aun así afirma «Cumple» con 9.8 kA. Depende de D-3: con la
/// alternativa (a) debe quedar «No concluyente» mientras el aviso no se
/// resuelva.
#[test]
#[ignore = "H-03 abierto (D-3)"]
fn h03_una_carga_que_parece_motor_no_da_cumple() {
    let result = calc(&casos::h03_bombas(false));
    assert_eq!(result.kind_hints.len(), 2, "el aviso ya existe");
    let g = group(&result, Some("TG"));
    assert_ne!(g.label(), "Cumple", "H-03: hoy {}", state(g));
}

// ----------------------------------------------------- H-04 criterio trifásico

/// H-04 (medio). 226/120 V: el modelo (`System::has_three_phases`) lo toma
/// como monofásico de tres hilos, pero la falla lo calcula como trifásico.
/// Con un solo criterio: `25 000 / 226 / 0.027 = 4.097 kA`; hoy 2.365 kA.
#[test]
#[ignore = "H-04 abierto"]
fn h04_un_solo_criterio_de_trifasico() {
    let project = casos::h04_relacion_1_88();
    assert!(!project.source.system.has_three_phases());
    let result = calc(&project);
    let bus = result.fault.as_ref().unwrap().source.as_ref();
    let ka = bus.map(|b| b.base_ka);
    assert!(
        ka.is_some_and(|ka| (ka - 25_000.0 / 226.0 / 0.027 / 1000.0).abs() < 1e-3),
        "H-04: hoy {ka:?} kA"
    );
}

// -------------------------------------------------- H-05 datos contradictorios

/// H-05 (medio). Conexión «Monofásico» en el transformador de la acometida
/// de un sistema 220Y/127 V: busca la impedancia de la tabla monofásica
/// (1.5 %) y calcula la corriente como trifásico (13.12 kA en vez de
/// 9.84 kA), sin aviso.
#[test]
#[ignore = "H-05 abierto"]
fn h05_conexion_que_contradice_el_sistema_se_avisa() {
    let result = calc(&casos::h05_conexion_contradictoria());
    let g = group(&result, Some("TG"));
    assert!(
        g.evaluation == Evaluation::Invalid || warned(&result, g, &["conexión", "monofásico"]),
        "H-05: sin aviso; hoy {}",
        state(g)
    );
}

/// H-05 (medio). Impedancia de 30 % en un transformador tipo poste de
/// 112.5 kVA, 13.2 kV: fuera del intervalo de su renglón de la tabla
/// (2.0 a 3.0 %). Hoy da 1.09 kA y «Cumple» sin aviso.
#[test]
#[ignore = "H-05 abierto"]
fn h05_impedancia_fuera_de_la_tabla_se_avisa() {
    let result = calc(&casos::h05_impedancia_30());
    let g = group(&result, Some("TG"));
    assert!(
        warned(&result, g, &["impedancia"]),
        "H-05: sin aviso; hoy {}",
        state(g)
    );
}

// --------------------------------------- H-06 transformador de la acometida

/// H-06 (medio). 300 kW en un transformador de acometida de 75 kVA queda
/// «Conforme» porque esa capacidad no se compara. Depende de D-5: con (a)
/// es una verificación si es propio; con (b), un aviso. Con cualquiera de
/// las dos algo debe decirlo.
#[test]
#[ignore = "H-06 abierto (D-5)"]
fn h06_capacidad_del_transformador_de_la_acometida() {
    let result = calc(&casos::h06_acometida_sobrecargada());
    let checked = result.panels.iter().flat_map(|p| p.runs()).any(|r| {
        r.checks
            .iter()
            .any(|c| c.name.to_lowercase().contains("transformador"))
    });
    let warned = result
        .warnings
        .iter()
        .any(|w| w.to_lowercase().contains("transformador"));
    assert!(
        checked || warned,
        "H-06: sin verificación ni aviso; hoy «{}»",
        compliance_text(&result)
    );
}

// ------------------------------------------------- H-07 texto del dictamen

/// Lo que ya está bien: el caso cumple todo con interruptores de 22 kA.
#[test]
fn h07_el_caso_cumple_todo() {
    let result = calc(&casos::h07_todo_cumple(false));
    assert_eq!(result.verdict.compliance, Compliance::Conforms);
    assert_eq!(group(&result, Some("TG")).label(), "Cumple");
}

/// H-07 (medio). Con falla estimada el dictamen dice «Conforme con
/// NOM-001-SEDE-2012: todo cumple y nada quedó sin evaluar», sin el «en lo
/// evaluado» que el programa usa en los demás casos (D-6).
#[test]
#[ignore = "H-07 abierto (D-6)"]
fn h07_el_dictamen_dice_en_lo_evaluado() {
    let result = calc(&casos::h07_todo_cumple(false));
    let text = compliance_text(&result);
    assert!(text.contains("en lo evaluado"), "H-07: hoy «{text}»");
}

// ------------------------------------------------ H-12 conteo de motores

/// H-12 (bajo). Un motor detrás de un transformador sin potencia cuenta
/// como «1 motor sin datos» en el punto de conexión, pero la lista del
/// estudio (la que usa la memoria) no lo tiene.
#[test]
#[ignore = "H-12 abierto"]
fn h12_un_solo_conteo_de_motores_sin_datos() {
    let result = calc(&casos::h12_motor_detras_de_transformador_sin_datos());
    let study = result.fault.as_ref().unwrap();
    let in_place = group(&result, Some("TG")).incomplete_motors;
    assert!(in_place > 0, "el lugar ya lo cuenta");
    assert!(
        study.incomplete_motors.len() >= in_place,
        "H-12: el lugar cuenta {in_place} y el estudio {}",
        study.incomplete_motors.len()
    );
}

// --------------------------------------------- H-13 y H-15 textos de reglas

/// H-13 (bajo). La referencia de la aportación de motores todavía dice que
/// se suma «en cada tablero»; desde la decisión 0028 es por zona de tensión.
#[test]
#[ignore = "H-13 abierto"]
fn h13_la_referencia_de_motores_habla_de_zonas() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/nom-001-sede-2012/references.csv");
    let text = std::fs::read_to_string(path).unwrap();
    let line = text
        .lines()
        .find(|l| l.starts_with("motor_fault_contribution,"))
        .unwrap();
    assert!(!line.contains("en cada tablero"), "H-13: hoy «{line}»");
}

/// H-15 (bajo). Sin la corriente de carga nominal, la nota del compresor
/// dice que la corriente estimada «solo se usa para la caída y el cuadro»,
/// pero también entra en la falla. El minisplit del ejemplo (C-03) solo
/// tiene la ampacidad mínima.
#[test]
#[ignore = "H-15 abierto"]
fn h15_la_nota_del_compresor_menciona_la_falla() {
    let result = calc(&ieestudio_project::example::house());
    let c03 = result.panels[0]
        .circuits
        .iter()
        .find(|c| c.id == "C-03")
        .unwrap();
    let notes: Vec<&str> = c03
        .run
        .as_ref()
        .unwrap()
        .steps
        .iter()
        .filter_map(|s| s.note.as_deref())
        .filter(|n| n.contains("solo se usa"))
        .collect();
    assert!(notes.is_empty(), "H-15: hoy «{}»", notes.join(" | "));
}

// ------------------------------------------- H-16 transformador 1:1 en 127 V

/// H-16 (bajo). Un transformador de aislamiento 127 → 127 V que sale de un
/// tablero 220Y/127 V no se marca como 1:1: se compara el sistema completo
/// (220/127 V) con el secundario (127 V) en vez de la tensión de la
/// conexión.
#[test]
#[ignore = "H-16 abierto"]
fn h16_un_aislamiento_127_se_marca_uno_a_uno() {
    let result = calc(&casos::h16_aislamiento_127());
    assert!(
        result.duplicate_flags.iter().any(|f| f.panel_id == "TA-1"),
        "H-16: sin aviso"
    );
}

// ---------------------------------------- pendientes 17 a 20 de Opus

/// Pendiente 17 (Opus). El secundario usa la conexión del primario: un
/// elevador de 127 V a 240/120 V deja el tablero en una fase y rechaza un
/// circuito de 240 V que su secundario sí tiene.
#[test]
#[ignore = "Pendiente 17 abierto"]
fn p17_el_secundario_tiene_su_propia_conexion() {
    let result = calc(&casos::p17_secundario_con_otra_conexion());
    let tf = result.panels.iter().find(|p| p.id == "TF-1").unwrap();
    let d1 = tf.circuits.iter().find(|c| c.id == "D-1").unwrap();
    assert!(
        d1.run.is_some(),
        "17: hoy «{}»",
        d1.checks.first().map_or("", |c| c.detail.as_str())
    );
}

/// Pendiente 19 (Opus). Un circuito de 0 m se calcula con caída nula y
/// cumple. Depende de la decisión de la etapa 2C (si 0 m es «sin tramo»,
/// «dato faltante» o «inválido»); con cualquiera de las dos últimas no puede
/// cumplir.
#[test]
#[ignore = "Pendiente 19 abierto (decisión 2C)"]
fn p19_un_circuito_de_cero_metros_no_cumple_solo() {
    let result = calc(&casos::p19_longitud_cero());
    let a1 = &result.panels[0].circuits[0];
    assert_ne!(a1.status, Status::Complies, "19: hoy {:?}", a1.status);
}

// ------------------------------------------------------------------ informe

/// Estado actual de cada caso, para la tabla de la etapa 0.
#[test]
#[ignore = "informe, no es una verificación"]
fn informe() {
    println!("| Archivo | Lugar | Estado | Dictamen |");
    println!("|---|---|---|---|");
    for (name, project) in casos::all() {
        let result = calc(&project);
        let verdict = format!("{:?}", result.verdict.compliance);
        match &result.fault {
            Some(f) => {
                for g in &f.groups {
                    println!("| {name} | {} | {} | {verdict} |", g.place, state(g));
                }
                if f.groups.is_empty() {
                    println!("| {name} | — | sin lugares | {verdict} |");
                }
            }
            None => println!("| {name} | — | sin estudio de falla | {verdict} |"),
        }
    }
}
