extern crate libqinit;
extern crate libq;
extern crate clap;

use libq::logger;
use libq::daemon::write_pid_file;

use libqinit::tasks::TaskRegistry;
use std::path::PathBuf;
use clap::{Arg, App};

enum RunLevel {
    SingleUserMode
}

impl RunLevel {
    fn from(i: &str) -> Option<Self> {
        return match i {
            "1" | "singleusermode" => Some(RunLevel::SingleUserMode),
            _ => None
        }
    }

    fn to_str(&self) -> &str {
        return match self {
            RunLevel::SingleUserMode => "singleusermode"
        }
    }

    fn get_stage_name(&self) -> &str {
        return match self {
            RunLevel::SingleUserMode => "singleusermode",
        }
    }
}

fn main() -> Result<(), ()>{
    let args = App::new("qinit")
                    .version("0.1")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("init system")
                    .arg(Arg::with_name("pidfile").long("pidfile").help("Sets the PID file to use"))
                    .arg(Arg::with_name("socketfile").long("socket").help("Sets the socket file to use"))
                    .arg(Arg::with_name("taskdir").long("taskdir").takes_value(true).multiple(true).help("Specifies the directories to look for tasks in"))
                    .arg(Arg::with_name("level").index(1))
                    .get_matches();

    let logger = logger::with_name_as_json("qinit");
    let pidfile = PathBuf::from(args.value_of("pidfile").unwrap_or("/run/qinit/active.pid"));
    let socketfile = PathBuf::from(args.value_of("socketfile").unwrap_or("/run/qinit/socket"));
    let run_level_name = args.value_of("level").unwrap_or("singleusermode");
    let run_level = match RunLevel::from(run_level_name) {
        Some(rl) => rl,
        None => {
            logger.info().msg(format!("Invalid run level {}. Bailing", run_level_name));
            return Err(());
        }
    };

    match write_pid_file(pidfile) {
        Ok(()) => {},
        Err(err) => {
            logger.info().with_string("error", err.to_string()).smsg("Failed to start qinit");
        }
    }

    let task_dirs = match args.values_of("taskdir") {
        Some(values) => values.map(|p| PathBuf::from(p)).collect(),
        None => vec![PathBuf::from("/etc/qinit/tasks")]
    };

    let task_registry = match TaskRegistry::load_from_disk(&task_dirs){
        Ok(reg) => reg,
        Err(err) => {
            logger.debug().with_string("error", format!("{}", err)).msg(format!("Failed to load task definitions. Dropping into a shell"));
            return Err(());
        }
    };

    logger.info().msg(format!("Loaded {} tasks", task_registry.len()));

    match task_registry.get_task(run_level.get_stage_name()) {
        Some(task) => {},
        None => {
            logger.info().msg(format!("Failed to find task {} for run level {}", run_level.get_stage_name(), run_level.to_str()));
        }
    }

    return Ok(());
}