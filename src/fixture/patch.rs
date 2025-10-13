use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

/// These are the patch entries in the universe
#[derive(Clone)]
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

/// info on a single channel of a fixture
#[derive(Clone)]
pub struct ChannelInfo {
    pub function: ChannelType,
    pub offset: u8, // the distance from the start bit of the fixture
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ChannelType {
    RED,
    BLUE,
    GREEN,
    PAN,
    TILT,
    INTENSITY,
    Custom(String),
}

pub static ETC_SOURCE_FOUR_CONVENTIONAL: LazyLock<Arc<FixtureProfile>> = LazyLock::new(|| {
    Arc::new(FixtureProfile {
        name: "ETC Source Four Conventional".to_string(),
        footprint: 1,
        channels: [(ChannelType::INTENSITY, 0u8)].into_iter().collect(),
    })
});

pub static RGB_LED_FIXTURE: LazyLock<Arc<FixtureProfile>> = LazyLock::new(|| {
    Arc::new(FixtureProfile {
        name: "RGB LED Fixture".to_string(),
        footprint: 4,
        channels: [
            (ChannelType::INTENSITY, 0u8),
            (ChannelType::RED, 1u8),
            (ChannelType::GREEN, 2u8),
            (ChannelType::BLUE, 3u8),
        ]
        .into_iter()
        .collect(),
    })
});
