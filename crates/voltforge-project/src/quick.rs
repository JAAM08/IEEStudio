//! Cálculo rápido de caída de tensión de un solo circuito, sin proyecto.
//!
//! Devuelve el resultado, el procedimiento paso a paso con los valores
//! sustituidos y, cuando no cumple, alternativas ya calculadas.

use serde::{Deserialize, Serialize};
use voltforge_core::current::nominal_current;
use voltforge_core::sizing::{SMALLEST_PARALLEL_SIZE, TERMINAL_60C_LIMIT_A};
use voltforge_core::tables::TempRating;
use voltforge_core::voltage_drop::{
    max_length_m, temperature_coefficient, voltage_drop, VoltageDropInput, VoltageDropResult,
    TABLE_9_TEMPERATURE_C,
};
use voltforge_core::{CircuitKind, ConduitType, Material, Profile, RacewayMaterial, Size, System};

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
    /// Longitud de un sentido, en metros.
    pub length_m: f64,
    pub size: Size,
    pub conductors_per_phase: u32,
    pub material: Material,
    pub conduit: ConduitType,
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
            length_m: 95.0,
            size: Size::new("3/0"),
            conductors_per_phase: 1,
            material: Material::Copper,
            conduit: ConduitType::Emt,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickResult {
    pub current_a: f64,
    pub drop: VoltageDropResult,
    pub status: Status,
    /// Longitud máxima con la que la selección actual cumple el límite.
    pub max_length_m: f64,
    /// Ampacidad de tabla de la selección, sin factores de corrección.
    pub table_ampacity_a: Option<f64>,
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

/// Ampacidad de tabla de `n` conductores por fase, en la columna de las
/// terminales: 60 °C hasta 100 A y 75 °C por encima. Sin factores de corrección.
fn table_ampacity(
    profile: &Profile,
    size: &Size,
    material: Material,
    n: u32,
    current_a: f64,
) -> Option<f64> {
    let column = if current_a <= TERMINAL_60C_LIMIT_A {
        TempRating::C60
    } else {
        TempRating::C75
    };
    profile
        .ampacity(size, material, column)
        .ok()
        .map(|a| a * f64::from(n))
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

/// Calcula la caída de tensión de un circuito suelto.
///
/// Devuelve un error con explicación cuando las tablas no cubren el caso o un
/// dato no es válido; ese caso se muestra como «no evaluado».
pub fn quick_voltage_drop(profile: &Profile, input: &QuickInput) -> Result<QuickResult, String> {
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
    let complies = drop.drop_percent <= input.limit_percent;
    let n = input.conductors_per_phase;
    let rank = profile.size_rank(&input.size).map_err(|e| e.to_string())?;
    let parallel_rank = profile
        .size_rank(&Size::new(SMALLEST_PARALLEL_SIZE))
        .map_err(|e| e.to_string())?;
    let ampacity = table_ampacity(profile, &input.size, input.material, n, current_a);

    let mut warnings = Vec::new();
    if n > 1 && rank < parallel_rank {
        warnings.push(format!(
            "La norma permite conductores en paralelo a partir de {}; el {} no se puede instalar en paralelo.",
            size_label(&Size::new(SMALLEST_PARALLEL_SIZE)),
            size_label(&input.size)
        ));
    }
    if let Some(a) = ampacity {
        if a < current_a {
            warnings.push(format!(
                "La corriente de {current_a:.2} A supera la ampacidad de tabla de la selección ({a:.0} A, sin factores de corrección)."
            ));
        }
    }

    Ok(QuickResult {
        current_a,
        steps: steps(profile, input, current_a, &drop),
        recommendations: recommendations(
            profile,
            input,
            &base,
            &drop,
            max_length,
            rank,
            parallel_rank,
        ),
        status: if complies {
            Status::Complies
        } else {
            Status::Fails
        },
        max_length_m: max_length,
        table_ampacity_a: ampacity,
        drop,
        warnings,
    })
}

fn steps(
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

    if input.load_in_amperes {
        steps.push(Step {
            title: "Corriente del circuito".into(),
            formula: None,
            substitution: format!("I = {current_a:.2} A"),
            note: Some("Corriente capturada directamente.".into()),
        });
    } else {
        let p = plain(input.power_w);
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
        steps.push(Step {
            title: "Corriente nominal".into(),
            formula: Some(formula.into()),
            substitution,
            note: Some(voltage.into()),
        });
    }

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
        formula: Some(format!("ΔV = {k_symbol} · I · L · (R · cos θ + X · sen θ) / n")),
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
    max_length: f64,
    rank: usize,
    parallel_rank: usize,
) -> Vec<Recommendation> {
    let limit = input.limit_percent;
    let n = input.conductors_per_phase;
    let mut list = Vec::new();

    // Caída y suficiencia de ampacidad de tabla de una alternativa.
    let try_option = |size: &Size, count: u32| -> Option<f64> {
        let candidate = VoltageDropInput {
            size: size.clone(),
            conductors_per_phase: count,
            ..base.clone()
        };
        let percent = voltage_drop(profile, &candidate).ok()?.drop_percent;
        let ampacity = table_ampacity(profile, size, input.material, count, base.current_a)?;
        (ampacity >= base.current_a).then_some(percent)
    };

    if drop.drop_percent > limit {
        // Calibre mayor con los mismos conductores por fase.
        let mut skipped = Vec::new();
        for c in profile.sizes.iter().skip(rank + 1) {
            let size = Size::new(&c.size);
            let Some(percent) = try_option(&size, n) else {
                continue;
            };
            if percent <= limit {
                let per_phase = if n == 1 {
                    "un conductor por fase".to_string()
                } else {
                    format!("{n} conductores por fase")
                };
                let mut detail = format!("Calibre mínimo que cumple con {per_phase}.");
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
            skipped.push(format!("{} queda en {percent:.2} %", size_label(&size)));
        }

        // Más conductores por fase, con el menor calibre que cumple.
        'parallel: for count in (n + 1).max(2)..=MAX_SUGGESTED_PARALLEL {
            for c in profile.sizes.iter().skip(parallel_rank) {
                let size = Size::new(&c.size);
                let Some(percent) = try_option(&size, count) else {
                    continue;
                };
                if percent <= limit {
                    list.push(Recommendation {
                        title: format!("{count} de {} por fase", size_label(&size)),
                        value: format!("{percent:.2} %"),
                        detail: format!(
                            "Conductores en paralelo, cada juego en su tubería. La norma permite paralelo a partir de {}.",
                            size_label(&Size::new(SMALLEST_PARALLEL_SIZE))
                        ),
                        apply: Some((size, count)),
                    });
                    break 'parallel;
                }
            }
        }
    }

    let (title, detail) = if drop.drop_percent > limit {
        (
            format!("Mantener {}", size_label(&input.size)),
            format!(
                "Longitud máxima con la que la selección actual cumple el {limit:.2} %. El tramo capturado mide {} m.",
                plain(input.length_m)
            ),
        )
    } else {
        (
            "Margen de longitud".to_string(),
            format!(
                "Longitud máxima con la que la selección actual sigue cumpliendo el {limit:.2} %. El tramo capturado mide {} m.",
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
