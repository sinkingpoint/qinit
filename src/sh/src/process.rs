extern crate nix;

use nix::unistd::{fork, ForkResult, Pid, getpid, setpgid, tcsetpgrp, execvp};
use nix::sys::signal;
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use std::io::Write;
use builtins;

pub struct Shell {
    is_interactive: bool,
    parent_pgid: Pid,
    terminal_fd: i32,
    builtins: HashMap<String, builtins::Builtin>,
    exitcode: Option<u8>,
}

impl Shell {
    pub fn new(interactive: bool, pgid: Pid, terminal_fd: i32) -> Shell {
        let mut builtin_map = HashMap::new();
        builtin_map.insert(String::from("exit"), builtins::exit as builtins::Builtin);
        return Shell {
            is_interactive: interactive,
            parent_pgid: pgid,
            terminal_fd: terminal_fd,
            builtins: builtin_map,
            exitcode: None,
        }
    }

    pub fn get_parent_pgid(&self) -> Pid {
        return self.parent_pgid;
    }

    pub fn is_interactive(&self) -> bool {
        return self.is_interactive;
    }

    pub fn is_builtin(&self, name: &String) -> bool{
        return self.builtins.contains_key(name);
    }

    pub fn run_builtin(&mut self, name: &String, argv: &Vec<String>) -> Result<u8, ()> {
        return self.builtins.get(name).unwrap()(self, argv);
    }

    pub fn exit(&mut self, code: u8) {
        self.exitcode = Some(code);
    }

    pub fn write(&self, text: &str) {
        if self.is_interactive() {
            print!("{}", text);
        }
        std::io::stdout().flush().expect("Failed writing to stdout");
    }

    pub fn has_exitted(&self) -> Option<u8> {
        return self.exitcode;
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

    pub fn launch(&self, shell: &mut Shell, pgid: Pid, foreground: bool) {
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
            Err(e) => {
                panic!("Failed to exec: {}", e);
            }
        }
    }
}
