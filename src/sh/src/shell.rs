use std::io::Write;
use nix::unistd::{Pid,tcgetpgrp,tcsetpgrp,getpid,setpgid,getpgrp,isatty};
use nix::sys::signal;
use std::collections::HashMap;
use builtins;

pub struct Shell {
    is_interactive: bool,
    parent_pgid: Pid,
    terminal_fd: i32,
    builtins: HashMap<String, builtins::Builtin>,
    exitcode: Option<u8>,
}

impl Shell {
    pub fn new() -> Shell {
        let is_interactive = match isatty(libq::io::STDIN_FD) {
            Ok(tty) => tty,
            Err(errno) => {
                panic!("STDIN is being weird: {}", errno);
            }
        };
    
        let my_pgid = getpgrp();
        if is_interactive {
            let mut fg_pgid = match tcgetpgrp(libq::io::STDIN_FD) {
                Ok(is_fg) => is_fg,
                Err(errno) => {
                    panic!("STDIN is being weird: {}", errno);
                }
            };
    
            while fg_pgid != my_pgid {
                signal::kill(my_pgid, signal::SIGTTIN).unwrap();
                fg_pgid = match tcgetpgrp(libq::io::STDIN_FD) {
                    Ok(is_fg) => is_fg,
                    Err(errno) => {
                        panic!("STDIN is being weird: {}", errno);
                    }
                };
            }
    
            unsafe {
                signal::signal(signal::SIGINT, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGQUIT, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGTSTP, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGTTIN, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGTTOU, signal::SigHandler::SigIgn).unwrap();
            }
    
            let my_pid = getpid();
            setpgid(my_pid, my_pid).expect("Failed to set PGID for shell");
            tcsetpgrp(libq::io::STDIN_FD, my_pid).expect("Failed to become the foreground process");
        }

        let mut builtin_map = HashMap::new();
        builtin_map.insert(String::from("exit"), builtins::exit as builtins::Builtin);
        return Shell {
            is_interactive: is_interactive,
            parent_pgid: my_pgid,
            terminal_fd: libq::io::STDIN_FD,
            builtins: builtin_map,
            exitcode: None,
        }
    }

    pub fn take_control_of_tty(&self, foreground: bool) {
        if self.is_interactive {
            let pid = getpid();
            setpgid(pid, self.parent_pgid).expect("Failed to set pgid");
            if foreground {
                tcsetpgrp(self.terminal_fd, self.parent_pgid).expect("Failed to set pgid");
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