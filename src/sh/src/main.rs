extern crate nix;
extern crate libq;

use nix::sys::signal;
use nix::unistd;
use std::env;

mod process;
mod builtins;

fn _init_shell() -> process::Shell{
    let is_interactive = match unistd::isatty(libq::io::STDIN_FD) {
        Ok(tty) => tty,
        Err(errno) => {
            panic!("STDIN is being weird: {}", errno);
        }
    };

    let my_pgid = unistd::getpgrp();
    if is_interactive {
        let mut fg_pgid = match unistd::tcgetpgrp(libq::io::STDIN_FD) {
            Ok(is_fg) => is_fg,
            Err(errno) => {
                panic!("STDIN is being weird: {}", errno);
            }
        };

        while fg_pgid != my_pgid {
            signal::kill(my_pgid, signal::SIGTTIN).unwrap();
            fg_pgid = match unistd::tcgetpgrp(libq::io::STDIN_FD) {
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

        let my_pid = unistd::getpid();
        unistd::setpgid(my_pid, my_pid).expect("Failed to set PGID for shell");
        unistd::tcsetpgrp(libq::io::STDIN_FD, my_pid).expect("Failed to become the foreground process");
    }

    return process::Shell::new(is_interactive, my_pgid, libq::io::STDIN_FD);
}

fn process_line(shell: &process::Shell, tokens: &Vec<String>) -> Option<u32> {
    let mut pipeline = Vec::new();
    let mut current_process = process::Process::new();

    if tokens.len() == 0 {
        return None;
    }

    for token in tokens.iter() {
        match token.as_str() {
            "|" => {
                pipeline.push(current_process);
                current_process = process::Process::new();
            },
            _ => {
                current_process.add_argv(&token.to_string());
            }
        }
    }

    pipeline.push(current_process);
    return None;
}

const VERSION: &str = "0.0.1";

fn print_prompt(shell: &process::Shell, continue_prompt: bool) {
    if continue_prompt {
        shell.write("> ");
    }
    else {
        let this_argv0 = env::args().next().unwrap();
        let this_exe = std::path::Path::new(&this_argv0);
        let prompt = format!("{}-{}$ ", this_exe.file_name().unwrap().to_string_lossy(), VERSION);
        shell.write(&prompt);
    }
}

fn main() {
    let mut shell = _init_shell();
    let reader = std::io::stdin();
    let mut current_buffer = String::new();
    let mut at_eof = false;
    while !at_eof {
        print_prompt(&shell, current_buffer != "");
        let mut new_line = String::new();
        match reader.read_line(&mut new_line) {
            Ok(0) => at_eof = true,
            Ok(_) => {},
            Err(ioerr) => {
                panic!("Failed reading: {}", ioerr)
            }
        }
        current_buffer.push_str(&new_line.trim_end());
        let mut tokenizer = libq::strings::Tokenizer::new(&current_buffer);
        let _tokens = tokenizer.try_tokenize();
        match _tokens {
            Err(err) if err.is_continuable() => {
                println!("Continuable error: {}", err);
            },
            Err(_err) => {
                eprintln!("Error!!!");
                current_buffer.clear();
            },
            Ok(tokens) => {
                //Process Tokens
                current_buffer.clear();
                match process_line(&shell, &tokens) {
                    Some(pipeline) => {},
                    None => {}
                }

                match shell.has_exitted() {
                    Some(exitcode) => break,
                    None => {}
                }
            }
        }
    }

    shell.write("\n\nGoodbye");
}
