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
                eprintln!(
                    "Usage: {} handshake <torrent_file> <peer_ip:peer_port>",
                    args[0]
                );
                std::process::exit(1);
            }
            commands::handshake(&args[2], &args[3])
        }
        "download_piece" => {
            if args.len() < 5 {
                eprintln!(
                    "Usage: {} download_piece -o <output> <torrent_file> <piece_index>",
                    args[0]
                );
                std::process::exit(1);
            }
            // Parse: download_piece -o <output> <torrent> <piece_index>
            if args[2] != "-o" {
                eprintln!("Expected -o flag");
                std::process::exit(1);
            }
            let output_path = &args[3];
            let torrent_path = &args[4];
            let piece_index: u32 = args[5].parse().unwrap_or_else(|_| {
                eprintln!("Invalid piece index: {}", args[5]);
                std::process::exit(1);
            });
            commands::download_piece(torrent_path, output_path, piece_index)
        }
        "download" => {
            if args.len() < 4 {
                eprintln!("Usage: {} download -o <output> <torrent_file>", args[0]);
                std::process::exit(1);
            }
            // Parse: download -o <output> <torrent>
            if args[2] != "-o" {
                eprintln!("Expected -o flag");
                std::process::exit(1);
            }
            let output_path = &args[3];
            let torrent_path = &args[4];
            commands::download(torrent_path, output_path)
        }
        "magnet_parse" => {
            if args.len() < 3 {
                eprintln!("Usage: {} magnet_parse <magnet-link>", args[0]);
                std::process::exit(1);
            }
            commands::magnet_parse(&args[2])
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
