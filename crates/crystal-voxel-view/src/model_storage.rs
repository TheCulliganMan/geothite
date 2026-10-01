//! Bounded decoding of build-generated gzip data inside each catalog's existing
//! OnceLock loader. Parsed geometry is cached; temporary JSON is dropped.
use serde::{de::DeserializeOwned, Deserialize};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, io::Read};

const FORMAT: &str = "geothite-model-gzip-file-v1";
const MAX_MODEL_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) enum Source<'a> {
    Plain(&'a str),
    Gzip {
        metadata: &'a str,
        payload: &'a [u8],
    },
}
impl<'a> From<&'a str> for Source<'a> {
    fn from(value: &'a str) -> Self {
        Self::Plain(value)
    }
}
impl<'a> From<&'a String> for Source<'a> {
    fn from(value: &'a String) -> Self {
        Self::Plain(value)
    }
}
impl<'a> From<&Source<'a>> for Source<'a> {
    fn from(value: &Source<'a>) -> Self {
        *value
    }
}

macro_rules! include_model {
    ($path:expr) => {
        $crate::model_storage::Source::Gzip {
            metadata: include_str!(concat!(env!("OUT_DIR"), "/", $path, ".meta.json")),
            payload: include_bytes!(concat!(env!("OUT_DIR"), "/", $path, ".gz")),
        }
    };
}
pub(crate) use include_model;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Compressed<'a> {
    storage: &'a str,
    bytes: usize,
    sha256: &'a str,
    gzip_bytes: usize,
    gzip_sha256: &'a str,
}
fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn decode(source: Source<'_>) -> Result<Cow<'_, str>, String> {
    let Source::Gzip { metadata, payload } = source else {
        let Source::Plain(source) = source else {
            unreachable!()
        };
        if source.is_empty() || source.len() > MAX_MODEL_BYTES {
            return Err("plain model exceeds bounded input size".into());
        }
        return Ok(Cow::Borrowed(source));
    };
    let envelope: Compressed<'_> = serde_json::from_str(metadata).map_err(|e| e.to_string())?;
    if envelope.storage != FORMAT
        || envelope.bytes == 0
        || envelope.bytes > MAX_MODEL_BYTES
        || envelope.gzip_bytes == 0
        || envelope.gzip_bytes > MAX_MODEL_BYTES * 2
    {
        return Err("unsupported model storage or bounded size".into());
    }
    if !valid_digest(envelope.sha256) || !valid_digest(envelope.gzip_sha256) {
        return Err("invalid model storage digest".into());
    }
    if payload.len() != envelope.gzip_bytes
        || format!("{:x}", Sha256::digest(payload)) != envelope.gzip_sha256
    {
        return Err("compressed model identity differs".into());
    }
    let mut decoder = flate2::bufread::GzDecoder::new(payload);
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

pub(crate) fn parse<'a, T: DeserializeOwned>(source: impl Into<Source<'a>>) -> Result<T, String> {
    serde_json::from_str(&decode(source.into())?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    include!(concat!(env!("OUT_DIR"), "/model_inventory.rs"));

    #[test]
    fn every_embedded_model_matches_exact_raw_canonical_bytes() {
        assert!(!EMBEDDED_MODELS.is_empty());
        for (relative, source) in EMBEDDED_MODELS {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
            let raw = std::fs::read(path).unwrap();
            assert_eq!(decode(*source).unwrap().as_bytes(), raw, "{relative}");
            let Source::Gzip { payload, .. } = source else {
                panic!("expected build artifact")
            };
            assert_eq!(&payload[4..8], &[0; 4], "no build timestamp: {relative}");
            assert_eq!(payload[9], 255, "platform-independent gzip: {relative}");
        }
    }
    fn packed(bytes: &[u8]) -> (String, Vec<u8>) {
        let mut compressor = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
        compressor.write_all(bytes).unwrap();
        let gzip = compressor.finish().unwrap();
        (metadata(bytes, &gzip), gzip)
    }
    fn metadata(bytes: &[u8], gzip: &[u8]) -> String {
        serde_json::json!({"storage":FORMAT,"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes)),"gzip_bytes":gzip.len(),"gzip_sha256":format!("{:x}",Sha256::digest(gzip))}).to_string()
    }
    fn source<'a>(pair: &'a (String, Vec<u8>)) -> Source<'a> {
        Source::Gzip {
            metadata: &pair.0,
            payload: &pair.1,
        }
    }
    #[test]
    fn binary_gzip_preserves_exact_json_and_float_bits() {
        let original = "{\"values\":[-0.0,1.234567890123456,3],\"label\":\"é🧩\"}\n";
        let stored = packed(original.as_bytes());
        assert_eq!(
            decode(source(&stored)).unwrap().as_bytes(),
            original.as_bytes()
        );
        let a: serde_json::Value = parse(original).unwrap();
        let b: serde_json::Value = parse(source(&stored)).unwrap();
        assert_eq!(a, b);
        assert_eq!(
            b["values"][0].as_f64().unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
    }
    #[test]
    fn plain_test_documents_are_borrowed() {
        assert!(matches!(
            decode(Source::Plain("{\"primitives\":[]}")).unwrap(),
            Cow::Borrowed(_)
        ));
    }
    #[test]
    fn gzip_rejects_bad_metadata_and_compressed_identity() {
        let stored = packed(b"{\"a\":1}");
        let original: serde_json::Value = serde_json::from_str(&stored.0).unwrap();
        for (field, value) in [
            ("storage", serde_json::json!("unknown")),
            ("bytes", serde_json::json!(MAX_MODEL_BYTES + 1)),
            ("bytes", serde_json::json!(0)),
            ("bytes", serde_json::json!(1)),
            ("gzip_bytes", serde_json::json!(1)),
            ("gzip_bytes", serde_json::json!(MAX_MODEL_BYTES * 2 + 1)),
            ("sha256", serde_json::json!("0".repeat(64))),
            ("gzip_sha256", serde_json::json!("0".repeat(64))),
            ("sha256", serde_json::json!("A".repeat(64))),
            ("unknown", serde_json::json!(true)),
        ] {
            let mut bad = original.clone();
            bad[field] = value;
            assert!(
                decode(Source::Gzip {
                    metadata: &bad.to_string(),
                    payload: &stored.1
                })
                .is_err(),
                "{field}"
            );
        }
        let mut bad = stored.clone();
        bad.1[10] ^= 1;
        assert!(decode(source(&bad)).is_err());
    }
    #[test]
    fn gzip_rejects_truncation_trailing_members_and_invalid_utf8() {
        let raw = b"{\"a\":1}";
        let stored = packed(raw);
        for gzip in [
            stored.1[..stored.1.len() - 1].to_vec(),
            [stored.1.clone(), b"trailing".to_vec()].concat(),
            [stored.1.clone(), stored.1.clone()].concat(),
        ] {
            let data = metadata(raw, &gzip);
            assert!(decode(Source::Gzip {
                metadata: &data,
                payload: &gzip
            })
            .is_err());
        }
        assert!(decode(source(&packed(b"\xff"))).is_err());
    }
}
