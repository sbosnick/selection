// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::fmt::{Debug, Display, Formatter};

use failure::*;
use goblin::error::Error as GoblinError;

/// The error type for the different stages of preloading an elf file.
#[derive(Fail, Debug)]
pub enum Error {
    /// The input bytes are not a proper ELF file.
    #[fail(display = "The input bytes are not a proper ELF file.")]
    BadElf(#[cause] BadElfError),

    /// The input ELF file has failed a constraint validation
    #[fail(
        display = "The input ELF file does not satisfy a required constraint: {}",
        message
    )]
    InvalidElf { 
        /// The error message that describes the failed constraint.
        message: String,
    },

    /// The output bytes are too small for the layout of the output ELF file.
    #[fail(display = "The output bytes are too small for the layout of the output ELF file.")]
    OutputTooSmall,
}

#[doc(hidden)]
impl From<GoblinError> for Error {
    fn from(inner: GoblinError) -> Self {
        Error::BadElf(BadElfError(inner))
    }
}

/// A specilized Result type for elf preloading operations.
pub type Result<T> = std::result::Result<T, Error>;

/// A new type to wrap errors in parsing an ELF file.
#[derive(Fail)]
pub struct BadElfError(#[cause] GoblinError);

impl Display for BadElfError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error parsing the input bytes as an ELF file.")
    }
}

impl Debug for BadElfError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "goblin error parsing input bytes: {}", self.0)
    }
}
