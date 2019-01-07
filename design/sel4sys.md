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


# SPC-sel4syscrate
[[.cmake]]\:The 'seL4-sys' crate will use the [cmake] crate and a custom CMakeLists.txt file to
drive the compilation of libsel4. The custom CMakeLists.txt file is needed because
the CMakeLists.txt file at the root of the seL4 repository and in the libsel4 subdirectory
is are fragments that are intended to be embedded in a larger cmake based proejct. The
template for the custom CMakeLists.txt is the [base.cmake] file from the seL4_tools
project.

[[.bingen]]\:The crate will also use [bindgen] to generate Rust FFI code that calls into libsel4.
The uses of [bindgen] will blacklist any platform specific parts of libsel4.

The build.rs script for this crate will set cmake configuration options for building
libsel4 as described below.

- KernelArch: CARGO_CFG_TARGET_ARCH
- KernelArmSel4Arch: CARGO_CFG_TARGET_ARCH
- KernelX86Sel4Arch: CARGO_CFG_TARGET_ARCH
- KernelVerificationBuild: PROFILE
- KernelDebugBuild: PROFILE
- KernelPrinting: PROFILE
- HardwareDebugAPI: PROFILE
- KernelDangerousCodeInjection: off (default)
- KernelX86DangerousMSR: off (default)
- LibSel4FunctionAttributes: set to "public"

All other cmake configuration options will use their default values.

[cmake]: https://crates.io/crates/cmake
[base.cmake]: https://github.com/seL4/seL4_tools/blob/master/cmake-tool/base.cmake
[bindgen]: https://crates.io/crates/bindgen


# TST-sel4syscrate
The 'sel4-sys' crate shall have at least the following tests:

- [[.compile]]: a smoke test that a simple call from Rust code compiles
- [[.run]]: a smoke test that a simple call from Rust code runs in QEMU
- [[.platform]]: a test that compiling libsel4 for different platforms is binary
        identical

The last test verifies an implicit assumption of dividing out the platform specific
parts of libsel4 from the rest of that library.