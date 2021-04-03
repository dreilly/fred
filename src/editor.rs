use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use std::io::{stdout, Write};

use crate::{fred_file, term};

#[derive(Debug)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug)]
pub enum KeyState {
    Waiting(char),
    _WaitingForCommand(String),
    Inactive,
}

#[derive(Debug)]
pub struct Editor {
    pub lines: Vec<Line>,
    pub status: String,
    pub mode: EditorMode,
    pub v_draw_region: (usize, usize),
    pub h_draw_region: (usize, usize),
    pub draw_line: usize,
    pub key_state: KeyState,
    pub line_num_buf: Vec<char>,
}

#[derive(Debug)]
pub struct Line {
    pub line_chars: Vec<char>,
}

impl Line {
    fn insert_char_at_cursor(&mut self, i: usize, c: char) {
        self.line_chars.insert(i, c);
    }

    fn remove_char_at(&mut self, i: usize) {
        self.line_chars.remove(i - 1);
    }
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            lines: Vec::new(),
            status: "Normal".to_string(),
            mode: EditorMode::Normal,
            v_draw_region: (0, term::get_term_size().1),
            h_draw_region: (0, term::get_term_size().0),
            draw_line: 1,
            key_state: KeyState::Inactive,
            line_num_buf: Vec::new(),
        }
    }

    fn draw_line_numbers(&self, ln: usize) {
        let mut stdout = stdout();
        let pad = self.ln_pad() - ln.to_string().len();
        for _ in 0..pad {
            stdout.queue(Print(' ')).unwrap();
        }
        stdout.queue(Print(ln)).unwrap();

        stdout.queue(Print(' ')).unwrap();
        stdout.flush().unwrap();
    }

    fn update_line_num_buff(&mut self, n: usize) {
        let ln_as_string = n.to_string();
        self.line_num_buf = ln_as_string.chars().collect()
    }

    pub fn read_from_file(&mut self, f_name: String) {
        if let Ok(lines) = fred_file::read_lines(f_name) {
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
            self.update_line_num_buff(self.lines.len());
        }
    }

    pub fn draw_editor(&self, redraw: bool) -> Result<()> {
        let mut stdout = stdout();
        let region = self.v_draw_region;
        let mut region_end = region.1;
        if region_end > self.lines.len() {
            region_end = self.lines.len();
        }
        let iter = self.lines[region.0..region_end].iter().enumerate();
        for (pos, l) in iter {
            // let ln = iter.position(|x| x.1 == l);
            if pos >= region.1 - 1 {
                break;
            };
            stdout.queue(cursor::MoveToColumn(0))?;
            let ln = (region.0) + 1 + pos;
            self.draw_line_numbers(ln);
            for lc in &l.line_chars {
                stdout.queue(Print(lc))?;
            }
            stdout.queue(Print('\n'))?;
            stdout.flush()?;
        }

        self.draw_status();
        if !redraw {
            let pad = self.ln_pad();
            term::set_cursor_pos(pad as u16, 0);
        }
        Ok(())
    }

    fn draw_status(&self) {
        let draw_line = term::get_term_size().1;
        term::set_cursor_pos(0, draw_line as u16);
        let status_message = self.get_status_message();
        let mut stdout = stdout();
        stdout
            .queue(terminal::Clear(ClearType::CurrentLine))
            .unwrap();
        stdout.flush().unwrap();
        stdout
            .queue(Print(SetBackgroundColor(Color::DarkMagenta)))
            .unwrap();
        stdout
            .queue(Print(SetForegroundColor(Color::Black)))
            .unwrap();
        stdout.queue(Print(&status_message)).unwrap();
        stdout.queue(Print(ResetColor)).unwrap();
        stdout.flush().unwrap();
    }

    pub fn redraw(&self) -> Result<()> {
        let mut stdout = stdout();
        stdout.queue(terminal::Clear(ClearType::All))?;
        stdout.flush()?;
        self.draw_editor(true)
    }

    fn set_normal_mode(&mut self) {
        term::save_cursor_pos();
        self.mode = EditorMode::Normal;
        self.draw_status();
        term::restore_cursor_pos();
    }

    fn set_insert_mode(&mut self) {
        term::save_cursor_pos();
        self.mode = EditorMode::Insert;
        self.draw_status();
        term::restore_cursor_pos();
    }

    fn set_visual_mode(&mut self) {
        term::save_cursor_pos();
        self.mode = EditorMode::Visual;
        self.draw_status();
        term::restore_cursor_pos();
    }

    fn set_draw_line(&mut self, dl: usize) {
        self.draw_line = dl;
    }

    fn update_status(&mut self) {
        self.status = self.get_status_message();
    }

    fn update_v_draw_region(&mut self, start: usize, end: usize) {
        self.v_draw_region = (start, end)
    }

    fn update_key_state(&mut self, ks: KeyState) {
        term::save_cursor_pos();
        self.key_state = ks;
        self.draw_status();
        term::restore_cursor_pos();
    }

    fn get_key_state_text(&self) -> String {
        match self.key_state {
            KeyState::Waiting(c) => {
                format!("WAITING - {}", c)
            }
            _ => "INACTIVE".to_string(),
        }
    }

    fn get_status_message(&self) -> String {
        let ln_addend = if self.v_draw_region.0 > 0 {
            self.v_draw_region.0 + 1
        } else {
            self.v_draw_region.0
        };
        let term_size = term::get_term_size();
        let mut _status_text = String::new();
        let ln = self.draw_line + ln_addend;
        let ks = self.get_key_state_text();
        match self.mode {
            EditorMode::Normal => {
                _status_text = format!(
                    " NORMAL | Line: {}/{} | v_draw: {:?} h_draw: {:?} | DrawLine: {} | TermSize: {:?} | KeyState: {}",
                    ln,
                    self.lines.len(),
                    self.v_draw_region,
                    self.h_draw_region,
                    self.draw_line,
                    term_size,
                    ks,
                )
            }
            EditorMode::Insert => {
                _status_text = format!(
                    " INSERT | Line: {}/{} | v_draw: {:?} h_draw {:?}| DrawLine: {} | TermSize: {:?}",
                    ln,
                    self.lines.len(),
                    self.v_draw_region,
                    self.h_draw_region,
                    self.draw_line,
                    term_size,
                )
            }
            EditorMode::Visual => {
                _status_text = format!(
                    " VISUAL | Line: {}/{} | v_draw: {:?} h_draw {:?} | DrawLine: {} | TermSize: {:?}",
                    ln,
                    self.lines.len(),
                    self.v_draw_region,
                    self.h_draw_region,
                    self.draw_line,
                    term_size,
                )
            }
        }
        let pad = self.status_padding(_status_text.len(), term_size.0);
        format!("{}{}", _status_text, pad)
    }

    fn status_padding(&self, status_len: usize, term_width: usize) -> String {
        let mut pad = String::new();
        let pad_len = term_width - status_len;
        if status_len <= 0 {
            return pad;
        }

        // TODO - Is there a more idomatic way of doing this?
        for _ in 0..pad_len {
            pad = format!("{} ", pad);
        }

        pad
    }

    fn move_down(&mut self) {
        if self.v_draw_region.1 < self.lines.len() || self.draw_line < term::get_term_size().1 - 1 {
            let mut pos = cursor::position().unwrap().1 + 1;
            let term_size = term::get_term_size().1 as u16;
            let mut redraw_status_only = true;
            if pos < term_size - 1 {
                let mut stdout = stdout();
                stdout.queue(cursor::MoveDown(1)).unwrap();
                stdout.flush().unwrap();
                pos += 1;
            } else {
                let region = self.v_draw_region;
                self.update_v_draw_region(region.0 + 1, region.1 + 1);
                redraw_status_only = false;
            }
            self.set_draw_line(pos as usize);
            self.update_status();
            term::save_cursor_pos();
            if redraw_status_only {
                self.draw_status();
            } else {
                self.redraw().unwrap();
            }
            term::restore_cursor_pos();

            self.clamp_to_end_of_line();
        }
    }

    fn move_up(&mut self) {
        if self.v_draw_region.0 > 0 || self.draw_line > 1 {
            let mut pos = cursor::position().unwrap().1 + 1;
            let mut redraw_status_only = true;
            if pos > 1 {
                let mut stdout = stdout();
                stdout.queue(cursor::MoveUp(1)).unwrap();
                stdout.flush().unwrap();
                pos -= 1;
            } else {
                let region = self.v_draw_region;
                self.update_v_draw_region(region.0 - 1, region.1 - 1);
                let mut stdout = stdout();
                stdout.queue(cursor::MoveUp(1)).unwrap();
                stdout.flush().unwrap();
                pos -= 1;
                redraw_status_only = false;
            }
            self.set_draw_line(pos as usize);
            self.update_status();
            term::save_cursor_pos();
            if redraw_status_only {
                self.draw_status();
            } else {
                self.redraw().unwrap();
            }
            term::restore_cursor_pos();

            self.clamp_to_end_of_line();
        }
    }

    fn move_right(&mut self) {
        let pos = cursor::position().unwrap();
        let line = &self.get_line_from_cursor().line_chars;
        let line_len = line.len();
        if line_len > 0 {
            let stop_point = if line_len >= 2 {
                // this is len() - 2 because it assumes the last char in the vec is a new line character
                line_len - 2
            } else {
                line_len - 1
            };
            let pad = self.ln_pad() + 1;
            if pos.0 as usize <= (stop_point + pad) {
                let mut stdout = stdout();
                stdout.queue(cursor::MoveRight(1)).unwrap();
                stdout.flush().unwrap();
            }
        }
    }

    fn move_left(&self) {
        let pos = cursor::position().unwrap().0 as usize;
        let start_pos = self.line_num_buf.len() + 1;
        if pos > start_pos {
            let mut stdout = stdout();
            stdout.queue(cursor::MoveLeft(1)).unwrap();
            stdout.flush().unwrap();
        }
    }

    fn clamp_to_start_of_line(&mut self) {
        let new_pos = cursor::position().unwrap();
        let start_pos = self.line_num_buf.len() + 2;
        if new_pos.0 as usize <= start_pos {
            term::move_to_column(start_pos as u16);
        }
    }

    fn clamp_to_end_of_line(&mut self) {
        let new_pos = cursor::position().unwrap();
        let line_len = self.get_line_from_cursor().line_chars.len();
        let line_len_offset = if line_len == 0 {
            line_len + self.line_num_buf.len() + 2
        } else {
            line_len + self.line_num_buf.len() + 1
        };
        if new_pos.0 as usize > line_len {
            term::move_to_column(line_len_offset as u16);
        }
    }

    pub fn get_line_from_cursor(&mut self) -> &mut Line {
        let addend = if self.v_draw_region.0 == 0 {
            self.draw_line - 1
        } else {
            self.draw_line
        };
        let point = addend + self.v_draw_region.0;
        &mut self.lines[point]
    }

    fn get_line_num_buff_size(&self) -> usize {
        self.line_num_buf.len()
    }

    fn ln_pad(&self) -> usize {
        self.lines.len().to_string().len() + 1
    }

    pub fn handle_input(&mut self) -> Result<()> {
        loop {
            match self.mode {
                EditorMode::Insert => match read()? {
                    Event::Key(KeyEvent { code, modifiers: _ }) => match code {
                        KeyCode::Esc => {
                            self.set_normal_mode();
                        }
                        KeyCode::Tab => {
                            let pos = cursor::position()?;
                            let line = self.get_line_from_cursor();
                            line.insert_char_at_cursor(pos.0 as usize, '\t');
                            term::save_cursor_pos();
                            self.redraw()?;
                            term::restore_cursor_pos();
                            term::set_cursor_pos(pos.0 + 1, pos.1);
                        }
                        KeyCode::Backspace => {
                            let pos = cursor::position()?;
                            if pos.0 > 0 {
                                let pad = self.ln_pad() + 1;
                                let line = self.get_line_from_cursor();
                                line.remove_char_at(pos.0 as usize - pad);
                                term::save_cursor_pos();
                                self.redraw()?;
                                term::restore_cursor_pos();
                                self.move_left();
                            }
                        }
                        KeyCode::Char(c) => match c {
                            _ => {
                                let pos = cursor::position()?;
                                let pad = self.ln_pad();
                                let line = self.get_line_from_cursor();
                                line.insert_char_at_cursor(pos.0 as usize - pad, c);
                                term::save_cursor_pos();
                                self.redraw()?;
                                term::restore_cursor_pos();
                                term::set_cursor_pos(pos.0 + 1, pos.1);
                            }
                        },
                        _ => {}
                    },
                    _ => {}
                },
                _ => {
                    match read()? {
                        Event::Key(KeyEvent {
                            code,
                            modifiers: KeyModifiers::CONTROL,
                        }) => match code {
                            _ => {}
                        },
                        Event::Key(KeyEvent { code, modifiers: _ }) => {
                            match code {
                                KeyCode::Char(c) => match c {
                                    'h' => {
                                        self.move_left();
                                    }
                                    'j' => {
                                        self.move_down();
                                    }
                                    'k' => {
                                        self.move_up();
                                    }
                                    'l' => {
                                        self.move_right();
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
                                    'g' => match self.key_state {
                                        KeyState::Inactive => {
                                            self.update_key_state(KeyState::Waiting(c));
                                        }
                                        KeyState::Waiting(_) => {
                                            let x = cursor::position()?.0;
                                            self.update_v_draw_region(0, term::get_term_size().1);
                                            self.set_draw_line(1);
                                            self.redraw()?;
                                            term::set_cursor_pos(x, 0);
                                            self.update_key_state(KeyState::Inactive);
                                        }
                                        _ => {}
                                    },
                                    'G' => {
                                        // TODO this will break on files with less lines than the
                                        // terminal size.
                                        let x = cursor::position().unwrap().0;
                                        let ts = term::get_term_size();
                                        let draw_region_start = self.lines.len() - ts.1;
                                        self.update_v_draw_region(
                                            draw_region_start,
                                            self.lines.len(),
                                        );
                                        self.set_draw_line(ts.1 - 1);
                                        self.redraw()?;
                                        term::set_cursor_pos(x, (ts.1 - 2) as u16);
                                        self.update_key_state(KeyState::Inactive);
                                    }
                                    'q' => {
                                        match self.key_state {
                                            KeyState::Waiting(cmd) => match cmd {
                                                ':' => {
                                                    //TODO prompt user before exiting
                                                    //TODO expect enter to follow like vim?
                                                    terminal::Clear(ClearType::All);
                                                    break;
                                                }
                                                _ => {}
                                            },
                                            _ => {}
                                        }
                                    }
                                    ':' => match self.key_state {
                                        KeyState::Inactive => {
                                            self.update_key_state(KeyState::Waiting(c));
                                        }
                                        KeyState::Waiting(_) => {
                                            self.update_key_state(KeyState::Inactive);
                                        }
                                        _ => {}
                                    },
                                    '0' => {
                                        // let pos = cursor::position()?;
                                        // term::set_cursor_pos(0, pos.1);
                                        term::move_to_column(0);
                                    }
                                    _ => {}
                                },
                                KeyCode::Enter => {}
                                _ => {}
                            };
                        }
                        Event::Mouse(_event) => {}
                        Event::Resize(_width, _height) => {}
                    }
                }
            }
        }
        Ok(())
    }
}
