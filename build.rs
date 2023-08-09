use std::env;
use std::path::PathBuf;

fn main() {
    let libdir_path = PathBuf::from("lib/survex")
        .canonicalize()
        .expect("Cannot find survex/src directory");

    let headers_path = libdir_path.join("img.h");
    let headers_path_str = headers_path.to_str().expect("Cannot convert path to string");

    let obj_path = libdir_path.join("img.o");
    let lib_path = libdir_path.join("libimg.a");

    println!("cargo:rustc-link-search={}", libdir_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=img");
    println!("cargo:rerun-if-changed={}", headers_path_str);

    if !std::process::Command::new("clang")
        .arg("-c")
        .arg("-o")
        .arg(&obj_path)
        .arg(libdir_path.join("img.c"))
        .output()
        .expect("Failed to execute clang")
        .status
        .success()
    {
        panic!("Failed to compile img.c");
    }

    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .arg(obj_path)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        panic!("Failed to create libimg.a");
    }

    let bindings = bindgen::Builder::default()
        .header(headers_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
