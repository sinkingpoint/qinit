use super::registry::SphereRegistry;
use libq::logger;
use nix::errno::Errno::{ECHILD, ENOENT};
use nix::sys::wait::{wait, WaitStatus};
use nix::unistd::{execv, fork, ForkResult};
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};

/// listen_for_children is responsible for waiting for signals (In particular, SIGCHLD) and telling the registry about it
pub fn listen_for_children(registry_lock: Arc<Mutex<SphereRegistry>>) {
    let logger = logger::with_name_as_json("process;listen_for_children");
    loop {
        match wait() {
            Ok(WaitStatus::Exited(pid, exit_code)) => {
                logger
                    .info()
                    .with_i32("pid", pid.as_raw())
                    .with_i32("exit_code", exit_code)
                    .smsg("Process exitted");
                let mut registry = registry_lock.lock().unwrap();
                registry.handle_child_exit(pid.as_raw() as u32, exit_code as u32);
            }
            Ok(_) => {}
            Err(err) => {
                if let Some(errno) = err.as_errno() {
                    if errno == ECHILD {
                        continue;
                    }
                }
                logger.info().with_string("error", err.to_string()).smsg("Failed to wait");
            }
        }
    }
}

/// fork_process forks this process, and execs the child using the given
/// argv, returning the child pid to the calling process (in the parent). argv[0] must be fully qualified
pub fn fork_process(argv: &Vec<String>) -> Option<u32> {
    let logger = logger::with_name_as_json("process;fork_process");

    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            return Some(child.as_raw() as u32);
        }
        Err(e) => {
            logger.info().with_string("error", e.to_string()).smsg("Failed to fork child");
            return None;
        }
        Ok(ForkResult::Child) => {}
    }

    // If we get here, we're the child so lets exec
    let path = CString::new(argv[0].bytes().collect::<Vec<u8>>()).unwrap();

    let argv: Vec<Vec<u8>> = argv
        .iter()
        .map(|arg| CString::new(arg.bytes().collect::<Vec<u8>>()).unwrap().into_bytes_with_nul())
        .collect();

    let argv = &argv
        .iter()
        .map(|arg| CStr::from_bytes_with_nul(arg).unwrap())
        .collect::<Vec<&CStr>>()[..];

    match execv(&path, argv) {
        Ok(_) => {}
        Err(e) => {
            if let Some(errno) = e.as_errno() {
                if errno == ENOENT {
                    std::process::exit(127);
                }
            }
            std::process::exit(128);
        }
    }

    return None;
}
