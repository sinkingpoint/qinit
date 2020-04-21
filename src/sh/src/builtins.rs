use process;

pub type Builtin = fn(&mut process::Shell, &Vec<String>) -> Result<u8, ()>;

pub fn exit(shell: &mut process::Shell, _argv: &Vec<String>) -> Result<u8, ()> {
    shell.exit(0);
    return Ok(0);
}
