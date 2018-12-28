# Copyright 2018 Steven Bosnick
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms

cmake_minimum_required(VERSION 3.7.2)
project(rustsel4build C ASM)

# The releative path to the released seL4 source.
set(KERNEL_PATH seL4-10.1.1)

# Set the architecture and sub-architecture using
# the same variables that ${KERNEL_PATH}/gcc.cmake uses.
if(AARCH32 OR AARCH32HF)
    set(KernelArch "arm" CACHE STRING "The kernel architecture.")
    set(KernelArmSel4Arch "aarch32")
elseif(AARCH64)
    set(KernelArch "arm" CACHE STRING "The kernel architecture.")
    set(KernelArmSel4Arch "aarch64")
elseif(IA32)
    set(KernelArch "x86" CACHE STRING "The kernel architecture.")
    set(KernelX86Sel4Arch "ia32")
elseif(X86_64)
    set(KernelArch "x86" CACHE STRING "The kernel architecture.")
    set(KernelX86Sel4Arch "x86_64")
else()
    message(FATAL_ERROR "One of AARCH32, AARCH32HF, AARCH64, IA32, or X86_64 is required.")
endif()

# add the kernel and its helpers
add_subdirectory("${KERNEL_PATH}" kernel)
include(${KERNEL_HELPERS_PATH})

# setup the build flags
include(flags.cmake)

# add libsel4
add_subdirectory("${KERNEL_PATH}/libsel4" libsel4)
