//! Fórmulas eléctricas de VoltForge.
//!
//! Corriente, caída de tensión, selección de conductor, protección, puesta a
//! tierra y tubería, con las tablas de un perfil de `voltforge-rules`. No
//! depende de ninguna interfaz.
//!
//! Se calcula con precisión completa y sin redondear resultados intermedios.
//! El redondeo es asunto de quien presenta los valores.

pub mod conduit;
pub mod current;
pub mod motor;
pub mod sizing;
pub mod voltage_drop;
