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
mod accumulate;

/// Exporters provide interfaces for sending the measured expectation values to
/// different types of data sinks.
mod export;

/// Helper classes for measures and measure registries.
mod measure;

/// The simulation orchestration engine is the core part of *ergothic*.
mod simulation;

/// Helpers for assembling binaries capable of running the same simulation in
/// development and production modes.
mod startup;


// Following are the elements of the public API.

/// Sample trait defines an object acting as a statistical sample.
pub use simulation::Sample;

/// Positional index of a measure in the measure registry. Indices are wrapped
/// in `MeasureIdx` type for type safety.
pub use measure::MeasureIdx;

/// Public interface to measure registry and the entry point function.
pub struct Simulation {
  name: String,
  measure_registry: measure::MeasureRegistry,
}

impl Simulation {
  /// Constructs a new simulation.
  pub fn new<N: ToString>(name: N) -> Simulation {
    Simulation {
      name: name.to_string(),
      measure_registry: measure::MeasureRegistry::new(),
    }
  }

  /// Registers a measure in the underlying measure registry and returns its
  /// positional index safely wrapped in the `MeasureIdx` type.
  pub fn add_measure<N: ToString>(&mut self, name: N) -> MeasureIdx {
    self.measure_registry.register(name.to_string())
  }

  /// Entry point function. All ergothic simulations should call this function.
  /// Consumes `self` to indicate that the simulation runs in an infinite loop
  /// and never returns.
  pub fn run<S: simulation::Sample, F>(self, f: F)
    where F: Fn(&S, &mut measure::Measures) {
    startup::run_simulation(&self.name, self.measure_registry, f);
  }
}
