pub mod cue;

use crate::{
    dmx_close, dmx_send_break, dmx_write,
    fixture::patch::{ChannelType, PatchedFixture},
};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use std::thread;

use anyhow::{anyhow, Result};

const DMX_BUFFER_LENGTH: i32 = 513;

pub struct Universe {
    pub id: u8,
    pub fixtures: Vec<Option<PatchedFixture>>, // Index by channel, None = no fixture on that channel
    dmx_buffer: [u8; DMX_BUFFER_LENGTH as usize], // 513 bytes: start code + 512 channels
}

impl Universe {
    pub fn new(id: u8) -> Self {
        Universe {
            id,
            fixtures: vec![],
            dmx_buffer: [0; DMX_BUFFER_LENGTH as usize],
        }
    }

    pub fn from_fixtures(id: u8, fixtures: Vec<PatchedFixture>) -> Self {
        let mut universe = Self::new(id);
        for fixture in fixtures {
            universe.add_fixture(fixture);
        }
        universe
    }

    /// Add a fixture at a specific channel index
    /// Safely resizes the vector if needed, filling gaps with None
    pub fn add_fixture(&mut self, fixture: PatchedFixture) {
        let channel = fixture.channel;

        if channel >= self.fixtures.len() {
            self.fixtures.resize(channel + 1, None);
        }

        self.fixtures[channel] = Some(fixture);
    }

    /// Remove a fixture from a specific channel
    /// returns the removed fixture
    pub fn remove_fixture(&mut self, channel: usize) -> Option<PatchedFixture> {
        if channel < self.fixtures.len() {
            self.fixtures[channel].take()
        } else {
            None
        }
    }

    /// Get a reference to a fixture at a specific channel
    pub fn get_fixture(&self, channel: usize) -> Option<&PatchedFixture> {
        self.fixtures.get(channel)?.as_ref()
    }

    /// Get a mutable reference to a fixture at a specific channel
    pub fn get_fixture_mut(&mut self, channel: usize) -> Option<&mut PatchedFixture> {
        self.fixtures.get_mut(channel)?.as_mut()
    }

    /// Set DMX values for a specific fixture by channel
    pub fn set_fixture_values(
        &mut self,
        channel: usize,
        values: &[(ChannelType, u8)],
    ) -> Result<()> {
        let mut updates: Vec<(usize, u8)> = Vec::new();
        if let Some(fixture) = self.get_fixture(channel) {
            for (function, new_value) in values {
                if let Some(offset) = fixture.profile.channels.get(function) {
                    let buffer_index = fixture.dmx_start as usize + *offset as usize + 1;
                    updates.push((buffer_index, *new_value));
                } else {
                    // maybe include error here but for now i'll just skip that channel
                    eprintln!("Channel: {} has no value: {:?}", fixture.channel, function);
                }
            }
        } else {
            return Err(anyhow!("No fixture found on channel {}", channel));
        }

        for (index, value) in updates {
            self.set_dmx_address(index, value)?;
        }

        Ok(())
    }

    /// quickly set the intensity of a light
    pub fn set_intensity(&mut self, channel: usize, intensity: u8) -> Result<()> {
        return self.set_fixture_values(channel, &[(ChannelType::Intensity, intensity)]);
    }

    pub fn set_rgb(&mut self, channel: usize, r: u8, g: u8, b: u8) -> Result<()> {
        return self.set_fixture_values(
            channel,
            &[
                (ChannelType::Red, r),
                (ChannelType::Green, g),
                (ChannelType::Blue, b),
            ],
        );
    }

    pub fn set_dmx_buffer(&mut self, new_buffer: &[u8; 513]) {
        //todo: check park values and make sure it isn't overwritten
        self.dmx_buffer = *new_buffer;
    }

    pub fn blackout(&mut self) -> Result<()> {
        let channels: Vec<usize> = self.fixtures.iter().flatten().map(|f| f.channel).collect();
        for channel in channels {
            self.set_intensity(channel, 0u8)?;
        }

        Ok(())
    }

    /// Set a single DMX channel value, functions should use this to ensure that values aren't being set incorrectly
    pub fn set_dmx_address(&mut self, dmx_address: usize, value: u8) -> Result<()> {
        if dmx_address == 0 {
            return Err(anyhow!("DMX address 0 is reserved for start code"));
        }
        if dmx_address >= 513 {
            return Err(anyhow!("DMX address must be between 1 and 512"));
        }

        self.dmx_buffer[dmx_address] = value;
        Ok(())
    }

    pub unsafe fn send_buffer(&self, fd: i32) -> Result<()> {
        dmx_send_break(fd);

        if dmx_write(fd, self.dmx_buffer.as_ptr(), DMX_BUFFER_LENGTH) < 0 {
            return Err(anyhow!("Dmx failed to write"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum UniverseCommand {
    // Single channel update
    SetChannel {
        channel: usize,
        value: u8,
    },

    // Multiple channels at once (for cues)
    SetMultiple {
        changes: Vec<(usize, u8)>,
    },

    // Complete cue with metadata
    PlayCue {
        cue_idx: usize,
        cue_data: [u8; 513],
        fade_time_ms: u32,
    },

    // Fixture-level commands
    SetFixture {
        fixture_channel: usize,
        intensity: Option<u8>,
        color: Option<(u8, u8, u8)>, // RGB
    },

    // Show control
    Blackout,

    // Query commands (with response channel)
    GetChannelValue {
        channel: usize,
        response: std::sync::mpsc::Sender<u8>,
    },

    // Get fixture channel information
    GetChannels {
        fixture_channel: usize,
        response: std::sync::mpsc::Sender<Option<Vec<(String, usize, usize)>>>, // (channel_type, dmx_address, offset)
    },

    GetDMXState(std::sync::mpsc::Sender<[u8; 513]>),
}

pub fn dmx_thread(
    mut universe: Universe, // Now OWNED by this thread
    command_rx: Receiver<UniverseCommand>,
    shutdown_rx: Receiver<()>,
    fd: i32,
) {
    println!("DMX thread started");

    let mut last_dmx_send = Instant::now();
    let dmx_interval = Duration::from_millis(25); // 40Hz DMX rate

    loop {
        // Check for shutdown
        if shutdown_rx.try_recv().is_ok() {
            println!("DMX thread shutting down");
            break;
        }

        // Process pending commands
        let mut commands_processed = 0;
        while let Ok(command) = command_rx.try_recv() {
            process_command(&mut universe, command);
            commands_processed += 1;

            // Prevent command processing from blocking DMX too long
            if commands_processed > 100 {
                break; // Process remaining commands next iteration
            }
        }

        // Send DMX at regular intervals
        #[cfg(not(feature = "no-dmx"))]
        if last_dmx_send.elapsed() >= dmx_interval {
            unsafe {
                if let Err(error) = universe.send_buffer(fd) {
                    eprintln!("DMX send error: {}", error);
                    break;
                }
            }
            last_dmx_send = Instant::now();
        }

        // 4. Small sleep to prevent busy-waiting
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    unsafe {
        dmx_close(fd);
    }
    println!("DMX thread stopped");
}

fn process_command(universe: &mut Universe, command: UniverseCommand) {
    match command {
        UniverseCommand::SetChannel { channel, value } => {
            if let Err(e) = universe.set_dmx_address(channel, value) {
                eprintln!("Failed to set channel {}: {}", channel, e);
            }
        }
        UniverseCommand::SetMultiple { changes } => {
            for (channel, value) in changes {
                if let Err(e) = universe.set_dmx_address(channel, value) {
                    eprintln!("Failed to set channel {}: {}", channel, e);
                }
            }
        }
        UniverseCommand::PlayCue {
            cue_idx,
            cue_data,
            fade_time_ms,
        } => {
            println!("Playing cue {} with {} channels", cue_idx, cue_data.len());

            if fade_time_ms == 0 {
                // Instant cue - apply immediately
                universe.set_dmx_buffer(&cue_data);
            } else {
                // TODO: Start fade process (would need fade engine)
                eprintln!("Fade not implemented yet, applying instantly");
                universe.set_dmx_buffer(&cue_data);
            }
        }
        UniverseCommand::SetFixture {
            fixture_channel,
            intensity,
            color,
        } => {
            // Find fixture and set its channels
            if let Some(_fixture) = universe.get_fixture(fixture_channel) {
                let mut updates = Vec::new();

                if let Some(intensity_val) = intensity {
                    updates.push((crate::fixture::patch::ChannelType::Intensity, intensity_val));
                }

                if let Some((r, g, b)) = color {
                    updates.push((crate::fixture::patch::ChannelType::Red, r));
                    updates.push((crate::fixture::patch::ChannelType::Green, g));
                    updates.push((crate::fixture::patch::ChannelType::Blue, b));
                }

                universe.set_fixture_values(fixture_channel, &updates).ok();
            }
        }
        UniverseCommand::Blackout => {
            println!("Blackout command received");
            universe.blackout().ok();
        }
        UniverseCommand::GetChannelValue { channel, response } => {
            let value = universe.dmx_buffer.get(channel).copied().unwrap_or(0);
            response.send(value).ok(); // Send response back
        }
        UniverseCommand::GetChannels {
            fixture_channel,
            response,
        } => {
            let channel_info = if let Some(fixture) = universe.get_fixture(fixture_channel) {
                let mut channels = Vec::new();

                for (channel_type, offset) in &fixture.profile.channels {
                    let dmx_address = fixture.dmx_start as usize + *offset as usize;
                    let type_name = format!("{:?}", channel_type); // Convert enum to string
                    channels.push((type_name, dmx_address, *offset as usize));
                }

                Some(channels)
            } else {
                None // No fixture found at this channel
            };

            response.send(channel_info).ok();
        }
        UniverseCommand::GetDMXState(response) => {
            response.send(universe.dmx_buffer).ok();
        }
    }
}
