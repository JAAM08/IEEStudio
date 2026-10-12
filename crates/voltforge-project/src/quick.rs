//! Cálculo rápido de un solo circuito, sin proyecto: corriente, protección,
//! verificación del conductor y caída de tensión.
//!
//! Devuelve el resultado, el procedimiento paso a paso con los valores
//! sustituidos y, cuando no cumple, alternativas ya calculadas.

use serde::{Deserialize, Serialize};
use voltforge_core::{CircuitKind, ConduitType, Material, Size, System};
use voltforge_electrical::conduit::{size_conduit, ConductorGroup, ConduitInput, Covering};
use voltforge_electrical::current::{design_current, nominal_current};
use voltforge_electrical::sizing::{
    check_conductor, equipment_grounding, select_conductor, standard_rating_at_least, SizingInput,
    VoltageDropLimit, SMALLEST_PARALLEL_SIZE,
};
use voltforge_electrical::voltage_drop::{
    max_length_m, voltage_drop, VoltageDropInput, VoltageDropResult,
};

use crate::calc::{is_wye, poles};
use crate::procedure::{
    conductor_steps, conduit_step, current_step, drop_steps, plain, size_label, ConductorData, Step,
};
use crate::Status;
use voltforge_core::Insulation;
use voltforge_rules::Profile;

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
    /// Tubería para las fases, el neutro y la tierra desnuda.
    pub conduit: Option<String>,
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

fn protection(
    profile: &Profile,
    input: &QuickInput,
    current_a: f64,
) -> Result<(Protection, Vec<Step>), String> {
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
    let norm = profile.id.to_uppercase();
    let mut steps = conductor_steps(&ConductorData {
        norm: &norm,
        continuous_a: sizing.continuous_a,
        noncontinuous_a: sizing.noncontinuous_a,
        design_a,
        breaker_a,
        poles: poles(input.kind),
        material: input.material,
        insulation: input.insulation,
        ambient_c: input.ambient_c,
        current_carrying_conductors: sizing.current_carrying_conductors,
        check: &check,
        minimum_size: minimum_size.as_ref(),
        grounding: grounding_size.as_ref().map(|g| (g, input.material)),
    });

    // Tubería: fases y neutro del calibre elegido, más la tierra desnuda.
    let has_neutral = input.kind == CircuitKind::LineToNeutral || input.has_neutral;
    let mut groups = vec![ConductorGroup {
        count: (poles(input.kind) + u32::from(has_neutral)) * n,
        size: input.size.clone(),
        covering: Covering::Insulated(input.insulation.dimension_group()),
    }];
    if let Some(ground) = &grounding_size {
        groups.push(ConductorGroup {
            count: 1,
            size: ground.clone(),
            covering: Covering::Bare,
        });
    }
    let conduit = size_conduit(
        profile,
        &ConduitInput {
            groups,
            conduit: input.conduit,
            nipple: false,
        },
    )
    .ok()
    .map(|c| {
        steps.push(conduit_step(&c, input.conduit, false));
        format!(
            "{} de {} mm ({}\")",
            input.conduit, c.conduit.metric_designator, c.conduit.trade_size
        )
    });

    let meets_ampacity = check.meets_ampacity();
    let result = Protection {
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
        conduit,
        status: if meets_ampacity && check.protected {
            Status::Complies
        } else {
            Status::Fails
        },
    };
    Ok((result, steps))
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

    let mut steps = vec![current_step(
        input.kind,
        (!input.load_in_amperes).then_some(input.power_w),
        input.power_factor,
        drop.reference_voltage_v,
        current_a,
    )];
    let protection = match protection(profile, input, current_a) {
        Ok((p, conductor)) => {
            steps.extend(conductor);
            Ok(p)
        }
        Err(e) => {
            steps.push(Step {
                title: "Protección y conductor".into(),
                formula: None,
                substitution: "No evaluado".into(),
                note: Some(format!("No se pudo calcular: {e}.")),
            });
            Err(e)
        }
    };
    let conductor_fails = matches!(&protection, Ok(p) if p.status == Status::Fails);
    steps.extend(drop_steps(
        &profile.id.to_uppercase(),
        &base,
        &drop,
        input.limit_percent,
    ));

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
