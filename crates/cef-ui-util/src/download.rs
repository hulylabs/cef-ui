use crate::{download, extract_zip};
use anyhow::Result;
use std::{fs::create_dir_all, path::Path};

/// The current CEF version.
pub const CEF_VERSION: &str = "v0.1.0";

/// Returns the platform-specific CEF artifacts url.
#[cfg(target_os = "linux")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-linux-x86_64.zip";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-macos-arm64.zip";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-macos-x86_64.zip";
#[cfg(target_os = "windows")]
const CEF_URL: &str =
    "https://github.com/hulylabs/cef-ui/releases/latest/download/cef-windows-x86_64.zip";

/// Downloads the tarball, untars it, and decompresses it. If the
/// target directory exists, then this function does nothing.
pub fn download_and_extract_cef(dir: &Path) -> Result<()> {
    if dir.exists() {
        return Ok(());
    }

    // Create the new directory.
    create_dir_all(dir)?;

    let data = download(&CEF_URL)?;
    extract_zip(&data, dir)?;

    Ok(())
}
