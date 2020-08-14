extern crate libq;
extern crate clap;
extern crate serde_json;
extern crate patient;

use libq::netlink::NetLinkSocket;
use libq::logger;

use std::collections::HashMap;
use std::io::{self, BufReader, BufRead};
use std::path::Path;

use patient::FreudianClient;

use clap::{App, Arg};

const FREUDIAN_TOPIC_NAME: &str = "uevents";

fn init_freudian() -> Result<FreudianClient, io::Error> {
    let mut freud_socket = FreudianClient::new(Path::new("/run/freudian/socket"))?;
    freud_socket.create_topic(FREUDIAN_TOPIC_NAME)?;

    return Ok(freud_socket);
}

fn main() {
    let args = App::new("qdevd")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("A UDev Daemon which pipes events into Freudian")
        .get_matches();

    let logger = logger::with_name_as_json("qdevd");
    logger.info().smsg("Starting QDevD");
    let socket = NetLinkSocket::new().unwrap();
    let reader = BufReader::new(socket);
    let mut freud_socket = match init_freudian() {
        Ok(socket) => socket,
        Err(e) => {
            logger.info().with_string("error", e.to_string()).smsg("Failed to open Freudian connection");
            return;
        }
    };

    let mut current_event = HashMap::new();
    for line in reader.split(0) {
        let line = match line {
            Ok(line) => String::from_utf8(line).unwrap(),
            Err(e) => {
                logger.debug().with_string("error", e.to_string()).smsg("Got an IO Error reading Netlink socket. Bailing");
                break;
            }
        };

        if !line.contains("=") {
            if current_event.len() > 0 {
                // Dispatch the old event
                let event_str = serde_json::to_string(&current_event);
                freud_socket.produce_message(FREUDIAN_TOPIC_NAME, event_str.unwrap().bytes().collect());
            }
        }
        else {
            current_event.insert(line.to_owned(), "");
        }
    }
}