// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

#![cfg(not(target_os = "none"))]

extern crate file_diff;
extern crate tempfile;

use std::{
    ffi::{OsStr, OsString},
    fs::{self, File},
    path::Path,
    process::Command,
};

use sel4_build;

#[test]
// Imlements #TST-sel4syscrate.platform
//
// NOTE: This test uses CMAKE_BUILD_TYPE Release. It fails with a Debug
// CMAKE_BUILD_TYPE, but the resulting libsel4.a files are identical once
// they have been stripped (this has been manually verified once). This suggests
// that the debug information in libsel4 is not platform independant, though
// the library itself its. The library itself being binary indential is
// sufficient for TST-sel4syscrate.platform so we are OK using a Release
// CMAKE_BUILD_TYPE for this test.
fn libsel4_platform_independance() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sabre_dir = tempfile::tempdir().unwrap();
    let omap3_dir = tempfile::tempdir().unwrap();
    let sabre_build = sabre_dir.as_ref().join("build");
    let omap3_build = omap3_dir.as_ref().join("build");
    fs::create_dir(&sabre_build).expect("Unable to create build dir.");
    fs::create_dir(&omap3_build).expect("Unable to create build dir.");
    sel4_build::copy_cmake_files(sabre_dir.as_ref(), manifest_dir)
        .expect("Unable to copy cmake files.");
    sel4_build::copy_cmake_files(omap3_dir.as_ref(), manifest_dir)
        .expect("Unable to copy cmake files.");


    cmake_build(sabre_dir.as_ref(), sabre_build.as_ref(), manifest_dir, "sabre");
    cmake_build(omap3_dir.as_ref(), omap3_build.as_ref(), manifest_dir, "omap3");

    let sabre_lib = sabre_dir.as_ref().join("build/libsel4/libsel4.a");
    let omap3_lib = omap3_dir.as_ref().join("build/libsel4/libsel4.a");
    assert!(
        diff_path(sabre_lib, omap3_lib),
        "libsel4.a different for sabre and omap3 platforms"
    );
}

fn cmake_build(src: &Path, build: &Path, manifest: &Path, platform: &str,) {
    let toolchain_file = manifest.join("seL4/gcc.cmake");

    let output = Command::new("cmake")
        .current_dir(build)
        .arg(src)
        .args(&["-G", "Ninja"])
        .arg(define_arg("CMAKE_TOOLCHAIN_FILE", toolchain_file))
        .arg(define_arg("LibSel4FunctionAttributes", "public"))
        .arg(define_arg("AARCH32", "1"))
        .arg(define_arg("KernelARMPlatform", platform))
        .arg(define_arg("CMAKE_BUILD_TYPE", "Release"))
        //.arg(define_arg("CMAKE_BUILD_TYPE", "Debug"))
        .output()
        .expect("Unable to run cmake to generate project");

    assert!(
        output.status.success(),
        "cmake generation not sucessful\n\nstdout\n======\n{}\n\nstderr\n======\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new("cmake")
        .current_dir(build)
        .arg("--build")
        .arg(".")
        .args(&["--target", "libsel4.a"])
        .output()
        .expect("Unable to run cmake to build project");

    assert!(
        output.status.success(),
        "cmake generation not sucessful\n\nstdout\n======\n{}\n\nstderr\n======\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn diff_path(f1: impl AsRef<Path>, f2: impl AsRef<Path>) -> bool {
    match (File::open(f1.as_ref()), File::open(f2.as_ref())) {
        (Ok(mut f1), Ok(mut f2)) => file_diff::diff_files(&mut f1, &mut f2),
        _ => false,
    }
}

fn define_arg(var: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> OsString {
    let mut arg = OsString::new();

    arg.push("-D");
    arg.push(var.as_ref());
    arg.push("=");
    arg.push(value.as_ref());

    arg
}
