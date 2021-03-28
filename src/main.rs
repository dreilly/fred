use crossterm::{
    terminal::{self, ClearType},
    QueueableCommand, Result,
};
use std::io::{stdout, Write};
use std::panic;
use std::path::Path;

mod editor;
use editor::Editor;
mod fredFile;
mod term;

fn die() -> Result<()> {
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<()> {
    panic::set_hook(Box::new(|i| {
        die().unwrap();
        println!("Unrecoverable error");
        // prints panic info
        // TODO - should only happen for debugging
        println!("{:?}", i);
    }));
    term::init_term()?;
    let mut editor = Editor::new();
    let args: Vec<String> = std::env::args().collect();
    let mut file_found = false;
    let mut file_name = String::new();
    if args.len() > 1 {
        file_name = args[1].to_string();
        file_found = Path::new(&file_name[..]).exists();
    }

    if file_found {
        editor.read_from_file(file_name);
    }

    editor.draw_editor(false)?;
    editor.handle_input()?;

    die()
}
