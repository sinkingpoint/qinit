extern crate libq;
use std::io;

use libq::terminal;

fn main() {
    terminal::set_cursor_position(&mut io::stdout(), 1, 1).unwrap();
    terminal::erase_display(&mut io::stdout(), terminal::EraseDisplayMode::All).unwrap();
}
