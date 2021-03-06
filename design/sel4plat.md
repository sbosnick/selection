# REQ-sel4platcrate
partof:
- REQ-configure
- REQ-microkernel
- REQ-rootserver
###
**selection** shall provide a collection of library crates, one for each platform 
supported by both seL4 and Rust. These platform crates shall expose any platform
specific parts of libsel4 for a given platform.

The platform crates shall also compile the seL4 microkernel binary itself as a side
effect of being built.

The platform crates shall expose *cargo* features for "classes" of seL4 Kernel\*
non-ABI options.


# SPC-sel4platcrate
[[.buildcrate]]\:The use of [cmake] and [bindgen] from the sel4-sys crate's build.rs
shall be extracted to a sel4-build crate that will contain the common build.rs logic
for sel4-sys and for all of the sel4-plat-\* crates. The custom CMakeLists.txt file 
from sel4-sys may be moved to sel4-build, if possible, to avoid having to copy it to
each platform crate.

There shall be the following platform crates:

- [[.pc99]]\: x86_64
- [[.sabre]]\: aarch32
- [[.omap3]]\: aarch32 (release only)
- [[.am335x]]\: aarch32 (release only)
- [[.exynos4]]\: aarch32
- [[.exynos5410]]\: aarch32
- [[.exynos5422]]\: aarch32
- [[.exynos5250]]\: aarch32
- [[.apq8064]]\: aarch32
- [[.wandq]]\: aarch32
- [[.imx7sabre]]\: aarch32
- [[.zynq7000]]\: aarch32
- [[.zynqmp]]\: aarch32
- [[.ultra96]]\: aarch32
- [[.allwinnera20]]\: aarch32
- [[.tk1]]\: aarch32
- [[.hikey]]\: aarch32 and aarch64
- [[.rpi3]]\: aarch32 and aarch64
- [[.tx1]]\: aarch64
- [[.tx2]]\: aarch64

The platform crates shall build the seL4 microkernal when they are being built for
the architecture listed, but is shall not be an error if the platform crate is being
built for a different architecture. For all of the platform crates the build.rs 
script shall use [bindgen] to generate the platform specific parts of libsel4.a. 
(This may be included as part of sel4-build.)

The omap3 and am335x platforms will initially be restricted to supporting the release
profile in cargo (i.e. when complied with the --release flag) because of [issue 116].
Once that issue is resolved **selection** shall [[.support_debug_on_omap]] and
[[.support_debug_on_am335x]].

[[.class]]\:The sel4-build crate shall expose two classes: Default, and Verified.
Addtional classes may be added in the future. The Default class will use the 
default values for all non-ABI Kernel\* options. The Verified class will use 
the values from the .cmake files in the configs subdirectory of the seL4 project 
root for the non-default values of non-ABI Kernel\* options. The Verified class
shall be exposed in each platform crate with a "verified-class" feature.

**selection** will not support the kzm platform for two reasons. First, it is the
only ARMv6A platform supported by seL4 and would require its own target specification
file for the rust compiler. Second the [sel4-kzm] supported hardware page indicates
that "The KZM is depreciated...".

[cmake]: https://crates.io/crates/cmake
[bindgen]: https://crates.io/crates/bindgen
[sel4-kzm]: https://docs.sel4.systems/Hardware/Kzm.html
[issue 116]: https://github.com/seL4/seL4/issues/116


# TST-sel4platcrate
The sel4-plat-pc99 crate shall have a smoke test that checks for a kernel.elf 
file in the expected location. This one smoke test will be a proxie for all of
the platform crates.