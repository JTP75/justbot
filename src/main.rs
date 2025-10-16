mod common;

use common::message_processing;
use std::io::Write;
use std::env;

use anthropic;
use dotenvy;

fn main() {

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            dotenvy::from_path(env_path).ok();
        }
    }

    message_processing::print_initial_message();

    let mut messages = Vec::<anthropic::types::Message>::new();

    while {
        print!("> ");
        std::io::stdout().flush().unwrap();
        true
    } {
        match std::io::stdin().lines().next() {
            Some(Ok(input)) => {
                if input.trim() == "exit" {
                    println!("Exiting rustbot. Goodbye!");
                    break;
                } else {
                    let response = message_processing::get_response_for_input(&input, &mut messages);
                    println!("{}", response);
                }
            }
            Some(Err(err)) => {
                eprintln!("error: {}", err);
                break;
            }
            None => break
        }
    }
}
