// Copyright 2018 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(not(target_os = "none"))]
fn main() {
    println!("The smoke test is not designed for a hosted environment.");
}

#[cfg(target_os = "none")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    freestanding::main();

    loop {}
}

#[cfg(target_os = "none")]
mod freestanding {
    extern crate sel4_sys;
    use core::panic::PanicInfo;

    // Implements #TST-sel4syscrate.compile
    pub fn main() {
        unsafe {
            sel4_sys::seL4_DebugPutChar(72);
        }
    }

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
}
