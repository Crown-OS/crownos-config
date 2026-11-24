use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use crownos_config::*;

/// How long to wait for a notification that should arrive.
const DELIVERED: Duration = Duration::from_secs(5);
/// How long to wait before concluding a notification will never arrive.
const SILENT: Duration = Duration::from_millis(400);

/// One test function, because `CROWN_CONFIG_DIR` is process-global and
/// cargo runs test functions on parallel threads.
#[test]
fn e2e() {
    let dir = setup_environment();

    check_paths(&dir);
    check_load_writes_nothing(&dir);
    check_layered_section(&dir);
    check_save_load_round_trip();
    check_save_records_its_own_hash();
    check_external_edit_breaks_the_hash();
    check_unparseable_file_is_not_clobbered();
    check_watcher();
    check_key_watcher();

    clear_environment(&dir);
}

/// `<root>/config` is the user's directory, `<root>/distro` and
/// `<root>/machine` are the default layers and `<root>/history` holds the
/// defaults of earlier releases.
fn setup_environment() -> PathBuf {
    let root = std::env::temp_dir().join(format!("crownos-config-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for directory in ["config", "distro", "machine", "history/0.2.0"] {
        std::fs::create_dir_all(root.join(directory)).expect("create temp config dir");
    }
    let layers = std::env::join_paths([root.join("distro"), root.join("machine")])
        .expect("join layer directories");
    // SAFETY: single-threaded section of a single test function; no other
    // test in this binary touches the environment.
    unsafe {
        std::env::set_var(CONFIG_DIR_ENV, root.join("config"));
        std::env::set_var(layers::CONFIG_LAYERS_ENV, layers);
        std::env::set_var(layers::CONFIG_HISTORY_ENV, root.join("history"));
    }

    root
}

fn clear_environment(root: &Path) {
    unsafe {
        std::env::remove_var(CONFIG_DIR_ENV);
        std::env::remove_var(layers::CONFIG_LAYERS_ENV);
        std::env::remove_var(layers::CONFIG_HISTORY_ENV);
    }
    let _ = std::fs::remove_dir_all(root);
}

/// `CROWN_CONFIG_DIR` wins over the platform config dir, and a section maps
/// to `<dir>/<section>.ron`.
fn check_paths(root: &Path) {
    assert_eq!(config_dir(), root.join("config"));
    assert_eq!(path_for(Wifi::SECTION), root.join("config/wifi.ron"));
}

/// A load of a missing file gives the defaults and writes no section file, so
/// each key follows the defaults of later releases. A file that the user
/// writes after that is never reduced.
fn check_load_writes_nothing(root: &Path) {
    let loaded: Wifi = load(Wifi::SECTION);
    assert_eq!(loaded, Wifi::default());
    assert!(!root.join("config/wifi.ron").exists());

    let hand_written = format!("({})\n", "enabled: true");
    std::fs::write(root.join("config/wifi.ron"), &hand_written).expect("write wifi");
    let _: Wifi = load(Wifi::SECTION);
    assert_eq!(
        std::fs::read_to_string(root.join("config/wifi.ron")).unwrap_or_default(),
        hand_written
    );
    std::fs::remove_file(root.join("config/wifi.ron")).expect("remove wifi");
}

/// The default layers, the user's changes on top, a save that writes only the
/// user's changes, a system update of the defaults, and the reduction of a
/// file that an earlier release wrote out in full.
fn check_layered_section(root: &Path) {
    let distro = root.join("distro/appearance.ron");
    let user_file = path_for(Appearance::SECTION);
    std::fs::write(&distro, "(accent: Blue, bar_height: 36)").expect("write distro layer");
    std::fs::write(root.join("machine/appearance.ron"), "(bar_height: 44)")
        .expect("write machine layer");

    let mut appearance: Appearance = load(Appearance::SECTION);
    assert_eq!(appearance.accent, AccentColor::Blue);
    assert_eq!(appearance.bar_height, 44);
    assert_eq!(appearance.transparency, Appearance::default().transparency);

    appearance.dark_mode = !appearance.dark_mode;
    let chosen_dark_mode = appearance.dark_mode;
    save(Appearance::SECTION, &appearance).expect("save appearance");
    let user_text = String::from_utf8(read(Appearance::SECTION)).unwrap_or_default();
    assert!(user_text.starts_with(USER_FILE_HEADER));
    assert!(user_text.contains("dark_mode"));
    assert!(
        !user_text.contains("accent"),
        "a default must not be written: {user_text}"
    );

    std::fs::write(&distro, "(accent: Green, bar_height: 36)").expect("update distro layer");
    let appearance: Appearance = load(Appearance::SECTION);
    assert_eq!(
        appearance.accent,
        AccentColor::Green,
        "an update changes a default"
    );
    assert_eq!(
        appearance.dark_mode, chosen_dark_mode,
        "the user's value wins"
    );

    std::fs::write(&distro, "(accent: Pink)").expect("update distro layer while the app runs");
    save(Appearance::SECTION, &appearance).expect("save with the defaults of the last load");
    let user_text = String::from_utf8(read(Appearance::SECTION)).unwrap_or_default();
    assert!(
        !user_text.contains("accent"),
        "a stale default must not be pinned: {user_text}"
    );
    let appearance: Appearance = load(Appearance::SECTION);
    assert_eq!(appearance.accent, AccentColor::Pink);

    std::fs::write(
        root.join("history/0.2.0/appearance.ron"),
        "(accent: Orange)",
    )
    .expect("write earlier release defaults");
    let written_in_full = Appearance {
        accent: AccentColor::Orange,
        transparency: 0.33,
        ..Appearance::default()
    };
    std::fs::write(&user_file, to_ron(&written_in_full)).expect("write a full user file");
    let reduced_marker = root.join("config/.layered/appearance");
    assert!(reduced_marker.exists(), "the first read marks the section");
    std::fs::remove_file(&reduced_marker).expect("make the home look like one from before layers");
    let appearance: Appearance = load(Appearance::SECTION);
    assert_eq!(
        appearance.accent,
        AccentColor::Pink,
        "an earlier default is not a user change"
    );
    assert_eq!(appearance.transparency, 0.33, "a real user change stays");
    let user_text = String::from_utf8(read(Appearance::SECTION)).unwrap_or_default();
    assert!(user_text.starts_with(USER_FILE_HEADER));
    assert!(!user_text.contains("accent") && !user_text.contains("bar_height"));
    assert!(user_text.contains("transparency"));
    let kept_copy = user_file.with_extension("ron.before-layers");
    assert_eq!(
        std::fs::read_to_string(&kept_copy).unwrap_or_default(),
        to_ron(&written_in_full),
        "the file as it was stays beside it"
    );
    let reduced_once = read(Appearance::SECTION);
    let _: Appearance = load(Appearance::SECTION);
    assert_eq!(
        read(Appearance::SECTION),
        reduced_once,
        "a second read changes nothing"
    );

    let hand_written = "// my own file\n(accent: Pink, transparency: 0.5)\n";
    std::fs::write(&user_file, hand_written).expect("write the user's own file");
    let appearance: Appearance = load(Appearance::SECTION);
    assert_eq!(appearance.transparency, 0.5);
    assert_eq!(
        std::fs::read_to_string(&user_file).unwrap_or_default(),
        hand_written,
        "a file that the user writes after the reduction is never rewritten by a read"
    );

    write_keys(Appearance::SECTION, [("blur_passes", "9")]).expect("a tool sets a key");
    let before_tool = Appearance::default().blur_passes.to_string();
    write_keys(Appearance::SECTION, [("blur_passes", before_tool.as_str())])
        .expect("the tool sets the key back");
    let user_text = String::from_utf8(read(Appearance::SECTION)).unwrap_or_default();
    assert!(
        !user_text.contains("blur_passes"),
        "a key that an edit sets back to its default leaves the file: {user_text}"
    );

    reset_keys(Appearance::SECTION, &["transparency"]).expect("reset transparency");
    let appearance: Appearance = load(Appearance::SECTION);
    assert_eq!(appearance.transparency, Appearance::default().transparency);

    for file in [
        distro,
        root.join("machine/appearance.ron"),
        user_file,
        kept_copy,
    ] {
        std::fs::remove_file(file).expect("remove layered test file");
    }
}

fn check_save_load_round_trip() {
    let appearance = Appearance {
        dark_mode: false,
        accent: AccentColor::Orange,
        transparency: 0.42,
        wallpaper: "/usr/share/backgrounds/crown.png".to_owned(),
        bar_height: 40,
        ..Appearance::default()
    };
    save(Appearance::SECTION, &appearance).expect("save appearance");

    let round_tripped: Appearance = load(Appearance::SECTION);
    assert_eq!(round_tripped, appearance);
}

/// `save` records the hash of the merged text that the watcher reads after
/// the write, so the watcher sees a match and stays quiet.
fn check_save_records_its_own_hash() {
    let merged = merged_section_text(Appearance::SECTION).unwrap_or_default();
    assert_eq!(
        last_written(Appearance::SECTION),
        Some(hash_bytes(merged.as_bytes())),
        "save() must record the hash of the merged text"
    );
}

/// An external edit changes the bytes, so the recorded hash no longer matches
/// and the change is *not* suppressed.
fn check_external_edit_breaks_the_hash() {
    write(Appearance::SECTION, b"(dark_mode: true)");

    let on_disk = read(Appearance::SECTION);
    assert_ne!(
        last_written(Appearance::SECTION),
        Some(hash_bytes(&on_disk))
    );
}

/// An unparseable file falls back to the default without clobbering it.
fn check_unparseable_file_is_not_clobbered() {
    let garbage = b"this is not RON at all";
    write(Wifi::SECTION, garbage);

    let broken: Wifi = load(Wifi::SECTION);
    assert_eq!(broken, Wifi::default());
    assert_eq!(
        read(Wifi::SECTION),
        garbage,
        "a parse failure must not overwrite the user's file"
    );
}

/// The watcher delivers external edits, but not our own writes.
fn check_watcher() {
    let (tx, rx) = channel();
    let subscription = subscribe_typed::<Sound, _>(Sound::SECTION, move |sound| {
        let _ = tx.send(sound);
    });

    check_own_save_is_suppressed(&rx);
    check_partial_write_is_dropped(&rx);
    check_external_write_is_delivered(&rx);
    check_identical_external_writes_deliver_once(&rx);
    check_external_write_equal_to_old_save_is_delivered(&rx);
    check_dropped_subscription_stops_delivering(subscription, &rx);
}

fn own_sound() -> Sound {
    Sound {
        output_volume: 11.0,
        ..Sound::default()
    }
}

fn check_own_save_is_suppressed(rx: &Receiver<Sound>) {
    let mine = own_sound();
    save(Sound::SECTION, &mine).expect("save sound");

    assert!(
        rx.recv_timeout(SILENT).is_err(),
        "save() should not notify this process's own subscribers"
    );
}

/// A half-written file must not fire the typed callback.
fn check_partial_write_is_dropped(rx: &Receiver<Sound>) {
    write(Sound::SECTION, b"(output_volume: 9");

    assert!(
        rx.recv_timeout(SILENT).is_err(),
        "unparseable contents must not reach a typed subscriber"
    );
}

/// A write that did not go through `save` must be delivered.
fn check_external_write_is_delivered(rx: &Receiver<Sound>) {
    let theirs = Sound {
        output_volume: 77.0,
        ..Sound::default()
    };
    write(Sound::SECTION, to_ron(&theirs).as_bytes());

    let received = rx
        .recv_timeout(DELIVERED)
        .expect("watcher should deliver an external edit");
    assert_eq!(received, theirs);
}

/// The same bytes written twice in a row reach the subscriber once.
fn check_identical_external_writes_deliver_once(rx: &Receiver<Sound>) {
    let bytes = read(Sound::SECTION);
    write(Sound::SECTION, &bytes);

    assert!(
        rx.recv_timeout(SILENT).is_err(),
        "identical bytes must not be delivered twice"
    );
}

/// After a different external write was delivered, an external write that
/// equals this process's earlier save is a real change and must be delivered.
fn check_external_write_equal_to_old_save_is_delivered(rx: &Receiver<Sound>) {
    let mine = own_sound();
    write(Sound::SECTION, to_ron(&mine).as_bytes());

    let received = rx
        .recv_timeout(DELIVERED)
        .expect("bytes equal to an old save must be delivered after another write");
    assert_eq!(received, mine);
}

fn check_dropped_subscription_stops_delivering(subscription: Subscription, rx: &Receiver<Sound>) {
    drop(subscription);
    write(Sound::SECTION, b"(output_volume: 1.0)");

    assert!(
        rx.recv_timeout(SILENT).is_err(),
        "a dropped Subscription must stop delivering"
    );
}

/// A key subscription fires for its own key only, not for its neighbours in
/// the same section.
fn check_key_watcher() {
    let (tx, rx) = channel();
    let subscription = subscribe_key(schema::appearance::BarHeight, move |bar_height| {
        let _ = tx.send(bar_height);
    });

    let mut appearance = Appearance {
        bar_height: 40,
        ..Appearance::default()
    };
    write(Appearance::SECTION, to_ron(&appearance).as_bytes());
    assert_eq!(
        rx.recv_timeout(DELIVERED)
            .expect("the first change should always be delivered"),
        40
    );

    // Same key, different section: a neighbour changed, so the file's bytes
    // changed, but this subscriber's value did not.
    appearance.transparency = 0.75;
    write(Appearance::SECTION, to_ron(&appearance).as_bytes());
    assert!(
        rx.recv_timeout(SILENT).is_err(),
        "a change to another key must not reach a key subscriber"
    );

    appearance.bar_height = 48;
    write(Appearance::SECTION, to_ron(&appearance).as_bytes());
    assert_eq!(
        rx.recv_timeout(DELIVERED)
            .expect("a change to the watched key should be delivered"),
        48
    );

    drop(subscription);
    appearance.bar_height = 24;
    write(Appearance::SECTION, to_ron(&appearance).as_bytes());
    assert!(
        rx.recv_timeout(SILENT).is_err(),
        "a dropped Subscription must stop delivering"
    );
}

/// Write a section's file directly, bypassing `save` — i.e. what another app
/// or `$EDITOR` would do.
fn write(section: &str, bytes: &[u8]) {
    std::fs::write(path_for(section), bytes).unwrap_or_else(|e| panic!("write {section}: {e}"));
}

fn read(section: &str) -> Vec<u8> {
    std::fs::read(path_for(section)).unwrap_or_else(|e| panic!("read {section}: {e}"))
}

fn to_ron<T: serde::Serialize>(value: &T) -> String {
    ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default()).expect("serialise")
}
