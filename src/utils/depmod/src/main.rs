extern crate libq;
extern crate clap;
extern crate lzma_rs;
extern crate nix;

use std::path::{Path, PathBuf};
use std::fs::{File, read_link, read_dir};
use std::io::{self, Read, BufReader, Seek, Write};
use std::collections::{HashMap, VecDeque};
use std::process::exit;

use clap::{App, Arg};
use libq::elf::{ElfBinary, SymType, SymBinding};
use libq::logger;
use libq::io::BufferReader;
use libq::iter::SplitOn;
use lzma_rs::xz_decompress;
use nix::sys::utsname::uname;

#[derive(Default, Debug)]
struct ModInfo {
    name: String,
    version: String,
    author: String,
    description: String,
    license: String,
    src_version: String,
    parameter_descriptions: HashMap<String, String>,
    parameter_types: HashMap<String, String>,
    firmware: Vec<String>,
    aliases: Vec<String>,
    dependencies: Vec<String>,
    return_trampoline: bool,
    in_tree: bool,
    version_magic: String
}

impl ModInfo {
    fn read<T: Seek + Read>(binary: &ElfBinary, reader: &mut T) -> Option<ModInfo> {
        let logger = logger::with_name_as_console("depmod;modinfo;read");
        let modinfo_section = match binary.read_section(reader, ".modinfo") {
            Ok(Some(section)) => section,
            Ok(None) => {
                logger.info().smsg("File doesn't have a modinfo section. Is it a Kernel Module?");
                return None;
            },
            Err(e) => {
                logger.info().with_string("error", e.to_string()).smsg("Failed to read file");
                return None;
            }
        };

        let mut modinfo = ModInfo::default();

        for line in modinfo_section.into_iter().split_on_exclusive(0).map(|line| String::from_utf8(line)) {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    logger.info().with_string("error", e.to_string()).smsg("Failed to parse as UTF-8");
                    return None;
                }
            };

            let parts: Vec<&str> = line.splitn(2, "=").collect();
            if parts.len() != 2 {
                logger.info().with_string("line", line).smsg("Line missing =, failed to parse as modinfo line");
                return None;
            }

            let key = parts[0];
            let value = parts[1];

            match key {
                "name" => modinfo.name = value.to_owned(),
                "vermagic" => modinfo.version_magic = value.to_owned(),
                "intree" => modinfo.in_tree = value == "Y",
                "retpoline" => modinfo.return_trampoline = value == "Y",
                "srcversion" => modinfo.src_version = value.to_owned(),
                "author" => modinfo.author = value.to_owned(),
                "description" => modinfo.description = value.to_owned(),
                "version" => modinfo.version = value.to_owned(),
                "license" => modinfo.license = value.to_owned(),
                "depends" | "alias" => {
                    if value.trim().len() != 0 {
                        if key == "depends" {
                            modinfo.dependencies.push(value.to_owned())
                        }
                        else if key == "alias" {
                            modinfo.aliases.push(value.to_owned())
                        }
                    }
                },
                "parm" | "parmtype" => {
                    let parts: Vec<&str> = value.trim().splitn(2, ":").collect();
                    if parts.len() != 2 {
                        logger.info().with_string("line", line).smsg("Line missing :, failed to parse as parm line");
                        return None;
                    }

                    if key == "parm" {
                        modinfo.parameter_descriptions.insert(parts[0].to_owned(), parts[1].to_owned());
                    }
                    else if key == "parmtype" {
                        modinfo.parameter_types.insert(parts[0].to_owned(), parts[1].to_owned());
                    }
                },
                _ => {
                    println!("Unhandled key: {}", key);
                }
            }
        }

        return Some(modinfo);
    }
}

fn write_aliases<W: Write>(modinfo: &ModInfo, writer: &mut W) {
    for alias in modinfo.aliases.iter() {
        writer.write_all(&format!("alias {} {}\n", alias, modinfo.name).bytes().collect::<Vec<u8>>()[..]);
    }
}

fn write_symbols<T: Seek + Read, W: Write>(binary: &ElfBinary, name: &str, reader: &mut T, writer: &mut W) {
    let logger = logger::with_name_as_console("depmod;write_symbols");
    let symbols = match binary.read_symbols(reader, ".symtab") {
        Ok(Some(symbols)) => symbols,
        _ => {
            logger.info().smsg("There are no symbols in this file");
            return;
        }
    };

    for sym in symbols.iter() {
        let binding = match sym.get_binding() {
            Ok(b) => b,
            Err(_) => {
                continue;
            }
        };

        let sym_type = match sym.get_type() {
            Ok(b) => b,
            Err(_) => {
                continue;
            }
        };

        if sym_type == SymType::Function && binding == SymBinding::Global {
            writer.write_all(&format!("alias symbol:{} {}\n", sym.name, name).bytes().collect::<Vec<u8>>()[..]);
        }
    }
}

enum LoadFileError {
    IOError(io::Error),
    UnzipError(lzma_rs::error::Error),
    UnknownFileExtention,
}

impl LoadFileError {
    fn to_string(&self) -> String {
        match self {
            LoadFileError::IOError(err) => err.to_string(),
            LoadFileError::UnzipError(_) => "Failed to load file as zipped".to_owned(),
            LoadFileError::UnknownFileExtention => "Unknown File Extention".to_owned()
        }
    }
}

impl From<io::Error> for LoadFileError {
    fn from(err: io::Error) -> Self {
        return LoadFileError::IOError(err);
    }
}

impl From<lzma_rs::error::Error> for LoadFileError {
    fn from(err: lzma_rs::error::Error) -> Self {
        return LoadFileError::UnzipError(err);
    }
}

fn load_file(path: &Path) -> Result<Vec<u8>, LoadFileError> {
    // TODO: Move this "Read a might be xz'd file" into libq
    let mut file = BufReader::new(File::open(path)?);
    let mut buffer: Vec<u8> = Vec::new();
    match path.extension() {
        Some(ext) => {
            match ext.to_str() {
                Some("o") | Some("ko") => {
                    file.read_to_end(&mut buffer)?;
                },
                Some("xz") => {
                    xz_decompress(&mut file, &mut buffer)?;
                },
                Some(_) | None => {
                    return Err(LoadFileError::UnknownFileExtention);
                }
            }
        },
        None => {
            file.read_to_end(&mut buffer)?;
        }
    }

    return Ok(buffer);
}

fn main() {
    let args = App::new("depmod")
    .version("0.1")
    .author("Colin D. <colin@quirl.co.nz>")
    .about("Generates module.dep and module.alias files for consumption by module loading things")
    .arg(Arg::with_name("dryrun").short("d").help("Only print out the files, rather than writing them"))
    .arg(Arg::with_name("kernel").short("k").help("The kernel version to generate module values for"))
    .get_matches();

    let uname = uname();
    let kernel = args.value_of("kernel").unwrap_or(uname.release());
    let dryrun = args.is_present("dryrun");

    let logger = logger::with_name_as_console("depmod");

    let mut aliases_out: Box<dyn Write>;
    let mut symbols_out: Box<dyn Write>;
    let mut names_out: Box<dyn Write>;

    if dryrun {
        aliases_out = Box::new(std::io::stdout());
        symbols_out = Box::new(std::io::stdout());
        names_out    = Box::new(std::io::stdout());
    }
    else {
        aliases_out = match File::create(format!("/lib/modules/{}/modules.alias", kernel)) {
            Ok(f) => Box::new(f),
            Err(e) => {
                logger.info().with_string("error", e.to_string()).smsg("failed to open aliases file");
                exit(1);
            }
        };

        symbols_out = match File::create(format!("/lib/modules/{}/modules.symbols", kernel)) {
            Ok(f) => Box::new(f),
            Err(e) => {
                logger.info().with_string("error", e.to_string()).smsg("failed to open symbols file");
                exit(1);
            }
        };

        names_out   = match File::create(format!("/lib/modules/{}/modules.names", kernel)) {
            Ok(f) => Box::new(f),
            Err(e) => {
                logger.info().with_string("error", e.to_string()).smsg("failed to open names file");
                exit(1);
            }
        };
    }

    let mut to_scan: VecDeque<PathBuf> = vec![PathBuf::from(format!("/lib/modules/{}", kernel))].into_iter().collect();

    while to_scan.len() > 0 {
        let path = to_scan.pop_front().unwrap();
        let dir_entries = match read_dir(&path) {
            Ok(e) => e,
            Err(e) => {
                println!("{}", &path.display());
                logger.info().with_string("error", e.to_string()).smsg("failed to read diectory");
                continue;
            }
        };
        let path_str = path.to_str().unwrap();

        for file in dir_entries {
            let file = match file {
                Ok(f) => f,
                Err(e) => {
                    logger.info().with_str("path", path_str).with_string("error", e.to_string()).smsg("failed to read file");
                    continue;
                }
            };

            let path = file.path();
            let path_str = path.to_str().unwrap();

            if path_str.contains("modules.") {
                continue;
            }

            // If it's a symlink, skip it so we don't process things more than once
            match read_link(&path) {
                Ok(_) => {
                    continue;
                }
                Err(_) => {}
            }

            // If it's a directory, add it to the list
            if path.is_dir() {
                to_scan.push_back(path);
                continue;
            }

            if path.is_file() {
                let buffer = match load_file(&path) {
                    Ok(buf) => buf,
                    Err(err) => {
                        logger.info().with_string("error", err.to_string()).with_str("path", path_str).smsg("Failed to load file");
                        continue;
                    }
                };

                let mut reader = BufferReader::new(&buffer);

                let elf_binary = match ElfBinary::read(&mut reader) {
                    Ok(bin) => bin,
                    Err(err) => {
                        logger.info().with_string("error", err.to_string()).with_str("path", path_str).smsg("Failed to load file as ELF Binary");
                        continue;
                    }
                };

                let mod_info = match ModInfo::read(&elf_binary, &mut reader) {
                    Some(info) => info,
                    None => {
                        logger.info().with_str("path", path_str).smsg("Failed to load file as Kernel Module");
                        continue;
                    }
                };

                write_aliases(&mod_info, &mut aliases_out);
                write_symbols(&elf_binary, &mod_info.name, &mut reader, &mut symbols_out);
                names_out.write_all(&format!("{}: {}", mod_info.name, path_str).bytes().collect::<Vec<u8>>());
            }
        }
    }
}