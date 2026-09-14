//! Canonical PCM decoding shared by frontends and audio device adapters.
//! Loop positions count stereo frames, with an exclusive end.
use crate::AudioPcmFormat;
use anyhow::{Context, Result};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DecodedPcmAudio {
    pub bytes: Arc<[u8]>,
    pub samples: Arc<[i16]>,
    pub format: AudioPcmFormat,
    pub loop_range: Option<(usize, usize)>,
}

pub fn decode_pcm(
    bytes: Vec<u8>,
    format: AudioPcmFormat,
    loop_start_sample: Option<usize>,
    loop_end_sample: Option<usize>,
) -> Result<DecodedPcmAudio> {
    if format.sample_rate_hz != 22_050 || format.channels != 2 || format.bits_per_sample != 16 {
        anyhow::bail!("canonical PCM must be 22.05 kHz stereo signed 16-bit data");
    }
    let block_align = usize::from(format.channels) * 2;
    if bytes.is_empty() || bytes.len() % block_align != 0 {
        anyhow::bail!("PCM byte length is not aligned to frame size");
    }
    let frame_count = bytes.len() / block_align;
    let loop_range = match (loop_start_sample, loop_end_sample) {
        (Some(start), Some(end)) if start < end && end <= frame_count => Some((start, end)),
        (None, None) => None,
        (Some(start), Some(end)) => {
            anyhow::bail!("PCM loop range [{start}, {end}) is outside {frame_count} frames")
        }
        _ => anyhow::bail!("PCM source has unpaired loop metadata"),
    };
    Ok(DecodedPcmAudio {
        samples: bytes
            .chunks_exact(2)
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]))
            .collect::<Vec<_>>()
            .into(),
        bytes: bytes.into(),
        format,
        loop_range,
    })
}

pub fn decode_gzip_pcm(
    compressed: &[u8],
    format: AudioPcmFormat,
    byte_len: usize,
    payload_hash: &str,
    loop_start_sample: Option<usize>,
    loop_end_sample: Option<usize>,
) -> Result<DecodedPcmAudio> {
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(compressed);
    let mut decoded = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut decoded).context("decompress PCM audio")?;
    if decoded.len() != byte_len || format!("{:08x}", pcm_fnv1a32(&decoded)) != payload_hash {
        anyhow::bail!("compressed PCM audio failed metadata validation");
    }
    decode_pcm(decoded, format, loop_start_sample, loop_end_sample)
}

pub fn decode_midi_pcm(
    midi_base64: &str,
    format: AudioPcmFormat,
    byte_len: usize,
    payload_hash: &str,
    loop_start_sample: Option<usize>,
    loop_end_sample: Option<usize>,
) -> Result<DecodedPcmAudio> {
    use crate::synth::{SynthContext, decode_midi, render};
    let rendered = render(&decode_midi(midi_base64)?, SynthContext::cartridge()?)?;
    let bytes = rendered
        .downsample()
        .into_iter()
        .flat_map(i16::to_le_bytes)
        .collect::<Vec<_>>();
    if bytes.len() != byte_len || format!("{:08x}", pcm_fnv1a32(&bytes)) != payload_hash {
        anyhow::bail!("synthesized audio failed canonical PCM validation");
    }
    decode_pcm(bytes, format, loop_start_sample, loop_end_sample)
}

/// Cartridge cry register values, separate from host playback speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrySynthesisParameters {
    pub pitch: u16,
    pub length: u16,
}

impl CrySynthesisParameters {
    pub fn slow(pitch: i16, length: i16) -> Self {
        Self {
            pitch: (pitch as u16).wrapping_sub(0x140),
            length: (length as u16).wrapping_add(0x60),
        }
    }
}

/// Metadata verifies the unmodified bundled source before derived synthesis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModifiedCryRequest {
    pub format: AudioPcmFormat,
    pub byte_len: usize,
    pub payload_hash: String,
    pub parameters: CrySynthesisParameters,
}

pub fn decode_modified_cry(midi: &str, request: &ModifiedCryRequest) -> Result<DecodedPcmAudio> {
    use crate::synth::{SynthContext, decode_midi, render};
    let mut program = decode_midi(midi)?;
    anyhow::ensure!(program.cry_pitch.is_some() && program.cry_length.is_some(),
        "modified cry requires a species cry program");
    // Preserve the pack's integrity contract; derived output has a different
    // hash and frame count and must not be compared to the ordinary cry.
    decode_midi_pcm(midi, request.format.clone(), request.byte_len,
        &request.payload_hash, None, None)?;
    program.cry_pitch = Some(i64::from(request.parameters.pitch));
    program.cry_length = Some(i64::from(request.parameters.length));
    let rendered = render(&program, SynthContext::cartridge()?)?;
    anyhow::ensure!(rendered.loop_samples.is_empty(), "modified cry must terminate");
    let bytes = rendered.downsample().into_iter().flat_map(i16::to_le_bytes).collect();
    decode_pcm(bytes, request.format.clone(), None, None)
}

fn pcm_fnv1a32(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c9dc5u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x01000193)
    })
}

/// Decode a packed audio source without opening an audio device.
/// MIDI synthesis is synchronous; a frontend may run this on its audio worker.
pub fn decode_program_source(source: crate::AudioProgramSource) -> Result<DecodedPcmAudio> {
    use crate::AudioProgramSource;
    match source {
        AudioProgramSource::Pcm {
            bytes,
            format,
            loop_start_sample,
            loop_end_sample,
        } => decode_pcm(bytes, format, loop_start_sample, loop_end_sample),
        AudioProgramSource::PcmGzip {
            bytes,
            format,
            byte_len,
            payload_hash,
            loop_start_sample,
            loop_end_sample,
        } => decode_gzip_pcm(
            &bytes,
            format,
            byte_len,
            &payload_hash,
            loop_start_sample,
            loop_end_sample,
        ),
        AudioProgramSource::Midi {
            midi_base64,
            format,
            byte_len,
            payload_hash,
            loop_start_sample,
            loop_end_sample,
        } => decode_midi_pcm(
            &midi_base64,
            format,
            byte_len,
            &payload_hash,
            loop_start_sample,
            loop_end_sample,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn format() -> AudioPcmFormat {
        AudioPcmFormat {
            sample_rate_hz: 22_050,
            channels: 2,
            bits_per_sample: 16,
        }
    }

    #[test]
    fn raw_and_compressed_sources_preserve_samples_and_frame_loop() {
        let bytes = [i16::MIN, i16::MAX, 123, -456]
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect::<Vec<_>>();
        let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gzip.write_all(&bytes).unwrap();
        let compressed = gzip.finish().unwrap();
        let decoded = decode_gzip_pcm(
            &compressed,
            format(),
            bytes.len(),
            &format!("{:08x}", pcm_fnv1a32(&bytes)),
            Some(1),
            Some(2),
        )
        .unwrap();
        assert_eq!(&*decoded.samples, &[i16::MIN, i16::MAX, 123, -456]);
        assert_eq!(&*decoded.bytes, bytes);
        assert_eq!(decoded.loop_range, Some((1, 2)));
        assert!(
            decode_gzip_pcm(&compressed, format(), bytes.len(), "00000000", None, None).is_err()
        );
        assert!(
            decode_gzip_pcm(
                &compressed,
                format(),
                bytes.len() + 4,
                &format!("{:08x}", pcm_fnv1a32(&bytes)),
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_format_alignment_and_loop_metadata() {
        assert!(decode_pcm(vec![0; 3], format(), None, None).is_err());
        assert!(decode_pcm(Vec::new(), format(), None, None).is_err());
        for (start, end) in [(Some(0), None), (Some(1), Some(1)), (Some(0), Some(3))] {
            assert!(decode_pcm(vec![0; 8], format(), start, end).is_err());
        }
        let mut mono = format();
        mono.channels = 1;
        assert!(decode_pcm(vec![0; 8], mono, None, None).is_err());
    }
}
