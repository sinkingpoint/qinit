extern crate breuer;
extern crate clap;
extern crate libq;

use clap::{App, Arg};
use std::path::Path;

use breuer::{FreudianSocket, FreudianSocketError};
use libq::logger;

fn main() {
    let args = App::new("freudian")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .arg(Arg::with_name("pidfile").long("pidfile").help("Sets the PID file to use"))
        .arg(
            Arg::with_name("socketfile")
                .long("socket")
                .takes_value(true)
                .help("Sets the socket file to use"),
        )
        .about("A message bus daemon, loosely based on Kafka")
        .get_matches();

    let logger = logger::with_name_as_json("freudian");
    let socketfile_str = args.value_of("socketfile").unwrap_or("/run/freudian/socket");
    let socketfile = Path::new(socketfile_str);

    let socket = match FreudianSocket::new(socketfile) {
        Ok(socket) => socket,
        Err(err) => {
            match err {
                FreudianSocketError::ProcessAlreadyRunning => logger
                    .info()
                    .with_str("path", socketfile_str)
                    .smsg("A Process is already listening on this socket"),
                FreudianSocketError::IOError(err) => logger
                    .info()
                    .with_str("path", socketfile_str)
                    .with_string("error", err.to_string())
                    .smsg("Failed to open socket file"),
            }

            return;
        }
    };

    socket.listen_and_serve();
}
