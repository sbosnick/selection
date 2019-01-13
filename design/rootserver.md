# REQ-rootserver
partof: REQ-finalbinary
###
The rootserver will be a binary crate provided by the user that relies on library
crates provided by **selection**. The boot loader support libraries shall ensure
that the seL4 microkernel tranfers control to the rootserver after its initial booting.


# REQ-sel4crate
partof: REQ-rootserver
###
**selection** shall provide a library crate called 'seL4' that exposes a Rust
friendly interface to the kernel objects from the the seL4 microkernel. This
library will be the main connection from a server (including the rootserver)
running on a seL4 microkernel (and using this project).


# REQ-sel4rtcrate
partof: REQ-rootserver
###
**selection** shall provide a library crate called 'seL4-rt' that provides a
[no-std] Rust runtime and a way of exposing the entry point required by a seL4
rootserver. 'seL4-rt' shall provide the Rust types to expose the seL4 BootInfo
frame. 'sel4-rt' will not be appropriate for servers other than the rootserver.