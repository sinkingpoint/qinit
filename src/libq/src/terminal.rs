use std::io::{self, Write};

const ESC: &str = "\x1b[";

pub fn set_cursor_position<T>(f: &mut T, x: u32, y: u32) -> Result<(), io::Error> where T: Write{
    // Format is ESC[x;yH
    f.write_all(&format!("{}{};{}H", ESC, x, y).as_bytes())?;
    return Ok(());
}

pub enum EraseDisplayMode {
    ToEnd,
    FromBeginning,
    All,
    AllAndClear,
}

pub fn erase_display<T>(f: &mut T, mode: EraseDisplayMode) -> Result<(), io::Error> where T: Write{
    let mode_num = match mode {
        EraseDisplayMode::ToEnd => 0,
        EraseDisplayMode::FromBeginning => 1,
        EraseDisplayMode::All => 2,
        EraseDisplayMode::AllAndClear => 3
    };

    f.write_all(format!("{}{}J", ESC, mode_num).as_bytes())?;
    return Ok(());
}

/// Represents a color that an ANSI terminal can output as
#[derive(Debug)]
pub enum TerminalColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Reset,
}

impl TerminalColor {
    pub fn to_num(&self) -> i32 {
        return match self {
            TerminalColor::Black => 0,
            TerminalColor::Red => 1,
            TerminalColor::Green => 2,
            TerminalColor::Yellow => 3,
            TerminalColor::Blue => 4,
            TerminalColor::Magenta => 5,
            TerminalColor::Cyan => 6,
            TerminalColor::White => 7,
            TerminalColor::BrightBlack => 8,
            TerminalColor::BrightRed => 9,
            TerminalColor::BrightGreen => 10,
            TerminalColor::BrightYellow => 11,
            TerminalColor::BrightBlue => 12,
            TerminalColor::BrightMagenta => 13,
            TerminalColor::BrightCyan => 14,
            TerminalColor::BrightWhite => 15,
            TerminalColor::Reset => 39
        }
    }
}

/// Sets the terminals foreground color to the given color. Text after this will be that color
pub fn set_foreground_color<T>(f: &mut T, c: TerminalColor) -> Result<(), io::Error> where T: Write{
    let command: String = match c {
        TerminalColor::Reset => format!("{}39m", ESC),
        _ => format!("{}38;5;{}m", ESC, c.to_num())
    };
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Sets the terminals foreground color to the given raw rgb values
pub fn set_foreground_color_raw<T>(f: &mut T, r: u8, g: u8, b: u8) -> Result<(), io::Error> where T: Write{
    let command: String = format!("{}38;2;{};{};{}m", ESC, r, g, b);
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Sets the terminals background color to the given color. Text after this will be that color
pub fn set_background_color<T>(f: &mut T, c: TerminalColor) -> Result<(), io::Error> where T: Write{
    let command: String = match c {
        TerminalColor::Reset => format!("{}49m", ESC),
        _ => format!("{}48;5;{}m", ESC, c.to_num())
    };
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Sets the terminals background color to the given raw rgb values
pub fn set_background_color_raw<T>(f: &mut T, r: u8, g: u8, b: u8) -> Result<(), io::Error> where T: Write{
    let command: String = format!("{}48;2;{};{};{}m", ESC, r, g, b);
    f.write_all(command.as_bytes())?;
    return Ok(());
}