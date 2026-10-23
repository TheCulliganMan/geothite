//! Explicit visual QA readbacks, excluded from ordinary timing campaigns.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use bevy::ecs::system::Commands;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

type Completions = Arc<Mutex<BTreeMap<u64, Option<std::result::Result<(), String>>>>>;

pub(super) struct MotionCapture {
    directory: Option<PathBuf>,
    last_tick: Option<u64>,
    completions: Completions,
}

impl MotionCapture {
    pub fn new(directory: &Path) -> Result<Self> {
        let directory = (std::env::var("CRYSTAL_AUDIT_FRAME_CAPTURE").as_deref() == Ok("1"))
            .then(|| directory.join("motion-frames"));
        if let Some(directory) = &directory {
            std::fs::create_dir(directory)?;
        }
        Ok(Self {
            directory,
            last_tick: None,
            completions: Arc::default(),
        })
    }

    pub fn wants_tick(&self, tick: u64) -> bool {
        self.directory.is_some() && self.last_tick != Some(tick)
    }

    pub fn request(&mut self, id: u64, tick: u64, screenshot: Screenshot, commands: &mut Commands) {
        let Some(directory) = &self.directory else {
            return;
        };
        self.last_tick = Some(tick);
        self.completions
            .lock()
            .expect("motion capture completions")
            .insert(id, None);
        let completions = Arc::clone(&self.completions);
        let path = directory.join(format!("frame-{id:06}.png"));
        commands.spawn(screenshot).observe(
            move |event: bevy::ecs::observer::On<ScreenshotCaptured>| {
                let result = (|| -> Result<()> {
                    event
                        .image
                        .clone()
                        .try_into_dynamic()?
                        .to_rgb8()
                        .save(&path)?;
                    Ok(())
                })()
                .map_err(|error| error.to_string());
                completions
                    .lock()
                    .expect("motion capture completions")
                    .insert(id, Some(result));
            },
        );
    }

    pub fn receipt(&self) -> serde_json::Value {
        let completions = self.completions.lock().expect("motion capture completions");
        serde_json::json!({
            "enabled": self.directory.is_some(),
            "semantics": "visual QA only; request source IDs; readbacks perturb timing; no interpolation",
            "frames": *completions,
        })
    }

    pub fn pending(&self) -> bool {
        self.completions
            .lock()
            .expect("motion capture completions")
            .values()
            .any(Option::is_none)
    }

    pub fn error(&self) -> Option<String> {
        self.completions
            .lock()
            .expect("motion capture completions")
            .values()
            .find_map(|result| {
                result
                    .as_ref()
                    .and_then(|result| result.as_ref().err())
                    .cloned()
            })
    }
}
