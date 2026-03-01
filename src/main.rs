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
        "peers" => commands::peers(&args[2]),
        "handshake" => {
            if args.len() < 4 {
                eprintln!("Usage: {} handshake <torrent_file> <peer_ip:peer_port>", args[0]);
                std::process::exit(1);
            }
            commands::handshake(&args[2], &args[3])
        }
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
