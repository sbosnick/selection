// Copyright 2018 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

extern crate sel4_sys;

// Implements #TST-sel4syscrate.compile
//
// This example code is really just a smoke test to ensure that the 
// build script is properly setting up linking to the sel4 native library.
// This program isn't really a useful example of anything. It is complied
// in a std environment (i.e. without #[no_std]) despite the fact that the 
// sel4 native library is not designed for such an environment. The resulting
// binary is unlikely to be able to run usefully in any environment.
//
// This smoke test was designed this way because the other crates necessary for
// a proper test haven't yet been written.
fn main() {
    unsafe {
        sel4_sys::seL4_DebugPutChar(72);
    }
}
