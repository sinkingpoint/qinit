use shell;
use std::collections::HashMap;

pub type Builtin = fn(&mut shell::Shell, &Vec<String>, &shell::IOTriple) -> i32;

pub fn get_builtin_registry() -> HashMap<&'static str, Builtin> {
    let mut registry = HashMap::new();
    registry.insert("exit", exit as Builtin);
    registry.insert("export", export as Builtin);
    return registry;
}

pub fn exit(shell: &mut shell::Shell, _argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    shell.exit(0);
    return 0;
}

pub fn export(shell: &mut shell::Shell, argv: &Vec<String>, _streams: &shell::IOTriple) -> i32 {
    let mut iter = argv.iter();
    iter.next(); // Skip first token, because that'll be "export"
    for token in iter {
        let parts: Vec<&str> = token.split('=').collect();

        let (name, value) = match parts.len() {
            1 => (String::from(parts[0]), String::new()),
            2 => (String::from(parts[0]), String::from(parts[1])),
            _ => {
                return 1
            }
        };

        shell.set_variable(shell::Variable::new(name, value, true));
    }

    return 0;
}