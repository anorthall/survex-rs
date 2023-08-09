use std::env;
use std::path::PathBuf;

fn main() {
    let outdir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let libdir_path = PathBuf::from("lib/survex")
        .canonicalize()
        .expect("Cannot find survex/src directory");

    let headers_path = libdir_path.join("img.h");
    let headers_path_str = headers_path
        .to_str()
        .expect("Cannot convert path to string");

    let obj_path = outdir_path.join("img.o");
    let lib_path = outdir_path.join("libimg.a");

    println!("cargo:rustc-link-search={}", outdir_path.to_str().unwrap());
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
        .allowlist_function("img_.*")
        .allowlist_type("img_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(outdir_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
