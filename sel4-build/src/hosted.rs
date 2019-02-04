// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use askama::Template;
use bindgen;
use cmake;
use failure::Fallible;

/// The CMake build target to build.
pub enum CMakeTarget {
    /// Build "libsel4.a".
    Library,

    /// Build "kernel.elf".
    Kernel(Platform),
}

/// The seL4 platform to build.
#[derive(Clone, Copy)]
pub enum Platform {
    /// The ia32 and x86_64 PC 99 platform.
    Pc99,

    /// The Sabre platform
    Sabre,

    /// The Omap3 platform (BeagleBoard)
    Omap3,

    /// The Am335x platform (BeagleBone Black)
    Am335x,

    /// The Exynos4 platform
    Exynos4,

    /// The Exynos5410 platform
    Exynos5410,

    /// The Exynos5422 platform
    Exynos5422,

    /// The Exynos5250 platform
    Exynos5250,

    /// The Apq8064 platform
    Apq8064,

    /// The Wandq platform
    Wandq,

    /// The Imx7Sabre platform
    Imx7Sabre,

    /// The Zynq7000 platform
    Zynq7000,

    /// The Zynqmp platform
    Zynqmp,

    /// The Ultra96 platform
    Ultra96,

    /// The Tk1 platform
    Tk1,

    /// The Hikey plaftorm
    Hikey,
}

impl CMakeTarget {
    /// Invoke CMake to generate and build the CMakeTarget
    pub fn build(&self) {
        // check if we sould build anything
        let arch = Arch::from_cargo();
        let profile = Profile::from_cargo();
        if !self.should_build(arch, profile) {
            return;
        }

        // get the build directies
        let dirs = BuildDirs::from_cargo();

        // copy cmake files
        copy_cmake_files(&dirs.out_dir, &dirs.manifest_dir)
            .expect("Unable to copy required cmake files");

        // run the CMake configure and build
        cmake::Config::new(&dirs.out_dir)
            .generator("Ninja")
            .define("CMAKE_TOOLCHAIN_FILE", dirs.toolchain_file())
            .define("LibSel4FunctionAttributes", "public")
            .set_arch_and_platform(arch, self)
            .set_profile(profile, arch)
            .set_cmake_target(self)
            .very_verbose(true)
            .build();

        // copy the build artifact to the expected location
        fs::copy(dirs.build_artifact_src(self), dirs.build_artifact_dst(self))
            .expect("Unable to copy the final build artifact");
    }

    /// Generate the bindings for the appropriate type of build target.
    pub fn bindgen(&self) {
        use self::CMakeTarget::{Kernel, Library};

        // check if we sould build anything
        let arch = Arch::from_cargo();
        let profile = Profile::from_cargo();
        if !self.should_build(arch, profile) {
            return;
        }

        // get the build directies
        let dirs = BuildDirs::from_cargo();

        // generate the bindings
        match self {
            Kernel(platform) => bindgen::builder()
                .header(dirs.plat_header(*platform))
                .use_core()
                .ctypes_prefix("cty")
                .generate_comments(false)
                .set_platform_profile(profile)
                .generate()
                .unwrap()
                .write_to_file(dirs.bindings_file())
                .unwrap(),
            Library => bindgen::builder()
                .header(dirs.sel4_header())
                .use_core()
                .ctypes_prefix("cty")
                .generate_comments(false)
                // Exclude the CONFIG_* constants because these include constants for
                // configuration options that will be different in the final microkernal
                // and it would be misleading to include them here.
                .blacklist_item("CONFIG_.*")
                // Exclude the constants related to the HardwareDebugAPI because these
                // vary from platform to platform and belong in the platform specific
                // crate instead of here.
                .blacklist_item("seL4_FirstBreakpoint")
                .blacklist_item("seL4_FirstDualFunctionMonitor")
                .blacklist_item("seL4_FirstWatchpoint")
                .blacklist_item("seL4_NumDualFunctionMonitors")
                .blacklist_item("seL4_NumExclusiveBreakpoints")
                .blacklist_item("seL4_NumExclusiveWatchpoints")
                .blacklist_item("seL4_NumHWBreakpoints")
                .clang_args(
                    arch.include_dirs(&dirs.libsel4_src(), &dirs.libsel4_build())
                        .iter()
                        .map(|path| format!("-I{}", path.display())),
                )
                .clang_arg(format!("-I{}", dirs.kernel_gen_config().display()))
                .generate()
                .unwrap()
                .write_to_file(dirs.bindings_file())
                .unwrap(),
        }
    }

    fn should_build(&self, arch: Arch, profile: Profile) -> bool {
        use self::Arch::{Aarch32, Aarch64, Ia32, X86_64};
        use self::CMakeTarget::{Kernel, Library};
        use self::Platform::*;
        use self::Profile::Release;

        match self {
            Library => true,
            Kernel(platform) => match platform {
                Pc99 => arch == Ia32 || arch == X86_64,
                Sabre | Exynos4 | Exynos5410 | Exynos5422 | Exynos5250 | Apq8064 | Wandq
                | Imx7Sabre | Zynq7000 | Zynqmp | Ultra96 | Tk1 => arch == Aarch32,
                Omap3 | Am335x => arch == Aarch32 && profile == Release,
                Hikey => arch == Aarch32 || arch == Aarch64,
            },
        }
    }
}

impl Platform {
    fn plat_include_dir_name(&self) -> &'static str {
        use self::Platform::*;

        match self {
            Pc99 => "pc99",
            Wandq | Sabre => "imx6",
            Omap3 => "omap3",
            Am335x => "am335x",
            Exynos5410 | Exynos5422 | Exynos5250 => "exynos5",
            Ultra96 | Zynqmp => "zynqmp",
            Exynos4 => "exynos4",
            Imx7Sabre => "imx7",
            Apq8064 => "apq8064",
            Zynq7000 => "zynq7000",
            Tk1 => "tk1",
            Hikey => "hikey",
        }
    }
}

trait CmakeExt {
    fn set_arch_and_platform(&mut self, arch: Arch, target: &CMakeTarget) -> &mut Self;
    fn set_profile(&mut self, profile: Profile, arch: Arch) -> &mut Self;
    fn set_cmake_target(&mut self, target: &CMakeTarget) -> &mut Self;
}

impl CmakeExt for cmake::Config {
    fn set_arch_and_platform(&mut self, arch: Arch, target: &CMakeTarget) -> &mut Self {
        use self::CMakeTarget::Kernel;
        use self::Platform::*;

        if let Kernel(platform) = target {
            match platform {
                Pc99 => None,
                Sabre => Some("sabre"),
                Omap3 => Some("omap3"),
                Am335x => Some("am335x"),
                Exynos4 => Some("exynos4"),
                Exynos5410 => Some("exynos5410"),
                Exynos5422 => Some("exynos5422"),
                Exynos5250 => Some("exynos5250"),
                Apq8064 => Some("apq8064"),
                Wandq => Some("wandq"),
                Imx7Sabre => Some("imx7sabre"),
                Zynq7000 => Some("zynq7000"),
                Zynqmp => Some("zynqmp"),
                Ultra96 => Some("ultra96"),
                Tk1 => Some("tk1"),
                Hikey => Some("hikey"),
            }
            .map(|plat_name| self.define("KernelARMPlatform", plat_name));
        }

        let arch: &str = arch.into();
        self.define(arch, "1")
    }

    fn set_profile(&mut self, profile: Profile, arch: Arch) -> &mut Self {
        use self::Profile::*;
        use self::Arch::Aarch64;

        let result = match profile {
            Release => self.define("KernelVerificationBuild", "ON"),
            Debug => self
                .define("KernelVerificationBuild", "OFF")
                .define("KernelDebugBuild", "ON")
                .define("KernelPrinting", "ON"),
        };

        match (profile, arch) {
            (Release, _) | (Debug, Aarch64) => result,
            (Debug, _) => result
                .define("HardwareDebugAPI", "ON"),
        }
    }

    fn set_cmake_target(&mut self, target: &CMakeTarget) -> &mut Self {
        use self::CMakeTarget::*;

        match target {
            Library => self.build_target("libsel4.a"),
            Kernel(_) => self.build_target("kernel.elf"),
        }
    }
}

trait BuilderExt {
    fn set_platform_profile(self, profile: Profile) -> Self;
}

impl BuilderExt for bindgen::Builder {
    fn set_platform_profile(self, profile: Profile) -> Self {
        use self::Profile::{Debug, Release};

        match profile {
            Release => self,
            Debug => self.clang_arg("-DCONFIG_HARDWARE_DEBUG_API"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Arch {
    Ia32,
    X86_64,
    Aarch32,
    Aarch64,
}

impl Arch {
    fn new(arch: impl AsRef<str>) -> Result<Arch, String> {
        use self::Arch::*;

        match arch.as_ref() {
            "x86" => Ok(Ia32),
            "x86_64" => Ok(X86_64),
            "arm" => Ok(Aarch32),
            "aarch64" => Ok(Aarch64),
            arch => Err(format!("Unsupported architecture: {}", arch)),
        }
    }

    fn from_cargo() -> Arch {
        Arch::new(env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set."))
            .expect("CARGO_CFG_TARGET_ARCH not set to supported architecture.")
    }

    fn include_dirs(&self, src: &Path, build: &Path) -> Vec<PathBuf> {
        use self::Arch::*;

        // common directories
        let mut dirs = vec![
            src.join("include"),
            build.join("include"),
            build.join("autoconf"),
            build.join("gen_config"),
        ];

        // arch_include directories
        match self {
            Ia32 | X86_64 => {
                dirs.push(src.join("arch_include/x86"));
                dirs.push(build.join("arch_include/x86"));
            }
            Aarch32 | Aarch64 => {
                dirs.push(src.join("arch_include/arm"));
                dirs.push(build.join("arch_include/arm"));
            }
        }

        // sel4_arch_include directories
        match self {
            Ia32 => {
                dirs.push(src.join("sel4_arch_include/ia32"));
                dirs.push(build.join("sel4_arch_include/ia32"));
            }
            X86_64 => {
                dirs.push(src.join("sel4_arch_include/x86_64"));
                dirs.push(build.join("sel4_arch_include/x86_64"));
            }
            Aarch32 => {
                dirs.push(src.join("sel4_arch_include/aarch32"));
                dirs.push(build.join("sel4_arch_include/aarch32"));
            }
            Aarch64 => {
                dirs.push(src.join("sel4_arch_include/aarch64"));
                dirs.push(build.join("sel4_arch_include/aarch64"));
            }
        }

        // mode_include directories
        match self {
            Ia32 | Aarch32 => dirs.push(src.join("mode_include/32")),
            X86_64 | Aarch64 => dirs.push(src.join("mode_include/64")),
        }

        // sel4_plat_include directories
        // NOTE: The contentes of the files in these direcotries will
        // be blacklisted at a later stage, but the direcotries have to
        // be included here for bindgen to work.
        match self {
            Ia32 | X86_64 => dirs.push(src.join("sel4_plat_include/pc99")),
            Aarch32 => dirs.push(src.join("sel4_plat_include/imx6")),
            Aarch64 => dirs.push(src.join("sel4_plat_include/tx1")),
        }

        dirs
    }
}

impl From<Arch> for &'static str {
    fn from(arch: Arch) -> Self {
        use self::Arch::*;

        match arch {
            Ia32 => "IA32",
            X86_64 => "X86_64",
            Aarch32 => "AARCH32",
            Aarch64 => "AARCH64",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Profile {
    Release,
    Debug,
}

impl Profile {
    fn new(profile: impl AsRef<str>) -> Result<Profile, String> {
        use self::Profile::*;

        match profile.as_ref() {
            "release" => Ok(Release),
            "debug" => Ok(Debug),
            profile => Err(format!("Unrecognized profile: {}", profile)),
        }
    }

    fn from_cargo() -> Profile {
        Profile::new(env::var("PROFILE").expect("PROFILE not set."))
            .expect("PROFILE not set to recognized profile.")
    }
}

struct BuildDirs {
    out_dir: PathBuf,
    manifest_dir: PathBuf,
}

impl BuildDirs {
    fn new(out_dir: &str, manifest_dir: &str) -> BuildDirs {
        BuildDirs {
            out_dir: PathBuf::from(out_dir),
            manifest_dir: PathBuf::from(manifest_dir),
        }
    }

    fn from_cargo() -> BuildDirs {
        BuildDirs::new(
            &get_cargo_var("OUT_DIR"),
            &get_cargo_var("CARGO_MANIFEST_DIR"),
        )
    }

    fn toolchain_file(&self) -> PathBuf {
        self.manifest_dir.join("seL4/gcc.cmake")
    }

    fn build_artifact_src(&self, target: &CMakeTarget) -> PathBuf {
        use self::CMakeTarget::*;

        match target {
            Library => self.out_dir.join("build/libsel4/libsel4.a"),
            Kernel(_) => self.out_dir.join("build/kernel/kernel.elf"),
        }
    }

    fn build_artifact_dst(&self, target: &CMakeTarget) -> PathBuf {
        use self::CMakeTarget::*;

        match target {
            Library => self.out_dir.join("libsel4.a"),
            Kernel(_) => self.out_dir.join("kernel.elf"),
        }
    }

    fn sel4_header(&self) -> String {
        self.manifest_dir
            .join("seL4/libsel4/include/sel4/sel4.h")
            .into_os_string()
            .into_string()
            .expect("Path to sel4.h contained non-UTF8 characters")
    }

    fn plat_header(&self, platform: Platform) -> String {
        let mut plat_dir = self.manifest_dir.join("seL4/libsel4/sel4_plat_include");
        plat_dir.push(platform.plat_include_dir_name());
        plat_dir.push("sel4/plat/api/constants.h");

        plat_dir
            .into_os_string()
            .into_string()
            .expect("Path to sel4 platform specifc constants containted non-UTF8 characers")
    }

    fn libsel4_src(&self) -> PathBuf {
        self.manifest_dir.join("seL4/libsel4")
    }

    fn libsel4_build(&self) -> PathBuf {
        self.out_dir.join("build/libsel4")
    }

    fn kernel_gen_config(&self) -> PathBuf {
        self.out_dir.join("build/kernel/gen_config")
    }

    fn bindings_file(&self) -> PathBuf {
        self.out_dir.join("bindings.rs")
    }
}

/// Get the value of a cargo environment variable and panic if it is not set.
pub fn get_cargo_var(var: &str) -> String {
    env::var(var).expect(&format!("{} not set.", var))
}

/// Copy the requied root cmake files.
///
/// This function can be called from outside of a build.rs context (it does
/// not depend on the cargo environment variables being set).
pub fn copy_cmake_files<P, Q>(out_dir: P, manifest_dir: Q) -> Fallible<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let rootfile = out_dir.as_ref().join("CMakeLists.txt");
    let flagsfile = out_dir.as_ref().join("flags.cmake");

    fs::write(
        rootfile,
        CMakeListsTemplate::new(manifest_dir.as_ref()).render()?,
    )?;
    fs::write(flagsfile, include_str!("flags.cmake"))?;

    Ok(())
}

#[derive(Template)]
#[template(path = "CMakeLists.txt")]
struct CMakeListsTemplate<'a> {
    manifest_dir: &'a str,
}

impl<'a> CMakeListsTemplate<'a> {
    fn new(manifest_dir: &Path) -> CMakeListsTemplate {
        CMakeListsTemplate {
            manifest_dir: manifest_dir
                .to_str()
                .expect("manifest_dir contains non-UTF8 characters"),
        }
    }
}
