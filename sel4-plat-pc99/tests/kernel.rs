// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

#![cfg(not(target_os = "none"))]

use std::{fs, path::Path};

#[test]
// Implements #TST-sel4platcrate
fn kernel_elf_in_expected_location() {
    let out_dir = Path::new(env!("OUT_DIR"));

    let metadata = fs::metadata(out_dir.join("kernel.elf")).expect("Unable to find kernel.elf");

    if !metadata.is_file() {
        panic!("kernel.elf is not a file");
    }
}
