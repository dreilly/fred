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
use editor::{Editor, EditorMode, KeyState, Line};
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
        key_state: KeyState::Inactive,
    };
    Ok(editor)
}

fn main() -> Result<()> {
    panic::set_hook(Box::new(|i| {
        die().unwrap();
        println!("Unrecoverable error");
        //let pancic_info = i.to_string();
        println!("{:?}", i);
    }));
    let mut editor = init_editor()?;
    let args: Vec<String> = std::env::args().collect();
    let mut file_found = false;
    let mut file_name = String::new();
    if args.len() > 1 {
        file_name = args[1].to_string();
        file_found = Path::new(&file_name[..]).exists();
    }

    if file_found {
        if let Ok(lines) = read_lines(file_name) {
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
    }

    editor.draw_editor(false)?;
    editor.handle_input()?;

    die()
}
