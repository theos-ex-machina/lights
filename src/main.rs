use std::sync::{Arc, Mutex};

#[cfg(not(feature = "no-dmx"))]
use std::{ffi::CString, thread, time::Duration};

use crate::fixture::{
    patch::{PatchedFixture, ETC_SOURCE_FOUR_CONVENTIONAL, RGB_LED_FIXTURE},
    Universe,
};

mod cli;
mod fixture;
mod gui;

// Include the bindgen-generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    // Create the universe with initial fixtures
    let mut universe = Universe::new(0);

    // Add some example fixtures
    let source_four = PatchedFixture {
        id: "Source Four".to_string(),
        channel: 1,
        profile: ETC_SOURCE_FOUR_CONVENTIONAL.clone(),
        dmx_start: 1,
        label: "Center Downlight".to_string(),
    };
    universe.add_fixture(source_four);

    let source_four_2 = PatchedFixture {
        id: "Source Four 2".to_string(),
        channel: 2,
        profile: ETC_SOURCE_FOUR_CONVENTIONAL.clone(),
        dmx_start: 2,
        label: "Side Light".to_string(),
    };
    universe.add_fixture(source_four_2);

    // Add RGB LED fixture
    let rgb_led = PatchedFixture {
        id: "RGB LED".to_string(),
        channel: 3,
        profile: RGB_LED_FIXTURE.clone(),
        dmx_start: 10,
        label: "RGB Strip".to_string(),
    };
    universe.add_fixture(rgb_led);

    // Wrap universe in Arc<Mutex<>> for thread-safe sharing
    let universe = Arc::new(Mutex::new(universe));

    // DMX hardware support - disabled only when 'no-dmx' feature is enabled
    #[cfg(not(feature = "no-dmx"))]
    {
        // Open DMX port
        let port = CString::new("COM3").expect("Failed to create port string");
        let fd = unsafe { dmx_open(port.as_ptr()) };

        if fd >= 0 {
            println!("✓ DMX port opened successfully!");

            // Clone the Arc for the DMX thread
            let universe_dmx = Arc::clone(&universe);

            // Spawn DMX output thread
            thread::spawn(move || {
                println!("DMX output thread started - sending data every 25ms");

                loop {
                    // Lock the universe and send buffer
                    if let Ok(universe_guard) = universe_dmx.lock() {
                        unsafe {
                            if let Err(error) = universe_guard.send_buffer(fd) {
                                eprintln!("DMX send error: {}", error);
                                break;
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
        } else {
            eprintln!("⚠ Failed to open DMX port COM3 - running without DMX output");
        }
    }

    #[cfg(feature = "no-dmx")]
    {
        println!("ℹ Running in NO-DMX mode - DMX hardware disabled");
        println!("  To enable DMX: cargo run (without --features no-dmx)");
    }

    // Start Tauri application
    tauri::Builder::default()
        .manage(universe)
        .invoke_handler(tauri::generate_handler![
            gui::get_fixtures,
            gui::set_channel_value,
            gui::blackout,
            gui::set_intensity,
            gui::set_rgb
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
