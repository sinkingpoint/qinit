extern crate clap;
extern crate libq;
extern crate patient;

use clap::{App, AppSettings, Arg, SubCommand};
use std::path::Path;
use std::io::{self, Read};
use std::char;

use libq::io::{read_u32, Endianness, BufferReader};
use libq::logger::{self, ConsoleRecordWriter, Logger};

use patient::{FreudianClient, Status, UUID};

fn friendly_status(s: &Status) -> String {
    return (match s {
        Status::Ok => "OK",
        Status::NothingHappened => "Nothing Happened",
        Status::No => "Permission Denied",
        Status::DoesntExist => "Doesn't Exist",
        Status::MalformedRequest => "Malformed Request",
        Status::ServerError => "Internal Server Error"
    }).to_owned();
}

fn friendly_message(bytes: Vec<u8>) -> String {
    let mut build = String::new();
    for byte in bytes.into_iter() {
        match char::from_u32(byte as u32) {
            Some(chr) => {
                build.push_str(&chr.escape_default().to_string())
            },
            None => {
                build.push_str(&format!("\\x{:x}", byte));
            }
        }
    }

    return build;
}

fn handle_status_message(result: &Result<Status, io::Error>, logger: &Logger<ConsoleRecordWriter>) {
    let msg = match result {
        Ok(status) => friendly_status(status),
        Err(e) => e.to_string()
    };

    logger.info().msg(msg);
}

fn main() {
    let args = App::new("freudian")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("message bus daemon")
        .arg(Arg::with_name("socketfile").long("socket").help("Sets the socket file to use"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("topic")
                .setting(AppSettings::SubcommandRequired)
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Creates a topic with the given name")
                        .arg(Arg::with_name("topic_name").takes_value(true).required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a topic with the given name")
                        .arg(Arg::with_name("topic_name").takes_value(true).required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("publish")
                        .about("Publishes a message onto a given topic")
                        .arg(Arg::with_name("topic_name").takes_value(true).required(true).index(1))
                        .arg(Arg::with_name("message").takes_value(true).required(true).index(2)),
                )
                .subcommand(
                    SubCommand::with_name("ls").about("Lists the names of all the existing topics")
                )
                .subcommand(
                    SubCommand::with_name("subcount")
                        .about("Creates a topic with the given name")
                        .arg(Arg::with_name("topic_name").takes_value(true).required(true).index(1)),
                ),
        )
        .subcommand(
            SubCommand::with_name("subscription")
                .setting(AppSettings::SubcommandRequired)
                .about("Commands for working with subscriptions")
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Creates a subscription on the given topic")
                        .arg(Arg::with_name("topic_name").takes_value(true).required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Deletes a subscription with the given subscription ID")
                        .arg(Arg::with_name("sub_id").takes_value(true).required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("read")
                        .about("Uses a given subscription ID to read a message")
                        .arg(Arg::with_name("sub_id").takes_value(true).required(true).index(1))
                        .arg(Arg::with_name("wait").takes_value(true).short("w").long("wait")),
                ),
        )
        .get_matches();

    let logger = logger::with_name_as_console("freudctl");
    let socketfile = Path::new(args.value_of("socketfile").unwrap_or("/run/freudian/socket"));
    let mut client = match FreudianClient::new(socketfile) {
        Ok(client) => client,
        Err(e) => {
            logger
                .info()
                .with_string("error", e.to_string())
                .smsg("Failed to open Freud socket");
            return;
        }
    };

    match args.subcommand() {
        ("topic", Some(matches)) => match matches.subcommand() {
            ("create", Some(matches)) => {
                handle_status_message(&client.create_topic(matches.value_of("topic_name").unwrap()), &logger);
            }
            ("delete", Some(matches)) => {
                handle_status_message(&client.delete_topic(matches.value_of("topic_name").unwrap()), &logger);
            }
            ("publish", Some(matches)) => {
                handle_status_message(
                    &client.produce_message(matches.value_of("topic_name").unwrap(), matches.value_of("message").unwrap().bytes().collect()),
                    &logger,
                );
            }
            ("ls", Some(_)) => {
                match client.get_topics() {
                    Ok(resp) => {
                        if resp.response_type == Status::Ok {
                            let mut reader = BufferReader::new(&resp.message);
                            let num = read_u32(&mut reader, &Endianness::Little).unwrap();
                            for _ in 0..num {
                                let size = read_u32(&mut reader, &Endianness::Little).unwrap() as usize;
                                let mut title_buffer = vec![0;size];
                                reader.read_exact(&mut title_buffer).unwrap();
                                logger.info().msg(format!("{}", String::from_utf8(title_buffer).unwrap()));
                            }
                        }
                        else {
                            logger.info().msg(friendly_status(&resp.response_type));
                        }
                    },
                    Err(err) => {
                        logger.info().msg(err.to_string());
                    }
                }
            }
            ("subcount", Some(matches)) => {
                let name = matches.value_of("topic_name").unwrap();
                match client.get_num_subscibers(name) {
                    Ok(resp) => {
                        if resp.response_type == Status::Ok {
                            let mut reader = BufferReader::new(&resp.message);
                            let num = read_u32(&mut reader, &Endianness::Little).unwrap();
                            logger.info().msg(format!("{} has {} subscribers", name, num));
                        }
                        else {
                            logger.info().msg(friendly_status(&resp.response_type));
                        }
                    },
                    Err(err) => {
                        logger.info().msg(err.to_string());
                    }
                }
            }
            _ => {}
        },
        ("subscription", Some(matches)) => match matches.subcommand() {
            ("create", Some(matches)) => {
                match client.subscribe(matches.value_of("topic_name").unwrap()) {
                    Ok((Some(uuid), _)) => {
                        logger.info().msg(format!("Subscription ID: {}", uuid));
                    },
                    Ok((_, err)) => {
                        logger.info().msg(friendly_status(&err));
                    },
                    Err(err) => {
                        logger.info().msg(err.to_string());
                    }
                }
            }
            ("delete", Some(matches)) => {
                let uuid = match UUID::try_from_string(matches.value_of("sub_id").unwrap()) {
                    Some(uuid) => uuid,
                    None => {
                        logger.info().smsg("Invalid UUID");
                        return;
                    }
                };
                handle_status_message(
                    &client.unsubscribe(&uuid),
                    &logger,
                );
            }
            ("read", Some(matches)) => {
                let uuid = match UUID::try_from_string(matches.value_of("sub_id").unwrap()) {
                    Some(uuid) => uuid,
                    None => {
                        logger.info().smsg("Invalid UUID");
                        return;
                    }
                };

                let wait = match matches.value_of("wait").unwrap_or("0").parse::<u32>() {
                    Ok(num) => num,
                    Err(_) => {
                        logger.info().smsg("Invalid Wait Time");
                        return;
                    }
                };

                match client.consume_message(&uuid, wait) {
                    Ok(resp) => {
                        if resp.response_type != Status::Ok {
                            logger.info().msg(friendly_status(&resp.response_type));
                            return;
                        }

                        logger.info().msg(format!("Message: {}", friendly_message(resp.message)));
                    },
                    Err(err) => {
                        logger.info().msg(err.to_string());
                    }
                }
            }
            _ => {}
        },
        _ => {}
    };
}
