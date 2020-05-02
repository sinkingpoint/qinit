extern crate clap;
extern crate nix;

use clap::{Arg,App};
use nix::mount::{mount,MsFlags};
use nix::errno::Errno::ENOENT;


fn main() {
    let args = App::new("mount")
                      .version("0.1")
                      .author("Colin D. <colin@quirl.co.nz>")
                      .about("Mount a filesystem.")
                      .arg(Arg::with_name("type").short("t").long("type").takes_value(true).required(true))
                      .arg(Arg::with_name("options").short("o").long("options").takes_value(true))
                      .arg(Arg::with_name("device").takes_value(true).index(1).required(true))
                      .arg(Arg::with_name("dir").takes_value(true).index(2).required(true))
                      .get_matches();
    
    let device = args.value_of("device");
    let dir = args.value_of("dir").unwrap();
    let mounttype = args.value_of("type");
    match mount::<str, str, str, str>(device, dir, mounttype, MsFlags::empty(), None) {
        Ok(()) => {},
        Err(err) => {
            if let Some(errno) = err.as_errno() {
                if errno == ENOENT {
                    eprintln!("mount {}: {}", dir, "mount point doesn't exist");
                }
                else {
                    eprintln!("{}", err);
                }
            }
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
}