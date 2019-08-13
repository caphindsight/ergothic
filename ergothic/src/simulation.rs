use ::measure::Measures;
use ::std::time::Duration;
use ::std::time::SystemTime;

/// A configuration sample from the ergodic distribution must implement this
/// trait in order to be used in the *ergothic* simulation.
pub trait Sample {
  /// Creates a new configuration sample with randomized degrees of freedom.
  fn prepare() -> Self;

  /// Generally, randomized samples are highly atypical. In order to improve the
  /// quality of simulation results, a configuration sample has to be
  /// thermalized before measuring physical observables. This generally means
  /// running the simulation without recording observables for a couple dozens
  /// of iterations.
  /// Simulation engines allowed free to call this function from time to time to
  /// get rid of possible biases and improve ergodicity, as long as it is not on
  /// the critical path.
  fn thermalize(&mut self) {
    for _ in 0..20 {
      self.mutate();
    }
  }

  /// Make a randomized step in the configuration step. This function drives
  /// statistical simulations in *ergothic*. When implementing it, make sure
  /// that your implementation is not biased.
  /// The most common implementation of `mutate` uses the Metropolis algorithm.
  /// You may want to check out the `metropolis` module for useful helpers.
  fn mutate(&mut self);
}

/// Simulation parameters.
pub struct Parameters {
  /// The name of the simulation.
  pub name: String,

  /// List of measures relevant to the simulation. Each `flush_interval`, the
  /// measures from that list will be exported to the data sink.
  pub measures: ::measure::Measures,

  /// The polymorphic data exporter. Simulation engine will send measured data
  /// to the exporter every `flush_interval` seconds.
  pub exporter: Box<dyn (::export::Exporter)>,
  
  /// Interval between subsequent flushes of the accumulated values.
  pub flush_interval: Duration,

  /// Panic after this many export errors in a row.
  pub max_export_errors_in_row: Option<usize>,
}

/// Runs the simulation in the infinite loop. Consumes `self`.
/// The function `measure_fn` is called for each configuration sample. It is
/// supplied with the (immutable) sample, and a mutable reference to the
/// collection of measures. The function should calculate the values of the
/// physical quantities for the configuration sample and supply those to the
/// accumulators contained in the collection of measures.
pub fn run<S: Sample, F>(mut parameters: Parameters, measure_fn: F)
  where F: Fn(&S, &mut Measures) {
  info!("Running ergothic simulation \"{}\".", &parameters.name);
  // Prepare and thermalize a sample.
  let mut sample = S::prepare();
  sample.thermalize();
  let mut last_export_timestamp = SystemTime::now();
  let mut export_errors_in_row: usize = 0;
  loop {
    // Mutate the sample. This draws a new configuration from the ergodic
    // distribution.
    sample.mutate();

    // Measure and record the values of observables.
    measure_fn(&sample, &mut parameters.measures);

    if last_export_timestamp.elapsed().unwrap() >=
      parameters.flush_interval {
      last_export_timestamp = SystemTime::now();
      // Export a new data point containing the accumulated expectations.
      match parameters.exporter.export(&parameters.measures) {
        Ok(()) => {
          export_errors_in_row = 0;
          // Exported a data point. Reset the accumulated expectations and
          // continue the simulation.
          parameters.measures.reset();
        },
        Err(::export::ExportError(ref err)) => {
          export_errors_in_row += 1;
          // Export failed. Reporting an error and keeping the accumulated
          // expectations in hope of exporting them the next time.
          error!("Failed to export measured values: {:?}", err);
        },
      }
      if let Some(ref max_export_errors_in_row) =
             parameters.max_export_errors_in_row {
        if export_errors_in_row >= *max_export_errors_in_row {
          panic!("Reached a maximum of {} export errors in row.",
                 *max_export_errors_in_row);
        }
      }
    }
  }
}
