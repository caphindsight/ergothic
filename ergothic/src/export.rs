use std::collections::HashMap;
use super::measure;

/// An interface to a data sink accepting accumulated expectation values.
pub trait Exporter {
  /// Performs a single export operation. Note that it does not reset the
  /// accumulated values, which is the job of the simulation engine.
  fn export(&mut self, measures: &measure::Measures);
}

/// Keeps a copy of measures. On `export(..)`, merges the reported data and
/// outputs the accumulated values to stdout.
pub struct DebugExporter {
  accs: HashMap<String, measure::Acc>,
}

impl DebugExporter {
  /// Constructs a new DebugExporter.
  pub fn new() -> DebugExporter {
    DebugExporter {
      accs: HashMap::new(),
    }
  }
}

impl Exporter for DebugExporter {
  fn export(&mut self, measures: &measure::Measures) {
    // Merge the reported values to the global accumulated values.
    for measure in measures.slice() {
      if self.accs.contains_key(&measure.name) {
        let acc = self.accs.get_mut(&measure.name).unwrap();
        acc.merge(measure.acc.clone());
      } else {
        self.accs.insert(measure.name.clone(), measure.acc.clone());
      }
    }

    // Output the global accumulated values to stdout.
    println!("Measurements flushed:");
    for (name, acc) in self.accs.iter() {
      println!("{}:\t {}\t (err: {})", name, acc.value(), acc.uncertainty());
    }
    println!();
  }
}
