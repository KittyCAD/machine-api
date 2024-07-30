use std::env;
use std::path::PathBuf;

use anyhow::Result;

fn main() {
    build_orcaslicer().unwrap();
    let orcaslicer_dir = orcaslicer_dir();
    let orcaslicer_build_dir = orcaslicer_build_dir().unwrap();
    let arch = get_arch().unwrap();

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", orcaslicer_dir.display());

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    //println!("cargo:rustc-link-lib=bz2");

    let dep_include_dir = orcaslicer_dir
        .join("deps")
        .join(format!("build_{}", arch))
        .join(format!("OrcaSlicer_dep_{}", arch))
        .join("usr")
        .join("local")
        .join("include");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.hpp")
        .clang_arg(format!("-I{}", orcaslicer_dir.join("src").display()))
        .clang_arg(format!(
            "-I{}",
            orcaslicer_build_dir.join("src").join("libslic3r").display()
        ))
        .clang_arg(format!(
            "-I{}",
            orcaslicer_build_dir.join("src").join("libslic3r").display()
        ))
        .clang_arg(format!("-I{}", dep_include_dir.display()))
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

fn orcaslicer_build_dir() -> Result<PathBuf> {
    let arch = get_arch()?;
    Ok(orcaslicer_dir().join(format!("build_{}", arch)))
}

// Build on macos.
#[cfg(target_os = "macos")]
fn build_orcaslicer() -> Result<()> {
    // Check if the build already exists.
    if orcaslicer_build_dir()?
        .join("src")
        .join("slic3r")
        .join("Release")
        .join("liblibslic3r_gui.a")
        .exists()
    {
        return Ok(());
    }

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
    // Check if the build already exists.
    if orcaslicer_build_dir()?
        .join("src")
        .join("slic3r")
        .join("Release")
        .join("liblibslic3r_gui.a")
        .exists()
    {
        return Ok(());
    }

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

fn get_arch() -> Result<String> {
    match env::var("CARGO_CFG_TARGET_ARCH") {
        Ok(arch) => Ok(match arch.as_str() {
            "aarch64" => "arm64".to_string(),
            a => a.to_string(),
        }),
        Err(err) => anyhow::bail!("Failed to get target arch: {}", err),
    }
}
