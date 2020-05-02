use shell;

use libq::io::{RawFdReader};
use std::os::unix::io::FromRawFd;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap,VecDeque};

pub type Builtin = fn(&mut shell::Shell, &Vec<String>, &shell::IOTriple) -> i32;

pub fn get_builtin_registry() -> HashMap<&'static str, Builtin> {
    let mut registry = HashMap::new();
    registry.insert("exit", exit as Builtin);
    registry.insert("export", export as Builtin);
    registry.insert("read", read as Builtin);
    registry.insert("local", local as Builtin);
    return registry;
}

fn exit(shell: &mut shell::Shell, _argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    shell.exit(0);
    return 0;
}

fn _set_variables(shell: &mut shell::Shell, argv: &Vec<String>, environment: bool) -> i32 {
    let mut iter = argv.iter();
    iter.next(); // Skip first token, because that'll be the builtin name
    for token in iter {
        let parts: Vec<&str> = token.split('=').collect();

        let (name, value) = match parts.len() {
            1 => (String::from(parts[0]), String::from(shell.get_variable(parts[0]))),
            2 => (String::from(parts[0]), String::from(parts[1])),
            _ => {
                return 1
            }
        };

        shell.set_variable(shell::Variable::new(name, value, environment));
    }
    return 0;
}

fn export(shell: &mut shell::Shell, argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    return _set_variables(shell, argv, true)
}

fn local(shell: &mut shell::Shell, argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    return _set_variables(shell, argv, false)
}

fn read(shell: &mut shell::Shell, argv: &Vec<String>, streams: &shell::IOTriple) -> i32 {
    let mut variable_names = VecDeque::new();
    let mut arg_iter = argv.iter();
    arg_iter.next();
    for arg in arg_iter {
        if arg.starts_with("-") {
            // We have an arg
        }
        else {
            variable_names.push_back(arg);
        }
    }

    if variable_names.len() == 0 {
        return 0;
    }

    let input_file = unsafe { RawFdReader::from_raw_fd(streams.stdin) };
    let mut input_file = BufReader::new(input_file);

    let mut buffer = String::new();
    input_file.read_line(&mut buffer);

    let mut current_variable = String::new();
    for chr in buffer.chars() {
        if chr.is_whitespace() && variable_names.len() > 1 {
            if current_variable.len() > 0 {
                // Skip the whitespace if we're still searching for the next value
                let variable_name = variable_names.pop_front().unwrap().trim_end().to_string();
                println!("Found value {} for variable {}", current_variable, variable_name);
                shell.set_variable(shell::Variable::new(variable_name, current_variable, false));
                current_variable = String::new();
            }
        }
        else {
            current_variable.push(chr);
        }
    }

    shell.set_variable(shell::Variable::new(variable_names.pop_front().unwrap().trim_end().to_string(), current_variable, false));
    return 0;
}