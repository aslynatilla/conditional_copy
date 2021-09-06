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
        for t in targets {
            let maybe_file = fs::File::open(&t);
            match maybe_file {
                Ok(_) => {}
                //  This should not be necessary, and it should have been already validated
                _ => todo!("Handle non existing file in target somewhere!"),
            }
            let mut destination = controller.destination();
            destination.push(t.file_name().unwrap());

            match fs::File::open(destination.as_path()) {
                Err(e) => match e.kind() {
                    std::io::ErrorKind::PermissionDenied => eprintln!("Permission denied"),
                    std::io::ErrorKind::NotFound => {
                        fs::File::create(&destination).unwrap();
                        std::fs::copy(&t, destination).unwrap();
                    }
                    _ => {
                        eprintln!("Unhandled error!")
                    }
                },
                _ => {
                    eprintln!("File {:?} already exists.", destination.as_path());
                    todo!("Add logic to update the file conditionally.");
                }
            }
        }
        return Ok(());
    } else {
        let (mut width, mut height) = terminal::size()?;
        let mut shown_targets = targets.iter().take(height as usize).collect::<Vec<_>>();

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
                    Event::Resize(new_width, new_height) => {
                        width = new_width;
                        height = new_height;
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
