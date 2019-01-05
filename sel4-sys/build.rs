// Copyright 2018 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

extern crate bindgen;
extern crate cmake;

use std::{env, fs};
use std::path::{Path, PathBuf};

fn main() {
    // get the base directories and the architecture
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_root = Path::new("seL4-10.1.1");
    let arch = Arch::from_cargo();

    // build the native sel4 library
    // Implements #SPC-sel4syscrate.cmake
    let toolchain_file = src_root.join("gcc.cmake");
    cmake::Config::new(".")
        .generator("Ninja")
        .define("CMAKE_TOOLCHAIN_FILE", toolchain_file)
        .define("LibSel4FunctionAttributes", "public")
        .set_arch(arch)
        .set_profile(Profile::from_cargo())
        .build_target("libsel4.a")
        .very_verbose(true)
        .build();

    // copy the native sel4 library to the expected location
    let from = out_dir.join("build/libsel4/libsel4.a");
    let to = out_dir.join("libsel4.a");
    fs::copy(from, to).expect("Unable to copy libsel4.a");

    // generate the rust bindings to the native sel4 library
    // Implements #SPC-sel4syscrate.bingen
    let libsel4_src = src_root.join("libsel4");
    let libsel4_build = out_dir.join("build/libsel4");
    let kernel_build = out_dir.join("build/kernel");
    let header = libsel4_src.join("include/sel4/sel4.h");
    bindgen::builder().header(header.to_str().unwrap())
        .use_core()
        .ctypes_prefix("cty")
        .generate_comments(false)
        // Exclude the CONFIG_* constants because these include constants for
        // configuration options that will be different in the final microkernal
        // and it would be misleading to include them here.
        .blacklist_item("CONFIG_.*")
        // Exclude the constants related to the HardwareDebugAPI because these
        // vary from platform to platform and belong in the platform specific
        // crate instead of here.
        .blacklist_item("seL4_FirstBreakpoint")
        .blacklist_item("seL4_FirstDualFunctionMonitor")
        .blacklist_item("seL4_FirstWatchpoint")
        .blacklist_item("seL4_NumDualFunctionMonitors")
        .blacklist_item("seL4_NumExclusiveBreakpoints")
        .blacklist_item("seL4_NumExclusiveWatchpoints")
        .blacklist_item("seL4_NumHWBreakpoints")
        .clang_args(
            arch.include_dirs(&libsel4_src, &libsel4_build)
                .iter()
                .map(|path| format!("-I{}", path.display()))
        )
        .clang_arg(format!("-I{}", kernel_build.join("gen_config").display()))
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();
}

trait CmakeExt {
    fn set_arch(&mut self, arch: Arch) -> &mut Self;
    fn set_profile(&mut self, profile: Profile) -> &mut Self;
}

impl CmakeExt for cmake::Config {
    fn set_arch(&mut self, arch: Arch) -> &mut Self {
        let arch: &str = arch.into();
        self.define(arch, "1")
    }

    fn set_profile(&mut self, profile: Profile) -> &mut Self {
        use self::Profile::*;

        match profile {
            Release => {
                self.define("KernelVerificationBuild", "ON")
            }
            Debug => {
                self.define("KernelVerificationBuild", "OFF")
                    .define("KernelDebugBuild", "ON")
                    .define("KernelPrinting", "ON")
                    .define("HardwareDebugAPI", "ON")
            }
        }
    }
}

#[derive(Clone, Copy)]
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

    fn include_dirs(&self, src: &Path, build: &Path) -> Vec<PathBuf> {
        use self::Arch::*;

        // common directories
        let mut dirs = vec! [
            src.join("include"),
            build.join("include"),
            build.join("autoconf"),
            build.join("gen_config"),
        ];

        // arch_include directories
        match self {
            Ia32 | X86_64 => {
                dirs.push(src.join("arch_include/x86"));
                dirs.push(build.join("arch_include/x86"));
            }
            Aarch32 | Aarch64 => {
                dirs.push(src.join("arch_include/arm"));
                dirs.push(build.join("arch_include/arm"));
            }

        }

        // sel4_arch_include directories
        match self {
            Ia32 => {
                dirs.push(src.join("sel4_arch_include/ia32"));
                dirs.push(build.join("sel4_arch_include/ia32"));
            }
            X86_64 => {
                dirs.push(src.join("sel4_arch_include/x86_64"));
                dirs.push(build.join("sel4_arch_include/x86_64"));
            }
            Aarch32 => {
                dirs.push(src.join("sel4_arch_include/aarch32"));
                dirs.push(build.join("sel4_arch_include/aarch32"));
            }
            Aarch64 => {
                dirs.push(src.join("sel4_arch_include/aarch64"));
                dirs.push(build.join("sel4_arch_include/aarch64"));
            }
        }

        // mode_include directories
        match self {
            Ia32 | Aarch32 => dirs.push(src.join("mode_include/32")),
            X86_64 | Aarch64 => dirs.push(src.join("mode_include/64")),
        }

        // sel4_plat_include directories
        // NOTE: The contentes of the files in these direcotries will
        // be blacklisted at a later stage, but the direcotries have to
        // be included here for bindgen to work.
        match self {
            Ia32 | X86_64 => dirs.push(src.join("sel4_plat_include/pc99")),
            Aarch32 => dirs.push(src.join("sel4_plat_include/imx6")),
            Aarch64 => dirs.push(src.join("sel4_plat_include/tx1")),
        }

        dirs
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

enum Profile {
    Release,
    Debug,
}

impl Profile {
    fn new(profile: impl AsRef<str>) -> Result<Profile, String> {
        use self::Profile::*;

        match profile.as_ref() {
            "release" => Ok(Release),
            "debug" => Ok(Debug),
            profile => Err(format!("Unrecognized profile: {}", profile)),
        }
    }

    fn from_cargo() -> Profile {
        Profile::new(env::var("PROFILE").expect("PROFILE not set."))
            .expect("PROFILE not set to recognized profile.")
    }
}
