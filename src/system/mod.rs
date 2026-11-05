//! Machine-wide settings the CrownOS installer writes under `/etc/crownos`,
//! read by every user's apps and never written by them.

pub mod ai;
pub mod android;

pub use ai::{Accelerator, AcceleratorKind, AcceleratorVendor, AiConfig, AiConfigError, Backend};
pub use android::{ANDROID_CONFIG_PATH, AndroidConfig, AndroidConfigError, AndroidTier};
