use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::fixture::{
    patch::{ChannelType, PatchedFixture},
    Universe,
};

pub type UniverseState = Arc<Mutex<Universe>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct FixtureInfo {
    pub channel: usize,
    pub label: String,
    pub dmx_start: u16,
    pub profile_name: String,
    pub channels: HashMap<String, u8>,
}

impl From<&PatchedFixture> for FixtureInfo {
    fn from(fixture: &PatchedFixture) -> Self {
        let channels = fixture
            .profile
            .channels
            .iter()
            .map(|(channel_type, offset)| {
                let key = match channel_type {
                    ChannelType::RED => "RED".to_string(),
                    ChannelType::GREEN => "GREEN".to_string(),
                    ChannelType::BLUE => "BLUE".to_string(),
                    ChannelType::INTENSITY => "INTENSITY".to_string(),
                    ChannelType::PAN => "PAN".to_string(),
                    ChannelType::TILT => "TILT".to_string(),
                    ChannelType::Custom(name) => name.clone(),
                };
                (key, *offset)
            })
            .collect();

        FixtureInfo {
            channel: fixture.channel,
            label: fixture.label.clone(),
            dmx_start: fixture.dmx_start,
            profile_name: fixture.profile.name.clone(),
            channels,
        }
    }
}

#[tauri::command]
pub async fn get_fixtures(universe: State<'_, UniverseState>) -> Result<Vec<FixtureInfo>, String> {

    let universe_guard = universe.lock().map_err(|e| {
        let err = format!("Failed to lock universe: {:?}", e);
        eprintln!("{}", err);
        err
    })?;

    let fixtures: Vec<FixtureInfo> = universe_guard
        .fixtures
        .iter()
        .filter_map(|fixture_opt| {
            if let Some(f) = fixture_opt {
                println!("Found fixture: channel={}, label={}", f.channel, f.label);
                Some(FixtureInfo::from(f))
            } else {
                None
            }
        })
        .collect();

    Ok(fixtures)
}

#[tauri::command]
pub async fn set_channel_value(
    universe: State<'_, UniverseState>,
    channel: usize,
    channel_type: String,
    value: u8,
) -> Result<(), String> {
    let mut universe_guard = universe
        .lock()
        .map_err(|_| "Failed to lock universe".to_string())?;

    let channel_type_enum = match channel_type.as_str() {
        "RED" => ChannelType::RED,
        "GREEN" => ChannelType::GREEN,
        "BLUE" => ChannelType::BLUE,
        "INTENSITY" => ChannelType::INTENSITY,
        "PAN" => ChannelType::PAN,
        "TILT" => ChannelType::TILT,
        name => ChannelType::Custom(name.to_string()),
    };

    universe_guard.set_fixture_values(channel, &[(channel_type_enum, value)])?;
    Ok(())
}

#[tauri::command]
pub async fn blackout(universe: State<'_, UniverseState>) -> Result<(), String> {
    let mut universe_guard = universe
        .lock()
        .map_err(|_| "Failed to lock universe".to_string())?;

    universe_guard.blackout()
}

#[tauri::command]
pub async fn set_intensity(
    universe: State<'_, UniverseState>,
    channel: usize,
    intensity: u8,
) -> Result<(), String> {
    let mut universe_guard = universe
        .lock()
        .map_err(|_| "Failed to lock universe".to_string())?;

    universe_guard.set_intensity(channel, intensity)
}

#[tauri::command]
pub async fn set_rgb(
    universe: State<'_, UniverseState>,
    channel: usize,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), String> {
    let mut universe_guard = universe
        .lock()
        .map_err(|_| "Failed to lock universe".to_string())?;

    universe_guard.set_rgb(channel, r, g, b)
}
