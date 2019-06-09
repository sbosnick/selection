// Copyright 2019 Steven Bosnick
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// The tests in this file and the other integratin tests implement #TST-elfpreload.

use elf_preload::{Input, LayoutStrategy};
use goblin::elf::{program_header, Elf};

static SMOKETEST_ELF: &'static [u8] = include_bytes!("../test_data/smoketest");
static KERNEL_ELF: &'static [u8] = include_bytes!("../test_data/kernel.elf");

#[test]
fn elf_preload_is_idempotent_for_specificed_start() {
    idempotent_test(SMOKETEST_ELF, LayoutStrategy::SpecifiedStart(5000));
}

#[test]
fn elf_preload_is_idempotent_for_from_input() {
    idempotent_test(KERNEL_ELF, LayoutStrategy::FromInput);
}

// Implements #TST-elfpreload.idempotent
fn idempotent_test(input: &[u8], strategy: LayoutStrategy) {
    let output = run_preload(input, strategy);

    let elf = Elf::parse(&output).expect("Output file invalid");

    let mut offset = 0;
    for phdr in (&elf.program_headers)
        .iter()
        .filter(|p| p.p_type == program_header::PT_LOAD)
    {
        assert_eq!(phdr.p_offset, offset);
        offset += phdr.p_filesz;
    }
    assert_eq!(output.len(), offset as usize);
}

#[test]
fn elf_preload_has_expected_segments_for_specified_start() {
    expected_segments_test(SMOKETEST_ELF, LayoutStrategy::SpecifiedStart(5000));
}

#[test]
fn elf_preload_has_expected_segment_for_from_input() {
    expected_segments_test(KERNEL_ELF, LayoutStrategy::FromInput);
}

// Implements #TST-elfpreload.segments
fn expected_segments_test(input: &[u8], strategy: LayoutStrategy) {
    let output = run_preload(input, strategy);

    let elf = Elf::parse(&output).expect("Output file invalid");

    assert!(elf.program_headers.len() >= 2);
    assert_eq!(elf.program_headers[0].p_type, program_header::PT_PHDR);
    for phdr in (&elf.program_headers).iter().skip(1) {
        assert_eq!(phdr.p_type, program_header::PT_LOAD);
    }
}

#[test]
fn elf_preload_has_no_sections_for_specified_start() {
    no_sections_test(SMOKETEST_ELF, LayoutStrategy::SpecifiedStart(5000));
}

#[test]
fn elf_preload_has_no_sections_for_from_input() {
    no_sections_test(KERNEL_ELF, LayoutStrategy::FromInput);
}

// Implements #TST-elfpreload.nosections
fn no_sections_test(input: &[u8], strategy: LayoutStrategy) {
    let output = run_preload(input, strategy);

    let elf = Elf::parse(&output).expect("Output file invalid");

    assert_eq!(elf.section_headers.len(), 0);
}

fn run_preload(input: &[u8], strategy: LayoutStrategy) -> Vec<u8> {
    let input = Input::new(input).expect("Unable to read input file");
    let layout = input
        .layout(strategy)
        .expect("Unable to layout output file");
    let mut output = vec![0xd0; layout.required_size()];
    let mut writer = layout.output(&mut output).expect("Unable to create writer");
    writer.write().expect("Unable to write output file");

    output
}
