use std::io::{self, Write};

use nix::pty::Winsize;
use nix::libc::c_int;
use nix::sys::termios::{Termios, ControlFlags, InputFlags, OutputFlags, LocalFlags, SpecialCharacterIndices};
use nix::{ioctl_none, ioctl_read, ioctl_write_ptr};

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

/// Enables or disables echoing of characters as they're typed
pub fn set_echo_mode(settings: &mut Termios, echo: bool) {
    let local_flags = LocalFlags::ECHO | LocalFlags::ECHOE | LocalFlags::ECHOKE;

    if echo {
        settings.local_flags |= local_flags;
    }
    else {
        settings.local_flags &= !local_flags;
    }
}

pub fn reset_virtual_console(settings: &mut Termios, keep_cflags: bool, utf8_support: bool) {
    let ttydef_iflag: InputFlags = InputFlags::BRKINT | InputFlags::ISTRIP | InputFlags::ICRNL | InputFlags::IXON | InputFlags::IXANY;
    let ttydef_oflag: OutputFlags = OutputFlags::OPOST | OutputFlags::ONLCR;
    let ttydef_lflag: LocalFlags = LocalFlags::ECHO | LocalFlags::ICANON | LocalFlags::ISIG | LocalFlags::IEXTEN | LocalFlags::ECHOE | LocalFlags::ECHOCTL | LocalFlags::ECHOKE;
    let ttydef_cflag: ControlFlags = ControlFlags::CREAD | ControlFlags::CS7 | ControlFlags::PARENB | ControlFlags::HUPCL;

    settings.input_flags |= ttydef_iflag;
    settings.output_flags |= ttydef_oflag;
    settings.local_flags |= ttydef_lflag;

    if !keep_cflags {
        settings.control_flags |= ttydef_cflag;
    }

    settings.input_flags |= InputFlags::BRKINT | InputFlags::ICRNL | InputFlags::IMAXBEL;
    settings.input_flags &= !(InputFlags::IGNBRK | InputFlags::INLCR | InputFlags::IGNCR | InputFlags::IXOFF | InputFlags::IXANY | InputFlags::ISTRIP);

    settings.output_flags |= OutputFlags::OPOST | OutputFlags::ONLCR | OutputFlags::NL0 | OutputFlags::CR0 | OutputFlags::TAB0 | OutputFlags::BS0 | OutputFlags::VT0 | OutputFlags::FF0;
    settings.output_flags &= !(OutputFlags::OLCUC | OutputFlags::OCRNL | OutputFlags::ONOCR | OutputFlags::ONLRET | OutputFlags::OFILL);

    settings.local_flags |= LocalFlags::ISIG | LocalFlags::ICANON | LocalFlags::IEXTEN | LocalFlags::ECHO | LocalFlags::ECHOE | LocalFlags::ECHOK | LocalFlags::ECHOKE | LocalFlags::ECHOCTL;
    settings.local_flags &= !(LocalFlags::ECHONL | LocalFlags::ECHOPRT | LocalFlags::NOFLSH | LocalFlags::TOSTOP);

    if !keep_cflags {
        settings.control_flags |= ControlFlags::CREAD | ControlFlags::CS8 | ControlFlags::HUPCL;
        settings.control_flags &= !(ControlFlags::PARENB);
    }

    if utf8_support {
        settings.input_flags |= InputFlags::IUTF8;
    }
    else {
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

ioctl_read! {
    /// Make the given terminal the controlling terminal of the calling process. The calling
    /// process must be a session leader and not have a controlling terminal already. If the
    /// terminal is already the controlling terminal of a different session group then the
    /// ioctl will fail with **EPERM**, unless the caller is root (more precisely: has the
    /// **CAP_SYS_ADMIN** capability) and arg equals 1, in which case the terminal is stolen
    /// and all processes that had it as controlling terminal lose it.
    tiocsctty, b'T', 0x0E, c_int
}

ioctl_none! {
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
    tiocnotty, b'T', 0x22
}

ioctl_read! {
    /// get the status of modem bits.
    tiocmget, b'T', 0x0E, c_int
}

ioctl_read! {
    /// Gets current keyboard mode.  argp points to a long which is
    /// set to one of these:
    /// 
    /// K_RAW         0x00  /* Raw (scancode) mode */
    /// K_XLATE       0x01  /* Translate keycodes using keymap */
    /// K_MEDIUMRAW   0x02  /* Medium raw (scancode) mode */
    /// K_UNICODE     0x03  /* Unicode mode */
    /// K_OFF         0x04  /* Disabled mode; since Linux 2.6.39 */
    kdgkbmode, b'K', 0x44, c_int
}

ioctl_read! {
    /// Get window size
    tiocgwinsz, b'T', 0x13, Winsize
}

ioctl_write_ptr! {
    /// Get window size
    tiocswinsz, b'T', 0x14, Winsize
}
