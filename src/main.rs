extern crate docopt;

use docopt::Docopt;
use fdname;

const USAGE: &'static str = "
fdname: File and Directory Renaming Tool.

Usage:
  fdname [<dir>] [options] prefix <new>
  fdname [<dir>] [options] suffix <new>
  fdname [<dir>] [options] replace <old> [<new>]
  fdname --help

Options:
  -f      Rename files only.
  -d      Rename directories only.
  -r      Rename in subdirectories/recursive.
  --help  Show this message.
";

fn main() {
    let args: fdname::Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    fdname::run(args).unwrap_or_else(|error| {
        println!("Problem renaming files: {}", error);
        std::process::exit(1);
    });
}
