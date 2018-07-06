/// An `Acc` (short for accumulator) is a counter that can consume samples from
/// an ergodic process. In *ergothic*, `Acc`s correspond to statistical
/// observables. For example, in a lattice QFT simulation `Acc`s would
/// correspond to Schwinger functions, Wilson loops, etc.
/// Implementation is optimized for correctness (avoiding round-off errors), not
/// performance. It is expected that updating `Acc`s is not on the critical path
/// of the simulation. For Quantum Field Theory on the lattice in 4 spacetime
/// dimensions that is usually the case.
#[derive(Clone, Deserialize, Serialize)]
pub struct Acc {
  count: f64,
  mean: f64,
  mean2: f64,
}

impl Acc {
  /// Constructs an empty `Acc`. The mean value is set to 0, and the statistical
  /// uncertainty is NaN.
  pub fn new() -> Acc {
    Acc {
      mean: 0.0,
      count: 0.0,
      mean2: 0.0,
    }
  }
  
  /// Gives the mean of previously consumed samples. It approximates the
  /// expectation value of the physical observable corresponding to the `Acc`.
  pub fn value(&self) -> f64 {
    self.mean
  }

  /// Gives the statistical error estimate based on the standard deviation and
  /// size of the distribution of consumed samples.
  /// The statistical error is equal to the standard deviation divided by the
  /// square root of the size of the distribution. The intuition for this
  /// formula can be developed by considering the random walk problem.
  pub fn uncertainty(&self) -> f64 {
    ((self.mean2 - self.mean.powi(2)) / self.count).sqrt()
  }

  /// Consumes a sample value. This function should be called every time a new
  /// sample is drawn from the ergodic process. The `value` argument represents
  /// the value that the physical observable corresponding to the `Acc` takes in
  /// the sample configuration. The correctness of the approximation of the
  /// expectation value by the mean value of the `Acc` entirely depends on how
  /// this function is called. That is, if the algorithm that draws random
  /// sample configurations is biased in any way, the `Arc` will not reproduce
  /// the correct expectation value.
  pub fn consume(&mut self, value: f64) {
    self.count += 1.0;
    self.mean += (value - self.mean) / self.count;
    self.mean2 += (value.powi(2) - self.mean2) / self.count;
  }

  /// Merges another `Acc` into this one. Semantically equivalent to calling
  /// `self.consume(..)` for each of the samples consumed previously by `other`.
  /// Destructs `other` upon completion.
  pub fn merge(&mut self, mut other: Acc) {
    let total_count = self.count + other.count;
    self.mean -= self.mean * (other.count / total_count);
    other.mean -= other.mean * (self.count / total_count);
    self.mean += other.mean;
    self.mean2 -= self.mean2 * (other.count / total_count);
    other.mean2 -= other.mean2 * (self.count / total_count);
    self.mean2 += other.mean2;
    self.count = total_count;
  }
}

/// Represents a physical observable. Measuring expectation values of
/// observables is the purpose of any *ergothic* simulation.
#[derive(Serialize, Deserialize)]
pub struct Measure {
  /// The human-readable name given to the observable. Generally use the same
  /// notation as the one used in the paper/preprint accomodating the
  /// simulation.
  pub name: String,

  /// The corresponding accumulator. Consumes values of the observable, measured
  /// for configuration samples drawn from the ergodic ensemble.
  pub acc: Acc,
}

/// A collection of physical observables. Determining expectation values of each
/// of the measures with reasonable accuracy is the sole purpose of the
/// *ergothic* simulation.
#[derive(Serialize, Deserialize)]
pub struct Measures {
  measures: Vec<Measure>,
}

/// A thin wrapper around a positional index corresponding to a specific
/// measure. Instead of using interior mutability, we demand that userspace code
/// refers to specific measures by their indices, safely wrapped in
/// `MeasureIdx`.
#[derive(Clone, Copy)]
pub struct MeasureIdx(usize);

impl Measures {
  /// Constructs an empty collection of measures.
  pub fn new() -> Measures {
    Measures {
      measures: Vec::new(),
    }
  }

  /// Returns an immutable slice of registered measures.
  pub fn slice(&self) -> &[Measure] {
    &self.measures
  }

  /// Registers a new measure with a given `name`. Returns a safely wrapped
  /// index of the measure in the collection of measures.
  pub fn register(&mut self, name: String) -> MeasureIdx {
    self.measures.push(Measure {
      name,
      acc: Acc::new(),
    });
    MeasureIdx(self.measures.len() - 1)
  }

  /// Returns the name of the measure pointed to by `idx`.
  pub fn name(&self, idx: MeasureIdx) -> &str {
    &self.measures[idx.0].name
  }

  /// Returns an immutable reference to the accumulator corresponding to the
  /// measure pointed to by `idx`.
  pub fn acc(&self, idx: MeasureIdx) -> &Acc {
    &self.measures[idx.0].acc
  }

  /// Returns a mutable reference to the accumulator corresponding to the
  /// measure pointer to by `idx`.
  pub fn acc_mut(&mut self, idx: MeasureIdx) -> &mut Acc {
    &mut self.measures[idx.0].acc
  }

  /// Resets accumulators for all measures, effectively forgetting about all
  /// recorded samples.
  pub fn reset_accs(&mut self) {
    for measure in self.measures.iter_mut() {
      measure.acc = Acc::new();
    }
  }
}
