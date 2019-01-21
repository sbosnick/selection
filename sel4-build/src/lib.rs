// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// Implements #SPC-sel4platcrate.buildcrate

//! The sel4-build crate provides build.rs support for the sel4-sys crate and 
//! the sel4-plat-* crates.
//!
//! This crate is really an implementation detail of those crates and there is
//! little reason to use it directly.
//!
//! The functions in this crate assume they are being called from a build.rs
//! file. In particular, they may panic if the usual CARGO_* environment variables
//! are not set.

#![cfg_attr(target_os = "none", no_std)]

#[cfg(not(target_os = "none"))]
pub use hosted::{CMakeTarget, Platform, copy_cmake_files, get_cargo_var};

#[cfg(not(target_os = "none"))]
mod hosted;
