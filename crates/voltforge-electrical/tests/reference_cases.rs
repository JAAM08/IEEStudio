//! Casos de referencia con resultado conocido.
//!
//! Los casos marcados «hoja de referencia» reproducen los ejemplos de las
//! hojas de cálculo del Ing. Juan Carlos Vega Mendoza. Los demás se
//! calcularon a mano con las fórmulas y tablas de la norma.

use voltforge_core::{
    CircuitKind, ConduitType, Error, InsulationGroup, Material, RacewayMaterial, Size, System,
};
use voltforge_electrical::conduit::{size_conduit, ConductorGroup, ConduitInput, Covering};
use voltforge_electrical::current::{design_current, nominal_current};
use voltforge_electrical::voltage_drop::{max_length_m, voltage_drop, VoltageDropInput};
use voltforge_rules::Profile;

/// Tolerancia en la unidad mostrada (dos decimales).
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

fn thw(count: u32, size: &str) -> ConductorGroup {
    ConductorGroup {
        count,
        size: Size::new(size),
        covering: Covering::Insulated(InsulationGroup::TwThwThhw),
    }
}

fn bare(count: u32, size: &str) -> ConductorGroup {
    ConductorGroup {
        count,
        size: Size::new(size),
        covering: Covering::Bare,
    }
}

// ------------------------------------------------------------------ corriente

#[test]
fn nominal_current_by_circuit_kind() {
    // Hoja de referencia: 940.8 W, fp 0.8, una fase a 120 V.
    close(
        nominal_current(
            940.8,
            0.8,
            CircuitKind::LineToNeutral,
            &System::SPLIT_240_120,
        )
        .unwrap(),
        9.8,
    );
    // 2,400 W, fp 0.9, dos fases a 220 V: 2400 / (220 · 0.9).
    close(
        nominal_current(2400.0, 0.9, CircuitKind::LineToLine, &System::WYE_220_127).unwrap(),
        12.12,
    );
    // 61,700 W, fp 0.9, tres fases a 220 V: 61700 / (√3 · 220 · 0.9).
    close(
        nominal_current(61_700.0, 0.9, CircuitKind::ThreePhase, &System::WYE_220_127).unwrap(),
        179.91,
    );
}

#[test]
fn design_current_only_raises_the_continuous_part() {
    close(design_current(nom(), 10.0, 0.0), 12.5);
    close(design_current(nom(), 0.0, 10.0), 10.0);
    close(design_current(nom(), 8.0, 6.0), 16.0);
}

#[test]
fn a_circuit_needs_the_voltage_it_connects_to() {
    let err = nominal_current(1000.0, 0.9, CircuitKind::ThreePhase, &System::SINGLE_127);
    assert!(matches!(err, Err(Error::InvalidInput(_))));
    let err = nominal_current(1000.0, 0.9, CircuitKind::LineToNeutral, &System::DELTA_440);
    assert!(matches!(err, Err(Error::InvalidInput(_))));
}

// ----------------------------------------------------------- caída de tensión

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

#[test]
fn voltage_drop_reference_sheet_without_temperature_adjustment() {
    // 2 · 9.8 · 0.050 · (3.9 · 0.8 + 0.207 · 0.6) = 3.18 V; 3.18 / 120 = 2.65 %
    let r = voltage_drop(nom(), &reference_branch()).unwrap();
    close(r.resistance_ohm_km, 3.9);
    close(r.reactance_ohm_km, 0.207);
    close(r.drop_v, 3.18);
    close(r.drop_percent, 2.65);
}

#[test]
fn voltage_drop_reference_sheet_with_resistance_at_60c() {
    // Hoja de referencia: R ajustada 3.711 Ω/km, 3.03 V, 2.53 %.
    let input = VoltageDropInput {
        conductor_temperature_c: Some(60.0),
        ..reference_branch()
    };
    let r = voltage_drop(nom(), &input).unwrap();
    close(r.table_resistance_ohm_km, 3.9);
    close(r.resistance_ohm_km, 3.71);
    close(r.drop_v, 3.03);
    close(r.drop_percent, 2.53);
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
fn three_phase_feeder_and_its_alternatives() {
    // √3 · 179.91 · 0.095 · (0.259 · 0.9 + 0.171 · 0.4359) = 9.11 V; 4.14 %
    let r = voltage_drop(nom(), &feeder("3/0", 1)).unwrap();
    close(r.drop_v, 9.11);
    close(r.drop_percent, 4.14);
    // Calibres mayores con un conductor por fase.
    close(
        voltage_drop(nom(), &feeder("4/0", 1)).unwrap().drop_percent,
        3.49,
    );
    close(
        voltage_drop(nom(), &feeder("250", 1)).unwrap().drop_percent,
        3.15,
    );
    close(
        voltage_drop(nom(), &feeder("300", 1)).unwrap().drop_percent,
        2.77,
    );
    // Dos conductores 1/0 por fase.
    close(
        voltage_drop(nom(), &feeder("1/0", 2)).unwrap().drop_percent,
        2.89,
    );
    // Longitud máxima del 3/0 para 3 %.
    let max = max_length_m(nom(), &feeder("3/0", 1), 3.0).unwrap();
    assert!((max - 68.8).abs() < 0.1, "longitud máxima {max}");
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

// -------------------------------------------------------------------- tubería

#[test]
fn conduit_reference_sheet_five_conductors() {
    // Hoja de referencia: 3 × 12 THW + 1 × 250 kcmil THW + tierra desnuda 6 AWG
    // = 3 · 11.68 + 296.5 + 17.09 = 348.63 mm²; al 40 % resulta 35 mm (1¼").
    for conduit in [
        ConduitType::Emt,
        ConduitType::Imc,
        ConduitType::Rmc,
        ConduitType::Pvc40,
    ] {
        let input = ConduitInput {
            groups: vec![thw(3, "12"), thw(1, "250"), bare(1, "6")],
            conduit,
            nipple: false,
        };
        let r = size_conduit(nom(), &input).unwrap();
        close(r.conductor_area_mm2, 348.63);
        assert_eq!(r.conductor_count, 5);
        assert_eq!(r.allowed_fill_fraction, 0.40);
        assert_eq!(r.conduit.metric_designator, 35, "{conduit}");
        assert_eq!(r.jam_ratio, None);
    }
}

#[test]
fn conduit_fill_depends_on_the_number_of_conductors() {
    let emt = |groups| ConduitInput {
        groups,
        conduit: ConduitType::Emt,
        nipple: false,
    };
    // Un conductor de 500 kcmil THW (509.7 mm²) al 53 %: EMT 35 admite 513 mm².
    let one = size_conduit(nom(), &emt(vec![thw(1, "500")])).unwrap();
    assert_eq!(one.allowed_fill_fraction, 0.53);
    assert_eq!(one.conduit.metric_designator, 35);
    // Dos conductores 2 AWG THW (172 mm²) al 31 %: EMT 27 admite exactamente 172 mm².
    let two = size_conduit(nom(), &emt(vec![thw(2, "2")])).unwrap();
    assert_eq!(two.allowed_fill_fraction, 0.31);
    close(two.conductor_area_mm2, 172.0);
    assert_eq!(two.conduit.metric_designator, 27);
    // Cuatro conductores 8 AWG THW (112.76 mm²) al 40 %: EMT 21 admite 137 mm².
    let four = size_conduit(nom(), &emt(vec![thw(4, "8")])).unwrap();
    assert_eq!(four.allowed_fill_fraction, 0.40);
    close(four.conductor_area_mm2, 112.76);
    assert_eq!(four.conduit.metric_designator, 21);
}

#[test]
fn nipple_allows_sixty_percent() {
    // 12 × 10 AWG THW = 188.16 mm². Al 40 %: EMT 27 (222). Como niple al 60 %: EMT 21 (206).
    let groups = vec![thw(12, "10")];
    let normal = ConduitInput {
        groups: groups.clone(),
        conduit: ConduitType::Emt,
        nipple: false,
    };
    let nipple = ConduitInput {
        groups,
        conduit: ConduitType::Emt,
        nipple: true,
    };
    assert_eq!(
        size_conduit(nom(), &normal)
            .unwrap()
            .conduit
            .metric_designator,
        27
    );
    let r = size_conduit(nom(), &nipple).unwrap();
    assert_eq!(r.allowed_fill_fraction, 0.60);
    assert_eq!(r.conduit.metric_designator, 21);
}

#[test]
fn three_equal_conductors_are_upsized_when_they_can_jam() {
    // 3 × 6 AWG XHHW: d = 6.96 mm, 114.18 mm². Cabe en EMT 21 (137 mm²), pero
    // 20.9 / 6.96 = 3.00 cae entre 2.8 y 3.2, así que se sube a EMT 27.
    let input = ConduitInput {
        groups: vec![ConductorGroup {
            count: 3,
            size: Size::new("6"),
            covering: Covering::Insulated(InsulationGroup::Xhhw),
        }],
        conduit: ConduitType::Emt,
        nipple: false,
    };
    let r = size_conduit(nom(), &input).unwrap();
    assert!(r.upsized_for_jamming);
    assert_eq!(r.conduit.metric_designator, 27);
    close(r.jam_ratio.unwrap(), 26.6 / 6.96);

    // 3 × 8 AWG THW en EMT 21: 20.9 / 5.994 = 3.49, sin riesgo.
    let ok = ConduitInput {
        groups: vec![thw(3, "8")],
        conduit: ConduitType::Emt,
        nipple: false,
    };
    let r = size_conduit(nom(), &ok).unwrap();
    assert!(!r.upsized_for_jamming);
    assert_eq!(r.conduit.metric_designator, 21);
}

#[test]
fn conduit_reports_when_nothing_fits() {
    let input = ConduitInput {
        groups: vec![thw(12, "1000")],
        conduit: ConduitType::Emt,
        nipple: false,
    };
    assert!(matches!(
        size_conduit(nom(), &input),
        Err(Error::NoConduitFits { .. })
    ));
    let empty = ConduitInput {
        groups: vec![],
        conduit: ConduitType::Emt,
        nipple: false,
    };
    assert!(matches!(
        size_conduit(nom(), &empty),
        Err(Error::InvalidInput(_))
    ));
}

// ---------------------------------------------------------------------- tablas

#[test]
fn embedded_tables_are_complete_and_consistent() {
    let p = nom();
    assert_eq!(p.sizes.len(), 30);
    assert_eq!(p.ampacity.len(), 30);
    assert_eq!(p.standard_ratings.len(), 40);
    assert_eq!(p.grounding.len(), 19);
    assert_eq!(p.temperature_correction.len(), 16);
    assert_eq!(p.bundling_adjustment.len(), 6);
    // Los calibres van en orden creciente de área.
    assert!(p.sizes.windows(2).all(|w| w[0].area_mm2 < w[1].area_mm2));
    assert!(p.size_rank(&Size::new("1/0")).unwrap() > p.size_rank(&Size::new("1")).unwrap());
    for conduit in ConduitType::ALL {
        let rows: Vec<_> = p.conduit_rows(conduit).collect();
        assert!(rows.len() >= 10, "{conduit}");
        assert!(
            rows.windows(2)
                .all(|w| w[0].area_100_mm2 < w[1].area_100_mm2),
            "{conduit}"
        );
        for r in rows {
            let circle = std::f64::consts::PI / 4.0 * r.internal_diameter_mm.powi(2);
            assert!(
                (r.area_100_mm2 - circle).abs() / circle < 0.01,
                "{conduit} {r:?}"
            );
            assert!(
                (r.area_40_mm2 - 0.40 * r.area_100_mm2).abs() <= 1.5,
                "{conduit} {r:?}"
            );
        }
    }
    // La errata del calibre 10 en la Tabla 5 está corregida.
    close(
        p.insulated(InsulationGroup::TwThwThhw, &Size::new("10"))
            .unwrap()
            .area_mm2,
        15.68,
    );
}
