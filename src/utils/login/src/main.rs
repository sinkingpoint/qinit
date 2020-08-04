extern crate clap;
extern crate libq;
extern crate nix;

use clap::{App, Arg};

use libq::io::STDIN_FD;
use libq::logger;
use libq::passwd::{PasswdEntry, ShadowEntry};
use libq::terminal::{reset_virtual_console, set_echo_mode};

use nix::errno::Errno;
use nix::sys::termios::{tcgetattr, tcsetattr, SetArg, Termios};
use nix::unistd::{execve, setgid, setuid, Gid, Uid};

use std::ffi::{CStr, CString};

use std::io::{self, BufReader, Read, StdoutLock, Write};

const PASSWORD_ATTEMPTS: u8 = 5;

fn main() {
    let logger = logger::with_name_as_json("login");
    let args = App::new("login")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Start new sessions on the system")
        .arg(
            Arg::with_name("nodestroy")
                .short("p")
                .help("Don't clear envronment variables after authentication"),
        )
        .arg(Arg::with_name("username").index(1).help("The username to login as").required(true))
        .get_matches();

    let username = args.value_of("username").unwrap();
    let user_passwd = match PasswdEntry::by_username_str(username) {
        Some(user) => user,
        None => {
            logger.info().msg(format!("Failed to find user with name `{}`", username));
            return;
        }
    };

    let user_shadow = match ShadowEntry::by_username_str(username) {
        Some(user) => user,
        None => {
            logger.info().msg(format!("Failed to find user with name `{}`", username));
            return;
        }
    };

    let mut termio_settings = match tcgetattr(STDIN_FD) {
        Ok(ts) => ts,
        Err(err) => {
            logger
                .info()
                .with_string("error", err.to_string())
                .msg(format!("Failed to find user with name `{}`", username));
            return;
        }
    };

    match disable_echo(&mut termio_settings) {
        Ok(_) => {}
        Err(()) => {
            logger.info().msg(format!("Failed to disable terminal echoing"));
            return;
        }
    }

    let mut successful = false;
    for _ in 0..PASSWORD_ATTEMPTS {
        let password = read_password().unwrap();
        if user_shadow.password_hash.verify(&password) {
            successful = true;
            break;
        } else {
            print!("\nPassword incorrect!\n");
        }
    }

    match reset_terminal(&mut termio_settings) {
        Ok(_) => {}
        Err(()) => {
            logger.info().msg(format!("Failed to reset terminal modes"));
            return;
        }
    }

    if successful {
        match setgid(Gid::from_raw(user_passwd.gid)) {
            Ok(_) => {}
            Err(err) => {
                logger.info().with_string("error", err.to_string()).smsg("Failed to set GID");
                return;
            }
        }

        match setuid(Uid::from_raw(user_passwd.uid)) {
            Ok(_) => {}
            Err(err) => {
                logger.info().with_string("error", err.to_string()).smsg("Failed to set UID");
                return;
            }
        }

        let shell = user_passwd.shell.to_str().unwrap();
        let shell_bin = CString::new(shell).unwrap();
        let args: Vec<Vec<u8>> = [shell]
            .iter()
            .map(|arg| CString::new(*arg).unwrap().into_bytes_with_nul())
            .collect();
        let args = &args
            .iter()
            .map(|arg| CStr::from_bytes_with_nul(arg).unwrap())
            .collect::<Vec<&CStr>>()[..];
        let env: Vec<Vec<u8>> = [format!("PATH=/bin").as_str()]
            .iter()
            .map(|env| CString::new(*env).unwrap().into_bytes_with_nul())
            .collect();
        let env = &env
            .iter()
            .map(|env| CStr::from_bytes_with_nul(env).unwrap())
            .collect::<Vec<&CStr>>()[..];

        match execve(&shell_bin, &args[..], &env[..]) {
            Ok(_) => {} // We should never get here. A sucessful execvp will never get here as it will be running the other program
            Err(err) => {
                if let Some(errno) = err.as_errno() {
                    if errno == Errno::ENOENT {
                        eprintln!("No such command: {}", "/sbin/login");
                        std::process::exit(127);
                    }
                } else {
                    logger
                        .debug()
                        .with_string("error", err.to_string())
                        .smsg("Failed to exec login process");
                }
            }
        }
    }
}

fn prompt(lock: &mut StdoutLock) {
    print!("Password: ");
    lock.flush().expect("Failed to flush stdout");
}

fn reset_terminal(term_settings: &mut Termios) -> Result<(), ()> {
    reset_virtual_console(term_settings, true, true);
    match tcsetattr(STDIN_FD, SetArg::TCSADRAIN, &term_settings) {
        Ok(_) => {}
        Err(_) => {
            return Err(());
        }
    }

    return Ok(());
}

fn disable_echo(term_settings: &mut Termios) -> Result<(), ()> {
    set_echo_mode(term_settings, false);
    match tcsetattr(STDIN_FD, SetArg::TCSADRAIN, &term_settings) {
        Ok(_) => {}
        Err(_) => {
            return Err(());
        }
    }

    return Ok(());
}

fn read_password() -> Result<String, Option<io::Error>> {
    let logger = logger::with_name_as_json("login;read_password");

    let reader = BufReader::new(io::stdin());
    let mut build = Vec::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    prompt(&mut handle);

    for byte in reader.bytes() {
        let byte = match byte {
            Ok(b) => b,
            Err(err) => {
                logger
                    .info()
                    .with_string("error", err.to_string())
                    .smsg("Failed to read from stdin");
                return Err(Some(err));
            }
        };

        match byte {
            // 0x0D == CR , 0x0A == NL. Terminate the username read
            0x0D | 0x0A => {
                print!("\n");
                if build.len() > 0 {
                    return Ok(String::from_utf8_lossy(&build[..]).to_string());
                } else {
                    prompt(&mut handle);
                }
            }
            // 0x7f == DEL , 0x08 == Backspace. Delete the last char (if it exists)
            0x7F | 0x08 => match build.pop() {
                Some(_) => {}
                None => {}
            },
            c => {
                build.push(c);
            }
        }

        handle.flush()?;
    }

    loop {}
}
