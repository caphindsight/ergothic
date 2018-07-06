use std::collections::HashMap;
use super::bson;
use super::measure;
use super::mongodb;

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

/// Exports the measured values to a remote MongoDB collection. Each call to
/// `export(..)` will create a new document containing the internal states of
/// all of the accumulators provided.
pub struct MongoExporter {
  _client: mongodb::Client,
  collection: mongodb::coll::Collection,
  write_concern: Option<mongodb::common::WriteConcern>,
  formatted_addr: String,
}

impl MongoExporter {
  /// Constructs a new MongoExporter.
  /// Example usage:
  /// ```
  /// let exporter = MongoExporter::new(
  ///   /*addr=*/"mongodb://localhost:27017,localhost:27018/",
  ///   /*db_name=*/"ergothic_results",
  ///   /*coll_name=*/"my_simulation",
  ///   /*write_concern=*/None);
  /// ```
  pub fn new(addr: &str, db_name: &str, coll_name: &str,
             write_concern: Option<mongodb::common::WriteConcern>)
         -> MongoExporter {
    use mongodb::ThreadedClient;
    use mongodb::db::ThreadedDatabase;
    let client = mongodb::Client::with_uri(addr)
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
  fn export(&mut self, measures: &measure::Measures) {
    let serialized_data = bson::to_bson(measures);
    if serialized_data.is_err() {
      error!("Error occured when serializing measurements to BSON:\n{}",
             serialized_data.err().unwrap());
      return;
    }
    let serialized_data = serialized_data.unwrap();
    if let bson::Bson::Document(doc) = serialized_data {
      let res = self.collection.insert_one(doc, self.write_concern.clone());
      if res.is_err() {
        error!("Error occured when writing data to MongoDB:\n{}",
               res.err().unwrap());
        return;
      }
      let res = res.unwrap();
      if res.acknowledged {
        if let Some(bson::Bson::ObjectId(id)) = res.inserted_id {
          info!("Measurements flushed to {}, obj_id={}",
                self.formatted_addr, id.to_hex());
        } else {
          warn!("MongoDB acknowledged measurements, but didn't return ObjectId.");
        }
      } else {
        error!("MongoDB didn't acknowledge measurements.");
      }
    } else {
      error!("Error occured when serializing measurements to Bson:\nInvalid bson structure produced: {:?}",
             &serialized_data);
      return;
    }
  }
}
