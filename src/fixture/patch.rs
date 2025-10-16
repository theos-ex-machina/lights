use crate::fixture::ofl::{OflFixture, OflMode};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::LazyLock;

/// These are the patch entries in the universe
#[derive(Clone)]
#[allow(unused)]
pub struct PatchedFixture {
    pub id: String,
    pub channel: usize,
    pub profile: Arc<FixtureProfile>,
    pub dmx_start: u16,
    pub label: String,
}

/// describes one fixture type (ex, source four conventional)
#[derive(Clone)]
pub struct FixtureProfile {
    pub name: String,
    pub footprint: u8,
    /// Type, offset
    pub channels: HashMap<ChannelType, u8>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[allow(unused)]
pub enum ChannelType {
    // Color channels
    Red,
    Green,
    Blue,
    Amber,
    Lime,
    Cyan,
    Magenta,
    Yellow,
    White,
    WarmWhite,
    CoolWhite,
    Uv,

    // Movement
    Pan,
    Tilt,
    PanFine,
    TiltFine,

    // General
    Intensity,
    Dimmer,
    Strobe,

    // Color mixing/selection
    ColorMacros,
    ColorTemperature,
    Hue,
    Saturation,

    // Effects
    Gobo,
    GoboRotation,
    Prism,
    Iris,
    Focus,
    Zoom,
    Frost,

    // Control
    ModeSelect,
    Speed,
    SoundSensitivity,

    // Generic/Custom
    Custom(String),
}

impl ChannelType {
    /// Convert from OFL capability type string to ChannelType
    pub fn from_ofl_capability_type(capability_type: &str) -> Self {
        match capability_type {
            "Intensity" => ChannelType::Intensity,
            "ColorIntensity" => ChannelType::Intensity, // Will need color context
            "Pan" => ChannelType::Pan,
            "Tilt" => ChannelType::Tilt,
            "PanContinuous" => ChannelType::Pan,
            "TiltContinuous" => ChannelType::Tilt,
            "ColorPreset" => ChannelType::ColorMacros,
            "ColorTemperature" => ChannelType::ColorTemperature,
            "Strobe" => ChannelType::Strobe,
            "StrobeSpeed" => ChannelType::Strobe,
            "StrobeDuration" => ChannelType::Strobe,
            "Generic" => ChannelType::Custom("Generic".to_string()),
            "NoFunction" => ChannelType::Custom("NoFunction".to_string()),
            _ => ChannelType::Custom(capability_type.to_string()),
        }
    }

    /// Convert from OFL channel name to ChannelType
    pub fn from_ofl_channel_name(channel_name: &str) -> Self {
        let name_lower = channel_name.to_lowercase();
        match name_lower.as_str() {
            "red" => ChannelType::Red,
            "green" => ChannelType::Green,
            "blue" => ChannelType::Blue,
            "amber" => ChannelType::Amber,
            "lime" => ChannelType::Lime,
            "cyan" => ChannelType::Cyan,
            "magenta" => ChannelType::Magenta,
            "yellow" => ChannelType::Yellow,
            "white" => ChannelType::White,
            "warm white" | "warmwhite" => ChannelType::WarmWhite,
            "cool white" | "coolwhite" => ChannelType::CoolWhite,
            "uv" => ChannelType::Uv,
            "pan" => ChannelType::Pan,
            "tilt" => ChannelType::Tilt,
            "pan fine" => ChannelType::PanFine,
            "tilt fine" => ChannelType::TiltFine,
            "intensity" => ChannelType::Intensity,
            "dimmer" => ChannelType::Dimmer,
            "strobe" => ChannelType::Strobe,
            "color macros" => ChannelType::ColorMacros,
            "color temperature" => ChannelType::ColorTemperature,
            "hue" => ChannelType::Hue,
            "saturation" => ChannelType::Saturation,
            "gobo" => ChannelType::Gobo,
            "gobo rotation" => ChannelType::GoboRotation,
            "prism" => ChannelType::Prism,
            "iris" => ChannelType::Iris,
            "focus" => ChannelType::Focus,
            "zoom" => ChannelType::Zoom,
            "frost" => ChannelType::Frost,
            "mode select" => ChannelType::ModeSelect,
            "speed" => ChannelType::Speed,
            "sound sensitivity" => ChannelType::SoundSensitivity,
            _ => ChannelType::Custom(channel_name.to_string()),
        }
    }
}

pub static ETC_SOURCE_FOUR_CONVENTIONAL: LazyLock<Arc<FixtureProfile>> = LazyLock::new(|| {
    Arc::new(FixtureProfile {
        name: "ETC Source Four Conventional".to_string(),
        footprint: 1,
        channels: [(ChannelType::Intensity, 0u8)].into_iter().collect(),
    })
});

impl FixtureProfile {
    /// Create a FixtureProfile from an OFL fixture and mode
    pub fn from_ofl_fixture(ofl_fixture: &OflFixture, mode: &OflMode) -> Self {
        let mut channels = HashMap::new();

        for (channel_offset, channel_name) in mode.channels.iter().enumerate() {
            // Look up the channel definition in the OFL fixture
            if let Some(channel_def) = ofl_fixture.available_channels.get(channel_name) {
                // First try to infer from the channel name, as this is usually more specific
                let channel_type_from_name = ChannelType::from_ofl_channel_name(channel_name);

                let channel_type = match channel_type_from_name {
                    // If the name didn't match a known type, fall back to capability type
                    ChannelType::Custom(_) => {
                        if let Some(capability) = &channel_def.capability {
                            // For ColorIntensity capabilities, try to infer color from the "color" field
                            if capability.capability_type == "ColorIntensity" {
                                if let Some(color) = &capability.color {
                                    ChannelType::from_ofl_channel_name(color)
                                } else {
                                    ChannelType::from_ofl_capability_type(
                                        &capability.capability_type,
                                    )
                                }
                            } else {
                                ChannelType::from_ofl_capability_type(&capability.capability_type)
                            }
                        } else if let Some(capabilities) = &channel_def.capabilities {
                            // Multiple capabilities - use the first one
                            if let Some(first_cap) = capabilities.first() {
                                ChannelType::from_ofl_capability_type(&first_cap.capability_type)
                            } else {
                                channel_type_from_name
                            }
                        } else {
                            // No capabilities defined, keep the custom type
                            channel_type_from_name
                        }
                    }
                    // If the name matched a known type, use it
                    _ => channel_type_from_name,
                };

                channels.insert(channel_type, channel_offset as u8);
            }
        }

        FixtureProfile {
            name: format!("{} ({})", ofl_fixture.name, mode.name),
            footprint: mode.channels.len() as u8,
            channels,
        }
    }
}
