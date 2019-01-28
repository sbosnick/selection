// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// Implements #SPC-sel4platcrate.omap3

//! This crate provides the platform specifc parts of the sel4 library for the Omap3
//! platform.
//!
//! This crate will be empty if the target architecture isn't arm
//! A side effect of building this project (on the arm architecture) 
//! is that the seL4 microkernal for the Omap3 platform will be built.
//!
//! Currently this crate will also be empty for a debug profile. This is tied
//! to [issue 116] in the seL4 project.
//!
//! [issue 116]: https://github.com/seL4/seL4/issues/116

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate cty;

#[cfg(all(target_arch = "arm", not(debug_assertions)))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
