//! Perfiles normativos de VoltForge.
//!
//! Un perfil es el conjunto de tablas con que se calcula, por ejemplo la
//! NOM-001-SEDE-2012. Los valores no están en el código: se leen de los
//! archivos CSV de `data/<perfil>/`, que se incrustan al compilar.

pub mod csv;
pub mod references;
pub mod tables;

pub use references::{cite_all, Parameters, Rule, RuleReference};
pub use tables::Profile;
