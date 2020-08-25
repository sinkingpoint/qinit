use std::ffi::{CStr, CString};
use nix::unistd::{fork, ForkResult, execv};
use libq::logger;
use nix::errno::Errno::ENOENT;

pub fn fork_process(argv: &Vec<&str>) -> Option<u32> {
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
    let path = CString::new(argv[0]).unwrap();

    let argv: Vec<Vec<u8>> = argv.iter()
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