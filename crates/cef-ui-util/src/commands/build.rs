use crate::build_exe;
use anyhow::Result;
use tracing::info;

/// Build a cargo package.
pub struct BuildCommand {
    /// The binary to build.
    pub binary: String,

    /// The profile to use.
    pub profile: String,

    /// The target to build for.
    pub target: String
}

impl BuildCommand {
    pub fn run(&self) -> Result<()> {
        info!("Building {} ..", self.binary);

        build_exe(&self.binary, &self.profile, &self.target)?;

        info!("Done!");

        Ok(())
    }
}
