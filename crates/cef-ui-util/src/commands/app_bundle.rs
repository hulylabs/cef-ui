use crate::{copy_files, get_cef_target_dir};
use anyhow::Result;
use std::{
    fs::{File, copy, create_dir_all, remove_dir_all},
    io::{Read, Write},
    path::{Path, PathBuf}
};
use tracing::info;

/// Use this to build app bundles on macOS.
pub struct AppBundleSettings {
    /// The profile to use.
    pub profile: String,

    /// The target to use.
    pub target: String,

    /// The artifacts directory.
    pub artifacts_dir: PathBuf,

    /// The name of the app bundle (aka, {app_name}.app). This name is
    /// also the name that must be used for the main executable within
    /// the app bundle. This is required by CEF.
    pub app_name: String,

    /// The main executable name.
    pub main_exe_name: String,

    /// The helper executable name.
    pub helper_exe_name: String,

    /// The resources directory where the app bundle template
    /// files are stored (for example, plists and icons).
    pub resources_dir: PathBuf,

    /// The org name to use in plists.
    pub org_name: String
}

impl AppBundleSettings {
    pub fn run(&self) -> Result<()> {
        info!("Building app bundle {} ..", self.app_name);

        let target_dir = get_cef_target_dir(&self.profile, &self.target)?;
        let app_dir = target_dir.join(format!("{}.app", self.app_name));

        info!("Remove any existing app");
        if app_dir.exists() {
            remove_dir_all(&app_dir)?;
        }

        info!("Create main bundle folders");
        create_dir_all(app_dir.clone())?;
        create_dir_all(app_dir.join("Contents/Frameworks"))?;
        create_dir_all(app_dir.join("Contents/MacOS"))?;
        create_dir_all(app_dir.join("Contents/Resources"))?;

        info!("Create the main plist");
        {
            let org_name = format!("org.{}.{}", self.org_name, self.app_name);

            Self::create_plist(
                &self
                    .resources_dir
                    .join("Info.plist"),
                &app_dir.join("Contents/Info.plist"),
                &org_name,
                &self.app_name
            )?;
        }

        info!("Copy the main icon");
        copy(
            self.resources_dir.join("Icon.icns"),
            app_dir.join("Contents/Resources/Icon.icns")
        )?;

        info!("Copy the English localization");
        copy_files(
            &self
                .resources_dir
                .join("English.lproj"),
            &app_dir.join("Contents/Resources/English.lproj")
        )?;

        info!("Copy the main executable");
        copy(
            target_dir.join(&self.main_exe_name),
            app_dir
                .join("Contents/MacOS/")
                .join(&self.app_name)
        )?;

        info!("Copy the CEF framework");
        copy_files(
            &self
                .artifacts_dir
                .join("Chromium Embedded Framework.framework"),
            &app_dir.join("Contents/Frameworks/Chromium Embedded Framework.framework")
        )?;

        info!("Function to create helper app bundles");
        let create_helper = |name: Option<&str>| -> Result<()> {
            let org_name = match name {
                Some(name) => {
                    let name = name.to_lowercase();

                    format!("org.{}.{}.helper.{}", self.org_name, self.app_name, name)
                },
                None => format!("org.{}.{}.helper", self.org_name, self.app_name)
            };

            let app_name = match name {
                Some(name) => format!("{} Helper ({})", self.app_name, name),
                None => format!("{} Helper", self.app_name)
            };

            let app_dir = app_dir.join(format!("Contents/Frameworks/{}.app", app_name));

            info!("Create helper bundle folders");
            create_dir_all(app_dir.clone())?;
            create_dir_all(app_dir.join("Contents/MacOS"))?;

            info!("Create the helper plist");
            Self::create_plist(
                &self
                    .resources_dir
                    .join("HelperInfo.plist"),
                &app_dir.join("Contents/Info.plist"),
                &org_name,
                &app_name
            )?;

            copy(
                target_dir.join(&self.helper_exe_name),
                app_dir
                    .join("Contents/MacOS")
                    .join(app_name)
            )?;

            Ok(())
        };

        info!("Create the helper bundles");
        create_helper(None)?;
        create_helper(Some("Alerts"))?;
        create_helper(Some("GPU"))?;
        create_helper(Some("Plugin"))?;
        create_helper(Some("Renderer"))?;

        info!("Done!");

        Ok(())
    }

    /// Create the main plist file.
    fn create_plist(src: &Path, dst: &Path, org_name: &str, app_name: &str) -> Result<()> {
        let mut file = File::open(src)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        contents = contents
            .replace("{org-name}", org_name)
            .replace("{app-name}", app_name);

        let mut output = File::create(dst)?;

        output.write_all(contents.as_bytes())?;

        Ok(())
    }
}
