//! Unsafe Rust bindings to the Survex `img.c` library
//!
//! The functions, constants and structs contained within this module are direct bindings to
//! `img.c` and `img.h` from the [Survex project](https://github.com/ojwb/survex). For more
//! information about their function, refer to the source code for the aforementioned files
//! within the Survex repository.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
