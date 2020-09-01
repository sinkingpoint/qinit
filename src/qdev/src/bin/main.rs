extern crate patient;
extern crate clap;
extern crate libq;

use patient::{FreudianClient, Status};

use clap::{App, Arg};

use std::path::Path;

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

    let mut logger = logger::with_name_as_json("qdev");

    let sub_id = match client.subscribe(FREUDIAN_TOPIC_NAME) {
        Ok((Some(uuid), _)) => uuid,
        Ok((None, status)) => {
            // TODO: Error here
            logger.info().smsg("Failed to subscribe to Freudian");
            return;
        },
        Err(err) => {
            logger.info().smsg("Failed to talk to Freudian");
            return;
        }
    };

    loop {
        let resp = match client.consume_message(&sub_id, 30) {
            Ok(resp) => {
                if resp.response_type != Status::Ok {
                    break;
                }

                let message = String::from_utf8(resp.message);
                println!("{:?}", message);
                message
            },
            Err(err) => {
                // logger.info().msg(err.to_string());
                return;
            }
        };
    }
}