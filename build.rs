use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .file("csrc/flanterm.c")
        .file("csrc/backends/fb.c")
        .include("csrc")
        .include("csrc/backends")
        .compile("flanterm");

    let bindings = bindgen::Builder::default()
        .use_core()
        // The input header we would like to generate
        // bindings for.
        .header("csrc/wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
