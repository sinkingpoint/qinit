use shell;

use libq::io::{RawFdReader};
use std::os::unix::io::FromRawFd;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap,VecDeque};
use strings;

pub type Builtin = fn(&mut shell::Shell, &Vec<String>, &shell::IOTriple) -> i32;

pub fn get_builtin_registry() -> HashMap<&'static str, Builtin> {
    let mut registry = HashMap::new();
    registry.insert("exit", exit as Builtin);
    registry.insert("export", export as Builtin);
    registry.insert("read", read as Builtin);
    registry.insert("local", local as Builtin);
    registry.insert("[", comparison as Builtin);
    registry.insert("exec", exec as Builtin);
    return registry;
}

trait ComparisonOperator {
    fn as_string(&self) -> String;
    fn requires_two_args(&self) -> bool;
    fn execute(&self, left: &Option<&String>, right: &Option<&String>) -> bool;
}

struct EqualsComparison{}
impl ComparisonOperator for EqualsComparison {
    fn as_string(&self) -> String {
        return String::from("==");
    }

    fn requires_two_args(&self) -> bool {
        return true;
    }

    fn execute(&self, left: &Option<&String>, right: &Option<&String>) -> bool{
        return left.unwrap() == right.unwrap();
    }
}

struct NotEqualsComparison{}
impl ComparisonOperator for NotEqualsComparison {
    fn as_string(&self) -> String {
        return String::from("!=");
    }

    fn requires_two_args(&self) -> bool {
        return true;
    }

    fn execute(&self, left: &Option<&String>, right: &Option<&String>) -> bool{
        return left.unwrap() != right.unwrap();
    }
}

struct NullComparison{}
impl ComparisonOperator for NullComparison {
    fn as_string(&self) -> String {
        return String::from("-n");
    }

    fn requires_two_args(&self) -> bool {
        return false;
    }

    fn execute(&self, _left: &Option<&String>, right: &Option<&String>) -> bool{
        return right.unwrap() == "";
    }
}

struct NotNullComparison{}
impl ComparisonOperator for NotNullComparison {
    fn as_string(&self) -> String {
        return String::from("-z");
    }

    fn requires_two_args(&self) -> bool {
        return false;
    }

    fn execute(&self, _left: &Option<&String>, right: &Option<&String>) -> bool{
        return right.unwrap() != "";
    }
}

#[derive(PartialEq)]
#[derive(Debug)]
enum ComparisonState {
    Left,
    Comparison,
    Right,
    Done
}

#[derive(Debug)]
struct ComparisonBuilder {
    state: ComparisonState,
    left: Option<String>,
    comparison: Option<String>,
    right: Option<String>
}

impl ComparisonBuilder {
    fn new() -> ComparisonBuilder{
        return ComparisonBuilder{
            state: ComparisonState::Left,
            left: None,
            comparison: None,
            right: None,
        }
    }

    fn ingest_token(&mut self, token: &String) -> Result<(), ()> {
        if token.starts_with("-") && self.state == ComparisonState::Left{
            self.comparison = Some(token.clone());
            self.state = ComparisonState::Right;
        }
        else if self.state == ComparisonState::Left {
            self.left = Some(token.clone());
            self.state = ComparisonState::Comparison;
        }
        else if self.state == ComparisonState::Comparison {
            self.comparison = Some(token.clone());
            self.state = ComparisonState::Right;
        }
        else if self.state == ComparisonState::Right {
            self.right = Some(token.clone());
            self.state = ComparisonState::Done;
        }
        else if self.state == ComparisonState::Done && token == &String::from("]"){}
        else {
            return Err(());
        }

        return Ok(());
    }

    fn is_done(&self) -> bool {
        return self.state == ComparisonState::Done;
    }

    fn execute(&self, shell: &shell::Shell) -> i32 {
        let comparisons: Vec<&dyn ComparisonOperator> = vec![
            &EqualsComparison{},
            &NotEqualsComparison{},
            &NullComparison{},
            &NotNullComparison{}
        ];

        for comparison in comparisons.iter() {
            if &comparison.as_string() == self.comparison.as_ref().unwrap() {
                let left = match &self.left {
                    None => None,
                    Some(left) => {
                        match strings::do_value_pipeline(&left, shell) {
                            Ok(words) => Some(words.join(" ")),
                            Err(err) => {
                                eprintln!("Bad Subtitution: {}", err);
                                return 128;
                            }
                        }
                    }
                };

                let right = match &self.right {
                    None => None,
                    Some(right) => {
                        match strings::do_value_pipeline(&right, shell) {
                            Ok(words) => Some(words.join(" ")),
                            Err(err) => {
                                eprintln!("Bad Subtitution: {}", err);
                                return 128;
                            }
                        }
                    }
                };

                if comparison.requires_two_args() && left.is_none() {
                    eprintln!("Parsing failed near {}: {:?}", comparison.as_string(), &self);
                    return 128;
                }

                if comparison.execute(&left.as_ref(), &right.as_ref()) {
                    return 0;
                }
                return 1;
            }
        }

        println!("Unknown operation: {}", self.comparison.as_ref().unwrap());
        return 127;
    }
}

fn comparison(shell: &mut shell::Shell, argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    if argv.last().is_some() && argv.last() != Some(&String::from("]")) {
        eprintln!("[: ] expected");
        return 2;
    }

    let mut iter = argv.iter();
    iter.next(); // Skip the [
    let mut builder = ComparisonBuilder::new();
    for token in iter {
        if let Err(()) = builder.ingest_token(token) {
            eprintln!("Parsing failed near {} {:?}", token, builder);
            return 128;
        }
    }

    if !builder.is_done() {
        eprintln!("Missing operands");
        return 128;
    }

    return builder.execute(shell);
}

fn exec(shell: &mut shell::Shell, argv: &Vec<String>, streams: &shell::IOTriple) -> i32 {
    let argv = argv[1..].to_vec();
    let mut new_proc = shell::Process::new(argv[0].clone(), argv, false);
    if let Err(e) = new_proc.cement_args(shell) {
        eprintln!("Failed to subtitute: {}", e);
        return 0;
    }
    new_proc.execute(shell, None, streams);
    return 1;
}

fn exit(shell: &mut shell::Shell, _argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    shell.exit(0);
    return 0;
}

fn _set_variables(shell: &mut shell::Shell, argv: &Vec<String>, environment: bool) -> i32 {
    let mut iter = argv.iter().peekable();
    if iter.peek().is_some() && iter.peek() == Some(&&String::from("local")) || iter.peek() == Some(&&String::from("export")) {
        iter.next(); // Skip first token, because that'll be the builtin name
    }
    for token in iter {
        let parts: Vec<&str> = token.splitn(2, "=").collect();

        let (name, value) = match parts.len() {
            1 => (String::from(parts[0]), String::from(shell.get_variable(parts[0]))),
            2 => {
                match strings::do_value_pipeline(&String::from(parts[1]), shell) {
                    Ok(words) => (String::from(parts[0]), words.join(" ")),
                    Err(_err) => {
                        eprintln!("Bad Substitution: {}", parts[1]);
                        continue;
                    }
                }
            },
            _ => {
                eprintln!("Bad Substitution: {}", token);
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