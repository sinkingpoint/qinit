extern crate clap;
extern crate libq;
extern crate nix;

use clap::{App, Arg};
use libq::qnix::to_mode;
use nix::sys::stat::{makedev, mknod, SFlag};

fn is_int(val: String) -> Result<(), String> {
    match val.parse::<u32>() {
        Ok(_) => return Ok(()),
        Err(_) => return Err(String::from("Must be a number")),
    };
}

fn main() {
    let args = App::new("mknod")
        .version("1.0")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Create the special file NAME of the given TYPE.")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .value_name("MODE")
                .default_value("611")
                .help("set file permission bits to MODE, not a=rw - umask")
                .validator(is_int),
        )
        .arg(Arg::with_name("NAME").index(1).required(true))
        .arg(
            Arg::with_name("TYPE")
                .index(2)
                .required(true)
                .possible_values(&["b", "c", "u", "p"]),
        )
        .arg(
            Arg::with_name("MAJOR")
                .index(3)
                .required_ifs(&[("TYPE", "b"), ("TYPE", "c"), ("TYPE", "u")])
                .validator(is_int),
        )
        .arg(
            Arg::with_name("MINOR")
                .index(4)
                .required_ifs(&[("TYPE", "b"), ("TYPE", "c"), ("TYPE", "u")])
                .validator(is_int),
        )
        .get_matches();

    if args.value_of("TYPE").unwrap() == "p" && (args.is_present("MAJOR") || args.is_present("MINOR")) {
        eprintln!("Fifos do not have major and minor device numbers.");
        eprintln!("Try 'mknod --help' for more information.");
        std::process::exit(1);
    }

    let kind = match args.value_of("TYPE") {
        Some("b") => SFlag::S_IFBLK,
        Some("c") | Some("u") => SFlag::S_IFCHR,
        Some("p") => SFlag::S_IFIFO,
        None | Some(_) => {
            eprintln!("Unknown type");
            std::process::exit(1);
        }
    };

    let mode = match to_mode(args.value_of("mode").unwrap().to_string()) {
        Ok(mode) => mode,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let major = args.value_of("MAJOR").unwrap().parse::<u64>().unwrap();
    let minor = args.value_of("MINOR").unwrap().parse::<u64>().unwrap();
    let dev = makedev(major, minor);

    match mknod(args.value_of("NAME").unwrap(), kind, mode, dev) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
}
