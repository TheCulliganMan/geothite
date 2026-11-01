//! Small metadata records decoded incrementally after their raw pages arrive.
//! The legacy representation remains an ordinary serde sequence.
//! Records contain leaf metadata, with no nested Bulk/context-sensitive values:
//! their CBOR is decoded after the header declaration scope has ended.
use super::blobs::{self, BlobStore, Bulk};
use bevy::platform::time::Instant;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};

const MAX_RECORDS: usize = 262_144;
const MAX_METADATA_BYTES: usize = 16 * 1024 * 1024;
const MAX_RECORD_BYTES: usize = 16 * 1024;

struct BoundedWriter<'a> {
    bytes: &'a mut Vec<u8>,
    limit: usize,
}
impl std::io::Write for BoundedWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(
                "terrain metadata record or total exceeds size bounds",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Records<T> {
    Plain(Vec<T>),
    Pending {
        count: usize,
        bytes: Option<Bulk<u8>>,
        input: Vec<u8>,
        offset: usize,
        decoded: Vec<T>,
    },
}
impl<T> From<Vec<T>> for Records<T> {
    fn from(values: Vec<T>) -> Self {
        Self::Plain(values)
    }
}
impl<T> FromIterator<T> for Records<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        values.into_iter().collect::<Vec<_>>().into()
    }
}
impl<T> Records<T> {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Plain(v) => v.len(),
            Self::Pending { count, .. } => *count,
        }
    }
    pub(crate) fn is_ready(&self) -> bool {
        matches!(self, Self::Plain(_))
    }
    pub(crate) fn as_slice(&self) -> Result<&[T], String> {
        match self {
            Self::Plain(v) => Ok(v),
            Self::Pending { .. } => Err("terrain metadata records are incomplete".into()),
        }
    }
    pub(crate) fn into_vec(self) -> Result<Vec<T>, String> {
        match self {
            Self::Plain(v) => Ok(v),
            Self::Pending { .. } => Err("terrain metadata records are incomplete".into()),
        }
    }
}
impl<T: DeserializeOwned> Records<T> {
    /// Each independently bounded CBOR record consumes one quota unit. This
    /// transfers the completed raw allocation, then keeps its offset across
    /// frames; it never clones the full metadata buffer.
    pub(crate) fn resolve(
        &mut self,
        store: &mut BlobStore,
        deadline: Instant,
        max_elements: usize,
    ) -> Result<bool, String> {
        if self.is_ready() {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        let Self::Pending {
            count,
            bytes,
            input,
            offset,
            decoded,
        } = self
        else {
            unreachable!()
        };
        if let Some(buffer) = bytes {
            if !buffer.resolve(store)? {
                return Ok(false);
            }
            *input = bytes.take().unwrap().into_vec()?;
        }
        if *count > MAX_RECORDS || input.len() > MAX_METADATA_BYTES {
            return Err("terrain metadata count or total exceeds bounds".into());
        }
        for _ in 0..max_elements {
            if decoded.len() == *count {
                break;
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            let prefix_end = offset
                .checked_add(4)
                .ok_or("terrain metadata offset overflow")?;
            let prefix = input
                .get(*offset..prefix_end)
                .ok_or("truncated terrain metadata record length")?;
            let length = u32::from_le_bytes(prefix.try_into().unwrap()) as usize;
            if length == 0 || length > MAX_RECORD_BYTES {
                return Err("terrain metadata record exceeds size bounds".into());
            }
            let end = prefix_end
                .checked_add(length)
                .ok_or("terrain metadata record length overflow")?;
            let record = input
                .get(prefix_end..end)
                .ok_or("truncated terrain metadata record")?;
            let mut reader = std::io::Cursor::new(record);
            let value = ciborium::from_reader(&mut reader)
                .map_err(|e| format!("invalid terrain metadata record: {e}"))?;
            if reader.position() as usize != length {
                return Err("trailing bytes within terrain metadata record".into());
            }
            decoded.push(value);
            *offset = end;
        }
        if decoded.len() != *count {
            return Ok(false);
        }
        if *offset != input.len() {
            return Err("terrain metadata count or trailing bytes mismatch".into());
        }
        let values = std::mem::take(decoded);
        *self = Self::Plain(values);
        Ok(true)
    }
}
impl<T: Serialize> Serialize for Records<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self::Plain(values) = self else {
            return Err(serde::ser::Error::custom(
                "cannot serialize incomplete terrain metadata records",
            ));
        };
        if !blobs::header_encoding_active() {
            return values.serialize(serializer);
        }
        if values.len() > MAX_RECORDS {
            return Err(serde::ser::Error::custom(
                "terrain metadata record count exceeds bounds",
            ));
        }
        let mut bytes = Vec::new();
        let mut max_record_bytes = 0usize;
        for record in values {
            let prefix = bytes.len();
            if prefix > MAX_METADATA_BYTES - 4 {
                return Err(serde::ser::Error::custom(
                    "terrain metadata total exceeds size bounds",
                ));
            }
            bytes.extend_from_slice(&[0; 4]);
            let limit = (prefix + 4 + MAX_RECORD_BYTES).min(MAX_METADATA_BYTES);
            ciborium::into_writer(
                record,
                BoundedWriter {
                    bytes: &mut bytes,
                    limit,
                },
            )
            .map_err(serde::ser::Error::custom)?;
            let length = bytes.len() - prefix - 4;
            if length == 0 || length > MAX_RECORD_BYTES || bytes.len() > MAX_METADATA_BYTES {
                return Err(serde::ser::Error::custom(
                    "terrain metadata record or total exceeds size bounds",
                ));
            }
            max_record_bytes = max_record_bytes.max(length);
            bytes[prefix..prefix + 4].copy_from_slice(&(length as u32).to_le_bytes());
        }
        let _trace = crate::loading_trace::span("worker.metadata_record_sizes", || {
            serde_json::json!({
                "type": std::any::type_name::<T>(), "count": values.len(),
                "max_record_bytes": max_record_bytes, "total_bytes": bytes.len(),
            })
        });
        #[cfg(test)]
        if std::env::var_os("CRYSTAL_TERRAIN_WORKER_HEADER_PROFILE").is_some() {
            eprintln!(
                "terrain_worker_records type={} count={} max_record_bytes={} total_bytes={}",
                std::any::type_name::<T>(),
                values.len(),
                max_record_bytes,
                bytes.len()
            );
        }
        (values.len() as u64, Bulk::from(bytes)).serialize(serializer)
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Records<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if !blobs::header_decoding_active() {
            return Vec::deserialize(deserializer).map(Self::Plain);
        }
        let (count, bytes) = <(u64, Bulk<u8>)>::deserialize(deserializer)?;
        let count = usize::try_from(count).map_err(serde::de::Error::custom)?;
        if count > MAX_RECORDS
            || bytes.len() > MAX_METADATA_BYTES
            || count.saturating_mul(5) > bytes.len()
        {
            return Err(serde::de::Error::custom(
                "terrain metadata count or total exceeds bounds",
            ));
        }
        Ok(Self::Pending {
            count,
            bytes: Some(bytes),
            input: Vec::new(),
            offset: 0,
            decoded: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Record {
        ordinal: u32,
        bits: [u32; 3],
    }
    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(1)
    }
    fn records() -> Vec<Record> {
        (0..10_000)
            .map(|ordinal| Record {
                ordinal,
                bits: [0x80000000, 0x7fc12345, ordinal],
            })
            .collect()
    }
    #[test]
    fn legacy_records_are_byte_identical_to_vec_and_paged_records_yield() {
        let values = records();
        let mut legacy = Vec::new();
        ciborium::into_writer(&values, &mut legacy).unwrap();
        let list = Records::from(values.clone());
        let mut bytes = Vec::new();
        ciborium::into_writer(&list, &mut bytes).unwrap();
        assert_eq!(bytes, legacy);
        let legacy_list: Records<Record> = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(legacy_list.into_vec().unwrap(), values);
        let (header, mut pages) = blobs::encode_header(17, &list).unwrap();
        assert!(header.len() < 128);
        let (mut restored, mut store) =
            blobs::decode_header::<Records<Record>>(17, &header).unwrap();
        assert_eq!(restored.len(), values.len());
        assert!(!restored.resolve(&mut store, deadline(), 1).unwrap());
        while let Some(page) = pages.next_page().unwrap() {
            let (id, offset) = blobs::append_packet(&mut store, 17, &page).unwrap();
            pages.acknowledge(17, id, offset).unwrap();
        }
        assert!(
            !restored
                .resolve(&mut store, Instant::now(), usize::MAX)
                .unwrap()
        );
        assert!(!restored.resolve(&mut store, deadline(), 1).unwrap());
        assert!(restored.as_slice().is_err());
        let mut slices = 1;
        while !restored.resolve(&mut store, deadline(), 127).unwrap() {
            slices += 1;
        }
        assert!(slices > 70);
        assert!(store.all_consumed());
        assert_eq!(restored.into_vec().unwrap(), values);
    }
    fn pending(count: usize, input: Vec<u8>) -> Records<Record> {
        Records::Pending {
            count,
            bytes: None,
            input,
            offset: 0,
            decoded: Vec::new(),
        }
    }
    #[test]
    fn metadata_rejects_bad_lengths_counts_trailing_and_oversized_records() {
        let mut store = BlobStore::default();
        for bytes in [
            vec![0, 0, 0],
            vec![0, 0, 0, 0],
            vec![10, 0, 0, 0, 0],
            ((MAX_RECORD_BYTES as u32 + 1).to_le_bytes()).to_vec(),
        ] {
            assert!(
                pending(1, bytes)
                    .resolve(&mut store, deadline(), 1)
                    .is_err()
            );
        }
        assert!(
            pending(0, vec![0])
                .resolve(&mut store, deadline(), 1)
                .is_err()
        );
        let mut record = Vec::new();
        ciborium::into_writer(&records()[0], &mut record).unwrap();
        let mut frame = ((record.len() + 1) as u32).to_le_bytes().to_vec();
        frame.extend(record);
        frame.push(0);
        assert!(
            pending(1, frame)
                .resolve(&mut store, deadline(), 1)
                .is_err()
        );
        let too_many = Records::from(vec![0u8; MAX_RECORDS + 1]);
        assert!(blobs::encode_header(1, &too_many).is_err());
        let oversized = Records::from(vec![vec![0u8; MAX_RECORD_BYTES + 1]]);
        assert!(blobs::encode_header(1, &oversized).is_err());
    }
}
