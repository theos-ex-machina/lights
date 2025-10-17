use crate::universe::UniverseCommand;
use std::{sync::mpsc::Sender, time::Duration};
use anyhow::{anyhow, Context, Result};

pub struct CueEngine {
    command_tx: Sender<UniverseCommand>,
    current_cue: Option<usize>,
    cues: Vec<Cue>,
}

impl CueEngine {
    pub fn new(command_tx: Sender<UniverseCommand>) -> Self {
        Self {
            command_tx,
            current_cue: None,
            cues: Vec::new(),
        }
    }

    pub fn record_cue(&mut self, name: &str, time_in: u64) -> Result<()> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();

        self.command_tx
            .send(UniverseCommand::GetDMXState(response_tx))
            .with_context(|| "Failed to get DMX state")?;

        let state = response_rx
            .recv_timeout(Duration::from_millis(100))
            .with_context(|| "Timeout reciving DMX state")?;

        if let Some(cue_idx) = self.cues.iter().position(|cue| cue.name == name) {
            self.cues[cue_idx].time_in = Duration::from_millis(time_in);
            self.cues[cue_idx].channels = state;
        } else {
            self.cues.push(Cue {
                name: name.to_string(),
                time_in: Duration::from_millis(time_in),
                channels: state,
            });
        }

        Ok(())
    }

    pub fn delete_cue(&mut self, cue_id: &str) -> Result<()> {
        let cue_index = match self.cues.iter().position(|cue| cue.name == cue_id) {
            Some(idx) => idx,
            None => {
                return Err(anyhow!("There is no cue \"{}\"", cue_id));
            }
        };

        self.delete_cue_idx(cue_index)
    }

    pub fn delete_cue_idx(&mut self, cue_index: usize) -> Result<()> {
        if cue_index > self.cues.len() {
            return Err(anyhow!("Cue {} out of bounds", cue_index));
        }
        self.cues.remove(cue_index);

        Ok(())
    }

    pub fn go(&mut self) -> Result<()> {
        let next_cue_index = self.current_cue.map_or(0, |c| c + 1);

        if let Some(cue) = self.cues.get(next_cue_index) {
            self.command_tx
                .send(UniverseCommand::PlayCue {
                    cue_idx: next_cue_index,
                    cue_data: cue.channels.clone(),
                    fade_time_ms: cue.time_in.as_millis() as u32,
                })
                .with_context(|| "Failed to send cue command")?;

            self.current_cue = Some(next_cue_index);
            println!("GO: Moving to cue {}", next_cue_index + 1);
            Ok(())
        } else {
            Err(anyhow!("No cue {} available", next_cue_index + 1))
        }
    }

    pub fn back(&mut self) -> Result<()> {
        if let Some(current) = self.current_cue {
            if current > 0 {
                let prev_cue_index = current - 1;

                if let Some(cue) = self.cues.get(prev_cue_index) {
                    self.command_tx
                        .send(UniverseCommand::PlayCue {
                            cue_idx: prev_cue_index,
                            cue_data: cue.channels.clone(),
                            fade_time_ms: cue.time_in.as_millis() as u32,
                        })
                        .with_context(|| "Failed to send cue command")?;

                    self.current_cue = Some(prev_cue_index);
                    println!("BACK: Moving to cue {}", prev_cue_index + 1);
                    Ok(())
                } else {
                    Err(anyhow!("Previous cue not found"))
                }
            } else {
                Err(anyhow!("Already at first cue"))
            }
        } else {
            Err(anyhow!("No current cue"))
        }
    }

    pub fn go_to_cue(&mut self, cue_id: &str) -> Result<()> {
        let cue_index = match self.cues.iter().position(|cue| cue.name == cue_id) {
            Some(idx) => idx,
            None => {
                return Err(anyhow!("There is no cue \"{}\"", cue_id));
            }
        };

        self.go_to_cue_idx(cue_index)
    }

    pub fn go_to_cue_idx(&mut self, cue_number: usize) -> Result<()> {
        let cue_index = cue_number.saturating_sub(1); // Convert 1-based to 0-based

        if let Some(cue) = self.cues.get(cue_index) {
            self.command_tx
                .send(UniverseCommand::PlayCue {
                    cue_idx: cue_index,
                    cue_data: cue.channels.clone(),
                    fade_time_ms: cue.time_in.as_millis() as u32,
                })
                .with_context(|| "Failed to send cue command")?;

            self.current_cue = Some(cue_index);
            println!("GOTO: Jumped to cue {}", cue_number);
            Ok(())
        } else {
            Err(anyhow!("Cue {} not found", cue_number))
        }
    }
}

pub struct Cue {
    name: String,
    time_in: Duration,
    channels: [u8; 513],
}
