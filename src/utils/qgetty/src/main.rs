extern crate clap;
extern crate nix;
extern crate libq;
extern crate libc;

use clap::{App, Arg};

use nix::errno::Errno;
use nix::unistd::{getpid, isatty, close, Pid, tcsetpgrp, dup, execve};
use nix::fcntl::{fcntl, open, OFlag, FcntlArg};
use nix::sys::stat::{fstat, Mode};
use nix::sys::termios::{tcflush, FlushArg, SpecialCharacterIndices, tcgetattr, tcsetattr, tcgetsid, Termios, ControlFlags, BaudRate, cfgetispeed, cfgetospeed, cfsetispeed, cfsetospeed, SetArg, InputFlags, OutputFlags, LocalFlags};
use nix::sys::signal;
use nix::pty::Winsize;

use libq::passwd::GroupEntry;
use libq::logger::{self, Logger, JSONRecordWriter};
use libq::terminal::{erase_display, EraseDisplayMode, reset_virtual_console, tiocsctty, tiocnotty, kdgkbmode, tiocswinsz, tiocgwinsz};
use libq::io::{FileType, STDIN_FD, STDOUT_FD, STDERR_FD, S_IRWOTH};

use std::path::PathBuf;
use std::os::unix::io::RawFd;
use std::io::{self, BufReader, Read, Write};
use std::ffi::{CStr, CString};

use libc::{fchown, fchmod, vhangup};

#[derive(PartialEq)]
enum KeyboardMode {
    Raw,       /* Raw (scancode) mode */
    XLate,     /* Translate keycodes using keymap */
    MediumRaw, /* Medium raw (scancode) mode */
    Unicode,   /* Unicode mode */
    Off,       /* Disabled mode; since Linux 2.6.39 */
    Invalid    /* We received an invalid value from the API */
}

impl From<i32> for KeyboardMode {
    fn from(a: i32) -> KeyboardMode {
        return match a {
            0x00 => KeyboardMode::Raw,
            0x01 => KeyboardMode::XLate,
            0x02 => KeyboardMode::MediumRaw,
            0x03 => KeyboardMode::Unicode,
            0x04 => KeyboardMode::Off,
            _ => KeyboardMode::Invalid
        };
    }
}

struct GettyOptions {
    term_type: Option<String>,
    keyboard_mode: KeyboardMode,
    autolog_user: Option<String>,
    should_hangup: bool,
    is_vconsole: bool,
    utf8_support: bool,
    eight_bit_mode: bool,
    dont_clear: bool,
    keep_baud: bool,
    keep_cflags: bool,
    speeds: Vec<BaudRate>
}

fn try_int(n: String) -> Result<(), String> {
    return match n.parse::<i32>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Input must be a valid number".to_owned())
    };
}

fn main() -> Result<(), ()> {
    let args = App::new("qgetty")
                    .version("0.1")
                    .author("Colin D. <colin@quirl.co.nz>")
                    .about("A _very_ minimal getty implementation")
                    .arg(Arg::with_name("autolog").short("a").long("autologin").takes_value(true).help("Automatically log in the specified user without asking for a username or password."))
                    .arg(Arg::with_name("hangup").short("R").long("--hangup").help("Sets the TTY to bind to"))
                    .arg(Arg::with_name("noreset").short("c").long("--noreset").help("Do not reset terminal cflags"))
                    .arg(Arg::with_name("noclear").short("J").long("--noclear").help("Don't clear the terminal"))
                    .arg(Arg::with_name("delay").long("delay").validator(try_int).help("Sleep seconds before opening tty"))
                    .arg(Arg::with_name("keepbaud").short("s").long("keep-baud").validator(try_int).help("Try to keep the existing baud rate"))
                    .arg(Arg::with_name("tty").index(1).help("Sets the TTY to bind to").required(true))
                    .arg(Arg::with_name("termtype").index(2).help("Specifies the terminal type to use"))
                    .get_matches();

    let tty = args.value_of("tty").unwrap();
    
    unsafe {
        let action = signal::SigAction::new(signal::SigHandler::SigIgn, signal::SaFlags::SA_RESTART, signal::SigSet::empty());
        signal::sigaction(signal::Signal::SIGHUP, &action).expect("Failed to set signal action");
        signal::sigaction(signal::Signal::SIGQUIT, &action).expect("Failed to set signal action");
        signal::sigaction(signal::Signal::SIGINT, &action).expect("Failed to set signal action");
    }

    let mut options = GettyOptions{
        term_type: match args.value_of("termtype") {
            Some(s) => Some(s.to_owned()),
            None => None
        },
        autolog_user: match args.value_of("autolog") {
            Some(s) => Some(s.to_owned()),
            None => None
        },
        should_hangup: args.is_present("hangup"),
        is_vconsole: false,
        utf8_support: false,
        eight_bit_mode: false,
        dont_clear: args.is_present("noclear"),
        keep_baud: args.is_present("keepbaud"),
        keep_cflags: args.is_present("noreset"),
        keyboard_mode: KeyboardMode::Invalid,
        speeds: Vec::new()
    };

    let mut termio_settings = match open_tty(tty, &mut options) {
        Ok(settings) => settings,
        Err(_) => {
            return Err(());
        }
    };

    let logger = logger::with_name_as_json("qgetty");
    match tcsetpgrp(STDIN_FD, getpid()) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to set terminal process group");
        }
    }

    if options.speeds.len() == 0 && !options.is_vconsole {
        options.speeds.push(BaudRate::B9600);
        options.keep_baud = true;
    }

    init_term_settings(&mut termio_settings, &mut options);

    let username = match read_username() {
        Ok(username) => username,
        Err(_) => {
            return Err(());
        }
    };

    let login_bin = CString::new("/sbin/login").unwrap();
    let args: Vec<Vec<u8>> = ["/sbin/login", username.as_str()].into_iter().map(|arg| CString::new(*arg).unwrap().into_bytes_with_nul()).collect();
    let args = &args.iter().map(|arg| CStr::from_bytes_with_nul(arg).unwrap()).collect::<Vec<&CStr>>()[..];

    let env: Vec<Vec<u8>> = [format!("TERM={}", options.term_type.unwrap()).as_str()].into_iter().map(|env| CString::new(*env).unwrap().into_bytes_with_nul()).collect();
    let env = &env.iter().map(|env| CStr::from_bytes_with_nul(env).unwrap()).collect::<Vec<&CStr>>()[..];

    match execve(&login_bin, &args[..], &env[..]) {
        Ok(_) => {} // We should never get here. A sucessful execvp will never get here as it will be running the other program
        Err(err) => {
            if let Some(errno) = err.as_errno() {
                if errno == Errno::ENOENT {
                    eprintln!("No such command: {}", "/sbin/login");
                    std::process::exit(127);
                }
            }
            else {
                logger.debug().with_string("error", err.to_string()).smsg("Failed to exec login process");
            }
        }
    }

    return Ok(());
}

fn read_username() -> Result<String, Option<io::Error>> {
    let logger = logger::with_name_as_json("qgetty;read_username");
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    handle.write_all(b"login: ")?;
    handle.flush()?;

    let reader = BufReader::new(io::stdin());
    let mut build = Vec::new();

    for byte in reader.bytes() {
        let byte = match byte {
            Ok(b) => b,
            Err(err) => {
                logger.info().with_string("error", err.to_string()).smsg("Failed to read from stdin");
                return Err(Some(err));
            }
        };

        match byte {
            // 0x0D == CR , 0x0A == NL. Terminate the username read
            0x0D | 0x0A => {
                if build.len() > 0 {
                    handle.write_all(b"\n")?;
                    return Ok(String::from_utf8_lossy(&build[..]).to_string());
                }
                else {
                    handle.write_all(b"\nlogin: ")?;
                }
            },
            // 0x7f == DEL , 0x08 == Backspace. Delete the last char (if it exists)
            0x7F | 0x08 => {
                match build.pop(){
                    Some(_) => {
                        handle.write_all(b"\x08 \x08")?;
                    },
                    None => {}
                }
            },
            c => {
                build.push(c);
                handle.write_all(&mut [c])?;
            }
        }

        handle.flush()?;
    }

    return Err(None);
}

fn set_blocking(fd: RawFd) {
    let logger = logger::with_name_as_json("qgetty;set_blocking");
    let current_mode = match fcntl(fd, FcntlArg::F_GETFL) {
        Ok(m) => m,
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to get stdin attributes");
            0
        }
    };

    // And set them, minus the O_NONBLOCK flag
    match fcntl(fd, FcntlArg::F_SETFL(OFlag::from_bits_truncate(current_mode) & !(OFlag::O_NONBLOCK))) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to set stdin to blocking");
        }
    }
}

fn reset_vc(settings: &mut Termios, options: &mut GettyOptions) {
    let logger = logger::with_name_as_json("qgetty;reset_vc");
    reset_virtual_console(settings, options.keep_cflags, options.utf8_support);

    match tcsetattr(STDIN_FD, SetArg::TCSADRAIN, settings) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to set terminal attributes");
        }
    }

    // Make stdin block
    set_blocking(STDIN_FD);
}

fn init_term_settings(termio_settings: &mut Termios, options: &mut GettyOptions) {
    let logger = logger::with_name_as_json("qgetty;init_term_settings");
    // VConsole
    if options.is_vconsole {
        logger.debug().smsg("Device is a virtual console");
        options.utf8_support = options.keyboard_mode == KeyboardMode::Unicode;

        // reset vc
        reset_vc(termio_settings, options);

        if termio_settings.control_flags & (ControlFlags::CS8 | ControlFlags::PARODD | ControlFlags::PARENB) == ControlFlags::CS8 {
            options.eight_bit_mode = true;
            if !options.dont_clear {
                match erase_display(&mut io::stdout(), EraseDisplayMode::All) {
                    Ok(_) => {},
                    Err(err) => {
                        logger.debug().with_string("error", err.to_string()).smsg("Failed to clear terminal");
                    }
                }
            }

            return;
        }
    }

    // Serial Line
    logger.debug().smsg("Device is a serial line");

    // Set us up for reading the username, if we aren't given one in the options
    if options.autolog_user.is_none() {
        if options.utf8_support {
            termio_settings.input_flags |= InputFlags::IUTF8;
        }
        else {
            termio_settings.input_flags = InputFlags::empty();
        }
    }

    termio_settings.local_flags = LocalFlags::empty();
    
    // OPOST -> Disable Post Processing (e.g. tr 'U+00A0' '\n')
    // ONLCR -> Don't munge new line chars - we want to handle them ourselves
    termio_settings.output_flags &= !(OutputFlags::OPOST | OutputFlags::ONLCR);

    if !options.keep_cflags {
        // CS8 -> 8 bit chars, no parity
        // HUPCL -> Send a Hangup when the last process exits
        // CREAD -> Enable reading chars
        // CLOCAL -> local connection, no modem contol
        termio_settings.control_flags = ControlFlags::CS8 | ControlFlags::HUPCL | ControlFlags::CREAD | (termio_settings.control_flags & ControlFlags::CLOCAL);
    }

    let ispeed: BaudRate;
    let ospeed: BaudRate;
    if options.keep_baud || options.speeds.len() == 0 {
        ispeed = match cfgetispeed(termio_settings) {
            BaudRate::B0 => BaudRate::B9600,
            speed => speed
        };

        ospeed = match cfgetospeed(termio_settings) {
            BaudRate::B0 => BaudRate::B9600,
            speed => speed
        };
    }
    else {
        ispeed = *options.speeds.last().unwrap();
        ospeed = ispeed;
    }

    match cfsetispeed(termio_settings, ispeed) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to set terminal input speed");
        }
    }

    match cfsetospeed(termio_settings, ospeed) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to set terminal output speed");
        }
    }

    // Return `read` calls immediately after 1 char (No buffering)
    termio_settings.control_chars[SpecialCharacterIndices::VTIME as usize] = 0;
    termio_settings.control_chars[SpecialCharacterIndices::VMIN as usize] = 1;

    let mut winsize = Winsize{
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0
    };

    unsafe {
        match tiocgwinsz(STDIN_FD, &mut winsize) {
            Ok(0) => {
                if winsize.ws_row == 0 {
                    winsize.ws_row = 24;
                }
    
                if winsize.ws_col == 0 {
                    winsize.ws_col = 80;
                }
    
                match tiocswinsz(STDIN_FD, &mut winsize) {
                    Ok(_) => {},
                    Err(err) => {
                        logger.debug().with_string("error", err.to_string()).smsg("Failed to set window size");
                    }
                }
            },
            Ok(_) | Err(_) => {}
        }
    }

    match tcflush(STDIN_FD, FlushArg::TCIOFLUSH) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to flush IO streams");
        }
    }

    match tcsetattr(STDIN_FD, SetArg::TCSANOW, termio_settings) {
        Ok(_) => {},
        Err(err) => {
            logger.debug().with_string("error", err.to_string()).smsg("Failed to set terminal attributes");
        }
    }

    set_blocking(STDIN_FD);
}

fn try_reparenting(fd: RawFd, mypid: Pid, logger: &Logger<JSONRecordWriter>, tty_name: &str) {
    let needs_reparenting = match tcgetsid(fd) {
        Ok(pid) => {
            logger.debug().with_str("tty", tty_name).msg(format!("TTY Session PID: {}, My PID: {}", pid, mypid));
            pid != mypid
        },
        Err(err) => {
            logger.debug().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to get terminal session ID");
            true
        }
    };

    if needs_reparenting {
        unsafe {
            let mut arg = 1;
            match tiocsctty(fd, &mut arg) {
                Ok(-1) => {},
                Ok(_) => {},
                Err(err) => {
                    logger.debug().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to make the terminal the controlling terminal of this process");
                }
            }
        }
    }
}

fn open_tty(tty_name: &str, options: &mut GettyOptions) -> Result<Termios, Option<nix::Error>>{
    let mut logger = logger::with_name_as_json("qgetty;open_tty");
    logger.set_debug_mode(true);

    let mut already_closed = false;

    if tty_name != "-" {
        let tty_group_id = match GroupEntry::by_groupname(&"tty".to_owned()) {
            Some(group) => group.gid,
            None => 0
        };

        let tty_path: PathBuf = {
            let mut tmp = PathBuf::from("/dev");
            tmp.push(tty_name);
            tmp
        };

        logger.debug().with_str("tty", tty_name).msg(format!("Opening {} as the new TTY", tty_path.display()));

        let fd = match open(&tty_path, OFlag::O_RDWR | OFlag::O_NOCTTY | OFlag::O_NONBLOCK, Mode::empty()) {
            Ok(fd) => fd,
            Err(err) => {
                logger.info().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to open terminal as stdin");
                return Err(Some(err));
            }
        };

        unsafe {
            if fchown(fd, 0, tty_group_id) != 0 || fchmod(fd, if tty_group_id != 0 {0o620} else {0o600}) != 0 {
                match Errno::last() {
                    Errno::EROFS => {
                        logger.debug().with_str("tty", tty_name).smsg("Failed to lock down terminal ownership");
                    },
                    _ => {
                        logger.info().with_str("tty", tty_name).smsg("Failed to lock down terminal ownership");
                        return Err(None);
                    }
                }
            }
        }

        // Some sanity checking
        // make sure it's a character device
        match fstat(fd) {
            Ok(fs) => {
                let dev_type = FileType::from_stat(fs).unwrap();
                if dev_type != FileType::CharacterDevice {
                    logger.info().with_str("tty", tty_name).smsg("TTY is not a terminal device");
                }
            },
            Err(err) => {
                logger.info().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to stat tty");
            }
        }

        // Make sure it's a tty
        match isatty(fd) {
            Ok(true) => {},
            Ok(false) => {
                logger.info().with_str("tty", tty_name).smsg("Given device is not a TTY");
                return Err(None);
            },
            Err(err) => {
                logger.info().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to check if tty is a terminal");
                return Err(Some(err));
            }
        };

        try_reparenting(fd, getpid(), &logger, tty_name);

        close(STDIN_FD)?;
        if options.should_hangup {
            unsafe {
                match tiocnotty(fd) {
                    Ok(0) => {},
                    Ok(err) => {
                        logger.debug().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("tiocnotty failed");
                    },
                    Err(err) => {
                        logger.debug().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("tiocnotty failed");
                    }
                }
            }

            close(fd)?;
            close(STDOUT_FD)?;
            close(STDERR_FD)?;
            already_closed = true;

            unsafe { 
                let hangup = vhangup();
                if hangup != 0 {
                    logger.info().with_str("tty", tty_name).with_string("error", hangup.to_string()).smsg("Failed to hangup exiting terminal. Bailing");
                    return Err(None);
                }
            }
        }
        else {
            close(fd)?;
        }

        match open(&tty_path, OFlag::O_RDWR | OFlag::O_NOCTTY | OFlag::O_NONBLOCK, Mode::empty()) {
            Ok(0) => {},
            Ok(fd) => {
                logger.info().with_str("tty", tty_name).msg(format!("Failed to open terminal as stdin - got FD {} instead", fd));
                return Err(None);
            }
            Err(err) => {
                logger.info().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to open terminal as stdin");
                return Err(Some(err));
            }
        };

        try_reparenting(STDIN_FD, getpid(), &logger, tty_name);
    }
    else {
        // STDIN is already an open terminal port. Sanity check it
        match fcntl(STDIN_FD, FcntlArg::F_GETFL) {
            Ok(res) => {
                if (res as u32) & S_IRWOTH != S_IRWOTH {
                    logger.info().with_str("tty", tty_name).smsg("Given FD isn't open for R/W");
                    return Err(None);
                }
            },
            Err(err) => {
                logger.info().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to stat given FD");
                return Err(Some(err));
            }
        }
    }

    match tcsetpgrp(STDIN_FD, getpid()) {
        Ok(()) => {},
        Err(err) => {
            logger.debug().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to set terminal process group");
        }
    }

    if !already_closed {
        close(STDOUT_FD)?;
        close(STDERR_FD)?;
    }

    match dup(STDIN_FD) {
        Ok(STDOUT_FD) => {},
        _ => {
            logger.info().with_str("tty", tty_name).smsg("Failed to create stdout stream");
        }
    }

    match dup(STDIN_FD) {
        Ok(STDERR_FD) => {},
        _ => {
            logger.info().with_str("tty", tty_name).smsg("Failed to create stderr stream");
        }
    }

    let termio_settings = match tcgetattr(STDIN_FD) {
        Ok(ts) => Ok(ts),
        Err(err) => {
            logger.info().with_str("tty", tty_name).with_string("error", err.to_string()).smsg("Failed to get terminal settings");
            return Err(Some(err));
        }
    };

    let mut kbmode: i32 = 0;

    unsafe {
        match kdgkbmode(STDIN_FD, &mut kbmode) {
            Ok(0) => {
                options.keyboard_mode = KeyboardMode::from(kbmode);
                options.is_vconsole = true;
                if options.term_type.is_none() {
                    options.term_type = Some("linux".to_owned());
                }
            }
            Ok(_) | Err(_) => {
                options.keyboard_mode = KeyboardMode::Raw;
                if options.term_type.is_none() {
                    options.term_type = Some("vt102".to_owned());
                }
            }
        }
    }


    return termio_settings;
}