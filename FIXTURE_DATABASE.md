# Fixture Database Integration

This document shows how the OFL (Open Fixture Library) fixture database has been integrated into the lights DMX controller.

## Implementation Overview

The implementation adds several new modules to handle fixture data:

### 1. OFL Data Structures (`src/fixture/ofl.rs`)
- `OflFixture`: Complete fixture definition from JSON
- `OflMode`: Different channel configurations (e.g., 5ch, RGB, etc.)  
- `OflCapability`: Individual channel capabilities and ranges
- `OflManufacturers`: Manufacturer database

### 2. Fixture Loader (`src/fixture/loader.rs`)
- `FixtureLoader`: Loads JSON files from the `fixture-data/` directory
- Caches loaded fixtures for performance
- Provides discovery methods for manufacturers and fixtures

### 3. Fixture Registry (`src/fixture/registry.rs`)
- `FixtureRegistry`: High-level interface for fixture management
- Creates `PatchedFixture` instances from OFL data
- Handles mode selection and profile creation

### 4. Enhanced Patch System (`src/fixture/patch.rs`)
- Extended `ChannelType` enum with many more channel types
- Conversion methods from OFL channel names/types to `ChannelType`
- `FixtureProfile::from_ofl_fixture()` method

## Usage Examples

### Basic Registry Usage

```rust
use crate::fixture::registry::FixtureRegistry;

// Create registry and load manufacturers database
let mut registry = FixtureRegistry::new("fixture-data")?;

// Discover available manufacturers
let manufacturers = registry.get_manufacturers();
println!("Available manufacturers: {}", manufacturers.len());

// Find fixtures for a specific manufacturer
let etc_fixtures = registry.get_fixtures_for_manufacturer("etc")?;
println!("ETC has {} fixtures", etc_fixtures.len());

// Search for specific fixture types
let par_fixtures = registry.search_fixtures("par")?;
println!("Found {} PAR fixtures", par_fixtures.len());
```

### Creating Fixtures from Database

```rust
// Create a patched fixture from the database
let fixture = registry.create_patched_fixture(
    "etc",                      // Manufacturer
    "colorsource-par",          // Fixture model
    "5 Channel (Default)",      // Mode name
    1,                          // Universe channel
    10,                         // DMX start address
    "Front wash".to_string(),   // Label
)?;

// Add to universe
universe.add_fixture(fixture);
```

### Available Channel Types

The `ChannelType` enum now supports:

**Color Channels:**
- `Red`, `Green`, `Blue`, `Amber`, `Lime`, `Cyan`, `Magenta`, `Yellow`
- `White`, `WarmWhite`, `CoolWhite`, `Uv`

**Movement:**
- `Pan`, `Tilt`, `PanFine`, `TiltFine`

**General:**
- `Intensity`, `Dimmer`, `Strobe`

**Color Control:**
- `ColorMacros`, `ColorTemperature`, `Hue`, `Saturation`

**Effects:**
- `Gobo`, `GoboRotation`, `Prism`, `Iris`, `Focus`, `Zoom`, `Frost`

**Control:**
- `ModeSelect`, `Speed`, `SoundSensitivity`

**Custom:**
- `Custom(String)` for unsupported or vendor-specific channels

## Database Statistics

As of this implementation, the fixture database contains:

- **129 manufacturers** including major lighting brands
- **Thousands of fixture definitions** across all manufacturers
- **Multiple modes per fixture** (e.g., 1ch, 3ch, 5ch, RGB, RGBW, etc.)

### Sample Manufacturers
- ETC (16 fixtures)
- American DJ (100+ fixtures)
- Chauvet Professional
- Martin Professional  
- Clay Paky
- Robe
- Elation
- And many more...

## Integration Benefits

1. **No More Hardcoded Fixtures**: Load any fixture from the comprehensive database
2. **Multiple Modes**: Each fixture supports multiple channel configurations
3. **Standardized Data**: Uses the Open Fixture Library standard format
4. **Easy Discovery**: Search and browse available fixtures
5. **Future-Proof**: New fixtures can be added by updating JSON files

## Example Output

When running the program, you'll see:

```
Lights DMX Controller
====================
✓ Loaded fixture database from fixture-data/

Fixture Database Information:
============================
Available manufacturers: 129
  1. 5Star Systems (5star-systems)
  2. Abstract (abstract)
  3. Acoustic Control (acoustic-control)
  4. ADB (adb)
  5. AFX (afx)
  ... and 124 more

ETC fixtures available: 16
  1. colorsource-par
  2. colorsource-par-deep-blue
  3. colorsource-spot
  ... and 13 more

PAR fixtures found: 101
  1. acoustic-control / par-180-cob-3in1
  2. american-dj / dotz-par
  3. american-dj / flat-par-qa12
  ... and 98 more

✓ Created ETC ColorSource PAR fixture
```

This shows the system successfully loaded 129 manufacturers and created a working fixture from the database.