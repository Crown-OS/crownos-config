//! Machine-wide settings under `/etc/crownos` and `/var/lib/crownos`. The
//! CrownOS installer writes them, and the `crownos-setup` runner also writes
//! `setup.ron` and the setup task states. Every user's apps read them and
//! never write them.

pub mod ai;
pub mod setup;
pub mod setup_state;

pub use ai::{Accelerator, AcceleratorKind, AcceleratorVendor, AiConfig, AiConfigError, Backend};
pub use setup::{
    Browser, BrowserEngine, DeveloperTool, DeveloperTools, PackageSource, SETUP_CONFIG_PATH,
    SetupConfig, SetupConfigError, UnknownBrowser, UnknownDeveloperTool,
};
pub use setup_state::{
    SETUP_STATE_DIR, SetupStateError, SetupTask, TaskState, TaskStatus, UnknownSetupTask,
};
