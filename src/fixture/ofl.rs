use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Open Fixture Library fixture definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflFixture {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    pub categories: Vec<String>,
    pub meta: OflMeta,
    pub links: Option<OflLinks>,
    pub physical: Option<OflPhysical>,
    pub rdm: Option<OflRdm>,
    #[serde(rename = "availableChannels")]
    pub available_channels: HashMap<String, OflChannel>,
    pub modes: Vec<OflMode>,
    #[serde(rename = "fixtureKey")]
    pub fixture_key: String,
    #[serde(rename = "manufacturerKey")]
    pub manufacturer_key: String,
    #[serde(rename = "oflURL")]
    pub ofl_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflMeta {
    pub authors: Vec<String>,
    #[serde(rename = "createDate")]
    pub create_date: String,
    #[serde(rename = "lastModifyDate")]
    pub last_modify_date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflLinks {
    pub manual: Option<Vec<String>>,
    #[serde(rename = "productPage")]
    pub product_page: Option<Vec<String>>,
    pub video: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflPhysical {
    pub dimensions: Option<Vec<f32>>,
    pub weight: Option<f32>,
    pub power: Option<f32>,
    #[serde(rename = "DMXconnector")]
    pub dmx_connector: Option<String>,
    pub bulb: Option<OflBulb>,
    pub lens: Option<OflLens>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflBulb {
    #[serde(rename = "type")]
    pub bulb_type: String,
    pub lumens: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflLens {
    #[serde(rename = "degreesMinMax")]
    pub degrees_min_max: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflRdm {
    #[serde(rename = "modelId")]
    pub model_id: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflChannel {
    #[serde(rename = "fineChannelAliases")]
    pub fine_channel_aliases: Option<Vec<String>>,
    pub capability: Option<OflCapability>,
    pub capabilities: Option<Vec<OflCapability>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflCapability {
    #[serde(rename = "dmxRange")]
    pub dmx_range: Option<Vec<u8>>,
    #[serde(rename = "type")]
    pub capability_type: String,
    pub color: Option<String>,
    pub colors: Option<Vec<String>>,
    pub comment: Option<String>,
    // Add more fields as needed for different capability types
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflMode {
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    #[serde(rename = "rdmPersonalityIndex")]
    pub rdm_personality_index: Option<u32>,
    pub channels: Vec<String>,
}

/// Manufacturers database
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflManufacturers {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[serde(flatten)]
    pub manufacturers: HashMap<String, OflManufacturer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OflManufacturer {
    pub name: String,
    pub website: Option<String>,
    #[serde(rename = "rdmId")]
    pub rdm_id: Option<u32>,
}
