//! Bundle ignored CC0 people for browser installation, with Rust validation.
use base64::Engine as _;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: bundle_open_people EXTERNAL_CONVERTED_ROOT EXTERNAL_OUTPUT.json".into(),
        );
    }
    let root = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    let mut people = BTreeMap::new();
    for name in crystal_voxel_view::OPEN_PEOPLE {
        let path = root.join(format!("people/kenney/{name}.glb"));
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let mut bytes = Vec::new();
        file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("person exceeds 4 MiB".into());
        }
        people.insert(
            name,
            base64::engine::general_purpose::STANDARD.encode(bytes),
        );
    }
    let json = serde_json::to_string(&serde_json::json!({"version":1,"people":people}))?;
    crystal_voxel_view::install_open_people_bundle(&json)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, &json)?;
    let mut zip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    zip.write_all(json.as_bytes())?;
    let mut gzip = output.into_os_string();
    gzip.push(".gz");
    std::fs::write(PathBuf::from(gzip), zip.finish()?)?;
    Ok(())
}
