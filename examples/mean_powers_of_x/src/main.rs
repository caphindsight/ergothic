// Example statistical simulation using the ergothic library.
//
// This simulation computes the averages of the first 10 powers of `x` where `x`
// is a statistical variable uniformly distributed between 0 and 1.
//
// How to run:
// $ ./mean_powers_of_x
//
// In production mode, you will need to set up an instance of MongoDB.
// Run the simulation like this:
// $ ./mean_powers_of_x --production --mongo=mongodb://host:port
//   --mongo_db=database_name --mongo_coll=collection_name
//   --flush_interval_secs=10 --flush_interval_randomization=0
// This command will launch the simulation in production mode and export a data
// point to the mongodb instance each 10 seconds.
// Make sure to build the optimized binary.

extern crate ergothic;
extern crate pretty_env_logger;
extern crate rand;

// MySample is a configuration sample describing the system under consideration.
// Here it only has a single value `x`.
struct MySample {
  x: f64,
  rng: rand::rngs::ThreadRng,
  unif: rand::distributions::Uniform<f64>,
}

impl ergothic::Sample for MySample {
  // Prepare a randomized configuration. In our simple case, setting initial `x`
  // to zero is enough.
  fn prepare() -> MySample {
    MySample {
      x: 0.0,
      rng: rand::thread_rng(),
      unif: rand::distributions::Uniform::new_inclusive(0.0, 1.0),
    }
  }

  // On large configuration spaces, this function should move the randomized
  // sample to a regular point of the configuration space. That is, there's a
  // bias towards underrepresented points for recently initialized samples.
  // Thermalization tries to get rid of this bias. Typically, this function
  // usually calls mutate ~10-20 times. Here, it is only necessary to call it
  // once.
  fn thermalize(&mut self) {
    self.mutate();
  }

  // The main function which drives the simulation engine. Applies a randomized
  // mutation to the sample, thus making a single "step" in the configuration
  // spaces. The walk is assumed to be ergodic (in simple words, mutate is
  // assumed to not have any consistent bias.
  fn mutate(&mut self) {
    use rand::distributions::Distribution;
    // Set x to a random value in range [0.0, 1.0].
    self.x = self.unif.sample(&mut self.rng);
  }
}

fn main() {
  let mut simulation = ergothic::Simulation::new(
      "mean values of powers of [0..1]");
  
  let mut powers_of_x = Vec::with_capacity(10);
  for i in 0..10 {
    // Register a measure corresponding to X to the power of `i`.
    powers_of_x.push(simulation.add_measure(format!("Mean X^{}", i)));
  }

  // The entry-point function. It will parse the command line arguments and
  // set up the simulation parameters.
  simulation.run(|s: &MySample, ms| {
    // This is the measurement lambda. Its job is to measure the registered
    // measures in a given statisticle sample `s` and record the values in `ms`.
    for i in 0..10 {
      // Record X^i in the measure associated to i-th power of X.
      ms.accumulate(powers_of_x[i], s.x.powi(i as i32));
    }
  });
}
