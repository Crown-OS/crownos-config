use super::OutputSetting;

/// A shell surface that the user can show or hide on each output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellRole {
    Bar,
    Dock,
    Notifications,
}

impl ShellRole {
    /// Whether an output with no setting for this role shows it. The bar shows
    /// on all outputs; the dock and notifications show only on the primary output.
    pub const fn default_shown(self, primary: bool) -> bool {
        match self {
            Self::Bar => true,
            Self::Dock | Self::Notifications => primary,
        }
    }
}

impl OutputSetting {
    /// Whether this output shows `role`. `primary` is true when this output is
    /// the primary output.
    pub fn shows(&self, role: ShellRole, primary: bool) -> bool {
        self.shell_role_choice(role)
            .unwrap_or_else(|| role.default_shown(primary))
    }

    const fn shell_role_choice(&self, role: ShellRole) -> Option<bool> {
        match role {
            ShellRole::Bar => self.bar,
            ShellRole::Dock => self.dock,
            ShellRole::Notifications => self.notifications,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_ROLES: [ShellRole; 3] = [ShellRole::Bar, ShellRole::Dock, ShellRole::Notifications];

    #[test]
    fn the_bar_shows_on_every_output_by_default() {
        let unset = OutputSetting::default();
        assert!(unset.shows(ShellRole::Bar, true));
        assert!(unset.shows(ShellRole::Bar, false));
    }

    #[test]
    fn the_dock_and_notifications_show_only_on_the_primary_by_default() {
        let unset = OutputSetting::default();
        for role in [ShellRole::Dock, ShellRole::Notifications] {
            assert!(unset.shows(role, true), "{role:?} on the primary");
            assert!(!unset.shows(role, false), "{role:?} on a secondary");
        }
    }

    #[test]
    fn an_unset_setting_agrees_with_the_default_for_a_monitor_without_an_entry() {
        let unset = OutputSetting::default();
        for role in ALL_ROLES {
            for primary in [true, false] {
                assert_eq!(unset.shows(role, primary), role.default_shown(primary));
            }
        }
    }

    #[test]
    fn a_choice_overrides_the_default() {
        let chosen = OutputSetting {
            bar: Some(false),
            dock: Some(true),
            notifications: Some(true),
            ..OutputSetting::default()
        };
        assert!(!chosen.shows(ShellRole::Bar, true));
        assert!(chosen.shows(ShellRole::Dock, false));
        assert!(chosen.shows(ShellRole::Notifications, false));
    }
}
