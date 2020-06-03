extern crate libqinit;
extern crate libq;
extern crate clap;

use libq::logger::{self};
use libqinit::tasks::TaskRegistry;
use std::path::PathBuf;
use clap::{Arg, App};

fn main() -> Result<(), ()>{
    let args = App::new("qinit")
                    .version("0.1")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("init system")
                    .arg(Arg::with_name("pidfile").long("pidfile").help("Sets the PID file to use"))
                    .arg(Arg::with_name("socketfile").long("socket").help("Sets the socket file to use"))
                    .arg(Arg::with_name("taskdir").long("taskdir").takes_value(true).multiple(true).help("Specifies the directories to look for tasks in"))
                    .get_matches();

    let pidfile = PathBuf::from(args.value_of("pidfile").unwrap_or("/run/freudian/active.pid"));
    let socketfile = PathBuf::from(args.value_of("socketfile").unwrap_or("/run/freudian/socket"));

    let task_dirs = match args.values_of("taskdir") {
        Some(values) => values.map(|p| PathBuf::from(p)).collect(),
        None => vec![PathBuf::from("/etc/qinit/tasks")]
    };

    let logger = logger::with_name_as_json("test");
    let task_registry = match TaskRegistry::load_from_disk(&task_dirs){
        Ok(reg) => reg,
        Err(err) => {
            logger.debug().with_string("error", format!("{}", err)).msg(format!("Failed to load task definitions. Dropping into a shell"));
            return Err(());
        }
    };

    logger.info().msg(format!("Loaded {} tasks", task_registry.len()));

    return Ok(());
}