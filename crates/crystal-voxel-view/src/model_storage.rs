//! Bounded lossless decoding for original model documents, used only inside
//! each catalog's existing OnceLock loader. Parsed meshes/rigs are cached; the
//! temporary decoded JSON is dropped rather than retained beside the mesh.
use base64::Engine;
use serde::{Deserialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, io::Read};

const FORMAT: &str = "geothite-model-gzip-v1";
const MAX_MODEL_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Compressed<'a> {
    storage: &'a str,
    bytes: usize,
    sha256: &'a str,
    data: &'a str,
}

fn decode(source: &str) -> Result<Cow<'_, str>, String> {
    if !source.contains("\"storage\"") {
        return Ok(Cow::Borrowed(source));
    }
    #[derive(Deserialize)]
    struct Probe<'a> {
        storage: Option<&'a str>,
    }
    let probe: Probe<'_> = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if probe.storage.is_none() {
        return Ok(Cow::Borrowed(source));
    }
    let envelope: Compressed<'_> = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if envelope.storage != FORMAT || envelope.bytes == 0 || envelope.bytes > MAX_MODEL_BYTES {
        return Err("unsupported model storage or decoded size".into());
    }
    if envelope.sha256.len() != 64
        || !envelope
            .sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("invalid model storage digest".into());
    }
    if envelope.data.len() > MAX_MODEL_BYTES * 2 {
        return Err("compressed model exceeds bounded input size".into());
    }
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(envelope.data)
        .map_err(|e| e.to_string())?;
    let mut decoder = flate2::bufread::GzDecoder::new(compressed.as_slice());
    let mut decoded = Vec::with_capacity(envelope.bytes);
    decoder
        .by_ref()
        .take(envelope.bytes as u64 + 1)
        .read_to_end(&mut decoded)
        .map_err(|e| e.to_string())?;
    if decoded.len() != envelope.bytes || !decoder.into_inner().is_empty() {
        return Err("model decoded size or gzip boundary differs".into());
    }
    if format!("{:x}", Sha256::digest(&decoded)) != envelope.sha256 {
        return Err("model decoded SHA-256 differs".into());
    }
    String::from_utf8(decoded)
        .map(Cow::Owned)
        .map_err(|e| e.to_string())
}

pub(crate) fn parse<T: DeserializeOwned>(source: &str) -> Result<T, String> {
    serde_json::from_str(&decode(source)?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn packed(bytes: &[u8]) -> String {
        let mut compressor = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
        compressor.write_all(bytes).unwrap();
        serde_json::json!({"storage":FORMAT,"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes)),"data":base64::engine::general_purpose::STANDARD.encode(compressor.finish().unwrap())}).to_string()
    }
    #[test]
    fn model_storage_preserves_exact_json_and_float_bits() {
        let original = "{\"values\":[-0.0,1.234567890123456,3],\"label\":\"é🧩\"}\n";
        assert_eq!(
            decode(&packed(original.as_bytes())).unwrap().as_bytes(),
            original.as_bytes()
        );
        let a: serde_json::Value = parse(original).unwrap();
        let b: serde_json::Value = parse(&packed(original.as_bytes())).unwrap();
        assert_eq!(a, b);
        assert_eq!(
            b["values"][0].as_f64().unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
    }
    #[test]
    fn model_storage_plain_documents_are_borrowed() {
        assert!(matches!(
            decode("{\"primitives\":[]}").unwrap(),
            Cow::Borrowed(_)
        ));
    }
    #[test]
    fn model_storage_rejects_bad_hash_size_encoding_and_format() {
        let source: serde_json::Value = serde_json::from_str(&packed(b"{\"a\":1}")).unwrap();
        for (field, value) in [
            ("storage", serde_json::json!("unknown")),
            ("bytes", serde_json::json!(MAX_MODEL_BYTES + 1)),
            ("bytes", serde_json::json!(1)),
            ("sha256", serde_json::json!("0".repeat(64))),
            ("data", serde_json::json!("not base64")),
        ] {
            let mut bad = source.clone();
            bad[field] = value;
            assert!(decode(&bad.to_string()).is_err(), "{field}");
        }
        let mut bad = source.clone();
        let mut gzip = base64::engine::general_purpose::STANDARD
            .decode(source["data"].as_str().unwrap())
            .unwrap();
        gzip.extend_from_slice(b"trailing");
        bad["data"] = serde_json::json!(base64::engine::general_purpose::STANDARD.encode(gzip));
        assert!(decode(&bad.to_string()).is_err());
    }
}
