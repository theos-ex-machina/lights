mod cli;
mod fixture;
mod universe;

use std::{ffi::CString, thread};

use crate::{
    cli::run_cli,
    fixture::registry::FixtureRegistry,
    universe::{cue::CueEngine, dmx_thread, Universe},
};

// Include the bindgen-generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    // Create command channel
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    let mut registry = match FixtureRegistry::new("fixture-data") {
        Ok(registry) => {
            println!("✓ Loaded fixture database from fixture-data/");
            registry
        }
        Err(e) => {
            println!("⚠ Could not load fixture database: {}", e);
            return;
        }
    };

    // Create universe (will be moved to DMX thread)
    let mut universe = Universe::new(0);

    match registry.create_patched_fixture(
        "etc",
        "colorsource-par",
        "5 Channel (Default)",
        1,  // Channel 1
        10, // DMX start address 10
        "Front wash".to_string(),
    ) {
        Ok(fixture) => universe.add_fixture(fixture),
        Err(error) => eprintln!("Error adding fixture: {}", error),
    }

    // Setup DMX
    let port = CString::new("COM3").expect("Failed to create port string");
    let fd = unsafe { dmx_open(port.as_ptr()) };

    #[cfg(not(feature = "no-dmx"))]
    if fd < 0 {
        eprintln!("Failed to open DMX port COM3");
        return;
    }

    // Start DMX thread (takes ownership of universe)
    let dmx_handle = thread::spawn(move || {
        dmx_thread(universe, command_rx, shutdown_rx, fd);
    });

    // Create cue engine with command sender
    let mut show = CueEngine::new(command_tx.clone());

    // run cli
    run_cli(command_tx.clone(), &mut show);

    // Shutdown
    println!("Shutting down...");
    shutdown_tx.send(()).ok();
    dmx_handle.join().ok();
}

#[allow(dead_code)]
fn demonstrate_fixture_registry(registry: &mut FixtureRegistry) {
    println!("\nFixture Database Information:");
    println!("============================");

    // Show available manufacturers
    if let Some(manufacturers) = registry.get_manufacturers() {
        println!("Available manufacturers: {}", manufacturers.len());
        let mut mfg_names: Vec<_> = manufacturers.keys().collect();
        mfg_names.sort();
        for (i, name) in mfg_names.iter().take(5).enumerate() {
            if let Some(mfg) = manufacturers.get(*name) {
                println!("  {}. {} ({})", i + 1, mfg.name, name);
            }
        }
        if manufacturers.len() > 5 {
            println!("  ... and {} more", manufacturers.len() - 5);
        }
    }

    // Try to show some ETC fixtures
    match registry.get_fixtures_for_manufacturer("etc") {
        Ok(fixtures) => {
            println!("\nETC fixtures available: {}", fixtures.len());
            for (i, fixture) in fixtures.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, fixture);
            }
            if fixtures.len() > 3 {
                println!("  ... and {} more", fixtures.len() - 3);
            }
        }
        Err(_) => {
            println!("\nNo ETC fixtures found");
        }
    }

    // Search for PAR fixtures
    match registry.search_fixtures("par") {
        Ok(results) => {
            println!("\nPAR fixtures found: {}", results.len());
            for (i, (mfg, fixture)) in results.iter().take(3).enumerate() {
                println!("  {}. {} / {}", i + 1, mfg, fixture);
            }
            if results.len() > 3 {
                println!("  ... and {} more", results.len() - 3);
            }
        }
        Err(_) => {
            println!("\nNo PAR fixtures found");
        }
    }

    println!();
}

/// reads from the dmx frame and dumps it to std out
#[allow(dead_code)]
unsafe fn dump_frame(fd: i32) {
    let mut buffer = [0u8; 513]; // DMX frame can be up to 513 bytes (start code + 512 channels)
    let mut frame_count: u64 = 0;

    let mut num_bytes: i32;
    loop {
        num_bytes = dmx_read_frame(fd, buffer.as_mut_ptr(), buffer.len() as i32);

        if num_bytes > 0 {
            frame_count += 1;
            let frame_type = match buffer[0] {
                0x00 => "DMX Lighting Data",
                0xCC => "RDM",
                0x17 => "Text Packets",
                _ => "probably some manufacturer bs",
            };

            println!(
                "Frame {}: {} bytes, type: {}",
                frame_count, num_bytes, frame_type
            );

            print!("Data: ");
            for i in 0..(num_bytes as usize) {
                print!("{:02X} ", buffer[i]);
                // Add newline every 16 bytes for readability
                if (i + 1) % 16 == 0 {
                    println!();
                    print!("      ");
                }
            }
            println!();
            println!("---");
        } else if num_bytes == 0 {
            // No data available, short delay to prevent busy waiting
            std::thread::sleep(std::time::Duration::from_millis(1));
        } else {
            eprintln!("Error reading DMX frame: {}", num_bytes);
            break;
        }
    }
}
