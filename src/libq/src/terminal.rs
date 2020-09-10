use std::fmt;
use std::io::{self, Read, Write};

use super::io::STDIN_FD;

use nix::libc::c_int;
use nix::pty::Winsize;
use nix::sys::termios::{ControlFlags, InputFlags, LocalFlags, OutputFlags, SpecialCharacterIndices, Termios};
use nix::{ioctl_none_bad, ioctl_read_bad, ioctl_write_ptr_bad};

const ESC: &str = "\x1b[";

pub fn set_cursor_position<T>(f: &mut T, x: u32, y: u32) -> Result<(), io::Error>
where
    T: Write,
{
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

pub const ESCAPE_CHAR: u8 = 0x1B;
pub const CTRL_C_BYTE: u8 = 0x03;
pub const EOF_BYTE: u8 = 0x04;
pub const NULL_BYTE: u8 = '\0' as u8;
pub const BACKSPACE_BYTE: u8 = 0x08;
pub const TAB_BYTE: u8 = '\t' as u8;
pub const DELETE_BYTE: u8 = 0x7F;
pub const CARRIAGE_RETURN_BYTE: u8 = '\r' as u8;
pub const NEW_LINE_BYTE: u8 = '\n' as u8;

pub fn erase_display<T>(f: &mut T, mode: EraseDisplayMode) -> Result<(), io::Error>
where
    T: Write,
{
    let mode_num = match mode {
        EraseDisplayMode::ToEnd => 0,
        EraseDisplayMode::FromBeginning => 1,
        EraseDisplayMode::All => 2,
        EraseDisplayMode::AllAndClear => 3,
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
            TerminalColor::Reset => 39,
        };
    }
}

/// Sets the terminals foreground color to the given color. Text after this will be that color
pub fn set_foreground_color<T>(f: &mut T, c: TerminalColor) -> Result<(), io::Error>
where
    T: Write,
{
    let command: String = match c {
        TerminalColor::Reset => format!("{}39m", ESC),
        _ => format!("{}38;5;{}m", ESC, c.to_num()),
    };
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Sets the terminals foreground color to the given raw rgb values
pub fn set_foreground_color_raw<T>(f: &mut T, r: u8, g: u8, b: u8) -> Result<(), io::Error>
where
    T: Write,
{
    let command: String = format!("{}38;2;{};{};{}m", ESC, r, g, b);
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Sets the terminals background color to the given color. Text after this will be that color
pub fn set_background_color<T>(f: &mut T, c: TerminalColor) -> Result<(), io::Error>
where
    T: Write,
{
    let command: String = match c {
        TerminalColor::Reset => format!("{}49m", ESC),
        _ => format!("{}48;5;{}m", ESC, c.to_num()),
    };
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Sets the terminals background color to the given raw rgb values
pub fn set_background_color_raw<T>(f: &mut T, r: u8, g: u8, b: u8) -> Result<(), io::Error>
where
    T: Write,
{
    let command: String = format!("{}48;2;{};{};{}m", ESC, r, g, b);
    f.write_all(command.as_bytes())?;
    return Ok(());
}

/// Enables or disables echoing of characters as they're typed
pub fn set_echo_mode(settings: &mut Termios, echo: bool) {
    let local_flags = LocalFlags::ECHO | LocalFlags::ECHOE | LocalFlags::ECHOKE;

    if echo {
        settings.local_flags |= local_flags;
    } else {
        settings.local_flags &= !local_flags;
    }
}

pub fn reset_virtual_console(settings: &mut Termios, keep_cflags: bool, utf8_support: bool) {
    let ttydef_iflag: InputFlags = InputFlags::BRKINT | InputFlags::ISTRIP | InputFlags::ICRNL | InputFlags::IXON | InputFlags::IXANY;
    let ttydef_oflag: OutputFlags = OutputFlags::OPOST | OutputFlags::ONLCR;
    let ttydef_lflag: LocalFlags = LocalFlags::ECHO
        | LocalFlags::ICANON
        | LocalFlags::ISIG
        | LocalFlags::IEXTEN
        | LocalFlags::ECHOE
        | LocalFlags::ECHOCTL
        | LocalFlags::ECHOKE;
    let ttydef_cflag: ControlFlags = ControlFlags::CREAD | ControlFlags::CS7 | ControlFlags::PARENB | ControlFlags::HUPCL;

    settings.input_flags |= ttydef_iflag;
    settings.output_flags |= ttydef_oflag;
    settings.local_flags |= ttydef_lflag;

    if !keep_cflags {
        settings.control_flags |= ttydef_cflag;
    }

    settings.input_flags |= InputFlags::BRKINT | InputFlags::ICRNL | InputFlags::IMAXBEL;
    settings.input_flags &=
        !(InputFlags::IGNBRK | InputFlags::INLCR | InputFlags::IGNCR | InputFlags::IXOFF | InputFlags::IXANY | InputFlags::ISTRIP);

    settings.output_flags |= OutputFlags::OPOST
        | OutputFlags::ONLCR
        | OutputFlags::NL0
        | OutputFlags::CR0
        | OutputFlags::TAB0
        | OutputFlags::BS0
        | OutputFlags::VT0
        | OutputFlags::FF0;
    settings.output_flags &= !(OutputFlags::OLCUC | OutputFlags::OCRNL | OutputFlags::ONOCR | OutputFlags::ONLRET | OutputFlags::OFILL);

    settings.local_flags |= LocalFlags::ISIG
        | LocalFlags::ICANON
        | LocalFlags::IEXTEN
        | LocalFlags::ECHO
        | LocalFlags::ECHOE
        | LocalFlags::ECHOK
        | LocalFlags::ECHOKE
        | LocalFlags::ECHOCTL;
    settings.local_flags &= !(LocalFlags::ECHONL | LocalFlags::ECHOPRT | LocalFlags::NOFLSH | LocalFlags::TOSTOP);

    if !keep_cflags {
        settings.control_flags |= ControlFlags::CREAD | ControlFlags::CS8 | ControlFlags::HUPCL;
        settings.control_flags &= !(ControlFlags::PARENB);
    }

    if utf8_support {
        settings.input_flags |= InputFlags::IUTF8;
    } else {
        settings.input_flags &= !(InputFlags::IUTF8);
    }

    settings.control_chars[SpecialCharacterIndices::VTIME as usize] = 0;
    settings.control_chars[SpecialCharacterIndices::VMIN as usize] = 0;
    settings.control_chars[SpecialCharacterIndices::VINTR as usize] = 0x63 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VQUIT as usize] = 0o34;
    settings.control_chars[SpecialCharacterIndices::VERASE as usize] = 0o177;
    settings.control_chars[SpecialCharacterIndices::VKILL as usize] = 0x75 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VEOF as usize] = 0x64 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VSWTC as usize] = 0;
    settings.control_chars[SpecialCharacterIndices::VSTART as usize] = 0x71 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VSTOP as usize] = 0x73 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VSUSP as usize] = 0x7A & 0o37;
    settings.control_chars[SpecialCharacterIndices::VEOL as usize] = 0;
    settings.control_chars[SpecialCharacterIndices::VREPRINT as usize] = 0x72 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VDISCARD as usize] = 0x6F & 0o37;
    settings.control_chars[SpecialCharacterIndices::VWERASE as usize] = 0x77 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VLNEXT as usize] = 0x76 & 0o37;
    settings.control_chars[SpecialCharacterIndices::VEOL2 as usize] = 0;
}

ioctl_read_bad! {
    /// Make the given terminal the controlling terminal of the calling process. The calling
    /// process must be a session leader and not have a controlling terminal already. If the
    /// terminal is already the controlling terminal of a different session group then the
    /// ioctl will fail with **EPERM**, unless the caller is root (more precisely: has the
    /// **CAP_SYS_ADMIN** capability) and arg equals 1, in which case the terminal is stolen
    /// and all processes that had it as controlling terminal lose it.
    tiocsctty, 0x540E, c_int
}

ioctl_none_bad! {
    // Detach the calling process from its controlling terminal.

    // If the process is the session leader, then SIGHUP and SIGCONT signals
    // are sent to the foreground process group and all processes in the
    // current session lose their controlling tty.

    // This ioctl(2) call works only on file descriptors connected to
    // /dev/tty.  It is used by daemon processes when they are invoked by a
    // user at a terminal.  The process attempts to open /dev/tty.  If the
    // open succeeds, it detaches itself from the terminal by using
    // TIOCNOTTY, while if the open fails, it is obviously not attached to a
    // terminal and does not need to detach itself.
    tiocnotty, 0x5422
}

ioctl_read_bad! {
    /// get the status of modem bits.
    tiocmget, 0x5415, c_int
}

ioctl_read_bad! {
    /// Gets current keyboard mode.  argp points to a long which is
    /// set to one of these:
    ///
    /// K_RAW         0x00  /* Raw (scancode) mode */
    /// K_XLATE       0x01  /* Translate keycodes using keymap */
    /// K_MEDIUMRAW   0x02  /* Medium raw (scancode) mode */
    /// K_UNICODE     0x03  /* Unicode mode */
    /// K_OFF         0x04  /* Disabled mode; since Linux 2.6.39 */
    kdgkbmode, 0x4B44, c_int
}

ioctl_read_bad! {
    /// Get window size
    tiocgwinsz, 0x5413, Winsize
}

pub fn get_window_size() -> Result<Winsize, nix::Error> {
    let mut winsize = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        match tiocgwinsz(STDIN_FD, &mut winsize) {
            Ok(_) => {
                return Ok(winsize);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

ioctl_write_ptr_bad! {
    /// Set window size
    tiocswinsz, 0x5414, Winsize
}

#[derive(Debug)]
pub enum AnsiEscapeCode {
    CursorUp(u32),
    CursorDown(u32),
    CursorForward(u32),
    CursorBack(u32),
    CursorNextLine(u32),
    CursorPreviousLine(u32),
    CursorColumn(u32),
    CursorPosition(u32, u32),
    EraseInDisplay(u32),
    EraseInLine(u32),
    PageUp(u32),
    PageDown(u32),
    Unknown(char, Option<u32>, Option<u32>, Option<u32>),
}

impl fmt::Display for AnsiEscapeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

impl AnsiEscapeCode {
    pub fn to_string(&self) -> String {
        return match self {
            AnsiEscapeCode::CursorUp(n) => format!("{}{}A", ESC, n),
            AnsiEscapeCode::CursorDown(n) => format!("{}{}B", ESC, n),
            AnsiEscapeCode::CursorForward(n) => format!("{}{}C", ESC, n),
            AnsiEscapeCode::CursorBack(n) => format!("{}{}D", ESC, n),
            AnsiEscapeCode::CursorNextLine(n) => format!("{}{}E", ESC, n),
            AnsiEscapeCode::CursorPreviousLine(n) => format!("{}{}F", ESC, n),
            AnsiEscapeCode::CursorColumn(n) => format!("{}{}G", ESC, n),
            AnsiEscapeCode::CursorPosition(n, m) => format!("{}{};{}H", ESC, n, m),
            AnsiEscapeCode::EraseInDisplay(n) => format!("{}{}J", ESC, n),
            AnsiEscapeCode::EraseInLine(n) => format!("{}{}K", ESC, n),
            AnsiEscapeCode::PageUp(n) => format!("{}{}S", ESC, n),
            AnsiEscapeCode::PageDown(n) => format!("{}{}T", ESC, n),
            AnsiEscapeCode::Unknown(cmd, a, b, c) => {
                let mut build = String::from(ESC);
                let mut num_args = 0;
                for arg in [a, b, c].iter() {
                    if arg.is_some() {
                        if num_args > 0 {
                            build.push(';');
                        }

                        num_args += 1;

                        build.push_str(arg.unwrap().to_string().as_str());
                    }
                }

                build.push(*cmd);
                build
            }
        };
    }

    pub fn read_from_stdin() -> Option<AnsiEscapeCode> {
        let mut stdin = io::stdin();
        let mut arg_buffer = String::new();
        let command: char;
        let mut args: [u32; 3] = [1, 1, 1];
        let mut num_args: usize = 0;
        let mut buffer: [u8; 1] = [0];
        let mut hit_open = false;
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) | Err(_) => {
                    return None; // EOF before we completed
                }
                Ok(_) => {}
            };

            let byte = buffer[0];
            let chr: char = byte.into();

            if chr != '[' && !hit_open {
                return None; // Malformed. We expect [ immediately after the ESC which enters this function
            } else if chr == '[' {
                if hit_open {
                    return None; // Malformed, we expect [ only once
                } else {
                    hit_open = true;
                    continue;
                }
            }

            match chr {
                c if c.is_numeric() => {
                    arg_buffer.push(c);
                }
                ';' => {
                    if num_args >= 3 {
                        return None; // Malformed. Too Many Args
                    }
                    if arg_buffer == "" {
                        arg_buffer.push('1');
                    }
                    args[num_args] = arg_buffer.parse().unwrap();
                    arg_buffer.clear();
                    num_args += 1;
                }
                c => {
                    command = c;
                    break;
                }
            }
        }

        if num_args >= 3 {
            return None; // Malformed. Too Many Args
        }
        if arg_buffer == "" {
            arg_buffer.push('1');
        }

        args[num_args] = arg_buffer.parse().unwrap();
        num_args += 1;

        return match command {
            'A' => Some(AnsiEscapeCode::CursorUp(args[0])),
            'B' => Some(AnsiEscapeCode::CursorDown(args[0])),
            'C' => Some(AnsiEscapeCode::CursorForward(args[0])),
            'D' => Some(AnsiEscapeCode::CursorBack(args[0])),
            'E' => Some(AnsiEscapeCode::CursorNextLine(args[0])),
            'F' => Some(AnsiEscapeCode::CursorPreviousLine(args[0])),
            'G' => Some(AnsiEscapeCode::CursorColumn(args[0])),
            'H' => Some(AnsiEscapeCode::CursorPosition(args[0], args[1])),
            'J' => Some(AnsiEscapeCode::EraseInDisplay(args[0])),
            'K' => Some(AnsiEscapeCode::EraseInLine(args[0])),
            'S' => Some(AnsiEscapeCode::PageUp(args[0])),
            'T' => Some(AnsiEscapeCode::PageDown(args[0])),
            c => match num_args {
                0 => Some(AnsiEscapeCode::Unknown(c, None, None, None)),
                1 => Some(AnsiEscapeCode::Unknown(c, Some(args[0]), None, None)),
                2 => Some(AnsiEscapeCode::Unknown(c, Some(args[0]), Some(args[1]), None)),
                3 => Some(AnsiEscapeCode::Unknown(c, Some(args[0]), Some(args[1]), Some(args[2]))),
                _ => None,
            },
        };
    }
}
