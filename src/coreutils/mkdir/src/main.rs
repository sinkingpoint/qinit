extern crate clap;
extern crate nix;
extern crate libq;

use std::path::Path;
use clap::{App,Arg};

use libq::qnix::to_mode;
use nix::errno::Errno;
use nix::unistd::mkdir;
use nix::sys::stat::Mode;

fn mkdir_if_not_exists(path: &str, m: Mode) -> Result<(), nix::Error> {
    let path = Path::new(path);
    if path.is_dir() {
        return Err(nix::Error::from_errno(Errno::EEXIST));
    }

    return mkdir(path, m);
}

fn main() {
    let args = App::new("mkdir")
                    .version("0.1")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("Create the DIRECTORY(ies), if they do not already exist.")
                    .arg(Arg::with_name("mode").short("m").long("mode").help("set file mode (ala chmod). umask").takes_value(true).default_value("775"))
                    .arg(Arg::with_name("parent").short("p").long("parents").help("Create parent directories as needed"))
                    .arg(Arg::with_name("verbose").short("v").long("verbose").help("Print out a line for every directory created"))
                    .arg(Arg::with_name("directories").takes_value(true).value_name("DIRECTORY").index(1).required(true))
                    .get_matches();

    let directories = args.values_of("directories").unwrap();
    let mode = to_mode(args.value_of("mode").unwrap().to_string()).expect("Failed to convert mode");
    for dir in directories {
        mkdir_if_not_exists(dir, mode);
    }
}