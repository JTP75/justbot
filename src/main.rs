mod common;

use std::env;

use anthropic::types::Message;
use dotenvy;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use common::message_processing;

fn main() {

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            dotenvy::from_path(env_path).ok();
        }
    }

    message_processing::print_initial_message();

    let mut messages = Vec::<Message>::new();

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("error: {e}");
            return;
        }
    };

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if line.trim() == "exit" {
                    println!("Exiting rustbot. Goodbye!");
                    break;
                } else if line.trim() == "wexit" {
                    println!("Exiting rustbot and saving. Goodbye!");
                    break;
                } else {
                    let response = message_processing::route_command(&line, &mut messages);
                    println!("{}", response);
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
