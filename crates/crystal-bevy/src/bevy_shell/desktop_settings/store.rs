use super::state::{Camera, Preferences};
use std::{
    io::{self, Write},
    path::Path,
};

pub(super) fn load(path: &Path) -> io::Result<Preferences> {
    let bytes = match std::fs::read(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Preferences::default()),
        result => result?,
    };
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut prefs = Preferences::default();
    if let Some(volume) = value["volume"].as_u64() {
        prefs.volume = volume.min(100) as u8;
    }
    if let Some(muted) = value["muted"].as_bool() {
        prefs.muted = muted;
    }
    if let Some(view) = value["view_3d"].as_bool() {
        prefs.view_3d = view;
    }
    if let Some(fullscreen) = value["fullscreen"].as_bool() {
        prefs.fullscreen = fullscreen;
    }
    let camera = &value["camera"];
    prefs.camera = Camera {
        zoom: camera["zoom"]
            .as_f64()
            .map_or(prefs.camera.zoom, |value| value as f32),
        rotation: camera["rotation"]
            .as_f64()
            .map_or(prefs.camera.rotation, |value| value as f32),
        pitch: camera["pitch"]
            .as_f64()
            .map_or(prefs.camera.pitch, |value| value as f32),
        first_person: camera["first_person"].as_bool().unwrap_or(false),
    };
    prefs.sanitize();
    Ok(prefs)
}

pub(super) fn save(path: &Path, prefs: Preferences) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "settings directory"))?;
    std::fs::create_dir_all(parent)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pending = parent.join(format!(".settings-{}-{nonce:x}.tmp", std::process::id()));
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending)?;
    let data = serde_json::json!({ "version": 1, "volume": prefs.volume, "muted": prefs.muted,
    "view_3d": prefs.view_3d, "fullscreen": prefs.fullscreen, "camera": {
        "zoom": prefs.camera.zoom, "rotation": prefs.camera.rotation,
        "pitch": prefs.camera.pitch, "first_person": prefs.camera.first_person,
    }});
    let result = output
        .write_all(data.to_string().as_bytes())
        .and_then(|_| output.sync_all());
    drop(output);
    let result = result.and_then(|_| std::fs::rename(&pending, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&pending);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_settings_round_trip_and_atomic_updates_leave_no_scratch() {
        let directory =
            std::env::temp_dir().join(format!("geothite-settings-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("settings.json");
        let mut prefs = Preferences {
            volume: 30,
            muted: true,
            view_3d: true,
            ..Preferences::default()
        };
        save(&path, prefs).unwrap();
        assert_eq!(load(&path).unwrap(), prefs);
        prefs.volume = 80;
        save(&path, prefs).unwrap();
        assert_eq!(load(&path).unwrap(), prefs);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::write(
            &path,
            br#"{"volume":500,"camera":{"zoom":1e300,"pitch":-1000}}"#,
        )
        .unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.volume, 100);
        assert_eq!(loaded.camera.zoom, 1.0);
        assert_eq!(loaded.camera.pitch, 5.0);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
