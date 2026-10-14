//! Tipos del dominio eléctrico que comparten el motor, las reglas y el proyecto.

/// Material del conductor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Material {
    Copper,
    Aluminum,
}

/// Material de la canalización, que determina la columna de la Tabla 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RacewayMaterial {
    Pvc,
    Aluminum,
    Steel,
}

/// Tipo de tubo conduit de la Tabla 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConduitType {
    Emt,
    Imc,
    Rmc,
    Pvc40,
    Pvc80,
}

impl ConduitType {
    pub const ALL: [ConduitType; 5] = [Self::Emt, Self::Imc, Self::Rmc, Self::Pvc40, Self::Pvc80];

    /// Clave con la que aparece en los archivos de datos.
    pub fn code(self) -> &'static str {
        match self {
            Self::Emt => "EMT",
            Self::Imc => "IMC",
            Self::Rmc => "RMC",
            Self::Pvc40 => "PVC40",
            Self::Pvc80 => "PVC80",
        }
    }

    /// Material de la canalización para la Tabla 9.
    pub fn raceway_material(self) -> RacewayMaterial {
        match self {
            Self::Emt | Self::Imc | Self::Rmc => RacewayMaterial::Steel,
            Self::Pvc40 | Self::Pvc80 => RacewayMaterial::Pvc,
        }
    }
}

impl std::fmt::Display for ConduitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

/// Grupo de aislamientos con las mismas dimensiones en la Tabla 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InsulationGroup {
    /// TW, THW, THHW, THW-2
    TwThwThhw,
    /// RHH, RHW, RHW-2 (con cubierta)
    RhhRhw,
    /// THHN, THWN, THWN-2
    ThhnThwn,
    /// XHHW, XHHW-2, XHH
    Xhhw,
}

impl InsulationGroup {
    /// Clave con la que aparece en los archivos de datos.
    pub fn code(self) -> &'static str {
        match self {
            Self::TwThwThhw => "TW_THW_THHW_THW2",
            Self::RhhRhw => "RHH_RHW_RHW2",
            Self::ThhnThwn => "THHN_THWN_THWN2",
            Self::Xhhw => "XHHW_XHHW2_XHH",
        }
    }
}

/// Calibre del conductor tal como lo nombra la norma: `"14"`, `"1/0"`, `"250"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Size(pub String);

impl Size {
    pub fn new(s: &str) -> Self {
        Size(s.trim().to_string())
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Columna de temperatura de la tabla de ampacidades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TempRating {
    C60,
    C75,
    C90,
}

impl TempRating {
    pub fn celsius(self) -> f64 {
        match self {
            TempRating::C60 => 60.0,
            TempRating::C75 => 75.0,
            TempRating::C90 => 90.0,
        }
    }
}

/// Tipo de aislamiento del conductor (Tabla 310-104(a)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Insulation {
    Tw,
    Thw,
    Thhw,
    Thw2,
    Thhn,
    Thwn,
    Thwn2,
    Xhhw,
    Xhhw2,
    Rhh,
    Rhw,
    Rhw2,
}

impl Insulation {
    /// Temperatura nominal en lugar seco o mojado. `None` si el aislamiento
    /// no se permite en esa condición.
    pub fn rating(self, wet_location: bool) -> Option<TempRating> {
        use Insulation::*;
        use TempRating::*;
        match (self, wet_location) {
            (Tw, _) => Some(C60),
            (Thw | Thwn | Rhw, _) => Some(C75),
            (Thw2 | Thwn2 | Xhhw2 | Rhw2, _) => Some(C90),
            (Thhw | Xhhw, false) => Some(C90),
            (Thhw | Xhhw, true) => Some(C75),
            (Thhn | Rhh, false) => Some(C90),
            (Thhn | Rhh, true) => None,
        }
    }

    /// Grupo de dimensiones de la Tabla 5.
    pub fn dimension_group(self) -> InsulationGroup {
        use Insulation::*;
        match self {
            Tw | Thw | Thhw | Thw2 => InsulationGroup::TwThwThhw,
            Thhn | Thwn | Thwn2 => InsulationGroup::ThhnThwn,
            Xhhw | Xhhw2 => InsulationGroup::Xhhw,
            Rhh | Rhw | Rhw2 => InsulationGroup::RhhRhw,
        }
    }
}

/// Tipo de motor de corriente alterna, para la Tabla 430-52. Los motores de
/// una fase no se distinguen: los decide el tipo de circuito.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MotorType {
    /// Inducción de jaula de ardilla, distinto del diseño B de alta eficiencia.
    #[default]
    SquirrelCage,
    /// Diseño B energéticamente eficiente.
    DesignBEnergyEfficient,
    WoundRotor,
    Synchronous,
}

impl MotorType {
    /// Clave de la fila en la tabla de protección del perfil.
    pub fn code(self) -> &'static str {
        match self {
            MotorType::SquirrelCage => "squirrel_cage",
            MotorType::DesignBEnergyEfficient => "design_b_energy_efficient",
            MotorType::WoundRotor => "wound_rotor",
            MotorType::Synchronous => "synchronous",
        }
    }
}
