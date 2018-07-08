use ::accumulate::Acc;
use ::std::collections::HashMap;

/// Represents a physical observable. Measuring expectation values of
/// observables is the purpose of any *ergothic* simulation.
#[derive(Clone, Serialize, Deserialize)]
pub struct Measure {
  /// The human-readable name given to the observable. Generally use the same
  /// notation as the one used in the paper/preprint accomodating the
  /// simulation.
  pub name: String,

  /// The corresponding accumulator. Consumes values of the observable, measured
  /// for configuration samples drawn from the ergodic ensemble.
  pub acc: Acc,
}

/// A thin wrapper around a positional index corresponding to a specific
/// measure. Instead of using interior mutability, we demand that userspace code
/// refers to specific measures by their indices, safely wrapped in
/// `MeasureIdx`.
#[derive(Clone, Copy)]
pub struct MeasureIdx(usize);

/// A collection of physical observables. Determining expectation values of each
/// of the measures with reasonable accuracy is the sole purpose of the
/// *ergothic* simulation.
#[derive(Clone, Serialize, Deserialize)]
pub struct Measures {
  measures: Vec<Measure>,
}

impl Measures {
  /// Constructs an empty collection of measures.
  pub fn new_empty() -> Measures {
    Measures {
      measures: Vec::new(),
    }
  }

  /// Returns an immutable slice of registered measures.
  pub fn slice(&self) -> &[Measure] {
    &self.measures
  }

  /// Returns an immutable reference to the measure pointed to by `idx`.
  pub fn get(&self, idx: MeasureIdx) -> &Measure {
    &self.measures[idx.0]
  }

  /// Resets accumulators for all measures, effectively forgetting about all
  /// recorded samples.
  pub fn reset(&mut self) {
    for measure in self.measures.iter_mut() {
      measure.acc = Acc::new();
    }
  }

  /// Returns a mutable reference to the accumulator corresponding to the
  /// measure pointed to by `idx`.
  pub fn accumulator(&mut self, idx: MeasureIdx) -> &mut Acc {
    &mut self.measures[idx.0].acc
  }

  /// Shorthand for `self.accumulator(idx).consume(value)`.
  pub fn accumulate(&mut self, idx: MeasureIdx, value: f64) {
    self.accumulator(idx).consume(value);
  }
}

pub struct MeasureRegistry {
  measures: Measures,
  name_index: HashMap<String, MeasureIdx>,
}

/// Contains a list of measures and a map from measure names to measure indexes.
impl MeasureRegistry {
  /// Constructs an empty `MeasureRegistry` object.
  pub fn new() -> MeasureRegistry {
    MeasureRegistry {
      measures: Measures::new_empty(),
      name_index: HashMap::new(),
    }
  }
  
  /// Returns an immutable reference to the collection of measures.
  pub fn measures(&self) -> &Measures {
    &self.measures
  }

  /// Lookup of the measure by its name. Returns a measure index or `None` if a
  /// measure with a given name doesn't exist.
  pub fn find(&mut self, name: &str) -> Option<MeasureIdx> {
    self.name_index.get(name).cloned()
  }

  /// Returns an interior-immutable list of measures suitable for using in the
  /// *ergodic* simulation engine. Destructs `self`.
  /// Previously returned by `self.register(..)` measure indices can be used to
  /// introspect the resulting list of measures.
  pub fn freeze(self) -> Measures {
    self.measures
  }
  
  /// Registers a new measure with a given `name`. Returns a safely wrapped
  /// index of the measure in the collection of measures. If a measure with the
  /// same name has been registered before, panics.
  pub fn register(&mut self, name: String) -> MeasureIdx {
    if self.name_index.contains_key(&name) {
      panic!("Ambiguous measure definition: '{}' was registered twice.", &name);
    }
    self.measures.measures.push(Measure {
      name: name.clone(),
      acc: Acc::new(),
    });
    let res_idx = MeasureIdx(self.measures.measures.len() - 1);
    self.name_index.insert(name, res_idx);
    res_idx
  }

  /// Returns a mutable reference to the accumulator corresponding to a measure
  /// pointed to by `idx`.
  pub fn accumulator(&mut self, idx: MeasureIdx) -> &mut Acc {
    self.measures.accumulator(idx)
  }
}
