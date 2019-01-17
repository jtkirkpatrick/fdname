extern crate structopt;
extern crate walkdir;

use std::env;
use std::ffi;
use std::io;
use std::path;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
#[structopt(name = "fdname", about = "File and Directory Renaming Tool")]
pub struct Opt {
    #[structopt(parse(from_os_str), help = "Optional root directory.")]
    root_dir: Option<path::PathBuf>,
    #[structopt(name = "dirs", long, short, help = "Apply to directories.")]
    dirs: bool,
    #[structopt(name = "files", long, short, help = "Apply to files.")]
    files: bool,
    #[structopt(name = "recursive", long, short, help = "Apply to the tree.")]
    recursive: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "prefix")]
    Prefix {
        #[structopt(help = "Add <new> to the front of every entry.")]
        new: String,
    },
    #[structopt(name = "suffix")]
    Suffix {
        #[structopt(help = "Add <new> to the end of every entry.")]
        new: String,
    },
    #[structopt(name = "replace")]
    Replace {
        #[structopt(help = "Remove <old> from every entry.")]
        old: String,
        #[structopt(help = "Insert <new> where <old> was in each entry.")]
        new: String,
    },
}

pub fn run(mut opt: Opt) -> io::Result<()> {
    // Files and directories default to False. If neither is specified,
    // then both will be used -- i.e., -df is the default mode.
    if !(opt.dirs || opt.files) {
        opt.dirs = true;
        opt.files = true;
    }

    let naming_function = |path: &path::Path| match &opt.cmd {
        Command::Prefix { new } => prefix(path, &new),
        Command::Suffix { new } => suffix(path, &new),
        Command::Replace { old, new } => replace(path, &old, &new),
    };

    let root = root_dir(&opt)?;

    // min_depth(1) excludes the root directory itself.
    // contents_first(true) is needed to allow renaming and recursion to work.
    let mut walker = WalkDir::new(root).min_depth(1).contents_first(true);

    // If the recursion flag is not set, max_depth(1) is what we want.
    if !opt.recursive {
        walker = walker.max_depth(1);
    };

    for entry in walker {
        let entry = entry?;
        let entry_path = entry.path();

        if opt.dirs && entry_path.is_dir() {
            naming_function(entry_path)?;
        } else if opt.files && entry_path.is_file() {
            naming_function(entry_path)?;
        } else {
            continue;
        }
    }

    Ok(())
}

fn root_dir(opt: &Opt) -> io::Result<(path::PathBuf)> {
    let root: path::PathBuf = match &opt.root_dir {
        Some(path) => path.clone(),
        None => env::current_dir()?,
    };

    if !root.exists() {
        return Err(io::Error::new(io::ErrorKind::Other, "Path does not exist!"));
    } else {
        return Ok(root);
    }
}

fn stem_ext(path: &path::Path) -> Option<(ffi::OsString, ffi::OsString)> {
    let stem: &ffi::OsStr = match path.file_stem() {
        Some(stem) => stem,
        None => return None,
    };

    let ext = match path.extension() {
        Some(ext) => ext,
        None => ffi::OsStr::new(""),
    };

    Some((stem.to_os_string(), ext.to_os_string()))
}

fn prefix(path: &path::Path, prefix: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let mut name = ffi::OsString::from(prefix);
    name.push(stem);
    let name = name.as_os_str();

    std::fs::rename(
        path,
        path.with_file_name(name).with_extension(ext).as_path(),
    )
}

fn suffix(path: &path::Path, suffix: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let mut name = ffi::OsString::from(stem);
    name.push(suffix);
    let name = name.as_os_str();

    std::fs::rename(
        path,
        path.with_file_name(name).with_extension(ext).as_path(),
    )
}

fn replace(path: &path::Path, old: &str, new: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name = stem.to_string_lossy();
    let name = name.replace(old, new);
    let name = ffi::OsString::from(name);
    let name = name.as_os_str();

    std::fs::rename(
        path,
        path.with_file_name(name).with_extension(ext).as_path(),
    )
}
