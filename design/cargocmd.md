# REQ-cargocmd
partof: REQ-finalbinary
###
**selection** shall have a cargo subcommand called "cargo-fit" which can be
invoked as "cargo fit ...". "cargo-fit" shall be a CLI application which will

- locate a bootloader crate, the seL4 microkernel, and a root server crate in
  the current workspace,
- post-process the three ELF files into the expected format, and
- combined the post-processed ELF files into a u-boot FIT format file


# SPC-cargocmd
"cargo-fit" shall use [structopt], [exitfailure], [human-panic], and [failure]
to implement a CLI for locating, post-processing, and combining the three ELF
files that make up the final binary.

It shall use the [cargo-metadata] crate to locate three ELF files assocated
with the current *cargo* workspace:

- the output of a user-supplied binary crate which is the bootloader,
- the seL4 microkernel build as a side-effect of building one of the
  sel4-plat-\* crates, and
- the output of a user-supplied binary crate which is the root server.

The bootloader and the root server shall be identified using convention over
configuration, but will have a fall-back of configuration in the workspace
root Cargo.toml file (in a "package.metadata" table). The seL4 microkernel
shall be identified by looking for a "kernel.elf" file in the "OUT\_DIR" of
a sel4-plat-\* crate that is a dependency of the root server.

"cargo-fit" shall use [[SPC-elfpreload]] to post-process the three identified
ELF binaries (bootloader, seL4 microkernel, and the root server). The paddr
for the seL4 microkernel shall be specified to follow the paddr from the input
file. The paddr for the bootloader shall be specified to an address that places 
the whole bootloader before the microkernel. The paddr for the root server
shall be specified to an address that places the root server immediately after
the microkernel in physical memory.

"cargo-fit" shall use [[SPC-fitimage]] to combine the three post-processed ELF
files into the final binary file. It shall use the description from the package
metadata for the root server pacakage as the description  for the FIT file. It
may provide a means of configuring an FDT file to use in the FIT file (possibly
in a "package.metadata" table).

[cargo-metadata]: https://crates.io/crates/cargo_metadata
[structopt]: https://crates.io/crates/structopt 
[exitfailure]: https://crates.io/crates/exitfailure
[human-panic]: https://crates.io/crates/human-panic
[failure]: https://crates.io/crates/failure


# TST-cargocmd
"cargo-fit" shall have at least the following tests

- [[.nobootloader]]: a workspace without a bootloader crate is an error
- [[.nokernel]]: a workspace that does not build the seL4 microkernel through
    a sel4-plat-\* crate is an error
- [[.manykernels]]: a workspace that builds more than 1 seL4 microkernel is an error
- [[.norootserver]]: a workspace without a rootserver is an error
- [[.sucess]]: a workspace with a bootloader, a rootserver, and exactly one
    seL4 microkernel shall sucessfully produce the expected output file


# SPC-elfpreload
partof: SPC-cargocmd
###
**selection** shall have a library crate called "elf-preload" for post-processing
ELF binary files into a memory image that is ready to be memcpy()'d to a particular
address in physical memory. "elf-preload" shall use [goblin] for its ELF manipulation.

The output of the post-processing shall also be an ELF file, but it shall have the
following constraints:

- [[.programheader]]: it shall have program headers but no section headers;
- [[.ptphdr]]: it shall have a PT_PHDR program header;
- [[.ptload]]: other than the PT_PHDR, all other program headers shall be PT_LOAD;
- [[.nobss]]: the filesz and memsz in each program header shall be equal
- [[.paddr]]: the paddr in each program header shall be set as described below
- [[.plenum]]: all parts of the ELF file will be described in the program headers

These constraints imply that the post-processed ELF file will not have any PT_DYNAMIC
program headers which means that there will not be a dynamic segment. This in turn
means that the file will have to be absolutely positioned(with no relocations). The
lack of a dynamic segment and section headers means that there is no need for string
tables or a symbol table. The plenum constraint requires that the ELF header and the
program headers be included in the first PT_LOAD program header.

The paddr for each PT_LOAD program header shall immediately follow the end of 
the segment identified by the previous program header (modulo padding). The 
paddr for the first PT_LOAD program header shall either be specified in the library call 
or shall be specified to follow the paddr from the input binary. It shall be an 
error to specify using paddr from an input binary that lacks appropriate paddr values.

Any input ELF file that cannot be post-processed to comply with the constraints shall
cause an error.

The layout of the output file will be the following:

```
[elf header]
[PT_PHDR]
[PT_LOAD: elf header and all program headers]
[PT_LOAD: segment 1]
[PT_LOAD: segment 2]
[PT_LOAD: segment 3]
[segment 1 contents]
[segment 2 contents]
[segment 3 contents]
```

[goblin]: https://crates.io/crates/goblin


# SPC-fitimage
partof: SPC-cargocmd
###
**selection** shall have a library crate called "fitimage" for making a u-boot
FIT image from a collection of post-processed ELF files (one of which is the
bootloader) and, optionally, an FDT file.

## FDT Support
"fitimage" shall have an internal module for creating a tree structure whose
nodes have properties whose property names are interned (using [string-interner]).
This tree structure shall have a method for flattening it to flattened devicetree
format (dtb) as specified in chapter 5 of the [device tree specification].
([[.fdtmodule]])

## Fit Format File
"fitimage" shall produce a [fit format][fit-format] file (built using the fdt
module) with the following constraints:

- [[.addresscells]]: the image-tree/#address-cells shall be 1 for 32 bit targets 
    and 2 for 64 bit targets;
- [[.timestamp]]: the image-tree/timestamp shall be set to a provided value or
    (by default) the current time
- [[.description]]: the image-tree/description shall optionally be set to a
    provided value
- [[.bootloader]]: the provided bootloader ELF file shall be included as an
    image of type "kernal" with an os of type "linux"
- [[.otherelf]]: the other provided ELF files shall be included as images
    of type "ramdisk" and os type "linux", they shall have the load address set;
- [[.singleconfig]]: the image-tree/configurations shall have exactly one
    configuration node which shall be the default;
- [[.configkernel]]: the kernel property of the configuration shall identify
    the bootloader image;
- [[.configdescription]]: the description property of the configuration shall
    be set to a fixed description;
- [[.configloadables]]: the loadables property of the configuration shall
    be set to a list of all the post-processed ELF files except the bootloader;
- [[.configfdt]]: the fdt property of the configuration shall be set to the
    provided FDT file with an overlay FDT of the "fit-image" FDT (described below)
    if an FDT file is specified or to just the "fit-image" FDT file otherwise;

## Fit-Image FDT
As described above, the fit format file produced by "fitimage" shall contain a
"fit-image" FDT which is specified as either the sole FDT in the configuration or
as an overlay FDT. The purpose of the "fit-image" FDT is to pass information about
the loadables included in the configuration to the bootloader. ([[.fitimageftd]])

The root node of the fit-image FDT shall have the following layout:

```
/ o fit-image
    | - #address-cells = <1>
    |
    o image-1 
    | | - load-addr = <00000000>
    |
    o image-2 
    | | - load-addr = <00000000>
    | ...
```

The #address-cells property shall be manadetory and shall specify the number of
32 bit cells required to represent the load addresses for each of the included
images. It shall be 1 for 32 bit targets and 2 for 64 bit targets.

The load-addr property of each image shall be manadetory. Its size in 32 bit cells
shall be specified by the #address-cells property. It shall specify the physical 
address at which the indicated image is to be loaded (or was loaded when the 
bootloader is running). It will point to an ELF header with further information
about the image since the loadable images include in the file produced by "fitimage"
are required to be post-processed ELF files.

[device tree specification]: https://github.com/devicetree-org/devicetree-specification/releases/tag/v0.2
[fit-format]: https://github.com/u-boot/u-boot/blob/master/doc/uImage.FIT/source_file_format.txt
[string-interner]: https://crates.io/crates/string-interner


# TST-elfpreload
The "elf-preload" crate shall have at least the following tests:

- [[.elflint]]: the output of "elf-preload" shall pass a run of eu-elflint from
    the [elfutils] project;
- [[.idempotent]]: running 'objcopy -O binary ..." on the output of "elf-preload"
    shall produce the identical file;
- [[.nosections]]: the output of "elf-preload" shall not contain section headers;
- [[.segments]]: the output of "elf-preload" shall contain a PT_PHDR segment, 
    PT_LOAD segments, and nothing else;

[elfutils]: https://sourceware.org/elfutils/


# TST-fitimage
The "fitimage" crate shall have at least the following tests:

- [[.dtcrecognize]]: the output of "fitimage" shall be recognized by the [dtc] command
- [[.fitimagematch]]: the "fit-image" FDT that is produced by "fitimage" shall match
    in both image name and load-addr property with the image names and load property
    of the corresponding subnode of the "images" node in the output file itself for
    all "loadable" images

[dtc]: http://manpages.ubuntu.com/manpages/trusty/man1/dtc.1.html
