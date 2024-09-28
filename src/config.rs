use std::collections::BTreeMap;

#[derive(Default)]
pub struct Config {
    pub show_plugins: bool,
}

impl From<BTreeMap<String, String>> for Config {
    fn from(config: BTreeMap<String, String>) -> Self {
        Self {
            show_plugins: config.get("show_plugins").map(|s| s == "true").unwrap_or(false),
        }
    }
}

