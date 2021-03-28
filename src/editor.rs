use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use std::io::{stdout, Write};

use crate::{fredFile, term};

pub enum EditorMode {
    Normal,
    Insert,
    Visual,
}

pub enum KeyState {
    Waiting(char),
    Inactive,
}

pub struct Editor {
    pub lines: Vec<Line>,
    pub status: String,
    pub mode: EditorMode,
    pub draw_region: (usize, usize),
    pub draw_line: usize,
    pub key_state: KeyState,
}

pub struct Line {
    pub line_chars: Vec<char>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            lines: Vec::new(),
            status: "Normal".to_string(),
            mode: EditorMode::Normal,
            draw_region: (0, term::get_term_size().1),
            draw_line: 1,
            key_state: KeyState::Inactive,
        }
    }

    pub fn read_from_file(&mut self, f_name: String) {
        if let Ok(lines) = fredFile::read_lines(f_name) {
            for row in lines {
                let mut line: Line = Line {
                    line_chars: Vec::new(),
                };
                if let Ok(r) = row {
                    for c in r.chars() {
                        line.line_chars.push(c);
                    }
                }
                self.lines.push(line);
            }
        }
    }
    pub fn draw_editor(&self, redraw: bool) -> Result<()> {
        let mut stdout = stdout();
        let region = self.draw_region;
        let mut region_end = region.1;
        if region_end > self.lines.len() {
            region_end = self.lines.len();
        }
        let iter = self.lines[region.0..region_end].iter().enumerate();
        for (pos, l) in iter {
            if pos >= region.1 - 1 {
                break;
            };
            stdout.queue(cursor::MoveToColumn(0))?;
            for lc in &l.line_chars {
                match lc {
                    _ => {
                        stdout.queue(Print(lc))?;
                    }
                }
            }
            stdout.queue(Print('\n'))?;
            stdout.flush()?;
        }

        let status_message = self.get_status_message();
        let pos = cursor::position().unwrap();
        term::set_cursor_pos(0, pos.1);

        stdout.queue(Print(SetBackgroundColor(Color::DarkMagenta)))?;
        stdout.queue(Print(SetForegroundColor(Color::Black)))?;
        stdout.queue(Print(&status_message))?;
        stdout.queue(Print(ResetColor))?;
        stdout.flush()?;

        if !redraw {
            term::set_cursor_pos(0, 0);
        }
        Ok(())
    }

    pub fn redraw(&self) -> Result<()> {
        let mut stdout = stdout();
        stdout.queue(terminal::Clear(ClearType::All))?;
        stdout.flush()?;
        self.draw_editor(true)
    }

    fn set_insert_mode(&mut self) {
        self.mode = EditorMode::Insert
    }

    fn set_visual_mode(&mut self) {
        self.mode = EditorMode::Visual
    }

    fn set_draw_line(&mut self, dl: usize) {
        self.draw_line = dl;
    }

    fn update_status(&mut self) {
        self.status = self.get_status_message();
    }

    fn update_draw_region(&mut self, start: usize, end: usize) {
        self.draw_region = (start, end)
    }

    fn get_status_message(&self) -> String {
        let ln_addend = if self.draw_region.0 > 0 {
            self.draw_region.0 + 1
        } else {
            self.draw_region.0
        };
        let term_size = term::get_term_size();
        let mut status_text = String::new();
        let ln = self.draw_line + ln_addend;
        match self.mode {
            EditorMode::Normal => {
                status_text = format!(
                    " NORMAL | Line: {}/{} | DrawRegion: {:?} | DrawLine: {} | TermSize: {:?}",
                    ln,
                    self.lines.len(),
                    self.draw_region,
                    self.draw_line,
                    term_size,
                )
            }
            EditorMode::Insert => {
                status_text = format!(
                    " INSERT | Line: {}/{} | DrawRegion: {:?} | DrawLine: {} | TermSize: {:?}",
                    ln,
                    self.lines.len(),
                    self.draw_region,
                    self.draw_line,
                    term_size,
                )
            }
            EditorMode::Visual => {
                status_text = format!(
                    " VISUAL | Line: {}/{} | DrawRegion: {:?} | DrawLine: {} | TermSize: {:?}",
                    ln,
                    self.lines.len(),
                    self.draw_region,
                    self.draw_line,
                    term_size,
                )
            }
        }
        let pad = self.status_padding(status_text.len(), term_size.0);
        format!("{}{}", status_text, pad)
    }

    fn status_padding(&self, status_len: usize, term_width: usize) -> String {
        let mut pad = String::new();
        let pad_len = term_width - status_len;
        if status_len <= 0 {
            return pad;
        }

        // TODO - Is there a idomatic way of doing this?
        for _ in 0..pad_len {
            pad = format!("{} ", pad);
        }

        pad
    }

    pub fn handle_input(&mut self) -> Result<()> {
        loop {
            match read()? {
                Event::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::CONTROL,
                }) => match code {
                    KeyCode::Char('q') => {
                        terminal::Clear(ClearType::All);
                        break;
                    }
                    _ => {}
                },
                Event::Key(KeyEvent { code, modifiers: _ }) => {
                    match code {
                        KeyCode::Char(c) => match c {
                            'h' => {
                                let mut stdout = stdout();
                                stdout.queue(cursor::MoveLeft(1))?;
                                stdout.flush()?;
                            }
                            'j' => {
                                if self.draw_region.1 < self.lines.len()
                                    || self.draw_line < term::get_term_size().1 - 1
                                {
                                    let mut pos = cursor::position().unwrap().1 + 1;
                                    let term_size = term::get_term_size().1 as u16;
                                    if pos < term_size - 1 {
                                        let mut stdout = stdout();
                                        stdout.queue(cursor::MoveDown(1))?;
                                        stdout.flush()?;
                                        pos += 1;
                                    } else {
                                        let region = self.draw_region;
                                        self.update_draw_region(region.0 + 1, region.1 + 1);
                                    }
                                    self.set_draw_line(pos as usize);
                                    self.update_status();
                                    term::save_cursor_pos();
                                    self.redraw()?;
                                    term::restore_cursor_pos();
                                }
                            }
                            'k' => {
                                if self.draw_region.0 > 0 || self.draw_line > 1 {
                                    let mut pos = cursor::position().unwrap().1 + 1;
                                    if pos > 1 {
                                        let mut stdout = stdout();
                                        stdout.queue(cursor::MoveUp(1))?;
                                        stdout.flush()?;
                                        pos -= 1;
                                    } else {
                                        let region = self.draw_region;
                                        self.update_draw_region(region.0 - 1, region.1 - 1);
                                        let mut stdout = stdout();
                                        stdout.queue(cursor::MoveUp(1))?;
                                        stdout.flush()?;
                                        pos -= 1;
                                    }
                                    self.set_draw_line(pos as usize);
                                    self.update_status();
                                    term::save_cursor_pos();
                                    self.redraw()?;
                                    term::restore_cursor_pos();
                                }
                            }
                            'l' => {
                                let mut stdout = stdout();
                                stdout.queue(cursor::MoveRight(1))?;
                                stdout.flush()?;
                            }
                            'i' => {
                                self.set_insert_mode();
                            }
                            'v' => {
                                self.set_visual_mode();
                            }
                            'a' => match self.mode {
                                EditorMode::Insert => {}
                                _ => {}
                            },
                            'g' => {
                                // TODO update keystate here to await next command
                                // if valid next key  execute move
                                // else clear key state and exit
                            }
                            ':' => {
                                // TODO command entry
                            }
                            _ => {}
                        },
                        KeyCode::Enter => {}
                        KeyCode::Up => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveUp(1))?;
                            stdout.flush()?;
                        }
                        KeyCode::Down => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveDown(1))?;
                            stdout.flush()?;
                        }
                        KeyCode::Left => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveLeft(1))?;
                            stdout.flush()?;
                        }
                        KeyCode::Right => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveRight(1))?;
                            stdout.flush()?;
                        }
                        _ => {}
                    };
                }
                Event::Mouse(_event) => {}
                Event::Resize(_width, _height) => {}
            }
        }

        Ok(())
    }
}
