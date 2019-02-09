# REQ-configure
partof: REQ-purpose
###
The configuration of the seL4 microkernel build shall be mapped into the *cargo*
build system in as natural a way as possible.

The seL4 microkernel configuration options can be devided into four categories:

1. KernelPlatform
2. Other Kernel\* options that affect the user/kernel ABI
3. Other Kernel\* options that don't affect the user/kernel ABI
4. LibSel4\* options

## KernelPlatform
The mechanism for setting the appropriate KernelPlatform value shall be exposed 
by having a separate crate for each supported platform and requiring the user to
depend on a platform crate. It may be possible to support depending on one platform
crate per architecture.

## Kernel\* ABI Options
Kernel\* ABI options (other than options with 'Dangerous' in their name) shall be set
based on the configuration that *cargo* makes available to build.rs scripts. This will
primarily be the environment variables CARGO_CFG_TARGET_ARCH and PROFILE. Dangerous 
Kernel\* ABI options shall be disabled.

## Kernel\* non-ABI Options
Kernel\* options that do not affect the user/kernel ABI (i.e. tweaks) shall be
grouped into meaningful "classes" which are then exposed as *cargo* features on
all of the platform crates.

## LibSel4\* options
These options may be set to whatever fixed values most facilitate building a Rust
interface to the libsel4 library.


# REQ-finalbinary
partof: REQ-purpose
###
The final binary produced by the *cargo* build shall be a single file that can be
booted by the *U-Boot* `bootm` command. It will be produced by a custom *cargo*
subcommand which in turn invokes the *U-Boot* mkimage command to create a FIT
format file.

The final FIT file shall contain three executables (which will be post-processed
ELF files) as follow:

- a boot loader [written in Rust]
- the seL4 microkernel [from a released version of the seL4 project]
- a root server [written in Rust]

It may be possible to extend the format of the final binary to include other
servers in addition to the root server, but this would require support from the
boot loader and the root server.


# REQ-purpose
The goal of this project is to provide a natural embedding of the building
of a system based around the seL4 microkernel into the Rust ecosystem. A project
using this embedding will be built using *cargo* and will be able to be bootstrapped
from *U-Boot*.

The formal proof that accompanies seL4 is superior to the additional assurances that
come with Rust's borrow checker, but it may have limits on its scaleability. **selection**
is an attept to combine the formally proven seL4 microkernel with the addition assurances
that Rust brings for the parts of the whole system that are not formally proven.

**selection** shall support the following architectures: x86_64, aarch32, and aarch64.
It will not initally support the ia32 (or i686) architecture, though such support may be
added later if an appropriate target spec json can be developed.


# REQ-testing
partof: REQ-purpose
###
**selection** shall expose a means of easily running test under QEUM or on 
actual hardware. The means of running tests shall be linked into the *cargo*
test subcommand, if possible.
