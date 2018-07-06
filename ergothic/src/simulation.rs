use std::time::Duration;
use std::time::SystemTime;
use super::measure;
use super::export;

/// A configuration sample from the ergodic distribution must implement this
/// trait in order to be used in the *ergothic* simulation.
pub trait Sample {
  /// Creates a new configuration sample with randomized degrees of freedom.
  fn prepare_randomized() -> Self;

  /// Generally, randomized samples are highly atypical. In order to improve the
  /// quality of simulation results, a configuration sample has to be
  /// thermalized before measuring physical observables. This generally means
  /// running the simulation without recording observables for a couple dozens
  /// of iterations.
  /// Simulation engines allowed free to call this function from time to time to
  /// get rid of possible biases and improve ergodicity, as long as it is not on
  /// the critical path.
  fn thermalize(&mut self);

  /// Make a randomized step in the configuration step. This function drives
  /// statistical simulations in *ergothic*. When implementing it, make sure
  /// that your implementation is not biased.
  /// The most common implementation of `mutate` uses the Metropolis algorithm.
  /// You may want to check out the `metropolis` module for useful helpers.
  fn mutate(&mut self);
}

/// Simulation parameters.
pub struct Parameters {
  /// Interval between subsequent flushes of the accumulated values.
  pub flush_interval: Duration,

  /// The polymorphic data exporter. Simulation engine will send measured data
  /// to the exporter every `flush_interval` seconds.
  pub exporter: Box<export::Exporter>,
}

impl Parameters {
  /// Construct simulation parameters with defaults.
  pub fn new() -> Parameters {
    Parameters {
      flush_interval: Duration::from_secs(60),
      exporter: Box::new(export::DebugExporter::new()),
    }
  }
}

/// The statistical simulation. *Ergothic* will do its best to handle all
/// implementation details and reduce boilerplate.
/// `Simulation<S>` is not a trait, and overriding the simulation algorithm is
/// not possible. However, everything specific to the model is configurable by
/// implementing the functions of the `Sample` trait.
pub struct Simulation<S: Sample> {
  sample: S,
  measures: measure::Measures,
  parameters: Parameters,
}

impl <S: Sample> Simulation<S> {
  /// Constructs a new `Simulation<S>`. Prepares a randomized sample of type `S`
  /// and an empty collection of measures, corresponding to physical observables.
  pub fn new(parameters: Parameters) -> Simulation<S> {
    Simulation {
      sample: S::prepare_randomized(),
      measures: measure::Measures::new(),
      parameters,
    }
  }

  /// Returns an immutable reference to the simulation parameters.
  pub fn parameters(&self) -> &Parameters {
    &self.parameters
  }

  /// Returns an immutable reference to the collection of measures.
  pub fn measures(&self) -> &measure::Measures {
    &self.measures
  }

  /// Returns a mutable reference to the collection of measures.
  pub fn measures_mut(&mut self) -> &mut measure::Measures {
    &mut self.measures
  }

  /// Runs the simulation in the infinite loop. Consumes `self`.
  /// The function `measure_fn` is called for each configuration sample. It is
  /// supplied with the (immutable) sample, and a mutable reference to the
  /// collection of measures. The function should calculate the values of the
  /// physical quantities for the configuration sample and supply those to the
  /// accumulators contained in the collection of measures.
  pub fn run<F>(mut self, measure_fn: F)
    where F: Fn(&S, &mut measure::Measures) {
    self.sample.thermalize();
    let mut last_export_timestamp = SystemTime::now();
    loop {
      self.sample.mutate();
      measure_fn(&self.sample, &mut self.measures);
      if last_export_timestamp.elapsed().unwrap() >=
         self.parameters.flush_interval {
         last_export_timestamp = SystemTime::now();
         self.parameters.exporter.export(&self.measures);
         self.measures.reset_accs();
      }
    }
  }
}
