//! Referencias normativas de las reglas de cálculo.
//!
//! La regla (la fórmula o el criterio que aplica el motor) y su referencia
//! (dónde lo dice la norma) son cosas separadas. Cada regla tiene un
//! identificador fijo; la cláusula que le corresponde en cada edición vive en
//! `data/<perfil>/references.csv`. Cambiar de edición es cambiar esos datos
//! y sus casos de prueba, sin tocar las fórmulas.
//!
//! En el repositorio solo hay identificadores de cláusulas y descripciones
//! propias, nunca el texto de la norma.

use voltforge_core::Size;

/// Dónde dice la norma lo que aplica una regla.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RuleReference {
    /// Clave de la norma, por ejemplo `NOM-001-SEDE-2012`.
    pub code: String,
    pub title: String,
    pub edition: String,
    /// Artículo, sección o tabla, por ejemplo `240-6(a)`.
    pub clause: Option<String>,
    pub source_url: Option<String>,
}

impl RuleReference {
    /// Texto corto para citar: `NOM-001-SEDE-2012, 240-6(a)`.
    pub fn cite(&self) -> String {
        match &self.clause {
            Some(clause) => format!("{}, {clause}", self.code),
            None => self.code.clone(),
        }
    }
}

/// Cita varias referencias en una línea, agrupadas por norma:
/// `NOM-001-SEDE-2012: Tabla 310-15(b)(16); 110-14(c)`.
pub fn cite_all(references: &[RuleReference]) -> Option<String> {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for r in references {
        let Some(clause) = r.clause.as_deref() else {
            continue;
        };
        match groups.iter_mut().find(|(code, _)| *code == r.code) {
            Some((_, clauses)) => {
                if !clauses.contains(&clause) {
                    clauses.push(clause);
                }
            }
            None => groups.push((&r.code, vec![clause])),
        }
    }
    (!groups.is_empty()).then(|| {
        groups
            .iter()
            .map(|(code, clauses)| format!("{code}: {}", clauses.join("; ")))
            .collect::<Vec<_>>()
            .join(" · ")
    })
}

/// Reglas que aplica el motor. El identificador no cambia entre ediciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rule {
    DesignCurrent,
    StandardRatings,
    ConductorProtection,
    SmallConductors,
    Terminals,
    Ampacity,
    TemperatureCorrection,
    Bundling,
    Neutral,
    Parallel,
    EquipmentGrounding,
    GroundingIncrease,
    ConductorProperties,
    Impedance,
    ResistanceTemperature,
    InsulatedDimensions,
    ConduitDimensions,
    ConduitFill,
    Nipple,
    Jamming,
    BranchDrop,
    FeederDrop,
    MotorFullLoadCurrent,
    MotorBranchConductor,
    MotorFeederConductor,
    MotorBranchProtection,
    MotorStarting,
    MotorFeederProtection,
    MotorOverload,
    MotorConductorProtection,
    CompressorRating,
    CompressorConductor,
    CompressorProtection,
    EquipmentNameplate,
    BreakerRating,
    LightingLoad,
    ReceptacleOutletLoad,
    DwellingReceptacles,
    OfficeReceptacleLoad,
    FeederLoad,
    LightingDemand,
    ReceptacleDemand,
    ApplianceDemand,
    KitchenDemand,
    InterruptingRating,
}

impl Rule {
    pub const ALL: [Rule; 45] = [
        Rule::DesignCurrent,
        Rule::StandardRatings,
        Rule::ConductorProtection,
        Rule::SmallConductors,
        Rule::Terminals,
        Rule::Ampacity,
        Rule::TemperatureCorrection,
        Rule::Bundling,
        Rule::Neutral,
        Rule::Parallel,
        Rule::EquipmentGrounding,
        Rule::GroundingIncrease,
        Rule::ConductorProperties,
        Rule::Impedance,
        Rule::ResistanceTemperature,
        Rule::InsulatedDimensions,
        Rule::ConduitDimensions,
        Rule::ConduitFill,
        Rule::Nipple,
        Rule::Jamming,
        Rule::BranchDrop,
        Rule::FeederDrop,
        Rule::MotorFullLoadCurrent,
        Rule::MotorBranchConductor,
        Rule::MotorFeederConductor,
        Rule::MotorBranchProtection,
        Rule::MotorStarting,
        Rule::MotorFeederProtection,
        Rule::MotorOverload,
        Rule::MotorConductorProtection,
        Rule::CompressorRating,
        Rule::CompressorConductor,
        Rule::CompressorProtection,
        Rule::EquipmentNameplate,
        Rule::BreakerRating,
        Rule::LightingLoad,
        Rule::ReceptacleOutletLoad,
        Rule::DwellingReceptacles,
        Rule::OfficeReceptacleLoad,
        Rule::FeederLoad,
        Rule::LightingDemand,
        Rule::ReceptacleDemand,
        Rule::ApplianceDemand,
        Rule::KitchenDemand,
        Rule::InterruptingRating,
    ];

    /// Identificador con el que aparece en `references.csv`.
    pub fn id(self) -> &'static str {
        match self {
            Rule::DesignCurrent => "design_current",
            Rule::StandardRatings => "standard_ratings",
            Rule::ConductorProtection => "conductor_protection",
            Rule::SmallConductors => "small_conductors",
            Rule::Terminals => "terminals",
            Rule::Ampacity => "ampacity",
            Rule::TemperatureCorrection => "temperature_correction",
            Rule::Bundling => "bundling",
            Rule::Neutral => "neutral",
            Rule::Parallel => "parallel",
            Rule::EquipmentGrounding => "equipment_grounding",
            Rule::GroundingIncrease => "grounding_increase",
            Rule::ConductorProperties => "conductor_properties",
            Rule::Impedance => "impedance",
            Rule::ResistanceTemperature => "resistance_temperature",
            Rule::InsulatedDimensions => "insulated_dimensions",
            Rule::ConduitDimensions => "conduit_dimensions",
            Rule::ConduitFill => "conduit_fill",
            Rule::Nipple => "nipple",
            Rule::Jamming => "jamming",
            Rule::BranchDrop => "branch_drop",
            Rule::FeederDrop => "feeder_drop",
            Rule::MotorFullLoadCurrent => "motor_full_load_current",
            Rule::MotorBranchConductor => "motor_branch_conductor",
            Rule::MotorFeederConductor => "motor_feeder_conductor",
            Rule::MotorBranchProtection => "motor_branch_protection",
            Rule::MotorStarting => "motor_starting",
            Rule::MotorFeederProtection => "motor_feeder_protection",
            Rule::MotorOverload => "motor_overload",
            Rule::MotorConductorProtection => "motor_conductor_protection",
            Rule::CompressorRating => "compressor_rating",
            Rule::CompressorConductor => "compressor_conductor",
            Rule::CompressorProtection => "compressor_protection",
            Rule::EquipmentNameplate => "equipment_nameplate",
            Rule::BreakerRating => "breaker_rating",
            Rule::LightingLoad => "lighting_load",
            Rule::ReceptacleOutletLoad => "receptacle_outlet_load",
            Rule::DwellingReceptacles => "dwelling_receptacles",
            Rule::OfficeReceptacleLoad => "office_receptacle_load",
            Rule::FeederLoad => "feeder_load",
            Rule::LightingDemand => "lighting_demand",
            Rule::ReceptacleDemand => "receptacle_demand",
            Rule::ApplianceDemand => "appliance_demand",
            Rule::KitchenDemand => "kitchen_demand",
            Rule::InterruptingRating => "interrupting_rating",
        }
    }
}

/// Valores normativos que no son tablas: factores, límites y calibres mínimos.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameters {
    /// Factor de la carga continua para la corriente de diseño (210-19(a)(1)).
    pub continuous_load_factor: f64,
    /// Hasta esta corriente las terminales se toman de 60 °C (110-14(c)).
    pub terminal_60c_limit_a: f64,
    /// Hasta esta capacidad se admite el interruptor normalizado inmediato
    /// superior a la ampacidad del conductor (240-4(b)).
    pub next_size_up_limit_a: f64,
    /// Calibre mínimo de un circuito.
    pub smallest_circuit_size: Size,
    /// Calibre mínimo para conductores en paralelo (310-10(h)).
    pub smallest_parallel_size: Size,
    /// Intervalo de la relación de diámetros con riesgo de atascamiento.
    pub jam_ratio_min: f64,
    pub jam_ratio_max: f64,
    /// Ocupación permitida en niples de 60 cm o menos.
    pub nipple_fill_fraction: f64,
    /// Conductores de un solo motor, sobre su corriente a plena carga (430-22).
    pub motor_conductor_factor: f64,
    /// Motor mayor de un grupo, en el alimentador (430-24(1)).
    pub motor_largest_factor: f64,
    /// Relevador de sobrecarga con factor de servicio de 1.15 o más, o
    /// elevación de temperatura de 40 °C o menos (430-32(a)(1)).
    pub motor_overload_factor_marked: f64,
    /// Relevador de sobrecarga de los demás motores (430-32(a)(1)).
    pub motor_overload_factor_other: f64,
    /// Corriente del motor hasta la que aplica el factor «small» de arranque
    /// del interruptor de tiempo inverso (430-52(c)(1), excepción 2(3)).
    pub motor_starting_limit_a: f64,
    pub motor_starting_factor_small: f64,
    pub motor_starting_factor_large: f64,
    /// Conductores de un motocompresor (440-32).
    pub compressor_conductor_factor: f64,
    /// Protección de un motocompresor y su máximo si no arranca (440-22(a)).
    pub compressor_protection_factor: f64,
    pub compressor_protection_max_factor: f64,
    /// No se exige una protección menor que esta (440-22(a), excepción).
    pub compressor_minimum_protection_a: f64,
    /// Carga mínima de cada salida de contacto, en VA (220-14(i)).
    pub receptacle_outlet_va: f64,
    /// Factor de los aparatos fijos de una vivienda (220-53) y número mínimo
    /// de aparatos en el alimentador para aplicarlo.
    pub appliance_demand_factor: f64,
    pub appliance_demand_min_count: u32,
}

/// Datos de la norma de un perfil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormativeDocument {
    pub code: String,
    pub title: String,
    pub edition: String,
    pub source_url: Option<String>,
}

/// Cláusula y descripción de una regla en un perfil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseEntry {
    pub clause: String,
    pub description: String,
}
