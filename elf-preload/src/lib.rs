// Copyright 2019 Steven Bosnick
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

//! Library to convert ELF files into a form that is suitable for loading with
//! [`std::ptr::copy_nonoverlapping`].
//!
//! In general an arbitrary ELF file cannot be loaded by copying its bytes to
//! its expected load address. This library transforms ELF files that complies with
//! certain constraints into ELF files that can be loaded by copying the entire
//! contents of the file to the expected load address.
//!
//! The main types in the library are [`Input`], [`Layout`], and [`OutputWriter`].
//! `Input` is the parsed bytes of the input ELF file that have also passed some
//! tests to ensure the input ELF file complies with the constraints. `Layout` is
//! (as the name suggest) the layout of the output ELF file without having copied
//! the bytes into that file. `OutputWriter` is a type that can write (part or
//! all of) the output into the output file. It can also be split in two and
//! each the resulting OutputWriter's can then write its own part of the output.
//!
//! The rational for the split is that parsing and processing `Input` and writing
//! the output with a collection of `OutputWriter` are potentially parallelizable.
//! `Layout`, on the otherhand, is inherently serialized.

// Note: The idea for the potentially parallelizable Input and OutputWriter with
// an inherently serialized Layout in the middle is from Ian Lance Taylor's 20
// part blog post on linkers, and specifically from https://www.airs.com/blog/archives/47

#![deny(missing_docs, unsafe_code)]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod arch;
mod error;
mod input;
mod layout;
mod output;

pub use arch::Arch;
pub use error::{BadElfError, Error, Result};
pub use input::Input;
pub use layout::{Layout, LayoutStrategy};
pub use output::OutputWriter;

const PAGE_SIZE: usize = 4096;
