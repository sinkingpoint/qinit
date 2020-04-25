extern crate nix;
extern crate libq;

use std::env;

mod builtins;
mod shell;
mod ast;

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
        let tokens = tokenizer.try_tokenize();
        match tokens {
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
                if tokens.len() == 0 {
                    continue;
                }
                match ast::parse_into_ast(&tokens) {
                    Ok(ast) => {
                        ast.execute(&mut shell, None, &shell::IOTriple::new());
                    },
                    Err(err) => {
                        if !err.continuable {
                            eprintln!("Error: {}", err.error);
                            current_buffer.clear();
                        }
                    }
                }

                match shell.has_exitted() {
                    Some(_exitcode) => break,
                    None => {}
                }
            }
        }
    }

    shell.write("\n\nGoodbye");
}
