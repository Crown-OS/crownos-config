//! `/etc/crownos/ai.ron`: the accelerators the installer found and the AI
//! backends it set up for them. No file means AI features were not chosen.

use std::io;
use std::path::Path;

use serde::Deserialize;

pub const AI_CONFIG_PATH: &str = "/etc/crownos/ai.ron";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum AcceleratorVendor {
    Nvidia,
    Amd,
    Intel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum AcceleratorKind {
    Gpu,
    Npu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Accelerator {
    pub vendor: AcceleratorVendor,
    pub kind: AcceleratorKind,
    pub device_id: u16,
}

/// An inference stack. [`AiConfig::backends`] lists them best first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Backend {
    Maxine,
    Cuda,
    MiGraphX,
    OpenVino,
    VitisAi,
    Cpu,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub accelerators: Vec<Accelerator>,
    #[serde(default)]
    pub backends: Vec<Backend>,
}

#[derive(Debug, thiserror::Error)]
pub enum AiConfigError {
    #[error("could not read {AI_CONFIG_PATH}: {0}")]
    Read(#[from] io::Error),
    #[error("could not parse {AI_CONFIG_PATH}: {0}")]
    Parse(#[from] ron::error::SpannedError),
}

impl AiConfig {
    /// The installed system's AI setup, or `None` when it has none.
    pub fn load() -> Result<Option<Self>, AiConfigError> {
        Self::load_from(Path::new(AI_CONFIG_PATH))
    }

    pub fn load_from(path: &Path) -> Result<Option<Self>, AiConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Ok(Some(Self::parse(&text)?)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn parse(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }

    /// Where `backend` ranks, best first; `None` when it was not set up.
    pub fn rank(&self, backend: Backend) -> Option<usize> {
        self.backends.iter().position(|&listed| listed == backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WRITTEN_BY_THE_INSTALLER: &str = "(
    accelerators: [
        (vendor: Nvidia, kind: Gpu, device_id: 0x2684),
        (vendor: Intel, kind: Npu, device_id: 0x7d1d),
    ],
    backends: [Maxine, Cuda, OpenVino, Cpu],
)
";

    #[test]
    fn reads_what_the_installer_writes() {
        let config = AiConfig::parse(WRITTEN_BY_THE_INSTALLER).expect("valid");
        assert_eq!(
            config.accelerators[0],
            Accelerator {
                vendor: AcceleratorVendor::Nvidia,
                kind: AcceleratorKind::Gpu,
                device_id: 0x2684,
            }
        );
        assert_eq!(config.rank(Backend::Maxine), Some(0));
        assert_eq!(config.rank(Backend::Cpu), Some(3));
        assert_eq!(config.rank(Backend::MiGraphX), None);
    }

    #[test]
    fn a_missing_file_means_no_ai_setup() {
        let missing = Path::new("/nonexistent/crownos/ai.ron");
        assert!(matches!(AiConfig::load_from(missing), Ok(None)));
    }

    #[test]
    fn an_empty_record_parses() {
        assert_eq!(AiConfig::parse("()").expect("valid"), AiConfig::default());
    }
}
