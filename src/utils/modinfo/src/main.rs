extern crate clap;
extern crate libq;
extern crate lzma_rs;

use std::char;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::exit;

use clap::{App, Arg};
use libq::elf::ElfBinary;
use libq::io::BufferReader;
use libq::logger;
use lzma_rs::xz_decompress;

fn main() {
    let args = App::new("modinfo")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Prints information about a given Linux Kernel module")
        .arg(
            Arg::with_name("field")
                .short("F")
                .long("field")
                .takes_value(true)
                .help("Only print this field, one per line"),
        )
        .arg(Arg::with_name("file").index(1).help("The module to read").required(true))
        .get_matches();

    let path = Path::new(args.value_of("file").unwrap());
    let logger = logger::with_name_as_console("readelf");
    if !path.exists() {
        logger
            .info()
            .with_str("path", args.value_of("file").unwrap())
            .smsg("Path doesn't exist");
        exit(1);
    }

    let mut file = match File::open(path) {
        Ok(f) => BufReader::new(f),
        Err(e) => {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .with_string("error", e.to_string())
                .smsg("Failed to open file");
            exit(1);
        }
    };

    let mut buffer: Vec<u8> = Vec::new();
    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("o") | Some("ko") => match file.read_to_end(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    logger
                        .info()
                        .with_str("path", args.value_of("file").unwrap())
                        .with_string("error", format!("{}", e))
                        .smsg("Failed to read file");
                    exit(1);
                }
            },
            Some("xz") => match xz_decompress(&mut file, &mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    logger
                        .info()
                        .with_str("path", args.value_of("file").unwrap())
                        .with_string("error", format!("{:?}", e))
                        .smsg("Failed to open file as xz");
                    exit(1);
                }
            },
            Some(ext) => {
                logger
                    .info()
                    .with_str("path", args.value_of("file").unwrap())
                    .msg(format!("Unrecognised file extension: {}", ext));
                exit(1);
            }
            None => {
                logger
                    .info()
                    .with_str("path", args.value_of("file").unwrap())
                    .smsg("Filename isn't valid Unicode?");
                exit(1);
            }
        },
        None => match file.read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(e) => {
                logger
                    .info()
                    .with_str("path", args.value_of("file").unwrap())
                    .with_string("error", format!("{}", e))
                    .smsg("Failed to read file");
                exit(1);
            }
        },
    };

    let mut reader = BufferReader::new(&buffer);

    let binary = match ElfBinary::read(&mut reader) {
        Ok(bin) => bin,
        Err(e) => {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .with_string("error", e.to_string())
                .smsg("Failed to read file as an ELF binary");
            exit(1);
        }
    };

    let modinfo_section = match binary.read_section(&mut reader, ".modinfo") {
        Ok(Some(section)) => section,
        Ok(None) => {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .smsg("File doesn't have a modinfo section. Is it a Kernel Module?");
            exit(1);
        }
        Err(e) => {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .with_string("error", e.to_string())
                .smsg("Failed to read file");
            exit(1);
        }
    };

    let filtering = args.is_present("field");
    let mut key = String::new();
    let mut value = String::new();
    let mut looking_for_key = true;
    for byte in modinfo_section.into_iter() {
        let new_char = match char::from_u32(byte as u32) {
            Some(c) => c,
            None => {
                logger
                    .info()
                    .with_str("path", args.value_of("file").unwrap())
                    .smsg("Found an invalid character while reading modinfo. Bailing");
                exit(1);
            }
        };

        if new_char == '=' && looking_for_key {
            looking_for_key = false;
        } else if new_char == '\0' && !looking_for_key {
            looking_for_key = true;
            if (filtering && args.value_of("field").unwrap().to_lowercase() == key) || !filtering {
                println!("{}: {}", key, value);
            }

            key.clear();
            value.clear();
        } else if looking_for_key {
            key.push(new_char);
        } else {
            value.push(new_char);
        }
    }
}
