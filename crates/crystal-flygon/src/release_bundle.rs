//! Seal browser assets before inventory creation; no separately remembered step.
use sha2::{Digest, Sha256};
use std::{fs, io, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn rewrite(root: &Path, file: &str, old: &str, new: &str) -> Result<()> {
    let path = root.join(file);
    let source = fs::read_to_string(&path)?;
    // Match the complete quoted URL, never a prefix of e.g. .json.
    let changed = source
        .replace(&format!("\"{old}\""), &format!("\"{new}\""))
        .replace(&format!("'{old}'"), &format!("'{new}'"));
    fs::write(path, changed)?;
    Ok(())
}
fn version_js(root: &Path, stem: &str) -> Result<String> {
    let bytes = fs::read(root.join(format!("{stem}.js")))?;
    let name = format!("{stem}-{}.js", hash(&bytes));
    fs::write(root.join(&name), bytes)?;
    Ok(name)
}
fn gzip(root: &Path, file: &str) -> Result<()> {
    let output = Command::new("gzip")
        .args(["-n", "-9", "-c"])
        .arg(root.join(file))
        .output()?;
    if !output.status.success() {
        return Err(format!("gzip failed for {file}").into());
    }
    fs::write(root.join(format!("{file}.gz")), output.stdout)?;
    Ok(())
}
pub fn seal(root: &Path) -> Result<()> {
    let glue = fs::read_to_string(root.join("flygon/crystal_flygon.js"))?;
    let wasm = fs::read(root.join("flygon/crystal_flygon_bg.wasm"))?;
    let mut combined = glue.as_bytes().to_vec();
    combined.extend_from_slice(&wasm);
    let name = format!("crystal_flygon-{}", hash(&combined));
    let versioned_glue = glue.replace("crystal_flygon_bg.wasm", &format!("{name}.wasm"));
    fs::write(root.join(format!("flygon/{name}.js")), versioned_glue)?;
    fs::write(root.join(format!("flygon/{name}.wasm")), wasm)?;
    for worker in ["flygon-worker", "flygon-view-worker"] {
        rewrite(
            root,
            &format!("{worker}.js"),
            "./flygon/crystal_flygon.js",
            &format!("./flygon/{name}.js"),
        )?;
        rewrite(
            root,
            &format!("{worker}.js"),
            "./flygon/crystal_flygon_bg.wasm",
            &format!("./flygon/{name}.wasm"),
        )?;
        let versioned = version_js(root, worker)?;
        for ui in ["flygon.js", "flygon-view.js"] {
            rewrite(
                root,
                ui,
                &format!("./{worker}.js"),
                &format!("./{versioned}"),
            )?;
        }
    }
    let input = version_js(root, "flygon-input-context")?;
    rewrite(root, "flygon.js", "./flygon-input-context.js", &format!("./{input}"))?;
    let view = version_js(root, "flygon-view")?;
    rewrite(root, "flygon.js", "./flygon-view.js", &format!("./{view}"))?;
    let ui = version_js(root, "flygon")?;
    rewrite(root, "flygon.html", "./flygon.js", &format!("./{ui}"))?;
    // Preserve raw entry points for older clients, but replace every compressed
    // variant too: COPY onto an existing image must never retain stale HTML/JS.
    for dir in [root.to_path_buf(), root.join("flygon")] {
        for file in fs::read_dir(dir)? {
            let path = file?.path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|v| v.to_str()),
                    Some("html" | "js" | "css" | "wasm" | "json")
                )
            {
                let relative = path.strip_prefix(root)?.to_str().ok_or("non-UTF8 asset")?;
                if relative != "flygon-release.json" {
                    gzip(root, relative)?;
                }
            }
        }
    }
    verify_compression(root)
}
pub fn seal_manifest(root: &Path) -> Result<()> {
    gzip(root, "flygon-release.json")
}
pub fn verify_compression(root: &Path) -> Result<()> {
    for required in ["index.html.gz", "flygon.html.gz"] {
        if !root.join(required).is_file() {
            return Err(format!("Missing compressed entry page {required}").into());
        }
    }
    for dir in [root.to_path_buf(), root.join("flygon")] {
        for file in fs::read_dir(dir)? {
            let path = file?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("gz") {
                continue;
            }
            let plain = path.with_extension("");
            let output = Command::new("gzip")
                .args(["-d", "-c"])
                .arg(&path)
                .output()?;
            if !output.status.success() || hash(&output.stdout) != hash(&fs::read(&plain)?) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Stale or corrupt compressed asset {}", path.display()),
                )
                .into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sealing_preserves_json_url_and_replaces_stale_compressed_html() {
        let root = std::env::temp_dir().join(format!("flygon-seal-test-{}", std::process::id()));
        fs::create_dir_all(root.join("flygon")).unwrap();
        for (name, text) in [
            ("index.html", "new game entry"),
            ("flygon-input-context.js", "export const context = 1;"),
            ("flygon.html", "<script src=\"./flygon.js\"></script>"),
            (
                "flygon.js",
                "import './flygon-view.js'; fetch('./flygon-view.json'); new Worker('./flygon-worker.js');",
            ),
            ("flygon-view.js", "new Worker('./flygon-view-worker.js');"),
            (
                "flygon-worker.js",
                "import './flygon/crystal_flygon.js'; fetch('./flygon/crystal_flygon_bg.wasm');",
            ),
            (
                "flygon-view-worker.js",
                "import './flygon/crystal_flygon.js';",
            ),
            (
                "flygon/crystal_flygon.js",
                "fetch('crystal_flygon_bg.wasm');",
            ),
            ("flygon/crystal_flygon_bg.wasm", "test binary"),
            ("flygon-view.json", "{}"),
        ] {
            fs::write(root.join(name), text).unwrap();
        }
        fs::write(root.join("index.html.gz"), b"stale").unwrap();
        seal(&root).unwrap();
        let ui = fs::read_to_string(root.join("flygon.js")).unwrap();
        assert!(ui.contains("fetch('./flygon-view.json')"));
        assert!(!ui.contains("import './flygon-view.js'"));
        assert!(
            !fs::read_to_string(root.join("flygon.html"))
                .unwrap()
                .contains("src=\"./flygon.js\"")
        );
        verify_compression(&root).unwrap();
        fs::write(root.join("index.html"), "changed after compression").unwrap();
        assert!(verify_compression(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
