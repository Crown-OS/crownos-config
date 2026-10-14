//! The status bar: where it sits, how it is shaped and what is on it.
//!
//! Widgets are named by id. Built-ins use their own name (`"clock"`); a
//! plugin's widget is `"plugin:<plugin id>/<widget id>"`. A built-in missing
//! from all three lists is hidden.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BarPosition {
    #[default]
    Top,
    Bottom,
}

pub const PLUGIN_PREFIX: &str = "plugin:";

crate::section! {
    pub struct Bar in "bar", keys BarKey {
        /// Overrides `appearance.bar_height`.
        pub height as Height: Option<u32> = None,
        pub position as Position: BarPosition = BarPosition::Top,
        /// Detach the bar from the screen edge by `margin`.
        pub floating as Floating: bool = false,
        pub margin as Margin: u16 = 0,
        pub radius as Radius: Option<u16> = None,
        pub left as Left: Vec<String> = vec!["clock".to_owned()],
        pub center as Center: Vec<String> = Vec::new(),
        pub right as Right: Vec<String> = [
            "layout",
            "weather",
            "caffeine",
            "volume",
            "brightness",
            "bluetooth",
            "wifi",
            "battery",
            "notifications",
        ]
        .map(str::to_owned)
        .to_vec(),
    }
}

impl Bar {
    pub fn height_or(&self, fallback: u32) -> u32 {
        self.height.unwrap_or(fallback)
    }

    /// `(plugin, widget)` of a `plugin:<id>/<widget>` entry.
    pub fn plugin_entry(entry: &str) -> Option<(&str, &str)> {
        entry.strip_prefix(PLUGIN_PREFIX)?.split_once('/')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_entries_split_into_plugin_and_widget() {
        assert_eq!(
            Bar::plugin_entry("plugin:dev.alice.cpu/graph"),
            Some(("dev.alice.cpu", "graph"))
        );
        assert_eq!(Bar::plugin_entry("clock"), None);
    }
}
