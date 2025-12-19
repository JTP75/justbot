use puetce::{common::config_const::{json::BOT_CONFIG, keys::DEFAULT_NAME}, *};

use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread};

use rustyline::{self,error::ReadlineError};

use crate::{app::{puetce::PuetceApp, session::SessionManager}};

fn main() {

    // setup env logger
    let _ = env_logger::builder().try_init();

    log::info!("Starting up...");

    // init bot and session mgr
    let mut bot = PuetceApp::new(crate::common::config
        ::get_config::<String>(BOT_CONFIG, DEFAULT_NAME)
        .unwrap_or("rustbot".into()));
    let mut sm = SessionManager::new();
    
    if !bot.http_client.is_healthy() {
        log::error!("Backend server health check failed (probably not running)");
        drop(bot);
        return;
    }

    log::info!("Connected to backend at {}", bot.http_client.addr());
    log::info!("Session initialized");

    print_big_banner_puetce();
    
    // print initital message, todays date, and motd
    println!("\x1b[1;32m>>\x1b[0m Hi, I'm \x1b[0;33mPuetce\x1b[0m! Type 'help' to see what I can do.");
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
    let mut running = true;
    while running {
        match rl.readline("\x1b[1;33m<<\x1b[0m ") {
            Ok(line) => {
                let ssf = Arc::new(AtomicBool::new(false));
                let ssf_copy = ssf.clone();
                let message = "Thinking...";
                let spinner = thread::spawn(move || spinner_thread(message, ssf_copy));

                let tokens: Vec<_> = line.split_whitespace().collect();
                let first = match tokens.len() { 0 => "", _ => tokens[0] };
                match first {
                    "exit" | "wexit" | "q" | "wq" => {
                        let response = handle_bot_command(&mut sm, &mut bot, &line);

                        ssf.store(true, Ordering::Relaxed);
                        let _ = spinner.join();

                        if let Some(r) = response { println!("{}", r); }
                        running = false
                    },
                    _ => {
                        let _ = rl.add_history_entry(&line);
                        let response = handle_bot_command(&mut sm, &mut bot, &line);

                        ssf.store(true, Ordering::Relaxed);
                        let _ = spinner.join();

                        if let Some(r) = response { println!("{}", r); }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                log::info!("Received SIGINT");
                let response = handle_bot_command(&mut sm, &mut bot, "exit");
                if let Some(r) = response { println!("{}", r); }
                running = false
            },
            Err(ReadlineError::Eof) => {
                running = false
            },
            Err(err) => {
                log::error!("Readline failed: {:?}", err);
                running = false
            }
        }
    }

    // save rustyline history
    let result = rl.save_history(&sm.data_dir.join("rustyline_history.txt"));
    if let Err(e) = result {
        log::warn!("Failed to save rustyline history: {e}");
    }
    drop(rl);

    // drop bot
    drop(bot);

    return;
}

/// callback for handling bot commands
fn handle_bot_command(sm: &mut SessionManager, bot: &mut PuetceApp, input: &str) -> Option<String> {
    match bot.handle_command(sm, input) {
        Ok(Some(response)) => Some(format!("\x1b[1;32m>>\x1b[0m {}", response)),
        Ok(None) => None,
        Err(e) => Some(format!("\x1b[1;31m>>\x1b[0m {}", e))
    }
}

/// Thread routine for the spinner
pub fn spinner_thread(message: &str, stop_flag: Arc<AtomicBool>) {
    let spinner = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut i = 0;
    while !stop_flag.load(Ordering::Relaxed) {
        print!("\r{} {}", spinner[i%spinner.len()], message);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(80));
        i += 1;
    }
    print!("\r");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

/// Prints the PUETCE banner
/// 
/// - it is pronounced PWAYCHAY
fn print_big_banner_puetce() {
    // developers note: this doesnt mean anything, github copilot just hallucinated it and i thought it looked cool
    println!(r#"
██████╗ ██╗   ██╗███████╗████████╗ ██████╗███████╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔════╝██╔════╝
██████╔╝██║   ██║█████╗     ██║   ██║     █████╗
██╔═══╝ ██║   ██║██╔══╝     ██║   ██║     ██╔══╝
██║     ╚██████╔╝███████╗   ██║   ╚██████╗███████╗
╚═╝      ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝╚══════╝
"#);
}