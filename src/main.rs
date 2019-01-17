extern crate structopt;

use structopt::StructOpt;
use fdname;

fn main() {
    let opt = fdname::Opt::from_args();

    fdname::run(opt).unwrap_or_else(|error| {
        println!("Problem renaming files: {}", error);
        std::process::exit(1);
    });
}
