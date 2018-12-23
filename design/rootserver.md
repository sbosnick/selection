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


# REQ-sel4platcrate
partof:
- REQ-configure
- REQ-rootserver
- REQ-microkernel
###
**selection** shall provide a collection of library crates, one for each platform 
supported by both seL4 and Rust. These platform crates shall expose any platform
specific parts of libsel4 for a given platform.

The platform crates shall also compile the seL4 microkernel binary itself as a side
effect of being built.

The platform crates shall expose *cargo* features for "profiles" of seL4 Kernel\*
non-ABI options.


# REQ-sel4rtcrate
partof: REQ-rootserver
###
**selection** shall provide a library crate called 'seL4-rt' that provides a
[no-std] Rust runtime and a way of exposing the entry point required by a seL4
rootserver. 'seL4-rt' shall provide the Rust types to expose the seL4 BootInfo
frame. 'sel4-rt' will not be appropriate for servers other than the rootserver.


# REQ-sel4syscrate
partof:
- REQ-configure
- REQ-rootserver
###
**selection** shall provide a library crate called 'seL4-sys' that is generated
from the libsel4 subdirectory of the seL4 source code. This crate shall exclude
any platform specific parts of libsel4.

'seL4-sys' shall expose the seL4 Kernel\* ABI options (other than KernelPlatform)
through the use of the configuration the *cargo* makes available to build.rs scripts.
