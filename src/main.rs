extern crate actix;
extern crate futures;
extern crate env_logger;
#[macro_use] extern crate log;
extern crate rand;
#[macro_use] extern crate structopt;
extern crate time;

use actix::prelude::*;
use futures::{future, Future};
use structopt::StructOpt;
use std::path::PathBuf;

mod args;
mod messages;
mod sorting_actor;
mod supervisor;
mod util;


fn run_system(args: args::CliArgs) {
    let input_path: PathBuf = args.input;
    let n: usize = args.n;
    let k: usize = args.k;
    let output_path: PathBuf = args.output;

    let system = System::new("Sorting system");

    let supervisor = Arbiter::start(move |_| supervisor::SupervisorActor::new("MASTER", n, k));

    let numbers: Vec<i64> = util::read_numbers(input_path).unwrap();
    debug!("Done reading numbers: Vec[{}]", numbers.len());
    let sort_req = messages::SortingRequest::new(numbers);
    let res = supervisor.send(sort_req);

    Arbiter::spawn(
        res.then(|r| {
            match r {
                Ok(r) => {
                    debug!("SortingResponse[Vec[{:?}], Duration: {} (ms)]", r.values.len(), r.duration);

                    if output_path.to_str().unwrap() != "-" {
                        let write_ok = util::write_numbers(output_path, &r.values).unwrap();
                        debug!("Write ok: {:?}", write_ok);
                    }

                    println!("{}", r.duration);
                },
                _ => error!("Error occurred!")
            }

            System::current().stop();
            future::result(Ok(()))
        })
    );

    system.run();
}


fn setup_logger(args: &args::CliArgs) {
    let log_level = match args.debug {
        true => "debug",
        false => "info"
    };

    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();
}


fn main() {
    let args = args::CliArgs::from_args();
    setup_logger(&args);
    run_system(args);
}
