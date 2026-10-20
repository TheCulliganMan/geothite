use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RgbColor(pub [u8; 3]);

impl From<RgbColor> for Color {
    fn from(value: RgbColor) -> Self {
        let [red, green, blue] = value.0;
        Self::Rgb(red, green, blue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TuiTheme {
    pub background: RgbColor,
    pub foreground: RgbColor,
    pub muted: RgbColor,
    pub border: RgbColor,
    pub accent: RgbColor,
    pub selected_background: RgbColor,
    pub selected_foreground: RgbColor,
    pub danger: RgbColor,
    pub water: RgbColor,
    pub grass: RgbColor,
}

impl Default for TuiTheme {
    fn default() -> Self {
        Self {
            background: RgbColor([3, 3, 18]),
            foreground: RgbColor([231, 235, 255]),
            muted: RgbColor([137, 148, 181]),
            border: RgbColor([72, 202, 228]),
            accent: RgbColor([255, 205, 73]),
            selected_background: RgbColor([53, 91, 170]),
            selected_foreground: RgbColor([255, 255, 255]),
            danger: RgbColor([255, 107, 129]),
            water: RgbColor([84, 170, 255]),
            grass: RgbColor([99, 214, 128]),
        }
    }
}

impl TuiTheme {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).with_context(|| format!("read TUI theme {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("parse TUI theme {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_theme_is_exact_and_valid() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../modpacks/text-tui/theme.json");
        let theme = TuiTheme::from_path(path).expect("bundled theme");
        assert_eq!(theme.background, RgbColor([3, 3, 18]));
        assert_ne!(theme.foreground, theme.background);
    }
}
