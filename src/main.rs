use std::{
    ffi::CString,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    cli::run_cli_threaded,
    fixture::{
        patch::{ChannelType, PatchedFixture, ETC_SOURCE_FOUR_CONVENTIONAL},
        registry::FixtureRegistry,
        Universe,
    },
};

mod cli;
mod fixture;

// Include the bindgen-generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    println!("Lights DMX Controller");
    println!("====================");

    // Initialize fixture registry
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

    // Create the universe with fixtures from registry
    let mut universe = Universe::new(0);

    // Try to load an ETC ColorSource PAR as an example
    match registry.create_patched_fixture(
        "etc",
        "colorsource-par",
        "5 Channel (Default)",
        1,  // Channel 1
        10, // DMX start address 10
        "Front wash".to_string(),
    ) {
        Ok(fixture) => {
            println!("✓ Created ETC ColorSource PAR fixture");
            universe.add_fixture(fixture);

            // Set some initial values
            if let Err(error) = universe.set_fixture_values(
                1,
                &[
                    (ChannelType::Intensity, 200u8),
                    (ChannelType::Red, 255u8),
                    (ChannelType::Green, 100u8),
                    (ChannelType::Blue, 50u8),
                ],
            ) {
                eprintln!("Error setting fixture values: {}", error);
            }
        }
        Err(e) => {
            println!("⚠ Could not load ETC ColorSource PAR: {}", e);

            // Fall back to conventional fixture
            let source_four = PatchedFixture {
                id: "Source Four".to_string(),
                channel: 75,
                profile: ETC_SOURCE_FOUR_CONVENTIONAL.clone(),
                dmx_start: 1,
                label: "center downlight".to_string(),
            };
            universe.add_fixture(source_four);

            if let Err(error) = universe.set_fixture_values(75, &[(ChannelType::Intensity, 255u8)])
            {
                eprintln!("{}", error);
            }
        }
    }

    run_with_universe(universe);
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

fn run_with_universe(universe: Universe) {
    // Wrap universe in Arc<Mutex<>> for thread-safe sharing
    let universe = Arc::new(Mutex::new(universe));

    // Open DMX port
    let port = CString::new("COM3").expect("Failed to create port string");
    let fd = unsafe { dmx_open(port.as_ptr()) };

    if fd < 0 {
        eprintln!("Failed to open DMX port COM3");
        return;
    }
    println!("DMX port opened successfully!");

    // Create shutdown channel for clean thread termination
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Clone the Arc for the DMX thread
    let universe_dmx = Arc::clone(&universe);

    // Spawn DMX output thread
    let dmx_handle = thread::spawn(move || {
        println!("DMX output thread started - sending data every 25ms");

        loop {
            // Check for shutdown signal (non-blocking)
            if shutdown_rx.try_recv().is_ok() {
                println!("DMX thread received shutdown signal");
                break;
            }

            // Lock the universe and send buffer
            if let Ok(universe_guard) = universe_dmx.lock() {
                unsafe {
                    if let Err(error) = universe_guard.send_buffer(fd) {
                        eprintln!("DMX send error: {}", error);
                        break; // Exit thread on error
                    }
                }
            } else {
                eprintln!("Failed to lock universe mutex");
                break;
            }

            // DMX refresh rate: ~40Hz (25ms between frames)
            thread::sleep(Duration::from_millis(25));
        }

        // Clean up: close DMX port
        unsafe {
            dmx_close(fd);
            println!("DMX port closed");
        }
    });

    // Run CLI on main thread with shared universe
    println!("Starting CLI interface...");
    println!("Type 'quit' or 'exit' to stop the program");

    run_cli_threaded(Arc::clone(&universe));

    // Signal DMX thread to stop cleanly
    println!("Shutting down...");
    if let Err(e) = shutdown_tx.send(()) {
        eprintln!("Failed to send shutdown signal: {}", e);
    }

    // Wait for DMX thread to finish gracefully
    if let Err(e) = dmx_handle.join() {
        eprintln!("DMX thread panicked: {:?}", e);
    } else {
        println!("DMX thread shut down cleanly");
    }
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
