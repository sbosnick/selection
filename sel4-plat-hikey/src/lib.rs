// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// Implements #SPC-sel4platcrate.hikey

//! This crate provides the platform specifc parts of the sel4 library for the Hikey
//! platform.
//!
//! This crate will be empty if the target architecture isn't arm or aarch64
//! A side effect of building this project (on the arm and aarch64 architectures)
//! is that the seL4 microkernal for the Hikey platform will be built.

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate cty;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
