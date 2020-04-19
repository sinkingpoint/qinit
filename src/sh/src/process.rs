extern crate nix;

use nix::unistd::{fork, ForkResult, Pid, getpid, setpgid, tcsetpgrp, execvp};
use nix::sys::signal;
use std::ffi::{CStr, CString};
use nix::sys::wait::{waitpid, WaitPidFlag};

pub struct Shell {
    is_interactive: bool,
    pub parent_pgid: Pid,
    terminal_fd: i32,
}

impl Shell {
    pub fn new(interactive: bool, pgid: Pid, terminal_fd: i32) -> Shell {
        return Shell {
            is_interactive: interactive,
            parent_pgid: pgid,
            terminal_fd: terminal_fd
        }
    }
}

pub struct Process {
    proc_name: String,
    argv: Vec<String>
}

impl Process {
    pub fn new() -> Process {
        return Process{
            proc_name: String::new(),
            argv: Vec::new()
        };
    }

    pub fn add_argv(&mut self, arg: &String) {
        if self.proc_name == "" {
            self.proc_name = arg.clone();
        }
        self.argv.push(arg.clone());
    }

    pub fn launch(&self, shell: &Shell, pgid: Pid, foreground: bool) {
        if shell.is_interactive {
            let pid = getpid();
            setpgid(pid, pgid).expect("Failed to set pgid");
            if foreground {
                tcsetpgrp(shell.terminal_fd, pgid).expect("Failed to set pgid");
            }

            unsafe {
                signal::signal(signal::SIGINT, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGQUIT, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGTSTP, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGTTIN, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGTTOU, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGCHLD, signal::SigHandler::SigDfl).unwrap();
            }
        }

        let c_path = CString::new(self.proc_name.as_str()).unwrap();
        let cstr_argv: Vec<Vec<u8>> = self.argv.iter().map(|arg| CString::new(arg.as_str()).unwrap().into_bytes_with_nul()).collect();
        let argv = &cstr_argv.iter().map(|arg| CStr::from_bytes_with_nul(arg).unwrap()).collect::<Vec<&CStr>>()[..];
        match execvp(&c_path, argv) {
            Ok(_) => {
            },
            Err(_) => {
                panic!("Failed to exec!");
            }
        }
    }
}

pub struct Pipeline {
    processes: Vec<Process>,
    pgid: Option<Pid>
}

impl Pipeline {
    pub fn new(pipeline: Vec<Process>, pgid: Option<Pid>) -> Pipeline{
        return Pipeline {
            processes: pipeline,
            pgid: pgid
        };
    }

    pub fn start(&self, shell: &Shell, foreground: bool) {
        for process in &self.processes {
            match fork() {
                Ok(ForkResult::Parent { child, .. }) => {
                    waitpid(child, Some(WaitPidFlag::WUNTRACED | WaitPidFlag::__WALL)).expect("Failed waiting for child");
                    tcsetpgrp(shell.terminal_fd, shell.parent_pgid);
                }
                Ok(ForkResult::Child) => {
                    process.launch(shell, self.pgid.unwrap(), foreground);
                },
                Err(_) => println!("Fork failed"),
            }
        }
    }
}