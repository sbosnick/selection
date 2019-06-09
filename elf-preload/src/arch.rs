// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use crate::Error;
use goblin::container::Ctx;
use goblin::elf::header;
use std::fmt;

/// The architecture for an ELF file.
///
/// The architecture is a combination of the endianness, the bit-size (64 vs.
/// 32 bit), and the machine of the ELF file. This information is all drawn from
/// the ELF header in the file.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Arch {
    machine: u16,
    ctx: Ctx,
}

impl Arch {
    pub(crate) fn new(header: &header::Header) -> Result<Self, Error> {
        let container = header.container()?;
        let endian = header.endianness()?;

        Ok(Arch {
            machine: header.e_machine,
            ctx: Ctx::new(container, endian),
        })
    }

    pub(crate) fn ctx(&self) -> Ctx {
        self.ctx
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let size = if self.ctx.is_big() {
            "64 bit"
        } else {
            "32 bit"
        };
        let endian = if self.ctx.is_little_endian() {
            "little endian"
        } else {
            "big endian"
        };
        let machine = header::machine_to_str(self.machine);

        write!(f, "{}, {}, {}", size, endian, machine)
    }
}

impl From<Arch> for header::Header {
    fn from(arch: Arch) -> Self {
        let mut header = Self::new(arch.ctx);
        header.e_machine = arch.machine;
        header
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use goblin::container::{Container, Endian};
    use goblin::elf::header::EM_ARM;

    pub(crate) fn create_arch(c: Container, e: Endian) -> Arch {
        Arch {
            machine: EM_ARM,
            ctx: Ctx::new(c, e),
        }
    }
}
