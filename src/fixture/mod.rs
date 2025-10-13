pub mod patch;

use crate::{
    dmx_send_break, dmx_write,
    fixture::patch::{ChannelType, PatchedFixture},
};

const DMX_BUFFER_LENGTH: i32 = 513;

pub struct Universe {
    pub id: u8,
    pub fixtures: Vec<Option<PatchedFixture>>, // Index by channel, None = no fixture on that channel
    pub dmx_buffer: [u8; DMX_BUFFER_LENGTH as usize], // 513 bytes: start code + 512 channels
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
    ) -> Result<(), String> {
        let mut updates: Vec<(usize, u8)> = Vec::with_capacity(values.len());

        if let Some(fixture) = self.get_fixture(channel) {
            for (function, new_value) in values {
                if let Some(offset) = fixture.profile.channels.get(function) {
                    let buffer_index = fixture.dmx_start as usize + *offset as usize + 1;
                    updates.push((buffer_index, *new_value));
                } else {
                    eprintln!("Channel: {} has no value: {:?}", fixture.channel, function);
                }
            }
        } else {
            return Err(format!("No fixture found on channel {}", channel));
        }

        for (index, value) in updates {
            if let Err(error) = self.set_dmx_address(index, value) {
                return Err(error);
            }
        }

        Ok(())
    }

    /// quickly set the intensity of a light
    pub fn set_intensity(&mut self, channel: usize, intensity: u8) -> Result<(), String> {
        return self.set_fixture_values(channel, &[(ChannelType::INTENSITY, intensity)]);
    }

    pub fn set_rgb(&mut self, channel: usize, r: u8, g: u8, b: u8) -> Result<(), String> {
        return self.set_fixture_values(
            channel,
            &[
                (ChannelType::RED, r),
                (ChannelType::GREEN, g),
                (ChannelType::BLUE, b),
            ],
        );
    }

    pub fn blackout(&mut self) -> Result<(), String> {
        let channels: Vec<usize> = self.fixtures.iter().flatten().map(|f| f.channel).collect();
        for channel in channels {
            if let Err(error) = self.set_intensity(channel, 0u8) {
                return Err(error);
            }
        }

        Ok(())
    }

    /// Set a single DMX channel value, functions should use this to ensure that values aren't being set incorrectly
    pub fn set_dmx_address(&mut self, dmx_address: usize, value: u8) -> Result<(), String> {
        if dmx_address == 0 {
            return Err("DMX address 0 is reserved for start code".to_string());
        }
        if dmx_address >= 513 {
            return Err("DMX address must be between 1 and 512".to_string());
        }

        self.dmx_buffer[dmx_address] = value;
        Ok(())
    }

    pub unsafe fn send_buffer(&self, fd: i32) -> Result<(), String> {
        dmx_send_break(fd);

        if dmx_write(fd, self.dmx_buffer.as_ptr(), DMX_BUFFER_LENGTH) < 0 {
            return Err("Dmx failed to write".to_string());
        }

        Ok(())
    }
}
