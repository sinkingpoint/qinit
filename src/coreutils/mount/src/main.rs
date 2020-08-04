extern crate clap;
extern crate libq;
extern crate nix;

use clap::{App, Arg};
use libq::blkid::Device;
use nix::errno::Errno::ENOENT;
use nix::mount::{mount, MsFlags};
use std::path::PathBuf;

fn derive_actual_fs_type(dev: &str) -> Option<String> {
    if let Ok(device) = Device::from_path(PathBuf::from(dev)) {
        if let Some(probe_result) = device.probe() {
            return Some(probe_result.get_fs());
        }
    }

    return None;
}

fn main() {
    let args = App::new("mount")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Mount a filesystem.")
        .arg(Arg::with_name("type").short("t").long("type").takes_value(true))
        .arg(Arg::with_name("options").short("o").long("options").takes_value(true))
        .arg(Arg::with_name("device").takes_value(true).index(1).required(true))
        .arg(Arg::with_name("dir").takes_value(true).index(2).required(true))
        .get_matches();

    let device = args.value_of("device");
    let dir = args.value_of("dir").unwrap();
    let mut mounttype = args.value_of("type");
    let actual_type: String;
    if mounttype == Some("auto") || mounttype == Some("") || mounttype == None {
        if let Some(fs_type) = derive_actual_fs_type(device.unwrap()) {
            actual_type = fs_type;
            mounttype = Some(&actual_type);
        } else {
            mounttype = None;
        }
    }

    match mount::<str, str, str, str>(device, dir, mounttype, MsFlags::empty(), None) {
        Ok(()) => {}
        Err(err) => {
            if let Some(errno) = err.as_errno() {
                if errno == ENOENT {
                    eprintln!("mount {}: {}", dir, "mount point doesn't exist");
                } else {
                    eprintln!("mount: Error: {}", err);
                }
            } else {
                eprintln!("mount: {}", err);
                std::process::exit(1);
            }
        }
    };
}
