crate::section! {
    pub struct Display in "display", keys DisplayKey {
        /// Deprecated: per-output brightness replaces this value. Kept so old files load.
        pub brightness as Brightness: f64 = 80.0,
        pub night_light as NightLight: bool = false,
        pub night_light_warmth as NightLightWarmth: f64 = 50.0,
    }
}

// `scale` used to live here as a coarse system-wide enum. Scale is per-monitor
// now — see `compositor::OutputSetting::scale` — and an output with no setting
// of its own gets one guessed from its physical size, which is what the old
// global could never do on a mixed-DPI desk.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_with_the_old_global_brightness_still_loads() -> Result<(), ron::error::SpannedError> {
        let parsed: Display =
            crate::parser::options().from_str("(brightness: 80.0, night_light: true)")?;
        assert_eq!(parsed.brightness, 80.0);
        assert!(parsed.night_light);
        Ok(())
    }
}
