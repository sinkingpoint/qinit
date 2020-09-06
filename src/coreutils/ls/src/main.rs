extern crate chrono;
extern crate clap;
extern crate libq;
extern crate nix;

use libq::io::FileType;
use libq::passwd;
use libq::strings::format_file_mode;
use libq::terminal::get_window_size;

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

/// Represents the mode of ls we're running (Which files to include)
enum LsAllMode {
    /// Only non hidden files are shown
    None,

    /// All hidden files, except for . and .. are shown
    AlmostAll,

    /// All files are shown
    All,
}

impl LsAllMode {
    /// Construct an LsAllMode based on the flags given
    fn from_flags(all: bool, almost_all: bool) -> LsAllMode {
        if all {
            // If all is set, the result is all (No matter about almost_all)
            return LsAllMode::All;
        } else if almost_all {
            // Otherwise if !all && almost_all, we're doing almost all
            return LsAllMode::AlmostAll;
        }

        // Otherwise we do None
        return LsAllMode::None;
    }

    /// Determins whether the given mode would accept the given path to be printed
    fn accept(&self, p: &PathBuf) -> bool {
        if let Some(name) = p.components().last() {
            return match self {
                // All accepts everything
                LsAllMode::All => true,

                // AlmostAll accepts everything except . and ..
                LsAllMode::AlmostAll => name != Component::CurDir && name != Component::ParentDir,

                // None accepts everything not a hidden file
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

/// Returns a list of paths in a directory (returned by read_dir), alphabetically sorted by name
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

/// Transforms the entry into a String, formatting it based on the context
/// If it's a parent dir (. or ..), then we just return that
/// If we've ls'd inside a dir e.g. `ls dir/`, then we prefix everything with the full path
/// Otherwise (e.g. if we're doing a recursive print), we return the
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
    /// Whether we're doing extended stat output
    long: bool,

    /// The mode to determine which files we're printing
    all_mode: LsAllMode,

    /// Whether we will recurse into the directories we find
    recursive: bool,

    /// Whether we're just printing the directory listing, or the files inside it
    directory: bool,
}

/// An Iterator that allows iterating over a file tree, optionally recursing into directories we find
/// or only giving the given directory back
struct LsIter {
    /// Whethe to recurse into directories we find
    recurse: bool,

    /// Whether to just return the given dir
    directory: bool,

    /// A list of files still to return in the current directory
    current_directory: VecDeque<PathBuf>,

    /// The list of directories we have not yet processed, e.g. if we're recursing
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
    fn to_string(&self, mode: &LsMode, user_map: &HashMap<u32, String>, group_map: &HashMap<u32, String>) -> String {
        if self.is_new_dir && mode.recursive {
            return format!("{}: \n", self.path.display());
        }

        if !mode.long {
            return format!("{} ", format_file_name(self));
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
            return format!(
                "{}. {} {} {} {} {} {}\n",
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

fn do_ls(
    path: &PathBuf,
    mode: &LsMode,
    user_map: &HashMap<u32, String>,
    group_map: &HashMap<u32, String>,
    window_width: usize,
) -> Result<(), ()> {
    let ls_iter = LsIter::new(path, mode.recursive, mode.directory);

    let mut printed_something = false;

    // If we aren't ls'ing in long mode, then we will buffer the results and print them in a grid
    // for each dir we go into
    let mut to_print_buffer = Vec::new();
    for result in ls_iter {
        if result.is_new_dir && printed_something {
            // For every new directory, if we have a buffer
            if to_print_buffer.len() > 0 {
                print_as_grid(&to_print_buffer, window_width);
                to_print_buffer.clear();
            }
            // Print a seperator
            println!("\n");
        } else if result.is_new_dir && !mode.recursive && !mode.directory {
            // Skip the name of the dir if we're not recursing
            continue;
        }

        // Second condition overrides the -a parameter if we're printing the current dir in directory mode
        if mode.all_mode.accept(&result.path) || (!printed_something && mode.directory) {
            if !mode.long && !result.is_new_dir {
                to_print_buffer.push(result.to_string(mode, user_map, group_map))
            } else {
                print!("{}", result.to_string(mode, user_map, group_map));
            }
            printed_something = true;
        }
    }

    if to_print_buffer.len() > 0 {
        print_as_grid(&to_print_buffer, window_width);
        to_print_buffer.clear();
    }

    if !mode.long && printed_something {
        print!("\n");
    }

    return match printed_something {
        true => Ok(()),
        false => Err(()),
    };
}

/// Calculates an optimal grid size to organise the given strings into an (n x m) grid,
/// with each column padded to the maximum width of it's entries, such that no row exceeds
/// some maximum length
fn organise_into_grid(items: &Vec<String>, max_width: usize) -> (usize, usize) {
    let mut num_columns = items.len();
    let mut num_rows = ((items.len() as f64) / (num_columns as f64)).floor() as usize;
    let lengths: Vec<usize> = items.iter().map(|x| x.len()).collect();

    // The width of each row is the maximum width in each column + 2 (spaces for padding) for each column
    // TODO: A Binary search is probably more efficient here
    while {
        let row_width: usize = (0..num_columns)
            .map(|i| lengths[i * num_rows..(i + 1) * num_rows].iter().max().unwrap_or(&0))
            .sum::<usize>()
            + (num_columns - 1) * 2;
        row_width > max_width && num_columns > 1
    } {
        num_columns -= 1;
        num_rows = ((items.len() as f64) / (num_columns as f64)).floor() as usize;
    }

    return (num_columns, num_rows);
}

/// prints the given items as an (n x m) grid, such that no row is greater than max_width characters wide
/// Currently this is arranged in columnular order, i.e. subsequent values go down the column rather than along the row
/// This is useful when formatting directories when there are lots of children
fn print_as_grid(items: &Vec<String>, max_width: usize) {
    // calculate the optimal shape of the grid
    let (ncols, nrows) = organise_into_grid(items, max_width);

    // calculate the lengths of each string, and the resulting size of each column
    let lengths: Vec<usize> = items.iter().map(|x| x.len()).collect();
    let col_widths = (0..ncols)
        .map(|i| *lengths[i * nrows..(i + 1) * nrows].iter().max().unwrap_or(&0))
        .collect::<Vec<usize>>();

    for y in 0..nrows {
        // This gets a bit complicated, so a diagram may be helpful:
        // Consider a grid (where the given number is the index into items)
        // 0 2 4 6
        // 1 3 5 7
        // The following for loop loops over a tuple of the column index, and the items index
        // So for row 0, it would be (0, 0), (1, 2), (2, 4), (3, 6)
        // And for row 1 it would be (0, 1), (1, 3), (2, 5), (3, 7)
        for (x, i) in (y..(ncols * nrows + y - 1)).step_by(nrows).enumerate() {
            print!("{}", items[i]);

            if items[i].len() != col_widths[x] {
                // We need to pad if the item isn't the biggest
                let padding = col_widths[x] - items[i].len();
                print!("{}", (0..padding).map(|_| " ").collect::<String>());
            }

            if x == ncols - 1 {
                print!("\n");
            } else {
                print!("  ");
            }
        }
    }
}

fn main() {
    let args = App::new("ls")
        .version("0.2")
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

    // Get the window width, defaulting to 80 chars wide because 1980 was cool
    let window_width = get_window_size().and_then(|x| Ok(x.ws_col.into())).unwrap_or(80);

    let mut exit_code = 0;
    for file in files.iter() {
        match do_ls(file, &mode, &uid_map, &gid_map, window_width) {
            Ok(_) => {}
            Err(_) => {
                exit_code = 1;
            }
        }
    }

    process::exit(exit_code);
}
