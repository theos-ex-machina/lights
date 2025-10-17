use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::fixture::{
    loader::FixtureLoader,
    ofl::{OflFixture, OflManufacturer},
    patch::{FixtureProfile, PatchedFixture},
};

use anyhow::{anyhow, Result};

/// Registry for managing fixture definitions and creating patched fixtures
pub struct FixtureRegistry {
    loader: FixtureLoader,
    profile_cache: HashMap<String, Arc<FixtureProfile>>, // Key: "manufacturer/fixture/mode"
}

impl FixtureRegistry {
    pub fn new<P: AsRef<Path>>(fixture_data_path: P) -> Result<Self> {
        let mut loader = FixtureLoader::new(fixture_data_path);
        loader.load_manufacturers()?;

        Ok(FixtureRegistry {
            loader,
            profile_cache: HashMap::new(),
        })
    }

    /// Get all available manufacturers
    pub fn get_manufacturers(&self) -> Option<&HashMap<String, OflManufacturer>> {
        self.loader.get_manufacturers()
    }

    /// Get available fixtures for a manufacturer
    pub fn get_fixtures_for_manufacturer(
        &self,
        manufacturer: &str,
    ) -> Result<Vec<String>> {
        self.loader.list_fixtures_for_manufacturer(manufacturer)
    }

    /// Get available modes for a specific fixture
    pub fn get_modes_for_fixture(
        &mut self,
        manufacturer: &str,
        fixture_name: &str,
    ) -> Result<Vec<String>> {
        let fixture = self.loader.load_fixture(manufacturer, fixture_name)?;
        Ok(fixture.modes.iter().map(|mode| mode.name.clone()).collect())
    }

    /// Get or create a fixture profile for a specific manufacturer/fixture/mode combination
    pub fn get_fixture_profile(
        &mut self,
        manufacturer: &str,
        fixture_name: &str,
        mode_name: &str,
    ) -> Result<Arc<FixtureProfile>> {
        let cache_key = format!("{}/{}/{}", manufacturer, fixture_name, mode_name);

        // Return cached profile if available
        if let Some(profile) = self.profile_cache.get(&cache_key) {
            return Ok(profile.clone());
        }

        // Load the fixture if not already loaded
        let fixture = self.loader.load_fixture(manufacturer, fixture_name)?;

        // Find the requested mode
        let mode = fixture
            .modes
            .iter()
            .find(|m| m.name == mode_name)
            .ok_or_else(|| {
                anyhow!(
                    "Mode '{}' not found for fixture '{}/{}'",
                    mode_name, manufacturer, fixture_name
                )
            })?;

        // Create the profile
        let profile = Arc::new(FixtureProfile::from_ofl_fixture(fixture, mode));

        // Cache it
        self.profile_cache.insert(cache_key, profile.clone());

        Ok(profile)
    }

    /// Create a patched fixture with the specified parameters
    pub fn create_patched_fixture(
        &mut self,
        manufacturer: &str,
        fixture_name: &str,
        mode_name: &str,
        channel: usize,
        dmx_start: u16,
        label: String,
    ) -> Result<PatchedFixture> {
        let profile = self.get_fixture_profile(manufacturer, fixture_name, mode_name)?;

        Ok(PatchedFixture {
            id: format!("{}/{}", manufacturer, fixture_name),
            channel,
            profile,
            dmx_start,
            label,
        })
    }

    /// Discover all available fixtures across all manufacturers
    pub fn discover_all_fixtures(
        &self,
    ) -> Result<HashMap<String, Vec<String>>> {
        self.loader.discover_all_fixtures()
    }

    /// Search for fixtures by name (case-insensitive partial match)
    pub fn search_fixtures(
        &self,
        search_term: &str,
    ) -> Result<Vec<(String, String)>> {
        let all_fixtures = self.discover_all_fixtures()?;
        let search_lower = search_term.to_lowercase();
        let mut results = Vec::new();

        for (manufacturer, fixtures) in all_fixtures {
            for fixture in fixtures {
                if fixture.to_lowercase().contains(&search_lower) {
                    results.push((manufacturer.clone(), fixture));
                }
            }
        }

        results.sort();
        Ok(results)
    }

    /// Get fixture information (returns the loaded OFL fixture data)
    pub fn get_fixture_info(
        &mut self,
        manufacturer: &str,
        fixture_name: &str,
    ) -> Result<&OflFixture> {
        self.loader.load_fixture(manufacturer, fixture_name)
    }

    /// List all cached profiles
    pub fn get_cached_profiles(&self) -> &HashMap<String, Arc<FixtureProfile>> {
        &self.profile_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_fixture_registry() {
        // This test assumes the fixture-data directory exists
        if Path::new("fixture-data").exists() {
            let mut registry = FixtureRegistry::new("fixture-data").unwrap();

            // Test manufacturer discovery
            let manufacturers = registry.get_manufacturers();
            assert!(manufacturers.is_some());

            // Test fixture discovery
            let all_fixtures = registry.discover_all_fixtures().unwrap();
            assert!(!all_fixtures.is_empty());

            // Test search
            let search_results = registry.search_fixtures("par").unwrap();
            // Should find some fixtures with "par" in the name
            println!("Found {} fixtures matching 'par'", search_results.len());
        }
    }
}
