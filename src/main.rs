mod common;
mod commands;
mod rustbot;
mod connection;

use std::process::Command;

use rustyline::{self,error::ReadlineError};

use crate::{rustbot::{bot::RustBot, session::SessionManager}};

fn handle_bot_command(sm: &mut SessionManager, bot: &mut RustBot, input: &str) -> Option<String> {
    match bot.handle_command(sm, input) {
        Ok(Some(response)) => Some(format!("\x1b[1;32m>>\x1b[0m {}", response)),
        Ok(None) => None,
        Err(e) => Some(format!("\x1b[1;31m>>\x1b[0m {}", e))
    }
}

fn startup() -> Result<(),Box<dyn std::error::Error>> {
    log::info!("Entering startup...");

    let output = Command::new("docker-compose")
        .arg("up")
        .arg("-d")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "docker-compose up service(s) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    log::info!("Startup complete!");
    Ok(())
}

fn shutdown() -> Result<(),Box<dyn std::error::Error>> {
    log::info!("Entering shutdown...");

    let output = Command::new("docker-compose")
        .arg("down")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "docker-compose down service(s) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    log::info!("Shutdown complete!");
    Ok(())
}

fn main() {

    // setup env logger
    let _ = env_logger::builder().try_init();

    // call startup checks
    if let Err(e) = startup() {
        println!("\x1b[1;31mStartup failed.\x1b[0m {e}");
        return;
    }

    // init bot and session mgr
    let mut bot = RustBot::new(crate::common::config
        ::get_config::<String>("bot_config.rs", "default_name")
        .unwrap_or("rustbot".into()));
    let mut sm = SessionManager::new();
    
    // print initital message, todays date, and motd
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

    // set up rustyline
    let rl_config = rustyline::config::Builder::new()
        .history_ignore_space(true)
        .history_ignore_dups(true).unwrap()
        .max_history_size(5000).unwrap()
        .build();
    let mut rl = rustyline::DefaultEditor::with_config(rl_config).unwrap();
    let result = rl.load_history(&sm.data_dir.join("rustyline_history.txt"));
    if let Err(e) = result {
        log::warn!("Failed to load rustyline history: {e}");
    }

    // cli loop
    loop {
        match rl.readline("\x1b[1;33m<<\x1b[0m ") {
            Ok(line) => {
                let tokens: Vec<_> = line.split_whitespace().collect();
                let first = match tokens.len() { 0 => "", _ => tokens[0] };
                match first {
                    "exit" | "wexit" | "q" | "wq" => {
                        log::info!("Terminating command called");
                        let response = handle_bot_command(&mut sm, &mut bot, &line);
                        if let Some(r) = response { println!("{}", r); }
                        break;
                    },
                    _ => {
                        let _ = rl.add_history_entry(&line);
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

    let result = rl.save_history(&sm.data_dir.join("rustyline_history.txt"));
    if let Err(e) = result {
        log::warn!("Failed to save rustyline history: {e}");
    }

    // call shutdown checks
    if let Err(e) = shutdown() {
        println!("\x1b[1;31mShutdown failed.\x1b[0m {e}");
        return;
    }
}
