use std::env;
use std::path::PathBuf;

mod build_support;

use crate::build_support::dependency::BuildDependency;

fn main() {
    let build = crate::build_support::build::Build::new().unwrap();

    // Create a new dependency for glew.
    //let mut glew = build_support::dependency::Glew::new(&build);
    //glew.build().unwrap();

    // Create a new dependency for glfw.
    let mut glfw = build_support::dependency::Glfw::new(&build);
    glfw.build().unwrap();

    // Create a dependency for orcaslicer.
    let mut orcaslicer = build_support::dependency::Orcaslicer::new(&build);
    orcaslicer.build().unwrap();

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
