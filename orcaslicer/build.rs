use std::env;
use std::path::PathBuf;

use anyhow::Result;

fn main() {
    build_orcaslicer().unwrap();

    // Get the current directory.
    let current_dir = env::current_dir().unwrap();

    // Tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search={}/../bindings/Orcaslicer",
        current_dir.display()
    );

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    //println!("cargo:rustc-link-lib=bz2");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn orcaslicer_dir() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    current_dir.join("..").join("bindings").join("Orcaslicer")
}

// Build on macos.
#[cfg(target_os = "macos")]
fn build_orcaslicer() -> Result<()> {
    // Build the deps.
    let output = std::process::Command::new("./build_release_macos.sh")
        .current_dir(orcaslicer_dir())
        .arg("-d")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build Orcaslicer deps: {}", stderr);
    }

    // Build the slicer.
    let output = std::process::Command::new("./build_release_macos.sh")
        .current_dir(orcaslicer_dir())
        .arg("-s")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build Orcaslicer: {}", stderr);
    }

    Ok(())
}

// Build on linux.
#[cfg(target_os = "linux")]
fn build_orcaslicer() -> Result<()> {
    // Build the slicer.
    let output = std::process::Command::new("./BuildLinux.sh")
        .current_dir(orcaslicer_dir())
        .arg("-ds")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build Orcaslicer: {}", stderr);
    }

    Ok(())
}

// Build on windows.
#[cfg(target_os = "windows")]
fn build_orcaslicer() -> Result<()> {
    // Build the slicer.
    let output = std::process::Command::new("build_release.bat")
        .current_dir(orcaslicer_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build Orcaslicer: {}", stderr);
    }

    Ok(())
}
