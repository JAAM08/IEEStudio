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
}

impl Rule {
    pub const ALL: [Rule; 22] = [
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
