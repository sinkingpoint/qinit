extern crate clap;
extern crate patient;
extern crate libq;

use clap::{Arg, App, SubCommand, AppSettings};
use std::path::PathBuf;

use libq::logger::{self, Logger, ConsoleRecordWriter};

use patient::{FreudianClientError, ResponseType, FreudianClient};

fn friendly_error<'a>(f: FreudianClientError) -> String {
    return match f {
        FreudianClientError::EmptyResponse => String::from("Got an empty response from the server"),
        FreudianClientError::ResponseTooSmall => String::from("Got a response, but the advertised message size was more than the data sent"),
        FreudianClientError::ExtraData => String::from("Got a response, but the advertised message size was less than the data sent"),
        FreudianClientError::ServerResponse(r) => r.as_string(),
        FreudianClientError::BrokenSocket => String::from("Got an IO error trying to talk to Freudian")
    };
}

fn handle_status_message(response: Result<ResponseType, FreudianClientError>, logger: &Logger<ConsoleRecordWriter>) {
    match response {
        Ok(response_type) => {
            logger.info().msg(response_type.as_string());
        },
        Err(client_err) => {
            logger.info().msg(friendly_error(client_err));
        }
    }
}

fn handle_response_message(response: Result<(ResponseType, String), FreudianClientError>, body_preamble: &str, logger: &Logger<ConsoleRecordWriter>) {
    match response {
        Ok((response_type, body)) => {
            handle_status_message(Ok(response_type), logger);
            if body.len() > 0 {
                logger.info().msg(format!("{}: {}", body_preamble, body));
            }
        },
        Err(client_err) => {
            logger.info().msg(friendly_error(client_err));
        }
    }
}

fn main() {
    let args = App::new("freudian")
                    .version("0.1")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("message bus daemon")
                    .arg(Arg::with_name("socketfile").long("socket").help("Sets the socket file to use"))
                    .setting(AppSettings::SubcommandRequired)
                    .subcommand(SubCommand::with_name("topic")
                                .setting(AppSettings::SubcommandRequired)
                                .subcommand(SubCommand::with_name("create")
                                           .about("Creates a topic with the given name")
                                           .arg(Arg::with_name("topic_name")
                                                .takes_value(true)
                                                .required(true)
                                                .index(1)))
                                .subcommand(SubCommand::with_name("delete")
                                            .about("Delete a topic with the given name")
                                            .arg(Arg::with_name("topic_name")
                                                    .takes_value(true)
                                                    .required(true)
                                                    .index(1)))
                                .subcommand(SubCommand::with_name("publish")
                                            .about("Publishes a message onto a given topic")
                                            .arg(Arg::with_name("topic_name")
                                                    .takes_value(true)
                                                    .required(true)
                                                    .index(1))
                                            .arg(Arg::with_name("message")
                                                    .takes_value(true)
                                                    .required(true)
                                                    .index(2))))
                    .subcommand(SubCommand::with_name("subscription")
                                .setting(AppSettings::SubcommandRequired)
                                .about("Commands for working with subscriptions")
                                .subcommand(SubCommand::with_name("create")
                                           .about("Creates a subscription on the given topic")
                                           .arg(Arg::with_name("topic_name")
                                                .takes_value(true)
                                                .required(true)
                                                .index(1)))
                                .subcommand(SubCommand::with_name("delete")
                                .about("Deletes a subscription with the given subscription ID")
                                .arg(Arg::with_name("sub_id")
                                        .takes_value(true)
                                        .required(true)
                                        .index(1)))
                                .subcommand(SubCommand::with_name("read")
                                .about("Uses a given subscription ID to read a message")
                                .arg(Arg::with_name("sub_id")
                                        .takes_value(true)
                                        .required(true)
                                        .index(1))))
                    .get_matches();

    let logger = logger::with_name_as_console("freudctl");
    let socketfile = PathBuf::from(args.value_of("socketfile").unwrap_or("/run/freudian/socket"));
    let mut client = match FreudianClient::new(socketfile) {
        Ok(client) => client,
        Err(e) => {
            logger.info().with_string("error", e.to_string()).smsg("Failed to open Freud socket");
            return;
        }
    };
    
    match args.subcommand() {
        ("topic", Some(matches)) => {
            match matches.subcommand() {
                ("create", Some(matches)) => {
                    handle_status_message(client.create_topic(matches.value_of("topic_name").unwrap()), &logger);
                },
                ("delete", Some(matches)) => {
                    handle_status_message(client.delete_topic(matches.value_of("topic_name").unwrap()), &logger);
                },
                ("publish", Some(matches)) => {
                    handle_status_message(client.send_message(matches.value_of("topic_name").unwrap(), matches.value_of("message").unwrap()), &logger);
                }
                _                       => {},
            }
        },
        ("subscription", Some(matches)) => {
            match matches.subcommand() {
                ("create", Some(matches)) => {
                    handle_response_message(client.create_subscription(matches.value_of("topic_name").unwrap()), "Subscription ID: ", &logger);
                },
                ("delete", Some(_matches)) => {
                    println!("Delete Subscription");
                    println!("Not implemented");
                },
                ("read", Some(matches)) => {
                    handle_response_message(client.read_message(matches.value_of("sub_id").unwrap()), "Message: ", &logger);
                },
                _                       => {},
            }
        },
        _                       => {},
    };
}