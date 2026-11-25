//! Saneamiento de la fase 0.1 (`claude/fase-0-propuesta-c1-c2-tablero-vacio.md`):
//!
//! - **C-1**: una tubería compartida con conductores de otros circuitos no se
//!   puede verificar por ocupación con los datos del programa (decisión 0035).
//! - **C-2**: en el cálculo rápido, el conductor de puesta a tierra no se
//!   omite en silencio.
//! - **Tablero vacío**: un tablero sin circuitos, ni él ni sus subtableros, es
//!   un dato que falta, no un dato inválido.
//!
//! Los proyectos se guardan también en `fixtures/fase-0/` para abrirlos en la
//! aplicación o en la línea de comandos. Si se cambia un caso aquí, se
//! regeneran con `IEES_REGENERAR=1 cargo test -p ieestudio-project --test fase0`.

mod casos;

use ieestudio_core::{CircuitKind, Material, Size};
use ieestudio_project::quick::{quick_circuit, QuickInput, QuickResult};
use ieestudio_project::verdict::{Calculation, Compliance};
use ieestudio_project::{calculate, Project, ProjectResult, RunResult, Status};
use ieestudio_rules::Profile;

fn calc(project: &Project) -> ProjectResult {
    calculate(project).expect("el proyecto se calcula")
}

fn check<'a>(run: &'a RunResult, name: &str) -> &'a ieestudio_project::Check {
    run.checks
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| {
            panic!(
                "{} no tiene la verificación «{name}»: {:?}",
                run.id, run.checks
            )
        })
}

// ------------------------------------------------------------------ casos

/// C-1: el caso que cumple todo (`h07`) con el circuito A-1 de 1500 W en
/// 12 AWG y seis portadores de otros circuitos en su tubería. La ampacidad
/// cumple (17.50 A para 11.81 A); lo único pendiente es la ocupación.
pub fn c1_tuberia_compartida() -> Project {
    let mut p = casos::h07_todo_cumple(false);
    let a1 = &mut p.panels[0].circuits[0];
    a1.load.power_w = Some(1500.0);
    a1.overrides.size = Some(Size::new("12"));
    a1.overrides.other_current_carrying_conductors = 6;
    p
}

/// Tablero vacío: el subtablero TA-1 de `h16` sin circuitos. TA-1 se
/// alimenta por un transformador: lo vacío pasa por el primario y por el
/// secundario.
pub fn v_tablero_vacio() -> Project {
    let mut p = casos::h16_aislamiento_127();
    p.panels[1].circuits.clear();
    p
}

/// Tablero vacío en toda la jerarquía: ni TG ni TA-1 tienen circuitos, como
/// un proyecto recién creado con un subtablero.
pub fn v_jerarquia_vacia() -> Project {
    let mut p = v_tablero_vacio();
    p.panels[0].circuits.clear();
    p
}

/// Lo que no cambia (decisión D-3): TA-1 con su circuito capturado en 0 W.
pub fn v_circuito_de_cero_watts() -> Project {
    let mut p = casos::h16_aislamiento_127();
    p.panels[1].circuits[0].load.power_w = Some(0.0);
    p
}

fn all() -> Vec<(&'static str, Project)> {
    vec![
        ("c1-tuberia-compartida.iees", c1_tuberia_compartida()),
        ("v-tablero-vacio.iees", v_tablero_vacio()),
        ("v-jerarquia-vacia.iees", v_jerarquia_vacia()),
        ("v-circuito-de-cero-watts.iees", v_circuito_de_cero_watts()),
    ]
}

/// C-2: el caso de la auditoría. Aluminio 4 AWG THW en EMT, 12 kW, fp 0.9,
/// tres fases, 220Y/127 V, 20 m. La Tabla 250-122 del perfil no da calibre
/// de aluminio para el interruptor de 35 A.
fn c2_aluminio() -> QuickInput {
    QuickInput {
        material: Material::Aluminum,
        size: Size::new("4"),
        power_w: 12_000.0,
        length_m: 20.0,
        ..QuickInput::default()
    }
}

fn quick(input: &QuickInput) -> QuickResult {
    quick_circuit(Profile::nom_001_sede_2012(), input).expect("el cálculo rápido corre")
}

/// Los `.iees` de `fixtures/fase-0/` son los proyectos de este archivo.
#[test]
fn los_archivos_de_casos_estan_al_dia() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/fase-0");
    let regenerate = std::env::var_os("IEES_REGENERAR").is_some();
    for (name, project) in all() {
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

// -------------------------------------------------------------------- C-1

/// C-1: la ocupación de una tubería compartida queda sin evaluar, no se
/// publica una tubería y el dictamen no puede ser «Conforme / Completo».
#[test]
fn c1_la_tuberia_compartida_queda_sin_evaluar() {
    let result = calc(&c1_tuberia_compartida());
    let a1 = &result.panels[0].circuits[0];
    let tuberia = check(a1, "Tubería");
    assert_eq!(tuberia.status, Status::NotEvaluated, "{}", tuberia.detail);
    assert!(tuberia.detail.contains("compartida"), "{}", tuberia.detail);
    assert!(
        tuberia.detail.contains("6 conductores"),
        "{}",
        tuberia.detail
    );
    assert!(tuberia.is_shared_raceway());
    assert!(a1.run.as_ref().unwrap().conduit.is_none());
    assert_eq!(a1.status, Status::NotEvaluated);
    assert!(!a1.invalid_data);
    assert_eq!(result.verdict.compliance, Compliance::Incomplete);
    assert_ne!(
        (result.verdict.compliance, result.verdict.calculation),
        (Compliance::Conforms, Calculation::Complete)
    );
}

/// C-1: la ampacidad, el calibre, el interruptor y la tierra son los mismos
/// que antes de la corrección (medidos en `f04b59b`).
#[test]
fn c1_la_ampacidad_no_cambia() {
    let result = calc(&c1_tuberia_compartida());
    let a1 = &result.panels[0].circuits[0];
    let run = a1.run.as_ref().unwrap();
    assert_eq!(run.size, Size::new("12"));
    assert_eq!(run.breaker_a, 15.0);
    assert_eq!(run.current_carrying_conductors, 8);
    assert!((run.allowable_ampacity_a - 17.5).abs() < 1e-9);
    assert_eq!(run.conductors_per_phase, 1);
    assert!(!run.parallel_auto);
    assert_eq!(run.grounding_size, Size::new("12"));
    for name in ["Ampacidad", "Protección", "Caída de tensión"] {
        assert_eq!(check(a1, name).status, Status::Complies, "{name}");
    }
}

/// C-1: el paso de tubería del procedimiento se conserva y dice que es un
/// mínimo para los conductores del circuito, no la tubería a instalar.
#[test]
fn c1_el_procedimiento_dice_que_es_un_minimo() {
    let result = calc(&c1_tuberia_compartida());
    let run = result.panels[0].circuits[0].run.as_ref().unwrap();
    let step = run
        .steps
        .iter()
        .find(|s| s.title.starts_with("Tubería"))
        .expect("hay paso de tubería");
    let note = step.note.as_deref().unwrap_or_default();
    assert!(note.contains("no es la tubería a instalar"), "{note}");
}

/// C-1: sin otros portadores todo queda como antes: EMT de 16 mm y cumple.
#[test]
fn c1_sin_otros_portadores_todo_igual() {
    let mut p = c1_tuberia_compartida();
    p.panels[0].circuits[0]
        .overrides
        .other_current_carrying_conductors = 0;
    let result = calc(&p);
    let a1 = &result.panels[0].circuits[0];
    assert_eq!(check(a1, "Tubería").status, Status::Complies);
    assert!(!check(a1, "Tubería").is_shared_raceway());
    let conduit = a1.run.as_ref().unwrap().conduit.as_ref().unwrap();
    assert_eq!(conduit.metric_designator, 16);
    assert_eq!(result.verdict.compliance, Compliance::Conforms);
}

/// C-1: si los conductores del propio circuito no caben en ningún tubo, eso
/// sigue siendo «No cumple»: con más conductores tampoco caben.
#[test]
fn c1_si_no_cabe_sigue_sin_cumplir() {
    let mut p = c1_tuberia_compartida();
    let a1 = &mut p.panels[0].circuits[0];
    a1.kind = CircuitKind::ThreePhase;
    a1.phases = vec![
        ieestudio_project::Phase::A,
        ieestudio_project::Phase::B,
        ieestudio_project::Phase::C,
    ];
    // Cuatro de 1000 kcmil THW (953.8 mm² cada uno) más la tierra rebasan
    // el 40 % del EMT más grande, de 103 mm (3808 mm²).
    a1.has_neutral = true;
    a1.overrides.size = Some(Size::new("1000"));
    let result = calc(&p);
    assert_eq!(
        check(&result.panels[0].circuits[0], "Tubería").status,
        Status::Fails
    );
}

/// C-1: el mismo criterio en el alimentador y en el secundario de un
/// transformador, que se dimensionan con la misma función.
#[test]
fn c1_alimentador_y_secundario() {
    let mut p = casos::h16_aislamiento_127();
    p.panels[1]
        .feeder
        .overrides
        .other_current_carrying_conductors = 3;
    p.panels[1]
        .transformer
        .as_mut()
        .unwrap()
        .secondary_overrides
        .other_current_carrying_conductors = 3;
    let result = calc(&p);
    let ta1 = &result.panels[1];
    for run in [&ta1.feeder, ta1.secondary.as_ref().unwrap()] {
        let t = check(run, "Tubería");
        assert_eq!(t.status, Status::NotEvaluated, "{}: {}", run.name, t.detail);
        assert!(run.run.as_ref().unwrap().conduit.is_none(), "{}", run.name);
    }
    assert_ne!(result.verdict.compliance, Compliance::Conforms);
}

// -------------------------------------------------------------------- C-2

/// C-2: sin conductor de puesta a tierra no hay tubería ni conformidad.
#[test]
fn c2_sin_tierra_no_hay_tuberia_ni_conformidad() {
    let r = quick(&c2_aluminio());
    let p = r.protection.as_ref().unwrap();
    // Lo que no cambia: interruptor y ampacidad.
    assert_eq!(p.breaker_a, 35.0);
    assert!((p.allowable_ampacity_a - 55.0).abs() < 1e-9);
    assert_eq!(p.status, Status::Complies);
    // La tierra falta y lo dice.
    assert!(p.grounding_size.is_none());
    let why = p
        .grounding_error
        .as_deref()
        .expect("dice por qué falta la tierra");
    assert!(why.contains("250-122"), "{why}");
    // Sin tierra no se calcula la tubería.
    assert!(p.conduit.is_none(), "{:?}", p.conduit);
    assert!(p.conduit_error.is_some());
    assert!(p.minimum_size_error.is_some());
    let v = r.verdict();
    assert_eq!(v.compliance, Compliance::Incomplete);
    assert_eq!(v.calculation, Calculation::Incomplete);
}

/// C-2: la causa del calibre mínimo solo aparece cuando falta por el
/// conductor de puesta a tierra. Si ningún calibre de fase cumple la carga,
/// no se da causa y el resultado es el de `f04b59b`.
#[test]
fn c2_la_causa_del_minimo_solo_por_la_tierra() {
    // Cobre 14 AWG, 90 kW, una fase a 127 V: 787.40 A, ningún calibre cumple
    // con un conductor por fase. La tierra de 800 A sí está en la tabla.
    let r = quick(&QuickInput {
        kind: CircuitKind::LineToNeutral,
        size: Size::new("14"),
        power_w: 90_000.0,
        length_m: 30.0,
        ..QuickInput::default()
    });
    let p = r.protection.as_ref().unwrap();
    assert!(p.minimum_size.is_none());
    assert!(p.minimum_size_error.is_none(), "{:?}", p.minimum_size_error);
    assert!(p.grounding_error.is_none());
    assert_eq!(p.status, Status::Fails);
    let v = r.verdict();
    assert_eq!(v.compliance, Compliance::DoesNotConform);
    assert_eq!(v.calculation, Calculation::Complete);

    // Con aluminio y 35 A el mínimo falta por la tierra, y lo dice.
    let r = quick(&c2_aluminio());
    let p = r.protection.as_ref().unwrap();
    let why = p.minimum_size_error.as_deref().expect("dice por qué");
    assert!(why.contains("250-122"), "{why}");
}

/// C-2: ninguna combinación da una tubería calculada sin su conductor de
/// puesta a tierra.
#[test]
fn c2_ninguna_tuberia_sin_tierra() {
    for material in [Material::Copper, Material::Aluminum] {
        for (size, watts) in [
            ("6", 4_000.0),
            ("4", 12_000.0),
            ("1/0", 30_000.0),
            ("4/0", 50_000.0),
        ] {
            let r = quick(&QuickInput {
                material,
                size: Size::new(size),
                power_w: watts,
                length_m: 20.0,
                ..QuickInput::default()
            });
            if let Ok(p) = &r.protection {
                assert!(
                    p.conduit.is_none() || p.grounding_size.is_some(),
                    "{material:?} {size} {watts} W: tubería {:?} sin tierra",
                    p.conduit
                );
                if p.grounding_size.is_none() {
                    assert_ne!(r.verdict().compliance, Compliance::Conforms);
                }
            }
        }
    }
}

/// C-2: con un interruptor mayor de 100 A la tabla sí da tierra de aluminio
/// y el resultado no cambia: 50 kW, 145.8 A, 150 A; la tierra de 4 AWG de
/// aluminio de la tabla se aumenta en proporción al 4/0 (250-122(b)) y da
/// 2 AWG, el valor de `f04b59b`.
#[test]
fn c2_aluminio_con_renglon_en_la_tabla_sin_cambio() {
    let r = quick(&QuickInput {
        material: Material::Aluminum,
        size: Size::new("4/0"),
        power_w: 50_000.0,
        length_m: 20.0,
        ..QuickInput::default()
    });
    let p = r.protection.as_ref().unwrap();
    assert_eq!(p.breaker_a, 150.0);
    assert_eq!(p.grounding_size, Some(Size::new("2")));
    assert!(p.grounding_error.is_none());
    assert!(p.conduit.is_some());
    assert!(p.conduit_error.is_none());
    assert_eq!(r.verdict().calculation, Calculation::Complete);
}

/// C-2: el proyecto ya lo hacía bien: con aluminio y un interruptor de 100 A
/// o menos el tramo queda sin evaluar. Se fija para que no cambie.
#[test]
fn c2_el_proyecto_ya_no_omitia_la_tierra() {
    let mut p = casos::h07_todo_cumple(false);
    p.settings.material = Material::Aluminum;
    let result = calc(&p);
    let a1 = &result.panels[0].circuits[0];
    assert_eq!(a1.status, Status::NotEvaluated);
    assert!(a1.run.is_none());
    assert!(check(a1, "Cálculo").detail.contains("250-122"));
}

// ----------------------------------------------------------- tablero vacío

/// Un tablero sin circuitos es un dato que falta: «Datos incompletos», no
/// «Datos inválidos». Vale para el primario y para el secundario.
#[test]
fn v_un_tablero_sin_circuitos_es_incompleto() {
    let result = calc(&v_tablero_vacio());
    let ta1 = &result.panels[1];
    for run in [&ta1.feeder, ta1.secondary.as_ref().unwrap()] {
        assert!(!run.invalid_data, "{}", run.name);
        assert_eq!(run.status, Status::NotEvaluated);
        let detail = &check(run, "Cálculo").detail;
        assert!(detail.starts_with("falta"), "{}: {detail}", run.name);
        assert!(detail.contains("TA-1"), "{detail}");
    }
    assert_eq!(result.verdict.compliance, Compliance::Incomplete);
}

/// Toda la jerarquía vacía, como un proyecto nuevo: los dos tableros quedan
/// incompletos y el dictamen también.
#[test]
fn v_una_jerarquia_vacia_es_incompleta() {
    let result = calc(&v_jerarquia_vacia());
    for panel in &result.panels {
        assert!(!panel.feeder.invalid_data, "{}", panel.id);
        assert!(
            check(&panel.feeder, "Cálculo").detail.starts_with("falta"),
            "{}",
            panel.id
        );
    }
    assert_eq!(result.verdict.compliance, Compliance::Incomplete);
}

/// D-3: un tablero con circuitos capturados en 0 W se clasifica como hasta
/// ahora: dato inválido. No cambia en esta corrección.
#[test]
fn v_circuitos_de_cero_watts_sin_cambio() {
    let result = calc(&v_circuito_de_cero_watts());
    let ta1 = &result.panels[1];
    assert!(ta1.feeder.invalid_data);
    assert_eq!(
        check(&ta1.feeder, "Cálculo").detail,
        "dato inválido: el tablero no tiene carga"
    );
    assert_eq!(result.verdict.compliance, Compliance::InvalidData);
}

/// Una potencia negativa sigue siendo un dato inválido.
#[test]
fn v_una_potencia_negativa_sigue_invalida() {
    let mut p = casos::h16_aislamiento_127();
    p.panels[1].circuits[0].load.power_w = Some(-10.0);
    let result = calc(&p);
    assert!(result.panels[1].feeder.invalid_data);
    assert_eq!(result.verdict.compliance, Compliance::InvalidData);
}
