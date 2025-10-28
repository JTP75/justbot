mod commands;
mod rustbot;
mod connection;

use dotenvy;
use rustyline::{DefaultEditor,error::ReadlineError};

use crate::rustbot::{bot::{DEFAULT_NAME, RustBot}, session::SessionManager};

fn handle_bot_command(sm: &SessionManager, bot: &mut RustBot, input: &str) -> String {
    match bot.handle_command(sm, input) {
        Ok(Some(response)) => format!("\x1b[1;32m>>\x1b[0m {}", response),
        Ok(None) => "\x1b[1;32m>>\x1b[0m Command executed successfully.".into(),
        Err(e) => format!("\x1b[1;31m>>\x1b[0m {}", e)
    }
}

// todo unify this with other commands please...
fn handle_session_command(sm: &mut SessionManager, bot: &mut RustBot, tokens: Vec<&str>) -> String {
    sm.handle_command(tokens, bot)
}

fn main() {

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            dotenvy::from_path(env_path).ok();
        }
    }

    let mut bot = RustBot::new(DEFAULT_NAME);
    let mut sm = SessionManager::new();
    
    println!("\x1b[1;32m>>\x1b[0m Hi, I'm \x1b[0;33mrustbot\x1b[0m! Type 'help' to see what I can do.");

    match sm.load_motd(&mut bot) {
        Ok(_) => println!("\x1b[1;32m>>\x1b[0m Today is {}.\n\x1b[1;32m>>\x1b[0m {}", 
            bot.get_motd().0.format("%A, %B %-d, %Y"), 
            bot.get_motd().1.unwrap_or("No motd today because justin cant code :(".into())
        ),
        Err(e) => println!("\x1b[1;31m>>\x1b[0m Failed to load motd on startup: {}", e)
    }

    let mut rl = DefaultEditor::new().unwrap();
    loop {
        match rl.readline("\x1b[1;33m<<\x1b[0m ") {
            Ok(line) => {
                let tokens: Vec<_> = line.split_whitespace().collect();
                let first = match tokens.len() { 0 => "", _ => tokens[0] };
                match first {
                    "exit" | "wexit" | "q" | "wq" => {
                        let response = handle_session_command(&mut sm, &mut bot, tokens);
                        println!("{}", response);
                        break;
                    },
                    "save" | "w" | "list" | "load" => {
                        let response = handle_session_command(&mut sm, &mut bot, tokens);
                        println!("{}", response);
                    },
                    _ => {
                        let response = handle_bot_command(&sm, &mut bot, &line);
                        println!("{}", response);
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
