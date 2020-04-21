extern crate nix;

use std::ffi::{CStr, CString};
use nix::unistd::execvp;
use shell;

pub struct Process {
    pub proc_name: String,
    pub argv: Vec<String>
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

    pub fn launch(&self, shell: &mut shell::Shell, foreground: bool) {
        shell.take_control_of_tty(foreground);
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
