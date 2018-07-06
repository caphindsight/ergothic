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
  pretty_env_logger::init();

  let mut params = ergothic::simulation::Parameters::new();
  params.flush_interval = std::time::Duration::from_secs(2);
  params.exporter = Box::new(ergothic::export::MongoExporter::new(
      "mongodb://localhost:27017",
      "ergothic_data",
      "test",
      None
  ));
  let mut sim = ergothic::simulation::Simulation::<MySample>::new(params);

  // Mean value of X. Should be 0.5.
  let mean_x = sim.measures_mut().register("Mean X  ".to_string());

  // Mean value of X^2. Should be 1/3.
  let mean_x2 = sim.measures_mut().register("Mean X^2".to_string());

  sim.run(|s, ms| {
    ms.acc_mut(mean_x).consume(s.x);
    ms.acc_mut(mean_x2).consume(s.x.powi(2));
  });
}
