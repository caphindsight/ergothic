extern crate ergothic;
extern crate pretty_env_logger;
extern crate rand;

struct MySample {
  x: f64,
  rng: rand::rngs::ThreadRng,
  unif: rand::distributions::Uniform<f64>,
}

impl ergothic::simulation::Sample for MySample {
  fn prepare_randomized() -> MySample {
    MySample {
      x: 0.0,
      rng: rand::thread_rng(),
      unif: rand::distributions::Uniform::new_inclusive(0.0, 1.0),
    }
  }

  fn thermalize(&mut self) {
    self.mutate();
  }

  fn mutate(&mut self) {
    use rand::distributions::Distribution;
    self.x = self.unif.sample(&mut self.rng);
  }
}

fn main() {
  let mut measures = ergothic::measure::MeasureRegistry::new();

  // Mean value of X. Should be 0.5.
  let mean_x = measures.register("Mean X".to_string());

  // Mean value of X^2. Should be 1/3.
  let mean_x2 = measures.register("Mean X^2".to_string());

  ergothic::run_simulation(
    "mean values of powers of [0..1]", measures, |s: &MySample, ms| {
    ms.accumulate(mean_x, s.x);
    ms.accumulate(mean_x2, s.x.powi(2));
  });
}
