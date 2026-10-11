//! Modelo del proyecto de VoltForge y su cálculo completo.
//!
//! Una instalación se captura una sola vez en [`Project`]; de ese modelo
//! [`calculate`] deriva cada circuito, cada alimentador y el cuadro de cargas.

mod calc;
pub mod example;
mod model;
pub mod quick;

pub use calc::{
    calculate, Check, ConduitChoice, PanelResult, ProjectResult, Run, RunResult, Status,
};
pub use model::{
    Circuit, Feeder, Load, LoadKind, Overrides, Panel, Phase, Project, Settings, Source,
    FORMAT_VERSION,
};
