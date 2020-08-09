use std::{env, ffi, io};
use std::path::{ Path, PathBuf };
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
#[structopt(name = "fdname", about = "File and Directory Renaming Tool")]
pub struct Opt {
    #[structopt(parse(from_os_str), help = "Optional root directory.")]
    root_dir: Option<PathBuf>,
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
    #[structopt(name="hash", help="Completely hash each name.")]
    Hash {},
    #[structopt(name="lowercase", help="Change name to all lowercase characters.")]
    Lowercase {},
    #[structopt(name = "prefix")]
    Prefix {
        #[structopt(help = "Add <new> to the front of every entry.")]
        new: String,
    },
    #[structopt(name="remove", help="Remove <sub> from names.")]
    Remove {
        #[structopt(help="Remove <sub> from names.")]
        sub: String,
    },
    #[structopt(name = "replace")]
    Replace {
        #[structopt(help = "Remove <old> from every entry.")]
        old: String,
        #[structopt(help = "Insert <new> where <old> was in each entry. Leave blank to remove <old> only.")]
        new: String,
    },
    #[structopt(name = "suffix")]
    Suffix {
        #[structopt(help = "Add <new> to the end of every entry.")]
        new: String,
    },
    #[structopt(name="uppercase", help="Change name to all uppercase characters.")]
    Uppercase {},
    #[structopt(name="whitespace", help="Remove all white space in names.")]
    Whitespace {},
}

pub fn run(mut opt: Opt) -> io::Result<()> {
    // Files and directories default to False. If neither is specified,
    // then both will be used -- i.e., -df is the default mode.
    if !(opt.dirs || opt.files) {
        opt.dirs = true;
        opt.files = true;
    }

    let naming_function = |path: &Path| match &opt.cmd {
        Command::Hash {} => hash(path),
        Command::Lowercase {} => lowercase(path),
        Command::Prefix { new } => prefix(path, &new),
        Command::Remove { sub } => remove(path, &sub),
        Command::Replace { old, new } => replace(path, &old, &new),
        Command::Suffix { new } => suffix(path, &new),
        Command::Uppercase {} => uppercase(path),
        Command::Whitespace {} => whitespace(path),
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

fn root_dir(opt: &Opt) -> io::Result<PathBuf> {
    let root: PathBuf = match &opt.root_dir {
        Some(path) => path.clone(),
        None => env::current_dir()?,
    };

    if !root.exists() {
        return Err(io::Error::new(io::ErrorKind::Other, "Path does not exist!"));
    } else {
        return Ok(root);
    }
}

fn stem_ext(path: &Path) -> Option<(ffi::OsString, ffi::OsString)> {
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

fn hash(path: &Path) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };
    
    let mut hasher = DefaultHasher::new();
    stem.hash(&mut hasher);

    let name = hasher.finish();
    let name = ffi::OsString::from(name.to_string());
    let name = name.as_os_str();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn lowercase(path: &Path) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name: String = stem.into_string().unwrap().to_lowercase();
    let name = Path::new(&name[..]).file_stem().unwrap();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn prefix(path: &Path, prefix: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let mut name = ffi::OsString::from(prefix);
    name.push(stem);
    let name = name.as_os_str();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn remove(path: &Path, sub: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name = stem.to_string_lossy();
    let name = name.replace(sub, "");
    let name = ffi::OsString::from(name);
    let name = name.as_os_str();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn replace(path: &Path, old: &str, new: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name = stem.to_string_lossy();
    let name = name.replace(old, new);
    let name = ffi::OsString::from(name);
    let name = name.as_os_str();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn suffix(path: &Path, suffix: &str) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let mut name = ffi::OsString::from(stem);
    name.push(suffix);
    let name = name.as_os_str();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn uppercase(path: &Path) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name: String = stem.into_string().unwrap().to_uppercase();
    let name = Path::new(&name[..]).file_stem().unwrap();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}

fn whitespace(path: &Path) -> io::Result<()> {
    let (stem, ext) = match stem_ext(path) {
        Some((stem, ext)) => (stem, ext),
        None => return Ok(()),
    };

    let name: String = stem.into_string().unwrap().chars().filter(|c| !c.is_whitespace()).collect();
    let name = Path::new(&name[..]).file_stem().unwrap();

    std::fs::rename(path, path.with_file_name(name).with_extension(ext).as_path())
}