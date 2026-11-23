//! `/etc/crownos/ai.ron`: the accelerators the installer found and the AI
//! backends it set up for them. No file means AI features were not chosen.

use std::borrow::Cow;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

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
///
/// This is the first schema. It mixes runtimes and devices; new readers use
/// [`Runtime`] and [`Device`] through [`AiConfig::ranked_devices`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Backend {
    Maxine,
    Cuda,
    MiGraphX,
    OpenVino,
    VitisAi,
    Cpu,
}

/// The engine that loads and runs a model.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Runtime {
    OnnxRuntime,
    LlamaCpp,
    Maxine,
    /// A server with an OpenAI-compatible API: a local sidecar, a LAN machine or a cloud.
    OpenAiCompatible,
    /// A runtime that a newer version of this schema added.
    #[serde(other)]
    Unknown,
}

/// The execution target that a [`Runtime`] runs a model on.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Device {
    TensorRt,
    Cuda,
    Rocm,
    MiGraphX,
    OpenVinoNpu,
    OpenVinoGpu,
    VitisAi,
    Vulkan,
    Cpu,
    Remote,
    /// A device that a newer version of this schema added.
    #[serde(other)]
    Unknown,
}

impl Runtime {
    pub const fn supports(self, device: Device) -> bool {
        use Device as D;
        match self {
            Self::OnnxRuntime => matches!(
                device,
                D::TensorRt
                    | D::Cuda
                    | D::MiGraphX
                    | D::OpenVinoNpu
                    | D::OpenVinoGpu
                    | D::VitisAi
                    | D::Cpu
            ),
            Self::LlamaCpp => matches!(device, D::Cuda | D::Rocm | D::Vulkan | D::Cpu),
            Self::Maxine => matches!(device, D::Cuda),
            Self::OpenAiCompatible => matches!(device, D::Remote),
            Self::Unknown => false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub accelerators: Vec<Accelerator>,
    #[serde(default)]
    pub backends: Vec<Backend>,
    /// The execution targets set up on this machine, best first. Empty in files
    /// written before this field: [`AiConfig::ranked_devices`] then derives them.
    #[serde(default)]
    pub devices: Vec<Device>,
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

    /// The execution targets set up on this machine, best first, without duplicates.
    pub fn ranked_devices(&self) -> Cow<'_, [Device]> {
        if !self.devices.is_empty() {
            return Cow::Borrowed(&self.devices);
        }
        let mut devices: Vec<Device> = Vec::new();
        for &device in self
            .backends
            .iter()
            .flat_map(|&backend| self.legacy_devices(backend))
        {
            if !devices.contains(&device) {
                devices.push(device);
            }
        }
        if self.has_accelerator(|kind, _| kind == AcceleratorKind::Gpu) {
            let before_cpu = devices
                .iter()
                .position(|&device| device == Device::Cpu)
                .unwrap_or(devices.len());
            devices.insert(before_cpu, Device::Vulkan);
        }
        Cow::Owned(devices)
    }

    /// Where `device` ranks in [`AiConfig::ranked_devices`]; `None` when it was not set up.
    pub fn device_rank(&self, device: Device) -> Option<usize> {
        self.ranked_devices()
            .iter()
            .position(|&listed| listed == device)
    }

    fn legacy_devices(&self, backend: Backend) -> &'static [Device] {
        let intel_npu = self.has_accelerator(|kind, vendor| {
            (kind, vendor) == (AcceleratorKind::Npu, AcceleratorVendor::Intel)
        });
        let intel_gpu = self.has_accelerator(|kind, vendor| {
            (kind, vendor) == (AcceleratorKind::Gpu, AcceleratorVendor::Intel)
        });
        match backend {
            Backend::Maxine => &[],
            Backend::Cuda => &[Device::TensorRt, Device::Cuda],
            Backend::MiGraphX => &[Device::MiGraphX],
            Backend::OpenVino => match (intel_npu, intel_gpu) {
                (true, false) => &[Device::OpenVinoNpu],
                (false, true) => &[Device::OpenVinoGpu],
                _ => &[Device::OpenVinoNpu, Device::OpenVinoGpu],
            },
            Backend::VitisAi => &[Device::VitisAi],
            Backend::Cpu => &[Device::Cpu],
        }
    }

    fn has_accelerator(&self, wanted: impl Fn(AcceleratorKind, AcceleratorVendor) -> bool) -> bool {
        self.accelerators
            .iter()
            .any(|accelerator| wanted(accelerator.kind, accelerator.vendor))
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
    fn derives_devices_from_a_file_without_them() {
        let config = AiConfig::parse(WRITTEN_BY_THE_INSTALLER).expect("valid");
        assert_eq!(
            config.ranked_devices().as_ref(),
            [
                Device::TensorRt,
                Device::Cuda,
                Device::OpenVinoNpu,
                Device::Vulkan,
                Device::Cpu
            ]
        );
        assert_eq!(config.device_rank(Device::Cuda), Some(1));
        assert_eq!(config.device_rank(Device::Rocm), None);
    }

    #[test]
    fn listed_devices_win_over_the_derived_ones() {
        let config =
            AiConfig::parse("(backends: [Cuda, Cpu], devices: [Cuda, Vulkan, Remote, Cpu])")
                .expect("valid");
        assert_eq!(
            config.ranked_devices().as_ref(),
            [Device::Cuda, Device::Vulkan, Device::Remote, Device::Cpu]
        );
    }

    #[test]
    fn a_reader_of_the_first_schema_ignores_the_devices_field() {
        #[derive(Deserialize)]
        struct FirstSchema {
            backends: Vec<Backend>,
        }
        let first: FirstSchema =
            ron::from_str("(backends: [Cuda, Cpu], devices: [Rocm, Vulkan, Cpu])").expect("valid");
        assert_eq!(first.backends, [Backend::Cuda, Backend::Cpu]);
    }

    #[test]
    fn a_device_from_a_newer_schema_reads_as_unknown() {
        let config = AiConfig::parse("(devices: [Npu9000, Cuda])").expect("valid");
        assert_eq!(config.devices, [Device::Unknown, Device::Cuda]);
        assert!(!Runtime::OnnxRuntime.supports(Device::Unknown));
    }

    #[test]
    fn runtimes_name_the_devices_they_run_on() {
        assert!(Runtime::LlamaCpp.supports(Device::Vulkan));
        assert!(!Runtime::OnnxRuntime.supports(Device::Vulkan));
        assert!(Runtime::OpenAiCompatible.supports(Device::Remote));
        assert!(!Runtime::OpenAiCompatible.supports(Device::Cpu));
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
