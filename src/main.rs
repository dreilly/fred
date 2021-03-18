use core::panic;
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, ClearType, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand, Result,
};
use std::fs::File;
use std::io::{self, stdout, BufRead, Write};
use std::path::Path;

enum Editor_Mode {
    Normal,
    Insert,
    Visual,
}
struct Editor {
    lines: Vec<Line>,
    mode: Editor_Mode,
}

struct Line {
    line_chars: Vec<char>,
}

impl Editor {
    fn draw_editor(&self) -> Result<()> {
        let mut stdout = stdout();
        for (pos, l) in self.lines.iter().enumerate() {
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
        Ok(())
    }
}

fn init_editor() -> Result<(Editor)> {
    let mut stdout = stdout();
    stdout.queue(terminal::EnterAlternateScreen)?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.flush()?;
    terminal::enable_raw_mode()?;
    let editor = Editor {
        lines: Vec::new(),
        mode: Editor_Mode::Normal,
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
        Err(e) => panic!("{}", e), //println!("{:?}", e),
        Ok(f) => Ok(io::BufReader::new(f).lines()),
    }
}

fn main() -> Result<()> {
    // let args: Vec<String> = std::env::args().collect();
    // println!("{:?}", args);
    // if args.len() > 1 && args[1].len() > 0 {
    //     //arg[1] should be a file name,
    //     //read file stuff from here i guess
    //     println!("{}", args[1]);
    // }

    let mut editor = init_editor()?;

    if let Ok(lines) = read_lines("src/main.rs") {
        let mut lines_buff: Vec<Line> = Vec::new();
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

    editor.draw_editor()?;

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
            Event::Key(KeyEvent { code, modifiers }) => {
                match code {
                    KeyCode::Char(c) => match c {
                        'h' => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveLeft(1))?;
                            stdout.flush()?;
                        }
                        'j' => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveDown(1))?;
                            stdout.flush()?;
                        }
                        'k' => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveUp(1))?;
                            stdout.flush()?;
                        }
                        'l' => {
                            let mut stdout = stdout();
                            stdout.queue(cursor::MoveRight(1))?;
                            stdout.flush()?;
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
            Event::Mouse(event) => {
                println!("{:?}", event)
            }
            Event::Resize(width, height) => {
                println!("width: {} and height: {}", width, height)
            }
        }
    }

    die()
}
