extern crate libq;
extern crate nix;
extern crate regex;

use std::env;
use std::fs::File;

mod ast;
mod builtins;
mod shell;
mod shio;
mod strings;

use shio::{InputFile, LineReader, ShStdin};

const VERSION: &str = "0.0.1";

fn print_prompt(shell: &shell::Shell, process_name: &String, continue_prompt: bool) {
    if continue_prompt {
        shell.write("> ");
    } else {
        let this_exe = std::path::Path::new(process_name);
        let prompt = format!("{}-{}$ ", this_exe.file_name().unwrap().to_string_lossy(), VERSION);
        shell.write(&prompt);
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let mut argv: Vec<String> = env::args().collect();
    argv.reverse();
    let is_repl = argv.len() == 1;
    let exe_name = argv.pop().expect("Program name missing?");
    let mut reader: Box<dyn LineReader>;
    if !is_repl {
        let file = match File::open(argv[1].clone()) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Couldn't open file {}: {}", argv[1], err);
                return;
            }
        };
        argv.pop().expect("File name missing?");
        reader = Box::new(InputFile::new(file));
    } else {
        reader = Box::new(ShStdin::new());
    }
    argv.reverse();
    let mut shell = shell::Shell::new(is_repl, argv);
    let mut current_buffer = String::new();
    let mut at_eof = false;
    while !at_eof {
        let mut new_line = String::new();
        print_prompt(&shell, &exe_name, current_buffer != "");
        match reader.next_line(&mut new_line, &mut shell) {
            Ok(0) => at_eof = true,
            Ok(_) => {}
            Err(ioerr) => panic!("Failed reading: {}", ioerr),
        }
        current_buffer.push_str(&new_line.trim_end());
        let mut tokenizer = libq::strings::Tokenizer::new(&current_buffer, vec!['\n', ';', '|', '&', '#']);
        let tokens = tokenizer.try_tokenize();
        match tokens {
            Err(err) if err.is_continuable() => {
                println!("Continuable error: {}", err);
            }
            Err(_err) => {
                eprintln!("Error!!!");
                current_buffer.clear();
            }
            Ok(tokens) => {
                //Process Tokens
                if tokens.len() == 0 {
                    current_buffer.clear();
                    continue;
                }
                match ast::parse_into_ast(&tokens) {
                    Ok(ast) => {
                        shell.add_history_line(current_buffer.clone());
                        current_buffer.clear();
                        ast.execute(&mut shell, None, &shell::IOTriple::new());
                    }
                    Err(err) => {
                        if err.continuable {
                            current_buffer.push('\n');
                        } else {
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
