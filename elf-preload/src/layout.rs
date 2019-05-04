// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use goblin::elf::{program_header, Header, ProgramHeader};
use goblin::container::Ctx;
use crate::{Arch, OutputWriter, PAGE_SIZE, Result};

/// The strategy to use for picking the starting physical address for the segments
/// of the output file.
#[derive(Debug, PartialEq)]
pub enum StartPAddr {
    /// The starting physical address is selected so as to maintain the physical
    /// addresses of the existing segments from the input file.
    FromInput,

    /// The starting physical address is set to the specified physical address.
    Specified(u64),
}

impl StartPAddr {
    fn layout<'a, I>(&self, input: I, ctx: Ctx) -> Vec<ProgramHeader> 
    where
        I: ExactSizeIterator<Item = &'a ProgramHeader>,
    {
        use StartPAddr::{FromInput, Specified};
        let count = input.len() + 2;
        let mut phdrs = Vec::with_capacity(count);

        match self {
            Specified(start) => {
                phdrs.push(create_phdr_header(*start, count, ctx));
                phdrs.push(create_first_load_header(*start, count, ctx));

                let mut min_vaddr = u64::max_value();
                let mut offset = first_load_header_size(count, ctx);
                let mut paddr = start + offset;
                for phdr in input {
                    align_offset_and_paddr(&mut offset, &mut paddr, phdr.p_vaddr, phdr.p_align);
                    extend_load_header_size(phdrs.last_mut(), offset);
                    phdrs.push(create_subsequent_load_header(offset, paddr, phdr));
                    offset += phdr.p_memsz;
                    paddr += phdr.p_memsz;
                    min_vaddr = min_vaddr.min(phdr.p_vaddr);
                }
                
                adjust_phdr_header(&mut phdrs[0], min_vaddr, count, ctx);
                adjust_first_load_header(&mut phdrs[1], min_vaddr, count, ctx);
            }
            FromInput => unimplemented!(),
        }

        phdrs
    }
}

fn create_phdr_header(start_paddr: u64, count: usize, ctx: Ctx) -> ProgramHeader {
    let size = program_header_size(count, ctx);
    let header_size = Header::size(&ctx) as u64;

    let mut phdr = ProgramHeader {
        p_type: program_header::PT_PHDR,
        p_offset: header_size,
        p_paddr: start_paddr + header_size,
        p_filesz: size,
        p_memsz: size,
        p_align: PAGE_SIZE as u64,
        ..ProgramHeader::new()
    };

    phdr.read();

    phdr
}

fn adjust_phdr_header(phdr: &mut ProgramHeader, lowest_vaddr: u64, count: usize, ctx: Ctx) {
    let vaddr = align_down(lowest_vaddr - program_header_size(count, ctx), phdr.p_offset, phdr.p_align);

    debug_assert!(phdr.p_type == program_header::PT_PHDR);
    debug_assert!(phdr.p_offset % phdr.p_align == vaddr % phdr.p_align);

    phdr.p_vaddr = vaddr;
}

fn create_first_load_header(start_paddr: u64, count: usize, ctx: Ctx) -> ProgramHeader {
    let size = first_load_header_size(count, ctx);

    let mut load = ProgramHeader {
        p_offset: 0,
        p_paddr: start_paddr,
        p_filesz: size,
        p_memsz: size,
        p_align: PAGE_SIZE as u64,
        ..ProgramHeader::new()
    };

    load.read();

    load
}

fn adjust_first_load_header(load: &mut ProgramHeader, lowest_vaddr: u64, count: usize, ctx: Ctx) {
    let vaddr = align_down(lowest_vaddr - first_load_header_size(count, ctx), load.p_offset, load.p_align);

    debug_assert!(load.p_type == program_header::PT_LOAD);
    debug_assert!(load.p_offset % load.p_align == vaddr % load.p_align);

    load.p_vaddr = vaddr;
}

fn create_subsequent_load_header(offset: u64, paddr: u64, input: &ProgramHeader) -> ProgramHeader {
    assert!(offset % input.p_align == input.p_vaddr % input.p_align);

    ProgramHeader {
        p_type: program_header::PT_LOAD,
        p_offset: offset,
        p_paddr: paddr,
        p_vaddr: input.p_vaddr,
        p_filesz: input.p_memsz,  // not a typo
        p_memsz: input.p_memsz,
        p_flags: input.p_flags,
        p_align: input.p_align,
    }
}

fn extend_load_header_size(load: Option<&mut ProgramHeader>, next_offset: u64) {
    load.map(|ph| {
        ph.p_filesz = next_offset - ph.p_offset;
        ph.p_memsz = next_offset - ph.p_offset;
    });
}

fn align_offset_and_paddr(offset: &mut u64, paddr: &mut u64, vaddr: u64, align: u64) {
    let aoffset = *offset % align;
    let avaddr = vaddr % align;

    let adjustment = if aoffset > avaddr {
        (avaddr + align) - aoffset
    } else {
        avaddr - aoffset
    };

    *offset += adjustment;
    *paddr += adjustment;
}

fn align_down(input: u64, reference: u64, align: u64) -> u64 {
    input - ((input.max(reference) - input.min(reference)) % align)
}

fn first_load_header_size(count: usize, ctx: Ctx) -> u64 {
    Header::size(&ctx) as u64 + program_header_size(count, ctx)
}

fn program_header_size(count: usize, ctx: Ctx) -> u64 {
    ((count + 2) * ProgramHeader::size(&ctx)) as u64
}

/// The layout of the output file. Created by the [`layout`][Input::layout] method.
#[derive(Debug)]
pub struct Layout<'a> {
    in_phdr: Vec<ProgramHeader>,
    out_phdr: Vec<ProgramHeader>,
    arch: Arch,
    input: &'a [u8],
}

impl<'a> Layout<'a> {
    pub (crate) fn new<'b, I>(arch: Arch, phdr: I, input: &'a [u8], start: StartPAddr) -> Self
    where
        I: ExactSizeIterator<Item = &'b ProgramHeader>
    {
        let in_phdr: Vec<_> = phdr.cloned().collect();
        let out_phdr = start.layout(in_phdr.iter(), arch.ctx());

        Layout { in_phdr, out_phdr, arch, input }
    }

    /// The required size of the output represented by this layout.
    pub fn required_size(&self) -> usize {
        self.out_phdr.iter()
            .filter(|ph| ph.p_type == program_header::PT_LOAD)
            .map(|ph| ph.p_filesz as usize)
            .sum()
    }

    /// Prepare to write to the given output bytes, which must be at least
    /// [`required_size`][Layout::required_size] in length.
    pub fn output<'b>(&'a self, _output: &'b mut [u8]) -> Result<OutputWriter<'a,'b>> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arch::test::create_arch;
    use goblin::container::{Container, Endian};

    #[test]
    fn specified_layout_required_size_is_header_and_loadables() {
        let arch = create_arch(Container::Little, Endian::Little);
        let memsz = 100;
        let offset = 100;
        let phdr = vec![ProgramHeader{ 
                            p_memsz: memsz, 
                            p_align: PAGE_SIZE as u64,
                            p_vaddr: offset + 4*PAGE_SIZE as u64,
                            p_offset: offset ,
                            ..ProgramHeader::new() },
        ];

        let sut = Layout::new(arch, phdr.iter(), &[], StartPAddr::Specified(0));
        let size = sut.required_size();

        assert_eq!(size, PAGE_SIZE + (offset as usize) + (memsz as usize));
    }
}
