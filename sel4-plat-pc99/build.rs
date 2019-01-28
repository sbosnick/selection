// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use sel4_build::{CMakeTarget, Platform};

fn main() {
    let target = CMakeTarget::Kernel(Platform::Pc99);

    // build the kernel for the PC 99 platform
    target.build();

    // generate the rust bindings to the platform specifc parts of the sel4 library
    target.bindgen();
}
