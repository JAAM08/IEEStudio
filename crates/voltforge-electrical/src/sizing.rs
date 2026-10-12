//! Selección de conductor, interruptor y conductor de puesta a tierra.
//!
//! Criterios (NOM-001-SEDE-2012):
//!
//! 1. Interruptor: capacidad normalizada (240-6) igual o inmediata superior a
//!    la corriente de diseño, que es el 125 % de la carga continua más la no
//!    continua.
//! 2. Terminales (110-14(c)): hasta 100 A se usa la columna de 60 °C y arriba
//!    de 100 A la de 75 °C, sin exceder la temperatura del aislamiento.
//! 3. Tamaño mínimo (210-19(a), 215-2): la ampacidad en la columna de
//!    terminales, sin factores, no debe ser menor que la corriente de diseño.
//! 4. Condiciones de uso (310-15(b)): la ampacidad en la columna del
//!    aislamiento, corregida por temperatura ambiente y por agrupamiento, no
//!    debe ser menor que la carga.
//! 5. Protección (240-4): el interruptor no debe exceder la ampacidad del
//!    conductor, salvo el valor normalizado inmediato superior hasta 800 A, y
//!    los límites de 240-4(d) para conductores pequeños.
//! 6. Paralelo (310-10(h)): solo desde 1/0 AWG.
//! 7. Puesta a tierra (250-122): por la capacidad del interruptor, con el
//!    incremento proporcional cuando los conductores de fase se sobredimensionan.

use crate::current::design_current;
use crate::voltage_drop::{voltage_drop, VoltageDropInput};
use voltforge_core::{Error, Insulation, Material, Result, Size, TempRating};
use voltforge_rules::{Profile, Rule};

/// Límite de caída de tensión que el conductor también debe cumplir.
#[derive(Debug, Clone, PartialEq)]
pub struct VoltageDropLimit {
    /// Datos del tramo. El calibre y los conductores por fase se sustituyen
    /// por los del conductor que se evalúa.
    pub input: VoltageDropInput,
    pub limit_percent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizingInput {
    /// Corriente de la carga continua, en amperes.
    pub continuous_a: f64,
    /// Corriente de la carga no continua, en amperes.
    pub noncontinuous_a: f64,
    pub material: Material,
    /// Material del conductor de puesta a tierra de equipos.
    pub grounding_material: Material,
    pub insulation: Insulation,
    pub wet_location: bool,
    /// Temperatura ambiente, en °C.
    pub ambient_c: f64,
    /// Conductores portadores de corriente en la canalización.
    pub current_carrying_conductors: u32,
    /// Tramo de 60 cm o menos: no se aplica el factor de agrupamiento.
    pub nipple: bool,
    pub conductors_per_phase: u32,
    /// Temperatura de las terminales. `None` aplica la regla de 110-14(c).
    pub terminal_rating: Option<TempRating>,
    /// Admite también las capacidades de 16, 32 y 63 A.
    pub include_iec_ratings: bool,
    /// Calibre mínimo impuesto por el usuario.
    pub minimum_size: Option<Size>,
    pub voltage_drop: Option<VoltageDropLimit>,
}

/// Verificaciones de un calibre para el circuito.
#[derive(Debug, Clone, PartialEq)]
pub struct ConductorCheck {
    pub size: Size,
    pub conductors_per_phase: u32,
    /// Columna del aislamiento usada para las condiciones de uso.
    pub insulation_rating: TempRating,
    /// Columna usada para las terminales.
    pub terminal_rating: TempRating,
    /// Ampacidad de tabla en la columna del aislamiento, por conductor.
    pub table_ampacity_a: f64,
    pub temperature_factor: f64,
    pub bundling_factor: f64,
    /// Ampacidad corregida del conjunto de conductores por fase.
    pub corrected_ampacity_a: f64,
    /// Ampacidad en la columna de terminales del conjunto, sin factores.
    pub terminal_ampacity_a: f64,
    /// Ampacidad permitida: la menor de las dos anteriores.
    pub allowable_ampacity_a: f64,
    /// La ampacidad de terminales cubre la corriente de diseño.
    pub meets_design_current: bool,
    /// La ampacidad corregida cubre la carga.
    pub meets_load: bool,
    /// El interruptor protege al conductor (240-4).
    pub protected: bool,
    /// Límite de 240-4(d), si el calibre lo tiene.
    pub small_conductor_limit_a: Option<f64>,
    /// El calibre puede instalarse en paralelo, si se pidió más de uno por fase.
    pub parallel_allowed: bool,
    pub voltage_drop_percent: Option<f64>,
    pub meets_voltage_drop: Option<bool>,
}

impl ConductorCheck {
    /// Cumple ampacidad y paralelo, sin considerar protección ni caída.
    pub fn meets_ampacity(&self) -> bool {
        self.meets_design_current && self.meets_load && self.parallel_allowed
    }

    pub fn passes(&self) -> bool {
        self.meets_ampacity() && self.protected && self.meets_voltage_drop.unwrap_or(true)
    }
}

/// Criterio que determinó el calibre seleccionado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Governing {
    Ampacity,
    Protection,
    VoltageDrop,
    UserMinimum,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grounding {
    /// Calibre de la Tabla 250-122 para el interruptor.
    pub table_size: Size,
    /// Calibre requerido tras el incremento proporcional de 250-122(b).
    pub size: Size,
    pub increased: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizingResult {
    pub load_a: f64,
    pub design_a: f64,
    pub breaker_a: f64,
    pub conductor: ConductorCheck,
    /// Menor calibre que cumple ampacidad y protección, sin caída de tensión.
    pub ampacity_size: Size,
    pub governing: Governing,
    pub grounding: Grounding,
}

/// Capacidad normalizada igual o inmediata superior a una corriente.
pub fn standard_rating_at_least(
    profile: &Profile,
    current_a: f64,
    include_iec: bool,
) -> Result<f64> {
    profile
        .standard_ratings
        .iter()
        .filter(|r| r.common || include_iec)
        .map(|r| r.rating_a)
        .find(|&r| r >= current_a - 1e-9)
        .ok_or(Error::NotInTable {
            table: profile.clause(Rule::StandardRatings),
            what: format!("una capacidad de al menos {current_a:.2} A"),
        })
}

fn validate(input: &SizingInput) -> Result<TempRating> {
    for (name, v) in [
        ("continua", input.continuous_a),
        ("no continua", input.noncontinuous_a),
    ] {
        if !(v.is_finite() && v >= 0.0) {
            return Err(Error::InvalidInput(format!(
                "la corriente {name} debe ser un número no negativo"
            )));
        }
    }
    if input.conductors_per_phase == 0 {
        return Err(Error::InvalidInput(
            "debe haber al menos un conductor por fase".into(),
        ));
    }
    input.insulation.rating(input.wet_location).ok_or_else(|| {
        Error::InvalidInput(format!(
            "el aislamiento {:?} no se permite en lugares mojados",
            input.insulation
        ))
    })
}

/// Evalúa un calibre concreto contra un interruptor dado.
pub fn check_conductor(
    profile: &Profile,
    input: &SizingInput,
    size: &Size,
    breaker_a: f64,
) -> Result<ConductorCheck> {
    let insulation_rating = validate(input)?;
    let load_a = input.continuous_a + input.noncontinuous_a;
    let design_a = design_current(profile, input.continuous_a, input.noncontinuous_a);
    let n = f64::from(input.conductors_per_phase);

    let terminal_rule =
        input
            .terminal_rating
            .unwrap_or(if breaker_a <= profile.parameters.terminal_60c_limit_a {
                TempRating::C60
            } else {
                TempRating::C75
            });
    let terminal_rating = terminal_rule.min(insulation_rating);

    let table_ampacity_a = profile.ampacity(size, input.material, insulation_rating)?;
    let temperature_factor = profile.temperature_factor(input.ambient_c, insulation_rating)?;
    let bundling_factor = if input.nipple {
        1.0
    } else {
        profile.bundling_factor(input.current_carrying_conductors)
    };
    let corrected_ampacity_a = table_ampacity_a * temperature_factor * bundling_factor * n;
    let terminal_ampacity_a = profile.ampacity(size, input.material, terminal_rating)? * n;
    let allowable_ampacity_a = corrected_ampacity_a.min(terminal_ampacity_a);

    let small_conductor_limit_a = profile
        .small_conductor_limits
        .iter()
        .find(|l| l.material == input.material && l.size == size.0)
        .map(|l| l.max_overcurrent_a);
    let within_small_limit = small_conductor_limit_a.map_or(true, |limit| breaker_a <= limit);
    let next_up =
        standard_rating_at_least(profile, allowable_ampacity_a, input.include_iec_ratings);
    let protected_by_rating = breaker_a <= allowable_ampacity_a + 1e-9
        || (breaker_a <= profile.parameters.next_size_up_limit_a
            && next_up.is_ok_and(|r| breaker_a <= r));

    let parallel_allowed = input.conductors_per_phase == 1
        || profile.size_rank(size)?
            >= profile.size_rank(&profile.parameters.smallest_parallel_size)?;

    let (voltage_drop_percent, meets_voltage_drop) = match &input.voltage_drop {
        Some(limit) => {
            let vd = voltage_drop(
                profile,
                &VoltageDropInput {
                    size: size.clone(),
                    conductors_per_phase: input.conductors_per_phase,
                    ..limit.input.clone()
                },
            )?;
            (
                Some(vd.drop_percent),
                Some(vd.drop_percent <= limit.limit_percent),
            )
        }
        None => (None, None),
    };

    Ok(ConductorCheck {
        size: size.clone(),
        conductors_per_phase: input.conductors_per_phase,
        insulation_rating,
        terminal_rating,
        table_ampacity_a,
        temperature_factor,
        bundling_factor,
        corrected_ampacity_a,
        terminal_ampacity_a,
        allowable_ampacity_a,
        meets_design_current: terminal_ampacity_a >= design_a - 1e-9,
        meets_load: corrected_ampacity_a >= load_a - 1e-9,
        protected: protected_by_rating && within_small_limit,
        small_conductor_limit_a,
        parallel_allowed,
        voltage_drop_percent,
        meets_voltage_drop,
    })
}

/// Conductor de puesta a tierra de equipos (250-122).
///
/// `ampacity_size` es el calibre de fase mínimo por ampacidad y `phase_size`
/// el instalado; si el segundo es mayor, el de puesta a tierra crece en la
/// misma proporción de áreas, sin exceder el calibre de fase cuando ambos
/// son del mismo material.
pub fn equipment_grounding(
    profile: &Profile,
    breaker_a: f64,
    material: Material,
    same_material_as_phase: bool,
    ampacity_size: &Size,
    phase_size: &Size,
) -> Result<Grounding> {
    let missing = || Error::NotInTable {
        table: profile.clause(Rule::EquipmentGrounding),
        what: format!("{material:?} con un dispositivo de {breaker_a} A"),
    };
    let row = profile
        .grounding
        .iter()
        .find(|r| breaker_a <= r.max_overcurrent_a)
        .ok_or_else(missing)?;
    let table_size = Size::new(&match material {
        Material::Copper => row.copper_size.clone(),
        Material::Aluminum => row.aluminum_size.clone().ok_or_else(missing)?,
    });
    let phase = profile.conductor(phase_size)?;
    let base = profile.conductor(ampacity_size)?;
    if phase.area_mm2 <= base.area_mm2 {
        return Ok(Grounding {
            size: table_size.clone(),
            table_size,
            increased: false,
        });
    }
    let required = profile.conductor(&table_size)?.area_mm2 * phase.area_mm2 / base.area_mm2;
    let grown = profile
        .sizes
        .iter()
        .find(|c| c.area_mm2 >= required - 1e-9)
        .map_or(phase, |c| {
            if same_material_as_phase && c.area_mm2 > phase.area_mm2 {
                phase
            } else {
                c
            }
        });
    let size = Size::new(&grown.size);
    Ok(Grounding {
        increased: size != table_size,
        size,
        table_size,
    })
}

/// Selecciona el interruptor, el menor conductor que cumple y su puesta a tierra.
pub fn select_conductor(profile: &Profile, input: &SizingInput) -> Result<SizingResult> {
    validate(input)?;
    let load_a = input.continuous_a + input.noncontinuous_a;
    let design_a = design_current(profile, input.continuous_a, input.noncontinuous_a);
    let breaker_a = standard_rating_at_least(profile, design_a, input.include_iec_ratings)?;

    let start = profile.size_rank(&profile.parameters.smallest_circuit_size)?;
    let user_min = match &input.minimum_size {
        Some(s) => profile.size_rank(s)?,
        None => start,
    };

    let mut by_ampacity = None;
    let mut by_protection = None;
    let mut chosen = None;
    for (rank, c) in profile.sizes.iter().enumerate().skip(start) {
        let size = Size::new(&c.size);
        // Un calibre sin valor en alguna tabla no es candidato.
        let Ok(check) = check_conductor(profile, input, &size, breaker_a) else {
            continue;
        };
        if check.meets_ampacity() {
            by_ampacity.get_or_insert(rank);
            if check.protected {
                by_protection.get_or_insert(rank);
                if rank >= user_min && check.passes() {
                    chosen = Some((rank, check));
                    break;
                }
            }
        }
    }
    let (rank, conductor) = chosen.ok_or(Error::NotInTable {
        table: profile.clause(Rule::Ampacity),
        what: format!(
            "un calibre que cumpla con {} conductor(es) por fase para {load_a:.2} A",
            input.conductors_per_phase
        ),
    })?;
    let ampacity_rank = by_ampacity.unwrap_or(rank);
    let protection_rank = by_protection.unwrap_or(rank);
    let governing = if rank > protection_rank.max(user_min) {
        Governing::VoltageDrop
    } else if user_min > protection_rank {
        Governing::UserMinimum
    } else if protection_rank > ampacity_rank {
        Governing::Protection
    } else {
        Governing::Ampacity
    };
    let ampacity_size = Size::new(&profile.sizes[protection_rank].size);
    let grounding = equipment_grounding(
        profile,
        breaker_a,
        input.grounding_material,
        input.grounding_material == input.material,
        &ampacity_size,
        &conductor.size,
    )?;
    Ok(SizingResult {
        load_a,
        design_a,
        breaker_a,
        conductor,
        ampacity_size,
        governing,
        grounding,
    })
}
