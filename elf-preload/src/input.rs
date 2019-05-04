// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use itertools::Itertools;
use goblin::elf::{Elf, ProgramHeader, header, program_header};
use crate::{Arch, Layout, PAGE_SIZE, LayoutStrategy, Result, Error};

/// An input ELF file that satisfies the necessary constraints for direct loading.
///
/// The constraints that the input ELF file must satisfy are:
/// * it must be an executable ELF file (not a shared library)
/// * it must contain neither a dynamic array nor an interpreter reference
#[derive(Debug)]
pub struct Input<'a> {
    arch: Arch,
    phdr: Vec<ProgramHeader>,
    input: &'a [u8],
}

impl<'a> Input<'a> {
    /// Create a new `Input` from the given input bytes.
    ///
    /// # Errors
    /// `new()` can return the following errors:
    /// * `Error::BadElf`: `input` is not an ELF file
    /// * `Error::InvalidElf`: `input` does not satisfy the required constraints
    pub fn new(input: &'a [u8]) -> Result<Self> {
        let elf = Elf::parse(input)?;
        let arch = Arch::new(&elf.header)?;
        verify(&elf)?;

        Ok(Input{
            arch,
            phdr: sort_loadable_headers(elf.program_headers).collect(),
            input,
        })
    }

    /// Layout the output file using the given strategy for selecting the
    /// starting physical address.
    ///
    /// # Errors
    /// `layout()` can return the following errors:
    /// * `Error::InvalidElf`: `start` is `FromInput` and the input contains sparse
    ///     segments with large gaps between their physical addresses
    pub fn layout(&'a self, start: LayoutStrategy) -> Result<Layout<'a>> {
        if start == LayoutStrategy::FromInput {
            verify_dense_segments(self.phdr.iter())?;
            verify_first_segment_not_near_zero(self.phdr.iter())?;
        }

        Ok(Layout::new(self.arch, self.phdr.iter(), self.input, start))
    }
}

fn sort_loadable_headers(phdr: impl IntoIterator<Item = ProgramHeader>) -> impl Iterator<Item = ProgramHeader> {
    phdr.into_iter()
        .filter(|ph| ph.p_type == program_header::PT_LOAD)
        .sorted_by_key(|ph| (ph.p_paddr, ph.p_vaddr))
}

fn verify(elf: &Elf) -> Result<()> {
    use Error::InvalidElf;

    let message = if elf.header.e_type != header::ET_EXEC {
        Some("Elf file not an executable file.")
    } else if elf.dynamic.is_some() {
        Some("Elf file contains a dynamic array.")
    } else if elf.interpreter.is_some() {
        Some("Elf file contains an interpretor.")
    } else {
        None
    };

    message.map_or(Ok(()), |message| Err(InvalidElf{message: message.to_owned()}))
}

fn verify_dense_segments<'a>(phdr: impl Iterator<Item = &'a ProgramHeader>) -> Result<()> {
    let max_segment_gap = phdr.tuple_windows::<(_,_)>()
        .map(|(ph1, ph2)| ph2.p_paddr - (ph1.p_paddr + ph1.p_memsz))
        .max();

    match max_segment_gap {
        Some(max) if max as usize > PAGE_SIZE => {
            let message = "ELF file segments are sparse with large gaps in their physical layout";
            Err(Error::InvalidElf{message: message.to_owned()})
        }
        _ => Ok(())
    }
}

fn verify_first_segment_not_near_zero<'a>(phdr: impl Iterator<Item = &'a ProgramHeader>) -> Result<()> {
    let min_paddr = phdr.map(|ph| ph.p_paddr).min();

    match min_paddr {
        Some(min) if (min as usize) < PAGE_SIZE => {
            let message = "ELF file's first segment physical address does not leave room for headers";
            Err(Error::InvalidElf{message: message.to_owned()})
        }
        _ => Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::CString;
    use crate::Error;
    use goblin::container::Ctx;
    use goblin::elf::header::{self, Header};
    use goblin::elf::program_header::{self, ProgramHeader};
    use goblin::elf::r#dyn as dynamic;
    use dynamic::Dyn;
    use scroll::Pwrite;
    use scroll::ctx::SizeWith;

    #[test]
    fn new_input_on_bad_bytes_is_error() {
        let input = [ 0xba, 0xdd, 0x00, 0xd8, 0xde, 0xad, 0xbe, 0xef ];

        let result = Input::new(&input);

        assert_matches!(result, Err(Error::BadElf(_)));
    }

    #[test]
    fn new_input_with_shared_library_is_error() {
        let mut input = vec![0; 512];
        let ctx = get_ctx();
        let mut offset: usize = 0;
        input.gwrite(
            Header {
                e_type: header::ET_DYN,
                e_phoff: Header::size(&ctx) as u64,
                e_phnum: 1,
                ..Header::new(ctx)
            }, 
            &mut offset).unwrap();
        input.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_filesz: 13,
                p_memsz: 16,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        input.gwrite(CString::new("Hello World!").expect("Bad CString"), &mut offset).unwrap();

        let result = Input::new(&input);

        assert_matches!(result, Err(Error::InvalidElf{message: _}));
    }

    #[test]
    fn new_input_with_dynamic_array_is_error() {
        let hello = CString::new("Hello World!").expect("Bad CString");
        let hello_len = hello.as_bytes_with_nul().len();
        let mut input = vec![0; 512];
        let ctx = get_ctx();
        let mut offset: usize = 0;
        write_header(&ctx, &mut input, &mut offset, 2);
        input.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + 2*ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_filesz: 100,
                p_memsz: 100,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        input.gwrite_with(
            ProgramHeader {
                p_type: program_header::PT_DYNAMIC,
                p_offset: (Header::size(&ctx) + 2*ProgramHeader::size(&ctx)+hello_len) as u64,
                p_vaddr: 0x2000,
                p_filesz: 2*Dyn::size_with(&ctx) as u64,
                p_memsz: 2*Dyn::size_with(&ctx) as u64,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        input.gwrite(CString::new("Hello World!").expect("Bad CString"), &mut offset).unwrap();
        input.gwrite_with(
            Dyn {
                d_tag: dynamic::DT_INIT,
                d_val: 0x1000,
            },
            &mut offset, ctx).unwrap();
        input.gwrite_with(
            Dyn {
                d_tag: dynamic::DT_NULL,
                d_val: 0,
            },
            &mut offset, ctx).unwrap();

        let result = Input::new(&input);

        assert_matches!(result, Err(Error::InvalidElf{message: _}));
    }

    #[test]
    fn new_input_with_interpretor_is_error() {
        let hello = CString::new("Hello World!").expect("Bad CString");
        let hello_len = hello.as_bytes_with_nul().len();
        let mut input = vec![0; 512];
        let ctx = get_ctx();
        let mut offset: usize = 0;
        write_header(&ctx, &mut input, &mut offset, 2);
        input.gwrite_with(
            ProgramHeader {
                p_type: program_header::PT_INTERP,
                p_offset: (Header::size(&ctx) + 2*ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_filesz: hello_len as u64,
                p_memsz: hello_len as u64,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        input.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + 2*ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_filesz: 100,
                p_memsz: 100,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        input.gwrite(hello, &mut offset).unwrap();

        let result = Input::new(&input);

        assert_matches!(result, Err(Error::InvalidElf{message: _}));
    }

    #[test]
    fn input_layout_with_from_input_start_and_sparse_segements_is_error() {
        let hello = CString::new("Hello World!").expect("Bad CString");
        let hello_len = hello.as_bytes_with_nul().len();
        let mut buffer = vec![0; 4*PAGE_SIZE];
        let ctx = get_ctx();
        let mut offset: usize = 0;
        write_header(&ctx, &mut buffer, &mut offset, 2);
        buffer.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + 2*ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_paddr: 0x4000,
                p_filesz: hello_len as u64,
                p_memsz: 100,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        buffer.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + 2*ProgramHeader::size(&ctx) + hello_len) as u64,
                p_vaddr: 0x1000 + 2 * PAGE_SIZE as u64,
                p_paddr: 0x4000 + 2 * PAGE_SIZE as u64,
                p_filesz: hello_len as u64,
                p_memsz: 100,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        buffer.gwrite(hello.clone(), &mut offset).unwrap();
        buffer.gwrite(hello, &mut offset).unwrap();

        let input = Input::new(&buffer).expect("Invalid ELF file passed to Input::new()");
        let result = input.layout(LayoutStrategy::FromInput);

        assert_matches!(result, Err(Error::InvalidElf{message: _}));
    }

    #[test]
    fn input_layout_with_from_input_start_and_first_segment_near_zero_is_error() {
        let hello = CString::new("Hello World!").expect("Bad CString");
        let hello_len = hello.as_bytes_with_nul().len();
        let mut buffer = vec![0; 4*PAGE_SIZE];
        let ctx = get_ctx();
        let mut offset: usize = 0;
        write_header(&ctx, &mut buffer, &mut offset, 2);
        buffer.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_paddr: 0x0010,
                p_filesz: hello_len as u64,
                p_memsz: 100,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        buffer.gwrite(hello, &mut offset).unwrap();

        let input = Input::new(&buffer).expect("Invalid ELF file passed to Input::new()");
        let result = input.layout(LayoutStrategy::FromInput);

        assert_matches!(result, Err(Error::InvalidElf{message: _}));
    }

    fn get_ctx() -> Ctx {
        use goblin::container::{Ctx, Container, Endian};

        Ctx::new(Container::Little, Endian::Little)
    }

    fn write_header(ctx: &Ctx, input: &mut [u8], offset: &mut usize, phnum: u16) {
        input.gwrite(
            Header {
                e_type: header::ET_EXEC,
                e_phoff: Header::size(ctx) as u64,
                e_phnum: phnum, 
                ..Header::new(*ctx)
            },
            offset).unwrap();
    }

}
