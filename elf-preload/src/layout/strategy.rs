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
    /// Tranforms the input program headers into the output program hearders
    /// in accorance with the given LayoutStrategy.
    ///
    /// The input program headers should be sorted by p_paddr and then by
    /// p_vaddr.
    // #SPC-elfpreload.ptphdr
    // #SPC-elfpreload.ptload
    // #SPC-elfpreload.nobss
    // #SPC-elfpreload.paddr
    // #SPC-elfpreload.plenum
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
                    paddr += align_adjustment(offset, phdr.p_vaddr, phdr.p_align);
                    offset += align_adjustment(offset, phdr.p_vaddr, phdr.p_align);
                    extend_load_header_size(phdrs.last_mut(), offset);
                    phdrs.push(create_subsequent_load_header(offset, paddr, phdr));
                    offset += phdr.p_memsz;
                    paddr += phdr.p_memsz;
                    min_vaddr = min_vaddr.min(phdr.p_vaddr);
                }
                
                adjust_phdr_header(&mut phdrs[0], min_vaddr, count, ctx, None);
                adjust_first_load_header(&mut phdrs[1], min_vaddr, count, ctx, None);
            }

            FromInput => {
                phdrs.push(create_phdr_header(0, count, ctx));
                phdrs.push(create_first_load_header(0, count, ctx));

                let mut min_vaddr = u64::max_value();
                let mut min_paddr = u64::max_value();
                let mut offset = first_load_header_size(count, ctx);
                for phdr in input {
                    offset += align_adjustment(offset, phdr.p_vaddr, phdr.p_align);
                    extend_load_header_size(phdrs.last_mut(), offset);
                    phdrs.push(create_subsequent_load_header(offset, phdr.p_paddr, phdr));
                    offset += phdr.p_memsz;
                    min_vaddr = min_vaddr.min(phdr.p_vaddr);
                    min_paddr = min_paddr.min(phdr.p_paddr);
                }

                let paddr_adjust = min_paddr - phdrs[1].p_memsz;
                adjust_phdr_header(&mut phdrs[0], min_vaddr, count, ctx, Some(paddr_adjust));
                adjust_first_load_header(&mut phdrs[1], min_vaddr, count, ctx, Some(paddr_adjust));
            }
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

fn adjust_phdr_header(
    phdr: &mut ProgramHeader, 
    lowest_vaddr: u64, 
    count: usize, 
    ctx: Ctx, 
    paddr_adjust: Option<u64>,
) {
    let vaddr = align_down(lowest_vaddr - program_header_size(count, ctx), phdr.p_offset, phdr.p_align);

    debug_assert!(phdr.p_type == program_header::PT_PHDR);
    debug_assert!(phdr.p_offset % phdr.p_align == vaddr % phdr.p_align);

    phdr.p_vaddr = vaddr;
    match paddr_adjust {
        Some(adjust) => phdr.p_paddr += adjust,
        None => {}
    }
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

fn adjust_first_load_header(
    load: &mut ProgramHeader, 
    lowest_vaddr: u64,
    count: usize,
    ctx: Ctx,
    paddr_adjust: Option<u64>,
) {
    let vaddr = align_down(lowest_vaddr - first_load_header_size(count, ctx), load.p_offset, load.p_align);

    debug_assert!(load.p_type == program_header::PT_LOAD);
    debug_assert!(load.p_offset % load.p_align == vaddr % load.p_align);

    load.p_vaddr = vaddr;
    match paddr_adjust {
        Some(adjust) => load.p_paddr += adjust,
        None => {}
    }
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

fn align_adjustment(input: u64, reference: u64, align: u64) -> u64 {
    (input.max(reference) - input.min(reference)) % align
}

fn align_down(input: u64, reference: u64, align: u64) -> u64 {
    input - align_adjustment(input, reference, align)
}

fn first_load_header_size(count: usize, ctx: Ctx) -> u64 {
    Header::size(&ctx) as u64 + program_header_size(count, ctx)
}

fn program_header_size(count: usize, ctx: Ctx) -> u64 {
    ((count + 2) * ProgramHeader::size(&ctx)) as u64
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn specified_start_layout_without_room_gives_gt_page_size_first_segment() {
        let phdr = vec![make_phdr(25, 100)];

        let sut = LayoutStrategy::SpecifiedStart(100);
        let out = sut.layout(phdr.iter(), new_ctx());

        // out[1] is the first PT_LOAD segment
        assert!(out[1].p_filesz >= PAGE_SIZE as u64);
    }

    #[test]
    fn specified_start_layout_with_room_gives_lt_page_size_first_segment() {
        let phdr = vec![make_phdr(1000, 100)];

        let sut = LayoutStrategy::SpecifiedStart(100);
        let out = sut.layout(phdr.iter(), new_ctx());

        // out[1] is the first PT_LOAD segment
        assert!(out[1].p_filesz < PAGE_SIZE as u64);
    }

    #[test]
    fn specified_start_layout_gives_phdr_and_load_segments() {
        let phdr = vec![make_phdr(1000, 100), make_phdr(1200,50)];

        let sut = LayoutStrategy::SpecifiedStart(100);
        let out = sut.layout(phdr.iter(), new_ctx());

        assert_eq!(out.len(), 4);
        assert_eq!(out[0].p_type, program_header::PT_PHDR);
        assert_eq!(out[1].p_type, program_header::PT_LOAD);
        assert_eq!(out[2].p_type, program_header::PT_LOAD);
        assert_eq!(out[3].p_type, program_header::PT_LOAD);
    }

    #[test]
    fn specified_start_layout_gives_full_filesz_segments() {
        let phdr = vec![make_phdr(1000, 100), make_phdr(1200,50)];

        let sut = LayoutStrategy::SpecifiedStart(100);
        let out = sut.layout(phdr.iter(), new_ctx());

        for ph in out {
            assert_eq!(ph.p_memsz, ph.p_filesz);
        }
    }

    #[test]
    fn specified_start_layout_gives_sorted_segments() {
        let phdr = vec![make_phdr(1000, 100), make_phdr(1200,50)];

        let sut = LayoutStrategy::SpecifiedStart(100);
        let out = sut.layout(phdr.iter(), new_ctx());

        for (l,r) in out.iter().tuple_windows() {
            assert!(l.p_type != program_header::PT_LOAD || l.p_vaddr <= r.p_vaddr);
        }
    }

    #[test]
    fn specified_start_layout_gives_plenum() {
        let phdr = vec![make_phdr(1000, 100), make_phdr(1200,50)];

        let sut = LayoutStrategy::SpecifiedStart(100);
        let out = sut.layout(phdr.iter(), new_ctx());

        for (l,r) in out.iter().tuple_windows() {
            if l.p_type == program_header::PT_PHDR {
                // the PT_PHDR is within the first PT_LOAD segment
                assert!(l.p_paddr >= r.p_paddr);
                assert!(l.p_paddr + l.p_filesz <= r.p_paddr + r.p_filesz);
            } else {
                // the next PT_LOAD segment is exactly after the current one
                assert_eq!(l.p_paddr + l.p_filesz, r.p_paddr);
                assert_eq!(l.p_offset + l.p_filesz, r.p_offset);
            }
        }
    }

    #[test]
    fn specified_start_layout_gives_specified_start() {
        let phdr = vec![make_phdr(1000, 100), make_phdr(1200,50)];
        let start = 100;

        let sut = LayoutStrategy::SpecifiedStart(start);
        let out = sut.layout(phdr.iter(), new_ctx());

        assert_eq!(out[1].p_paddr, start);
    }

    #[test]
    fn from_input_layout_gives_phdr_and_load_segments() {
        let phdr = vec![make_paddr_phdr(100, 100, 5200), make_paddr_phdr(1200, 50, 5500)];

        let sut = LayoutStrategy::FromInput;
        let out = sut.layout(phdr.iter(), new_ctx());

        assert_eq!(out.len(), 4);
        assert_eq!(out[0].p_type, program_header::PT_PHDR);
        assert_eq!(out[1].p_type, program_header::PT_LOAD);
        assert_eq!(out[2].p_type, program_header::PT_LOAD);
        assert_eq!(out[3].p_type, program_header::PT_LOAD);
    }

    #[test]
    fn from_input_layout_gives_full_filesz_segments() {
        let phdr = vec![make_paddr_phdr(100, 100, 5200), make_paddr_phdr(1200, 50, 5500)];

        let sut = LayoutStrategy::FromInput;
        let out = sut.layout(phdr.iter(), new_ctx());

        for ph in out {
            assert_eq!(ph.p_memsz, ph.p_filesz);
        }
    }

    #[test]
    fn from_input_layout_gives_sorted_segments() {
        let phdr = vec![make_paddr_phdr(100, 100, 5200), make_paddr_phdr(1200, 50, 5500)];

        let sut = LayoutStrategy::FromInput;
        let out = sut.layout(phdr.iter(), new_ctx());

        for (l,r) in out.iter().tuple_windows() {
            assert!(l.p_type != program_header::PT_LOAD || l.p_vaddr <= r.p_vaddr);
        }
    }

    #[test]
    fn from_input_layout_gives_plenum() {
        let phdr = vec![make_paddr_phdr(100, 100, 5200), make_paddr_phdr(1200, 50, 5500)];

        let sut = LayoutStrategy::FromInput;
        let out = sut.layout(phdr.iter(), new_ctx());

        for (l,r) in out.iter().tuple_windows() {
            if l.p_type == program_header::PT_PHDR {
                // The PT_PHDR is within the first PT_LOAD segment.
                assert!(l.p_paddr >= r.p_paddr);
                assert!(l.p_paddr + l.p_filesz <= r.p_paddr + r.p_filesz);
            } else {
                // The next PT_LOAD segment is exactly after the current one.
                // This does not check the p_paddr for a plenum constraint since
                // this strategy does not assign the p_paddr.
                assert_eq!(l.p_offset + l.p_filesz, r.p_offset);
            }
        }
    }

    #[test]
    fn from_input_layout_gives_paddr_from_input() {
        let paddr1 = 5200;
        let paddr2 = 5500;
        let phdr = vec![make_paddr_phdr(100, 100, paddr1), make_paddr_phdr(1200, 50, paddr2)];

        let sut = LayoutStrategy::FromInput;
        let out = sut.layout(phdr.iter(), new_ctx());

        assert_eq!(out[2].p_paddr, paddr1);
        assert_eq!(out[3].p_paddr, paddr2);
    }

    fn new_ctx() -> Ctx {
        use goblin::container::{Container, Endian};

        Ctx::new(Container::Big, Endian::Big)
    }

    fn make_phdr(rel_offset: u64, memsz: u64) -> ProgramHeader {
        ProgramHeader {
            p_memsz: memsz,
            p_align: PAGE_SIZE as u64,
            p_vaddr: rel_offset + 4*PAGE_SIZE as u64,
            p_offset: rel_offset + 1*PAGE_SIZE as u64,
            ..ProgramHeader::new()
        }
    }

    fn make_paddr_phdr(rel_offset: u64, memsz: u64, paddr: u64) -> ProgramHeader {
        ProgramHeader {
            p_paddr: paddr,
            ..make_phdr(rel_offset, memsz)
        }
    }
}
