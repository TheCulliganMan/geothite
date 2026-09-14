use super::*;
use base64::Engine as _;
const MAGIC: &[u8] = b"POKECRYSTAL-MIDI-1\0";
fn u32_at(bytes: &[u8], i: usize) -> Result<usize> {
    Ok(u32::from_be_bytes(
        bytes
            .get(i..i + 4)
            .context("truncated MIDI length")?
            .try_into()?,
    ) as usize)
}
fn vlq(bytes: &[u8], offset: &mut usize, end: usize) -> Result<usize> {
    let mut value = 0;
    for _ in 0..4 {
        ensure!(*offset < end, "truncated MIDI VLQ");
        let b = bytes[*offset];
        *offset += 1;
        value = (value << 7) | usize::from(b & 127);
        if b & 128 == 0 {
            return Ok(value);
        }
    }
    bail!("oversized MIDI VLQ")
}
pub fn decode_midi(base64: &str) -> Result<Program> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(base64)?;
    ensure!(bytes.starts_with(b"MThd"), "missing MIDI header");
    let mut chunk = 8 + u32_at(&bytes, 4)?;
    while chunk + 8 <= bytes.len() {
        let start = chunk + 8;
        let end = start
            .checked_add(u32_at(&bytes, chunk + 4)?)
            .context("MIDI overflow")?;
        ensure!(end <= bytes.len(), "truncated MIDI chunk");
        if &bytes[chunk..chunk + 4] == b"MTrk" {
            let mut i = start;
            let mut running = None;
            while i < end {
                vlq(&bytes, &mut i, end)?;
                ensure!(i < end, "truncated MIDI event");
                let status = if bytes[i] < 128 {
                    running.context("MIDI running status missing")?
                } else {
                    let s = bytes[i];
                    i += 1;
                    if s < 240 {
                        running = Some(s);
                    }
                    s
                };
                if status == 255 {
                    ensure!(i < end, "truncated MIDI meta event");
                    let kind = bytes[i];
                    i += 1;
                    let len = vlq(&bytes, &mut i, end)?;
                    ensure!(len <= end - i, "truncated MIDI metadata");
                    let payload = &bytes[i..i + len];
                    if kind == 127 && payload.starts_with(MAGIC) {
                        return Ok(serde_json::from_slice(&payload[MAGIC.len()..])?);
                    }
                    i += len;
                } else if status == 240 || status == 247 {
                    let len = vlq(&bytes, &mut i, end)?;
                    ensure!(len <= end - i, "truncated SysEx");
                    i += len;
                } else {
                    ensure!(status < 240, "unsupported MIDI system event");
                    i += if status & 240 == 192 || status & 240 == 208 {
                        1
                    } else {
                        2
                    };
                    ensure!(i <= end, "truncated MIDI channel event");
                }
            }
        }
        chunk = end;
    }
    bail!("MIDI has no Crystal audio program")
}
fn put_vlq(mut n: usize, out: &mut Vec<u8>) {
    let mut bytes = vec![(n & 127) as u8];
    n >>= 7;
    while n > 0 {
        bytes.push((n & 127) as u8 | 128);
        n >>= 7;
    }
    out.extend(bytes.iter().rev());
}
pub fn encode_midi(program: &Program) -> Result<String> {
    let mut payload = MAGIC.to_vec();
    payload.extend(serde_json::to_vec(program)?);
    let mut track = vec![0, 255, 127];
    put_vlq(payload.len(), &mut track);
    track.extend(payload);
    track.extend([0, 255, 47, 0]);
    let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x03\xc0MTrk".to_vec();
    bytes.extend(u32::try_from(track.len())?.to_be_bytes());
    bytes.extend(track);
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}
#[cfg(all(feature = "browser-synth", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn synthesize_crystal_midi(
    midi: &str,
) -> std::result::Result<js_sys::Object, wasm_bindgen::JsValue> {
    use wasm_bindgen::JsValue;
    let run = || -> Result<js_sys::Object> {
        let rendered = render(&decode_midi(midi)?, SynthContext::cartridge()?)?;
        let samples = rendered.downsample();
        let out = js_sys::Object::new();
        js_sys::Reflect::set(
            &out,
            &JsValue::from_str("samples"),
            &js_sys::Int16Array::from(samples.as_slice()),
        )
        .map_err(|_| anyhow::anyhow!("set PCM samples"))?;
        js_sys::Reflect::set(
            &out,
            &JsValue::from_str("sampleRate"),
            &JsValue::from_f64(22050.0),
        )
        .map_err(|_| anyhow::anyhow!("set sample rate"))?;
        Ok(out)
    };
    run().map_err(|e| JsValue::from_str(&format!("{e:#}")))
}
#[cfg(all(feature = "browser-synth", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn synthesize_modified_cry(midi: &str, request: &str)
    -> std::result::Result<js_sys::Object, wasm_bindgen::JsValue> {
    use wasm_bindgen::JsValue;
    let run = || -> Result<js_sys::Object> {
        let request = serde_json::from_str::<crate::pcm::ModifiedCryRequest>(request)?;
        let pcm = crate::pcm::decode_modified_cry(midi, &request)?;
        let out = js_sys::Object::new();
        js_sys::Reflect::set(&out, &JsValue::from_str("samples"),
            &js_sys::Int16Array::from(pcm.samples.as_ref()))
            .map_err(|_| anyhow::anyhow!("set cry PCM samples"))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("sampleRate"),
            &JsValue::from_f64(f64::from(pcm.format.sample_rate_hz)))
            .map_err(|_| anyhow::anyhow!("set cry sample rate"))?;
        Ok(out)
    };
    run().map_err(|error| JsValue::from_str(&format!("{error:#}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_midi() {
        for bytes in [
            b"MThd".as_slice(),
            b"MThd\0\0\0\x06\0\0\0\x01\x03\xc0MTrk\xff\xff\xff\xff",
        ] {
            assert!(decode_midi(&base64::engine::general_purpose::STANDARD.encode(bytes)).is_err());
        }
    }
}
