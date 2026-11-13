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

use ieestudio_core::{CircuitKind, System, Topology};
use ieestudio_project::evaluation::Evaluation;
use ieestudio_project::fault::{FaultOrigin, InterruptingGroup, IssueKind};
use ieestudio_project::verdict::Compliance;
use ieestudio_project::{calculate, Project, ProjectResult, Status};
use ieestudio_project::{Connection, FaultLocation, Mounting};

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
//
// Corregido en la etapa 1A con la decisión 0029: en un secundario 1F-3H por
// bus infinito manda la falla de fase a neutro, 1.67 veces la de fase a fase.

/// Factor de fase a neutro del perfil (decisión 0029).
const LN: f64 = 1.67;
/// 25 kVA, 3 % reducido al 2.7 %, 240 V: `104.17 A / 0.027 = 3.858 kA`.
const LL_25: f64 = 25_000.0 / 240.0 / 0.027 / 1000.0;

/// Matemática: la de fase a fase no cambia y la de fase a neutro es
/// `1.67 · 3.858 = 6.443 kA` (6.4429); la de fase a neutro es la que se compara.
#[test]
fn h01_la_falla_de_fase_a_neutro_es_el_factor_por_la_de_fase_a_fase() {
    let result = calc(&casos::h01_acometida_240_120());
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    let ln = bus.line_to_neutral.as_ref().expect("secundario 1F-3H");
    close(ln.line_to_line_ka, LL_25, 1e-9);
    assert_eq!(ln.factor, LN);
    close(bus.base_ka, LN * LL_25, 1e-9);
    close(bus.total_ka, 6.4429, 1e-4);
    assert_eq!(bus.basis(), "por bus infinito, de fase a neutro");
    let step = bus.steps.last().unwrap();
    assert_eq!(step.title, "Falla de fase a neutro (1F-3H)");
    assert_eq!(step.substitution, "Icc F-N = 1.67 · 3.86 kA = 6.44 kA");
}

/// H-01 (crítico). Acometida 1F-3H de 25 kVA con interruptores de 5 kA: ya
/// no «Cumple» con 3.86 kA; queda «No concluyente» con 6.44 kA de fase a
/// neutro y el dictamen ya no es «Conforme».
#[test]
fn h01_acometida_240_120_no_cumple_con_la_falla_entre_lineas() {
    let result = calc(&casos::h01_acometida_240_120());
    let g = group(&result, Some("TG"));
    assert_eq!(g.label(), "No concluyente", "H-01: hoy {}", state(g));
    close(g.fault_ka.unwrap(), LN * LL_25, 1e-9);
    assert!(g.line_to_neutral);
    assert!(g.summary().contains("de fase a neutro"), "{}", g.summary());
    assert_ne!(
        result.verdict.compliance,
        Compliance::Conforms,
        "H-01: hoy «{}»",
        compliance_text(&result)
    );
    // Cada interruptor dice contra qué falla se comparó.
    let detail = &result.panels[0].circuits[0]
        .interrupting
        .as_ref()
        .unwrap()
        .detail;
    assert_eq!(
        detail,
        "no concluyente: 5 kA es menor que los 6.44 kA estimados en las barras de TG por bus infinito, de fase a neutro, que es una cota superior; requiere estudio de cortocircuito"
    );
}

/// Con interruptores de 10 kA el mismo caso cumple: 6.44 kA ≤ 10 kA. El
/// factor no deja todo 1F-3H sin concluir.
#[test]
fn h01_acometida_240_120_cumple_si_alcanza_la_fase_a_neutro() {
    let mut project = casos::h01_acometida_240_120();
    casos::breakers(&mut project, 0, 10.0);
    let result = calc(&project);
    assert_eq!(group(&result, Some("TG")).label(), "Cumple");
}

/// H-01 en un transformador intermedio seco 480 → 240/120 V: el secundario
/// de TF-1 lleva el factor; las barras de 480Y/277 V de TG no.
#[test]
fn h01_intermedio_240_120_no_cumple_con_la_falla_entre_lineas() {
    let result = calc(&casos::h01_intermedio_240_120());
    let g = group(&result, Some("TF-1"));
    assert_eq!(g.label(), "No concluyente", "H-01: hoy {}", state(g));
    close(g.fault_ka.unwrap(), LN * LL_25, 1e-9);
    let tg = group(&result, Some("TG"));
    assert!(!tg.line_to_neutral);
    assert_eq!(tg.fault_ka, Some(25.0));
}

/// Ejemplo de la decisión 0029: 50 kVA, 3 %, 240 V dan 7.72 kA de fase a
/// fase y 12.89 kA de fase a neutro; con 10 kA, «No concluyente».
#[test]
fn h01_ejemplo_de_la_decision_50_kva() {
    let mut project = casos::h01_acometida_240_120();
    project.source.substation.as_mut().unwrap().kva = Some(50.0);
    casos::breakers(&mut project, 0, 10.0);
    let result = calc(&project);
    let g = group(&result, Some("TG"));
    close(g.fault_ka.unwrap(), 12.8858, 1e-4);
    assert_eq!(g.label(), "No concluyente");
}

/// El dato de la suministradora en un servicio 1F-3H no se multiplica: ya
/// es la corriente máxima disponible en el punto de conexión. La
/// trazabilidad lo dice.
#[test]
fn h01_el_dato_de_la_suministradora_no_se_multiplica() {
    let mut project = casos::h01_acometida_240_120();
    project.source.fault.available_ka = Some(5.0);
    project.source.fault.location = Some(ieestudio_project::FaultLocation::LowVoltage);
    let result = calc(&project);
    let study = result.fault.as_ref().unwrap();
    let bus = study.source.as_ref().unwrap();
    assert_eq!(bus.total_ka, 5.0);
    assert!(bus.line_to_neutral.is_none());
    assert!(study
        .source_trace
        .assumptions
        .iter()
        .any(|a| a.contains("no se multiplica")));
    assert_eq!(group(&result, Some("TG")).label(), "Cumple");
}

/// Solo un secundario 1F-3H lleva el factor: ni un trifásico 220Y/127 V ni
/// uno de una fase y dos hilos (127 V).
#[test]
fn h01_el_factor_solo_aplica_a_1f_3h() {
    let result = calc(&casos::h07_todo_cumple(false));
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    assert!(bus.line_to_neutral.is_none());

    let mut project = casos::single_board(
        ieestudio_core::System::SINGLE_127,
        ieestudio_core::CircuitKind::LineToNeutral,
    );
    project.source.substation = Some(casos::substation(
        10.0,
        Some(3.0),
        ieestudio_project::Connection::SinglePhase,
    ));
    let result = calc(&project);
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    assert!(bus.line_to_neutral.is_none());
    close(bus.total_ka, 10_000.0 / 127.0 / 0.027 / 1000.0, 1e-9);
}

/// Normativa: la regla cita la fuente del factor y dice que es un criterio
/// de ingeniería; las verificaciones la llevan junto con 110-9 y 110-10.
#[test]
fn h01_la_regla_cita_su_fuente() {
    use ieestudio_rules::{Profile, Rule};
    let profile = Profile::nom_001_sede_2012();
    let clause = profile.clause(Rule::LineToNeutralFault);
    assert!(clause.contains("Bussmann"), "{clause}");
    assert!(profile
        .rule_description(Rule::LineToNeutralFault)
        .unwrap()
        .contains("criterio de ingeniería"));
    assert_eq!(profile.parameters.fault_line_to_neutral_factor, LN);
    let result = calc(&casos::h01_acometida_240_120());
    let g = group(&result, Some("TG"));
    let clauses: Vec<&str> = g
        .references
        .iter()
        .filter_map(|r| r.clause.as_deref())
        .collect();
    assert!(clauses.contains(&clause.as_str()), "{clauses:?}");
    assert!(clauses.contains(&"110-9, 110-10"), "{clauses:?}");
    assert!(g.trace.assumptions.iter().any(|a| a.contains("1.67 veces")));
}

// --------------------------------------------- H-02 tensión del dato de CFE
//
// Corregido en la etapa 1B con la decisión 0029 (D-2). Subestación propia de
// 1000 kVA, 5.75 % (5.175 % reducida), 13.2 kV a 480Y/277 V:
// In = 1 000 kVA / (√3 · 480 V) = 1 202.81 A; bus infinito 23.24 kA.

/// In del transformador de 1000 kVA a 480 V, en amperes.
const IN_1000: f64 = 1_000_000.0 / (1.732_050_807_568_877_2 * 480.0);

/// H-02 (alto). Un dato en baja tensión mayor que el bus infinito de su
/// propio transformador es imposible: 50 kA contra 23.24 kA. «Dato
/// inválido» con la causa, nunca «Cumple».
#[test]
fn h02_un_dato_mayor_que_el_bus_infinito_se_avisa() {
    let result = calc(&casos::h02_subestacion_con_dato(
        50.0,
        Some(FaultLocation::LowVoltage),
        65.0,
    ));
    let g = group(&result, Some("TG"));
    assert_eq!(g.evaluation, Evaluation::Invalid, "H-02: hoy {}", state(g));
    assert_eq!(g.label(), "Dato inválido");
    let issue = &result
        .fault
        .as_ref()
        .unwrap()
        .source_issue
        .as_ref()
        .unwrap();
    assert_eq!(issue.kind, IssueKind::Invalid);
    assert!(
        issue.reason.contains("(50 kA) es mayor que la mayor posible con el transformador de la acometida (23.24 kA por bus infinito)"),
        "{}",
        issue.reason
    );
}

/// Un dato en baja tensión que no pasa del bus infinito se usa tal cual.
#[test]
fn h02_un_dato_en_baja_tension_se_usa_tal_cual() {
    let result = calc(&casos::h02_subestacion_con_dato(
        20.0,
        Some(FaultLocation::LowVoltage),
        22.0,
    ));
    let study = result.fault.as_ref().unwrap();
    let bus = study.source.as_ref().unwrap();
    assert_eq!(bus.origin, FaultOrigin::Supplier);
    assert_eq!(bus.total_ka, 20.0);
    assert_eq!(group(&result, Some("TG")).label(), "Cumple");
}

/// H-02 (alto). 5 kA sin decir si son de 13.2 kV o de 480 V: antes se
/// usaban en 480 V y los interruptores de 10 kA «cumplían». Ahora queda
/// «Sin evaluar» y el motivo pide indicar dónde.
#[test]
fn h02_un_dato_sin_tension_no_da_cumple() {
    let result = calc(&casos::h02_subestacion_con_dato(5.0, None, 10.0));
    let g = group(&result, Some("TG"));
    assert_eq!(g.label(), "Sin evaluar", "H-02: hoy {}", state(g));
    assert!(g
        .issue
        .as_deref()
        .unwrap()
        .starts_with(ieestudio_project::fault::MISSING_LOCATION));
    assert_ne!(result.verdict.compliance, Compliance::Conforms);
}

/// Matemática y referencia (ejemplo de la decisión 0029): 5 kA en 13.2 kV.
/// `Zred = 1000 / (√3 · 13.2 · 5 · 1000) = 0.008748`;
/// `Z = √(0.008748² + 0.05175²) = 0.052484`; `Icc = 1202.81 / 0.052484 =
/// 22.92 kA`. Con interruptores de 10 kA, «No concluyente».
#[test]
fn h02_un_dato_en_media_tension_pasa_por_el_transformador() {
    let result = calc(&casos::h02_subestacion_con_dato(
        5.0,
        Some(FaultLocation::Primary),
        10.0,
    ));
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    assert_eq!(bus.origin, FaultOrigin::SupplierPrimary);
    let z_net = 1000.0 / (3f64.sqrt() * 13.2 * 5.0 * 1000.0);
    let z = (z_net * z_net + 0.05175f64 * 0.05175).sqrt();
    close(bus.total_ka, IN_1000 / z / 1000.0, 1e-9);
    close(bus.total_ka, 22.9177, 1e-4);
    // Menor que el bus infinito y mayor que la suma de magnitudes.
    assert!(bus.total_ka < IN_1000 / 0.05175 / 1000.0);
    assert!(bus.total_ka > IN_1000 / (z_net + 0.05175) / 1000.0);
    let g = group(&result, Some("TG"));
    assert_eq!(g.label(), "No concluyente");
    assert!(g
        .summary()
        .contains("estimados con el dato de la suministradora en media tensión y el transformador, que es una cota superior"));
    let steps: Vec<&str> = bus.steps.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(
        steps,
        [
            "Corriente de falla de la suministradora",
            "Corriente nominal del transformador de la acometida",
            "Impedancia para la falla",
            "Impedancia de la red",
            "Falla en el secundario",
        ]
    );
    assert_eq!(
        bus.steps[3].substitution,
        "Zred = 1000 kVA / (√3 · 13.2 kV · 5 kA) = 0.00875"
    );
}

/// Con el dato en media tensión hacen falta los datos del transformador:
/// sin él o sin la tensión del primario, «Sin evaluar» con la causa.
#[test]
fn h02_un_dato_en_media_tension_necesita_el_transformador() {
    let mut project = casos::h02_subestacion_con_dato(5.0, Some(FaultLocation::Primary), 10.0);
    project.source.substation.as_mut().unwrap().primary_kv = None;
    let result = calc(&project);
    let issue = result.fault.as_ref().unwrap().source_issue.clone().unwrap();
    assert_eq!(issue.kind, IssueKind::Incomplete);
    assert!(issue.reason.contains("falta la tensión del primario"));

    project.source.substation = None;
    let result = calc(&project);
    let issue = result.fault.as_ref().unwrap().source_issue.clone().unwrap();
    assert_eq!(issue.kind, IssueKind::Incomplete);
    assert!(issue
        .reason
        .contains("falta el transformador de la acometida"));
}

/// En un secundario 1F-3H el dato de media tensión no reduce la falla: se
/// queda el bus infinito de fase a neutro (decisión 0029, D-1), y el
/// procedimiento lo dice.
#[test]
fn h02_en_1f_3h_el_dato_de_media_tension_no_reduce() {
    let mut project = casos::h01_acometida_240_120();
    project.source.fault.available_ka = Some(1.0);
    project.source.fault.location = Some(FaultLocation::Primary);
    let result = calc(&project);
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    assert_eq!(bus.origin, FaultOrigin::Substation);
    close(bus.total_ka, LN * LL_25, 1e-9);
    assert!(bus
        .steps
        .iter()
        .any(|s| s.title == "Dato de media tensión en un secundario 1F-3H"));
}

/// Normativa: la regla del equivalente de red cita la IEC 60909-0 con su
/// edición y dice que falta cotejar la de 2026; la verificación la lleva.
#[test]
fn h02_la_regla_de_la_red_cita_su_fuente() {
    use ieestudio_rules::{Profile, Rule};
    let profile = Profile::nom_001_sede_2012();
    let clause = profile.clause(Rule::NetworkFaultImpedance);
    assert!(clause.contains("IEC 60909-0:2016"), "{clause}");
    assert!(profile
        .rule_description(Rule::NetworkFaultImpedance)
        .unwrap()
        .contains("edición 2026"));
    let result = calc(&casos::h02_subestacion_con_dato(
        5.0,
        Some(FaultLocation::Primary),
        10.0,
    ));
    let g = group(&result, Some("TG"));
    assert!(g
        .references
        .iter()
        .any(|r| r.clause.as_deref() == Some(clause.as_str())));
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

/// H-03 (alto). Capturadas como «Otra carga», el programa avisaba que
/// parecen motor y aun así afirmaba «Cumple» con 9.8 kA. Corregido en la
/// etapa 1C (decisión 0029, D-3): «No concluyente» mientras no se resuelva.
#[test]
fn h03_una_carga_que_parece_motor_no_da_cumple() {
    let result = calc(&casos::h03_bombas(false));
    assert_eq!(result.kind_hints.len(), 2, "el aviso ya existe");
    let g = group(&result, Some("TG"));
    assert_eq!(g.label(), "No concluyente", "H-03: hoy {}", state(g));
    assert_eq!(g.suspected_loads, 2);
    // La falla no cambia: no se inventa la aportación.
    assert_eq!(g.fault_ka, Some(9.8));
    assert!(
        g.summary()
            .contains("pero 2 cargas que parecen motor o aire acondicionado están sin confirmar"),
        "{}",
        g.summary()
    );
    assert!(g
        .trace
        .missing
        .iter()
        .any(|m| m.contains("M-1 Bomba 50 hp: parece motor")));
    assert_ne!(result.verdict.compliance, Compliance::Conforms);
    let detail = &result.panels[0]
        .circuits
        .iter()
        .find(|c| c.id == "A-1")
        .unwrap()
        .interrupting
        .as_ref()
        .unwrap()
        .detail;
    assert_eq!(
        detail,
        "no concluyente: 10 kA ≥ 9.80 kA en las barras de TG, pero 2 cargas que parecen motor o aire acondicionado están sin confirmar"
    );
}

/// «Es otra carga» en las dos: cumple con 9.8 kA, el aviso desaparece y la
/// trazabilidad dice que se confirmaron. Con una sola, sigue sin concluir.
#[test]
fn h03_confirmar_otra_carga_la_resuelve() {
    let mut project = casos::h03_bombas(false);
    project.panels[0].circuits[3].load.confirmed_general = true;
    let result = calc(&project);
    assert_eq!(result.kind_hints.len(), 1);
    assert_eq!(group(&result, Some("TG")).label(), "No concluyente");

    project.panels[0].circuits[4].load.confirmed_general = true;
    let result = calc(&project);
    assert!(result.kind_hints.is_empty());
    assert_eq!(result.confirmed_general.len(), 2);
    let g = group(&result, Some("TG"));
    assert_eq!(g.label(), "Cumple", "{}", state(g));
    assert!(g
        .trace
        .contributors
        .iter()
        .any(|m| m.contains("M-1 Bomba 50 hp: confirmada como otra carga")));
}

/// La confirmación se guarda en el archivo y se vuelve a leer igual.
#[test]
fn h03_la_confirmacion_se_guarda() {
    let mut project = casos::h03_bombas(false);
    for c in &mut project.panels[0].circuits[3..] {
        c.load.confirmed_general = true;
    }
    let json = project.to_json();
    assert_eq!(json.matches("\"confirmed_general\": true").count(), 2);
    let read = Project::from_json(&json).unwrap();
    assert_eq!(read, project);
    assert!(calc(&read).kind_hints.is_empty());
    // Sin confirmar no se escribe nada: los archivos no cambian.
    assert!(!casos::h03_bombas(false)
        .to_json()
        .contains("confirmed_general"));
}

/// Una carga sin confirmar detrás de un transformador también deja sin
/// concluir las barras de arriba: si es motor, su aportación llegaría a
/// través del transformador.
#[test]
fn h03_una_sospecha_aguas_abajo_alcanza_las_barras_de_arriba() {
    let mut project = casos::h12_motor_detras_de_transformador_sin_datos();
    project.panels[1].transformer.as_mut().unwrap().kva = Some(15.0);
    let general = casos::h03_bombas(false).panels[0].circuits[3].load.clone();
    project.panels[1].circuits[0].load = general;
    let result = calc(&project);
    assert_eq!(group(&result, Some("TF-1")).suspected_loads, 1);
    assert_eq!(group(&result, Some("TG")).suspected_loads, 1);
    assert_eq!(group(&result, Some("TG")).label(), "No concluyente");
}

// ----------------------------------------------------- H-04 criterio trifásico

/// H-04 (medio, etapa 1D, decisión 0030). 226/120 V (1.883) no corresponde a
/// ningún sistema: la capacidad interruptiva calculada con el transformador
/// queda «Dato inválido», con las tensiones, la relación y las esperadas.
/// Antes daba 2.365 kA como trifásico y «Cumple»; nunca se toma como
/// 240/120 V.
#[test]
fn h04_una_relacion_incompatible_es_dato_invalido() {
    let project = casos::h04_relacion_1_88();
    assert_eq!(project.source.system.topology, None);
    let result = calc(&project);
    let g = group(&result, Some("TG"));
    assert_eq!(g.evaluation, Evaluation::Invalid, "{}", state(g));
    let fault = result.fault.as_ref().unwrap();
    assert!(fault.source.is_none());
    let issue = fault.source_issue.as_ref().unwrap();
    assert_eq!(issue.kind, IssueKind::Invalid);
    assert_eq!(
        issue.reason,
        "en el sistema de suministro, la relación entre 226 V y 120 V es 1.883; no corresponde a un 3F-4H en estrella (√3 = 1.732) ni a un 1F-3H (2), con ±1 %"
    );
    assert!(!compliance_text(&result).contains("Conforme"));
}

/// H-04. Declarado 1F-3H tampoco: la relación no es 2 con ±1 %.
#[test]
fn h04_declarado_con_relacion_incompatible_es_dato_invalido() {
    let mut project = casos::h04_relacion_1_88();
    project.source.system.topology = Some(Topology::SinglePhaseThreeWire);
    let result = calc(&project);
    let issue = result.fault.as_ref().unwrap().source_issue.clone().unwrap();
    assert_eq!(issue.kind, IssueKind::Invalid);
    assert!(
        issue.reason.ends_with("un 1F-3H pide 2.000, con ±1 %"),
        "{}",
        issue.reason
    );
    // Con relación 2.1 (252/120) igual: el criterio anterior de la falla lo
    // tomaba como trifásico.
    project.source.system.line_to_line_v = Some(252.0);
    let result = calc(&project);
    assert_eq!(group(&result, Some("TG")).evaluation, Evaluation::Invalid);
}

/// H-04. Sin topología declarada, el neutro del alimentador de dos fases no
/// se sabe si es portador: «Sin evaluar». Con la topología declarada sí se
/// calcula, aunque la relación sea incompatible, porque el alimentador no
/// usa la relación (decisión 0030, T-3 y T-5).
#[test]
fn h04_lo_que_no_depende_de_la_relacion_sigue() {
    let mut project = casos::h04_relacion_1_88();
    let result = calc(&project);
    let feeder = &result.panels[0].feeder;
    assert!(feeder.run.is_none());
    assert_eq!(feeder.status, Status::NotEvaluated);
    assert!(
        feeder.checks[0]
            .detail
            .starts_with("falta declarar la topología"),
        "{}",
        feeder.checks[0].detail
    );
    project.source.system.topology = Some(Topology::SinglePhaseThreeWire);
    let result = calc(&project);
    let feeder = result.panels[0]
        .feeder
        .run
        .as_ref()
        .expect("alimentador calculado");
    assert_eq!(feeder.poles, 2);
    assert_eq!(group(&result, Some("TG")).evaluation, Evaluation::Invalid);
}

/// H-04. Dentro del 1 %, la topología declarada se usa: 240/120 V da la
/// falla de 1F-3H como en la etapa 1A (3.86 kA · 1.67 = 6.44 kA).
#[test]
fn h04_una_relacion_dentro_de_la_tolerancia_se_calcula() {
    let result = calc(&casos::h01_acometida_240_120());
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    close(bus.base_ka, 6.4429, 1e-3);
}

/// H-04. La regla cita su fuente: criterio de ingeniería del perfil.
#[test]
fn h04_la_regla_cita_su_fuente() {
    let result = calc(&casos::h04_relacion_1_88());
    let check = result.panels[0]
        .runs()
        .find_map(|r| r.interrupting.as_ref())
        .expect("verificación de capacidad interruptiva");
    assert!(
        check.references.iter().any(|r| r
            .clause
            .as_deref()
            .is_some_and(|c| c.contains("decisión 0030"))),
        "{:?}",
        check.references
    );
}

// -------------------------------------------------- H-05 datos contradictorios

/// H-05 (medio, etapa 1D, decisión 0030 C-1). Conexión «Monofásico» en el
/// transformador de la acometida de un sistema 220Y/127 V. Antes buscaba la
/// impedancia de la tabla monofásica (1.5 %) y calculaba como trifásico:
/// 13.12 kA sin aviso. Ahora, «Dato inválido» con la orientación sobre los
/// bancos.
#[test]
fn h05_conexion_que_contradice_el_sistema_es_dato_invalido() {
    let result = calc(&casos::h05_conexion_contradictoria());
    let g = group(&result, Some("TG"));
    assert_eq!(g.evaluation, Evaluation::Invalid, "{}", state(g));
    let issue = result.fault.as_ref().unwrap().source_issue.clone().unwrap();
    assert_eq!(issue.kind, IssueKind::Invalid);
    assert_eq!(
        issue.reason,
        "la conexión del transformador de la acometida (Monofásico) no corresponde al sistema declarado (3F-4H, trifásico en estrella). Verifica si capturaste una unidad monofásica o un banco: varias unidades monofásicas que forman un banco trifásico se capturan con la conexión del banco completo"
    );
    let check = result.panels[0]
        .runs()
        .find_map(|r| r.interrupting.as_ref())
        .unwrap();
    assert!(check
        .references
        .iter()
        .any(|r| r.clause.as_deref() == Some("coherencia de datos (decisión 0030)")));
}

/// H-05. Al revés: una conexión trifásica en un sistema 1F-3H 240/120 V.
#[test]
fn h05_conexion_trifasica_en_un_monofasico_es_dato_invalido() {
    let mut project = casos::h01_acometida_240_120();
    project.source.substation.as_mut().unwrap().connection = Some(Connection::DeltaWye);
    let result = calc(&project);
    assert_eq!(group(&result, Some("TG")).evaluation, Evaluation::Invalid);
}

/// H-05 (C-2). Un banco de tres monofásicos que da 220Y/127 V se captura
/// con la conexión del banco: no hay contradicción y se calcula. Un
/// monofásico en 1F-3H tampoco contradice nada.
#[test]
fn h05_un_banco_con_su_conexion_se_calcula() {
    let mut project = casos::h05_conexion_contradictoria();
    project.source.substation.as_mut().unwrap().connection = Some(Connection::WyeWye);
    let result = calc(&project);
    assert!(result.fault.as_ref().unwrap().source.is_some());
    let result = calc(&casos::h01_acometida_240_120());
    assert!(result.fault.as_ref().unwrap().source.is_some());
}

/// H-05 (punto 8). Con el dato de la suministradora en baja tensión el
/// transformador no interviene: la conexión no se valida y el dato se usa.
#[test]
fn h05_con_dato_en_baja_tension_la_conexion_no_interviene() {
    let mut project = casos::h05_conexion_contradictoria();
    project.source.fault.available_ka = Some(10.0);
    project.source.fault.location = Some(FaultLocation::LowVoltage);
    let result = calc(&project);
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    assert_eq!(bus.base_ka, 10.0);
    assert_eq!(bus.origin, FaultOrigin::Supplier);
}

// ---------------------------- Neutro portador, 310-15(b)(5) (decisión 0030, T-5)

/// El alimentador del caso con su sistema, conexión, neutro y carga no
/// lineal: devuelve sus conductores portadores o el motivo de «Sin evaluar».
fn feeder_carrying(
    system: System,
    kind: CircuitKind,
    nonlinear: Option<bool>,
) -> Result<u32, String> {
    let mut project = casos::single_board(system, kind);
    project.panels[0].feeder.has_neutral = true;
    project.panels[0].feeder.overrides.nonlinear = nonlinear;
    let result = calc(&project);
    let feeder = &result.panels[0].feeder;
    match &feeder.run {
        Some(run) => Ok(run.current_carrying_conductors),
        None => Err(feeder.checks[0].detail.clone()),
    }
}

/// (2): dos fases y neutro de un 3F-4H en estrella: el neutro cuenta.
/// (1): de un 1F-3H solo lleva el desequilibrio: no cuenta.
#[test]
fn t5_dos_fases_y_neutro_segun_la_topologia() {
    assert_eq!(
        feeder_carrying(System::WYE_220_127, CircuitKind::LineToLine, None),
        Ok(3)
    );
    assert_eq!(
        feeder_carrying(System::SPLIT_240_120, CircuitKind::LineToLine, None),
        Ok(2)
    );
    let undeclared = System {
        topology: None,
        ..System::SPLIT_240_120
    };
    let why = feeder_carrying(undeclared, CircuitKind::LineToLine, None).unwrap_err();
    assert!(why.starts_with("falta declarar la topología"), "{why}");
}

/// (3): tres fases y neutro en estrella con carga mayormente no lineal: el
/// neutro cuenta; con carga lineal, (1), no; sin capturar, «Sin evaluar».
#[test]
fn t5_tres_fases_y_neutro_segun_la_carga_no_lineal() {
    let wye = System::WYE_220_127;
    assert_eq!(
        feeder_carrying(wye, CircuitKind::ThreePhase, Some(true)),
        Ok(4)
    );
    assert_eq!(
        feeder_carrying(wye, CircuitKind::ThreePhase, Some(false)),
        Ok(3)
    );
    let why = feeder_carrying(wye, CircuitKind::ThreePhase, None).unwrap_err();
    assert_eq!(
        why,
        "falta indicar si la carga es mayormente no lineal para saber si el neutro cuenta como conductor portador (310-15(b)(5)(3))"
    );
    // Un faltante no es un dato inválido (decisión 0027).
    assert!(!why.contains("dato inválido"));
}

/// Sin topología declarada no hay tres fases: el circuito no se calcula y
/// dice qué falta. Con tres fases declaradas se admite aunque la relación
/// no corresponda (T-4).
#[test]
fn t4_tres_fases_solo_si_se_declaran() {
    let undeclared = System {
        topology: None,
        ..System::WYE_220_127
    };
    let why = feeder_carrying(undeclared, CircuitKind::ThreePhase, Some(false)).unwrap_err();
    assert!(why.starts_with("falta declarar la topología"), "{why}");
    let odd = System {
        line_to_line_v: Some(226.0),
        line_to_neutral_v: Some(120.0),
        topology: Some(Topology::ThreePhaseWye),
    };
    assert_eq!(
        feeder_carrying(odd, CircuitKind::ThreePhase, Some(false)),
        Ok(3)
    );
}

// ------------------------- Pendientes de la NMX-J-116-ANCE-2017 (decisión 0030)
//
// N-1 a N-3 están decididos pero en pausa (ajuste del plan del 12 de
// noviembre): atienden casos poco comunes y además esperan la tabla y la
// nota 3 de la edición 2017. Al implementar N-1 faltan además: arriba del máximo con la placa
// confirmada (cumple con aviso) y la confirmación que se pierde al cambiar
// la impedancia, la potencia, la tensión del primario o el montaje.

/// N-1 (H-05). Impedancia de 30 % en un transformador tipo poste de
/// 112.5 kVA, 13.2 kV: arriba del intervalo de su renglón (2.00 a 3.00 %).
/// Hoy da 1.09 kA y «Cumple» sin aviso; debe quedar «No concluyente» hasta
/// confirmar la placa.
#[test]
#[ignore = "en pausa: ajuste del plan del 12 de noviembre (decisión 0030, N-1)"]
fn n1_impedancia_arriba_del_maximo_no_concluye() {
    let result = calc(&casos::h05_impedancia_30());
    let g = group(&result, Some("TG"));
    assert!(warned(&result, g, &["impedancia"]), "N-1: hoy {}", state(g));
    assert_eq!(g.label(), "No concluyente", "N-1: hoy {}", state(g));
}

/// N-1. Impedancia de 1 % en el mismo transformador: abajo del mínimo. Se
/// calcula con ella (la falla sale mayor) y solo se avisa.
#[test]
#[ignore = "en pausa: ajuste del plan del 12 de noviembre (decisión 0030, N-1)"]
fn n1_impedancia_abajo_del_minimo_solo_avisa() {
    let mut project = casos::h05_impedancia_30();
    project
        .source
        .substation
        .as_mut()
        .unwrap()
        .impedance_percent = Some(1.0);
    let result = calc(&project);
    let g = group(&result, Some("TG"));
    assert!(warned(&result, g, &["impedancia"]), "N-1: hoy {}", state(g));
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    // 112.5 kVA en 220 V: 295.24 A; con 1 % reducido 10 %, 32.80 kA.
    close(
        bus.base_ka,
        112_500.0 / (3f64.sqrt() * 220.0) / 0.009 / 1000.0,
        1e-3,
    );
}

/// N-2. Nota 3: un monofásico tipo subestación de 167 kVA, 13.2 kV, toma el
/// mínimo del trifásico tipo subestación de su clase (2.50 %), no el del
/// monofásico tipo poste (1.50 %), aunque sea de menos de 225 kVA.
#[test]
#[ignore = "en pausa: ajuste del plan del 12 de noviembre (decisión 0030, N-2)"]
fn n2_monofasico_tipo_subestacion_usa_la_columna_de_subestacion() {
    let mut project = casos::single_board(System::SPLIT_240_120, CircuitKind::LineToLine);
    let mut sub = casos::substation(167.0, None, Connection::SinglePhase);
    sub.mounting = Some(Mounting::Substation);
    project.source.substation = Some(sub);
    let result = calc(&project);
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    let ln = bus.line_to_neutral.as_ref().expect("1F-3H");
    close(ln.line_to_line_ka, 167_000.0 / 240.0 / 0.025 / 1000.0, 1e-3);
}

/// N-3. Banco de tres monofásicos tipo poste de 50 kVA (150 kVA), 13.2 kV,
/// 220Y/127 V, con impedancia de tabla: renglón monofásico de 50 kVA
/// (1.50 %), no el trifásico tipo poste de 150 kVA (2.00 %). Hoy da 19.68 kA;
/// debe dar 26.24 kA. Al implementar N-3 se marca aquí la casilla del banco.
#[test]
#[ignore = "en pausa: ajuste del plan del 12 de noviembre (decisión 0030, N-3)"]
fn n3_banco_de_monofasicos_usa_el_renglon_de_cada_unidad() {
    let mut project = casos::single_board(System::WYE_220_127, CircuitKind::ThreePhase);
    project.source.substation = Some(casos::substation(150.0, None, Connection::DeltaWye));
    let result = calc(&project);
    let bus = result.fault.as_ref().unwrap().source.as_ref().unwrap();
    close(
        bus.base_ka,
        150_000.0 / (3f64.sqrt() * 220.0) / 0.015 / 1000.0,
        1e-3,
    );
}

// --------------------------------------- H-06 transformador de la acometida

/// H-06 (medio). 300 kW en un transformador de acometida de 75 kVA queda
/// «Conforme» porque esa capacidad no se compara. Con la decisión 0029 (D-5)
/// se verifica si es propio y queda fuera de alcance si es de CFE; al
/// agregar ese dato, el caso se marca como propio y esta prueba pide «No
/// cumple».
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

/// H-07 (medio, etapa 1E, decisión 0029 D-6). Con falla estimada el
/// dictamen decía «Conforme con NOM-001-SEDE-2012: todo cumple…», sin el «en
/// lo evaluado». Ahora siempre lo lleva.
#[test]
fn h07_el_dictamen_dice_en_lo_evaluado() {
    let result = calc(&casos::h07_todo_cumple(false));
    let text = compliance_text(&result);
    assert_eq!(
        text,
        "Conforme en lo evaluado con NOM-001-SEDE-2012: lo que el programa verifica cumple y nada quedó sin evaluar. La capacidad interruptiva se verificó con un método conservador: la falla estimada es una cota superior."
    );
}

/// D-6: ningún dictamen afirma conformidad sin «en lo evaluado», y
/// «Pendiente» queda solo para la aprobación profesional.
#[test]
fn h07_ningun_dictamen_dice_conforme_a_secas_ni_pendiente() {
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/auditoria-pr11");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "iees") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let result = calc(&Project::from_json(&text).unwrap());
        let [calculation, compliance, approval] = result.verdict.lines("NOM-001-SEDE-2012");
        let name = path.display();
        assert!(
            !compliance.2.contains("Conforme con"),
            "{name}: {}",
            compliance.2
        );
        assert!(
            !compliance.2.contains("Pendiente"),
            "{name}: {}",
            compliance.2
        );
        assert!(!calculation.2.contains("Pendiente"), "{name}");
        assert!(approval.2.starts_with("Pendiente"));
        assert!(compliance
            .2
            .starts_with(result.verdict.compliance.heading()));
    }
}

/// D-6: un encabezado en orden de prioridad, con el conteo de lo demás.
#[test]
fn h07_los_cuatro_encabezados_en_orden_de_prioridad() {
    use ieestudio_project::verdict::{Outcome, Verdict};
    use Outcome::*;
    let text = |v: Verdict| v.lines("NOM-001-SEDE-2012")[1].2.clone();
    // No conforme gana aunque haya inválidos y faltantes.
    let v = Verdict::new(5, 1, 0, [Fails, Invalid, Invalid, Incomplete, Complies]);
    assert_eq!(v.compliance, Compliance::DoesNotConform);
    assert_eq!(
        text(v),
        "No conforme en lo evaluado con NOM-001-SEDE-2012: 1 verificación no cumple · además 2 evaluaciones con dato inválido, 1 tramo sin calcular y 1 verificación sin evaluar o no concluyente."
    );
    // Datos inválidos gana a lo incompleto; un tramo de dato inválido no se
    // cuenta dos veces.
    let v = Verdict::new(3, 2, 1, [Invalid, Incomplete]);
    assert_eq!(v.compliance, Compliance::InvalidData);
    assert_eq!(
        text(v),
        "Datos inválidos: requiere corrección: 2 evaluaciones con dato inválido · además 1 tramo sin calcular y 1 verificación sin evaluar o no concluyente."
    );
    let v = Verdict::new(3, 1, 0, [Incomplete, Incomplete, Complies]);
    assert_eq!(v.compliance, Compliance::Incomplete);
    assert_eq!(
        text(v),
        "Evaluación incompleta: 1 tramo sin calcular y 2 verificaciones sin evaluar o no concluyentes; no se puede declarar conforme."
    );
    let v = Verdict::new(2, 0, 0, [Complies, Complies]);
    assert_eq!(v.compliance, Compliance::Conforms);
    assert!(text(v).starts_with("Conforme en lo evaluado con NOM-001-SEDE-2012"));
}

/// D-6 en un caso: el dato de 0 kA es «Datos inválidos», no «sin evaluar».
#[test]
fn h07_un_dato_invalido_encabeza_como_dato_invalido() {
    let result = calc(&casos::h11_dato_invalido());
    assert_eq!(result.verdict.compliance, Compliance::InvalidData);
    assert_eq!(
        compliance_text(&result),
        "Datos inválidos: requiere corrección: 2 evaluaciones con dato inválido."
    );
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
