//! Muestra el procedimiento de dos cálculos de referencia.
//!
//! Ejecutar con: `cargo run --example procedimiento`

use voltforge_core::{
    CircuitKind, ConduitType, InsulationGroup, Material, RacewayMaterial, Size, System,
};
use voltforge_electrical::conduit::{size_conduit, ConductorGroup, ConduitInput, Covering};
use voltforge_electrical::current::nominal_current;
use voltforge_electrical::voltage_drop::{voltage_drop, VoltageDropInput};
use voltforge_rules::Profile;

fn main() -> Result<(), voltforge_core::Error> {
    let profile = Profile::nom_001_sede_2012();
    println!("Perfil: {}\n", profile.title);

    // Alimentador trifásico: 61.7 kW, fp 0.90, 220 V, 95 m, 3/0 AWG de cobre en acero.
    let system = System::WYE_220_127;
    let kind = CircuitKind::ThreePhase;
    let (power_w, pf, length_m) = (61_700.0, 0.90, 95.0);
    let current_a = nominal_current(power_w, pf, kind, &system)?;
    let input = VoltageDropInput {
        kind,
        system,
        current_a,
        length_m,
        power_factor: pf,
        size: Size::new("3/0"),
        material: Material::Copper,
        raceway: RacewayMaterial::Steel,
        conductors_per_phase: 1,
        conductor_temperature_c: None,
    };
    let r = voltage_drop(profile, &input)?;
    let sin = (1.0 - pf * pf).sqrt();
    println!("CAÍDA DE TENSIÓN");
    println!("1. I = P / (√3 · V · fp) = {power_w} / (1.732 · 220 · {pf}) = {current_a:.2} A");
    println!(
        "2. Tabla 9, 3/0 AWG cobre en conduit de acero: R = {} Ω/km, X = {} Ω/km",
        r.resistance_ohm_km, r.reactance_ohm_km
    );
    println!(
        "3. ΔV = √3 · {current_a:.2} · {:.3} · ({} · {pf} + {} · {sin:.3}) / 1 = {:.2} V",
        length_m / 1000.0,
        r.resistance_ohm_km,
        r.reactance_ohm_km,
        r.drop_v
    );
    println!(
        "4. e = {:.2} / {} · 100 = {:.2} %",
        r.drop_v, r.reference_voltage_v, r.drop_percent
    );
    println!(
        "5. Comprobación con la fórmula exacta: {:.2} V\n",
        r.exact_drop_v
    );

    // Tubería: 3 × 12 AWG THW, 1 × 250 kcmil THW y tierra desnuda 6 AWG en EMT.
    let thw = Covering::Insulated(InsulationGroup::TwThwThhw);
    let conduit = size_conduit(
        profile,
        &ConduitInput {
            groups: vec![
                ConductorGroup {
                    count: 3,
                    size: Size::new("12"),
                    covering: thw,
                },
                ConductorGroup {
                    count: 1,
                    size: Size::new("250"),
                    covering: thw,
                },
                ConductorGroup {
                    count: 1,
                    size: Size::new("6"),
                    covering: Covering::Bare,
                },
            ],
            conduit: ConduitType::Emt,
            nipple: false,
        },
    )?;
    println!("TUBERÍA CONDUIT");
    for g in &conduit.groups {
        println!(
            "   {} × {:>4}: {:>7.2} mm² c/u = {:>7.2} mm²",
            g.count, g.size.0, g.unit_area_mm2, g.total_area_mm2
        );
    }
    println!(
        "   Área total: {:.2} mm², {} conductores, ocupación permitida {:.0} %",
        conduit.conductor_area_mm2,
        conduit.conductor_count,
        conduit.allowed_fill_fraction * 100.0
    );
    println!(
        "   Tubo EMT de {} mm ({}\"): admite {} mm², ocupación real {:.1} %",
        conduit.conduit.metric_designator,
        conduit.conduit.trade_size,
        conduit.allowed_area_mm2,
        conduit.fill_percent
    );
    Ok(())
}
