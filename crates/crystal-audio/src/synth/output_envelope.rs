//! Opt-in output mastering for newly authored finite voices.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

/// A final amplitude envelope after the cartridge-style mixer and filters.
/// Existing programs omit it and retain their exact historical PCM output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputEnvelope {
    pub attack_ms: u16,
    pub release_ms: u16,
    /// Linear gain in percent, providing headroom without changing oscillators.
    pub gain_percent: u8,
}

impl OutputEnvelope {
    pub(super) fn apply(&self, samples: &mut [i16], rate: usize) -> Result<()> {
        ensure!(
            self.attack_ms <= 100 && self.release_ms <= 250,
            "output envelope exceeds limits"
        );
        ensure!(
            (1..=100).contains(&self.gain_percent),
            "invalid output gain"
        );
        let frames = samples.len() / 2;
        let attack = (usize::from(self.attack_ms) * rate / 1000).max(1);
        let release = (usize::from(self.release_ms) * rate / 1000).max(1);
        for (frame, stereo) in samples.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let in_gain = (frame as f64 / attack as f64).min(1.0);
            let out_gain = (frames.saturating_sub(frame + 1) as f64 / release as f64).min(1.0);
            let gain = in_gain * out_gain * f64::from(self.gain_percent) / 100.0;
            for sample in stereo {
                *sample = (f64::from(*sample) * gain).round() as i16;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OutputEnvelope;

    #[test]
    fn release_removes_a_filter_tail_and_preserves_stereo_balance() {
        let mut pcm = vec![6000; 8820];
        OutputEnvelope {
            attack_ms: 2,
            release_ms: 45,
            gain_percent: 75,
        }
        .apply(&mut pcm, 44100)
        .expect("master");
        assert_eq!(&pcm[..2], &[0, 0]);
        assert_eq!(&pcm[pcm.len() - 2..], &[0, 0]);
        assert_eq!(&pcm[1000..1002], &[4500, 4500]);
        assert!(
            pcm.as_chunks::<2>()
                .0
                .iter()
                .all(|frame| frame[0] == frame[1])
        );
    }
}
