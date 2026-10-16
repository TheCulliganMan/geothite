//! Bulk little-endian typed buffers, encoded as one CBOR byte string each.
//!
//! Every shipped native target and wasm32 is little-endian. Fail closed on
//! other targets rather than silently reinterpret their numeric byte order.
use serde::{Deserializer, Serializer, de::Error, ser::Error as _};

pub(crate) fn serialize<T: bytemuck::Pod, S: Serializer>(
    values: &[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if !cfg!(target_endian = "little") {
        return Err(S::Error::custom(
            "terrain bulk buffers require little-endian targets",
        ));
    }
    serializer.serialize_bytes(bytemuck::cast_slice(values))
}

pub(crate) fn deserialize<'de, T: bytemuck::Pod, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<T>, D::Error> {
    struct Visitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: bytemuck::Pod> serde::de::Visitor<'de> for Visitor<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a whole typed terrain buffer encoded as bytes")
        }
        fn visit_bytes<E: Error>(self, bytes: &[u8]) -> Result<Vec<T>, E> {
            if !cfg!(target_endian = "little") || bytes.len() % std::mem::size_of::<T>() != 0 {
                return Err(E::custom("invalid typed terrain byte count or byte order"));
            }
            // Allocate with T's alignment and perform one bulk byte copy. A
            // Vec<u8> cannot safely be reinterpreted with a stronger alignment.
            Ok(bytemuck::allocation::pod_collect_to_vec(bytes))
        }
        fn visit_byte_buf<E: Error>(self, bytes: Vec<u8>) -> Result<Vec<T>, E> {
            self.visit_bytes(&bytes)
        }
    }
    // Ciborium's borrowed-byte path accepts only its small scratch buffer.
    // Owned byte buffers handle complete large mesh payloads in bulk too.
    deserializer.deserialize_byte_buf(Visitor(std::marker::PhantomData))
}

#[cfg(test)]
mod tests {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Buffer(#[serde(with = "super")] Vec<[u32; 3]>);
    #[test]
    fn packed_buffers_preserve_all_bits_and_reject_partial_elements() {
        let values = vec![
            [0, (-0.0f32).to_bits(), 0x7fc12345],
            [u32::MAX, 7, 0x3f123456],
        ];
        let mut bytes = Vec::new();
        ciborium::into_writer(&Buffer(values.clone()), &mut bytes).unwrap();
        // Outer tuple plus one byte-string header; no scalar CBOR elements.
        assert!(bytes.len() <= values.len() * std::mem::size_of::<[u32; 3]>() + 4);
        let restored: Buffer = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(restored.0, values);
        assert_eq!(
            restored.0.as_ptr() as usize % std::mem::align_of::<[u32; 3]>(),
            0
        );
        let mut malformed = Vec::new();
        ciborium::into_writer(&ciborium::Value::Bytes(vec![0; 11]), &mut malformed).unwrap();
        assert!(ciborium::from_reader::<Buffer, _>(malformed.as_slice()).is_err());

        let large = Buffer((0..16_384).map(|i| [i, u32::MAX - i, 0x80000000]).collect());
        let mut bytes = Vec::new();
        ciborium::into_writer(&large, &mut bytes).unwrap();
        let restored: Buffer = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(
            restored.0, large.0,
            "bulk buffer exceeds the CBOR scratch size"
        );
    }
}
