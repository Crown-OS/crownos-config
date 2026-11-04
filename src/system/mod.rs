//! Machine-wide settings the CrownOS installer writes under `/etc/crownos`,
//! read by every user's apps and never written by them.

pub mod ai;

pub use ai::{Accelerator, AcceleratorKind, AcceleratorVendor, AiConfig, AiConfigError, Backend};
