// Copyright 2019 Steven Bosnick
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use elf_preload::{Input, LayoutStrategy};
use goblin::elf::Elf;

static SMOKETEST_ELF: &'static [u8] = include_bytes!("../test_data/smoketest");
static KERNEL_ELF: &'static [u8] = include_bytes!("../test_data/kernel.elf");

#[test]
fn elf_preload_writes_valid_elf_for_specified_start() {
    smoke_test(SMOKETEST_ELF, LayoutStrategy::SpecifiedStart(5000));
}

#[test]
fn elf_preload_writes_valid_elf_for_from_input() {
    smoke_test(KERNEL_ELF, LayoutStrategy::FromInput);
}

fn smoke_test(input: &[u8], strategy: LayoutStrategy) {
    let input = Input::new(input).expect("Unable to read input file");
    let layout = input
        .layout(strategy)
        .expect("Unable to layout output file");
    let mut output = vec![0xd0; layout.required_size()];
    let mut writer = layout.output(&mut output).expect("Unable to create writer");
    writer.write().expect("Unable to write output file");

    Elf::parse(&output).expect("Output file is an invalid Elf file!");
}
