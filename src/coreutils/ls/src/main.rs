extern crate chrono;
extern crate clap;
extern crate libq;
extern crate nix;

use libq::io::FileType;
use libq::passwd;
use libq::strings::format_file_mode;

use clap::{App, Arg};
use nix::fcntl::readlink;
use nix::sys::stat::{lstat, FileStat};
use std::path::{Component, PathBuf};

use chrono::{DateTime, Datelike, Local};

use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::process;
use std::time::{Duration, UNIX_EPOCH};

enum LsAllMode {
    None,
    AlmostAll,
    All,
}

impl LsAllMode {
    fn from_flags(all: bool, almost_all: bool) -> LsAllMode {
        if all {
            return LsAllMode::All;
        } else if almost_all {
            return LsAllMode::AlmostAll;
        }

        return LsAllMode::None;
    }

    fn accept(&self, p: &PathBuf) -> bool {
        if let Some(name) = p.components().last() {
            return match self {
                LsAllMode::All => true,
                LsAllMode::AlmostAll => name != Component::CurDir && name != Component::ParentDir,
                LsAllMode::None => match name {
                    Component::Prefix(_) | Component::CurDir | Component::ParentDir => false,
                    Component::RootDir => true,
                    Component::Normal(name) => !name.to_string_lossy().starts_with("."),
                },
            };
        }

        return false;
    }
}

fn get_children_sorted(path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut children: Vec<PathBuf> = fs::read_dir(path)?
        .map(|c| c.expect("ls: failed to convert child into path").path())
        .collect();
    children.sort_by(|a, b| {
        a.file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
            .cmp(&b.file_name().unwrap().to_string_lossy().to_lowercase())
    });
    return Ok(children);
}

fn format_file_name(entry: &LsResult) -> String {
    let full_path_str = entry.path.to_string_lossy();
    if full_path_str == "." || full_path_str == ".." {
        return format!("{} ", full_path_str);
    } else if entry.in_dir {
        return format!("{} ", entry.path.file_name().unwrap().to_string_lossy());
    }
    return format!("{} ", entry.path.display());
}

struct LsMode {
    long: bool,
    all_mode: LsAllMode,
    recursive: bool,
    directory: bool,
}

struct LsIter {
    recurse: bool,
    directory: bool,
    current_directory: VecDeque<PathBuf>,
    to_process_directories: VecDeque<PathBuf>,
}

struct LsResult {
    path: PathBuf,
    is_new_dir: bool,
    stat_result: FileStat,
    file_type: FileType,
    in_dir: bool,
}

impl LsResult {
    fn print(&self, mode: &LsMode, user_map: &HashMap<u32, String>, group_map: &HashMap<u32, String>) {
        if self.is_new_dir && mode.recursive {
            println!("{}: ", self.path.display());
            return;
        }

        if !mode.long {
            print!("{} ", format_file_name(self));
        } else {
            let sr = &self.stat_result;
            let d = UNIX_EPOCH + Duration::from_secs(sr.st_ctime as u64);
            let datetime = DateTime::<Local>::from(d);
            let format_str: &str;
            if datetime.year() == Local::now().year() {
                format_str = "%b %d %H:%M";
            } else {
                format_str = "%b %d %Y";
            }
            let timestamp_str = datetime.format(format_str).to_string();
            let mut file_name = format_file_name(self);
            if self.file_type == FileType::Link && mode.long {
                file_name.push_str(&format!("-> {}", readlink(&self.path).unwrap().to_string_lossy()));
            }

            let uid_str = format!("{}", sr.st_uid);
            let gid_str = format!("{}", sr.st_gid);
            let user = user_map.get(&sr.st_uid).unwrap_or(&uid_str);
            let group = group_map.get(&sr.st_gid).unwrap_or(&gid_str);
            println!(
                "{}. {} {} {} {} {} {}",
                format_file_mode(sr.st_mode),
                sr.st_nlink,
                user,
                group,
                sr.st_size,
                timestamp_str,
                file_name
            );
        }
    }
}

impl LsIter {
    fn new(base: &PathBuf, recurse: bool, directory: bool) -> LsIter {
        let mut to_process_directories = VecDeque::new();
        let mut current_directory = VecDeque::new();
        if base.is_dir() {
            to_process_directories.push_back(base.clone());
        } else {
            current_directory.push_back(base.clone());
        }
        return LsIter {
            recurse: recurse,
            directory: directory,
            current_directory: current_directory,
            to_process_directories: to_process_directories,
        };
    }
}

impl Iterator for LsIter {
    type Item = LsResult;
    fn next(&mut self) -> Option<LsResult> {
        if self.current_directory.len() == 0 {
            if self.to_process_directories.len() == 0 {
                return None;
            }

            let new_dir_path = self.to_process_directories.pop_front().unwrap();
            if !self.directory {
                self.current_directory = vec![PathBuf::from("."), PathBuf::from("..")].into_iter().collect();
                self.current_directory
                    .append(&mut get_children_sorted(&new_dir_path).unwrap().into_iter().collect());
            }

            return Some(LsResult {
                stat_result: lstat(&new_dir_path).unwrap(),
                path: new_dir_path,
                file_type: FileType::Directory,
                is_new_dir: true,
                in_dir: true,
            });
        }

        let mut new_entry = self.current_directory.pop_front().unwrap();
        while !new_entry.exists() {
            // There's a TOCTOU bug here - if the file is deleted in between us seeing it in `readdir` and here we would panic in the stat call
            // This prevents that by going through files until we hit an undeleted one. Yes, we do some extra processing + stat call but meh
            eprintln!("ls: {} no such file or directory", new_entry.display());
            new_entry = match self.next() {
                Some(entry) => entry.path,
                None => return None,
            };
        }
        let stat_result = lstat(&new_entry).unwrap();
        let entrytype = FileType::from_stat(stat_result).unwrap();
        let file_name = new_entry.to_string_lossy();

        if entrytype == FileType::Directory && self.recurse && file_name != "." && file_name != ".." {
            self.to_process_directories.push_back(new_entry.clone());
        }

        return Some(LsResult {
            path: new_entry,
            stat_result: stat_result,
            file_type: entrytype,
            is_new_dir: false,
            in_dir: true,
        });
    }
}

fn do_ls(path: &PathBuf, mode: &LsMode, user_map: &HashMap<u32, String>, group_map: &HashMap<u32, String>) -> Result<(), ()> {
    let ls_iter = LsIter::new(path, mode.recursive, mode.directory);

    let mut printed_something = false;
    for result in ls_iter {
        if result.is_new_dir && printed_something {
            // Print a seperator
            println!("\n");
        } else if result.is_new_dir && !mode.recursive && !mode.directory {
            // Skip the name of the dir if we're not recursing
            continue;
        }

        // Second condition overrides the -a parameter if we're printing the current dir in directory mode
        if mode.all_mode.accept(&result.path) || (!printed_something && mode.directory) {
            result.print(mode, user_map, group_map);
            printed_something = true;
        }
    }

    if !mode.long && printed_something {
        print!("\n");
    }

    return match printed_something {
        true => Ok(()),
        false => Err(()),
    };
}

fn main() {
    let args = App::new("ls")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Create the DIRECTORY(ies), if they do not already exist.")
        .arg(Arg::with_name("long").short("l").help("Use long listing format"))
        .arg(
            Arg::with_name("almost_all")
                .short("A")
                .long("almost_all")
                .help("show hidden files, except . and .."),
        )
        .arg(Arg::with_name("all").short("a").long("all").help("show hidden files"))
        .arg(
            Arg::with_name("recursive")
                .short("R")
                .long("recursive")
                .help("list subdirectories recursively"),
        )
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .help("list directories themselves, not their contents"),
        )
        .arg(Arg::with_name("file").takes_value(true).multiple(true).index(1))
        .get_matches();

    let files: Vec<PathBuf> = match args.values_of("file") {
        Some(f) => f.collect::<Vec<&str>>().iter().map(|f| PathBuf::from(f)).collect(),
        None => vec![PathBuf::from(".")],
    };

    let mode = LsMode {
        long: args.is_present("long"),
        all_mode: LsAllMode::from_flags(args.is_present("all"), args.is_present("almost_all")),
        recursive: args.is_present("recursive"),
        directory: args.is_present("directory"),
    };

    let mut uid_map = HashMap::new();
    for user in passwd::Users::new() {
        uid_map.insert(user.uid, user.username);
    }

    let mut gid_map = HashMap::new();
    for group in passwd::Groups::new() {
        gid_map.insert(group.gid, group.name);
    }

    let mut exit_code = 0;
    for file in files.iter() {
        match do_ls(file, &mode, &uid_map, &gid_map) {
            Ok(_) => {}
            Err(_) => {
                exit_code = 1;
            }
        }
    }

    process::exit(exit_code);
}
