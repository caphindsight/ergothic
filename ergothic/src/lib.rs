extern crate argparse;
extern crate bson;
extern crate mongodb;
extern crate prettytable;
extern crate rand;
extern crate serde;
extern crate simple_logger;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

/// Utilities related to accumulating mean values and statistical errors for
/// physical observables measured on sample configurations drawn from the
/// ergodic distribution.
pub mod accumulate;

/// Exporters provide interfaces for sending the measured expectation values to
/// different types of data sinks.
pub mod export;

/// Helper classes for measures and measure registries.
pub mod measure;

/// The simulation orchestration engine is the core part of *ergothic*.
pub mod simulation;

/// Helpers for assembling binaries capable of running the same simulation in
/// development and production modes.
pub mod startup;

/// Entry-point function for all simulations.
pub use startup::run_simulation;
