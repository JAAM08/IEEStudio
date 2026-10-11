//! Modelo de una instalación: fuente, tableros, circuitos y cargas.
//!
//! El proyecto se guarda como JSON legible. Todo lo que el usuario captura
//! vive aquí; los resultados se calculan a partir de este modelo y no se
//! guardan.

use serde::{Deserialize, Serialize};
use voltforge_core::sizing::Insulation;
use voltforge_core::{CircuitKind, ConduitType, Material, Size, System};

/// Versión del formato de archivo.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub format_version: u32,
    pub name: String,
    /// Identificador del perfil normativo, por ejemplo `nom-001-sede-2012`.
    pub profile: String,
    #[serde(default)]
    pub settings: Settings,
    pub source: Source,
    pub panels: Vec<Panel>,
}

/// Criterios de diseño que aplican a todo el proyecto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Límite de caída de tensión en cada circuito derivado, en porciento.
    pub branch_drop_limit_percent: f64,
    /// Límite de caída de tensión en cada alimentador, en porciento.
    pub feeder_drop_limit_percent: f64,
    /// Límite de la caída acumulada desde la fuente hasta la carga.
    pub total_drop_limit_percent: f64,
    /// Temperatura ambiente, en °C.
    pub ambient_c: f64,
    pub material: Material,
    pub insulation: Insulation,
    pub wet_location: bool,
    pub conduit: ConduitType,
    /// Conductor de puesta a tierra desnudo (`true`) o aislado.
    pub bare_grounding: bool,
    /// Temperatura para ajustar la resistencia en la caída de tensión.
    /// `None` usa la Tabla 9 tal cual (75 °C).
    pub resistance_temperature_c: Option<f64>,
    /// Admite interruptores de 16, 32 y 63 A.
    pub include_iec_ratings: bool,
    /// Desbalance entre fases a partir del cual se emite un aviso, en porciento.
    pub imbalance_warning_percent: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            branch_drop_limit_percent: 3.0,
            feeder_drop_limit_percent: 2.0,
            total_drop_limit_percent: 5.0,
            ambient_c: 30.0,
            material: Material::Copper,
            insulation: Insulation::Thw,
            wet_location: false,
            conduit: ConduitType::Emt,
            bare_grounding: true,
            resistance_temperature_c: None,
            include_iec_ratings: false,
            imbalance_warning_percent: 5.0,
        }
    }
}

/// Fuente de suministro: acometida o secundario de un transformador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub system: System,
}

/// Fase de un sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    A,
    B,
    C,
}

/// Tablero de distribución con su alimentador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    pub id: String,
    pub name: String,
    /// Tablero del que se alimenta. `None` si se alimenta de la fuente.
    #[serde(default)]
    pub parent: Option<String>,
    /// Conexión del alimentador: una, dos o tres fases.
    pub kind: CircuitKind,
    pub feeder: Feeder,
    #[serde(default)]
    pub circuits: Vec<Circuit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feeder {
    pub length_m: f64,
    /// Factor de demanda que se aplica a la carga conectada del tablero.
    #[serde(default = "one")]
    pub demand_factor: f64,
    /// El alimentador lleva neutro.
    #[serde(default = "yes")]
    pub has_neutral: bool,
    #[serde(default)]
    pub overrides: Overrides,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    pub id: String,
    pub name: String,
    pub kind: CircuitKind,
    /// Fases a las que se conecta, para el balanceo. Puede quedar vacío.
    #[serde(default)]
    pub phases: Vec<Phase>,
    pub load: Load,
    pub length_m: f64,
    /// Lleva neutro además de las fases. En un circuito de una fase siempre lo lleva.
    #[serde(default)]
    pub has_neutral: bool,
    #[serde(default)]
    pub overrides: Overrides,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Load {
    /// Potencia activa, en watts.
    pub power_w: f64,
    pub power_factor: f64,
    /// Carga continua: opera a su corriente máxima tres horas o más.
    #[serde(default)]
    pub continuous: bool,
    #[serde(default)]
    pub kind: LoadKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoadKind {
    #[default]
    General,
    Lighting,
    Motor,
    AirConditioning,
}

/// Decisiones del usuario que sustituyen lo que el programa propondría.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Overrides {
    /// Calibre forzado. El programa lo verifica, no lo cambia.
    pub size: Option<Size>,
    pub conductors_per_phase: Option<u32>,
    pub material: Option<Material>,
    pub insulation: Option<Insulation>,
    pub conduit: Option<ConduitType>,
    /// Tramo de 60 cm o menos entre cajas o gabinetes.
    pub nipple: bool,
    /// La mayor parte de la carga es no lineal: el neutro cuenta como
    /// conductor portador de corriente.
    pub nonlinear: bool,
    /// Otros conductores portadores de corriente en la misma canalización.
    pub other_current_carrying_conductors: u32,
}

fn one() -> f64 {
    1.0
}

fn yes() -> bool {
    true
}

impl Project {
    /// Lee un proyecto desde JSON.
    pub fn from_json(text: &str) -> Result<Project, String> {
        let project: Project =
            serde_json::from_str(text).map_err(|e| format!("archivo de proyecto inválido: {e}"))?;
        if project.format_version > FORMAT_VERSION {
            return Err(format!(
                "el archivo usa el formato {}, más nuevo que el que este programa entiende ({FORMAT_VERSION})",
                project.format_version
            ));
        }
        Ok(project)
    }

    /// Escribe el proyecto como JSON legible.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("el modelo siempre se puede serializar")
    }
}
