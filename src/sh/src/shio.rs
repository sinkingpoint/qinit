use libq::terminal::{self, AnsiEscapeCode};

use std::io::{self, stdin, Stdin, stdout, BufReader, Read, Write, BufRead};
use std::fs::File;
use std::cmp::{min, max};
use std::convert::TryFrom;

use shell::Shell;

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

    fn backspace(&mut self, dest: &mut String) {
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

        print!("{}{}{}", terminal::BACKSPACE_BYTE as char, AnsiEscapeCode::EraseInLine(0), &dest[self.head..]);
        if dest.len() > self.head {
            print!("{}", AnsiEscapeCode::CursorBack((dest.len() - self.head) as u32));
        }
    }

    fn new_line(&self, dest: &mut String) {
        stdout().write_all(b"\n");
        dest.push('\n');
    }

    fn sigint(&self, dest: &mut String, shell: &mut Shell) {
        shell.set_last_exit_code(130);
        print!("\n");
        dest.clear();
        dest.push('\n');
    }

    fn move_cursor(&mut self, dest: &String, amt: i32) {
        let bottom_clamp = -i32::try_from(self.head).unwrap();
        let top_clamp = (dest.len() - self.head) as i32;
        let new_amt = min(top_clamp, max(bottom_clamp, amt)); // Need to clamp amt to -head <= amt <= dest.len() - head
        self.head = (i32::try_from(self.head).unwrap() + new_amt) as usize;
        if new_amt > 0 {
            print!("{}", AnsiEscapeCode::CursorForward(amt as u32).to_string());
        }
        else if new_amt < 0 {
            print!("{}", AnsiEscapeCode::CursorBack((-amt) as u32).to_string());
        }
    }

    fn move_in_history(&mut self, dest: &mut String, shell: &Shell, amt: i32) {
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
            print!("{}", AnsiEscapeCode::CursorBack(self.head as u32));
        }

        print!("{}", AnsiEscapeCode::EraseInLine(0));

        dest.clear();
        if self.history_index == 0 {
            dest.push_str(self.typed_buffer.as_str());
        }
        else {
            dest.push_str(shell.get_history_line(self.history_index - 1).unwrap().as_str());
        }

        print!("{}", dest);
        self.head = dest.len();
    }

    fn handle_control_char(&mut self, dest: &mut String, shell: &mut Shell, byte: u8) {
        let c: char = byte.into();
        if byte == terminal::ESCAPE_CHAR {
            match AnsiEscapeCode::read_from_stdin() {
                Some(AnsiEscapeCode::CursorForward(n)) => {
                    self.move_cursor(dest, (i32::try_from(n).unwrap()));
                },
                Some(AnsiEscapeCode::CursorBack(n)) => {
                    self.move_cursor(dest, -(i32::try_from(n).unwrap()));
                },
                Some(AnsiEscapeCode::CursorUp(n)) => {
                    self.move_in_history(dest, shell, (i32::try_from(n).unwrap()));
                },
                Some(AnsiEscapeCode::CursorDown(n)) => {
                    self.move_in_history(dest, shell, -(i32::try_from(n).unwrap()));
                },
                _ => {}
            }
        }
        else {
            stdout().write_all(&[byte]);
        }
    }

    fn push_char(&mut self, dest: &mut String, byte: u8) {
        let c: char = byte.into();
        if self.head == dest.len() {
            dest.push(c);
        }
        else {
            dest.insert(self.head, c);
        }
        
        self.head += 1;

        print!("{}{}", AnsiEscapeCode::EraseInLine(0), &dest[self.head..]);
        if dest.len() > self.head {
            print!("{}", AnsiEscapeCode::CursorBack((dest.len() - self.head) as u32));
        }
    }
}

impl LineReader for ShStdin {
    fn next_line(&mut self, dest: &mut String, shell: &mut Shell) -> io::Result<usize> {
        dest.clear();
        self.reset();
        let mut stdout = stdout();
        let mut stdin = stdin();

        let mut buffer: [u8; 1] = [0];

        while let Ok(n) = stdin.read(&mut buffer) {
            if n == 0 {
                // We've hit EOF
                return Ok(0);
            }
            let byte = buffer[0];
            match byte {
                terminal::EOF_BYTE => {
                    if dest.len() == 0 {
                        return Ok(0);
                    }
                },
                terminal::DELETE_BYTE | terminal::BACKSPACE_BYTE => {
                    self.backspace(dest);
                },
                terminal::NEW_LINE_BYTE => {
                    self.new_line(dest);
                    return Ok(dest.len() + 1);
                },
                terminal::CTRL_C_BYTE => {
                    self.sigint(dest, shell);
                    return Ok(1);
                },
                byte if (byte as char).is_control() => {
                    self.handle_control_char(dest, shell, byte);
                },
                byte => {
                    stdout.write_all(&[byte]);
                    self.push_char(dest, byte);
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