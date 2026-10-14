//! Procedimiento paso a paso: cada cálculo con su fórmula, los valores
//! sustituidos y la tabla o artículo de donde sale.
//!
//! Lo comparten el cálculo rápido y cada circuito o alimentador del proyecto.

use ieestudio_core::units::m_to_km;
use ieestudio_core::Insulation;
use ieestudio_core::{CircuitKind, ConduitType, Material, RacewayMaterial, Size};
use ieestudio_electrical::conduit::ConduitResult;
use ieestudio_electrical::sizing::ConductorCheck;
use ieestudio_electrical::voltage_drop::{
    temperature_coefficient, VoltageDropInput, VoltageDropResult, TABLE_9_TEMPERATURE_C,
};
use ieestudio_rules::{Profile, Rule, RuleReference};
use serde::Serialize;

/// Un paso del procedimiento.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub title: String,
    /// Fórmula general, si el paso tiene una.
    pub formula: Option<String>,
    /// La fórmula con los valores sustituidos y su resultado.
    pub substitution: String,
    pub note: Option<String>,
    /// Dónde lo establece la norma. Vacío en pasos que solo son cálculo.
    pub references: Vec<RuleReference>,
}

/// Nombre de un calibre con su unidad: `3/0 AWG`, `300 kcmil`.
pub fn size_label(size: &Size) -> String {
    match size.0.parse::<u32>() {
        Ok(n) if n >= 250 => format!("{} kcmil", size.0),
        _ => format!("{} AWG", size.0),
    }
}

pub fn material_name(material: Material) -> &'static str {
    match material {
        Material::Copper => "cobre",
        Material::Aluminum => "aluminio",
    }
}

pub fn insulation_name(insulation: Insulation) -> &'static str {
    use Insulation::*;
    match insulation {
        Tw => "TW",
        Thw => "THW",
        Thhw => "THHW",
        Thw2 => "THW-2",
        Thhn => "THHN",
        Thwn => "THWN",
        Thwn2 => "THWN-2",
        Xhhw => "XHHW",
        Xhhw2 => "XHHW-2",
        Rhh => "RHH",
        Rhw => "RHW",
        Rhw2 => "RHW-2",
    }
}

fn raceway_name(raceway: RacewayMaterial) -> &'static str {
    match raceway {
        RacewayMaterial::Steel => "acero",
        RacewayMaterial::Pvc => "PVC",
        RacewayMaterial::Aluminum => "aluminio",
    }
}

/// Número con separador de miles y sin decimales sobrantes.
pub(crate) fn plain(value: f64) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    let (int, frac) = text.split_once('.').unwrap_or((text, ""));
    let mut grouped = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    if frac.is_empty() {
        grouped
    } else {
        format!("{grouped}.{frac}")
    }
}

/// Corriente nominal a partir de la potencia. Con `power_w` en `None` la
/// corriente se capturó directamente.
pub(crate) fn current_step(
    kind: CircuitKind,
    power_w: Option<f64>,
    power_factor: f64,
    voltage_v: f64,
    current_a: f64,
) -> Step {
    let Some(power_w) = power_w else {
        return Step {
            title: "Corriente del circuito".into(),
            formula: None,
            substitution: format!("I = {current_a:.2} A"),
            note: Some("Corriente capturada directamente.".into()),
            references: vec![],
        };
    };
    let p = plain(power_w);
    let v = plain(voltage_v);
    let pf = power_factor;
    let (formula, substitution) = match kind {
        CircuitKind::ThreePhase => (
            "I = P / (√3 · V · fp)",
            format!("I = {p} / (1.732 · {v} · {pf:.2}) = {current_a:.2} A"),
        ),
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => (
            "I = P / (V · fp)",
            format!("I = {p} / ({v} · {pf:.2}) = {current_a:.2} A"),
        ),
    };
    let voltage = match kind {
        CircuitKind::LineToNeutral => "V es la tensión de fase a neutro.",
        CircuitKind::LineToLine | CircuitKind::ThreePhase => "V es la tensión entre fases.",
    };
    Step {
        title: "Corriente nominal".into(),
        formula: Some(formula.into()),
        substitution,
        note: Some(voltage.into()),
        references: vec![],
    }
}

/// Datos de la selección de interruptor y conductor de un tramo.
pub(crate) struct ConductorData<'a> {
    pub profile: &'a Profile,
    pub continuous_a: f64,
    pub noncontinuous_a: f64,
    pub design_a: f64,
    pub breaker_a: f64,
    /// El usuario fijó el interruptor.
    pub breaker_forced: bool,
    /// La corriente de diseño ya tiene su paso (motores, alimentadores con
    /// motores): no se repite.
    pub given_design: bool,
    /// Circuito de motor: el interruptor ya tiene su paso y el conductor se
    /// protege según el artículo 430.
    pub motor_protection: bool,
    pub poles: u32,
    pub material: Material,
    pub insulation: Insulation,
    pub ambient_c: f64,
    pub current_carrying_conductors: u32,
    pub check: &'a ConductorCheck,
    /// Menor calibre que cumple ampacidad y protección, si se conoce.
    pub minimum_size: Option<&'a Size>,
    pub grounding: Option<(&'a Size, Material)>,
}

/// Corriente de diseño, interruptor, ampacidad, verificación y puesta a tierra.
pub(crate) fn conductor_steps(d: &ConductorData) -> Vec<Step> {
    let c = d.check;
    let r = |rules: &[Rule]| {
        rules
            .iter()
            .map(|&rule| d.profile.reference(rule))
            .collect::<Vec<_>>()
    };
    let load_a = d.continuous_a + d.noncontinuous_a;
    let mut steps = Vec::new();

    let formula = Some("Id = 1.25 · I continua + I no continua".to_string());
    if !d.given_design {
        steps.push(if d.continuous_a <= 0.0 {
            Step {
                title: "Corriente de diseño".into(),
                formula,
                substitution: format!("Id = {:.2} A", d.design_a),
                note: Some("Carga no continua: no lleva el 125 %.".into()),
                references: r(&[Rule::DesignCurrent]),
            }
        } else {
            Step {
                title: "Corriente de diseño".into(),
                formula,
                substitution: format!(
                    "Id = {} · {:.2} + {:.2} = {:.2} A",
                    d.profile.parameters.continuous_load_factor,
                    d.continuous_a,
                    d.noncontinuous_a,
                    d.design_a
                ),
                note: Some(
                    "Es continua la carga que opera a su corriente máxima tres horas o más.".into(),
                ),
                references: r(&[Rule::DesignCurrent]),
            }
        });
    }

    if !d.motor_protection {
        steps.push(Step {
            title: "Interruptor".into(),
            formula: None,
            substitution: format!("{}P-{:.0} A", d.poles, d.breaker_a),
            note: Some(if d.breaker_forced {
                format!(
                    "Fijado por ti. No debe ser menor que la corriente de diseño de {:.2} A y debe proteger al conductor.",
                    d.design_a
                )
            } else {
                format!(
                    "Capacidad normalizada igual o inmediata superior a {:.2} A.",
                    d.design_a
                )
            }),
            references: r(&if d.breaker_forced {
                vec![Rule::StandardRatings, Rule::BreakerRating]
            } else {
                vec![Rule::StandardRatings]
            }),
        });
    }

    steps.push(Step {
        title: "Ampacidad del conductor".into(),
        formula: Some("Ampacidad corregida = tabla · f. temperatura · f. agrupamiento · n".into()),
        substitution: format!(
            "{} · {} · {} · {} = {:.2} A",
            plain(c.table_ampacity_a),
            plain(c.temperature_factor),
            plain(c.bundling_factor),
            c.conductors_per_phase,
            c.corrected_ampacity_a
        ),
        note: Some(format!(
            "Ampacidad de tabla de {} de {} {}, columna de {:.0} °C. Ambiente de {} °C y {} conductores portadores de corriente. En terminales de {:.0} °C: {} A. Ampacidad permitida, la menor: {:.2} A.",
            size_label(&c.size),
            material_name(d.material),
            insulation_name(d.insulation),
            c.insulation_rating.celsius(),
            plain(d.ambient_c),
            d.current_carrying_conductors,
            c.terminal_rating.celsius(),
            plain(c.terminal_ampacity_a),
            c.allowable_ampacity_a
        )),
        references: r(&[
            Rule::Ampacity,
            Rule::TemperatureCorrection,
            Rule::Bundling,
            Rule::Neutral,
            Rule::Terminals,
        ]),
    });

    let passes = c.meets_ampacity() && c.protected;
    let ampacity = if c.meets_design_current && c.meets_load {
        format!(
            "La ampacidad cubre la carga de {load_a:.2} A y la corriente de diseño de {:.2} A.",
            d.design_a
        )
    } else {
        format!(
            "La ampacidad no cubre la carga de {load_a:.2} A o la corriente de diseño de {:.2} A.",
            d.design_a
        )
    };
    let protected = if d.motor_protection {
        format!(
            "El relevador de sobrecarga protege al conductor; el interruptor de {:.0} A, contra cortocircuito.",
            d.breaker_a
        )
    } else if c.protected {
        format!(
            "El interruptor de {:.0} A protege al conductor.",
            d.breaker_a
        )
    } else {
        format!(
            "El interruptor de {:.0} A no protege al conductor.",
            d.breaker_a
        )
    };
    let parallel = if c.parallel_allowed {
        ""
    } else {
        " Este calibre no se permite en paralelo: solo desde 1/0 AWG."
    };
    let minimum = match d.minimum_size {
        Some(size) if !passes => format!(" Calibre mínimo que cumple: {}.", size_label(size)),
        _ => String::new(),
    };
    steps.push(Step {
        title: "Verificación del conductor".into(),
        formula: None,
        substitution: format!(
            "{} con {:.2} A permitidos: {}",
            size_label(&c.size),
            c.allowable_ampacity_a,
            if passes { "cumple" } else { "no cumple" }
        ),
        note: Some(format!("{ampacity} {protected}{parallel}{minimum}")),
        references: r(&if d.motor_protection {
            vec![
                Rule::MotorBranchConductor,
                Rule::MotorConductorProtection,
                Rule::Parallel,
            ]
        } else {
            vec![
                Rule::DesignCurrent,
                Rule::ConductorProtection,
                Rule::SmallConductors,
                Rule::Parallel,
            ]
        }),
    });

    if let Some((ground, material)) = d.grounding {
        steps.push(Step {
            title: "Puesta a tierra de equipos".into(),
            formula: None,
            substitution: format!("{} de {}", size_label(ground), material_name(material)),
            note: Some(format!(
                "Por la capacidad del interruptor de {:.0} A, con incremento proporcional si el conductor de fase se sobredimensiona.",
                d.breaker_a
            )),
            references: r(&[Rule::EquipmentGrounding, Rule::GroundingIncrease]),
        });
    }
    steps
}

/// Selección de la tubería conduit.
pub(crate) fn conduit_step(
    profile: &Profile,
    result: &ConduitResult,
    conduit: ConduitType,
    nipple: bool,
) -> Step {
    let groups: Vec<String> = result
        .groups
        .iter()
        .map(|g| {
            format!(
                "{} × {} de {:.2} mm² = {:.2} mm²",
                g.count,
                size_label(&g.size),
                g.unit_area_mm2,
                g.total_area_mm2
            )
        })
        .collect();
    let mut note =
        format!(
        "{}. Con {} conductores la ocupación permitida es de {:.0} %{}; el tubo admite {:.2} mm².",
        groups.join("; "),
        result.conductor_count,
        result.allowed_fill_fraction * 100.0,
        if nipple { " por ser niple de 60 cm o menos" } else { "" },
        result.allowed_area_mm2
    );
    if result.upsized_for_jamming {
        note.push_str(" Se subió un tamaño para evitar el atascamiento de tres conductores.");
    }
    Step {
        title: "Tubería conduit".into(),
        formula: Some("Área de los conductores ≤ área permitida del tubo".into()),
        substitution: format!(
            "{conduit} de {} mm ({}\"): {:.2} mm² de conductores, ocupación de {:.1} %",
            result.conduit.metric_designator,
            result.conduit.trade_size,
            result.conductor_area_mm2,
            result.fill_percent
        ),
        note: Some(note),
        references: [
            Rule::InsulatedDimensions,
            Rule::ConductorProperties,
            Rule::ConduitFill,
            Rule::ConduitDimensions,
        ]
        .into_iter()
        .chain(nipple.then_some(Rule::Nipple))
        .chain(result.upsized_for_jamming.then_some(Rule::Jamming))
        .map(|rule| profile.reference(rule))
        .collect(),
    }
}

/// Caída de tensión: R y X, ajuste opcional, caída, porcentaje y comprobación.
pub(crate) fn drop_steps(
    profile: &Profile,
    input: &VoltageDropInput,
    drop: &VoltageDropResult,
    limit_percent: f64,
    limit_rule: Option<Rule>,
) -> Vec<Step> {
    let v = drop.reference_voltage_v;
    let pf = input.power_factor;
    let sin = (1.0 - pf * pf).sqrt();
    let (k_symbol, k_value) = match input.kind {
        CircuitKind::ThreePhase => ("√3", "1.732"),
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => ("2", "2"),
    };
    let mut steps = Vec::new();

    let r_table = drop.table_resistance_ohm_km;
    let x = drop.reactance_ohm_km;
    steps.push(Step {
        title: "Resistencia y reactancia del conductor".into(),
        formula: None,
        substitution: format!("R = {} Ω/km · X = {} Ω/km", plain(r_table), plain(x)),
        note: Some(format!(
            "{} de {} en conduit de {}, a 75 °C.",
            size_label(&input.size),
            material_name(input.material),
            raceway_name(input.raceway)
        )),
        references: vec![profile.reference(Rule::Impedance)],
    });

    let r = drop.resistance_ohm_km;
    let adjusted = input.conductor_temperature_c.is_some();
    if let Some(t) = input.conductor_temperature_c {
        let alpha = temperature_coefficient(input.material);
        steps.push(Step {
            title: "Ajuste de la resistencia por temperatura".into(),
            formula: Some("R2 = R1 · [1 + α · (T2 − 75)]".into()),
            substitution: format!(
                "R2 = {} · [1 + {alpha} · ({} − {})] = {r:.4} Ω/km",
                plain(r_table),
                plain(t),
                plain(TABLE_9_TEMPERATURE_C)
            ),
            note: Some(format!(
                "α = {alpha} para {}.",
                material_name(input.material)
            )),
            references: vec![profile.reference(Rule::ResistanceTemperature)],
        });
    }

    let r_text = if adjusted {
        format!("{r:.4}")
    } else {
        plain(r)
    };
    steps.push(Step {
        title: "Caída de tensión".into(),
        formula: Some(format!(
            "ΔV = {k_symbol} · I · L · (R · cos θ + X · sen θ) / n"
        )),
        substitution: format!(
            "ΔV = {k_value} · {:.2} · {:.4} · ({r_text} · {pf:.2} + {} · {sin:.3}) / {} = {:.2} V",
            input.current_a,
            m_to_km(input.length_m),
            plain(x),
            input.conductors_per_phase,
            drop.drop_v
        ),
        note: Some("L en kilómetros; n = conductores por fase; cos θ = fp.".into()),
        references: vec![profile.reference(Rule::Impedance)],
    });

    let relation = if drop.drop_percent <= limit_percent {
        "no excede"
    } else {
        "es mayor que"
    };
    steps.push(Step {
        title: "Porcentaje y comparación".into(),
        formula: Some("e = ΔV / V · 100".into()),
        substitution: format!(
            "e = {:.2} / {} · 100 = {:.2} %",
            drop.drop_v,
            plain(v),
            drop.drop_percent
        ),
        note: Some(format!(
            "{:.2} % {relation} el límite de {limit_percent:.2} %.",
            drop.drop_percent
        )),
        references: limit_rule
            .map(|rule| profile.reference(rule))
            .into_iter()
            .collect(),
    });

    steps.push(Step {
        title: "Comprobación con la fórmula exacta".into(),
        formula: None,
        substitution: format!(
            "ΔV exacta = {:.2} V · {:.2} %",
            drop.exact_drop_v, drop.exact_drop_percent
        ),
        note: Some(format!(
            "Resuelta con fasores; difiere {:.2} V de la fórmula aproximada.",
            (drop.exact_drop_v - drop.drop_v).abs()
        )),
        references: vec![],
    });
    steps
}

/// Caída acumulada desde la fuente hasta el final del tramo.
pub(crate) fn cumulative_step(
    profile: &Profile,
    upstream_percent: f64,
    own_percent: f64,
    limit_percent: f64,
) -> Step {
    let total = upstream_percent + own_percent;
    let relation = if total <= limit_percent {
        "no excede"
    } else {
        "es mayor que"
    };
    Step {
        title: "Caída acumulada desde la fuente".into(),
        formula: Some("e total = e de los alimentadores + e del tramo".into()),
        substitution: format!("e total = {upstream_percent:.2} + {own_percent:.2} = {total:.2} %"),
        note: Some(format!(
            "{total:.2} % {relation} el límite de {limit_percent:.2} %."
        )),
        references: vec![
            profile.reference(Rule::BranchDrop),
            profile.reference(Rule::FeederDrop),
        ],
    }
}
