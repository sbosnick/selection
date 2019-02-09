// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// Implements #SPC-sel4platcrate.tx1

//! This crate provides the platform specifc parts of the sel4 library for the Tx1
//! platform.
//!
//! This crate will be empty if the target architecture isn't arm
//! A side effect of building this project (on the arm architecture)
//! is that the seL4 microkernal for the Tx1 platform will be built.

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate cty;

#[cfg(target_arch = "aarch64")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
