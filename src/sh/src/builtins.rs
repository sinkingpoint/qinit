use shell;

pub type Builtin = fn(&mut shell::Shell, &Vec<String>) -> Result<u8, ()>;

pub fn exit(shell: &mut shell::Shell, _argv: &Vec<String>) -> Result<u8, ()> {
    shell.exit(0);
    return Ok(0);
}
