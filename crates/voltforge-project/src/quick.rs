//! Cálculo rápido de un solo circuito, sin proyecto: corriente, protección,
//! verificación del conductor y caída de tensión.
//!
//! Devuelve el resultado, el procedimiento paso a paso con los valores
//! sustituidos y, cuando no cumple, alternativas ya calculadas.

use serde::{Deserialize, Serialize};
use voltforge_core::current::{design_current, nominal_current, CONTINUOUS_LOAD_FACTOR};
use voltforge_core::sizing::{
    check_conductor, equipment_grounding, select_conductor, standard_rating_at_least, Insulation,
    SizingInput, VoltageDropLimit, SMALLEST_PARALLEL_SIZE,
};
use voltforge_core::voltage_drop::{
    max_length_m, temperature_coefficient, voltage_drop, VoltageDropInput, VoltageDropResult,
    TABLE_9_TEMPERATURE_C,
};
use voltforge_core::{CircuitKind, ConduitType, Material, Profile, RacewayMaterial, Size, System};

use crate::calc::{is_wye, poles};
use crate::Status;

/// Mayor número de conductores por fase que se propone en una recomendación.
const MAX_SUGGESTED_PARALLEL: u32 = 6;

/// Datos del cálculo rápido.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuickInput {
    pub system: System,
    pub kind: CircuitKind,
    /// `true` si la carga se captura como corriente; `false`, como potencia.
    pub load_in_amperes: bool,
    pub power_w: f64,
    pub current_a: f64,
    pub power_factor: f64,
    /// Carga continua: opera a su corriente máxima tres horas o más.
    pub continuous: bool,
    /// Longitud de un sentido, en metros.
    pub length_m: f64,
    pub size: Size,
    pub conductors_per_phase: u32,
    pub material: Material,
    pub insulation: Insulation,
    /// Lleva neutro además de las fases. Un circuito de una fase siempre lo lleva.
    pub has_neutral: bool,
    pub conduit: ConduitType,
    /// Temperatura ambiente, en °C.
    pub ambient_c: f64,
    pub limit_percent: f64,
    /// Temperatura del conductor para ajustar la resistencia; `None` usa 75 °C.
    pub resistance_temperature_c: Option<f64>,
}

impl Default for QuickInput {
    /// Alimentador trifásico de referencia: 61.7 kW a 95 m con 3/0 AWG.
    fn default() -> Self {
        QuickInput {
            system: System::WYE_220_127,
            kind: CircuitKind::ThreePhase,
            load_in_amperes: false,
            power_w: 61_700.0,
            current_a: 100.0,
            power_factor: 0.9,
            continuous: false,
            length_m: 95.0,
            size: Size::new("3/0"),
            conductors_per_phase: 1,
            material: Material::Copper,
            insulation: Insulation::Thw,
            has_neutral: false,
            conduit: ConduitType::Emt,
            ambient_c: 30.0,
            limit_percent: 3.0,
            resistance_temperature_c: None,
        }
    }
}

/// Un paso del procedimiento.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub title: String,
    /// Fórmula general, si el paso tiene una.
    pub formula: Option<String>,
    /// La fórmula con los valores sustituidos y su resultado.
    pub substitution: String,
    pub note: Option<String>,
}

/// Alternativa ya calculada. Nunca se aplica sola.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Recommendation {
    pub title: String,
    /// Resultado de la alternativa, por ejemplo `2.77 %`.
    pub value: String,
    pub detail: String,
    /// Calibre y conductores por fase que la alternativa propone, si cambia alguno.
    pub apply: Option<(Size, u32)>,
}

/// Corriente de diseño, interruptor y verificación del conductor elegido.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Protection {
    pub design_a: f64,
    pub breaker_a: f64,
    pub poles: u32,
    /// Conductores portadores de corriente considerados para el agrupamiento.
    pub current_carrying_conductors: u32,
    /// Ampacidad de tabla en la columna del aislamiento, por conductor.
    pub table_ampacity_a: f64,
    pub insulation_c: f64,
    pub temperature_factor: f64,
    pub bundling_factor: f64,
    pub corrected_ampacity_a: f64,
    pub terminal_c: f64,
    pub terminal_ampacity_a: f64,
    pub allowable_ampacity_a: f64,
    /// La ampacidad cubre la carga y la corriente de diseño.
    pub meets_ampacity: bool,
    /// El interruptor protege al conductor (240-4).
    pub protected: bool,
    /// Menor calibre que cumple ampacidad y protección con estos conductores por fase.
    pub minimum_size: Option<Size>,
    /// Conductor de puesta a tierra de equipos (250-122).
    pub grounding_size: Option<Size>,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickResult {
    /// Corriente nominal de la carga.
    pub current_a: f64,
    pub drop: VoltageDropResult,
    /// Estado de la caída de tensión.
    pub status: Status,
    /// Protección y conductor, o el motivo por el que no se pudo evaluar.
    pub protection: Result<Protection, String>,
    /// Longitud máxima con la que la selección actual cumple el límite.
    pub max_length_m: f64,
    pub steps: Vec<Step>,
    pub recommendations: Vec<Recommendation>,
    pub warnings: Vec<String>,
}

/// Nombre de un calibre con su unidad: `3/0 AWG`, `300 kcmil`.
pub fn size_label(size: &Size) -> String {
    match size.0.parse::<u32>() {
        Ok(n) if n >= 250 => format!("{} kcmil", size.0),
        _ => format!("{} AWG", size.0),
    }
}

fn material_name(material: Material) -> &'static str {
    match material {
        Material::Copper => "cobre",
        Material::Aluminum => "aluminio",
    }
}

fn insulation_name(insulation: Insulation) -> &'static str {
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

/// Número con separador de miles y sin decimales sobrantes, para el procedimiento.
fn plain(value: f64) -> String {
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

fn factor_text(kind: CircuitKind) -> (&'static str, &'static str) {
    match kind {
        CircuitKind::ThreePhase => ("√3", "1.732"),
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => ("2", "2"),
    }
}

fn drop_input(input: &QuickInput, current_a: f64) -> VoltageDropInput {
    VoltageDropInput {
        kind: input.kind,
        system: input.system,
        current_a,
        length_m: input.length_m,
        power_factor: input.power_factor,
        size: input.size.clone(),
        material: input.material,
        raceway: input.conduit.raceway_material(),
        conductors_per_phase: input.conductors_per_phase,
        conductor_temperature_c: input.resistance_temperature_c,
    }
}

/// Conductores portadores de corriente con `n` conductores por fase. El neutro
/// cuenta en un circuito de una fase y en uno de dos fases de un sistema en
/// estrella; en uno de tres fases con carga lineal no cuenta.
fn current_carrying(input: &QuickInput, n: u32) -> u32 {
    let neutral_counts = match input.kind {
        CircuitKind::LineToNeutral => true,
        CircuitKind::LineToLine => input.has_neutral && is_wye(&input.system),
        CircuitKind::ThreePhase => false,
    };
    poles(input.kind) * n + if neutral_counts { n } else { 0 }
}

/// Datos para evaluar ampacidad y protección con `n` conductores por fase.
/// Con `drop` también se exige el límite de caída de tensión.
fn sizing_input(
    input: &QuickInput,
    current_a: f64,
    n: u32,
    drop: Option<&VoltageDropInput>,
) -> SizingInput {
    let (continuous_a, noncontinuous_a) = if input.continuous {
        (current_a, 0.0)
    } else {
        (0.0, current_a)
    };
    SizingInput {
        continuous_a,
        noncontinuous_a,
        material: input.material,
        grounding_material: input.material,
        insulation: input.insulation,
        wet_location: false,
        ambient_c: input.ambient_c,
        current_carrying_conductors: current_carrying(input, n),
        nipple: false,
        conductors_per_phase: n,
        terminal_rating: None,
        include_iec_ratings: false,
        minimum_size: None,
        voltage_drop: drop.map(|d| VoltageDropLimit {
            input: d.clone(),
            limit_percent: input.limit_percent,
        }),
    }
}

fn protection(profile: &Profile, input: &QuickInput, current_a: f64) -> Result<Protection, String> {
    let n = input.conductors_per_phase;
    let sizing = sizing_input(input, current_a, n, None);
    let design_a = design_current(sizing.continuous_a, sizing.noncontinuous_a);
    let breaker_a =
        standard_rating_at_least(profile, design_a, false).map_err(|e| e.to_string())?;
    let check =
        check_conductor(profile, &sizing, &input.size, breaker_a).map_err(|e| e.to_string())?;
    let minimum_size = select_conductor(profile, &sizing)
        .ok()
        .map(|r| r.conductor.size);
    let grounding_size = equipment_grounding(
        profile,
        breaker_a,
        input.material,
        true,
        minimum_size.as_ref().unwrap_or(&input.size),
        &input.size,
    )
    .ok()
    .map(|g| g.size);
    let meets_ampacity = check.meets_ampacity();
    Ok(Protection {
        design_a,
        breaker_a,
        poles: poles(input.kind),
        current_carrying_conductors: sizing.current_carrying_conductors,
        table_ampacity_a: check.table_ampacity_a,
        insulation_c: check.insulation_rating.celsius(),
        temperature_factor: check.temperature_factor,
        bundling_factor: check.bundling_factor,
        corrected_ampacity_a: check.corrected_ampacity_a,
        terminal_c: check.terminal_rating.celsius(),
        terminal_ampacity_a: check.terminal_ampacity_a,
        allowable_ampacity_a: check.allowable_ampacity_a,
        meets_ampacity,
        protected: check.protected,
        minimum_size,
        grounding_size,
        status: if meets_ampacity && check.protected {
            Status::Complies
        } else {
            Status::Fails
        },
    })
}

/// Calcula un circuito suelto: corriente, protección, conductor y caída de tensión.
///
/// Devuelve un error con explicación cuando las tablas no cubren el caso o un
/// dato no es válido; ese caso se muestra como «no evaluado».
pub fn quick_circuit(profile: &Profile, input: &QuickInput) -> Result<QuickResult, String> {
    if !(input.limit_percent.is_finite() && input.limit_percent > 0.0) {
        return Err("el límite de caída debe ser mayor que cero".into());
    }
    let current_a = if input.load_in_amperes {
        input.current_a
    } else {
        nominal_current(input.power_w, input.power_factor, input.kind, &input.system)
            .map_err(|e| e.to_string())?
    };
    let base = drop_input(input, current_a);
    let drop = voltage_drop(profile, &base).map_err(|e| e.to_string())?;
    let max_length =
        max_length_m(profile, &base, input.limit_percent).map_err(|e| e.to_string())?;
    let drop_complies = drop.drop_percent <= input.limit_percent;
    let n = input.conductors_per_phase;
    let rank = profile.size_rank(&input.size).map_err(|e| e.to_string())?;
    let parallel_rank = profile
        .size_rank(&Size::new(SMALLEST_PARALLEL_SIZE))
        .map_err(|e| e.to_string())?;

    let mut warnings = Vec::new();
    if n > 1 && rank < parallel_rank {
        warnings.push(format!(
            "La norma permite conductores en paralelo a partir de {}; el {} no se puede instalar en paralelo.",
            size_label(&Size::new(SMALLEST_PARALLEL_SIZE)),
            size_label(&input.size)
        ));
    }

    let protection = protection(profile, input, current_a);
    let conductor_fails = matches!(&protection, Ok(p) if p.status == Status::Fails);

    let mut steps = vec![current_step(input, current_a, drop.reference_voltage_v)];
    match &protection {
        Ok(p) => steps.extend(protection_steps(profile, input, current_a, p)),
        Err(e) => steps.push(Step {
            title: "Protección y conductor".into(),
            formula: None,
            substitution: "No evaluado".into(),
            note: Some(format!("No se pudo calcular: {e}.")),
        }),
    }
    steps.extend(drop_steps(profile, input, current_a, &drop));

    Ok(QuickResult {
        current_a,
        steps,
        recommendations: recommendations(
            profile,
            input,
            &base,
            &drop,
            conductor_fails,
            max_length,
            rank,
            parallel_rank,
        ),
        status: if drop_complies {
            Status::Complies
        } else {
            Status::Fails
        },
        protection,
        max_length_m: max_length,
        drop,
        warnings,
    })
}

fn current_step(input: &QuickInput, current_a: f64, v: f64) -> Step {
    if input.load_in_amperes {
        return Step {
            title: "Corriente del circuito".into(),
            formula: None,
            substitution: format!("I = {current_a:.2} A"),
            note: Some("Corriente capturada directamente.".into()),
        };
    }
    let p = plain(input.power_w);
    let pf = input.power_factor;
    let (formula, substitution) = match input.kind {
        CircuitKind::ThreePhase => (
            "I = P / (√3 · V · fp)",
            format!(
                "I = {p} / (1.732 · {} · {pf:.2}) = {current_a:.2} A",
                plain(v)
            ),
        ),
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => (
            "I = P / (V · fp)",
            format!("I = {p} / ({} · {pf:.2}) = {current_a:.2} A", plain(v)),
        ),
    };
    let voltage = match input.kind {
        CircuitKind::LineToNeutral => "V es la tensión de fase a neutro.",
        CircuitKind::LineToLine | CircuitKind::ThreePhase => "V es la tensión entre fases.",
    };
    Step {
        title: "Corriente nominal".into(),
        formula: Some(formula.into()),
        substitution,
        note: Some(voltage.into()),
    }
}

fn protection_steps(
    profile: &Profile,
    input: &QuickInput,
    current_a: f64,
    p: &Protection,
) -> Vec<Step> {
    let norm = profile.id.to_uppercase();
    let n = input.conductors_per_phase;
    let mut steps = Vec::new();

    steps.push(if input.continuous {
        Step {
            title: "Corriente de diseño".into(),
            formula: Some("Id = 1.25 · I continua + I no continua".into()),
            substitution: format!(
                "Id = {CONTINUOUS_LOAD_FACTOR} · {current_a:.2} = {:.2} A",
                p.design_a
            ),
            note: Some("Carga continua: opera a su corriente máxima tres horas o más.".into()),
        }
    } else {
        Step {
            title: "Corriente de diseño".into(),
            formula: Some("Id = 1.25 · I continua + I no continua".into()),
            substitution: format!("Id = {:.2} A", p.design_a),
            note: Some("Carga no continua: no lleva el 125 %.".into()),
        }
    });

    steps.push(Step {
        title: "Interruptor".into(),
        formula: None,
        substitution: format!("{}P-{:.0} A", p.poles, p.breaker_a),
        note: Some(format!(
            "Capacidad normalizada igual o inmediata superior a {:.2} A (sección 240-6, {norm}).",
            p.design_a
        )),
    });

    steps.push(Step {
        title: "Ampacidad del conductor".into(),
        formula: Some("Ampacidad corregida = tabla · f. temperatura · f. agrupamiento · n".into()),
        substitution: format!(
            "{} · {} · {} · {n} = {:.2} A",
            plain(p.table_ampacity_a),
            plain(p.temperature_factor),
            plain(p.bundling_factor),
            p.corrected_ampacity_a
        ),
        note: Some(format!(
            "Tabla 310-15(b)(16): {} de {} {}, columna de {:.0} °C. Ambiente de {} °C y {} conductores portadores de corriente. En terminales de {:.0} °C: {} A. Ampacidad permitida, la menor: {:.2} A.",
            size_label(&input.size),
            material_name(input.material),
            insulation_name(input.insulation),
            p.insulation_c,
            plain(input.ambient_c),
            p.current_carrying_conductors,
            p.terminal_c,
            plain(p.terminal_ampacity_a),
            p.allowable_ampacity_a
        )),
    });

    let ampacity = if p.meets_ampacity {
        format!(
            "La ampacidad cubre la carga de {current_a:.2} A y la corriente de diseño de {:.2} A.",
            p.design_a
        )
    } else {
        format!(
            "La ampacidad no cubre la carga de {current_a:.2} A o la corriente de diseño de {:.2} A.",
            p.design_a
        )
    };
    let protected = if p.protected {
        format!(
            "El interruptor de {:.0} A protege al conductor (240-4).",
            p.breaker_a
        )
    } else {
        format!(
            "El interruptor de {:.0} A no protege al conductor (240-4).",
            p.breaker_a
        )
    };
    let minimum = match &p.minimum_size {
        Some(size) if p.status == Status::Fails => {
            format!(" Calibre mínimo que cumple: {}.", size_label(size))
        }
        _ => String::new(),
    };
    steps.push(Step {
        title: "Verificación del conductor".into(),
        formula: None,
        substitution: format!(
            "{} con {:.2} A permitidos: {}",
            size_label(&input.size),
            p.allowable_ampacity_a,
            if p.status == Status::Complies {
                "cumple"
            } else {
                "no cumple"
            }
        ),
        note: Some(format!("{ampacity} {protected}{minimum}")),
    });

    if let Some(ground) = &p.grounding_size {
        steps.push(Step {
            title: "Puesta a tierra de equipos".into(),
            formula: None,
            substitution: format!("{} de {}", size_label(ground), material_name(input.material)),
            note: Some(format!(
                "Tabla 250-122 para un interruptor de {:.0} A, con el incremento de 250-122(b) si el conductor de fase se sobredimensiona.",
                p.breaker_a
            )),
        });
    }
    steps
}

fn drop_steps(
    profile: &Profile,
    input: &QuickInput,
    current_a: f64,
    drop: &VoltageDropResult,
) -> Vec<Step> {
    let v = drop.reference_voltage_v;
    let pf = input.power_factor;
    let sin = (1.0 - pf * pf).sqrt();
    let (k_symbol, k_value) = factor_text(input.kind);
    let norm = profile.id.to_uppercase();
    let mut steps = Vec::new();

    let r_table = drop.table_resistance_ohm_km;
    let x = drop.reactance_ohm_km;
    steps.push(Step {
        title: "Resistencia y reactancia del conductor".into(),
        formula: None,
        substitution: format!("R = {} Ω/km · X = {} Ω/km", plain(r_table), plain(x)),
        note: Some(format!(
            "Tabla 9, {norm}: {} de {} en conduit de {}, 75 °C.",
            size_label(&input.size),
            material_name(input.material),
            raceway_name(input.conduit.raceway_material())
        )),
    });

    let r = drop.resistance_ohm_km;
    if let Some(t) = input.resistance_temperature_c {
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
            note: Some(format!("Tabla 8, nota 2, {norm}.")),
        });
    }

    let n = input.conductors_per_phase;
    let r_text = if input.resistance_temperature_c.is_some() {
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
            "ΔV = {k_value} · {current_a:.2} · {:.4} · ({r_text} · {pf:.2} + {} · {sin:.3}) / {n} = {:.2} V",
            input.length_m / 1000.0,
            plain(x),
            drop.drop_v
        ),
        note: Some("L en kilómetros; n = conductores por fase; cos θ = fp.".into()),
    });

    let relation = if drop.drop_percent <= input.limit_percent {
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
            "{:.2} % {relation} el límite de {:.2} %.",
            drop.drop_percent, input.limit_percent
        )),
    });

    steps.push(Step {
        title: "Comprobación con la fórmula exacta".into(),
        formula: None,
        substitution: format!(
            "ΔV exacta = {:.2} V · {:.2} %",
            drop.exact_drop_v, drop.exact_drop_percent
        ),
        note: Some(format!(
            "Resuelta con fasores; difiere {:.2} V de la fórmula de la Tabla 9.",
            (drop.exact_drop_v - drop.drop_v).abs()
        )),
    });
    steps
}

#[allow(clippy::too_many_arguments)]
fn recommendations(
    profile: &Profile,
    input: &QuickInput,
    base: &VoltageDropInput,
    drop: &VoltageDropResult,
    conductor_fails: bool,
    max_length: f64,
    rank: usize,
    parallel_rank: usize,
) -> Vec<Recommendation> {
    let limit = input.limit_percent;
    let n = input.conductors_per_phase;
    let drop_fails = drop.drop_percent > limit;
    let mut list = Vec::new();

    // Evalúa una alternativa contra el interruptor del circuito: devuelve su
    // caída de tensión y si cumple ampacidad, protección y caída.
    let try_option = |size: &Size, count: u32| -> Option<(f64, bool)> {
        let sizing = sizing_input(input, base.current_a, count, Some(base));
        let design_a = design_current(sizing.continuous_a, sizing.noncontinuous_a);
        let breaker_a = standard_rating_at_least(profile, design_a, false).ok()?;
        let check = check_conductor(profile, &sizing, size, breaker_a).ok()?;
        Some((check.voltage_drop_percent?, check.passes()))
    };

    if drop_fails || conductor_fails {
        // Calibre mayor con los mismos conductores por fase.
        let mut skipped = Vec::new();
        for c in profile.sizes.iter().skip(rank + 1) {
            let size = Size::new(&c.size);
            let Some((percent, passes)) = try_option(&size, n) else {
                continue;
            };
            if passes {
                let per_phase = if n == 1 {
                    "un conductor por fase".to_string()
                } else {
                    format!("{n} conductores por fase")
                };
                let mut detail = format!(
                    "Calibre mínimo que cumple ampacidad, protección y caída con {per_phase}."
                );
                if !skipped.is_empty() {
                    detail.push_str(&format!(
                        " Los intermedios no alcanzan: {}.",
                        skipped.join(", ")
                    ));
                }
                list.push(Recommendation {
                    title: format!("Subir a {}", size_label(&size)),
                    value: format!("{percent:.2} %"),
                    detail,
                    apply: Some((size, n)),
                });
                break;
            }
            skipped.push(if percent > limit {
                format!("{} queda en {percent:.2} %", size_label(&size))
            } else {
                format!("{} no cubre la corriente", size_label(&size))
            });
        }

        // Más conductores por fase, con el menor calibre que cumple.
        'parallel: for count in (n + 1).max(2)..=MAX_SUGGESTED_PARALLEL {
            for c in profile.sizes.iter().skip(parallel_rank) {
                let size = Size::new(&c.size);
                let Some((percent, true)) = try_option(&size, count) else {
                    continue;
                };
                list.push(Recommendation {
                    title: format!("{count} de {} por fase", size_label(&size)),
                    value: format!("{percent:.2} %"),
                    detail: format!(
                        "Conductores en paralelo; el agrupamiento se calculó con todos en la misma tubería. La norma permite paralelo a partir de {}.",
                        size_label(&Size::new(SMALLEST_PARALLEL_SIZE))
                    ),
                    apply: Some((size, count)),
                });
                break 'parallel;
            }
        }
    }

    let (title, detail) = if drop_fails {
        (
            format!("Mantener {}", size_label(&input.size)),
            format!(
                "Longitud máxima con la que la selección actual cumple el {limit:.2} % de caída. El tramo capturado mide {} m.",
                plain(input.length_m)
            ),
        )
    } else {
        (
            "Margen de longitud".to_string(),
            format!(
                "Longitud máxima con la que la selección actual sigue cumpliendo el {limit:.2} % de caída. El tramo capturado mide {} m.",
                plain(input.length_m)
            ),
        )
    };
    list.push(Recommendation {
        title,
        value: if max_length.is_finite() {
            format!("hasta {max_length:.1} m")
        } else {
            "sin límite".to_string()
        },
        detail,
        apply: None,
    });
    list
}
