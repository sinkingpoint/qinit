extern crate libq;
extern crate libfreudian;
extern crate clap;

mod functions;

use clap::{Arg, App};

use libfreudian::Bus;
use libfreudian::api::{self, MessageType, ResponseType};

use functions::{handle_topic_request};

use std::os::unix::net::{UnixStream, UnixListener};
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

use libq::daemon::PidFile;

fn handle_client(bus: &mut Arc<Mutex<Bus>>, mut stream: UnixStream) -> Result<(), io::Error>{
    loop {
        let mut header_buffer = [0, 0, 0];
        stream.read_exact(&mut header_buffer)?;
        let message_class = MessageType::from(header_buffer[0]);
        let message_length = ((header_buffer[1] as u16) << 8 | header_buffer[2] as u16) as usize;
        println!("Found message class: {:?} with length {}", message_class, message_length);
        let mut message_buffer: Vec<u8> = Vec::with_capacity(message_length);
        message_buffer.resize(message_length, 0 as u8);
        stream.read_exact(&mut message_buffer)?;

        let response = match message_class {
            MessageType::CreateTopic => handle_topic_request(bus, api::parse_as_create_topic_request(&message_buffer)),
            MessageType::DeleteTopic => handle_topic_request(bus, api::parse_as_delete_topic_request(&message_buffer)),
            MessageType::Subscribe => handle_topic_request(bus, api::parse_as_subscribe_request(&message_buffer)),
            _ => Ok(vec![ResponseType::Ok.into()])
        };

        let response = match response {
            Ok(resp) => resp,
            Err(()) => {
                return Err(io::Error::new(io::ErrorKind::Other, "Bailing out of thread - bus function died"));
            }
        };

        stream.write_all(&response)?;
    }
}

fn main() {
    let args = App::new("freudian")
                    .version("0.1")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("message bus daemon")
                    .arg(Arg::with_name("pidfile").long("pidfile").help("Sets the PID file to use"))
                    .arg(Arg::with_name("socketfile").long("socket").help("Sets the socket file to use"))
                    .get_matches();

    let pidfile = PathBuf::from(args.value_of("pidfile").unwrap_or("/run/freudian/active.pid"));
    let socketfile = PathBuf::from(args.value_of("socketfile").unwrap_or("/run/freudian/socket"));

    match PidFile::new(pidfile) {
        Ok(pf) => pf,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    if socketfile.exists() {
        // Remove the old socket file if we've managed to get a PID lock
        fs::remove_file(&socketfile).unwrap();
    }

    let socket = match UnixListener::bind(socketfile) {
        Ok(socket) => socket,
        Err(err) => {
            eprintln!("Failed to open socket: {}", err);
            return;
        }
    };

    let bus: Arc<Mutex<Bus>> = Arc::new(Mutex::new(Bus::new()));
    let mut children = Vec::new();

    for stream in socket.incoming() {
        match stream {
            Ok(stream) => {
                let mut bus_ptr = Arc::clone(&bus);
                children.push(thread::spawn(move|| handle_client(&mut bus_ptr, stream)));
            }
            Err(_err) => {
                break;
            }
        }
    }

    for child in children.into_iter() {
        child.join().unwrap().unwrap();
    }
}