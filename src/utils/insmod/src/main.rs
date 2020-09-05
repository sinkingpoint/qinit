extern crate clap;
extern crate libq;
extern crate nix;

use clap::{App, Arg};
use libq::logger;
use nix::kmod::{finit_module, ModuleInitFlags};
use nix::sys::utsname::uname;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::exit;

fn find_module_path(modname: &str) -> Result<Option<PathBuf>, io::Error> {
    let uname = uname();
    let kernel = uname.release();
    let file = File::open(format!("/lib/modules/{}/modules.names", kernel))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts = line.split(":").collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Ok(None);
        }

        if parts[0] == modname {
            return Ok(Some(PathBuf::from(parts[1].trim())));
        }
    }

    return Ok(None);
}

fn main() {
    let args = App::new("insmod")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Installs the given kernel module")
        .arg(
            Arg::with_name("file")
                .index(1)
                .help("The path of the kernel module to install")
                .required(true),
        )
        .arg(
            Arg::with_name("arguments")
                .index(2)
                .multiple(true)
                .help("The path of the kernel module to install"),
        )
        .get_matches();

    let mut mod_path = PathBuf::from(args.value_of("file").unwrap());
    let logger = logger::with_name_as_console("insmod");
    if !mod_path.exists() {
        if let Ok(Some(path)) = find_module_path(args.value_of("file").unwrap()) {
            mod_path = path;
        } else {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .smsg("Path doesn't exist");
            exit(1);
        }
    }

    let mod_args = match args.values_of("arguments") {
        Some(args) => args.collect::<Vec<&str>>().join(" "),
        None => String::new(),
    };

    let mod_file = match File::open(mod_path) {
        Ok(f) => f,
        Err(e) => {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .with_string("error", e.to_string())
                .smsg("Failed to open module file");
            exit(1);
        }
    };

    match finit_module(&mod_file, CString::new(mod_args).unwrap().as_c_str(), ModuleInitFlags::empty()) {
        Ok(_) => {}
        Err(e) => {
            logger
                .info()
                .with_str("path", args.value_of("file").unwrap())
                .with_string("error", e.to_string())
                .smsg("Failed to load module file");
            exit(1);
        }
    }
}
