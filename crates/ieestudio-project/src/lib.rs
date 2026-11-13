//! Modelo del proyecto de IEEStudio y su cálculo completo.
//!
//! Una instalación se captura una sola vez en [`Project`]; de ese modelo
//! [`calculate`] deriva cada circuito, cada alimentador y el cuadro de cargas.

pub mod balance;
mod calc;
pub mod connection;
mod demand;
pub mod evaluation;
pub mod example;
pub mod fault;
mod model;
mod motors;
pub mod positions;
mod procedure;
pub mod quick;
pub mod review;
mod transformer;
pub mod verdict;

pub use calc::{
    calculate, Check, ConduitChoice, ElectrodeConductor, PanelResult, ProjectResult, Run,
    RunResult, Status, TransformerResult, MISSING_NONLINEAR,
};
pub use demand::DemandLine;
pub use model::{
    Area, Board, BreakerData, Circuit, CompressorLoad, Connection, Demand, FaultLocation, Feeder,
    Load, LoadKind, MotorLoad, Mounting, Overrides, Panel, Phase, Project, ProjectInfo, Settings,
    Source, SourceFault, Substation, Transformer, FORMAT_VERSION,
};
pub use motors::hp_label;
pub use procedure::{insulation_name, material_name, size_label, Step};
