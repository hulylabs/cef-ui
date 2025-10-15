use std::env::var;

use crate::{copy_files, get_cef_artifacts_dir, get_cef_target_dir};
use anyhow::Result;

/// Call this in your binary crate's build.rs
/// file to properly link against CEF.
pub fn link_cef() -> Result<()> {
    let artifacts_dir = get_cef_artifacts_dir()?;
    println!(
        "cargo:warning=Artifacts directory: {}",
        artifacts_dir.display()
    );

    // Linker flags on x86_64 Linux.
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        // Copy the CEF binaries.
        copy_cef_linux()?;

        // This tells Rust where to find libcef.so at compile time.
        println!("cargo:rustc-link-search=native={}", artifacts_dir.display());

        // This tells Rust where to find libcef.so at runtime.
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/cef");
    }

    // Linker flags on arm64 macOS.
    #[cfg(target_os = "macos")]
    {
        // This tells Rust where to find the CEF framework at compile time.
        println!(
            "cargo:rustc-link-search=framework={}",
            artifacts_dir.display()
        );
    }

    // Linker flags on x86_64 Windows.
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        // Copy the CEF binaries.
        copy_cef_windows()?;

        // This tells Rust where to find libcef.lib at compile time.
        println!("cargo:rustc-link-search=native={}", artifacts_dir.display());
    }

    Ok(())
}

/// Copy the CEF files to the target directory on Linux.
#[allow(dead_code)]
fn copy_cef_linux() -> Result<()> {
    use crate::CEF_DIRECTORY;

    let profile = var("PROFILE")?;
    let src = get_cef_artifacts_dir()?;
    let dst = get_cef_target_dir(&profile, "")?.join(CEF_DIRECTORY);

    println!(
        "cargo:warning=Copying CEF files from {} to {}",
        src.display(),
        dst.display()
    );

    if src.exists() {
        println!("cargo:warning=Source directory exists");
    } else {
        println!("cargo:warning=Source directory does not exist");
    }

    if dst.exists() {
        println!("cargo:warning=Destination directory exists");
    } else {
        println!("cargo:warning=Destination directory does not exist");
    }

    copy_files(&src, &dst)?;

    println!("cargo:warning=Finished copying CEF files");

    Ok(())
}

/// Copy the CEF files to the target directory on Windows.
#[allow(dead_code)]
fn copy_cef_windows() -> Result<()> {
    let profile = var("PROFILE")?;
    let src = get_cef_artifacts_dir()?;
    let dst = get_cef_target_dir(&profile, "")?;

    // Copy the CEF binaries.
    copy_files(&src, &dst)?;

    Ok(())
}

/// Call this in your binary helper crate's build.rs file to
/// properly link against the sandbox library.
pub fn link_cef_helper() -> Result<()> {
    // We must link against the macOS sandbox library.
    println!("cargo:rustc-link-lib=sandbox");

    Ok(())
}
