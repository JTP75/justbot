mod common;

use common::message;

fn main() {
    message::print_initial_message();

    while let Some(Ok(input)) = std::io::stdin().lines().next() {
        if input.trim() == "exit" {
            println!("Exiting Rustbot. Goodbye!");
            break;
        } else {
            let response = message::get_response_for_input(&input);
            println!("{}", response);
        }
    }
}
