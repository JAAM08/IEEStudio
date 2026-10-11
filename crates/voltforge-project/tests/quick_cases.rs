//! Casos del cálculo rápido de caída de tensión, con resultados calculados a mano.

use voltforge_core::{CircuitKind, Profile, Size, System};
use voltforge_project::quick::{quick_voltage_drop, QuickInput};
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
    let r = quick_voltage_drop(Profile::nom_001_sede_2012(), &QuickInput::default()).unwrap();
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
    // Ampacidad de tabla a 75 °C: 2 · 150 = 300 A, cubre 179.91 A.
    let parallel = &r.recommendations[1];
    assert_eq!(parallel.apply, Some((Size::new("1/0"), 2)));
    assert_eq!(parallel.value, "2.89 %");
    assert_eq!(r.recommendations.len(), 3);
    assert_eq!(r.steps.len(), 5);
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
    let r = quick_voltage_drop(Profile::nom_001_sede_2012(), &input).unwrap();
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
    let r = quick_voltage_drop(Profile::nom_001_sede_2012(), &input).unwrap();
    assert_eq!(r.status, Status::Complies);
    assert_eq!(r.recommendations.len(), 1);
    assert!(r.recommendations[0].apply.is_none());
    assert!(r.warnings.is_empty());
}

/// Paralelo por debajo de 1/0 AWG y corriente sobre la ampacidad de tabla se avisan.
#[test]
fn warns_about_small_parallel_and_ampacity() {
    let input = QuickInput {
        size: Size::new("8"),
        conductors_per_phase: 2,
        ..QuickInput::default()
    };
    let r = quick_voltage_drop(Profile::nom_001_sede_2012(), &input).unwrap();
    // 8 AWG de cobre a 75 °C: 50 A; dos por fase 100 A, menos que 179.91 A.
    assert_eq!(r.table_ampacity_a, Some(100.0));
    assert_eq!(r.warnings.len(), 2);
}

/// Un sistema sin la tensión que el circuito requiere no se evalúa.
#[test]
fn missing_voltage_is_an_explained_error() {
    let input = QuickInput {
        system: System::SINGLE_127,
        ..QuickInput::default()
    };
    let e = quick_voltage_drop(Profile::nom_001_sede_2012(), &input).unwrap_err();
    assert!(e.contains("tres fases"), "{e}");
}
