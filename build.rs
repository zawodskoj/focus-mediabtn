//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(all(feature = "bootloader", not(feature = "elf")))]
    {
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(include_bytes!("linker-scripts/memory.x"))
            .unwrap();

        println!("cargo:rerun-if-changed=linker-scripts/memory.x");
    }

    #[cfg(all(feature = "bootloader", feature = "elf"))]
    {
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(include_bytes!("linker-scripts/memory-elf.x"))
            .unwrap();

        println!("cargo:rerun-if-changed=linker-scripts/memory-elf.x");
    }

    #[cfg(not(feature = "bootloader"))]
    {
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(include_bytes!("linker-scripts/memory-standalone.x"))
            .unwrap();
        println!("cargo:rerun-if-changed=linker-scripts/memory-standalone.x");
    }
    println!("cargo:rustc-link-search={}", out.display());
}

