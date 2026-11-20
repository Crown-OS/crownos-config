//! `/etc/crownos/setup.ron`: the post-install choices of the user. The
//! installer and the `crownos-setup` runner write this file. The runner,
//! Settings and crownos-computer-use read it.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const SETUP_CONFIG_PATH: &str = "/etc/crownos/setup.ron";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserEngine {
    Chromium,
    Gecko,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageSource {
    Repository,
    Aur,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Browser {
    Brave,
    Chrome,
    #[default]
    Chromium,
    Firefox,
}

#[derive(Debug, thiserror::Error)]
#[error("unknown browser `{0}`")]
pub struct UnknownBrowser(pub String);

impl Browser {
    pub const ALL: [Browser; 4] = [Self::Brave, Self::Chrome, Self::Chromium, Self::Firefox];

    pub fn id(self) -> &'static str {
        match self {
            Self::Brave => "brave",
            Self::Chrome => "chrome",
            Self::Chromium => "chromium",
            Self::Firefox => "firefox",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Brave => "Brave",
            Self::Chrome => "Google Chrome",
            Self::Chromium => "Chromium",
            Self::Firefox => "Firefox",
        }
    }

    pub fn package(self) -> &'static str {
        match self {
            Self::Brave => "brave-bin",
            Self::Chrome => "google-chrome",
            Self::Chromium => "chromium",
            Self::Firefox => "firefox",
        }
    }

    pub fn source(self) -> PackageSource {
        match self {
            Self::Chrome => PackageSource::Aur,
            Self::Brave | Self::Chromium | Self::Firefox => PackageSource::Repository,
        }
    }

    pub fn desktop_id(self) -> &'static str {
        match self {
            Self::Brave => "brave-browser.desktop",
            Self::Chrome => "google-chrome.desktop",
            Self::Chromium => "chromium.desktop",
            Self::Firefox => "firefox.desktop",
        }
    }

    pub fn app_id(self) -> &'static str {
        match self {
            Self::Brave => "brave-browser",
            Self::Chrome => "google-chrome",
            Self::Chromium => "chromium",
            Self::Firefox => "firefox",
        }
    }

    pub fn executable(self) -> &'static str {
        match self {
            Self::Brave => "/usr/bin/brave",
            Self::Chrome => "/usr/bin/google-chrome-stable",
            Self::Chromium => "/usr/bin/chromium",
            Self::Firefox => "/usr/bin/firefox",
        }
    }

    pub fn engine(self) -> BrowserEngine {
        match self {
            Self::Firefox => BrowserEngine::Gecko,
            Self::Brave | Self::Chrome | Self::Chromium => BrowserEngine::Chromium,
        }
    }

    /// The file name of the flags file under `~/.config`, for Chromium-based browsers.
    pub fn flags_file(self) -> Option<&'static str> {
        match self {
            Self::Brave => Some("brave-flags.conf"),
            Self::Chrome => Some("chrome-flags.conf"),
            Self::Chromium => Some("chromium-flags.conf"),
            Self::Firefox => None,
        }
    }

    /// True when the ISO already contains the browser.
    pub fn is_bundled(self) -> bool {
        self == Self::Chromium
    }
}

impl FromStr for Browser {
    type Err = UnknownBrowser;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|browser| browser.id() == id)
            .ok_or_else(|| UnknownBrowser(id.to_owned()))
    }
}

impl fmt::Display for Browser {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeveloperTool {
    ClaudeCode,
    Codex,
    Cursor,
}

#[derive(Debug, thiserror::Error)]
#[error("unknown developer tool `{0}`")]
pub struct UnknownDeveloperTool(pub String);

impl DeveloperTool {
    pub const ALL: [DeveloperTool; 3] = [Self::ClaudeCode, Self::Codex, Self::Cursor];

    pub fn id(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::Cursor => "Cursor",
        }
    }

    pub fn package(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "openai-codex",
            Self::Cursor => "cursor-bin",
        }
    }

    pub fn executable(self) -> &'static str {
        match self {
            Self::ClaudeCode => "/usr/bin/claude",
            Self::Codex => "/usr/bin/codex",
            Self::Cursor => "/usr/bin/cursor",
        }
    }
}

impl FromStr for DeveloperTool {
    type Err = UnknownDeveloperTool;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|tool| tool.id() == id)
            .ok_or_else(|| UnknownDeveloperTool(id.to_owned()))
    }
}

impl fmt::Display for DeveloperTool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeveloperTools {
    pub claude_code: bool,
    pub codex: bool,
    pub cursor: bool,
}

impl DeveloperTools {
    pub fn is_enabled(self, tool: DeveloperTool) -> bool {
        match tool {
            DeveloperTool::ClaudeCode => self.claude_code,
            DeveloperTool::Codex => self.codex,
            DeveloperTool::Cursor => self.cursor,
        }
    }

    pub fn set(&mut self, tool: DeveloperTool, enabled: bool) {
        let flag = match tool {
            DeveloperTool::ClaudeCode => &mut self.claude_code,
            DeveloperTool::Codex => &mut self.codex,
            DeveloperTool::Cursor => &mut self.cursor,
        };
        *flag = enabled;
    }

    pub fn enabled(self) -> impl Iterator<Item = DeveloperTool> {
        DeveloperTool::ALL
            .into_iter()
            .filter(move |&tool| self.is_enabled(tool))
    }

    pub fn any(self) -> bool {
        self.enabled().next().is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SetupConfig {
    pub browser: Browser,
    pub developer_tools: DeveloperTools,
}

#[derive(Debug, thiserror::Error)]
pub enum SetupConfigError {
    #[error("could not read {}: {source}", path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("could not parse {}: {source}", path.display())]
    Parse {
        path: PathBuf,
        source: Box<ron::error::SpannedError>,
    },
}

impl SetupConfig {
    /// The post-install choices, or `None` when the installer did not write them.
    pub fn load() -> Result<Option<Self>, SetupConfigError> {
        Self::load_from(Path::new(SETUP_CONFIG_PATH))
    }

    pub fn load_from(path: &Path) -> Result<Option<Self>, SetupConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse_from(&text, path).map(Some),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(SetupConfigError::Read {
                path: path.to_owned(),
                source,
            }),
        }
    }

    pub fn parse(text: &str) -> Result<Self, SetupConfigError> {
        Self::parse_from(text, Path::new(SETUP_CONFIG_PATH))
    }

    fn parse_from(text: &str, path: &Path) -> Result<Self, SetupConfigError> {
        ron::from_str(text).map_err(|source| SetupConfigError::Parse {
            path: path.to_owned(),
            source: Box::new(source),
        })
    }

    pub fn render(&self) -> String {
        let DeveloperTools {
            claude_code,
            codex,
            cursor,
        } = self.developer_tools;
        format!(
            "(\n    browser: {browser:?},\n    developer_tools: (claude_code: {claude_code}, codex: {codex}, cursor: {cursor}),\n)\n",
            browser = self.browser,
        )
    }

    pub fn wants_playwright(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIREFOX_WITH_CLAUDE_CODE: &str = "(
    browser: Firefox,
    developer_tools: (claude_code: true, codex: false, cursor: false),
)
";

    fn every_tool_combination() -> impl Iterator<Item = DeveloperTools> {
        (0..8u8).map(|bits| DeveloperTools {
            claude_code: bits & 1 != 0,
            codex: bits & 2 != 0,
            cursor: bits & 4 != 0,
        })
    }

    #[test]
    fn render_writes_the_documented_shape() {
        let config = SetupConfig {
            browser: Browser::Firefox,
            developer_tools: DeveloperTools {
                claude_code: true,
                ..DeveloperTools::default()
            },
        };
        assert_eq!(config.render(), FIREFOX_WITH_CLAUDE_CODE);
    }

    #[test]
    fn parse_reads_back_what_render_writes() {
        for browser in Browser::ALL {
            for developer_tools in every_tool_combination() {
                let config = SetupConfig {
                    browser,
                    developer_tools,
                };
                assert_eq!(SetupConfig::parse(&config.render()).expect("valid"), config);
            }
        }
    }

    #[test]
    fn an_empty_record_parses_to_the_default() {
        assert_eq!(
            SetupConfig::parse("()").expect("valid"),
            SetupConfig::default()
        );
    }

    #[test]
    fn a_record_with_only_a_browser_keeps_the_default_tools() {
        let config = SetupConfig::parse("(browser: Brave)").expect("valid");
        assert_eq!(config.browser, Browser::Brave);
        assert_eq!(config.developer_tools, DeveloperTools::default());
    }

    #[test]
    fn a_missing_file_means_no_setup_choices() {
        let missing = Path::new("/nonexistent/crownos/setup.ron");
        assert!(matches!(SetupConfig::load_from(missing), Ok(None)));
    }

    #[test]
    fn from_str_accepts_every_id_and_rejects_others() {
        for browser in Browser::ALL {
            assert_eq!(browser.id().parse::<Browser>().expect("known"), browser);
            assert_eq!(browser.to_string(), browser.id());
        }
        for tool in DeveloperTool::ALL {
            assert_eq!(tool.id().parse::<DeveloperTool>().expect("known"), tool);
        }
        assert!("edge".parse::<Browser>().is_err());
        assert!("edge".parse::<DeveloperTool>().is_err());
    }

    #[test]
    fn every_browser_has_an_executable_in_usr_bin() {
        for browser in Browser::ALL {
            let executable = browser.executable();
            assert!(executable.len() > "/usr/bin/".len(), "{browser}");
            assert!(executable.starts_with("/usr/bin/"), "{browser}");
        }
    }

    #[test]
    fn all_lists_every_browser_and_developer_tool() {
        let mut seen_browsers = [false; Browser::ALL.len()];
        for browser in Browser::ALL {
            let index = match browser {
                Browser::Brave => 0,
                Browser::Chrome => 1,
                Browser::Chromium => 2,
                Browser::Firefox => 3,
            };
            seen_browsers[index] = true;
        }
        assert!(seen_browsers.iter().all(|&listed| listed));

        let mut seen_tools = [false; DeveloperTool::ALL.len()];
        for tool in DeveloperTool::ALL {
            let index = match tool {
                DeveloperTool::ClaudeCode => 0,
                DeveloperTool::Codex => 1,
                DeveloperTool::Cursor => 2,
            };
            seen_tools[index] = true;
        }
        assert!(seen_tools.iter().all(|&listed| listed));
    }

    #[test]
    fn debug_matches_the_ron_variant_name() {
        for browser in Browser::ALL {
            assert_eq!(
                format!("{browser:?}"),
                ron::to_string(&browser).expect("serializable")
            );
        }
    }

    #[test]
    fn browser_table_is_consistent() {
        for browser in Browser::ALL {
            assert_eq!(
                browser.flags_file().is_some(),
                browser.engine() == BrowserEngine::Chromium,
                "{browser}"
            );
            if browser.is_bundled() {
                assert_eq!(browser.source(), PackageSource::Repository, "{browser}");
            }
            assert!(browser.desktop_id().ends_with(".desktop"), "{browser}");
            assert!(
                browser.desktop_id().starts_with(browser.app_id()),
                "{browser}"
            );
        }
    }

    #[test]
    fn developer_tool_display_is_the_id() {
        for tool in DeveloperTool::ALL {
            assert_eq!(tool.to_string(), tool.id());
        }
    }

    #[test]
    fn developer_tools_set_and_list_the_enabled_tools() {
        let mut tools = DeveloperTools::default();
        assert!(!tools.any());
        tools.set(DeveloperTool::Cursor, true);
        tools.set(DeveloperTool::Codex, true);
        tools.set(DeveloperTool::Codex, false);
        assert!(tools.any());
        assert_eq!(tools.enabled().collect::<Vec<_>>(), [DeveloperTool::Cursor]);
    }
}
