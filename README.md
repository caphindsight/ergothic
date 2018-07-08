# Ergothic
Ergothic is a collection of helpers for setting up and running distributed statistical Monte-Carlo simulations written in Rust.
It is multi-purpose and will work for any statistical simulation based on Monte-Carlo or its variations (Metropolis-Hastings, etc.).
However, its primary purpose is to ease the toil for simulating Quantum Field Theory on the lattice.

Ergothic will perform routine tasks unrelated to the subject of your research for you, allowing you to focus on the code that matters.
Simulations written with ergothic can run both in the single-threaded local environment, which is super easy to debug, and on clusters with tens of thousands of nodes.
No code changes are required to scale your simulation up to any number, that technical part has already been taken care of for you!

## Basic tutorial
Ergothic provides a very simple API that you can use to set up your simulation.
You only need to know about the following concepts.

### Samples
**A sample** is a point in the configuration space representing a possible configuration of the system.
For example, in lattice QCD a sample constitues an assignment of SU(3) group elements called the holonomies to link of the lattice.

#### In code
Create a data type representing a sample configuration in your simulation:

```rust
struct MySample {
  ...
}
```

Implement the trait `ergothic::simulation::Sample` for your sample. You will need to implement the following 3 methods:

```rust
trait Sample {
  fn prepare_randomized() -> Self;
  fn thermalize(&mut self) { ... }
  fn mutate(&mut self);
}
```

The meaning of those methods is discussed in what follows.

### Mutation
**Mutation** is the core operation which drives any simulation in ergothic.
Mutation changes your sample by randomizing its degrees of freedom, such that a crucial property called *ergodicity* holds:

> The probability density of observing a system in a particular sample configuration, averaged over time, is equal to the probability density of the statistical ensemble.
> The latter is a parameter of the simulation, and can usually be inferred from the underlying physics.
> For example, in lattice QCD this is the exponential of minus the Euclidean (Wick-rotated) Wilson action.

#### In code
Implement the `mutate` method of the `Sample` trait.
It is absolutely crucial that the algorithm that you are using in `mutate` generates samples with the correct probability density.

### Preparation & thermalization
Typically, recently initialized samples will be highly atypical.
This is because the initialization logic usually doesn't know about the probability density function.
It simply populates the fields of the sample configuration with zeroes or random values.

Getting rid of the initialization bias is known as **thermalization**.
Usually it can be implemented by applying a fixed number (10-20) of mutations to the sample.
However, ergothic lets you implement your own thermalization algorithm.

#### In code
Implement the `prepare_randomized` method of the `Sample` trait.

Optionally, you can implement the `thermalize` method. The default implementation applies `mutate` 20 times.

### Measures
**Measures** are statistical counters corresponding to the physical observables.
The purpose of any ergothic simulation is to establish expectation values and statistical uncertainties for a given list of measures.

#### In code
Create a `MeasureRegistry` and register measures in it. All measures must be given unique human-readable names.

```rust
fn main() {
  let mut reg = ergothic::measure::MeasureRegistry::new();
  let ground_state_energy = reg.register("Energy of the ground state".to_string());
  ...
}
```

### Measurement function
When your simulation runs, on each step you have a sample configuration.
Measuring the values of physical observables of interest and accumulating those values in the statistical counters is done by the measurement function.

#### In code
Pass a lambda to the entry-point function `ergothic::run_simulation`.

```rust
ergothic::run_simulation("My simulation",
  /*takes ownership of measure registry*/ reg, |s: &MyState, ms| {
  // Calculate the values of relevant observables in state `s` and accumulate them in `ms`.
  // Accumulating values is easy: just call
  // ms.accumulate(ground_state_energy, value);
  // where `value` is computed using the sample configuration `s`.
});
```

## Example
Let's put everything together and write a simple simulation.
Our simulation will compute the mean values of `x` and `x^2` where `x` is uniformly distributed within `[0 .. 1]`.

```rust
extern crate ergothic;
extern crate rand;

struct MySample {
  x: f64,  // Random variable within [0 .. 1].
}

impl ergothic::simulation::Sample for MySample {
  fn prepare_randomized() -> MySample {
    MySample{x: rand::random()}
  }
  
  fn mutate(&mut self) {
    self.x = rand::random();
  }
}

fn main() {
  let mut reg = ergothic::measure::MeasureRegistry::new();
  let x = reg.register("Mean X".to_string());  // Mean value of the random variable x.
  let x2 = reg.register("Mean X^2".to_string());  // Mean value of the square of x.
  ergothic::run_simulation("Computing expectations of random variable and its square",
    reg, |s: &MySample, ms| {
    ms.accumulate(x, s.x);  // Accumulate the value of x in the statistical counter for the corresponding measure.
    ms.accumulate(x2, s.x.powi(2));  // Accumulate the value of x^2 in the statistical counter for the corresponding measure.
  });
}
```

That's it! That simple code is fully functional, and it can run on clusters with tens of thousands of nodes, too!

### Running the example
To run the example in debug mode, simply run the following command:

```
cargo run
```

That's it!

### Example output
In debug mode, an ergothic simulation will output a table of measured values each 2 seconds.
Here's a table for our example from above:

```
Simulation uptime: 5 secs
Samples processed: 4839379
Aggregate values:
+----------+--------------------+------------------------+----------------------+
| MEASURE  |    EXPECTATION     |      UNCERTAINTY       | RELATIVE UNCERTAINTY |
+----------+--------------------+------------------------+----------------------+
|   Mean X | 0.4999631317520213 | 0.00013121218432651647 | 0.026244372033335638 |
| Mean X^2 | 0.3332809661876769 | 0.0001355082806320386  | 0.04065887175679013  |
+----------+--------------------+------------------------+----------------------+
```

We see that the expectations of X and X^2 are what we would expect from taking the integrals analytically (1/2 and 1/3 respectively).
Statistical uncertainties are of order 3% and 4% respectively after processing ~5 million samples.

## Scaling up
Now we want to fully take advantage of the huge computational resources which belong to our university / software company / cloud provider / etc.

### Computational model
Ergothic uses the "embarrassingly parallel" computational model.
All nodes participating in the simulation are doing the same thing – producing data points and sending those to the storage service.
This model has many advantages – it is super simple, scales perfectly, doesn't have any bottlenecks because the nodes don't communicate with each other.

You can analyze the intermediate (or final) results of your simulation by querying the database directly using the `ergothic_cli` command line tool.

### Setting up MongoDB
Ergothic needs a data sink where the data points will be exported to.
Currently, the only supported type of data sinks is [MongoDB](https://www.mongodb.com/), though implementing other exporters should be easy and is among the next goals for ergothic.

### Shipping the simulation
Build the optimized version of same code with

```
cargo build --release
```

Now you need a way to distribute the binary to the cluster nodes and run them.
For example, you can bundle the binary into a Docker container, and orchestrate the computation with [Kubernetes](https://kubernetes.io/).

Run the containers with the following command line arguments:

```
./my_simulation --production --mongo mongodb://hostname1:port2[,hostname2:port2,...] --mongo_db ergothic_data --mongo_coll my_simulation
```

Where:
* *hostnames* should resolve to the host running MongoDB nodes.
* *ports* should correspond to the exposed MongoDB ports.
* *--mongo_db* is the name of the database to send data points to.
* *--mongo_coll* is the name of the collection to send data points to.

In production mode, every node will produce a data point every ~5 min.
Data points will get accumulated in the database.

### Analyzing the results

As the simulation runs, data points are accumulated in the database.
This section describes how to query the database for aggregate values and uncertainties.

**TODO:** after the `ergothic_cli` tool is implemented, explain how to use it to analyze and manipulate the results.

## Remains to be done
Checklist of the most important features that are currently missing from ergothic:

- [ ] Data analyzer tool `ergothic_cli` for querying and aggregating the data points.
- [ ] Multithreaded jobs – ability to scale the simulation into NxM threads where N is the number of nodes and M is the number of threads per node. Threads shouldn't communicate with each other, the computational model remains embarassingly parallel.
- [ ] Exporting to local files and other databases – exporters to other formats.
