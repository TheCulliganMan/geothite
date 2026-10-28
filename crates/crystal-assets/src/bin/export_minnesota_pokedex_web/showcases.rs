//! Native scene showcases alongside the species and battle catalogs.
use std::fs;
use std::path::Path;

use anyhow::{Result, ensure};

pub(super) const NAV_CSS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../web-client/pokedex/state-fair-nav.css"
));
const PAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../web-client/pokedex/state-fair.html"
));
const CSS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../web-client/pokedex/state-fair.css"
));
const JS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../web-client/pokedex/state-fair.js"
));
const MEDIA: &str = "/pokedex/media/showcases/20261027-state-fair-v2";

pub(super) fn write(
    output: &Path,
    asset_base: &str,
    header: &str,
    footer: &str,
    species_count: usize,
) -> Result<()> {
    let page = PAGE
        .replace("{{HEADER}}", header)
        .replace("{{FOOTER}}", footer)
        .replace("{{SPECIES_COUNT}}", &species_count.to_string())
        .replace("{{ASSET_BASE}}", asset_base)
        .replace("{{FAIR_MEDIA}}", MEDIA)
        .replace(
            "href=\"/pokedex/state-fair.html\"",
            "href=\"/pokedex/state-fair.html\" aria-current=\"page\"",
        );
    ensure!(!page.contains("{{"), "unfilled showcase template field");
    fs::write(output.join("state-fair.html"), page)?;
    fs::write(output.join("state-fair.css"), CSS)?;
    fs::write(output.join("state-fair.js"), JS)?;
    Ok(())
}
