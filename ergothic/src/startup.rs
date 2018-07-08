use ::export::Exporter;
use ::measure::MeasureRegistry;
use ::measure::Measures;
use ::simulation::Parameters;
use ::structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="ergothic simulation",
  about="A distributed statistical simulation using ergothic library.")]
pub struct CmdArgs {
  /// Run in production mode. Exactly one exporter spec should be provided.
  #[structopt(long="production")]
  pub production_mode: bool,

  /// MongoDB connection string to export measurements to. Child arguments:
  /// [--mongo_db, --mongo_coll].
  /// Example: --mongo mongodb://localhost:27017/
  #[structopt(long="mongo")]
  pub mongo: Option<String>,

  /// MongoDB database name. Parent argument: --mongo.
  /// Example: --mongo_db ergothic_data
  #[structopt(long="mongo_db")]
  pub mongo_db: Option<String>,

  /// MongoDB collection name. Parent argument: --mongo.
  /// Example: --mongo_coll my_simulation
  #[structopt(long="mongo_coll")]
  pub mongo_coll: Option<String>,

  /// Flush interval for measurements in seconds.
  /// Example: --flush_interval_secs 600 (flush every 10 minutes).
  #[structopt(long="flush_interval_secs")]
  pub flush_interval_secs: Option<u64>,

  /// Relative magnitude of the flush interval randomization. Allowed range is
  /// [0, 1). The real interval will be chosen at random within this fraction of
  /// the --flush_interval_secs. Has effect only in production mode.
  /// Example: --flush_interval_randomization 0.2 (randomize within 20%).
  #[structopt(long="flush_interval_randomization", default_value="0.5")]
  pub flush_interval_randomization: f64,

  /// Simulation will panic after receiving this many export errors in a row.
  /// Default value is infinity.
  #[structopt(long="max_errors_in_row")]
  pub max_export_errors_in_row: Option<usize>,
}

/// Parses the command line arguments and produces simulation parameters.
pub fn construct_parameters(name: String, measures: Measures, args: CmdArgs)
       -> Parameters {
  let mut rng = ::rand::thread_rng();
  use ::rand::distributions::Distribution;
  let exporter: Box<Exporter>;
  if args.production_mode {
    if cfg!(debug_assertions) {
      panic!("Please build an optimized binary.");
    }
    if let Some(mongo) = args.mongo {
      let mongo_db = args.mongo_db
        .expect("Child argument --mongo_db is required.");
      let mongo_coll = args.mongo_coll
        .expect("Child argument --mongo_coll is required.");
      exporter = Box::new(
        ::export::MongoExporter::new(&mongo, &mongo_db, &mongo_coll, None));
    } else {
      panic!("Argument --mongo is required in production mode.");
    }
  } else {
    exporter = Box::new(::export::DebugExporter::new());
  }

  let flush_interval_secs;
  if let Some(flush_interval_secs_some) = args.flush_interval_secs {
    flush_interval_secs = flush_interval_secs_some;
  } else {
    if args.production_mode {
      // Default value for production is to flush every 5 minutes.
      flush_interval_secs = 300;
    } else {
      // Default value for development mode is to flush every 2 seconds.
      flush_interval_secs = 2;
    }
  }

  if args.flush_interval_randomization < 0.0 ||
     args.flush_interval_randomization >= 1.0 {
    panic!("Argument --path_integral_randomization should lie within [0, 1).");
  }

  let flush_interval_min = ::std::cmp::max(1, 
      (flush_interval_secs as f64 * (1.0 - args.flush_interval_randomization))
      .round() as u64);
  let flush_interval_max = (flush_interval_secs as f64 *
                           (1.0 + args.flush_interval_randomization))
                           .round() as u64;
  let flush_interval_dist =
    ::rand::distributions::Uniform::<u64>::new_inclusive(
      flush_interval_min, flush_interval_max);
  let flush_interval = ::std::time::Duration::from_secs(
    flush_interval_dist.sample(&mut rng));

  let max_export_errors_in_row = args.max_export_errors_in_row;

  Parameters {
    name,
    measures,
    exporter,
    flush_interval,
    max_export_errors_in_row,
  }
}

pub fn run_simulation<S, F>(name: &str, reg: MeasureRegistry, measure_fn: F)
  where S: ::simulation::Sample,
        F: Fn(&S, &mut Measures) {
  let cmd_args = CmdArgs::from_args();
  if cmd_args.production_mode {
    ::simple_logger::init().expect("Failed to initialize logger");
  } else {
    println!("Running ergothic simulation \"{}\".", name);
  }
  let parameters = construct_parameters(name.to_string(), reg.freeze(),
                                        cmd_args);
  ::simulation::run(parameters, measure_fn);
}
