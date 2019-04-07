// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use failure::*;

/// The error type for the different stages of preloading an elf file.
#[derive(Fail, Debug)]
pub enum Error {
    /// A dummy error.
    #[fail(display = "A dummy error occured")]
    Dummy,
}

/// A specilized Result type for elf preloading operations.
pub type Result<T> = std::result::Result<T, Error>;
