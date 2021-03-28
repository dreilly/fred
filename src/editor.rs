use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use std::io::{stdout, Write};

use crate::term;

pub enum EditorMode {
    Normal,
    Insert,
    Visual,
}
pub struct Editor {
    pub lines: Vec<Line>,
    pub status: String,
    pub mode: EditorMode,
    pub draw_region: (usize, usize),
    pub draw_line: usize,
}

pub struct Line {
    pub line_chars: Vec<char>,
}

impl Editor {
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
        //let stats_bar_background = Color::Rgb{ };

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

    pub fn set_insert_mode(&mut self) {
        self.mode = EditorMode::Insert
    }

    pub fn set_visual_mode(&mut self) {
        self.mode = EditorMode::Visual
    }

    pub fn set_draw_line(&mut self, dl: usize) {
        self.draw_line = dl;
    }

    pub fn update_status(&mut self) {
        self.status = self.get_status_message();
    }

    pub fn update_draw_region(&mut self, start: usize, end: usize) {
        self.draw_region = (start, end)
    }

    pub fn get_status_message(&self) -> String {
        let ln_addend = if self.draw_region.0 > 0 {
            self.draw_region.0 + 1
        } else {
            self.draw_region.0
        };
        let term_size = term::get_term_size();
        let mut status_text = String::with_capacity(term_size.0);
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
}
