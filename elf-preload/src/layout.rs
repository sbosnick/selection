// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{Arch, Error, OutputWriter, Result};
use goblin::elf::{header, program_header, Header, ProgramHeader};
use scroll::Pwrite;

mod strategy;

pub use strategy::LayoutStrategy;

/// The layout of the output file. Created by the [`layout`][Input::layout] method.
#[derive(Debug)]
pub struct Layout<'a> {
    in_phdr: Vec<ProgramHeader>,
    out_phdr: Vec<ProgramHeader>,
    arch: Arch,
    entry: u64,
    input: &'a [u8],
}

impl<'a> Layout<'a> {
    pub(crate) fn new<'b, I>(
        arch: Arch,
        phdr: I,
        input: &'a [u8],
        entry: u64,
        start: LayoutStrategy,
    ) -> Self
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
            entry,
        }
    }

    pub(crate) fn out_segments(&self) -> usize {
        LayoutStrategy::out_segments(&self.out_phdr)
    }

    pub(crate) fn segment_size(&self, segment: usize) -> usize {
        let real_segment = LayoutStrategy::out_index(segment);
        self.out_phdr[real_segment].file_range().len()
    }

    // #SPC-elfpreload.programheader
    pub(crate) fn write_segment<'b>(&self, segment: usize, output: &'b mut [u8]) -> Result<()> {
        if segment == 0 {
            let mut phoff = Header::size(&self.arch.ctx());
            let mut header: Header = self.arch.into();
            header.e_type = header::ET_EXEC;
            header.e_entry = self.entry;
            header.e_phoff = phoff as u64;
            header.e_phnum = self.out_phdr.len() as u16;
            output.pwrite(header, 0)?;
            for phdr in &self.out_phdr {
                output.gwrite_with(phdr.clone(), &mut phoff, self.arch.ctx())?;
            }
        } else {
            let in_seg = LayoutStrategy::in_index(segment);
            let out_seg = LayoutStrategy::out_index(segment);

            let in_range = self.in_phdr[in_seg].file_range();
            let out_range = self.out_phdr[out_seg].file_range();
            let initial_out_range = 0..in_range.len();
            let bss_out_range = in_range.len()..out_range.len();

            output[initial_out_range].copy_from_slice(&self.input[in_range]);
            for elt in &mut output[bss_out_range] {
                *elt = 0;
            }
        }

        Ok(())
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
    use goblin::elf::{
        program_header::{PT_LOAD, PT_PHDR},
        Elf,
    };
    use scroll::Pread;
    use std::ops::Range;

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
        let strategy = LayoutStrategy::SpecifiedStart(0);

        let sut = Layout::new(arch, phdr.iter(), &[], 0, strategy);
        let size = sut.required_size();

        assert_eq!(size, PAGE_SIZE + (offset as usize) + (memsz as usize));
    }

    #[test]
    fn layout_segment_size_reads_n_plus_1_out_phdr_filesz() {
        let filesz = 500;

        let sut = Layout {
            in_phdr: Vec::new(),
            out_phdr: vec![make_fake_pt_phdr(), make_load_header(0..filesz, None)],
            arch: create_arch(Container::Little, Endian::Little),
            input: &[],
            entry: 0,
        };
        let result = sut.segment_size(0);

        assert_eq!(result, filesz as usize);
    }

    #[test]
    fn layout_write_segment_copies_bytes_for_second_segment() {
        let in_char = 0xdb;
        let size = 355;
        let in_start = 300;
        let out_start = 501;
        let in_phdr = vec![make_load_header(in_start..(in_start + size), None)];
        let out_phdr = vec![
            make_fake_pt_phdr(),
            make_load_header(0..500, None),
            make_load_header(out_start..(out_start + size), None),
        ];
        let input = vec![in_char; 2000];
        let mut output = vec![0xc0; 5000];

        let sut = Layout {
            in_phdr,
            out_phdr,
            input: &input,
            arch: create_arch(Container::Little, Endian::Little),
            entry: 0,
        };
        sut.write_segment(1, &mut output)
            .expect("write_segement failed unexpectedly");

        for out_char in &output[0..size] {
            assert_eq!(*out_char, in_char);
        }
    }

    #[test]
    fn layout_write_segment_zeros_bss_for_second_segment() {
        let in_char = 0xdb;
        let in_size = 355;
        let in_start = 300;
        let out_start = 501;
        let out_size = in_size + 50;
        let in_phdr = vec![make_load_header(
            in_start..(in_start + in_size),
            Some(out_size as u64),
        )];
        let out_phdr = vec![
            make_fake_pt_phdr(),
            make_load_header(0..500, None),
            make_load_header(out_start..(out_start + out_size), None),
        ];
        let input = vec![in_char; 2000];
        let mut output = vec![0xc0; 5000];

        let sut = Layout {
            in_phdr,
            out_phdr,
            input: &input,
            arch: create_arch(Container::Little, Endian::Little),
            entry: 0,
        };
        sut.write_segment(1, &mut output)
            .expect("write_segement failed unexpectedly");

        for out_char in &output[in_size..out_size] {
            assert_eq!(*out_char, 0);
        }
    }

    #[test]
    fn layout_write_first_segement_writes_elf_header() {
        let arch = create_arch(Container::Little, Endian::Little);
        let size = Header::size(&arch.ctx()) + ProgramHeader::size(&arch.ctx());
        let out_phdr = vec![make_fake_pt_phdr(), make_load_header(0..size, None)];
        let mut output = vec![0xc0; 5000];

        let sut = Layout {
            out_phdr,
            arch,
            in_phdr: Vec::new(),
            input: &[],
            entry: 0,
        };
        sut.write_segment(0, &mut output)
            .expect("write_segement failed unexpectedly");

        let header = (&output).pread::<Header>(0).expect("Invalid Elf Header");
        let out_arch = Arch::new(&header).expect("Invalid Arch in Elf Header");
        assert_eq!(out_arch, arch);
        assert_eq!(header.e_type, header::ET_EXEC);
    }

    #[test]
    fn layout_write_first_segment_includes_entry_in_elf_header() {
        let arch = create_arch(Container::Little, Endian::Little);
        let entry = 0xd00dfeed;
        let size = Header::size(&arch.ctx()) + ProgramHeader::size(&arch.ctx());
        let out_phdr = vec![make_fake_pt_phdr(), make_load_header(0..size, None)];
        let mut output = vec![0xc0; 5000];

        let sut = Layout {
            out_phdr,
            arch,
            in_phdr: Vec::new(),
            input: &[],
            entry,
        };
        sut.write_segment(0, &mut output)
            .expect("write_segement failed unexpectedly");

        let header = (&output).pread::<Header>(0).expect("Invalid Elf Header");
        assert_eq!(header.e_entry, entry);
    }

    #[test]
    fn layout_write_first_segment_writes_phdrs() {
        let arch = create_arch(Container::Little, Endian::Little);
        let size = Header::size(&arch.ctx()) + ProgramHeader::size(&arch.ctx());
        let out_phdr = vec![make_fake_pt_phdr(), make_load_header(0..size, None)];
        let mut output = vec![0xc0; 5000];

        let sut = Layout {
            out_phdr,
            arch,
            in_phdr: Vec::new(),
            input: &[],
            entry: 0,
        };
        sut.write_segment(0, &mut output)
            .expect("write_segement failed unexpectedly");

        let elf = Elf::parse(&output).expect("Invalid Elf file");
        assert_eq!(elf.program_headers.len(), 2);
        assert_eq!(elf.program_headers[0].p_type, PT_PHDR);
        assert_eq!(elf.program_headers[1].p_type, PT_LOAD);
        assert_eq!(elf.program_headers[1].p_offset, 0);
        assert_eq!(elf.program_headers[1].p_filesz, size as u64);
    }

    fn make_fake_pt_phdr() -> ProgramHeader {
        ProgramHeader {
            p_type: program_header::PT_PHDR,
            ..ProgramHeader::new()
        }
    }

    fn make_load_header(file_range: Range<usize>, memsz: Option<u64>) -> ProgramHeader {
        let memsz = memsz.unwrap_or(file_range.len() as u64);
        ProgramHeader {
            p_offset: file_range.start as u64,
            p_filesz: file_range.len() as u64,
            p_memsz: memsz,
            ..ProgramHeader::new()
        }
    }
}
