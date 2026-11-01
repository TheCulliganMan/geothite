//! Hash the canonical identity serialization without retaining another pack.
use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{self, BufWriter, Write};

struct HashWriter(Sha256);
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

pub(super) fn hash(value: &impl Serialize) -> Result<String> {
    let mut digest = HashWriter(Sha256::new());
    {
        // CBOR emits small headers and integer writes. Batch those updates in
        // fixed storage rather than hashing each byte or retaining the pack.
        let mut writer = BufWriter::with_capacity(16 * 1024, &mut digest);
        ciborium::into_writer(value, &mut writer).context("encode compiled game pack identity")?;
        writer.flush().context("encode compiled game pack identity")?;
    }
    Ok(digest.0.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn streamed_identity_matches_buffered_cbor_for_nested_and_large_payloads() {
        let files = BTreeMap::from([
            ("empty", Vec::new()),
            ("artwork", (0..4 * 1024 * 1024).map(|i| (i % 251) as u8).collect()),
        ]);
        let value = (7_u16, files, [0.0_f32, -0.0, f32::INFINITY], "Unicode Pokémon");
        let mut encoded = Vec::new();
        ciborium::into_writer(&value, &mut encoded).unwrap();
        let expected: String = Sha256::digest(&encoded).iter()
            .map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(hash(&value).unwrap(), expected);
    }
}
