//! Motor de cálculo de VoltForge.
//!
//! Calcula instalaciones eléctricas de baja tensión con las tablas de un
//! perfil normativo. No depende de ninguna interfaz: la aplicación de
//! escritorio, la línea de comandos y los agentes usan estas mismas funciones.
//!
//! El motor trabaja con precisión completa y no redondea resultados
//! intermedios. El redondeo es asunto de quien presenta los valores.

pub mod conduit;
mod csv;
pub mod current;
mod error;
pub mod system;
pub mod tables;
pub mod voltage_drop;

pub use error::{Error, Result};
pub use system::{CircuitKind, System};
pub use tables::{ConduitType, InsulationGroup, Material, Profile, RacewayMaterial, Size};
