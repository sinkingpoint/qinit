extern crate nix;
extern crate libq;
extern crate regex;

use std::env;
use std::io;
use std::fs::File;
use std::io::BufRead;

mod builtins;
mod shell;
mod ast;
mod strings;

const VERSION: &str = "0.0.1";

trait LineReader {
    fn next_line(&mut self, &mut String) -> io::Result<usize>;
}

struct InputFile {
    stream: io::BufReader<File>
}

impl InputFile {
    fn new(f: File) -> InputFile {
        return InputFile{
            stream: io::BufReader::new(f),
        };
    }
}

impl LineReader for io::Stdin {
    fn next_line(&mut self, dest: &mut String) -> io::Result<usize> {
        return self.read_line(dest);
    }
}

impl LineReader for InputFile {
    fn next_line(&mut self, dest: &mut String) -> io::Result<usize> {
        return self.stream.read_line(dest);
    }
}

fn print_prompt(shell: &shell::Shell, process_name: &String, continue_prompt: bool) {
    if continue_prompt {
        shell.write("> ");
    }
    else {
        let this_exe = std::path::Path::new(process_name);
        let prompt = format!("{}-{}$ ", this_exe.file_name().unwrap().to_string_lossy(), VERSION);
        shell.write(&prompt);
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let argv: Vec<String> = env::args().collect();
    let is_repl = argv.len() == 1;
    let mut shell = shell::Shell::new(is_repl);
    let mut reader: Box<dyn LineReader>;
    if !is_repl {
        let file = match File::open(argv[1].clone()) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Couldn't open file {}: {}", argv[1], err);
                return;
            }
        };
        reader = Box::new(InputFile::new(file));
    }
    else {
        reader = Box::new(io::stdin());
    }
    let mut current_buffer = String::new();
    let mut at_eof = false;
    while !at_eof {
        let mut new_line = String::new();
        print_prompt(&shell, &argv[0], current_buffer != "");
        match reader.next_line(&mut new_line) {
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
                if tokens.len() == 0 {
                    continue;
                }
                match ast::parse_into_ast(&tokens) {
                    Ok(ast) => {
                        current_buffer.clear();
                        ast.execute(&mut shell, None, &shell::IOTriple::new());
                    },
                    Err(err) => {
                        if err.continuable {
                            current_buffer.push('\n');
                        }
                        else {
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
