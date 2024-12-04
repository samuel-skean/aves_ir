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
    // Is the solution to not owning this necessarily something without a lambda?
    let header_file_paths = headers_path.read_dir().expect("headers was not a directory").map(|e| {
        e.expect("Something wrong with a header file's directory entry.").path()
    });
    // It's definitely not useful to focus on this now, but it is irritating that it can't borrow the path.
    let header_file_path_strings = header_file_paths.map(|path| {
        path.to_str().unwrap().to_owned()
    });
    let src_path = libdir_path.join("src");
    let src_file_paths = src_path.read_dir().expect("src was not a directory").map(|e| {
        e.expect("Something wrong with a source file's directory entry.").path()
    });
    let build_path = libdir_path.join("build");

    // MY ADDITION: Tell Cargo to re-run the script if any of c files change:
    println!("cargo::rerun-if-changed={}", src_path.to_str().unwrap());
    
    cc::Build::new()
        .files(src_file_paths)
        .include(headers_path)
        .out_dir(build_path)
        .compile("aves");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .headers(header_file_path_strings)
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
