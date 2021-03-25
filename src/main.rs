use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use std::fs::File;
use std::io::{self, stdout, BufRead, Write};
use std::panic;
use std::path::Path;

mod editor;
use editor::{Editor, EditorMode, Line};
mod term;

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
        draw_region: (0, term::get_term_size().1),
        draw_line: 1,
    };
    Ok(editor)
}

fn main() -> Result<()> {
    panic::set_hook(Box::new(|_| {
        die().unwrap();
        println!("Unrecoverable error");
    }));
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
                                || editor.draw_line < term::get_term_size().1 - 1
                            {
                                let mut pos = cursor::position().unwrap().1 + 1;
                                let term_size = term::get_term_size().1 as u16;
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
                                term::save_cursor_pos();
                                editor.redraw()?;
                                term::restore_cursor_pos();
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
                                term::save_cursor_pos();
                                editor.redraw()?;
                                term::restore_cursor_pos();
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
                        'v' => {
                            editor.set_visual_mode();
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
            Event::Mouse(_event) => {}
            Event::Resize(_width, _height) => {}
        }
    }

    die()
}
