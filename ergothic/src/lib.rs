extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

/// Exporters provide interfaces for sending the measured expectation values to
/// different types of data sinks.
pub mod export;

/// Utilities related to accumulating mean values and statistical errors for
/// physical observables measured on sample configurations drawn from the
/// ergodic distribution.
pub mod measure;

/// The simulation orchestration engine is the core part of *ergothic*.
pub mod simulation;
