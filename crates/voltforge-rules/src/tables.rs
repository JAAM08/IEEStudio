//! Tablas normativas del perfil activo.
//!
//! Los valores no están escritos en el código: se leen de los archivos CSV de
//! `data/<perfil>/`, que se incrustan en el binario al compilar.

use std::collections::HashMap;
use std::sync::OnceLock;

use voltforge_core::{
    ConduitType, Error, InsulationGroup, Material, RacewayMaterial, Result, Size, TempRating,
};

use crate::csv;
use crate::references::{ClauseEntry, NormativeDocument, Parameters, Rule, RuleReference};

/// Fila de la Tabla 8 (cableado clase B).
#[derive(Debug, Clone)]
pub struct ConductorProperties {
    pub size: String,
    pub area_mm2: f64,
    pub kcmil: Option<f64>,
    pub strands: u32,
    pub bare_diameter_mm: f64,
    pub bare_area_mm2: f64,
}

/// Fila de la Tabla 9: ohms al neutro por kilómetro, a 75 °C.
#[derive(Debug, Clone)]
pub struct ImpedanceRow {
    pub size: String,
    pub x_pvc_or_aluminum: f64,
    pub x_steel: f64,
    pub r_cu_pvc: Option<f64>,
    pub r_cu_aluminum: Option<f64>,
    pub r_cu_steel: Option<f64>,
    pub r_al_pvc: Option<f64>,
    pub r_al_aluminum: Option<f64>,
    pub r_al_steel: Option<f64>,
}

/// Fila de la Tabla 5.
#[derive(Debug, Clone)]
pub struct InsulatedConductor {
    pub insulation_group: String,
    pub size: String,
    pub diameter_mm: f64,
    pub area_mm2: f64,
}

/// Fila de la Tabla 4.
#[derive(Debug, Clone, PartialEq)]
pub struct ConduitRow {
    pub conduit_type: String,
    pub metric_designator: u32,
    pub trade_size: String,
    pub internal_diameter_mm: f64,
    pub area_100_mm2: f64,
    pub area_60_mm2: f64,
    pub area_53_mm2: f64,
    pub area_31_mm2: f64,
    pub area_40_mm2: f64,
}

/// Fila de la Tabla 310-15(b)(16).
#[derive(Debug, Clone)]
pub struct AmpacityRow {
    pub size: String,
    pub cu_60c: Option<f64>,
    pub cu_75c: Option<f64>,
    pub cu_90c: Option<f64>,
    pub al_60c: Option<f64>,
    pub al_75c: Option<f64>,
    pub al_90c: Option<f64>,
}

/// Fila de la Tabla 310-15(b)(2)(a).
#[derive(Debug, Clone)]
pub struct TemperatureCorrectionRow {
    pub ambient_min_c: Option<f64>,
    pub ambient_max_c: f64,
    pub factor_60c: Option<f64>,
    pub factor_75c: Option<f64>,
    pub factor_90c: Option<f64>,
}

/// Fila de la Tabla 310-15(b)(3)(a).
#[derive(Debug, Clone)]
pub struct BundlingAdjustmentRow {
    pub conductors_min: u32,
    pub conductors_max: Option<u32>,
    pub factor: f64,
}

/// Fila de la Tabla 1 del capítulo 10.
#[derive(Debug, Clone)]
pub struct FillRow {
    pub conductors_min: u32,
    pub conductors_max: Option<u32>,
    pub fill_fraction: f64,
}

/// Fila de la Tabla 250-122.
#[derive(Debug, Clone)]
pub struct GroundingRow {
    pub max_overcurrent_a: f64,
    pub copper_size: String,
    pub aluminum_size: Option<String>,
}

/// Capacidad normalizada de un dispositivo de sobrecorriente (240-6).
#[derive(Debug, Clone, PartialEq)]
pub struct StandardRating {
    pub rating_a: f64,
    /// `false` para los valores de la serie IEC (16, 32 y 63 A), poco comunes
    /// en interruptores termomagnéticos del mercado mexicano.
    pub common: bool,
}

/// Límite de protección de un conductor pequeño (240-4(d)).
#[derive(Debug, Clone)]
pub struct SmallConductorLimit {
    pub material: Material,
    pub size: String,
    pub max_overcurrent_a: f64,
}

/// Fila de las tablas 430-248 (una fase) o 430-250 (tres fases): corriente a
/// plena carga por tensión nominal del motor.
#[derive(Debug, Clone)]
pub struct MotorCurrentRow {
    /// Potencia en caballos de fuerza, como la escribe la tabla: `7-1/2`.
    pub hp: String,
    pub kw: f64,
    /// Corriente por columna de tensión (`v115`, `v230`…); vacía si la tabla no la da.
    pub currents: Vec<(String, Option<f64>)>,
}

/// Columna de una tabla de motores que corresponde a un intervalo de
/// tensiones del sistema.
#[derive(Debug, Clone)]
pub struct MotorVoltageColumn {
    pub table: String,
    pub column: String,
    pub system_min_v: f64,
    pub system_max_v: f64,
}

/// Fila de la Tabla 430-52, en porciento de la corriente a plena carga.
#[derive(Debug, Clone)]
pub struct MotorProtectionRow {
    pub motor_type: String,
    pub nontime_fuse: f64,
    pub dual_element_fuse: f64,
    pub instantaneous_breaker: f64,
    pub inverse_time_breaker: f64,
}

/// Fila de la Tabla 220-12: tipo de inmueble o de área y su carga mínima de
/// alumbrado general.
#[derive(Debug, Clone, PartialEq)]
pub struct Occupancy {
    /// Identificador que se guarda en el proyecto: `dwellings`, `offices`…
    pub id: String,
    pub label: String,
    pub va_per_m2: f64,
    /// Unidad de vivienda: los contactos van con el alumbrado (220-14(j)).
    pub dwelling: bool,
    /// Área dentro de otro inmueble (pasillos, bodegas…), no un tipo de
    /// inmueble por sí misma.
    pub area_only: bool,
}

/// Escalón de una tabla de factores de demanda: el porciento se aplica a la
/// parte de la carga entre el escalón anterior y `up_to_va`.
#[derive(Debug, Clone, PartialEq)]
pub struct DemandTier {
    /// Tipo de inmueble al que aplica; vacío si la tabla no distingue.
    pub occupancy: String,
    /// Límite superior del escalón. `None` en el último.
    pub up_to_va: Option<f64>,
    pub percent: f64,
}

/// Fila de la Tabla 220-56.
#[derive(Debug, Clone, PartialEq)]
pub struct KitchenDemandRow {
    pub units_min: u32,
    pub units_max: Option<u32>,
    pub percent: f64,
}

/// Perfil normativo: el conjunto de tablas con el que se calcula.
#[derive(Debug)]
pub struct Profile {
    pub id: &'static str,
    pub title: &'static str,
    /// Calibres en orden creciente (Tabla 8).
    pub sizes: Vec<ConductorProperties>,
    size_index: HashMap<String, usize>,
    impedance: HashMap<String, ImpedanceRow>,
    insulated: HashMap<(String, String), InsulatedConductor>,
    pub conduits: Vec<ConduitRow>,
    pub ampacity: Vec<AmpacityRow>,
    pub temperature_correction: Vec<TemperatureCorrectionRow>,
    pub bundling_adjustment: Vec<BundlingAdjustmentRow>,
    pub fill: Vec<FillRow>,
    pub grounding: Vec<GroundingRow>,
    pub standard_ratings: Vec<StandardRating>,
    pub small_conductor_limits: Vec<SmallConductorLimit>,
    /// Tabla 430-248.
    pub single_phase_motors: Vec<MotorCurrentRow>,
    /// Tabla 430-250.
    pub three_phase_motors: Vec<MotorCurrentRow>,
    pub motor_voltage_columns: Vec<MotorVoltageColumn>,
    /// Tabla 430-52.
    pub motor_protection: Vec<MotorProtectionRow>,
    /// Tabla 220-12.
    pub occupancies: Vec<Occupancy>,
    /// Tabla 220-42. Un tipo de inmueble sin filas va al 100 %.
    pub lighting_demand: Vec<DemandTier>,
    /// Tabla 220-44.
    pub receptacle_demand: Vec<DemandTier>,
    /// 220-14(k): carga mínima de contactos por m², por tipo de inmueble.
    pub office_receptacles: Vec<(String, f64)>,
    /// Tabla 220-56.
    pub kitchen_demand: Vec<KitchenDemandRow>,
    /// Norma de la que salen las tablas.
    pub document: NormativeDocument,
    /// Factores, límites y calibres mínimos del perfil.
    pub parameters: Parameters,
    references: HashMap<String, ClauseEntry>,
}

macro_rules! table {
    ($profile:literal, $file:literal, $build:expr) => {
        csv::parse(
            $file,
            include_str!(concat!("../../../data/", $profile, "/", $file)),
            $build,
        )
    };
}

impl Profile {
    /// NOM-001-SEDE-2012, Instalaciones Eléctricas (utilización).
    /// Identificadores de los perfiles disponibles.
    pub const AVAILABLE: [&'static str; 1] = ["nom-001-sede-2012"];

    /// Perfil por su identificador, el que se guarda en el proyecto.
    pub fn by_id(id: &str) -> Option<&'static Profile> {
        match id {
            "nom-001-sede-2012" => Some(Profile::nom_001_sede_2012()),
            _ => None,
        }
    }

    pub fn nom_001_sede_2012() -> &'static Profile {
        static PROFILE: OnceLock<Profile> = OnceLock::new();
        PROFILE.get_or_init(|| {
            let sizes = table!(
                "nom-001-sede-2012",
                "table_8_conductor_properties.csv",
                |r| {
                    ConductorProperties {
                        size: r.text("size"),
                        area_mm2: r.get("area_mm2"),
                        kcmil: r.opt("kcmil"),
                        strands: r.get("strands"),
                        bare_diameter_mm: r.get("bare_diameter_mm"),
                        bare_area_mm2: r.get("bare_area_mm2"),
                    }
                }
            );
            let impedance = table!("nom-001-sede-2012", "table_9_impedance.csv", |r| {
                ImpedanceRow {
                    size: r.text("size"),
                    x_pvc_or_aluminum: r.get("x_pvc_or_aluminum"),
                    x_steel: r.get("x_steel"),
                    r_cu_pvc: r.opt("r_cu_pvc"),
                    r_cu_aluminum: r.opt("r_cu_aluminum"),
                    r_cu_steel: r.opt("r_cu_steel"),
                    r_al_pvc: r.opt("r_al_pvc"),
                    r_al_aluminum: r.opt("r_al_aluminum"),
                    r_al_steel: r.opt("r_al_steel"),
                }
            });
            let insulated = table!(
                "nom-001-sede-2012",
                "table_5_insulated_conductors.csv",
                |r| {
                    InsulatedConductor {
                        insulation_group: r.text("insulation_group"),
                        size: r.text("size"),
                        diameter_mm: r.get("diameter_mm"),
                        area_mm2: r.get("area_mm2"),
                    }
                }
            );
            let fields: HashMap<String, String> = table!("nom-001-sede-2012", "profile.csv", |r| (
                r.text("key"),
                r.text("value")
            ))
            .into_iter()
            .collect();
            let field = |key: &str| fields.get(key).cloned().unwrap_or_default();
            let number = |key: &str| -> f64 {
                field(key)
                    .parse()
                    .unwrap_or_else(|_| panic!("profile.csv: «{key}» no es un número"))
            };
            let parameters = Parameters {
                continuous_load_factor: number("continuous_load_factor"),
                terminal_60c_limit_a: number("terminal_60c_limit_a"),
                next_size_up_limit_a: number("next_size_up_limit_a"),
                smallest_circuit_size: Size::new(&field("smallest_circuit_size")),
                smallest_parallel_size: Size::new(&field("smallest_parallel_size")),
                jam_ratio_min: number("jam_ratio_min"),
                jam_ratio_max: number("jam_ratio_max"),
                nipple_fill_fraction: number("nipple_fill_fraction"),
                motor_conductor_factor: number("motor_conductor_factor"),
                motor_largest_factor: number("motor_largest_factor"),
                motor_overload_factor_marked: number("motor_overload_factor_marked"),
                motor_overload_factor_other: number("motor_overload_factor_other"),
                motor_starting_limit_a: number("motor_starting_limit_a"),
                motor_starting_factor_small: number("motor_starting_factor_small"),
                motor_starting_factor_large: number("motor_starting_factor_large"),
                compressor_conductor_factor: number("compressor_conductor_factor"),
                compressor_protection_factor: number("compressor_protection_factor"),
                compressor_protection_max_factor: number("compressor_protection_max_factor"),
                compressor_minimum_protection_a: number("compressor_minimum_protection_a"),
                receptacle_outlet_va: number("receptacle_outlet_va"),
                appliance_demand_factor: number("appliance_demand_factor"),
                appliance_demand_min_count: number("appliance_demand_min_count") as u32,
            };
            let motor_row = |columns: &'static [&'static str]| {
                move |r: &csv::Row| MotorCurrentRow {
                    hp: r.text("hp"),
                    kw: r.get("kw"),
                    currents: columns.iter().map(|c| (c.to_string(), r.opt(c))).collect(),
                }
            };
            let document = NormativeDocument {
                code: field("code"),
                title: field("title"),
                edition: field("edition"),
                source_url: fields.get("source_url").cloned(),
            };
            let references = table!("nom-001-sede-2012", "references.csv", |r| (
                r.text("rule"),
                ClauseEntry {
                    clause: r.text("clause"),
                    description: r.text("description"),
                }
            ))
            .into_iter()
            .collect();
            Profile {
                parameters,
                document,
                references,
                id: "nom-001-sede-2012",
                title: "NOM-001-SEDE-2012, Instalaciones Eléctricas (utilización)",
                size_index: sizes
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (c.size.clone(), i))
                    .collect(),
                sizes,
                impedance: impedance.into_iter().map(|r| (r.size.clone(), r)).collect(),
                insulated: insulated
                    .into_iter()
                    .map(|r| ((r.insulation_group.clone(), r.size.clone()), r))
                    .collect(),
                conduits: table!("nom-001-sede-2012", "table_4_conduit.csv", |r| ConduitRow {
                    conduit_type: r.text("type"),
                    metric_designator: r.get("metric_designator"),
                    trade_size: r.text("trade_size"),
                    internal_diameter_mm: r.get("internal_diameter_mm"),
                    area_100_mm2: r.get("area_100_mm2"),
                    area_60_mm2: r.get("area_60_mm2"),
                    area_53_mm2: r.get("area_53_mm2"),
                    area_31_mm2: r.get("area_31_mm2"),
                    area_40_mm2: r.get("area_40_mm2"),
                }),
                ampacity: table!("nom-001-sede-2012", "table_310_15_b_16_ampacity.csv", |r| {
                    AmpacityRow {
                        size: r.text("size"),
                        cu_60c: r.opt("cu_60c"),
                        cu_75c: r.opt("cu_75c"),
                        cu_90c: r.opt("cu_90c"),
                        al_60c: r.opt("al_60c"),
                        al_75c: r.opt("al_75c"),
                        al_90c: r.opt("al_90c"),
                    }
                }),
                temperature_correction: table!(
                    "nom-001-sede-2012",
                    "table_310_15_b_2_a_temperature_correction.csv",
                    |r| TemperatureCorrectionRow {
                        ambient_min_c: r.opt("ambient_min_c"),
                        ambient_max_c: r.get("ambient_max_c"),
                        factor_60c: r.opt("factor_60c"),
                        factor_75c: r.opt("factor_75c"),
                        factor_90c: r.opt("factor_90c"),
                    }
                ),
                bundling_adjustment: table!(
                    "nom-001-sede-2012",
                    "table_310_15_b_3_a_bundling_adjustment.csv",
                    |r| BundlingAdjustmentRow {
                        conductors_min: r.get("conductors_min"),
                        conductors_max: r.opt("conductors_max"),
                        factor: r.get("factor"),
                    }
                ),
                fill: table!("nom-001-sede-2012", "table_1_conduit_fill.csv", |r| {
                    FillRow {
                        conductors_min: r.get("conductors_min"),
                        conductors_max: r.opt("conductors_max"),
                        fill_fraction: r.get("fill_fraction"),
                    }
                }),
                grounding: table!(
                    "nom-001-sede-2012",
                    "table_250_122_equipment_grounding.csv",
                    |r| GroundingRow {
                        max_overcurrent_a: r.get("max_overcurrent_a"),
                        copper_size: r.text("copper_size"),
                        aluminum_size: r.opt_text("aluminum_size"),
                    }
                ),
                standard_ratings: table!(
                    "nom-001-sede-2012",
                    "section_240_6_standard_ratings.csv",
                    |r| StandardRating {
                        rating_a: r.get("rating_a"),
                        common: r.text("series") == "common",
                    }
                ),
                single_phase_motors: table!(
                    "nom-001-sede-2012",
                    "table_430_248_single_phase_motors.csv",
                    motor_row(&["v115", "v127", "v208", "v230"])
                ),
                three_phase_motors: table!(
                    "nom-001-sede-2012",
                    "table_430_250_three_phase_motors.csv",
                    motor_row(&["v115", "v200", "v208", "v230", "v460", "v575"])
                ),
                motor_voltage_columns: table!(
                    "nom-001-sede-2012",
                    "motor_voltage_columns.csv",
                    |r| MotorVoltageColumn {
                        table: r.text("table"),
                        column: r.text("column"),
                        system_min_v: r.get("system_min_v"),
                        system_max_v: r.get("system_max_v"),
                    }
                ),
                motor_protection: table!(
                    "nom-001-sede-2012",
                    "table_430_52_motor_protection.csv",
                    |r| MotorProtectionRow {
                        motor_type: r.text("motor_type"),
                        nontime_fuse: r.get("nontime_fuse"),
                        dual_element_fuse: r.get("dual_element_fuse"),
                        instantaneous_breaker: r.get("instantaneous_breaker"),
                        inverse_time_breaker: r.get("inverse_time_breaker"),
                    }
                ),
                occupancies: table!("nom-001-sede-2012", "table_220_12_lighting_load.csv", |r| {
                    Occupancy {
                        id: r.text("occupancy"),
                        label: r.text("label"),
                        va_per_m2: r.get("va_per_m2"),
                        dwelling: r.get("dwelling"),
                        area_only: r.get("area_only"),
                    }
                }),
                lighting_demand: table!(
                    "nom-001-sede-2012",
                    "table_220_42_lighting_demand.csv",
                    |r| DemandTier {
                        occupancy: r.text("occupancy"),
                        up_to_va: r.opt("up_to_va"),
                        percent: r.get("percent"),
                    }
                ),
                receptacle_demand: table!(
                    "nom-001-sede-2012",
                    "table_220_44_receptacle_demand.csv",
                    |r| DemandTier {
                        occupancy: String::new(),
                        up_to_va: r.opt("up_to_va"),
                        percent: r.get("percent"),
                    }
                ),
                office_receptacles: table!(
                    "nom-001-sede-2012",
                    "section_220_14_k_office_receptacles.csv",
                    |r| (r.text("occupancy"), r.get("va_per_m2"))
                ),
                kitchen_demand: table!(
                    "nom-001-sede-2012",
                    "table_220_56_kitchen_demand.csv",
                    |r| KitchenDemandRow {
                        units_min: r.get("units_min"),
                        units_max: r.opt("units_max"),
                        percent: r.get("percent"),
                    }
                ),
                small_conductor_limits: table!(
                    "nom-001-sede-2012",
                    "section_240_4_d_small_conductors.csv",
                    |r| SmallConductorLimit {
                        material: match r.text("material").as_str() {
                            "copper" => Material::Copper,
                            "aluminum" => Material::Aluminum,
                            other => panic!("material desconocido: {other}"),
                        },
                        size: r.text("size"),
                        max_overcurrent_a: r.get("max_overcurrent_a"),
                    }
                ),
            }
        })
    }

    /// Referencia normativa de una regla en este perfil.
    pub fn reference(&self, rule: Rule) -> RuleReference {
        RuleReference {
            code: self.document.code.clone(),
            title: self.document.title.clone(),
            edition: self.document.edition.clone(),
            clause: self.references.get(rule.id()).map(|e| e.clause.clone()),
            source_url: self.document.source_url.clone(),
        }
    }

    /// Cláusula de una regla, para mensajes: `Tabla 310-15(b)(16)`.
    pub fn clause(&self, rule: Rule) -> String {
        self.references
            .get(rule.id())
            .map_or_else(|| rule.id().to_string(), |e| e.clause.clone())
    }

    /// Descripción propia de lo que establece una regla.
    pub fn rule_description(&self, rule: Rule) -> Option<&str> {
        self.references
            .get(rule.id())
            .map(|e| e.description.as_str())
    }

    /// Propiedades del conductor desnudo (Tabla 8).
    pub fn conductor(&self, size: &Size) -> Result<&ConductorProperties> {
        self.size_index
            .get(&size.0)
            .map(|&i| &self.sizes[i])
            .ok_or_else(|| Error::UnknownSize(size.0.clone()))
    }

    /// Posición del calibre en el orden creciente de la Tabla 8.
    pub fn size_rank(&self, size: &Size) -> Result<usize> {
        self.size_index
            .get(&size.0)
            .copied()
            .ok_or_else(|| Error::UnknownSize(size.0.clone()))
    }

    /// Resistencia y reactancia de la Tabla 9, en ohms por kilómetro a 75 °C.
    pub fn impedance(
        &self,
        size: &Size,
        material: Material,
        raceway: RacewayMaterial,
    ) -> Result<(f64, f64)> {
        self.conductor(size)?;
        let missing = || Error::NotInTable {
            table: self.clause(Rule::Impedance),
            what: format!("calibre {size}, {material:?}, canalización {raceway:?}"),
        };
        let row = self.impedance.get(&size.0).ok_or_else(missing)?;
        let x = match raceway {
            RacewayMaterial::Steel => row.x_steel,
            RacewayMaterial::Pvc | RacewayMaterial::Aluminum => row.x_pvc_or_aluminum,
        };
        let r = match (material, raceway) {
            (Material::Copper, RacewayMaterial::Pvc) => row.r_cu_pvc,
            (Material::Copper, RacewayMaterial::Aluminum) => row.r_cu_aluminum,
            (Material::Copper, RacewayMaterial::Steel) => row.r_cu_steel,
            (Material::Aluminum, RacewayMaterial::Pvc) => row.r_al_pvc,
            (Material::Aluminum, RacewayMaterial::Aluminum) => row.r_al_aluminum,
            (Material::Aluminum, RacewayMaterial::Steel) => row.r_al_steel,
        };
        Ok((r.ok_or_else(missing)?, x))
    }

    /// Diámetro y área con aislamiento (Tabla 5).
    pub fn insulated(&self, group: InsulationGroup, size: &Size) -> Result<&InsulatedConductor> {
        self.conductor(size)?;
        self.insulated
            .get(&(group.code().to_string(), size.0.clone()))
            .ok_or_else(|| Error::NotInTable {
                table: self.clause(Rule::InsulatedDimensions),
                what: format!("calibre {size}, aislamiento {group:?}"),
            })
    }

    /// Ampacidad de la Tabla 310-15(b)(16), en amperes.
    pub fn ampacity(&self, size: &Size, material: Material, column: TempRating) -> Result<f64> {
        self.conductor(size)?;
        let missing = || Error::NotInTable {
            table: self.clause(Rule::Ampacity),
            what: format!("calibre {size}, {material:?}, {} °C", column.celsius()),
        };
        let row = self
            .ampacity
            .iter()
            .find(|r| r.size == size.0)
            .ok_or_else(missing)?;
        match (material, column) {
            (Material::Copper, TempRating::C60) => row.cu_60c,
            (Material::Copper, TempRating::C75) => row.cu_75c,
            (Material::Copper, TempRating::C90) => row.cu_90c,
            (Material::Aluminum, TempRating::C60) => row.al_60c,
            (Material::Aluminum, TempRating::C75) => row.al_75c,
            (Material::Aluminum, TempRating::C90) => row.al_90c,
        }
        .ok_or_else(missing)
    }

    /// Factor de corrección por temperatura ambiente, Tabla 310-15(b)(2)(a).
    pub fn temperature_factor(&self, ambient_c: f64, column: TempRating) -> Result<f64> {
        let missing = || Error::NotInTable {
            table: self.clause(Rule::TemperatureCorrection),
            what: format!(
                "{ambient_c} °C ambiente con conductor de {} °C",
                column.celsius()
            ),
        };
        // Un valor entre dos intervalos toma el de mayor temperatura.
        let row = self
            .temperature_correction
            .iter()
            .find(|r| ambient_c <= r.ambient_max_c)
            .ok_or_else(missing)?;
        match column {
            TempRating::C60 => row.factor_60c,
            TempRating::C75 => row.factor_75c,
            TempRating::C90 => row.factor_90c,
        }
        .ok_or_else(missing)
    }

    /// Factor de ajuste por número de conductores portadores de corriente,
    /// Tabla 310-15(b)(3)(a). Hasta tres conductores no hay ajuste.
    pub fn bundling_factor(&self, current_carrying: u32) -> f64 {
        self.bundling_adjustment
            .iter()
            .find(|r| {
                current_carrying >= r.conductors_min
                    && r.conductors_max.map_or(true, |max| current_carrying <= max)
            })
            .map_or(1.0, |r| r.factor)
    }

    /// Tipo de inmueble o de área de la Tabla 220-12.
    pub fn occupancy(&self, id: &str) -> Option<&Occupancy> {
        self.occupancies.iter().find(|o| o.id == id)
    }

    /// Escalones de la Tabla 220-42 de un tipo de inmueble. Vacío si va al
    /// 100 % («todos los demás»).
    pub fn lighting_demand_tiers(&self, occupancy: &str) -> Vec<&DemandTier> {
        self.lighting_demand
            .iter()
            .filter(|t| t.occupancy == occupancy)
            .collect()
    }

    /// Carga mínima de contactos por m² de 220-14(k), si el inmueble la tiene.
    pub fn office_receptacle_va_per_m2(&self, occupancy: &str) -> Option<f64> {
        self.office_receptacles
            .iter()
            .find(|(o, _)| o == occupancy)
            .map(|(_, v)| *v)
    }

    /// Factor de la Tabla 220-56 para un número de equipos de cocina.
    pub fn kitchen_demand_percent(&self, units: u32) -> Option<f64> {
        self.kitchen_demand
            .iter()
            .find(|r| units >= r.units_min && r.units_max.map_or(true, |max| units <= max))
            .map(|r| r.percent)
    }

    /// Filas de la Tabla 4 para un tipo de tubo, de menor a mayor.
    pub fn conduit_rows(&self, conduit: ConduitType) -> impl Iterator<Item = &ConduitRow> {
        self.conduits
            .iter()
            .filter(move |r| r.conduit_type == conduit.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pruebas normativas del perfil: cada regla tiene su cláusula y los
    /// parámetros tienen los valores de la NOM-001-SEDE-2012.
    #[test]
    fn every_rule_has_a_clause() {
        let profile = Profile::nom_001_sede_2012();
        for rule in Rule::ALL {
            let reference = profile.reference(rule);
            assert!(reference.clause.is_some(), "falta la cláusula de {rule:?}");
            assert_eq!(reference.code, "NOM-001-SEDE-2012");
        }
        assert_eq!(
            profile.reference(Rule::StandardRatings).cite(),
            "NOM-001-SEDE-2012, 240-6(a)"
        );
    }

    #[test]
    fn parameters_match_the_2012_edition() {
        let p = &Profile::nom_001_sede_2012().parameters;
        assert_eq!(p.continuous_load_factor, 1.25); // 210-19(a)(1)
        assert_eq!(p.terminal_60c_limit_a, 100.0); // 110-14(c)(1)
        assert_eq!(p.next_size_up_limit_a, 800.0); // 240-4(b)
        assert_eq!(p.smallest_parallel_size, Size::new("1/0")); // 310-10(h)
        assert_eq!(p.nipple_fill_fraction, 0.60); // Capítulo 10, notas, (4)
        assert_eq!((p.jam_ratio_min, p.jam_ratio_max), (2.8, 3.2)); // Tabla 1, nota 2
    }

    /// Pruebas normativas de las tablas de motores: valores conocidos de la
    /// NOM-001-SEDE-2012 en cada tabla y en sus extremos.
    #[test]
    fn motor_tables_match_the_2012_edition() {
        let profile = Profile::nom_001_sede_2012();
        let current = |rows: &[MotorCurrentRow], hp: &str, column: &str| {
            rows.iter()
                .find(|r| r.hp == hp)
                .and_then(|r| r.currents.iter().find(|(c, _)| c == column))
                .and_then(|(_, a)| *a)
        };
        let one = &profile.single_phase_motors;
        let three = &profile.three_phase_motors;
        assert_eq!(current(one, "1/6", "v115"), Some(4.4)); // Tabla 430-248
        assert_eq!(current(one, "1", "v127"), Some(14.0));
        assert_eq!(current(one, "10", "v230"), Some(50.0));
        assert_eq!(current(three, "1/2", "v230"), Some(2.2)); // Tabla 430-250
        assert_eq!(current(three, "10", "v575"), Some(11.0)); // errata del DOF: 44
        assert_eq!(current(three, "50", "v460"), Some(65.0));
        assert_eq!(current(three, "500", "v460"), Some(590.0));
        assert_eq!(current(three, "250", "v230"), None);
        assert_eq!(current(three, "3", "v115"), None);
        let inverse = |t: &str| {
            profile
                .motor_protection
                .iter()
                .find(|r| r.motor_type == t)
                .map(|r| r.inverse_time_breaker)
        };
        assert_eq!(inverse("squirrel_cage"), Some(250.0)); // Tabla 430-52
        assert_eq!(inverse("wound_rotor"), Some(150.0));
        let p = &profile.parameters;
        assert_eq!(p.motor_conductor_factor, 1.25); // 430-22
        assert_eq!(p.motor_largest_factor, 1.25); // 430-24(1)
        assert_eq!(
            (
                p.motor_overload_factor_marked,
                p.motor_overload_factor_other
            ),
            (1.25, 1.15)
        ); // 430-32(a)(1)
        assert_eq!(
            (
                p.compressor_protection_factor,
                p.compressor_protection_max_factor
            ),
            (1.75, 2.25)
        ); // 440-22(a)
        assert_eq!(p.compressor_minimum_protection_a, 15.0);
    }

    /// Pruebas normativas del artículo 220: valores de las tablas 220-12,
    /// 220-42, 220-44 y 220-56 y de las secciones 220-14 y 220-53.
    #[test]
    fn article_220_matches_the_2012_edition() {
        let profile = Profile::nom_001_sede_2012();
        let unit = |id: &str| profile.occupancy(id).map(|o| o.va_per_m2);
        assert_eq!(unit("dwellings"), Some(33.0)); // Tabla 220-12
        assert_eq!(unit("offices"), Some(39.0));
        assert_eq!(unit("banks"), Some(39.0));
        assert_eq!(unit("storage_warehouses"), Some(3.0));
        assert_eq!(unit("parking_garages"), Some(6.0));
        assert_eq!(unit("boarding_houses"), Some(17.0));
        assert_eq!(unit("halls_corridors"), Some(6.0));
        assert_eq!(profile.occupancies.len(), 21);
        assert!(profile.occupancy("dwellings").is_some_and(|o| o.dwelling));
        assert_eq!(profile.occupancies.iter().filter(|o| o.dwelling).count(), 1);
        let tiers = |id: &str| -> Vec<(Option<f64>, f64)> {
            profile
                .lighting_demand_tiers(id)
                .iter()
                .map(|t| (t.up_to_va, t.percent))
                .collect()
        };
        // Tabla 220-42
        assert_eq!(
            tiers("dwellings"),
            vec![(Some(3000.0), 100.0), (Some(120000.0), 35.0), (None, 25.0)]
        );
        assert_eq!(
            tiers("hotels"),
            vec![(Some(20000.0), 50.0), (Some(100000.0), 40.0), (None, 30.0)]
        );
        assert_eq!(
            tiers("hospitals"),
            vec![(Some(50000.0), 40.0), (None, 20.0)]
        );
        assert_eq!(
            tiers("storage_warehouses"),
            vec![(Some(12500.0), 100.0), (None, 50.0)]
        );
        assert!(tiers("offices").is_empty());
        // Tabla 220-44
        let r: Vec<_> = profile
            .receptacle_demand
            .iter()
            .map(|t| (t.up_to_va, t.percent))
            .collect();
        assert_eq!(r, vec![(Some(10000.0), 100.0), (None, 50.0)]);
        // Tabla 220-56
        assert_eq!(profile.kitchen_demand_percent(2), Some(100.0));
        assert_eq!(profile.kitchen_demand_percent(3), Some(90.0));
        assert_eq!(profile.kitchen_demand_percent(5), Some(70.0));
        assert_eq!(profile.kitchen_demand_percent(12), Some(65.0));
        // 220-14(i), 220-14(k) y 220-53
        assert_eq!(profile.parameters.receptacle_outlet_va, 180.0);
        assert_eq!(profile.office_receptacle_va_per_m2("offices"), Some(11.0));
        assert_eq!(profile.office_receptacle_va_per_m2("stores"), None);
        assert_eq!(profile.parameters.appliance_demand_factor, 0.75);
        assert_eq!(profile.parameters.appliance_demand_min_count, 4);
    }

    #[test]
    fn profiles_are_found_by_id() {
        for id in Profile::AVAILABLE {
            assert_eq!(Profile::by_id(id).map(|p| p.id), Some(id));
        }
        assert!(Profile::by_id("nom-001-sede-1999").is_none());
    }
}
