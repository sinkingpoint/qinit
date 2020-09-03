extern crate patient;
extern crate clap;
extern crate libq;
extern crate serde_json;

use patient::{FreudianClient, Status};

use clap::{App, Arg};

use std::path::Path;
use std::collections::HashMap;

use libq::logger;

const FREUDIAN_SOCKET: &str = "/run/freudian/socket";
const FREUDIAN_TOPIC_NAME: &str = "uevents";

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
    }
}