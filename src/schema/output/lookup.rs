use super::OutputSetting;
use crate::schema::Compositor;

/// The name that `compositor.ron` files a monitor under when no entry has its
/// connector name: `"MAKE MODEL SERIAL"` when the EDID gives a serial, else the
/// connector name.
pub fn output_identity(connector: &str, make: &str, model: &str, serial: Option<&str>) -> String {
    match serial {
        Some(serial) if !serial.is_empty() => format!("{make} {model} {serial}"),
        _ => connector.to_owned(),
    }
}

impl Compositor {
    /// The setting for a monitor: the entry named by its connector, else the
    /// entry named by its identity.
    pub fn output(&self, connector: &str, identity: &str) -> Option<&OutputSetting> {
        self.output_index(connector, identity)
            .and_then(|index| self.outputs.get(index))
    }

    /// The same entry as [`Self::output`]. When no entry matches, this adds
    /// an entry named by `identity` and gives it.
    pub fn output_entry(&mut self, connector: &str, identity: &str) -> &mut OutputSetting {
        let index = match self.output_index(connector, identity) {
            Some(index) => index,
            None => {
                self.outputs.push(OutputSetting {
                    name: identity.to_owned(),
                    ..OutputSetting::default()
                });
                self.outputs.len() - 1
            }
        };
        &mut self.outputs[index]
    }

    fn output_index(&self, connector: &str, identity: &str) -> Option<usize> {
        let index_of = |name: &str| self.outputs.iter().position(|setting| setting.name == name);
        index_of(connector).or_else(|| index_of(identity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONNECTOR: &str = "DP-1";
    const IDENTITY: &str = "Dell Inc. U2720Q 7XK2";

    fn named(name: &str, scale: f64) -> OutputSetting {
        OutputSetting {
            name: name.to_owned(),
            scale: Some(scale),
            ..OutputSetting::default()
        }
    }

    fn identity_listed_before_connector() -> Compositor {
        Compositor {
            outputs: vec![named(IDENTITY, 1.0), named(CONNECTOR, 2.0)],
            ..Compositor::default()
        }
    }

    #[test]
    fn the_identity_has_make_model_and_serial_when_a_serial_exists() {
        assert_eq!(
            output_identity(CONNECTOR, "Dell Inc.", "U2720Q", Some("7XK2")),
            IDENTITY
        );
    }

    #[test]
    fn the_identity_falls_back_to_the_connector_without_a_serial() {
        assert_eq!(
            output_identity(CONNECTOR, "Dell Inc.", "U2720Q", None),
            CONNECTOR
        );
        assert_eq!(
            output_identity(CONNECTOR, "Dell Inc.", "U2720Q", Some("")),
            CONNECTOR
        );
    }

    #[test]
    fn the_connector_entry_wins_over_an_earlier_identity_entry() {
        let compositor = identity_listed_before_connector();
        let found = compositor.output(CONNECTOR, IDENTITY);
        assert_eq!(found.map(|setting| setting.name.as_str()), Some(CONNECTOR));
    }

    #[test]
    fn the_identity_entry_matches_when_no_connector_entry_exists() {
        let compositor = Compositor {
            outputs: vec![named(IDENTITY, 1.0)],
            ..Compositor::default()
        };
        let found = compositor.output("HDMI-A-1", IDENTITY);
        assert_eq!(found.map(|setting| setting.name.as_str()), Some(IDENTITY));
        assert_eq!(compositor.output("HDMI-A-1", "Other 1 2"), None);
    }

    #[test]
    fn the_entry_to_edit_is_the_entry_that_is_read() {
        let mut compositor = identity_listed_before_connector();
        compositor.output_entry(CONNECTOR, IDENTITY).scale = Some(3.0);
        assert_eq!(compositor.outputs[0].scale, Some(1.0));
        assert_eq!(compositor.outputs[1].scale, Some(3.0));
        assert_eq!(compositor.outputs.len(), 2);
    }

    #[test]
    fn an_unknown_monitor_gets_a_new_entry_named_by_its_identity() {
        let mut compositor = Compositor::default();
        compositor.output_entry(CONNECTOR, IDENTITY).bar = Some(false);
        compositor.output_entry(CONNECTOR, IDENTITY).dock = Some(true);
        assert_eq!(compositor.outputs.len(), 1);
        assert_eq!(compositor.outputs[0].name, IDENTITY);
        assert_eq!(compositor.outputs[0].bar, Some(false));
        assert_eq!(compositor.outputs[0].dock, Some(true));
    }
}
