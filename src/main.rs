mod common;

use common::message;
use std::io::Write;
use dotenvy;

use anthropic;

fn main() {
    dotenvy::from_filename_override(".env").ok();
    
    message::print_initial_message();

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
                    let response = message::get_response_for_input(&input, &mut messages);
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
