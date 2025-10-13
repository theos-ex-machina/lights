use std::env;
use std::path::PathBuf;

fn main() {
    // Tauri build
    tauri_build::build();

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=.");

    // Tell cargo to tell rustc to link the dmx library
    println!("cargo:rustc-link-lib=static=dmx");

    // Tell cargo to invalidate the built crate whenever the C source changes
    println!("cargo:rerun-if-changed=csrc/dmx.c");
    println!("cargo:rerun-if-changed=include/dmx.h");

    // Compile the C library
    cc::Build::new()
        .file("csrc/dmx.c")
        .include("include")
        .compile("dmx");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for
        .header("include/dmx.h")
        // Tell bindgen about include paths
        .clang_arg("-Iinclude")
        // Generate bindings for functions only (no types/constants we don't need)
        .allowlist_function("dmx_.*")
        // Generate documentation from C comments
        .generate_comments(true)
        // Generate compatible bindings
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
