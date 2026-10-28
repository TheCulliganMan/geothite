//! Small canonical listening exports; synthesis remains in the Rust sound engine.
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};

/// Write 22.05 kHz stereo signed 16-bit PCM as a standard RIFF WAV.
///
/// Returns errors for empty/incomplete stereo frames, a RIFF size overflow,
/// or file I/O failure. Creates the file; its parent directory must exist.
pub fn write_canonical_wav(path: &Path, samples: &[i16]) -> Result<()> {
    ensure!(
        !samples.is_empty() && samples.len().is_multiple_of(2),
        "WAV needs complete stereo frames"
    );
    let size = u32::try_from(samples.len().checked_mul(2).context("PCM size overflow")?)?;
    let mut bytes = Vec::with_capacity(samples.len() * 2 + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(
        &size
            .checked_add(36)
            .context("WAV size overflow")?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&22050_u32.to_le_bytes());
    bytes.extend_from_slice(&88200_u32.to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&size.to_le_bytes());
    bytes.extend(samples.iter().flat_map(|sample| sample.to_le_bytes()));
    fs::write(path, bytes).with_context(|| format!("write listening WAV {}", path.display()))?;
    Ok(())
}
