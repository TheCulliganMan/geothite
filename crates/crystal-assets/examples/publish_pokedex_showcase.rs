//! Add the native fair showcase without regenerating a frozen species catalog.
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

#[path = "../src/bin/export_minnesota_pokedex_web/showcases.rs"]
mod showcases;

const LINK: &str = "<a href=\"/pokedex/state-fair.html\">State Fair</a>";

fn navigation(mut html: String) -> Result<String> {
    if html.contains("href=\"/pokedex/state-fair.html\"") {
        return Ok(html);
    }
    let start = html
        .find("<nav class=\"main-nav\"")
        .context("main navigation missing")?;
    let end = start
        + html[start..]
            .find("</nav>")
            .context("navigation closing tag missing")?;
    html.insert_str(end, LINK);
    Ok(html)
}

fn copy_release(source: &Path, output: &Path, old_base: &str, new_base: &str) -> Result<()> {
    fs::create_dir_all(output)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let target = output.join(entry.file_name());
        ensure!(
            !entry.file_type()?.is_symlink(),
            "release contains a symlink"
        );
        if entry.file_type()?.is_dir() {
            copy_release(&path, &target, old_base, new_base)?;
            continue;
        }
        let bytes = fs::read(&path)?;
        let extension = path.extension().and_then(|value| value.to_str());
        if matches!(extension, Some("html" | "json" | "js" | "css" | "svg")) {
            let mut text = String::from_utf8(bytes)?.replace(old_base, new_base);
            if extension == Some("html") {
                text = navigation(text)?;
            }
            if path.file_name().is_some_and(|name| name == "style.css")
                && !text.contains(showcases::NAV_CSS)
            {
                text.push_str(showcases::NAV_CSS);
            }
            fs::write(target, text)?;
        } else {
            fs::write(target, bytes)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "usage: publish_pokedex_showcase BASE_RELEASE NEW_RELEASE"
    );
    let source = Path::new(&args[0]);
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "release output already exists");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "release must remain ignored"
    );
    let url_base = |path: &Path| -> Result<String> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("release name")?;
        ensure!(
            name.bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "unsafe release name"
        );
        Ok(format!("/pokedex/releases/{name}"))
    };
    let old_base = url_base(source)?;
    let new_base = url_base(output)?;
    copy_release(source, output, &old_base, &new_base)?;
    let index = fs::read_to_string(output.join("index.html"))?;
    let fragment = |start: &str, end: &str| -> Result<&str> {
        let first = index.find(start).context("page fragment missing")?;
        let last = first
            + index[first..]
                .find(end)
                .context("fragment closing tag missing")?
            + end.len();
        Ok(&index[first..last])
    };
    let catalog: Value = serde_json::from_slice(&fs::read(output.join("pokedex.json"))?)?;
    let count = usize::try_from(catalog["species_count"].as_u64().context("species count")?)?;
    showcases::write(
        output,
        &new_base,
        fragment("<header", "</header>")?,
        fragment("<footer", "</footer>")?,
        count,
    )?;
    println!(
        "Added State Fair Chaos to {} with {count} frozen species and unchanged cry URLs",
        output.display()
    );
    Ok(())
}
