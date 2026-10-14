//! Modelo del proyecto de VoltForge y su cálculo completo.
//!
//! Una instalación se captura una sola vez en [`Project`]; de ese modelo
//! [`calculate`] deriva cada circuito, cada alimentador y el cuadro de cargas.

pub mod balance;
mod calc;
mod demand;
pub mod example;
mod fault;
mod model;
mod motors;
mod procedure;
pub mod quick;
mod transformer;
pub mod verdict;

pub use calc::{
    calculate, Check, ConduitChoice, ElectrodeConductor, PanelResult, ProjectResult, Run,
    RunResult, Status, TransformerResult,
};
pub use demand::DemandLine;
pub use model::{
    Area, Circuit, CompressorLoad, Demand, Feeder, Load, LoadKind, MotorLoad, Overrides, Panel,
    Phase, Project, ProjectInfo, Settings, Source, SourceFault, Transformer, FORMAT_VERSION,
};
pub use motors::hp_label;
pub use procedure::{insulation_name, material_name, size_label, Step};
