extern crate nix;
extern crate libq;

use std::env;

mod process;
mod builtins;
mod shell;

fn process_line(shell: &shell::Shell, tokens: &Vec<String>) -> Option<u32> {
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

fn print_prompt(shell: &shell::Shell, continue_prompt: bool) {
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
    let mut shell = shell::Shell::new();
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
