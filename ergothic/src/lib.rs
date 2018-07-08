//! Ergothic is a collection of helpers for setting up and running distributed
//! statistical Monte-Carlo simulations written in Rust. It is multi-purpose and
//! will work for any statistical simulation based on Monte-Carlo or its
//! variations (Metropolis-Hastings, etc.). However, its primary purpose is to
//! ease the toil for simulating Quantum Field Theory on the lattice.
//! Ergothic will perform routine tasks unrelated to the subject of your
//! research for you, allowing you to focus on the code that matters.
//! Simulations written with ergothic can run both in the single-threaded local
//! environment, which is super easy to debug, and on clusters with tens of
//! thousands of nodes. No code changes are required to scale your simulation up
//! to any number, that technical part has already been taking care for you!

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
