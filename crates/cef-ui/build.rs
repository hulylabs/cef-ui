use anyhow::Result;
use cef_ui_util::{download_and_extract_cef, get_cef_artifacts_dir};

fn main() -> Result<()> {
    let artifacts_dir = get_cef_artifacts_dir()?;
    download_and_extract_cef(&artifacts_dir)?;

    if std::env::var("CARGO_FEATURE_NO_CEF_LINK").is_ok() {
        return Ok(());
    }

    // Linker flags on x86_64 Linux.
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        // Link dynamically to CEF.
        println!("cargo:rustc-link-lib=dylib=cef");
    }

    // Linker flags on macOS.
    #[cfg(target_os = "macos")]
    {
        // Link dynamically to the CEF framework.
        println!("cargo:rustc-link-lib=framework=Chromium Embedded Framework");
    }

    // Linker flags on x86_64 Windows.
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        use cef_ui_util::get_cef_artifacts_dir;

        let artifacts_dir = get_cef_artifacts_dir()?;

        // Link statically to the CEF sandbox.
        println!("cargo:rustc-link-search=native={}", artifacts_dir.display());
        println!("cargo:rustc-link-lib=static=cef_sandbox");

        // Link dynamically to CEF.
        println!("cargo:rustc-link-lib=dylib=libcef");

        // Link dynamically to CEF dependencies.
        println!("cargo:rustc-link-lib=wbemuuid");
        println!("cargo:rustc-link-lib=propsys");
        println!("cargo:rustc-link-lib=delayimp");
    }

    Ok(())
}
