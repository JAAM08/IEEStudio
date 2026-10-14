//! Pruebas normativas: que la selección de conductor, interruptor y puesta
//! a tierra, y las advertencias, correspondan a las reglas del perfil
//! NOM-001-SEDE-2012.
//!
//! Los resultados esperados se calcularon a mano con las tablas
//! 310-15(b)(16), 310-15(b)(2)(a), 310-15(b)(3)(a) y 250-122.

use ieestudio_core::Insulation;
use ieestudio_core::{CircuitKind, Error, Material, RacewayMaterial, Size, System, TempRating};
use ieestudio_electrical::sizing::{
    check_conductor, select_conductor, standard_rating_at_least, Governing, SizingInput,
    VoltageDropLimit,
};
use ieestudio_electrical::voltage_drop::VoltageDropInput;
use ieestudio_rules::Profile;

fn nom() -> &'static Profile {
    Profile::nom_001_sede_2012()
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 0.01,
        "se esperaba {expected}, se obtuvo {actual}"
    );
}

/// Circuito de cobre THW a 30 °C, con dos conductores portadores de corriente.
fn base(continuous_a: f64, noncontinuous_a: f64) -> SizingInput {
    SizingInput {
        continuous_a,
        noncontinuous_a,
        material: Material::Copper,
        grounding_material: Material::Copper,
        insulation: Insulation::Thw,
        wet_location: false,
        ambient_c: 30.0,
        current_carrying_conductors: 2,
        nipple: false,
        conductors_per_phase: 1,
        terminal_rating: None,
        include_iec_ratings: false,
        minimum_size: None,
        voltage_drop: None,
        design_a: None,
        breaker_a: None,
        motor_protection: false,
    }
}

fn drop_limit(kind: CircuitKind, current_a: f64, length_m: f64, limit: f64) -> VoltageDropLimit {
    VoltageDropLimit {
        input: VoltageDropInput {
            kind,
            system: System::WYE_220_127,
            current_a,
            length_m,
            power_factor: 0.9,
            size: Size::new("14"),
            material: Material::Copper,
            raceway: RacewayMaterial::Steel,
            conductors_per_phase: 1,
            conductor_temperature_c: None,
        },
        limit_percent: limit,
    }
}

#[test]
fn standard_ratings_skip_the_iec_series_by_default() {
    close(
        standard_rating_at_least(nom(), 13.125, false).unwrap(),
        15.0,
    );
    close(standard_rating_at_least(nom(), 15.75, false).unwrap(), 20.0);
    close(standard_rating_at_least(nom(), 15.75, true).unwrap(), 16.0);
    close(
        standard_rating_at_least(nom(), 179.91, false).unwrap(),
        200.0,
    );
    close(
        standard_rating_at_least(nom(), 200.0, false).unwrap(),
        200.0,
    );
    assert!(standard_rating_at_least(nom(), 7000.0, false).is_err());
}

#[test]
fn continuous_lighting_circuit() {
    // 10.5 A continuos: diseño 13.125 A, interruptor de 15 A.
    // 14 AWG a 60 °C: 15 A ≥ 13.125 A; 240-4(d) permite 15 A.
    let r = select_conductor(nom(), &base(10.5, 0.0)).unwrap();
    close(r.design_a, 13.125);
    close(r.breaker_a, 15.0);
    assert_eq!(r.conductor.size, Size::new("14"));
    assert_eq!(r.conductor.terminal_rating, TempRating::C60);
    close(r.conductor.allowable_ampacity_a, 15.0);
    assert_eq!(r.governing, Governing::Ampacity);
    assert_eq!(r.grounding.size, Size::new("14"));
}

#[test]
fn voltage_drop_raises_the_conductor_and_the_grounding_conductor() {
    // 15.75 A no continuos, 22 m a 127 V. Por ampacidad: interruptor de 20 A y 12 AWG.
    let by_ampacity = select_conductor(nom(), &base(0.0, 15.75)).unwrap();
    close(by_ampacity.breaker_a, 20.0);
    assert_eq!(by_ampacity.conductor.size, Size::new("12"));
    assert_eq!(by_ampacity.grounding.size, Size::new("12"));

    // Con límite de 3 %: el 12 AWG da 3.29 % y el 10 AWG 1.96 %.
    let mut input = base(0.0, 15.75);
    input.voltage_drop = Some(drop_limit(CircuitKind::LineToNeutral, 15.75, 22.0, 3.0));
    let twelve = check_conductor(nom(), &input, &Size::new("12"), 20.0).unwrap();
    close(twelve.voltage_drop_percent.unwrap(), 3.29);
    assert_eq!(twelve.meets_voltage_drop, Some(false));
    assert!(!twelve.passes());

    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("10"));
    close(r.conductor.voltage_drop_percent.unwrap(), 1.96);
    assert_eq!(r.ampacity_size, Size::new("12"));
    assert_eq!(r.governing, Governing::VoltageDrop);
    // 250-122(b): la tierra crece en la proporción 5.261 / 3.31, de 12 a 10 AWG.
    assert_eq!(r.grounding.table_size, Size::new("12"));
    assert_eq!(r.grounding.size, Size::new("10"));
    assert!(r.grounding.increased);
}

#[test]
fn three_phase_feeder() {
    // 179.91 A: interruptor de 200 A, terminales a 75 °C, 3/0 AWG (200 A).
    let mut input = base(0.0, 179.91);
    input.current_carrying_conductors = 3;
    let r = select_conductor(nom(), &input).unwrap();
    close(r.breaker_a, 200.0);
    assert_eq!(r.conductor.terminal_rating, TempRating::C75);
    assert_eq!(r.conductor.size, Size::new("3/0"));
    assert_eq!(r.grounding.size, Size::new("6"));

    // Con 95 m y límite de 3 % sube a 300 kcmil (2.77 %). La tierra crece
    // 13.3 · 152 / 85.01 = 23.78 mm², que es 3 AWG (26.67 mm²).
    input.voltage_drop = Some(drop_limit(CircuitKind::ThreePhase, 179.91, 95.0, 3.0));
    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("300"));
    close(r.conductor.voltage_drop_percent.unwrap(), 2.77);
    assert_eq!(r.governing, Governing::VoltageDrop);
    assert_eq!(r.grounding.size, Size::new("3"));
}

#[test]
fn ambient_temperature_and_bundling_use_the_insulation_column() {
    // 40 A continuos, THHN (90 °C), 38 °C ambiente, 6 conductores.
    // Diseño 50 A, interruptor 50 A, terminales 60 °C.
    // 8 AWG: 40 A a 60 °C < 50 A. 6 AWG: 55 A ≥ 50 A; corregida 75 · 0.91 · 0.80 = 54.6 A.
    let mut input = base(40.0, 0.0);
    input.insulation = Insulation::Thhn;
    input.ambient_c = 38.0;
    input.current_carrying_conductors = 6;
    let r = select_conductor(nom(), &input).unwrap();
    close(r.breaker_a, 50.0);
    assert_eq!(r.conductor.size, Size::new("6"));
    assert_eq!(r.conductor.insulation_rating, TempRating::C90);
    close(r.conductor.temperature_factor, 0.91);
    close(r.conductor.bundling_factor, 0.80);
    close(r.conductor.corrected_ampacity_a, 54.6);
    close(r.conductor.terminal_ampacity_a, 55.0);
    close(r.conductor.allowable_ampacity_a, 54.6);
    assert_eq!(r.grounding.size, Size::new("10"));
}

#[test]
fn bundling_can_govern_and_a_nipple_removes_it() {
    // 30 A, THW, 12 conductores (factor 0.50): 10 AWG 17.5 A, 8 AWG 25 A, 6 AWG 32.5 A.
    let mut input = base(0.0, 30.0);
    input.current_carrying_conductors = 12;
    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("6"));
    close(r.conductor.corrected_ampacity_a, 32.5);
    // En un niple no se aplica el factor: 10 AWG con 30 A a 60 °C.
    input.nipple = true;
    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("10"));
    close(r.conductor.bundling_factor, 1.0);
}

#[test]
fn next_standard_rating_may_protect_the_conductor() {
    // 52 A: interruptor de 60 A. 6 AWG a 60 °C lleva 55 A; 240-4(b) admite
    // el valor normalizado inmediato superior, que es 60 A.
    let r = select_conductor(nom(), &base(0.0, 52.0)).unwrap();
    close(r.breaker_a, 60.0);
    assert_eq!(r.conductor.size, Size::new("6"));
    close(r.conductor.allowable_ampacity_a, 55.0);
    assert!(r.conductor.protected);
}

#[test]
fn small_conductor_limit_can_govern() {
    // 16 A con terminales marcadas a 75 °C: el 14 AWG lleva 20 A, pero
    // 240-4(d) lo limita a 15 A y el interruptor es de 20 A. Resulta 12 AWG.
    let mut input = base(0.0, 16.0);
    input.terminal_rating = Some(TempRating::C75);
    let fourteen = check_conductor(nom(), &input, &Size::new("14"), 20.0).unwrap();
    assert!(fourteen.meets_ampacity());
    assert!(!fourteen.protected);
    assert_eq!(fourteen.small_conductor_limit_a, Some(15.0));
    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("12"));
    assert_eq!(r.governing, Governing::Protection);
}

#[test]
fn parallel_conductors_start_at_one_aught() {
    // 400 A con dos conductores por fase: 200 A cada uno, 3/0 AWG a 75 °C.
    let mut input = base(0.0, 400.0);
    input.conductors_per_phase = 2;
    input.current_carrying_conductors = 3;
    let r = select_conductor(nom(), &input).unwrap();
    close(r.breaker_a, 400.0);
    assert_eq!(r.conductor.size, Size::new("3/0"));
    close(r.conductor.allowable_ampacity_a, 400.0);
    // 100 A con dos por fase: por ampacidad bastaría 6 AWG, pero el mínimo
    // en paralelo es 1/0 AWG.
    input.noncontinuous_a = 100.0;
    let six = check_conductor(nom(), &input, &Size::new("6"), 100.0).unwrap();
    assert!(six.meets_load && !six.parallel_allowed);
    assert_eq!(
        select_conductor(nom(), &input).unwrap().conductor.size,
        Size::new("1/0")
    );
}

#[test]
fn user_minimum_is_respected() {
    let mut input = base(10.5, 0.0);
    input.minimum_size = Some(Size::new("10"));
    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("10"));
    assert_eq!(r.governing, Governing::UserMinimum);
    assert_eq!(r.ampacity_size, Size::new("14"));
}

#[test]
fn aluminum_phase_conductors_with_copper_grounding() {
    // 100 A en aluminio XHHW-2: 1/0 AWG lleva 100 A a 60 °C. Tierra de cobre 8 AWG.
    let mut input = base(0.0, 100.0);
    input.material = Material::Aluminum;
    input.insulation = Insulation::Xhhw2;
    let r = select_conductor(nom(), &input).unwrap();
    assert_eq!(r.conductor.size, Size::new("1/0"));
    assert_eq!(r.grounding.size, Size::new("8"));
    // La Tabla 250-122 no da tierra de aluminio para 100 A.
    input.grounding_material = Material::Aluminum;
    assert!(matches!(
        select_conductor(nom(), &input),
        Err(Error::NotInTable { .. })
    ));
}

#[test]
fn conditions_the_tables_do_not_cover_are_reported() {
    // THHN no se permite en lugares mojados.
    let mut wet = base(0.0, 20.0);
    wet.insulation = Insulation::Thhn;
    wet.wet_location = true;
    assert!(matches!(
        select_conductor(nom(), &wet),
        Err(Error::InvalidInput(_))
    ));
    // TW (60 °C) no tiene factor de corrección a 58 °C ambiente.
    let mut hot = base(0.0, 20.0);
    hot.insulation = Insulation::Tw;
    hot.ambient_c = 58.0;
    assert!(matches!(
        select_conductor(nom(), &hot),
        Err(Error::NotInTable { .. })
    ));
}

/// 430-6(a)(1): la columna de la tabla depende de la tensión del sistema y
/// un caso fuera de la tabla no se supone.
#[test]
fn motor_current_comes_from_the_right_column() {
    use ieestudio_electrical::motor::full_load_current;
    let p = nom();
    let at =
        |hp: &str, three: bool, v: f64| full_load_current(p, hp, three, v).map(|m| m.full_load_a);
    assert_eq!(at("10", true, 220.0), Ok(28.0)); // columna de 230 V
    assert_eq!(at("10", true, 240.0), Ok(28.0));
    assert_eq!(at("10", true, 208.0), Ok(30.8));
    assert_eq!(at("10", true, 440.0), Ok(14.0)); // columna de 460 V
    assert_eq!(at("10", true, 480.0), Ok(14.0));
    assert_eq!(at("1", false, 127.0), Ok(14.0)); // columna de 127 V, propia de la NOM
    assert_eq!(at("1", false, 120.0), Ok(16.0)); // columna de 115 V
    assert!(matches!(
        at("10", true, 380.0),
        Err(Error::NotInTable { .. })
    ));
    assert!(matches!(
        at("3", true, 115.0),
        Err(Error::NotInTable { .. })
    ));
    assert!(matches!(
        at("15", false, 220.0),
        Err(Error::NotInTable { .. })
    ));
    assert!(matches!(
        at("13", true, 220.0),
        Err(Error::NotInTable { .. })
    ));
}

/// Tabla 430-52 y su excepción 1: el valor normalizado inmediato superior.
#[test]
fn motor_breaker_may_round_up_to_the_next_standard() {
    use ieestudio_core::MotorType;
    use ieestudio_electrical::motor::motor_branch;
    let p = nom();
    let b = motor_branch(p, 42.0, Some(MotorType::SquirrelCage), false).unwrap();
    assert_eq!(b.protection_percent, 250.0);
    assert_eq!(b.breaker_max_a, 110.0); // 105 A no es normalizado
    let w = motor_branch(p, 28.0, Some(MotorType::WoundRotor), false).unwrap();
    assert_eq!(w.protection_percent, 150.0);
    assert_eq!(w.breaker_max_a, 45.0); // 42 A → 45 A
    let one = motor_branch(p, 14.0, None, false).unwrap();
    assert_eq!(one.breaker_max_a, 35.0);
}

/// 440-22(a): el motocompresor no redondea hacia arriba y no se exige
/// menos de 15 A; la placa del equipo manda si la trae (440-22(c), 440-35).
#[test]
fn compressor_protection_rounds_down_with_a_floor() {
    use ieestudio_electrical::motor::{compressor_branch, CompressorNameplate};
    let p = nom();
    let rla = |a: f64| CompressorNameplate {
        rated_load_a: Some(a),
        ..Default::default()
    };
    assert_eq!(
        compressor_branch(p, &rla(16.7), false)
            .unwrap()
            .breaker_max_a,
        25.0
    );
    assert_eq!(
        compressor_branch(p, &rla(5.0), false)
            .unwrap()
            .breaker_max_a,
        15.0
    );
    let plate = CompressorNameplate {
        minimum_circuit_a: Some(18.0),
        maximum_protection_a: Some(30.0),
        ..Default::default()
    };
    let c = compressor_branch(p, &plate, false).unwrap();
    assert_eq!((c.conductor_a, c.breaker_max_a), (18.0, 30.0));
    assert!(c.from_equipment_nameplate);
    assert!(compressor_branch(p, &CompressorNameplate::default(), false).is_err());
}

/// 250-122(a): la puesta a tierra no se exige mayor que los conductores de
/// fase, aunque el interruptor del motor la pediría más grande.
#[test]
fn grounding_never_exceeds_the_phase_conductor() {
    use ieestudio_electrical::sizing::equipment_grounding;
    let g = equipment_grounding(
        nom(),
        35.0,
        Material::Copper,
        true,
        &Size::new("12"),
        &Size::new("12"),
    )
    .unwrap();
    assert_eq!(g.table_size, Size::new("10"));
    assert_eq!(g.size, Size::new("12"));
}
