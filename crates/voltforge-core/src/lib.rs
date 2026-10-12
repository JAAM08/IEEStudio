//! Base común de VoltForge: tipos del dominio eléctrico, unidades, errores y
//! validación de datos de entrada.
//!
//! No tiene dependencias obligatorias ni conoce tablas normativas, fórmulas o
//! interfaces. Las reglas (`voltforge-rules`), las fórmulas
//! (`voltforge-electrical`) y el proyecto (`voltforge-project`) se construyen
//! sobre estos tipos.

mod error;
pub mod system;
pub mod types;
pub mod units;
pub mod validation;

pub use error::{Error, Result};
pub use system::{CircuitKind, System};
pub use types::{
    ConduitType, Insulation, InsulationGroup, Material, RacewayMaterial, Size, TempRating,
};
