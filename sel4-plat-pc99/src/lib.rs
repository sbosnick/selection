// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// Implements #SPC-sel4platcrate.pc99
//
// This crate is the exemplar for other platform crates so it also implements
// #SPC-sel4platcrate

//! This crate provides the platform specifc parts of the sel4 library for the PC 99
//! platform.
//!
//! This crate will be empty if the target architecture is neither x86 nor x86_64.
//! A side effect of building this project (on the x86 or x86_64 architectures) 
//! is that the seL4 microkernal for the PC 99 platform will be built.

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate cty;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
