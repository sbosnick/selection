// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{Arch, Error, OutputWriter, Result};
use goblin::elf::{program_header, ProgramHeader};

mod strategy;

pub use strategy::LayoutStrategy;

/// The layout of the output file. Created by the [`layout`][Input::layout] method.
#[derive(Debug)]
pub struct Layout<'a> {
    in_phdr: Vec<ProgramHeader>,
    out_phdr: Vec<ProgramHeader>,
    arch: Arch,
    input: &'a [u8],
}

impl<'a> Layout<'a> {
    pub(crate) fn new<'b, I>(arch: Arch, phdr: I, input: &'a [u8], start: LayoutStrategy) -> Self
    where
        I: ExactSizeIterator<Item = &'b ProgramHeader>,
    {
        let in_phdr: Vec<_> = phdr.cloned().collect();
        let out_phdr = start.layout(in_phdr.iter(), arch.ctx());

        Layout {
            in_phdr,
            out_phdr,
            arch,
            input,
        }
    }

    pub(crate) fn out_segments(&self) -> usize {
        self.out_phdr.len() - 1
    }

    pub(crate) fn segment_size(&self, _segment: usize) -> usize {
        unimplemented!()
    }

    pub(crate) fn write_segment<'b>(&self, _segment: usize, _output: &'b mut[u8]) {
        unimplemented!()
    }

    /// The required size of the output represented by this layout.
    pub fn required_size(&self) -> usize {
        self.out_phdr
            .iter()
            .filter(|ph| ph.p_type == program_header::PT_LOAD)
            .map(|ph| ph.p_filesz as usize)
            .sum()
    }

    /// Prepare to write to the given output bytes, which must be at least
    /// [`required_size`][Layout::required_size] in length.
    pub fn output<'b>(&'a self, output: &'b mut [u8]) -> Result<OutputWriter<'a, 'b>> {
        if output.len() < self.required_size() {
            Err(Error::OutputTooSmall)
        } else {
            Ok(OutputWriter::new(self, output))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arch::test::create_arch;
    use crate::PAGE_SIZE;
    use goblin::container::{Container, Endian};

    #[test]
    fn specified_layout_required_size_is_header_and_loadables() {
        let arch = create_arch(Container::Little, Endian::Little);
        let memsz = 100;
        let offset = 100;
        let phdr = vec![ProgramHeader {
            p_memsz: memsz,
            p_align: PAGE_SIZE as u64,
            p_vaddr: offset + 4 * PAGE_SIZE as u64,
            p_offset: offset,
            ..ProgramHeader::new()
        }];

        let sut = Layout::new(arch, phdr.iter(), &[], LayoutStrategy::SpecifiedStart(0));
        let size = sut.required_size();

        assert_eq!(size, PAGE_SIZE + (offset as usize) + (memsz as usize));
    }
}
