mod bencode;
mod commands;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <command> <args...>", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let result = match command.as_str() {
        "decode" => commands::decode(&args[2]),
        "info" => commands::info(&args[2]),
        _ => {
            println!("unknown command: {}", command);
            return;
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
