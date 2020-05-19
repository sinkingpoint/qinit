extern crate libq;

use libq::io::STDOUT_FD;
use libq::terminal;

fn main() {
    terminal::set_cursor_position(STDOUT_FD, 1, 1).unwrap();
    terminal::erase_display(STDOUT_FD, terminal::EraseDisplayMode::All).unwrap();
}