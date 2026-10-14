//! Pruebas matemáticas: ecuaciones, unidades, linealidad y casos límite.
//!
//! No comparan contra la norma sino contra la física: que la fórmula
//! aproximada coincida con la exacta, que la caída sea proporcional a la
//! longitud, que los datos inválidos se rechacen y que lo que las tablas no
//! cubren se informe en lugar de suponerse.

use ieestudio_core::units::{m_to_km, percent, SQRT_3};
use ieestudio_core::{CircuitKind, Error, Material, RacewayMaterial, Size, System};
use ieestudio_electrical::current::{design_current, nominal_current};
use ieestudio_electrical::voltage_drop::{max_length_m, voltage_drop, VoltageDropInput};
use ieestudio_rules::Profile;

const TOL: f64 = 0.01;

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOL,
        "se esperaba {expected}, se obtuvo {actual}"
    );
}

fn nom() -> &'static Profile {
    Profile::nom_001_sede_2012()
}

fn reference_branch() -> VoltageDropInput {
    VoltageDropInput {
        kind: CircuitKind::LineToNeutral,
        system: System::SPLIT_240_120,
        current_a: 9.8,
        length_m: 50.0,
        power_factor: 0.8,
        size: Size::new("10"),
        material: Material::Copper,
        raceway: RacewayMaterial::Steel,
        conductors_per_phase: 1,
        conductor_temperature_c: None,
    }
}

fn feeder(size: &str, per_phase: u32) -> VoltageDropInput {
    VoltageDropInput {
        kind: CircuitKind::ThreePhase,
        system: System::WYE_220_127,
        current_a: nominal_current(61_700.0, 0.9, CircuitKind::ThreePhase, &System::WYE_220_127)
            .unwrap(),
        length_m: 95.0,
        power_factor: 0.9,
        size: Size::new(size),
        material: Material::Copper,
        raceway: RacewayMaterial::Steel,
        conductors_per_phase: per_phase,
        conductor_temperature_c: None,
    }
}

#[test]
fn unit_helpers() {
    assert!((SQRT_3 * SQRT_3 - 3.0).abs() < 1e-12);
    assert_eq!(m_to_km(95.0), 0.095);
    assert_eq!(percent(9.11, 220.0), 9.11 / 220.0 * 100.0);
}

#[test]
fn nominal_current_and_power_are_consistent() {
    // P = √3 · V · I · fp en trifásico; P = V · I · fp en los demás.
    let system = System::WYE_220_127;
    let i3 = nominal_current(10_000.0, 0.85, CircuitKind::ThreePhase, &system).unwrap();
    assert!((SQRT_3 * 220.0 * i3 * 0.85 - 10_000.0).abs() < 1e-9);
    let i1 = nominal_current(1_000.0, 0.9, CircuitKind::LineToNeutral, &system).unwrap();
    assert!((127.0 * i1 * 0.9 - 1_000.0).abs() < 1e-9);
    let i2 = nominal_current(1_000.0, 1.0, CircuitKind::LineToLine, &system).unwrap();
    assert!((220.0 * i2 - 1_000.0).abs() < 1e-9);
}

#[test]
fn invalid_inputs_are_rejected() {
    let system = System::WYE_220_127;
    for pf in [0.0, -0.5, 1.01, f64::NAN] {
        assert!(nominal_current(1_000.0, pf, CircuitKind::ThreePhase, &system).is_err());
    }
    assert!(nominal_current(-1.0, 0.9, CircuitKind::ThreePhase, &system).is_err());
    let negative = VoltageDropInput {
        length_m: -1.0,
        ..reference_branch()
    };
    assert!(matches!(
        voltage_drop(nom(), &negative),
        Err(Error::InvalidInput(_))
    ));
    let none = VoltageDropInput {
        conductors_per_phase: 0,
        ..reference_branch()
    };
    assert!(voltage_drop(nom(), &none).is_err());
}

#[test]
fn zero_length_or_current_gives_zero_drop() {
    let short = VoltageDropInput {
        length_m: 0.0,
        ..reference_branch()
    };
    assert_eq!(voltage_drop(nom(), &short).unwrap().drop_v, 0.0);
    let idle = VoltageDropInput {
        current_a: 0.0,
        ..reference_branch()
    };
    assert_eq!(voltage_drop(nom(), &idle).unwrap().drop_v, 0.0);
    // Sin caída no hay longitud máxima.
    assert!(max_length_m(nom(), &idle, 3.0).unwrap().is_infinite());
}

#[test]
fn unity_power_factor_leaves_only_resistance() {
    // Con fp = 1, sen θ = 0: ΔV = 2 · I · L · R.
    let input = VoltageDropInput {
        power_factor: 1.0,
        ..reference_branch()
    };
    let r = voltage_drop(nom(), &input).unwrap();
    assert!((r.drop_v - 2.0 * 9.8 * 0.050 * 3.9).abs() < 1e-9);
}

#[test]
fn max_length_reaches_exactly_the_limit() {
    let input = feeder("3/0", 1);
    let max = max_length_m(nom(), &input, 3.0).unwrap();
    let at_max = VoltageDropInput {
        length_m: max,
        ..input
    };
    assert!((voltage_drop(nom(), &at_max).unwrap().drop_percent - 3.0).abs() < 1e-9);
}

#[test]
fn design_current_only_raises_the_continuous_part() {
    close(design_current(nom(), 10.0, 0.0), 12.5);
    close(design_current(nom(), 0.0, 10.0), 10.0);
    close(design_current(nom(), 8.0, 6.0), 16.0);
}

#[test]
fn exact_formula_agrees_with_the_table_9_formula() {
    for input in [reference_branch(), feeder("3/0", 1), feeder("1/0", 2)] {
        let r = voltage_drop(nom(), &input).unwrap();
        assert!(
            (r.exact_drop_v - r.drop_v).abs() < 0.02,
            "exacta {} contra aproximada {}",
            r.exact_drop_v,
            r.drop_v
        );
    }
}

#[test]
fn voltage_drop_scales_with_length_and_parallel_conductors() {
    let base = voltage_drop(nom(), &feeder("3/0", 1)).unwrap().drop_v;
    let double = VoltageDropInput {
        length_m: 190.0,
        ..feeder("3/0", 1)
    };
    close(voltage_drop(nom(), &double).unwrap().drop_v, 2.0 * base);
    close(
        voltage_drop(nom(), &feeder("3/0", 2)).unwrap().drop_v,
        base / 2.0,
    );
}

#[test]
fn missing_table_values_are_reported_not_guessed() {
    // La Tabla 9 no tiene aluminio menor a 6 AWG ni calibres de 700 kcmil.
    let al = VoltageDropInput {
        material: Material::Aluminum,
        ..reference_branch()
    };
    assert!(matches!(
        voltage_drop(nom(), &al),
        Err(Error::NotInTable { .. })
    ));
    let big = VoltageDropInput {
        size: Size::new("700"),
        ..reference_branch()
    };
    assert!(matches!(
        voltage_drop(nom(), &big),
        Err(Error::NotInTable { .. })
    ));
    let unknown = VoltageDropInput {
        size: Size::new("13"),
        ..reference_branch()
    };
    assert!(matches!(
        voltage_drop(nom(), &unknown),
        Err(Error::UnknownSize(_))
    ));
}

/// Motores: los límites son productos simples de la corriente por los
/// factores del perfil, sin redondear.
#[test]
fn motor_limits_are_plain_products() {
    use ieestudio_core::MotorType;
    use ieestudio_electrical::motor::{
        compressor_branch, motor_branch, overload_limit, CompressorNameplate,
    };
    let p = Profile::nom_001_sede_2012();
    let b = motor_branch(p, 42.0, Some(MotorType::SquirrelCage), false).unwrap();
    assert!((b.conductor_a - 52.5).abs() < 1e-9); // 1.25 × 42
    assert!((b.protection_limit_a - 105.0).abs() < 1e-9); // 2.5 × 42
    assert!((b.starting_limit_a - 168.0).abs() < 1e-9); // 4 × 42
    let big = motor_branch(p, 124.0, Some(MotorType::SquirrelCage), false).unwrap();
    assert!((big.starting_limit_a - 372.0).abs() < 1e-9); // 3 × 124, más de 100 A
    let (factor, limit) = overload_limit(p, 26.0, true);
    assert_eq!(factor, 1.25);
    assert!((limit - 32.5).abs() < 1e-9);
    let (factor, limit) = overload_limit(p, 26.0, false);
    assert_eq!(factor, 1.15);
    assert!((limit - 29.9).abs() < 1e-9);
    let c = compressor_branch(
        p,
        &CompressorNameplate {
            rated_load_a: Some(10.0),
            branch_selection_a: Some(12.0),
            ..Default::default()
        },
        false,
    )
    .unwrap();
    assert_eq!(c.basis_a, 12.0); // la mayor de las dos corrientes
    assert!((c.conductor_a - 15.0).abs() < 1e-9);
    assert!((c.protection_limit_a - 21.0).abs() < 1e-9);
    assert_eq!(c.starting_limit_a, Some(27.0));
}
