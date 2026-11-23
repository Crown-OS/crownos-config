use std::sync::LazyLock;

const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

static BOOT_ID: LazyLock<Option<String>> = LazyLock::new(|| {
    std::fs::read_to_string(BOOT_ID_PATH)
        .ok()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
});

/// The identity a session grant is stored with: the boot id of the kernel.
///
/// It changes at each restart, so a session grant from an earlier boot never
/// applies, even when nobody removed it. The login session id is not used,
/// because user services such as crown-contextd do not share it with the
/// compositor. A new login in the same boot ends session grants through
/// [`ContextAccess::forget_session_grants`](super::ContextAccess::forget_session_grants).
/// `None` when the boot id cannot be read; then no session grant applies.
pub fn current_session() -> Option<&'static str> {
    BOOT_ID.as_deref()
}
