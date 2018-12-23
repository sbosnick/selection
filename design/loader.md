# REQ-bootloader
partof: REQ-finalbinary
###
The bootloader will be a binary crate provided by the user that relies on library
crates provided by **selection**. The bootloader will accept a handoff from 
*U-Boot*, then (through the support libraries) configure the initial VSpace
for the seL4 microkernel and handoff to the microkernel in a way that identifies
the rootserver.


# REQ-sel4loadercrate
partof: REQ-bootloader
###
**selection** shall provide a library crate called 'seL4-loader' which will use 
information from the *U-Boot* handoff to locate the seL4 microkernel and the 
rootserver in memory, set up the initial VSpace for booting the microkernel,
enable the MMU, then handoff to the microkernel passing it information about
the location of the rootserver.


# REQ-ubootrtcrate
partof: REQ-bootloader
###
**selection** shall provide a library crate called 'uboot-rt' that provides a
[no-std] Rust runtime and a way of exposing the entry point required for a *U-Boot*
handoff using the 'bootm' command. The provided entry point shall be suitable for
an "os = linux, type = kernal" kernal in a *U-Boot* FIT image.