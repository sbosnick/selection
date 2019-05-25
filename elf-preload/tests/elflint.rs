// Copyright 2019 Steven Bosnick
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

// Implements #TST-elfpreload.elflint

use std::fs;
use std::path::Path;
use std::process::Command;
use elf_preload::{Input, LayoutStrategy};
use tempfile;

static SMOKETEST_ELF: &'static [u8] = include_bytes!("../test_data/smoketest");
static KERNEL_ELF: &'static [u8] = include_bytes!("../test_data/kernel.elf");

#[test]
fn elf_preload_for_specfied_start_passes_lint() {
    elf_lint_test(SMOKETEST_ELF, LayoutStrategy::SpecifiedStart(5000));
}

#[test]
fn elf_preload_for_from_input_passes_lint() {
    elf_lint_test(KERNEL_ELF, LayoutStrategy::FromInput);
}

fn elf_lint_test(input: &[u8], strategy: LayoutStrategy) {
    let out_dir = tempfile::tempdir().expect("Couldn't create temp dir.");
    let out_file = out_dir.path().join("output.elf");

    let output = run_preload(input, strategy);
    fs::write(&out_file, output).expect("Unable to write output file");

    run_elflint(&out_file);
}

fn run_preload(input: &[u8], strategy: LayoutStrategy) -> Vec<u8> {
    let input = Input::new(input).expect("Unable to read input file");
    let layout = input.layout(strategy)
        .expect("Unable to layout output file");
    let mut output = vec![0xd0; layout.required_size()];
    let mut writer = layout.output(&mut output)
        .expect("Unable to create writer");
    writer.write().expect("Unable to write output file");

    output
}

fn run_elflint(filename: &Path) {
    let status = Command::new("eu-elflint")
        .arg("--strict")
        .arg("--quiet")
        .arg(filename)
        .status()
        .expect("Unable to run eu-elflint");

    assert!(status.success(), "eu-elflint did not exit sucessfully");
}
