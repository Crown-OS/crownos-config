//! `/etc/crownos/android.ron`: whether the installer set up Android apps and
//! how the setup service found this machine. No file means Android apps were
//! not chosen.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const ANDROID_CONFIG_PATH: &str = "/etc/crownos/android.ron";

/// How Android renders on this machine, decided by the setup service.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AndroidTier {
    /// Mesa on the compositor's GPU.
    Native,
    /// Venus through the host render server on an NVIDIA GPU.
    Venus,
    /// SwiftShader on the CPU.
    Software,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AndroidConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Whether the user chose the image with Google Play.
    #[serde(default)]
    pub google_play: bool,
    /// Absent until the setup service has probed the machine.
    #[serde(default)]
    pub tier: Option<AndroidTier>,
    /// Version of the installed CrownOS overlay bundle.
    #[serde(default)]
    pub bundle: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AndroidConfigError {
    #[error("could not read {ANDROID_CONFIG_PATH}: {0}")]
    Read(#[from] io::Error),
    #[error("could not parse {ANDROID_CONFIG_PATH}: {0}")]
    Parse(#[from] ron::error::SpannedError),
    #[error("could not write {ANDROID_CONFIG_PATH}: {0}")]
    Write(#[from] ron::Error),
}

impl AndroidConfig {
    /// The installed system's Android setup, or `None` when it has none.
    pub fn load() -> Result<Option<Self>, AndroidConfigError> {
        Self::load_from(Path::new(ANDROID_CONFIG_PATH))
    }

    pub fn load_from(path: &Path) -> Result<Option<Self>, AndroidConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Ok(Some(ron::from_str(&text)?)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// The text the installer and the setup service write.
    pub fn to_ron(&self) -> Result<String, AndroidConfigError> {
        Ok(ron::ser::to_string_pretty(self, Default::default())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WRITTEN_BY_THE_INSTALLER: &str = "(enabled: true, google_play: true)";

    #[test]
    fn reads_what_the_installer_writes() {
        let config: AndroidConfig = ron::from_str(WRITTEN_BY_THE_INSTALLER).expect("valid");
        assert!(config.enabled && config.google_play);
        assert_eq!(config.tier, None);
    }

    #[test]
    fn round_trips_what_the_setup_service_writes() {
        let config = AndroidConfig {
            enabled: true,
            google_play: false,
            tier: Some(AndroidTier::Venus),
            bundle: Some("2026.11.0".into()),
        };
        let text = config.to_ron().expect("serialise");
        assert_eq!(ron::from_str::<AndroidConfig>(&text).ok(), Some(config));
    }

    #[test]
    fn a_missing_file_means_android_was_not_chosen() {
        let missing = Path::new("/nonexistent/crownos/android.ron");
        assert!(matches!(AndroidConfig::load_from(missing), Ok(None)));
    }
}
