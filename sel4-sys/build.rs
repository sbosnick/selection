// Copyright 2018 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

extern crate cmake;

use std::{env, fs};
use std::path::PathBuf;

fn main() {
    // get the values of the needed cargo-supplied variables
    let out_dir = env::var("OUT_DIR").unwrap();
    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
        "x86" => "IA32",
        "x86_64" => "X86_64",
        "arm" => "AARCH32",
        "aarch64" => "AARCH64",
        arch => panic!("Unsupported architecture: {}", arch),
    };

    // build the native sel4 library
    cmake::Config::new(".")
        .generator("Ninja")
        .define("CMAKE_TOOLCHAIN_FILE", "seL4-10.1.1/gcc.cmake")
        .define("LibSel4FunctionAttributes", "public")
        .define(arch, "1")
        .build_target("libsel4.a")
        .very_verbose(true)
        .build();

    // copy the native sel4 library to the expected location
    let from: PathBuf = [out_dir.as_ref(), "build", "libsel4", "libsel4.a"].iter().collect();
    let to: PathBuf = [out_dir.as_ref(), "libsel4.a"].iter().collect();
    fs::copy(from, to).expect("Unable to copy libsel4.a");
}
