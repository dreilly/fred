use crossterm::{
    cursor,
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
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

pub fn init_term() -> Result<()> {
    let mut stdout = stdout();
    stdout.queue(terminal::EnterAlternateScreen)?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.flush()?;
    terminal::enable_raw_mode()?;

    Ok(())
}

#[allow(dead_code)]
pub fn set_cursor_blink() {
    let mut stdout = stdout();
    stdout.queue(cursor::EnableBlinking).unwrap();
    stdout.flush().unwrap();
}

#[allow(dead_code)]
pub fn set_cursor_solid() {
    let mut stdout = stdout();
    stdout.queue(cursor::DisableBlinking).unwrap();
    stdout.flush().unwrap();
}

pub fn die() -> Result<()> {
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}
