//! Casos del cálculo rápido de un circuito, con resultados calculados a mano.

use voltforge_core::{CircuitKind, Profile, Size, System};
use voltforge_project::quick::{quick_circuit, QuickInput};
use voltforge_project::Status;

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "se esperaba {expected}, se obtuvo {actual}"
    );
}

/// Alimentador trifásico de 61.7 kW, fp 0.90, 220 V, 95 m, 3/0 AWG de cobre en
/// tubo de acero. Tabla 9: R = 0.259, X = 0.171 Ω/km.
/// I = 61700 / (1.7321 · 220 · 0.9) = 179.91 A
/// ΔV = 1.7321 · 179.91 · 0.095 · (0.259·0.9 + 0.171·0.4359) = 9.11 V = 4.14 %
#[test]
fn reference_feeder_fails_and_gets_alternatives() {
    let r = quick_circuit(Profile::nom_001_sede_2012(), &QuickInput::default()).unwrap();
    close(r.current_a, 179.91, 0.01);
    close(r.drop.drop_v, 9.11, 0.01);
    close(r.drop.drop_percent, 4.14, 0.01);
    assert_eq!(r.status, Status::Fails);
    // Longitud máxima: 95 · 3 / 4.139 = 68.9 m.
    close(r.max_length_m, 68.9, 0.1);

    // 300 kcmil (R = 0.148, X = 0.167): 29.60 · (0.1332 + 0.0728) = 6.10 V = 2.77 %.
    let bigger = &r.recommendations[0];
    assert_eq!(bigger.apply, Some((Size::new("300"), 1)));
    assert_eq!(bigger.value, "2.77 %");
    // Dos de 1/0 (R = 0.39, X = 0.18): 29.60 · (0.351 + 0.0785) / 2 = 6.36 V = 2.89 %.
    // Ampacidad: 150 A · 0.80 (6 portadores) · 2 = 240 A, cubre 179.91 A y el
    // interruptor de 200 A.
    let parallel = &r.recommendations[1];
    assert_eq!(parallel.apply, Some((Size::new("1/0"), 2)));
    assert_eq!(parallel.value, "2.89 %");
    assert_eq!(r.recommendations.len(), 3);
    // Once pasos: corriente, diseño, interruptor, ampacidad, verificación,
    // tierra, tubería, R y X, caída, porcentaje y comprobación exacta.
    assert_eq!(r.steps.len(), 11);

    // Protección: carga no continua, Id = 179.91 A; 240-6 da 200 A.
    // 3/0 AWG THW a 75 °C: 200 A, factores 1.00 (30 °C, 3 portadores).
    // Tabla 250-122 para 200 A: 6 AWG de cobre.
    let p = r.protection.as_ref().unwrap();
    close(p.design_a, 179.91, 0.01);
    assert_eq!(p.breaker_a, 200.0);
    assert_eq!(p.poles, 3);
    close(p.allowable_ampacity_a, 200.0, 0.001);
    assert_eq!(p.status, Status::Complies);
    assert_eq!(p.grounding_size, Some(Size::new("6")));
    // Tubería: 3 × 3/0 THW de 201.10 mm² + 6 AWG desnudo de 17.09 mm² = 620.39 mm²;
    // al 40 %, el EMT de 53 mm admite 866 mm² y el de 41 mm solo 526 mm².
    assert_eq!(p.conduit.as_deref(), Some("EMT de 53 mm (2\")"));
}

/// Circuito de una fase con la corriente capturada: 15 A, 127 V, 20 m, 12 AWG
/// de cobre en acero, fp 1. Tabla 9: R = 6.6 Ω/km.
/// ΔV = 2 · 15 · 0.020 · 6.6 = 3.96 V = 3.12 %; con 10 AWG (R = 3.9): 1.84 %.
#[test]
fn single_phase_with_current_input() {
    let input = QuickInput {
        system: System::WYE_220_127,
        kind: CircuitKind::LineToNeutral,
        load_in_amperes: true,
        current_a: 15.0,
        power_factor: 1.0,
        length_m: 20.0,
        size: Size::new("12"),
        ..QuickInput::default()
    };
    let r = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    close(r.drop.drop_v, 3.96, 0.005);
    close(r.drop.drop_percent, 3.118, 0.005);
    assert_eq!(r.status, Status::Fails);
    assert_eq!(r.recommendations[0].apply, Some((Size::new("10"), 1)));
    assert_eq!(r.recommendations[0].value, "1.84 %");
}

/// Un calibre que cumple no recibe alternativas, solo su margen de longitud.
#[test]
fn complying_run_only_reports_margin() {
    let input = QuickInput {
        size: Size::new("300"),
        ..QuickInput::default()
    };
    let r = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    assert_eq!(r.status, Status::Complies);
    assert_eq!(r.recommendations.len(), 1);
    assert!(r.recommendations[0].apply.is_none());
    assert!(r.warnings.is_empty());
}

/// Paralelo por debajo de 1/0 AWG se avisa y la ampacidad insuficiente no cumple.
#[test]
fn warns_about_small_parallel_and_ampacity() {
    let input = QuickInput {
        size: Size::new("8"),
        conductors_per_phase: 2,
        ..QuickInput::default()
    };
    let r = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    // 8 AWG de cobre a 75 °C: 50 A · 0.80 (6 portadores) · 2 = 80 A, menos que 179.91 A.
    let p = r.protection.as_ref().unwrap();
    close(p.corrected_ampacity_a, 80.0, 0.001);
    assert_eq!(p.status, Status::Fails);
    assert_eq!(r.warnings.len(), 1);
}

/// Un sistema sin la tensión que el circuito requiere no se evalúa.
#[test]
fn missing_voltage_is_an_explained_error() {
    let input = QuickInput {
        system: System::SINGLE_127,
        ..QuickInput::default()
    };
    let e = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap_err();
    assert!(e.contains("tres fases"), "{e}");
}

/// Carga continua de una fase: 1500 W, fp 1, 127 V, 10 m, 14 AWG THW.
/// I = 1500 / 127 = 11.81 A; Id = 1.25 · 11.81 = 14.76 A; interruptor de 15 A.
/// 14 AWG: 15 A en terminales de 60 °C y 20 A a 75 °C; 240-4(d) admite 15 A.
/// Tierra para 15 A: 14 AWG. ΔV = 2 · 11.81 · 0.010 · 10.2 = 2.41 V = 1.90 %.
#[test]
fn continuous_load_sets_breaker_and_grounding() {
    let input = QuickInput {
        kind: CircuitKind::LineToNeutral,
        power_w: 1500.0,
        power_factor: 1.0,
        continuous: true,
        length_m: 10.0,
        size: Size::new("14"),
        ..QuickInput::default()
    };
    let r = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    let p = r.protection.as_ref().unwrap();
    close(r.current_a, 11.811, 0.001);
    close(p.design_a, 14.764, 0.001);
    assert_eq!(p.breaker_a, 15.0);
    assert_eq!(p.poles, 1);
    close(p.allowable_ampacity_a, 15.0, 0.001);
    assert_eq!(p.status, Status::Complies);
    assert_eq!(p.grounding_size, Some(Size::new("14")));
    close(r.drop.drop_percent, 1.897, 0.005);
    assert_eq!(r.status, Status::Complies);
}

/// 16 A continuos: Id = 20 A, interruptor de 20 A. El 14 AWG no lo admite
/// (240-4(d): 15 A); el mínimo es 12 AWG y se propone aunque la caída cumpla.
#[test]
fn undersized_conductor_gets_a_larger_size() {
    let input = QuickInput {
        kind: CircuitKind::LineToNeutral,
        load_in_amperes: true,
        current_a: 16.0,
        power_factor: 1.0,
        continuous: true,
        length_m: 5.0,
        size: Size::new("14"),
        ..QuickInput::default()
    };
    let r = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    let p = r.protection.as_ref().unwrap();
    assert_eq!(p.breaker_a, 20.0);
    assert_eq!(p.status, Status::Fails);
    assert_eq!(p.minimum_size, Some(Size::new("12")));
    assert_eq!(r.status, Status::Complies);
    assert_eq!(r.recommendations[0].apply, Some((Size::new("12"), 1)));
}
