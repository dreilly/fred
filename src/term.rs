use crossterm::{cursor, terminal, QueueableCommand};
use std::io::{stdout, Write};

pub fn save_cursor_pos() {
    let mut stdout = stdout();
    stdout.queue(cursor::SavePosition).unwrap();
    stdout.flush().unwrap();
}

pub fn restore_cursor_pos() {
    let mut stdout = stdout();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
}

pub fn set_cursor_pos(x: u16, y: u16) {
    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo(x, y)).unwrap();
    stdout.flush().unwrap();
}

pub fn get_term_size() -> (usize, usize) {
    let term_size = terminal::size().unwrap();
    (term_size.0 as usize, term_size.1 as usize)
}
