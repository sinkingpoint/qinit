extern crate clap;
extern crate libq;
extern crate libqinit;
extern crate nix;

use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

use libq::daemon::write_pid_file;
use libq::logger;

use clap::{App, Arg};
use libqinit::tasks::{TaskRegistry, TaskStatus};
use std::path::PathBuf;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

enum RunLevel {
    ShutdownMode,
    SingleUserMode,
}

impl RunLevel {
    fn from(i: &str) -> Option<Self> {
        return match i {
            "0" | "shutdownmode" => Some(RunLevel::ShutdownMode),
            "1" | "singleusermode" => Some(RunLevel::SingleUserMode),
            _ => None,
        };
    }

    fn get_stage_name(&self) -> &str {
        return match self {
            RunLevel::ShutdownMode => "shutdownmode",
            RunLevel::SingleUserMode => "singleusermode",
        };
    }
}

fn reap_processes(task_registry: Arc<Mutex<TaskRegistry>>) {
    loop {
        match waitpid(None, Some(WaitPidFlag::__WALL | WaitPidFlag::WUNTRACED)) {
            Ok(WaitStatus::Exited(child_pid, exit_code)) => {
                let mut task_registry = task_registry.lock().unwrap();
                task_registry.set_status_with_pid(child_pid, TaskStatus::Stopped(exit_code));
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), ()> {
    let args = App::new("qinit")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("init system")
        .arg(Arg::with_name("pidfile").long("pidfile").help("Sets the PID file to use"))
        .arg(Arg::with_name("socketfile").long("socket").help("Sets the socket file to use"))
        .arg(
            Arg::with_name("taskdir")
                .long("taskdir")
                .takes_value(true)
                .multiple(true)
                .help("Specifies the directories to look for tasks in"),
        )
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
        Ok(()) => {}
        Err(err) => {
            logger.info().with_string("error", err.to_string()).smsg("Failed to start qinit");
        }
    }

    let task_dirs = match args.values_of("taskdir") {
        Some(values) => values.map(|p| PathBuf::from(p)).collect(),
        None => vec![PathBuf::from("/etc/qinit/tasks")],
    };

    let task_registry: Arc<Mutex<TaskRegistry>> = match TaskRegistry::load_from_disk(&task_dirs) {
        Ok(reg) => Arc::new(Mutex::new(reg)),
        Err(err) => {
            logger
                .debug()
                .with_string("error", format!("{}", err))
                .msg(format!("Failed to load task definitions. Dropping into a shell"));
            return Err(());
        }
    };

    let reaper_registry = Arc::clone(&task_registry);
    std::thread::spawn(move || reap_processes(reaper_registry));

    {
        let mut task_registry = task_registry.lock().unwrap();
        match task_registry.execute_task(run_level.get_stage_name(), &HashMap::new()) {
            Ok(_) => {
                logger.info().msg(format!("Started QInit"));
            }
            Err(_) => {
                logger.info().msg(format!("Failed to start QInit"));
            }
        }
        logger.info().msg(format!("Loaded {} tasks", task_registry.len()));
    }

    loop {}
}
