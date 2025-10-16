use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fixture::ofl::{OflFixture, OflManufacturers};

pub struct FixtureLoader {
    fixture_data_path: PathBuf,
    manufacturers: Option<OflManufacturers>,
    loaded_fixtures: HashMap<String, OflFixture>, // Key format: "manufacturer/fixture"
}

impl FixtureLoader {
    pub fn new<P: AsRef<Path>>(fixture_data_path: P) -> Self {
        Self {
            fixture_data_path: fixture_data_path.as_ref().to_path_buf(),
            manufacturers: None,
            loaded_fixtures: HashMap::new(),
        }
    }

    /// Load the manufacturers database
    pub fn load_manufacturers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let manufacturers_path = self.fixture_data_path.join("manufacturers.json");
        let content = fs::read_to_string(&manufacturers_path)?;
        self.manufacturers = Some(serde_json::from_str(&content)?);
        Ok(())
    }

    /// Get list of available manufacturers
    pub fn get_manufacturers(
        &self,
    ) -> Option<&HashMap<String, crate::fixture::ofl::OflManufacturer>> {
        self.manufacturers.as_ref().map(|m| &m.manufacturers)
    }

    /// Load a specific fixture by manufacturer and fixture name
    pub fn load_fixture(
        &mut self,
        manufacturer: &str,
        fixture_name: &str,
    ) -> Result<&OflFixture, Box<dyn std::error::Error>> {
        let key = format!("{}/{}", manufacturer, fixture_name);

        // Check if fixture is already loaded
        if self.loaded_fixtures.contains_key(&key) {
            return Ok(&self.loaded_fixtures[&key]);
        }

        // Try to load the fixture file
        let fixture_path = self
            .fixture_data_path
            .join(manufacturer)
            .join(format!("{}.json", fixture_name));

        if !fixture_path.exists() {
            return Err(format!("Fixture file not found: {}", fixture_path.display()).into());
        }

        let content = fs::read_to_string(&fixture_path)?;
        let fixture: OflFixture = serde_json::from_str(&content)?;

        self.loaded_fixtures.insert(key.clone(), fixture);
        Ok(&self.loaded_fixtures[&key])
    }

    /// Discover available fixtures for a manufacturer
    pub fn list_fixtures_for_manufacturer(
        &self,
        manufacturer: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let manufacturer_dir = self.fixture_data_path.join(manufacturer);

        if !manufacturer_dir.exists() || !manufacturer_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut fixtures = Vec::new();
        for entry in fs::read_dir(&manufacturer_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    fixtures.push(stem.to_string());
                }
            }
        }

        fixtures.sort();
        Ok(fixtures)
    }

    /// Discover all available fixtures
    pub fn discover_all_fixtures(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut all_fixtures = HashMap::new();

        for entry in fs::read_dir(&self.fixture_data_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(manufacturer) = path.file_name().and_then(|s| s.to_str()) {
                    // Skip the manufacturers.json file and any hidden directories
                    if manufacturer.starts_with('.') || manufacturer == "manufacturers.json" {
                        continue;
                    }

                    match self.list_fixtures_for_manufacturer(manufacturer) {
                        Ok(fixtures) => {
                            if !fixtures.is_empty() {
                                all_fixtures.insert(manufacturer.to_string(), fixtures);
                            }
                        }
                        Err(_) => continue, // Skip directories that can't be read
                    }
                }
            }
        }

        Ok(all_fixtures)
    }

    /// Get a reference to a loaded fixture
    pub fn get_loaded_fixture(
        &self,
        manufacturer: &str,
        fixture_name: &str,
    ) -> Option<&OflFixture> {
        let key = format!("{}/{}", manufacturer, fixture_name);
        self.loaded_fixtures.get(&key)
    }

    /// Get all loaded fixtures
    pub fn get_all_loaded_fixtures(&self) -> &HashMap<String, OflFixture> {
        &self.loaded_fixtures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_fixture_loader() {
        // This test assumes the fixture-data directory exists
        if Path::new("fixture-data").exists() {
            let mut loader = FixtureLoader::new("fixture-data");

            // Test loading manufacturers
            assert!(loader.load_manufacturers().is_ok());
            assert!(loader.get_manufacturers().is_some());

            // Test discovering fixtures
            let all_fixtures = loader.discover_all_fixtures().unwrap();
            assert!(!all_fixtures.is_empty());
        }
    }
}
