// Copyright 2018 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use sel4_build;

fn main() {
    let target = sel4_build::CMakeTarget::Library;

    // build the native sel4 library
    // Implements #SPC-sel4syscrate.cmake
    target.build();

    // generate the rust bindings to the native sel4 library
    // Implements #SPC-sel4syscrate.bingen
    target.bindgen();

    // outuput the cargo metadata to link with libsel4.a
    if sel4_build::get_cargo_var("CARGO_CFG_TARGET_OS") == "none" {
        println!("cargo:rustc-link-lib=static=sel4");
        println!(
            "cargo:rustc-link-search=native={}",
            sel4_build::get_cargo_var("OUT_DIR")
        );
    }
}
