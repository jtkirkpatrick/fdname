#[macro_use]
extern crate serde_derive;
extern crate walkdir;

use std::env;
use std::ffi;
use std::io;
use std::path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct Args {
    arg_dir: String,
    arg_new: String,
    arg_old: String,
    cmd_prefix: bool,
    cmd_suffix: bool,
    cmd_replace: bool,
    flag_d: bool,
    flag_f: bool,
    flag_r: bool,
    flag_help: bool,
}

struct NameValues<'a> {
    old: &'a str,
    new: &'a str,
}

pub fn run(mut args: Args) -> io::Result<()> {
    // Files and directories default to False. If neither is specified,
    // then both will be used -- i.e., -df is the default mode.
    if !(args.flag_d || args.flag_f) {
        args.flag_d = true;
        args.flag_f = true;
    }

    let name_values = NameValues {
        old: &args.arg_old,
        new: &args.arg_new,
    };

    let naming_function: fn(&path::Path, &NameValues) -> io::Result<()> = if args.cmd_prefix {
        prefix
    } else if args.cmd_suffix {
        suffix
    } else if args.cmd_replace {
        replace
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No function selected!",
        ));
    };

    let root = root_dir(&args)?;

    // min_depth(1) excludes the root directory itself.
    // contents_first(true) is needed to allow renaming and recursion to work.
    let mut walker = WalkDir::new(root).min_depth(1).contents_first(true);

    // If the recursion flag is not set, max_depth(1) is what we want.
    if !args.flag_r {
        walker = walker.max_depth(1);
    };

    for entry in walker {
        let entry_path = entry?;
        let entry_path = entry_path.path();

        if args.flag_d && entry_path.is_dir() {
            naming_function(entry_path, &name_values)?;
        } else if args.flag_f && entry_path.is_file() {
            naming_function(entry_path, &name_values)?;
        } else {
            continue;
        }
    }

    Ok(())
}

fn root_dir(args: &Args) -> io::Result<(path::PathBuf)> {
    let root: path::PathBuf = if args.arg_dir.as_str() != "" {
        path::PathBuf::from(&args.arg_dir)
    } else {
        env::current_dir()?
    };

    if !root.exists() {
        return Err(io::Error::new(io::ErrorKind::Other, "Path does not exist!"));
    } else {
        return Ok(root);
    }
}

fn stem_ext(path: &path::Path) -> Option<(ffi::OsString, ffi::OsString)> {
    let stem = match path.file_stem() {
        Some(stem) => stem,
        None => return None,
    };

    let ext = match path.extension() {
        Some(ext) => ext,
        None => ffi::OsStr::new(""),
    };

    Some((stem.to_os_string(), ext.to_os_string()))
}

fn prefix(path: &path::Path, name_values: &NameValues) -> io::Result<()> {
    let prefix = name_values.new;

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

fn suffix(path: &path::Path, name_values: &NameValues) -> io::Result<()> {
    let suffix = name_values.new;

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

fn replace(path: &path::Path, name_values: &NameValues) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name = stem.to_string_lossy();
    let name = name.replace(name_values.old, name_values.new);
    let name = ffi::OsString::from(name);
    let name = name.as_os_str();

    std::fs::rename(
        path,
        path.with_file_name(name).with_extension(ext).as_path(),
    )
}
