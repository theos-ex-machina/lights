use std::{
    ffi::CString,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    cli::run_cli_threaded,
    fixture::{
        patch::{ChannelType, PatchedFixture, ETC_SOURCE_FOUR_CONVENTIONAL},
        Universe,
    },
};

mod cli;
mod fixture;

// Include the bindgen-generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    // Create the universe with initial fixture
    let mut universe = Universe::new(0);

    let source_four = PatchedFixture {
        id: "Source Four".to_string(),
        channel: 75,
        profile: ETC_SOURCE_FOUR_CONVENTIONAL.clone(),
        dmx_start: 1, // DMX addresses start at 1, not 0
        label: "center downlight".to_string(),
    };
    universe.add_fixture(source_four);

    // Set initial intensity
    if let Err(error) = universe.set_fixture_values(75, &[(ChannelType::INTENSITY, 255u8)]) {
        eprintln!("{}", error);
    }

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

    // Clone the Arc for the DMX thread
    let universe_dmx = Arc::clone(&universe);

    // Spawn DMX output thread
    let dmx_handle = thread::spawn(move || {
        println!("DMX output thread started - sending data every 25ms");

        loop {
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

    // Signal DMX thread to stop (in a real implementation, you'd use a channel or atomic bool)
    println!("Shutting down...");

    // Wait for DMX thread to finish
    if let Err(e) = dmx_handle.join() {
        eprintln!("DMX thread panicked: {:?}", e);
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
