use super::io::full_write_str;
use std::os::unix::io::RawFd;

const ESC: &str = "\x1b[";

pub fn set_cursor_position(fd: RawFd, x: u32, y: u32) -> Result<(), nix::Error> {
    // Format is ESC[x;yH
    full_write_str(fd, &format!("{}{};{}H", ESC, x, y))?;
    return Ok(());
}

pub enum EraseDisplayMode {
    ToEnd,
    FromBeginning,
    All,
    AllAndClear,
}

pub fn erase_display(fd: RawFd, mode: EraseDisplayMode) -> Result<(), nix::Error>{
    let mode_num = match mode {
        EraseDisplayMode::ToEnd => 0,
        EraseDisplayMode::FromBeginning => 1,
        EraseDisplayMode::All => 2,
        EraseDisplayMode::AllAndClear => 3
    };

    full_write_str(fd, &format!("{}{}J", ESC, mode_num))?;
    return Ok(());
}