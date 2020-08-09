extern crate libq;
extern crate clap;

use libq::netlink::NetLinkSocket;
use libq::logger;

use clap::{App, Arg};

fn main() {
    let args = App::new("qdevd")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("A UDev Daemon which pipes events into Freudian")
        .get_matches();

    let logger = logger::with_name_as_json("qdevd");
    logger.info().smsg("Starting QDevD");

    let socket = NetLinkSocket::new().unwrap();
    println!("{:?}", socket.read_raw_message());
}