//! Casos de referencia de `fixtures/engineering-cases/`: cada fila es un
//! cálculo hecho fuera del programa, con su fuente, que el motor debe
//! reproducir dentro de la tolerancia indicada.

use voltforge_core::{
    CircuitKind, ConduitType, Insulation, InsulationGroup, Material, RacewayMaterial, Size, System,
};
use voltforge_electrical::conduit::{size_conduit, ConductorGroup, ConduitInput, Covering};
use voltforge_electrical::current::nominal_current;
use voltforge_electrical::sizing::{select_conductor, SizingInput};
use voltforge_electrical::voltage_drop::{voltage_drop, VoltageDropInput};
use voltforge_rules::csv::{parse, Row};
use voltforge_rules::Profile;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/engineering-cases/nom-001-sede-2012/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("no se pudo leer {path}: {e}"))
}

fn nom() -> &'static Profile {
    Profile::nom_001_sede_2012()
}

fn kind(text: &str) -> CircuitKind {
    match text {
        "line_to_neutral" => CircuitKind::LineToNeutral,
        "line_to_line" => CircuitKind::LineToLine,
        "three_phase" => CircuitKind::ThreePhase,
        other => panic!("conexión desconocida: {other}"),
    }
}

fn material(text: &str) -> Material {
    match text {
        "copper" => Material::Copper,
        "aluminum" => Material::Aluminum,
        other => panic!("material desconocido: {other}"),
    }
}

fn insulation(text: &str) -> Insulation {
    use Insulation::*;
    match text {
        "TW" => Tw,
        "THW" => Thw,
        "THHW" => Thhw,
        "THW-2" => Thw2,
        "THHN" => Thhn,
        "THWN" => Thwn,
        "THWN-2" => Thwn2,
        "XHHW" => Xhhw,
        "XHHW-2" => Xhhw2,
        "RHH" => Rhh,
        "RHW" => Rhw,
        "RHW-2" => Rhw2,
        other => panic!("aislamiento desconocido: {other}"),
    }
}

fn yes(row: &Row, column: &str) -> bool {
    row.text(column) == "yes"
}

/// Compara un valor esperado opcional; devuelve la descripción del fallo.
fn compare(
    failures: &mut Vec<String>,
    id: &str,
    what: &str,
    expected: Option<f64>,
    actual: f64,
    tolerance: f64,
) {
    if let Some(expected) = expected {
        if (actual - expected).abs() > tolerance {
            failures.push(format!(
                "{id}: {what} esperado {expected}, calculado {actual:.4}"
            ));
        }
    }
}

#[test]
fn voltage_drop_cases() {
    let mut failures = Vec::new();
    let rows = parse("voltage_drop.csv", &fixture("voltage_drop.csv"), |r| {
        let system = System {
            line_to_line_v: r.opt("line_to_line_v"),
            line_to_neutral_v: r.opt("line_to_neutral_v"),
        };
        let kind = kind(&r.text("kind"));
        let power_factor: f64 = r.get("power_factor");
        let current_a = r.opt("current_a").unwrap_or_else(|| {
            nominal_current(r.get("power_w"), power_factor, kind, &system).unwrap()
        });
        let input = VoltageDropInput {
            kind,
            system,
            current_a,
            length_m: r.get("length_m"),
            power_factor,
            size: Size::new(&r.text("size")),
            material: material(&r.text("material")),
            raceway: match r.text("raceway").as_str() {
                "steel" => RacewayMaterial::Steel,
                "pvc" => RacewayMaterial::Pvc,
                "aluminum" => RacewayMaterial::Aluminum,
                other => panic!("canalización desconocida: {other}"),
            },
            conductors_per_phase: r.get("conductors_per_phase"),
            conductor_temperature_c: r.opt("conductor_temperature_c"),
        };
        (
            r.text("id"),
            input,
            r.opt::<f64>("expected_drop_v"),
            r.opt::<f64>("expected_drop_percent"),
            r.get::<f64>("tolerance"),
        )
    });
    assert!(!rows.is_empty());
    for (id, input, volts, percent, tolerance) in rows {
        let r = voltage_drop(nom(), &input).unwrap();
        compare(&mut failures, &id, "ΔV (V)", volts, r.drop_v, tolerance);
        compare(
            &mut failures,
            &id,
            "e (%)",
            percent,
            r.drop_percent,
            tolerance,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn conductor_sizing_cases() {
    let mut failures = Vec::new();
    let rows = parse(
        "conductor_sizing.csv",
        &fixture("conductor_sizing.csv"),
        |r| {
            let mat = material(&r.text("material"));
            let input = SizingInput {
                continuous_a: r.get("continuous_a"),
                noncontinuous_a: r.get("noncontinuous_a"),
                material: mat,
                grounding_material: Material::Copper,
                insulation: insulation(&r.text("insulation")),
                wet_location: false,
                ambient_c: r.get("ambient_c"),
                current_carrying_conductors: r.get("current_carrying"),
                nipple: yes(r, "nipple"),
                conductors_per_phase: r.get("conductors_per_phase"),
                terminal_rating: None,
                include_iec_ratings: false,
                minimum_size: None,
                voltage_drop: None,
                motor: None,
            };
            (
                r.text("id"),
                input,
                r.opt::<f64>("expected_design_a"),
                r.opt::<f64>("expected_breaker_a"),
                r.opt_text("expected_size"),
                r.opt::<f64>("expected_allowable_a"),
                r.opt_text("expected_grounding"),
            )
        },
    );
    assert!(!rows.is_empty());
    for (id, input, design, breaker, size, allowable, grounding) in rows {
        let r = select_conductor(nom(), &input).unwrap();
        compare(
            &mut failures,
            &id,
            "corriente de diseño",
            design,
            r.design_a,
            0.01,
        );
        compare(&mut failures, &id, "interruptor", breaker, r.breaker_a, 0.0);
        compare(
            &mut failures,
            &id,
            "ampacidad permitida",
            allowable,
            r.conductor.allowable_ampacity_a,
            0.01,
        );
        if let Some(size) = size {
            if r.conductor.size != Size::new(&size) {
                failures.push(format!(
                    "{id}: calibre esperado {size}, calculado {}",
                    r.conductor.size
                ));
            }
        }
        if let Some(ground) = grounding {
            if r.grounding.size != Size::new(&ground) {
                failures.push(format!(
                    "{id}: tierra esperada {ground}, calculada {}",
                    r.grounding.size
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `3x12 THW; 1x250 THW; 1x6 desnudo`
fn conductors(text: &str) -> Vec<ConductorGroup> {
    text.split(';')
        .map(|part| {
            let mut words = part.split_whitespace();
            let first = words.next().expect("grupo vacío");
            let (count, size) = first.split_once('x').expect("formato NxCALIBRE");
            let covering = match words.next() {
                Some("desnudo") => Covering::Bare,
                Some(name) => Covering::Insulated(match insulation(name).dimension_group() {
                    g @ (InsulationGroup::TwThwThhw
                    | InsulationGroup::ThhnThwn
                    | InsulationGroup::Xhhw
                    | InsulationGroup::RhhRhw) => g,
                }),
                None => panic!("falta el aislamiento en «{part}»"),
            };
            ConductorGroup {
                count: count.parse().expect("cantidad"),
                size: Size::new(size),
                covering,
            }
        })
        .collect()
}

#[test]
fn conduit_cases() {
    let mut failures = Vec::new();
    let rows = parse("conduit.csv", &fixture("conduit.csv"), |r| {
        let conduit = match r.text("conduit").as_str() {
            "EMT" => ConduitType::Emt,
            "IMC" => ConduitType::Imc,
            "RMC" => ConduitType::Rmc,
            "PVC40" => ConduitType::Pvc40,
            "PVC80" => ConduitType::Pvc80,
            other => panic!("tubo desconocido: {other}"),
        };
        (
            r.text("id"),
            ConduitInput {
                groups: conductors(&r.text("conductors")),
                conduit,
                nipple: yes(r, "nipple"),
            },
            r.opt::<f64>("expected_area_mm2"),
            r.opt::<f64>("expected_fill_percent_allowed"),
            r.opt::<u32>("expected_designator_mm"),
        )
    });
    assert!(!rows.is_empty());
    for (id, input, area, fill, designator) in rows {
        let r = size_conduit(nom(), &input).unwrap();
        compare(
            &mut failures,
            &id,
            "área (mm²)",
            area,
            r.conductor_area_mm2,
            0.01,
        );
        compare(
            &mut failures,
            &id,
            "ocupación permitida (%)",
            fill,
            r.allowed_fill_fraction * 100.0,
            0.001,
        );
        if let Some(d) = designator {
            if r.conduit.metric_designator != d {
                failures.push(format!(
                    "{id}: tubo esperado {d} mm, calculado {} mm",
                    r.conduit.metric_designator
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn motor_cases() {
    use voltforge_core::MotorType;
    use voltforge_electrical::motor::{
        compressor_branch, full_load_current, motor_branch, CompressorNameplate,
    };
    use voltforge_electrical::sizing::MotorSizing;

    let mut failures = Vec::new();
    let rows = parse("motors.csv", &fixture("motors.csv"), |r| {
        (
            r.text("id"),
            r.text("kind"),
            r.get::<u32>("phases"),
            r.text("hp"),
            r.get::<f64>("system_v"),
            r.text("motor_type"),
            CompressorNameplate {
                rated_load_a: r.opt("rated_load_a"),
                branch_selection_a: None,
                minimum_circuit_a: r.opt("minimum_circuit_a"),
                maximum_protection_a: r.opt("maximum_protection_a"),
            },
            r.get::<u32>("current_carrying"),
            r.get::<f64>("expected_full_load_a"),
            r.get::<f64>("expected_conductor_a"),
            r.get::<f64>("expected_breaker_a"),
            r.text("expected_size"),
            r.text("expected_grounding"),
        )
    });
    assert!(!rows.is_empty());
    for (id, kind, phases, hp, volts, motor_type, plate, ccc, flc, cond, brk, size, gnd) in rows {
        let (full_load_a, design_a, breaker_a) = if kind == "motor" {
            let motor_type = match motor_type.as_str() {
                "" => None,
                "squirrel_cage" => Some(MotorType::SquirrelCage),
                "design_b_energy_efficient" => Some(MotorType::DesignBEnergyEfficient),
                "wound_rotor" => Some(MotorType::WoundRotor),
                "synchronous" => Some(MotorType::Synchronous),
                other => panic!("tipo de motor desconocido: {other}"),
            };
            let current = full_load_current(nom(), &hp, phases == 3, volts).unwrap();
            let branch = motor_branch(nom(), current.full_load_a, motor_type, false).unwrap();
            (
                current.full_load_a,
                branch.conductor_a,
                branch.breaker_max_a,
            )
        } else {
            let branch = compressor_branch(nom(), &plate, false).unwrap();
            (branch.basis_a, branch.conductor_a, branch.breaker_max_a)
        };
        compare(
            &mut failures,
            &id,
            "corriente",
            Some(flc),
            full_load_a,
            0.01,
        );
        compare(
            &mut failures,
            &id,
            "corriente de diseño",
            Some(cond),
            design_a,
            0.01,
        );
        compare(&mut failures, &id, "interruptor", Some(brk), breaker_a, 0.0);
        let input = SizingInput {
            continuous_a: 0.0,
            noncontinuous_a: full_load_a,
            material: Material::Copper,
            grounding_material: Material::Copper,
            insulation: Insulation::Thw,
            wet_location: false,
            ambient_c: 30.0,
            current_carrying_conductors: ccc,
            nipple: false,
            conductors_per_phase: 1,
            terminal_rating: None,
            include_iec_ratings: false,
            minimum_size: None,
            voltage_drop: None,
            motor: Some(MotorSizing {
                design_a,
                breaker_a,
            }),
        };
        let r = select_conductor(nom(), &input).unwrap();
        if r.conductor.size != Size::new(&size) {
            failures.push(format!(
                "{id}: calibre esperado {size}, calculado {}",
                r.conductor.size
            ));
        }
        if r.grounding.size != Size::new(&gnd) {
            failures.push(format!(
                "{id}: tierra esperada {gnd}, calculada {}",
                r.grounding.size
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
