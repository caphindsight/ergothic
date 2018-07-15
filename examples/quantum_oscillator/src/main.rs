extern crate ergothic;
extern crate rand;

const N: usize = 30;  // Lattice size.
const A: f64 = 0.5;   // Lattice spacing.
const M: f64 = 1.0;   // Oscillator mass.
const K: f64 = 1.0;   // Oscillator spring tension.

// Potential energy V(x).
fn potential_v(x: f64) -> f64 {
  K * x.powi(2) / 2.0
}

// Trajectory of the oscillator: X(t) function.
struct Trajectory {
  x: Vec<f64>,
}

impl Trajectory {
  // Euclidean Lagrangian function for i-th link (between nodes i and i+1).
  fn lagrangian(&self, i: usize) -> f64 {
    assert!(i < N,
            "Trajectory::lagrangian(..): index {} out of range [0, {}).",
            i, N);
    let j = (i + 1) % N;
    let kinetic = M * (self.x[j] - self.x[i]).powi(2) / (2.0 * A);
    let potential = A * potential_v((self.x[i] + self.x[j]) / 2.0);
    // Euclidean signature, thus "+".
    kinetic + potential
  }

  // The part of the action that changes when the i-th node is mutated.
  fn contact_action(&self, i: usize) -> f64 {
    self.lagrangian(i) + self.lagrangian((i + N - 1) % N)
  }

  fn randomize(&mut self, n_times: usize) {
    use rand::distributions::Distribution;
    let mut rng = rand::prelude::thread_rng();
    let epsilon = 15.0;
    let uniform = rand::distributions::Uniform::<f64>
                      ::new_inclusive(-epsilon, epsilon);
    let uniform_prob = rand::distributions::Uniform::<f64>
                           ::new_inclusive(0.0, 1.0);
    for _ in 0..n_times {
      for i in 0..N {
        let old_x = self.x[i];
        let old_s = self.contact_action(i);
        self.x[i] = uniform.sample(&mut rng);
        let new_s = self.contact_action(i);
        let ds = new_s - old_s;
        if ds > 0.0 {
          // Metropolis-Hastings probabilistic step.
          let eta = uniform_prob.sample(&mut rng);
          if (-ds).exp() <= eta {
            // Restore the old value.
            self.x[i] = old_x;
          }
        }
      }
    }
  }
}

impl ergothic::Sample for Trajectory {
  fn prepare() -> Trajectory {
    Trajectory {
      x: vec![0.0; N],
    }
  }

  fn thermalize(&mut self) {
    self.randomize(500);
  }

  fn mutate(&mut self) {
    self.randomize(20);
  }
}

fn main() {
  let mut sim = ergothic::Simulation::new("Oscillator");
  // g[k] is the mean value of <X_i X_(i+k)> over i and over samples.
  let mut g = Vec::new();
  for i in 0..N {
    g.push(sim.add_measure(format!("G({})", i)));
  }
  sim.run(|s: &Trajectory, ms| {
    for k in 0..N {
      let mut g_k = 0.0;
      // Computing correlator $g[k] = N^{-1} \sum_i \left< X_i X_{i+k} \right>$.
      for i in 0..N {
        g_k += s.x[i] * s.x[(i + k) % N];
      }
      g_k /= N as f64;
      ms.accumulate(g[k], g_k);
    }
  });
}
