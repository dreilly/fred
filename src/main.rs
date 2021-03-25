use core::panic;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use std::fs::File;
use std::io::{self, stdout, BufRead, Write};
use std::path::Path;

enum EditorMode {
    Normal,
    Insert,
    Visual,
}
struct Editor {
    lines: Vec<Line>,
    status: String,
    mode: EditorMode,
    draw_region: (usize, usize),
    draw_line: usize,
}

struct Line {
    line_chars: Vec<char>,
}

impl Editor {
    fn draw_editor(&self, redraw: bool) -> Result<()> {
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
        set_cursor_pos(0, pos.1);
        stdout.queue(Print(&status_message))?;
        stdout.flush()?;

        if !redraw {
            set_cursor_pos(0, 0);
        }
        Ok(())
    }

    fn redraw(&self) -> Result<()> {
        let mut stdout = stdout();
        stdout.queue(terminal::Clear(ClearType::All))?;
        stdout.flush()?;
        self.draw_editor(true)
    }

    fn set_insert_mode(&mut self) {
        self.mode = EditorMode::Insert
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
        let ln = self.draw_line + ln_addend;
        match self.mode {
            EditorMode::Normal => {
                format!(
                    "NORMAL | Line: {}/{} | DrawRegion: {:?} | DrawLine: {} | TermSize: {:?}",
                    ln,
                    self.lines.len(),
                    self.draw_region,
                    self.draw_line,
                    get_term_size(),
                )
            }
            EditorMode::Insert => {
                format!("INSERT | {}", ln)
            }
            EditorMode::Visual => {
                format!("VISUAL | {}", ln)
            }
        }
    }
}

fn init_editor() -> Result<Editor> {
    let mut stdout = stdout();
    stdout.queue(terminal::EnterAlternateScreen)?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.flush()?;
    terminal::enable_raw_mode()?;
    let editor = Editor {
        lines: Vec::new(),
        status: "Normal".to_string(),
        mode: EditorMode::Normal,
        draw_region: (0, get_term_size().1),
        draw_line: 1,
    };
    Ok(editor)
}

fn die() -> Result<()> {
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename);
    match file {
        Err(e) => panic!("{}", e),
        Ok(f) => Ok(io::BufReader::new(f).lines()),
    }
}

fn save_cursor_pos() {
    let mut stdout = stdout();
    stdout.queue(cursor::SavePosition).unwrap();
    stdout.flush().unwrap();
}

fn restore_cursor_pos() {
    let mut stdout = stdout();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
}

fn set_cursor_pos(x: u16, y: u16) {
    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo(x, y)).unwrap();
    stdout.flush().unwrap();
}

fn get_term_size() -> (usize, usize) {
    let term_size = terminal::size().unwrap();
    (term_size.0 as usize, term_size.1 as usize)
}

fn main() -> Result<()> {
    let mut editor = init_editor()?;

    if let Ok(lines) = read_lines("src/main.rs") {
        for row in lines {
            let mut line: Line = Line {
                line_chars: Vec::new(),
            };
            if let Ok(r) = row {
                for c in r.chars() {
                    line.line_chars.push(c);
                }
            }
            editor.lines.push(line);
        }
    }

    editor.draw_editor(false)?;

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
                            if editor.draw_region.1 < editor.lines.len()
                                || editor.draw_line < get_term_size().1 - 1
                            {
                                let mut pos = cursor::position().unwrap().1 + 1;
                                let term_size = get_term_size().1 as u16;
                                if pos < term_size - 1 {
                                    let mut stdout = stdout();
                                    stdout.queue(cursor::MoveDown(1))?;
                                    stdout.flush()?;
                                    pos += 1;
                                } else {
                                    let region = editor.draw_region;
                                    editor.update_draw_region(region.0 + 1, region.1 + 1);
                                }
                                editor.set_draw_line(pos as usize);
                                editor.update_status();
                                save_cursor_pos();
                                editor.redraw()?;
                                restore_cursor_pos();
                            }
                        }
                        'k' => {
                            if editor.draw_region.0 > 0 || editor.draw_line > 1 {
                                let mut pos = cursor::position().unwrap().1 + 1;
                                if pos > 1 {
                                    let mut stdout = stdout();
                                    stdout.queue(cursor::MoveUp(1))?;
                                    stdout.flush()?;
                                    pos -= 1;
                                } else {
                                    let region = editor.draw_region;
                                    editor.update_draw_region(region.0 - 1, region.1 - 1);
                                    let mut stdout = stdout();
                                    stdout.queue(cursor::MoveUp(1))?;
                                    stdout.flush()?;
                                    pos -= 1;
                                }
                                editor.set_draw_line(pos as usize);
                                editor.update_status();
                                save_cursor_pos();
                                editor.redraw()?;
                                restore_cursor_pos();
                            }
                        }
                        'l' => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveRight(1))?;
                            stdout.flush()?;
                        }
                        'i' => {
                            editor.set_insert_mode();
                        }
                        'a' => match editor.mode {
                            EditorMode::Insert => {}
                            _ => {}
                        },
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
            Event::Mouse(event) => {}
            Event::Resize(width, height) => {}
        }
    }

    die()
}
