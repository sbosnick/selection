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

    // build the native sel4 library
    cmake::Config::new(".")
        .generator("Ninja")
        .define("CMAKE_TOOLCHAIN_FILE", "seL4-10.1.1/gcc.cmake")
        .define("LibSel4FunctionAttributes", "public")
        .set_arch(Arch::from_cargo())
        .build_target("libsel4.a")
        .very_verbose(true)
        .build();

    // copy the native sel4 library to the expected location
    let from: PathBuf = [out_dir.as_ref(), "build", "libsel4", "libsel4.a"].iter().collect();
    let to: PathBuf = [out_dir.as_ref(), "libsel4.a"].iter().collect();
    fs::copy(from, to).expect("Unable to copy libsel4.a");
}

trait CmakeExt {
    fn set_arch(&mut self, arch: Arch) -> &mut Self;
}

impl CmakeExt for cmake::Config {
    fn set_arch(&mut self, arch: Arch) -> &mut Self {
        let arch: &str = arch.into();
        self.define(arch, "1")
    }
}

enum Arch {
    Ia32,
    X86_64,
    Aarch32,
    Aarch64,
}

impl Arch {
    fn new(arch: impl AsRef<str>) -> Result<Arch, String> {
        use self::Arch::*;

        match arch.as_ref() {
            "x86" => Ok(Ia32),
            "x86_64" => Ok(X86_64),
            "arm" => Ok(Aarch32),
            "aarch64" => Ok(Aarch64),
            arch => Err(format!("Unsupported architecture: {}", arch)),
        }
    }

    fn from_cargo() -> Arch {
        Arch::new(env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set."))
            .expect("CARGO_CFG_TARGET_ARCH not set to supported architecture.")
    }
}

impl From<Arch> for &'static str {
    fn from(arch : Arch) -> Self {
        use self::Arch::*;

        match arch {
            Ia32 => "IA32",
            X86_64 => "X86_64",
            Aarch32 => "AARCH32",
            Aarch64 => "AARCH64",
        }
    }
}
