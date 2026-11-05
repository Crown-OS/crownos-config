//! Android apps, run by `crown-androidd`.
//!
//! The consumers are `crown-androidd`, which follows this file live, and the
//! settings panel's Android apps page, which writes it.

crate::section! {
    pub struct Android in "android", keys AndroidKey {
        /// Stop Android this many minutes after its last window closes, to give
        /// the memory back. Zero keeps it running until the session ends.
        pub idle_minutes as IdleMinutes: u32 = 5,

        /// Show the Downloads folder to Android apps.
        pub share_downloads as ShareDownloads: bool = true,

        /// Show the Pictures folder to Android apps.
        pub share_pictures as SharePictures: bool = false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_stops_five_minutes_after_its_last_window() {
        assert_eq!(Android::default().idle_minutes, 5);
    }

    #[test]
    fn round_trips_through_ron() {
        let android = Android {
            idle_minutes: 0,
            share_downloads: false,
            share_pictures: true,
        };
        let text = ron::ser::to_string_pretty(&android, Default::default())
            .expect("serialise android section");
        assert_eq!(ron::from_str::<Android>(&text).ok(), Some(android));
    }
}
