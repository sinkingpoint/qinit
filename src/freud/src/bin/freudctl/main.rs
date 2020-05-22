extern crate clap;
extern crate libfreudian;

use clap::{Arg, App, SubCommand, AppSettings};
use std::path::PathBuf;
use std::cmp::min;
use std::os::unix::net::UnixStream;
use std::io::{self, Read, Write};

use libfreudian::api::{MessageType, ResponseType, OneValueRequest, PutMessageRequest};

const MAX_MESSAGE_LENGTH: usize = 1 << 16 - 1;

fn make_request(class: MessageType, msg: Vec<u8>) -> Vec<u8> {
    let message_length = min(msg.len(), MAX_MESSAGE_LENGTH);
    let mut message = vec![class.into(), ((message_length & 0xFF00) >> 8) as u8, (message_length & 0xFF) as u8];
    for byte in msg.iter() {
        message.push(*byte);
    }

    return message;
}

fn create_topic(topic_name: &str) -> Vec<u8> {
    return make_request(MessageType::CreateTopic, OneValueRequest::new(MessageType::CreateTopic, topic_name.to_string()).into_bytes());
}

fn delete_topic(topic_name: &str) -> Vec<u8> {
    return make_request(MessageType::DeleteTopic, OneValueRequest::new(MessageType::DeleteTopic, topic_name.to_string()).into_bytes());
}

fn create_subscription(topic_name: &str) -> Vec<u8> {
    return make_request(MessageType::Subscribe, OneValueRequest::new(MessageType::Subscribe, topic_name.to_string()).into_bytes());
}

fn read_message(sub_id: &str) -> Vec<u8> {
    return make_request(MessageType::GetMessage, OneValueRequest::new(MessageType::GetMessage, sub_id.to_string()).into_bytes());
}

fn send_message(topic_id: &str, message: &str) -> Vec<u8> {
    return make_request(MessageType::ProduceMessage, PutMessageRequest::new(topic_id.to_string(), message.bytes().collect()).into_bytes());
}

fn send_to_socket(file: &PathBuf, data: Vec<u8>) -> Result<Vec<u8>, io::Error> {
    let mut stream = UnixStream::connect(file)?;
    stream.write_all(&data[..])?;

    let mut buffer: Vec<u8> = vec![0, 0, 0];
    stream.read_exact(&mut buffer)?;
    let message_size = ((buffer[1] as u16) << 8 | buffer[2] as u16) as usize;
    let mut message_buffer = vec![0;message_size];
    stream.read_exact(&mut message_buffer)?;

    buffer.append(&mut message_buffer);

    return Ok(buffer);
}

fn decode_output(data: Vec<u8>) -> Result<(), ()> {
    if data.len() == 0 {
        return Err(());
    }
    let mut iter = data.iter();
    let result = *iter.next().unwrap();
    println!("Status: {}", ResponseType::from(result).as_string());

    let message_size = (*iter.next().unwrap_or(&0) as u16) << 8 | (*iter.next().unwrap_or(&0) as u16);
    if message_size == 0 {
        return Ok(());
    }
    if data.len() < (3 + message_size) as usize {
        println!("Expected more in message, but server only sent {} bytes", data.len());
        return Err(());
    }

    match String::from_utf8(iter.map(|x| *x).collect()) {
        Ok(s) => println!("Got message: {}", s),
        Err(_) => eprintln!("Returned message wasn't printable")
    };

    return Ok(());
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
    let socketfile = PathBuf::from(args.value_of("socketfile").unwrap_or("/run/freudian/socket"));
    

    let mut message: Option<Vec<u8>> = None;
    match args.subcommand() {
        ("topic", Some(matches)) => {
            match matches.subcommand() {
                ("create", Some(matches)) => {
                    message = Some(create_topic(matches.value_of("topic_name").unwrap()));
                },
                ("delete", Some(matches)) => {
                    message = Some(delete_topic(matches.value_of("topic_name").unwrap()));
                },
                ("publish", Some(matches)) => {
                    message = Some(send_message(matches.value_of("topic_name").unwrap(), matches.value_of("message").unwrap()));
                }
                _                       => {},
            }
        },
        ("subscription", Some(matches)) => {
            match matches.subcommand() {
                ("create", Some(matches)) => {
                    message = Some(create_subscription(matches.value_of("topic_name").unwrap()));
                },
                ("delete", Some(_matches)) => {
                    println!("Delete Subscription");
                    println!("Not implemented");
                },
                ("read", Some(matches)) => {
                    message = Some(read_message(matches.value_of("sub_id").unwrap()));
                },
                _                       => {},
            }
        },
        _                       => {},
    };

    if message.is_none() {
        return; // We hit an unimplemented command
    }

    let message = message.unwrap();

    match send_to_socket(&socketfile, message) {
        Ok(response) => decode_output(response).unwrap(),
        Err(error) => {
            eprintln!("Got an error sending to freudian: {}", error);
        }
    }
}