// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use goblin::elf::{program_header, Header, ProgramHeader};
use goblin::container::Ctx;
use crate::PAGE_SIZE;

/// The strategy to use for laying out the output ELF file.
///
/// The stategy is mainly concerned with assigning the physical addresses for
/// the segments of the output file.
#[derive(Debug, PartialEq)]
pub enum LayoutStrategy {
    /// The starting physical address is selected so as to maintain the physical
    /// addresses of the existing segments from the input file.
    FromInput,

    /// The starting physical address is set to the specified physical address and
    /// later the subsequent physical addresses are set to exactly follow the end
    /// of the previous segment.
    SpecifiedStart(u64),
}

impl LayoutStrategy {
    pub (super) fn layout<'a, I>(&self, input: I, ctx: Ctx) -> Vec<ProgramHeader> 
    where
        I: ExactSizeIterator<Item = &'a ProgramHeader>,
    {
        use LayoutStrategy::{FromInput, SpecifiedStart};
        let count = input.len() + 2;
        let mut phdrs = Vec::with_capacity(count);

        match self {
            SpecifiedStart(start) => {
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
