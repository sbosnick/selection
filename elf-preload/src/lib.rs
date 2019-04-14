// Copyright 2019 Steven Bosnick
//
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

#[deny(missing_docs, unsafe_code)]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod error;
mod input;

pub use input::Input;
pub use error::{BadElfError, Error, Result};

use std::marker::PhantomData;

/// The layout of the output file. Created by the [`layout`][Input::layout] method.
pub struct Layout<'a> {
    _phantom: PhantomData<&'a u8>,
}

/// A potentially parallelizable writer for the output file. Created by the
/// [`output`][Layout::output] method.
pub struct OutputWriter<'a, 'b> {
    _phantoma: PhantomData<&'a u8>,
    _phantomb: PhantomData<&'b u8>,
}

/// The strategy to use for picking the starting physical address for the segments
/// of the output file.
pub enum StartPAddr {
    /// The starting physical address is selected so as to maintain the physical
    /// addresses of the existing segments from the input file.
    FromInput,

    /// The starting physical address is set to the specified physical address.
    Specified(u64),
}

impl<'a> Layout<'a> {
    /// The required size of the output represented by this layout.
    pub fn required_size(&self) -> usize {
        unimplemented!()
    }

    /// Prepare to write to the given output bytes, which must be at least
    /// [`required_size`][Layout::required_size] in length.
    pub fn output<'b>(&'a self, _output: &'b mut [u8]) -> Result<OutputWriter<'a,'b>> {
        unimplemented!()
    }
}

impl<'a, 'b> OutputWriter<'a, 'b> {
    /// Potentially split this `OutputWriter` into two independent `OutputWriter`'s
    /// for separate parts of the output.
    ///
    /// The signature of this function is intended to make it usable as the 
    /// `splitter` argument to the [rayon][rayon] [split] function.
    ///
    /// [rayon]: https://crates.io/crates/rayon
    /// [split]: https://docs.rs/rayon/1.0.3/rayon/iter/fn.split.html
    pub fn split(self) -> (Self, Option<Self>) {
        unimplemented!()
    }

    /// Write the portion of the output represented by this `OutputWriter` to the
    /// corresponding sub-slice of the output bytes passed to the 
    /// [`output`][Layout::output] method.
    pub fn write(&self) -> Result<()> {
        unimplemented!()
    }
}
