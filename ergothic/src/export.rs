use ::measure::Measures;
use ::measure::MeasureRegistry;
use ::std::time::SystemTime;

/// Errors returned by the exporter. Contain a string describing the cause of
/// the error.
#[derive(Debug)]
pub struct ExportError(pub String);

/// An interface to a data sink accepting accumulated expectation values.
pub trait Exporter {
  /// Performs a single export operation. Note that it does not reset the
  /// accumulated values, which is the job of the simulation engine.
  fn export(&mut self, measures: &Measures)
     -> Result<(), ExportError>;
}

/// Keeps a copy of measures. On `export(..)`, merges the reported data and
/// outputs the accumulated values to stdout.
pub struct DebugExporter {
  aggregated: MeasureRegistry,
  creation_timestamp: SystemTime,
}

impl DebugExporter {
  /// Constructs a new DebugExporter.
  pub fn new() -> DebugExporter {
    DebugExporter {
      aggregated: MeasureRegistry::new(),
      creation_timestamp: SystemTime::now(),
    }
  }
  
  /// Format the results in a pretty table.
  fn pretty_table(measures: &Measures) -> ::prettytable::Table {
    use ::prettytable::Table;
    use ::prettytable::row::Row;
    use ::prettytable::cell::Cell;
    use ::prettytable::format::Alignment;
    let mut table = Table::new();
    table.set_format(
        *::prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(Row::new(vec![
      Cell::new_align("MEASURE", Alignment::CENTER),
      Cell::new_align("EXPECTATION", Alignment::CENTER),
      Cell::new_align("UNCERTAINTY", Alignment::CENTER),
      Cell::new_align("RELATIVE UNCERTAINTY", Alignment::CENTER),
    ]));
    for measure in measures.slice() {
      let expectation = format!("{}", measure.acc.value());
      let uncertainty = format!("{}", measure.acc.uncertainty());
      let relative_uncertainty =
        format!("{}", measure.acc.uncertainty()
                    / measure.acc.value().abs());
      table.add_row(Row::new(vec![
        Cell::new_align(&measure.name, Alignment::RIGHT),
        Cell::new(&expectation),
        Cell::new(&uncertainty),
        Cell::new(&relative_uncertainty),
      ]));
    }
    table
  }
}

impl Exporter for DebugExporter {
  fn export(&mut self, measures: &Measures)
     -> Result<(), ExportError> {
    let mut samples_processed: usize = 0;
    // Merge the reported values to the global accumulated values.
    for measure in measures.slice() {
      let measure_idx = match self.aggregated.find(&measure.name) {
        Some(idx) => idx,
        None => self.aggregated.register(measure.name.clone()),
      };
      self.aggregated.accumulator(measure_idx).merge(measure.acc.clone());
      samples_processed =
        self.aggregated.accumulator(measure_idx).num_of_samples() as usize;
    }

    // Output the global accumulated values to stdout.
    println!();
    println!("Simulation uptime: {} secs",
             self.creation_timestamp.elapsed().unwrap().as_secs());
    println!("Samples processed: {}", samples_processed);
    println!("Aggregate values:");
    DebugExporter::pretty_table(self.aggregated.measures()).printstd();
    Ok(())
  }
}

/// Exports the measured values to a remote MongoDB collection. Each call to
/// `export(..)` will create a new document containing the internal states of
/// all of the accumulators provided.
/// `MongoExporter` will handle database errors (which will happen from time to
/// time) gracefully by returning an error value from `self.export(..)`.
/// However, serialization errors indicate a serious problem with the binary.
/// Thus if one is encountered, `MongoExporter` panicks immediately.
pub struct MongoExporter {
  _client: ::mongodb::Client,
  collection: ::mongodb::coll::Collection,
  write_concern: Option<::mongodb::common::WriteConcern>,
  formatted_addr: String,
}

impl MongoExporter {
  /// Constructs a new MongoExporter. Panics on errors.
  /// Example usage:
  /// ```
  /// let exporter = MongoExporter::new(
  ///   /*addr=*/"mongodb://localhost:27017,localhost:27018/",
  ///   /*db_name=*/"ergothic_results",
  ///   /*coll_name=*/"my_simulation",
  ///   /*write_concern=*/None);
  /// ```
  pub fn new(addr: &str, db_name: &str, coll_name: &str,
             write_concern: Option<::mongodb::common::WriteConcern>)
         -> MongoExporter {
    use ::mongodb::ThreadedClient;
    use ::mongodb::db::ThreadedDatabase;
    let client = ::mongodb::Client::with_uri(addr)
        .expect("Failed to initialize MongoDB client.");
    let coll = client.db(db_name).collection(coll_name);
    MongoExporter {
      _client: client,
      collection: coll,
      write_concern,
      formatted_addr: format!("{}, db={}, col={}", addr, db_name, coll_name),
    }
  }
}

impl Exporter for MongoExporter {
  fn export(&mut self, measures: &Measures) -> Result<(), ExportError> {
    let serialized_data = ::mongodb::to_bson(measures)
        .expect("Serialization error");
    if let ::mongodb::Bson::Document(doc) = serialized_data {
      match self.collection.insert_one(doc, self.write_concern.clone()) {
        Ok(res) => {
          if res.acknowledged {
            if let Some(::mongodb::Bson::ObjectId(id)) = res.inserted_id {
              info!("Measurements flushed to {}, obj_id={}",
                    self.formatted_addr, id.to_hex());
              Ok(())
            } else {
              Err(ExportError(format!(
                  "MongoDB didn't return a new object ID.")))
            }
          } else {
            Err(ExportError(format!(
                "MongoDB did not acknowledge measurements.")))
          }
        },
        Err(err) => {
          Err(ExportError(format!("{:?}", err)))
        },
      }
    } else {
      panic!("Serialization error: expected Bson::Document, found {}",
             &serialized_data);
    }
  }
}
