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

  /// Gives the number of recorded samples. Note that this function returns an
  /// `f64` due to the implementation specifics of `Acc`.
  pub fn num_of_samples(&self) -> f64 {
    self.count
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
    if value.is_nan() {
      return;
    }
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
