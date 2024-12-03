use std::env;
use std::path::PathBuf;

fn main() {
    // TODO: Make this rebuild the C code as well, on-demand.
    // This code is copied from a tutorial on rust-bindgen, modified minimally for these files.

    // This is the directory where the `c` library is located.
    let libdir_path = PathBuf::from("c_code")
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");

    // This is the path to the `c` headers file.
    let headers_path = libdir_path.join("include");
    let ir_h_path = headers_path.join("ir.h"); // TODO: Figure out what the issue is that prevents all the headers from working.

    let src_path = libdir_path.join("src");
    // MY ADDITION: This is the path to the c file.
    let ir_c_path = src_path.join("ir.c");
    let build_path = libdir_path.join("build");
    // This is the path to the intermediate object file for our library.
    let ir_o_path = build_path.join("ir.o");
    // This is the path to the static library file.
    let lib_path = build_path.join("libaves.a");

    // MY ADDITION: Tell Cargo to re-run the script if any of c files change:
    println!("cargo::rerun-if-changed={}", src_path.to_str().unwrap());

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo::rustc-link-search={}", build_path.to_str().unwrap());

    // Tell cargo to tell rustc to link our `aves` library. Cargo will
    // automatically know it must look for a `libaves.a` file.
    println!("cargo::rustc-link-lib=aves");

    // Run `clang` to compile the `ir.c` file into a `ir.o` object file.
    // Unwrap if it is not possible to spawn the process.
    if !std::process::Command::new("clang")
        .arg("-c")
        .arg("-o")
        .arg(&ir_o_path)
        .arg(&ir_c_path)
        .arg("-I")
        .arg(&headers_path)
        .output()
        .expect("could not spawn `clang`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not compile object file");
    }

    // Run `ar` to generate the `libir.a` file from the `ir.o` file.
    // Unwrap if it is not possible to spawn the process.
    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .arg(ir_o_path)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not emit library file");
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(ir_h_path.to_str().unwrap())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
