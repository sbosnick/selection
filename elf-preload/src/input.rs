// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use goblin::elf::{Elf, header::{self, Header}, program_header::{self, ProgramHeader}};
use goblin::container::Ctx;
use crate::{Layout, StartPAddr, Result, Error};

/// An input ELF file that satisfies the necessary constraints for direct loading.
#[derive(Debug)]
pub struct Input<'a> {
    elf: Elf<'a>,
}

impl<'a> Input<'a> {
    /// Create a new `Input` from the given input bytes.
    pub fn new(input: &'a [u8]) -> Result<Self> {
        let elf = Elf::parse(input)?;
        verify(&elf)?;

        Ok(Input{elf})
    }

    /// The size of the output ELF file corresponding to this input ELF file.
    pub fn output_size(&self) -> usize {
        let ctx = get_ctx(&self.elf);

        // The ELF header and the PT_PHDR ProgramHeader
        let header_sz = Header::size(&ctx) + ProgramHeader::size(&ctx);

        println!("program headers: {:?}", self.elf.program_headers);

        // The ProgramHeader and the segment contents for each loadable segment
        let loadable_sz: usize = self.elf.program_headers.iter()
            .filter(|ph| ph.p_type == program_header::PT_LOAD)
            .map(|ph| ProgramHeader::size(&ctx) + ph.p_memsz as usize)
            .sum();

        header_sz + loadable_sz
    }

    /// Layout the output file using the given strategy for selecting the
    /// starting physical address.
    pub fn layout(&self, _start: StartPAddr) -> Result<Layout<'a>> {
        unimplemented!()
    }
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

fn get_ctx<'a>(elf: &Elf<'a>) -> Ctx {
    use goblin::container::{Container, Endian};

    let container = if elf.is_64 { Container::Big } else { Container::Little };
    let endian = if !elf.little_endian { Endian::Big } else { Endian::Little };

    Ctx::new(container, endian)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::CString;
    use crate::Error;
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
        input.gwrite(CString::new("Hello World!").expect("Bad CString"), &mut offset).unwrap();

        let result = Input::new(&input);

        assert_matches!(result, Err(Error::InvalidElf{message: _}));
    }

    #[test]
    fn input_with_one_loadable_segment_has_expected_output_size() {
        let memsz: usize = 200;
        let mut buffer = vec![0; 512];
        let ctx = get_ctx();
        let mut offset: usize = 0;
        write_header(&ctx, &mut buffer, &mut offset, 1);
        buffer.gwrite_with(
            ProgramHeader {
                p_offset: (Header::size(&ctx) + ProgramHeader::size(&ctx)) as u64,
                p_vaddr: 0x1000,
                p_filesz: 100,
                p_memsz: memsz as u64,
                ..ProgramHeader::new()
            },
            &mut offset, ctx).unwrap();
        buffer.gwrite(CString::new("Hello World!").expect("Bad CString"), &mut offset).unwrap();

        let input = Input::new(&buffer).expect("Invalid Elf passed ot Input::new()");

        assert_eq!(input.output_size(), Header::size(&ctx) + 2*ProgramHeader::size(&ctx) + memsz);
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
