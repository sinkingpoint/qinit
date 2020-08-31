extern crate libq;
extern crate clap;
extern crate serde_json;
extern crate patient;

use libq::netlink::NetLinkSocket;
use libq::logger;

use std::collections::{HashMap, VecDeque};
use std::io::{self, BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::fs::{read_dir, read_link, File};

use patient::FreudianClient;

use clap::{App};

const FREUDIAN_TOPIC_NAME: &str = "uevents";
const ADD_COMMAND: &str = "add\n";

fn init_freudian() -> Result<FreudianClient, io::Error> {
    let mut freud_socket = FreudianClient::new(Path::new("/run/freudian/socket"))?;
    freud_socket.create_topic(FREUDIAN_TOPIC_NAME)?;

    // Wait until we have at least one subscriber (qdev)
    freud_socket.block_until_subscription(FREUDIAN_TOPIC_NAME, 1)?;

    return Ok(freud_socket);
}

/// do_initial_device_add does the initial device discovery setup - we recursively loop
/// through all of the /sys file system, and echo an "add" into any "uevent" files. This causes
/// the kernel to re-emit the uevent so that we can reprocess it
fn do_initial_device_add() -> Result<(), io::Error> {
    let mut logger = logger::with_name_as_json("qdevd;do_initial_device_add");
    logger.set_debug_mode(true);

    // A fifo queue to store directories we're yet to scan
    let mut to_scan = VecDeque::from(vec![PathBuf::from("/sys")]);
    while to_scan.len() > 0 {
        for file in read_dir(to_scan.pop_front().unwrap())? {
            let file = file?;
            let path = file.path();

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
                if let Some(name) = path.file_name() {
                    if let Some(name) = name.to_str() {
                        if name == "uevent" {
                            match File::create(&path) {
                                Ok(mut f) => {
                                    match f.write(&ADD_COMMAND.bytes().collect::<Vec<u8>>()[..]) {
                                        Ok(_) => {},
                                        Err(err) => {
                                            logger.debug().with_string("error", err.to_string()).with_string("path", format!("{}", path.display())).smsg("Failed to trigger uevent");
                                        }
                                    }
                                },
                                Err(err) => {
                                    logger.debug().with_string("error", err.to_string()).with_string("path", format!("{}", path.display())).smsg("Failed to open uevent trigger");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    return Ok(());
}

fn event_loop<T: BufRead>(mut freud_socket: FreudianClient, event_reader: T) {
    let logger = logger::with_name_as_json("qdevd");
    let mut current_event = HashMap::new();
    // uevent lines are in the format a=b\0, except for the first one which provides a summary (and doesn't have an =)
    for line in event_reader.split(0) {
        let line = match line {
            Ok(line) => String::from_utf8(line).unwrap(),
            Err(err) => {
                logger.info().with_string("error", err.to_string()).smsg("Got an IO Error reading Netlink socket. Bailing");
                break;
            }
        };

        // If there isn't an =, then we emit the event into freudian
        if !line.contains("=") {
            if current_event.len() > 0 {
                // Dispatch the old event
                let event_str = match serde_json::to_string(&current_event) {
                    Ok(s) => s,
                    Err(err) => {
                        logger.info().with_string("error", err.to_string()).smsg("Failed serialize message");
                        current_event.clear();
                        continue;
                    }
                };

                match freud_socket.produce_message(FREUDIAN_TOPIC_NAME, event_str.bytes().collect()) {
                    Ok(_) => {},
                    Err(err) => {
                        logger.info().with_string("error", err.to_string()).smsg("Failed to produce to Freudian");
                        break;
                    }
                }

                current_event.clear();
            }
        }
        else {
            // Otherwise, we split the event into a k=v parts and insert into the map
            let parts_iter: Vec<String> = line.splitn(2, "=").map(|s| s.to_owned()).collect();
            current_event.insert(parts_iter[0].clone(), parts_iter[1].clone());
        }
    }
}

fn main() {
    App::new("qdevd")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("A UDev Daemon which pipes events into Freudian")
        .get_matches();

    let logger = logger::with_name_as_json("qdevd");
    logger.info().smsg("Starting QDevD");
    let socket = NetLinkSocket::new().unwrap();
    let reader = BufReader::new(socket);
    let freud_socket = match init_freudian() {
        Ok(socket) => socket,
        Err(e) => {
            logger.info().with_string("error", e.to_string()).smsg("Failed to open Freudian connection");
            return;
        }
    };

    let event_loop_thread = std::thread::spawn(move || event_loop(freud_socket, reader));

    match do_initial_device_add() {
        Ok(_) => {},
        Err(err) => {
            logger.info().with_string("error", err.to_string()).smsg("Failed to do initial device discovery");
        }
    }

    event_loop_thread.join().unwrap();
}