use std::{
    io::{self, Write},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::fixture::Universe;

/// Helper function to parse arguments with better error handling
fn parse_arg<T: FromStr>(args: &[&str], index: usize, arg_name: &str) -> Result<T, String> {
    args.get(index)
        .ok_or_else(|| format!("Missing {} argument", arg_name))?
        .parse()
        .map_err(|_| format!("{} must be a valid number", arg_name))
}

/// Parse intensity with support for 'f'/'full'
fn parse_intensity(value: &str) -> Result<u8, String> {
    if value.contains('f') || value.contains("full") {
        Ok(255)
    } else {
        value
            .parse()
            .map_err(|_| "Intensity must be a number or 'f'/'full'".to_string())
    }
}

#[derive(Debug)]
enum Command {
    Channel {
        channel: usize,
        action: ChannelAction,
    },
    Address {
        address: usize,
        value: u8,
    },
    Blackout,
    Help,
    Unknown(String),
}

#[derive(Debug)]
enum ChannelAction {
    Intensity(u8),
    Rgb(u8, u8, u8),
}

fn parse_command(args: &[&str]) -> Command {
    if args.is_empty() {
        return Command::Unknown("Empty command".to_string());
    }

    match args[0] {
        cmd if cmd.starts_with("c") => {
            let channel = match parse_arg::<usize>(args, 1, "channel") {
                Ok(val) => val,
                Err(err) => return Command::Unknown(err),
            };

            if args.get(2).map_or(false, |s| s.contains("@")) {
                match args
                    .get(3)
                    .ok_or("Missing intensity".to_string())
                    .and_then(|s| parse_intensity(s))
                {
                    Ok(intensity) => Command::Channel {
                        channel,
                        action: ChannelAction::Intensity(intensity),
                    },
                    Err(err) => Command::Unknown(err),
                }
            } else if args.get(2).map_or(false, |s| s.contains("rgb")) {
                match (|| -> Result<(u8, u8, u8), String> {
                    let r = parse_arg::<u8>(args, 3, "red")?;
                    let g = parse_arg::<u8>(args, 4, "green")?;
                    let b = parse_arg::<u8>(args, 5, "blue")?;
                    Ok((r, g, b))
                })() {
                    Ok((r, g, b)) => Command::Channel {
                        channel,
                        action: ChannelAction::Rgb(r, g, b),
                    },
                    Err(err) => Command::Unknown(err),
                }
            } else {
                Command::Unknown(
                    "Use: c <channel> @ <intensity> or c <channel> rgb <r> <g> <b>".to_string(),
                )
            }
        }
        cmd if cmd.contains("a") => {
            match (
                parse_arg::<usize>(args, 1, "address"),
                args.get(3)
                    .ok_or("Missing value".to_string())
                    .and_then(|s| parse_intensity(s)),
            ) {
                (Ok(address), Ok(value)) => Command::Address { address, value },
                (Err(err), _) | (_, Err(err)) => Command::Unknown(err),
            }
        }
        "blackout" => Command::Blackout,
        "help" => Command::Help,
        _ => Command::Unknown(format!("Unknown command: {}", args[0])),
    }
}

pub fn run_cli(universe: &mut Universe) {
    println!("Type a command. type 'help; for help\n");
    loop {
        print!("> ");
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let args: Vec<&str> = input.trim().split_whitespace().collect();

        match parse_command(&args) {
            Command::Channel { channel, action } => {
                let result = match action {
                    ChannelAction::Intensity(intensity) => {
                        universe.set_intensity(channel, intensity)
                    }
                    ChannelAction::Rgb(r, g, b) => universe.set_rgb(channel, r, g, b),
                };

                if let Err(error) = result {
                    eprintln!("{}", error);
                }
            }
            Command::Address { address, value } => {
                if let Err(error) = universe.set_dmx_address(address, value) {
                    eprintln!("{}", error);
                }
            }
            Command::Blackout => {
                if let Err(error) = universe.blackout() {
                    eprintln!("Blackout error: {}", error);
                } else {
                    println!("Blackout applied");
                }
            }
            Command::Help => {
                println!("Available commands:");
                println!("  c <channel> @ <intensity>        - Set channel intensity (0-255 or 'f' for full)");
                println!("  c <channel> rgb <r> <g> <b>      - Set channel RGB values");
                println!("  a <address> @ <value>            - Set DMX address directly");
                println!("  blackout                         - Turn off all lights");
                println!("  help                             - Show this help");
            }
            Command::Unknown(err) => {
                println!("Error: {}", err);
                println!("Type 'help' for available commands.");
            }
        }
    }
}

/// CLI function that works with shared Universe for multi-threaded operation
pub fn run_cli_threaded(universe: Arc<Mutex<Universe>>) {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input == "quit" || input == "exit" {
            break;
        }

        // Parse and execute commands with brief universe locks
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        // Lock universe briefly for each command
        match universe.lock() {
            Ok(mut universe_guard) => {
                execute_command(&mut universe_guard, &args);
            }
            Err(_) => {
                eprintln!("Failed to access universe (mutex poisoned)");
                break;
            }
        }
        // Mutex is automatically released here
    }
}

/// Execute a parsed command on the universe
fn execute_command(universe: &mut Universe, args: &[&str]) {
    let command = parse_command(args);

    match command {
        Command::Channel { channel, action } => {
            let result = match action {
                ChannelAction::Intensity(intensity) => {
                    println!("Set channel {} to intensity {}", channel, intensity);
                    universe.set_intensity(channel, intensity)
                }
                ChannelAction::Rgb(r, g, b) => {
                    println!("Set channel {} to RGB({}, {}, {})", channel, r, g, b);
                    universe.set_rgb(channel, r, g, b)
                }
            };

            if let Err(error) = result {
                eprintln!("Error: {}", error);
            }
        }
        Command::Address { address, value } => {
            if let Err(error) = universe.set_dmx_address(address, value) {
                eprintln!("Error: {}", error);
            } else {
                println!("Set DMX address {} to value {}", address, value);
            }
        }
        Command::Blackout => {
            if let Err(error) = universe.blackout() {
                eprintln!("Blackout error: {}", error);
            } else {
                println!("Blackout applied");
            }
        }
        Command::Help => {
            print_help();
        }
        Command::Unknown(err) => {
            println!("Error: {}", err);
            println!("Type 'help' for available commands.");
        }
    }
}

/// Print help information
fn print_help() {
    println!("Commands:");
    println!("  c <channel> @ <intensity>      - Set fixture intensity (0-255 or 'f' for full)");
    println!("  c <channel> rgb <r> <g> <b>    - Set fixture RGB values");
    println!("  a <address> @ <value>          - Set DMX address directly");
    println!("  blackout                       - Turn off all lights");
    println!("  help                           - Show this help");
    println!("  quit/exit                      - Exit program");
}
