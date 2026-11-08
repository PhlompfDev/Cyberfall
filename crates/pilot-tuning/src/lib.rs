//! Loads tuning/pilot.toml.

use std::fs;
use std::path::Path;

use pilot_core::Tuning;

pub fn parse(text: &str) -> Result<Tuning, toml::de::Error> {
    toml::from_str(text)
}

pub fn load(path: &Path) -> Result<Tuning, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}
