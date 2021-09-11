use clap::{App, Arg};
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::{cursor, style, terminal, QueueableCommand};

use std::fs;
use std::io::{stdout, Write};
use std::time::Duration;

mod cc_controller;
use cc_controller::Controller;

fn retrieve_instructions(path: String) -> String {
    match fs::read_to_string(&path) {
        Ok(file_content) => file_content,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => panic!("The path to the specified instruction file was not correct. The file was not found."),
            _ => panic!("Unexpected error in reading the file.")
        }
    }
}

fn main() -> crossterm::Result<()> {
    let matches = App::new("Conditional Copy Utility")
        .version("1.0")
        .about("Copy files to a destination folder or update them, if necessary.")
        .arg(Arg::with_name("instructions").default_value("cc_instructions.cci"))
        .arg(
            Arg::with_name("interactive_flag")
                .short("i")
                .long("interactive")
                .help("The program runs in interactive mode"),
        )
        .get_matches();

    let instructions = matches
        .value_of("instructions")
        .map(String::from)
        .map(retrieve_instructions)
        .expect("Instructions could not be read properly.");

    let controller = Controller::new(instructions);
    let targets = controller.target_list();

    let interactivity = matches.is_present("interactive_flag");

    if !interactivity {
        controller.copy_all_targets()
    } else {
        let (_, height) = terminal::size()?;
        let shown_targets = targets.iter().take(height as usize).collect::<Vec<_>>();

        let mut c_y = 0u16;
        let mut output = stdout();
        output
            .queue(terminal::EnterAlternateScreen {})?
            .queue(terminal::Clear {
                0: terminal::ClearType::All,
            })?;
        for target in shown_targets.iter() {
            output.queue(style::Print(format!("[ ] {:?}\n", target)))?;
        }
        output.queue(cursor::MoveTo(1, 0))?.flush()?;

        'event_loop: loop {
            if poll(Duration::from_millis(200))? {
                match read()? {
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            println!("Exiting...!");
                            break 'event_loop;
                        }
                        KeyCode::Up if c_y > 0 => {
                            output.queue(cursor::MoveUp(1))?;
                            c_y -= 1;
                        }
                        KeyCode::Down if c_y < shown_targets.len() as u16 - 1 => {
                            output.queue(cursor::MoveDown(1))?;
                            c_y += 1;
                        }
                        _ => (),
                    },
                    Event::Resize(_, /*new_height*/ _) => {
                        //  height = new_height;
                        //  TODO: add logic for update on resize
                    }
                    _ => (),
                }

                output.flush()?;
            }
        }

        output.queue(terminal::LeaveAlternateScreen {})?;
        return Ok(());
    }
}
