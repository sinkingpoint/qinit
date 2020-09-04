extern crate patient;
extern crate clap;
extern crate libq;
extern crate serde_json;
extern crate regex;
extern crate nix;

use regex::Regex;

use patient::{FreudianClient, Status};

use clap::{App, Arg};

use std::path::Path;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::process::exit;
use std::process::Command;

use libq::logger;
use libq::strings::glob_to_regex;

use nix::sys::utsname::uname;

const FREUDIAN_SOCKET: &str = "/run/freudian/socket";
const FREUDIAN_TOPIC_NAME: &str = "uevents";
struct ModuleLoader {
    mod_aliases: Vec<(Regex, String)>
}

impl ModuleLoader {
    fn new() -> Option<ModuleLoader> {
        let uname = uname();
        let kernel = uname.release();
        let modalias_file = match File::open(format!("/lib/modules/{}/modules.alias", kernel)) {
            Ok(f) => f,
            Err(_) => {
                return None;
            }
        };

        let mut aliases = Vec::new();

        let reader = BufReader::new(modalias_file);
        for line in reader.lines() {
            if !line.is_ok() {
                return None;
            }

            let line = line.unwrap();
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 || parts[0] != "alias" {
                return None;
            }

            let regex = match Regex::new(&glob_to_regex(parts[1])) {
                Ok(r) => r,
                Err(_) => {
                    return None;
                }
            };

            aliases.push((regex, parts[2].to_owned()));
        }

        return Some(ModuleLoader{
            mod_aliases: aliases
        });
    }

    fn get_module_for(&self, mod_alias: &str) -> Option<&String> {
        for (regex, mod_name) in self.mod_aliases.iter() {
            if regex.is_match(&mod_alias) {
                return Some(mod_name);
            }
        }

        return None;
    }
}

fn main() {
    let args = App::new("qdev")
    .version("0.1")
    .author("Colin D. <colin@quirl.co.nz>")
    .about("Processes uevents, runs rules, and load modules")
    .arg(
        Arg::with_name("socket")
            .short("s")
            .takes_value(true)
            .help("Override the location of the freudian socket to connect to"),
    )
    .get_matches();

    let freudian_socket_file = args.value_of("socket").unwrap_or(FREUDIAN_SOCKET);

    let mut client = FreudianClient::new(Path::new(freudian_socket_file)).unwrap();

    let logger = logger::with_name_as_json("qdev");

    let module_loader = match ModuleLoader::new() {
        Some(m) => m,
        None => {
            logger.info().smsg("Failed to load modules.aliases file for this kernel release, bailing");
            exit(1);
        }
    };

    let sub_id = match client.subscribe(FREUDIAN_TOPIC_NAME) {
        Ok((Some(uuid), _)) => uuid,
        Ok((None, _)) => {
            logger.info().smsg("Failed to subscribe to Freudian");
            return;
        },
        Err(err) => {
            logger.info().with_string("error", err.to_string()).smsg("Failed to talk to Freudian");
            std::process::exit(1);
        }
    };

    loop {
        // TODO: Spawn worker threads for each message rather than handling them sequentially
        let message = match client.consume_message(&sub_id, 30) {
            Ok(resp) => {
                if resp.response_type != Status::Ok {
                    if resp.response_type != Status::NothingHappened {
                        break;
                    }

                    continue;
                }

                if let Ok(message) = String::from_utf8(resp.message) {
                    message
                }
                else {
                    logger.info().smsg("Invalid message from qdevd");
                    std::process::exit(2);
                }
            },
            Err(err) => {
                logger.info().msg(err.to_string());
                std::process::exit(3);
            }
        };

        let event: HashMap<String, String> = match serde_json::from_str(&message) {
            Ok(e) => e,
            Err(err) => {
                logger.info().with_string("error", err.to_string()).smsg("Invalid message from qdevd");
                std::process::exit(4);
            }
        };

        if let Some(action) = event.get("ACTION") {
            if action == "add" {
                // Load a driver if we haven't already
                if let Some(modalias) = event.get("MODALIAS") {
                    if let Some(modname) = module_loader.get_module_for(modalias) {
                        // We clone modname here so we can move it into the insmod thread
                        let modname = modname.clone();
                        logger.info().with_str("module_name", &modname).smsg("Loading module");
                        std::thread::spawn(move || {
                            let logger = logger::with_name_as_json("qdev;module_load_thread");
                            // TODO: Handle duplicate module loading cleanly (We get an EEXIST at the moment, which is treated as an error)
                            // TODO: Change this to modprobe, once we have such a thing (To handle depdencies)
                            match Command::new("/sbin/insmod").arg(modname).output() {
                                Ok(out) => {
                                    if out.status.code() != Some(0) {
                                        logger.info().smsg("Failed to load module with insmod");
                                    }
                                },
                                Err(e) => {
                                    logger.info().with_string("error", e.to_string()).smsg("Failed to load module with insmod");
                                }
                            };
                        });
                    }
                }
            }
        }
    }
}