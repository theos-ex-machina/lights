use std::{
    io::{self, Write},
    str::FromStr,
};

use crate::universe::cue::CueEngine;
use anyhow::{anyhow, Context, Result};

/// Helper function to parse arguments with better error handling
fn parse_arg<T: FromStr>(args: &[&str], index: usize, arg_name: &str) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value = args
        .get(index)
        .with_context(|| format!("Missing {} argument", arg_name))?;

    value.parse::<T>().with_context(|| {
        format!(
            "{} must be a valid {}",
            arg_name,
            std::any::type_name::<T>()
        )
    })
}

fn parse_intensity(value: &str) -> Result<u8> {
    if value.contains('f') || value.contains("full") {
        Ok(255)
    } else {
        value
            .parse()
            .with_context(|| "Intensity must be a number or 'f'/'full'".to_string())
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
    GetChannels(usize),
    Go,
    Back,
    RecordCue {
        name: String,
        time_in_ms: u32,
    },
    DeleteCue(String),
    Help,
    Error(anyhow::Error),
}

#[derive(Debug)]
enum ChannelAction {
    Intensity(u8),
    Rgb(u8, u8, u8),
}

fn parse_command(args: &[&str]) -> Command {
    if args.is_empty() {
        return Command::Error(anyhow!("Empty command"));
    }

    match args[0] {
        "c" => {
            let channel = match parse_arg::<usize>(args, 1, "channel") {
                Ok(val) => val,
                Err(e) => return Command::Error(e),
            };

            if args.get(2).map_or(false, |s| s.contains("@")) {
                match args
                    .get(3)
                    .ok_or_else(|| anyhow!("Missing intensity"))
                    .and_then(|s| parse_intensity(s))
                {
                    Ok(intensity) => Command::Channel {
                        channel,
                        action: ChannelAction::Intensity(intensity),
                    },
                    Err(e) => Command::Error(e),
                }
            } else if args.get(2).map_or(false, |s| s.contains("rgb")) {
                match (|| -> Result<(u8, u8, u8)> {
                    let r = parse_arg::<u8>(args, 3, "red")?;
                    let g = parse_arg::<u8>(args, 4, "green")?;
                    let b = parse_arg::<u8>(args, 5, "blue")?;
                    Ok((r, g, b))
                })() {
                    Ok((r, g, b)) => Command::Channel {
                        channel,
                        action: ChannelAction::Rgb(r, g, b),
                    },
                    Err(e) => Command::Error(e),
                }
            } else {
                Command::Error(anyhow::anyhow!(
                    "Use: c <channel> @ <intensity> or c <channel> rgb <r> <g> <b>"
                ))
            }
        }
        "a" => {
            match (
                parse_arg::<usize>(args, 1, "address"),
                args.get(3)
                    .ok_or(anyhow!("Missing value"))
                    .and_then(|s| parse_intensity(s)),
            ) {
                (Ok(address), Ok(value)) => Command::Address { address, value },
                (Err(e), _) | (_, Err(e)) => Command::Error(e),
            }
        }
        "get" => match parse_arg::<usize>(args, 1, "fixture_channel") {
            Ok(channel) => Command::GetChannels(channel),
            Err(e) => Command::Error(e),
        },
        "blackout" => Command::Blackout,
        "rc" => match parse_arg::<String>(args, 1, "cue_name") {
            Ok(name) => match parse_arg::<u32>(args, 2, "time_in") {
                Ok(time_in) => Command::RecordCue {
                    name: name,
                    time_in_ms: time_in,
                },
                Err(e) => Command::Error(e),
            },
            Err(e) => Command::Error(e),
        },
        "dc" => match parse_arg::<String>(args, 1, "cue_name") {
            Ok(name) => Command::DeleteCue(name),
            Err(e) => Command::Error(e),
        },
        "go" => Command::Go,
        "back" => Command::Back,
        "help" => Command::Help,
        _ => Command::Error(anyhow!("Unknown command: {}", args[0])),
    }
}

/// CLI that uses command channels instead of direct universe access
pub fn run_cli(
    command_tx: std::sync::mpsc::Sender<crate::universe::UniverseCommand>,
    show: &mut CueEngine,
) {
    println!("DMX Controller CLI - Command Mode");
    println!("Commands:");
    println!("  c <num> @ <intensity>         - Set fixture intensity");
    println!("  c <num> rgb <r> <g> <b>       - Set fixture RGB color");
    println!("  a <addr> @ <value>            - Set DMX address directly");
    println!("  channels <fixture>            - List channels for fixture");
    println!("  query <channel>               - Get current DMX value");
    println!("  blackout                      - Turn off all fixtures");
    println!("  quit/exit                     - Exit program");
    println!("  help                          - Show this help");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input");
            continue;
        }

        let args: Vec<&str> = input.trim().split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        // Check for quit commands first
        if matches!(args[0], "quit" | "exit" | "q") {
            break;
        }

        let command = parse_command(&args);

        match execute_command(&command, &command_tx, show) {
            Ok(should_quit) => {
                if should_quit {
                    break;
                }
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }

    println!("CLI exiting...");
}

fn execute_command(
    command: &Command,
    command_tx: &std::sync::mpsc::Sender<crate::universe::UniverseCommand>,
    show: &mut CueEngine,
) -> Result<bool> {
    use crate::universe::UniverseCommand;

    match command {
        Command::Channel { channel, action } => {
            match action {
                ChannelAction::Intensity(intensity) => {
                    command_tx
                        .send(UniverseCommand::SetFixture {
                            fixture_channel: *channel,
                            intensity: Some(*intensity),
                            color: None,
                        })
                        .with_context(|| "Failed to send fixture command")?;
                    println!("Set channel {} intensity to {}", channel, intensity);
                }
                ChannelAction::Rgb(r, g, b) => {
                    command_tx
                        .send(UniverseCommand::SetFixture {
                            fixture_channel: *channel,
                            intensity: None,
                            color: Some((*r, *g, *b)),
                        })
                        .with_context(|| "Failed to send fixture command")?;
                    println!("Set channel {} RGB to ({}, {}, {})", channel, r, g, b);
                }
            }
            Ok(false)
        }
        Command::Address { address, value } => {
            command_tx
                .send(UniverseCommand::SetChannel {
                    channel: *address,
                    value: *value,
                })
                .with_context(|| "Failed to send channel command")?;
            println!("Set DMX address {} to {}", address, value);

            Ok(false)
        }
        Command::Blackout => {
            command_tx
                .send(UniverseCommand::Blackout)
                .with_context(|| "Failed to send blackout command")?;
            println!("Blackout activated");

            Ok(false)
        }
        Command::GetChannels(fixture_channel) => {
            let (response_tx, response_rx) = std::sync::mpsc::channel();

            command_tx
                .send(UniverseCommand::GetChannels {
                    fixture_channel: *fixture_channel,
                    response: response_tx,
                })
                .with_context(|| "Failed to send GetChannels command")?;

            use std::time::Duration;
            match response_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Some(channels)) => {
                    println!("Fixture {} channels:", fixture_channel);
                    println!("  Type            DMX Addr  Offset");
                    println!("  --------------- --------- ------");
                    for (channel_type, dmx_address, offset) in channels {
                        println!("  {:15} {:9} {:6}", channel_type, dmx_address, offset);
                    }
                }
                Ok(None) => {
                    println!("No fixture found at channel {}", fixture_channel);
                }
                Err(_) => {
                    println!("Query timeout for fixture {}", fixture_channel);
                }
            }
            Ok(false)
        }
        Command::Go => {
            show.go()?;

            Ok(false)
        }
        Command::Back => {
            show.back()?;

            Ok(false)
        }
        Command::RecordCue { name, time_in_ms } => {
            show.record_cue(name, *time_in_ms as u64)?;

            Ok(false)
        }
        Command::DeleteCue(name) => {
            show.delete_cue(&name)?;

            Ok(false)
        }
        Command::Help => {
            println!("Available commands:");
            println!(
                "  c <num> @ <intensity>         - Set fixture intensity (0-255 or 'f' for full)"
            );
            println!("  c <num> rgb <r> <g> <b>       - Set fixture RGB color (0-255 each)");
            println!("  a <addr> @ <value>            - Set DMX address directly (1-512)");
            println!("  channels <fixture>            - List channels for fixture");
            println!("  blackout                      - Turn off all fixtures");
            println!("  quit/exit                     - Exit program");
            println!("  help                          - Show this help");
            println!();
            println!("Examples:");
            println!("  c 1 @ 255         - Set channel 1 to full intensity");
            println!("  c 1 @ f           - Set channel 1 to full intensity");
            println!("  c 1 rgb 255 0 0   - Set channel 1 to red");
            println!("  a 10 @ 128        - Set DMX address 10 to 128");
            println!("  get 1         - Show channels for fixture 1");
            Ok(false)
        }
        Command::Error(msg) => {
            println!("Error: {}", msg);
            println!("Type 'help' for available commands");
            Ok(false)
        }
    }
}
