use libq::terminal::{self, AnsiEscapeCode};
use libq::io::STDIN_FD;

use std::io::{self, stdout, Write, BufRead, StdoutLock};
use std::fs::File;
use std::cmp::{min, max};
use std::convert::TryFrom;

use shell::Shell;

use nix::unistd::read;

pub trait LineReader {
    fn next_line(&mut self, &mut String, &mut Shell) -> io::Result<usize>;
}

pub struct ShStdin {
    typed_buffer: String,
    history_index: usize,
    head: usize
}

pub struct InputFile {
    stream: io::BufReader<File>
}

impl InputFile {
    pub fn new(f: File) -> InputFile {
        return InputFile{
            stream: io::BufReader::new(f),
        };
    }
}

impl ShStdin {
    pub fn new() -> ShStdin {
        return ShStdin {
            typed_buffer: String::new(),
            history_index: 0,
            head: 0
        }
    }

    fn reset(&mut self) {
        self.typed_buffer.clear();
        self.history_index = 0;
        self.head = 0;
    }

    fn backspace(&mut self, dest: &mut String, stdout_handle: &mut StdoutLock) {
        if self.head == 0 {
            return; // Can't backspace if we're at the beginning
        }

        if self.head == dest.len() {
            dest.pop();
        }
        else {
            dest.remove(self.head-1);
        }

        self.head -= 1;

        write!(stdout_handle, "{}{}{}", terminal::BACKSPACE_BYTE as char, AnsiEscapeCode::EraseInLine(0), &dest[self.head..]);
        if dest.len() > self.head {
            write!(stdout_handle, "{}", AnsiEscapeCode::CursorBack((dest.len() - self.head) as u32));
        }
    }

    fn new_line(&self, dest: &mut String, stdout_handle: &mut StdoutLock) -> io::Result<()> {
        dest.push('\n');
        return write!(stdout_handle, "\n");
    }

    fn sigint(&self, dest: &mut String, shell: &mut Shell, stdout_handle: &mut StdoutLock) {
        shell.set_last_exit_code(130);
        write!(stdout_handle, "\n").expect("Failed to write to stdout");
        dest.clear();
        dest.push('\n');
    }

    fn move_cursor(&mut self, dest: &String, amt: i32, stdout_handle: &mut StdoutLock) {
        let bottom_clamp = -i32::try_from(self.head).unwrap();
        let top_clamp = (dest.len() - self.head) as i32;
        let new_amt = min(top_clamp, max(bottom_clamp, amt)); // Need to clamp amt to -head <= amt <= dest.len() - head
        self.head = (i32::try_from(self.head).unwrap() + new_amt) as usize;
        if new_amt > 0 {
            write!(stdout_handle, "{}", AnsiEscapeCode::CursorForward(amt as u32).to_string()).expect("Failed to write to stdout");
        }
        else if new_amt < 0 {
            write!(stdout_handle, "{}", AnsiEscapeCode::CursorBack((-amt) as u32).to_string()).expect("Failed to write to stdout");
        }
    }

    fn move_in_history(&mut self, dest: &mut String, shell: &Shell, amt: i32, stdout_handle: &mut StdoutLock) {
        let bottom_clamp = -i32::try_from(self.history_index).unwrap();
        let top_clamp = (shell.history_size() - self.history_index) as i32;
        let new_amt = min(top_clamp, max(bottom_clamp, amt)); // Need to clamp amt to -history_index <= amt <= shell.history_size() - history_index
        if new_amt == 0 {
            return;
        }

        if self.history_index == 0 {
            // If we're moving off the history head, store the currently typed text so we can come back to it
            self.typed_buffer = dest.clone();
        }

        self.history_index = (i32::try_from(self.history_index).unwrap() + new_amt) as usize;
        if self.head > 0 {
            write!(stdout_handle, "{}", AnsiEscapeCode::CursorBack(self.head as u32)).expect("Failed to write to stdout");
        }

        write!(stdout_handle, "{}", AnsiEscapeCode::EraseInLine(0)).expect("Failed to write to stdout");

        dest.clear();
        if self.history_index == 0 {
            dest.push_str(self.typed_buffer.as_str());
        }
        else {
            dest.push_str(shell.get_history_line(self.history_index - 1).unwrap().as_str());
        }

        write!(stdout_handle, "{}", dest).expect("Failed to write to stdout");
        self.head = dest.len();
    }

    fn handle_control_char(&mut self, dest: &mut String, shell: &mut Shell, byte: u8, stdout_handle: &mut StdoutLock) -> io::Result<()>{
        if byte == terminal::ESCAPE_CHAR {
            match AnsiEscapeCode::read_from_stdin() {
                Some(AnsiEscapeCode::CursorForward(n)) => {
                    self.move_cursor(dest, i32::try_from(n).unwrap(), stdout_handle);
                },
                Some(AnsiEscapeCode::CursorBack(n)) => {
                    self.move_cursor(dest, -(i32::try_from(n).unwrap()), stdout_handle);
                },
                Some(AnsiEscapeCode::CursorUp(n)) => {
                    self.move_in_history(dest, shell, i32::try_from(n).unwrap(), stdout_handle);
                },
                Some(AnsiEscapeCode::CursorDown(n)) => {
                    self.move_in_history(dest, shell, -(i32::try_from(n).unwrap()), stdout_handle);
                },
                _ => {}
            }
        }
        else {
            stdout_handle.write_all(&[byte]).expect("Failed to write to stdout");
        }

        return Ok(());
    }

    fn push_char(&mut self, dest: &mut String, byte: u8, stdout_handle: &mut StdoutLock) {
        let c: char = byte.into();
        if self.head == dest.len() {
            dest.push(c);
        }
        else {
            dest.insert(self.head, c);
        }
        
        self.head += 1;

        write!(stdout_handle, "{}{}", AnsiEscapeCode::EraseInLine(0), &dest[self.head..]).expect("Failed to write to stdout");
        if dest.len() > self.head {
            write!(stdout_handle, "{}", AnsiEscapeCode::CursorBack((dest.len() - self.head) as u32)).expect("Failed to write to stdout");
        }
    }
}

impl LineReader for ShStdin {
    fn next_line(&mut self, dest: &mut String, shell: &mut Shell) -> io::Result<usize> {
        dest.clear();
        self.reset();
        let stdout = stdout();
        let mut stdout = stdout.lock();

        let mut buffer: [u8; 1] = [0];

        while let Ok(n) = read(STDIN_FD, &mut buffer) {
            if n == 0 {
                if dest.len() == 0 {
                    return Ok(0);
                }
                continue;
            }
            let byte = buffer[0];
            match byte {
                terminal::EOF_BYTE => {
                    if dest.len() == 0 {
                        return Ok(0);
                    }
                },
                terminal::DELETE_BYTE | terminal::BACKSPACE_BYTE => {
                    self.backspace(dest, &mut stdout);
                },
                terminal::NEW_LINE_BYTE => {
                    self.new_line(dest, &mut stdout)?;
                    return Ok(dest.len() + 1);
                },
                terminal::CTRL_C_BYTE => {
                    self.sigint(dest, shell, &mut stdout);
                    return Ok(1);
                },
                byte if (byte as char).is_control() => {
                    self.handle_control_char(dest, shell, byte, &mut stdout)?;
                },
                byte => {
                    stdout.write_all(&[byte])?;
                    self.push_char(dest, byte, &mut stdout);
                },
            }

            stdout.flush()?;
        }

        return Ok(dest.len());
    }
}

impl LineReader for InputFile {
    fn next_line(&mut self, dest: &mut String, _shell: &mut Shell) -> io::Result<usize> {
        return self.stream.read_line(dest);
    }
}