//! Modelo de una instalación: fuente, tableros, circuitos y cargas.
//!
//! El proyecto se guarda como JSON legible. Todo lo que el usuario captura
//! vive aquí; los resultados se calculan a partir de este modelo y no se
//! guardan.

use ieestudio_core::Insulation;
use ieestudio_core::{CircuitKind, ConduitType, Material, MotorType, Size, System};
use serde::{Deserialize, Serialize};

/// Versión del formato de archivo. La 2 agrega los datos de motores y
/// motocompresores y el interruptor fijado. La 3 agrega el tipo de inmueble,
/// las áreas de cada tablero, los contactos, el método de demanda del
/// alimentador, los datos de la obra y los transformadores. Lee los archivos
/// de la 1 y la 2.
///
/// Los archivos de la versión 3 escritos antes de retirar el cortocircuito
/// (decisión 0018) traen campos de falla y de capacidad interruptiva; al
/// leerlos se ignoran y ya no se escriben. Por eso el formato sigue siendo el 3:
/// un programa anterior abre sin error lo que escribe este.
pub const FORMAT_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub format_version: u32,
    pub name: String,
    /// Identificador del perfil normativo, por ejemplo `nom-001-sede-2012`.
    pub profile: String,
    /// Datos de la obra para la memoria.
    #[serde(default)]
    pub info: ProjectInfo,
    /// Tipo de inmueble de la Tabla 220-12 (`dwellings`, `offices`…). Decide
    /// los factores de demanda del artículo 220. `None` si no se ha elegido.
    #[serde(default)]
    pub occupancy: Option<String>,
    #[serde(default)]
    pub settings: Settings,
    pub source: Source,
    pub panels: Vec<Panel>,
}

/// Datos de la obra que aparecen en la memoria. Ninguno interviene en el
/// cálculo.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectInfo {
    /// Cliente o propietario.
    pub client: String,
    /// Ubicación de la obra.
    pub location: String,
    /// Quién elaboró el cálculo.
    pub prepared_by: String,
    /// Responsable que revisa y firma el proyecto.
    pub responsible: String,
    /// Cédula profesional del responsable.
    pub license: String,
    /// Revisión del documento: `A`, `0`, `1`…
    pub revision: String,
    /// Fecha de la revisión, como se quiere ver en la memoria. Vacía usa la
    /// fecha en que se genera.
    pub date: String,
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
    /// Calibre mayor que el programa propone para un solo conductor por fase;
    /// arriba de él propone conductores en paralelo.
    pub largest_single_size: Size,
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
            largest_single_size: Size::new("500"),
        }
    }
}

/// Fuente de suministro: acometida o secundario de un transformador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub system: System,
    /// Compañía suministradora, para el unifilar: `CFE`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub supplier: String,
    /// Frecuencia del sistema en hertz, para el unifilar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_hz: Option<f64>,
    /// Lleva medidor en el punto de conexión.
    #[serde(default = "yes")]
    pub meter: bool,
    /// Transformador que alimenta la instalación: la subestación propia o el
    /// transformador de la suministradora. Con él se estima la corriente de
    /// falla por bus infinito.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substation: Option<Substation>,
    /// Corriente de falla que da la suministradora en el punto de conexión.
    /// Si se tiene, manda sobre la que se estima con la subestación.
    #[serde(default, skip_serializing_if = "SourceFault::is_empty")]
    pub fault: SourceFault,
}

impl Source {
    pub fn new(name: &str, system: System) -> Self {
        Source {
            name: name.to_string(),
            system,
            supplier: String::new(),
            frequency_hz: None,
            meter: true,
            substation: None,
            fault: SourceFault::default(),
        }
    }
}

/// Dato de falla de la suministradora. Los archivos de la decisión 0013 ya
/// lo traían con este mismo nombre.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceFault {
    /// Corriente de cortocircuito disponible, en kA.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_ka: Option<f64>,
}

impl SourceFault {
    pub fn is_empty(&self) -> bool {
        self.available_ka.is_none()
    }
}

/// Transformador que alimenta la instalación desde media tensión
/// (requisito I.2 del unifilar).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Substation {
    pub kva: f64,
    /// Impedancia de placa, en porciento.
    pub impedance_percent: f64,
    /// Sin dato de placa: la falla se calcula con la impedancia mínima de la
    /// norma de fabricación para su tipo, potencia y tensión.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub impedance_from_table: bool,
    /// Tensión del primario, en kV.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_kv: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection: Option<Connection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mounting: Option<Mounting>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub brand: String,
}

/// Conexión de los devanados de un transformador.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Connection {
    DeltaWye,
    WyeWye,
    DeltaDelta,
    WyeDelta,
    SinglePhase,
}

impl Connection {
    pub const ALL: [Connection; 5] = [
        Connection::DeltaWye,
        Connection::WyeWye,
        Connection::DeltaDelta,
        Connection::WyeDelta,
        Connection::SinglePhase,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Connection::DeltaWye => "Delta – estrella",
            Connection::WyeWye => "Estrella – estrella",
            Connection::DeltaDelta => "Delta – delta",
            Connection::WyeDelta => "Estrella – delta",
            Connection::SinglePhase => "Monofásico",
        }
    }
}

/// Montaje de un transformador.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mounting {
    Pole,
    PadMounted,
    Substation,
    Dry,
}

impl Mounting {
    pub const ALL: [Mounting; 4] = [
        Mounting::Pole,
        Mounting::PadMounted,
        Mounting::Substation,
        Mounting::Dry,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Mounting::Pole => "Tipo poste",
            Mounting::PadMounted => "Tipo pedestal",
            Mounting::Substation => "Tipo subestación",
            Mounting::Dry => "Tipo seco",
        }
    }
}

/// Datos de catálogo de un interruptor, para el unifilar (requisito I.4) y
/// para comparar su capacidad interruptiva con la corriente de falla. No
/// cambian el cálculo de conductores ni de protección.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BreakerData {
    /// Amperes de marco.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_a: Option<f64>,
    /// Capacidad interruptiva, en kA.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupting_ka: Option<f64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub model: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub brand: String,
    /// Intervalo de ajuste de un interruptor electrónico: `0.4–1.0 In`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub trip_range: String,
}

impl BreakerData {
    pub fn is_empty(&self) -> bool {
        *self == BreakerData::default()
    }
}

/// Datos de catálogo de un tablero: los de su gabinete y sus barras.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Board {
    /// Capacidad de las barras, en amperes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bus_a: Option<f64>,
    /// Capacidad interruptiva de los interruptores del tablero, en kA. La
    /// toma cada interruptor que no tiene la suya.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupting_ka: Option<f64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub model: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub brand: String,
}

impl Board {
    pub fn is_empty(&self) -> bool {
        *self == Board::default()
    }
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
    /// Áreas que atiende este tablero con sus propios circuitos, para la
    /// carga mínima de alumbrado (220-12). Las de sus subtableros se suman.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub areas: Vec<Area>,
    /// Transformador entre el alimentador y el tablero. El alimentador es su
    /// primario; el tablero y sus circuitos quedan con la tensión del
    /// secundario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transformer: Option<Transformer>,
    /// Gabinete y barras: marca, modelo, capacidad y la capacidad
    /// interruptiva de sus interruptores.
    #[serde(default, skip_serializing_if = "Board::is_empty")]
    pub board: Board,
    /// Capacidad interruptiva del tablero en los archivos de la decisión
    /// 0013. Al leerlos pasa a `board`; ya no se escribe.
    #[serde(default, skip_serializing)]
    pub interrupting_ka: Option<f64>,
    #[serde(default)]
    pub circuits: Vec<Circuit>,
}

/// Transformador de 600 V o menos que alimenta un tablero (artículo 450).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transformer {
    pub kva: f64,
    /// Impedancia de placa, en porciento.
    pub impedance_percent: f64,
    /// Sistema del secundario.
    pub secondary: System,
    /// Lleva protección en el secundario (el interruptor principal del
    /// tablero). Con ella el primario puede protegerse hasta el 250 %.
    #[serde(default = "yes")]
    pub secondary_protection: bool,
    /// Longitud de los conductores del secundario, del transformador al
    /// tablero, en metros.
    #[serde(default)]
    pub secondary_length_m: f64,
    #[serde(default = "yes")]
    pub secondary_has_neutral: bool,
    #[serde(default)]
    pub secondary_overrides: Overrides,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection: Option<Connection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mounting: Option<Mounting>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub brand: String,
}

/// Superficie de piso que atiende un tablero.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Area {
    #[serde(default)]
    pub description: String,
    /// Tipo de área de la Tabla 220-12. `None` toma el tipo de inmueble del
    /// proyecto.
    #[serde(default)]
    pub occupancy: Option<String>,
    pub m2: f64,
}

/// Cómo se obtiene la carga del alimentador a partir de la conectada.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Demand {
    /// Factores de demanda del artículo 220 según el tipo de inmueble.
    #[default]
    Article220,
    /// Toda la carga al 100 %, sin factores.
    Full,
    /// Factor propio del usuario sobre toda la carga. La norma no lo respalda
    /// por sí mismo: queda «no evaluado» si reduce la carga.
    Custom(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feeder {
    pub length_m: f64,
    #[serde(default)]
    pub demand: Demand,
    /// Factor de demanda de los archivos de formato 1 y 2. Al leerlos se
    /// convierte en `demand`; ya no se escribe.
    #[serde(default, skip_serializing)]
    pub demand_factor: Option<f64>,
    /// El alimentador lleva neutro.
    #[serde(default = "yes")]
    pub has_neutral: bool,
    #[serde(default)]
    pub overrides: Overrides,
    /// Capacidad interruptiva del interruptor en los archivos de la decisión
    /// 0013. Al leerlos pasa a `overrides.breaker`; ya no se escribe.
    #[serde(default, skip_serializing)]
    pub interrupting_ka: Option<f64>,
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
    /// Número de salidas de contacto, si `kind` es `Receptacles` (220-14(i)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outlets: Option<u32>,
    /// Datos del motor, si `kind` es `Motor`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motor: Option<MotorLoad>,
    /// Datos de placa del equipo, si `kind` es `AirConditioning`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compressor: Option<CompressorLoad>,
}

/// Motor de un circuito. Su corriente sale de las tablas por potencia y
/// tensión, no de la placa (430-6(a)(1)).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotorLoad {
    /// Potencia en caballos de fuerza, como en la tabla: `7-1/2`.
    pub hp: String,
    #[serde(default)]
    pub motor_type: MotorType,
    /// Corriente de la placa, para ajustar el relevador de sobrecarga.
    #[serde(default)]
    pub nameplate_a: Option<f64>,
    /// Factor de servicio de 1.15 o más, o elevación de temperatura de 40 °C
    /// o menos: el relevador puede ajustarse al 125 % en lugar del 115 %.
    #[serde(default)]
    pub service_factor_marked: bool,
}

/// Datos de placa de un motocompresor o de su equipo (artículo 440). Basta
/// la corriente de carga nominal, o la ampacidad mínima y la protección
/// máxima que marca el equipo.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressorLoad {
    /// Corriente de carga nominal (RLA).
    pub rated_load_a: Option<f64>,
    /// Corriente de selección del circuito derivado (BCSC).
    pub branch_selection_a: Option<f64>,
    /// Ampacidad mínima del circuito (MCA).
    pub minimum_circuit_a: Option<f64>,
    /// Protección máxima (MOP).
    pub maximum_protection_a: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoadKind {
    /// Otras cargas: van al 100 % en el alimentador.
    #[default]
    General,
    /// Alumbrado general (220-12, 220-42).
    Lighting,
    /// Contactos de uso general (220-14(i), 220-44).
    Receptacles,
    /// Aparato fijo que no es estufa, secadora, calefacción ni aire
    /// acondicionado: calentador de agua, lavavajillas… (220-53).
    Appliance,
    /// Equipo de cocina comercial (220-56).
    KitchenEquipment,
    Motor,
    AirConditioning,
}

impl LoadKind {
    /// Nombre para mostrar.
    pub fn label(self) -> &'static str {
        match self {
            LoadKind::General => "General",
            LoadKind::Lighting => "Alumbrado",
            LoadKind::Receptacles => "Contactos",
            LoadKind::Appliance => "Aparato fijo",
            LoadKind::KitchenEquipment => "Equipo de cocina",
            LoadKind::Motor => "Motor",
            LoadKind::AirConditioning => "Aire acondicionado",
        }
    }
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
    /// Interruptor fijado. El programa lo verifica, no lo cambia.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breaker_a: Option<f64>,
    /// Con conductores en paralelo, todos los juegos van en un solo tubo. Por
    /// omisión cada juego lleva su tubo.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub single_raceway: bool,
    /// Datos de catálogo del interruptor: marco, capacidad interruptiva,
    /// modelo, marca y ajuste.
    #[serde(skip_serializing_if = "BreakerData::is_empty")]
    pub breaker: BreakerData,
    /// Posiciones del interruptor en el tablero, como se escriben en el
    /// unifilar: `1`, `2,4`, `13,15,17`. Sin ella el programa las propone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
}

fn yes() -> bool {
    true
}

impl Project {
    /// Lee un proyecto desde JSON.
    pub fn from_json(text: &str) -> Result<Project, String> {
        let mut project: Project =
            serde_json::from_str(text).map_err(|e| format!("archivo de proyecto inválido: {e}"))?;
        if project.format_version > FORMAT_VERSION {
            return Err(format!(
                "el archivo usa el formato {}, más nuevo que el que este programa entiende ({FORMAT_VERSION})",
                project.format_version
            ));
        }
        // Formatos 1 y 2: el factor de demanda era un número sobre toda la
        // carga, 1 por omisión. Uno distinto de 1 se conserva como factor
        // propio; el 1 pasa al método del artículo 220.
        for panel in &mut project.panels {
            if let Some(factor) = panel.feeder.demand_factor.take() {
                if (factor - 1.0).abs() > 1e-9 {
                    panel.feeder.demand = Demand::Custom(factor);
                }
            }
            // Capacidad interruptiva de la decisión 0013: vuelve a usarse.
            if let Some(ka) = panel.interrupting_ka.take() {
                panel.board.interrupting_ka.get_or_insert(ka);
            }
            if let Some(ka) = panel.feeder.interrupting_ka.take() {
                panel
                    .feeder
                    .overrides
                    .breaker
                    .interrupting_ka
                    .get_or_insert(ka);
            }
        }
        Ok(project)
    }

    /// Escribe el proyecto como JSON legible, siempre en el formato actual.
    pub fn to_json(&self) -> String {
        let current = Project {
            format_version: FORMAT_VERSION,
            ..self.clone()
        };
        serde_json::to_string_pretty(&current).expect("el modelo siempre se puede serializar")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_demand_factors_are_migrated() {
        let mut project = crate::example::house();
        project.panels[0].feeder.demand = Demand::Article220;
        let json = project
            .to_json()
            .replace("\"demand\": \"Article220\"", "\"demand_factor\": 0.8");
        assert!(json.contains("demand_factor"));
        let read = Project::from_json(&json).unwrap();
        assert_eq!(read.panels[0].feeder.demand, Demand::Custom(0.8));
        let json = json.replace("0.8", "1.0");
        let read = Project::from_json(&json).unwrap();
        assert_eq!(read.panels[0].feeder.demand, Demand::Article220);
        assert!(!read.to_json().contains("demand_factor"));
    }

    /// Un archivo escrito con el cortocircuito de la decisión 0013 abre sin
    /// error. La corriente de falla de la suministradora y la capacidad
    /// interruptiva vuelven a usarse (decisión 0023); el factor de motores
    /// del proyecto se ignora porque ahora es del perfil.
    #[test]
    fn old_fault_data_is_read_again() {
        let project = crate::example::house();
        let json = project
            .to_json()
            .replacen(
                "\"source\": {",
                "\"source\": {\n    \"fault\": { \"available_ka\": 10.0 },",
                1,
            )
            .replacen(
                "\"imbalance_warning_percent\"",
                "\"motor_fault_multiplier\": 4.0,\n    \"imbalance_warning_percent\"",
                1,
            )
            .replacen(
                "\"has_neutral\": true,",
                "\"has_neutral\": true,\n      \"interrupting_ka\": 10.0,",
                1,
            );
        assert!(json.contains("available_ka"));
        assert!(json.contains("motor_fault_multiplier"));
        assert!(json.contains("interrupting_ka"));
        let read = Project::from_json(&json).unwrap();
        assert_eq!(read.source.fault.available_ka, Some(10.0));
        assert_eq!(
            read.panels[0].feeder.overrides.breaker.interrupting_ka,
            Some(10.0)
        );
        assert_eq!(read.panels[0].feeder.interrupting_ka, None);
        let written = read.to_json();
        assert!(!written.contains("motor_fault_multiplier"));
        // Se escribe en su lugar nuevo y se vuelve a leer igual.
        assert_eq!(Project::from_json(&written).unwrap(), read);
        let mut again = read.clone();
        again.source.fault = SourceFault::default();
        again.panels[0].feeder.overrides.breaker = BreakerData::default();
        assert_eq!(again, project);
    }

    #[test]
    fn projects_round_trip() {
        let project = crate::example::house();
        assert_eq!(Project::from_json(&project.to_json()).unwrap(), project);
    }
}
