//! Explicit sound for playback reviews, with quiet automated captures by default.
use anyhow::{Result, bail};

pub(super) fn muted(
    preview: bool,
    force_mute: bool,
    requested_audio: Option<&str>,
) -> Result<bool> {
    let audible_review = match requested_audio {
        None | Some("0") => false,
        Some("1") => true,
        Some(_) => bail!("CRYSTAL_PREVIEW_AUDIO must be 0 or 1"),
    };
    Ok(force_mute || (preview && !audible_review))
}

#[cfg(test)]
mod tests {
    use super::muted;

    #[test]
    fn ordinary_playback_and_explicit_reviews_have_sound() {
        assert!(!muted(false, false, None).expect("ordinary playback"));
        assert!(!muted(true, false, Some("1")).expect("explicit audible review"));
    }

    #[test]
    fn captures_are_quiet_and_explicit_mute_wins() {
        assert!(muted(true, false, None).expect("default capture"));
        assert!(muted(true, false, Some("0")).expect("quiet capture"));
        for preview in [false, true] {
            assert!(muted(preview, true, Some("1")).expect("explicit mute"));
        }
        assert!(muted(true, false, Some("yes")).is_err());
    }
}
