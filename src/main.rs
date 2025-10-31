mod common;
mod commands;
mod rustbot;
mod connection;

use dotenvy;
use rustyline::{DefaultEditor,error::ReadlineError};

use crate::{rustbot::{bot::RustBot, session::SessionManager}};

fn handle_bot_command(sm: &mut SessionManager, bot: &mut RustBot, input: &str) -> Option<String> {
    match bot.handle_command(sm, input) {
        Ok(Some(response)) => Some(format!("\x1b[1;32m>>\x1b[0m {}", response)),
        Ok(None) => None,
        Err(e) => Some(format!("\x1b[1;31m>>\x1b[0m {}", e))
    }
}

fn main() {

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            dotenvy::from_path(env_path).ok();
        }
    }

    let name: String = crate::common::config
        ::get_config("bot_config.rs", "default_name").unwrap_or("rustbot".into());
    let mut bot = RustBot::new(name);
    let mut sm = SessionManager::new();
    
    println!("\x1b[1;32m>>\x1b[0m Hi, I'm \x1b[0;33mrustbot\x1b[0m! Type 'help' to see what I can do.");

    match sm.load_motd(&mut bot) {
        Ok(_) => {
            handle_bot_command(&mut sm, &mut bot, "motd");
            println!("\x1b[1;32m>>\x1b[0m Today is {}.\n\x1b[1;32m>>\x1b[0m {}", 
                bot.get_motd().0.format("%A, %B %-d, %Y"), 
                bot.get_motd().1.unwrap_or("No motd today because justin cant code :(".into())
            )
        },
        Err(e) => println!("\x1b[1;31m>>\x1b[0m Failed to load motd on startup: {}", e)
    }

    let mut rl = DefaultEditor::new().unwrap();
    loop {
        match rl.readline("\x1b[1;33m<<\x1b[0m ") {
            Ok(line) => {
                let tokens: Vec<_> = line.split_whitespace().collect();
                let first = match tokens.len() { 0 => "", _ => tokens[0] };
                match first {
                    "exit" | "wexit" => {
                        let response = handle_bot_command(&mut sm, &mut bot, &line);
                        if let Some(r) = response { println!("{}", r); }
                        break;
                    },
                    _ => {
                        let response = handle_bot_command(&mut sm, &mut bot, &line);
                        if let Some(r) = response { println!("{}", r); }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Readline failed: {:?}", err);
                break;
            }
        }
    }
}
