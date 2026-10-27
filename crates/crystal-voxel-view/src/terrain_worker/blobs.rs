//! Owned typed buffers and bounded pages for the experimental V3 boundary.
//! Metadata carries ordinals; only raw pages move large geometry to the main
//! Wasm heap. No complete result buffer is copied or decoded on that thread.
use bytemuck::Pod;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{any::Any, cell::RefCell, collections::VecDeque, sync::Arc};

pub(crate) const PAGE_BYTES: usize = 256 * 1024;
// A 216x216 outdoor window includes connected scenery and collision data.
// Its completed buffers can exceed the former 512 MiB scene limit. Paging
// still transfers at most two 256 KiB pages before acknowledgment and releases
// acknowledged source buffers; this is not a monolithic message allocation.
const MAX_BYTES: usize = 768 * 1024 * 1024;
const MAX_BUFFERS: usize = 262_144;
const HEADER_MAGIC: &[u8; 8] = b"GTRNH003";
const PAGE_MAGIC: &[u8; 8] = b"GTRNB003";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Bulk<T: Pod + Send + Sync> {
    Owned(Arc<Vec<T>>),
    Pending { id: u32, count: usize },
}
impl<T: Pod + Send + Sync> From<Vec<T>> for Bulk<T> {
    fn from(values: Vec<T>) -> Self {
        Self::Owned(Arc::new(values))
    }
}
impl<T: Pod + Send + Sync> FromIterator<T> for Bulk<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        values.into_iter().collect::<Vec<_>>().into()
    }
}
impl<T: Pod + Send + Sync> Bulk<T> {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Owned(v) => v.len(),
            Self::Pending { count, .. } => *count,
        }
    }
    pub(crate) fn is_ready(&self) -> bool {
        matches!(self, Self::Owned(_))
    }
    pub(crate) fn as_slice(&self) -> Result<&[T], String> {
        match self {
            Self::Owned(v) => Ok(v),
            Self::Pending { .. } => Err("terrain buffer pages are incomplete".into()),
        }
    }
    pub(crate) fn into_vec(self) -> Result<Vec<T>, String> {
        match self {
            Self::Owned(v) => Arc::try_unwrap(v)
                .map_err(|_| "terrain buffer still has shared CPU ownership".into()),
            Self::Pending { .. } => Err("terrain buffer pages are incomplete".into()),
        }
    }
    pub(crate) fn resolve(&mut self, store: &mut BlobStore) -> Result<bool, String> {
        let Self::Pending { id, count } = *self else {
            return Ok(true);
        };
        let Some(values) = store.take::<T>(id, count)? else {
            return Ok(false);
        };
        *self = values.into();
        Ok(true)
    }
}

trait Source: Send + Sync {
    fn bytes(&self) -> &[u8];
    fn element_size(&self) -> usize;
}
struct TypedSource<T: Pod + Send + Sync>(Arc<Vec<T>>);
impl<T: Pod + Send + Sync> Source for TypedSource<T> {
    fn bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }
    fn element_size(&self) -> usize {
        std::mem::size_of::<T>()
    }
}
#[derive(Default)]
struct Sources {
    buffers: Vec<Option<Box<dyn Source>>>,
    total: usize,
}
impl Sources {
    fn register<T: Pod + Send + Sync>(&mut self, values: Arc<Vec<T>>) -> Result<u32, String> {
        let bytes = values
            .len()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or("terrain buffer length overflow")?;
        self.total = self
            .total
            .checked_add(bytes)
            .ok_or("terrain total buffer length overflow")?;
        if self.total > MAX_BYTES
            || self.buffers.len() >= MAX_BUFFERS
            || std::mem::size_of::<T>() == 0
            || std::mem::size_of::<T>() > PAGE_BYTES
        {
            return Err(format!(
                "terrain buffers exceed protocol bounds: {} bytes, {} buffers, {} byte elements",
                self.total,
                self.buffers.len(),
                std::mem::size_of::<T>(),
            ));
        }
        let id = self.buffers.len() as u32;
        self.buffers.push(Some(Box::new(TypedSource(values))));
        Ok(id)
    }
}

trait Destination: Any + Send + Sync {
    fn append(&mut self, offset: usize, bytes: &[u8]) -> Result<(), String>;
    fn complete(&self) -> bool;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}
struct TypedDestination<T: Pod + Send + Sync> {
    count: usize,
    values: Vec<T>,
}
impl<T: Pod + Send + Sync> Destination for TypedDestination<T> {
    fn append(&mut self, offset: usize, bytes: &[u8]) -> Result<(), String> {
        let size = std::mem::size_of::<T>();
        if bytes.is_empty()
            || bytes.len() > PAGE_BYTES
            || bytes.len() % size != 0
            || offset != self.values.len() * size
            || bytes.len() / size > self.count - self.values.len()
        {
            return Err("invalid terrain blob page offset or length".into());
        }
        // Reserve capacity without initializing/touching the entire buffer.
        // Only the bounded incoming page becomes live typed elements.
        if self.values.capacity() == 0 && self.count != 0 {
            self.values.reserve_exact(self.count);
        }
        if let Ok(values) = bytemuck::try_cast_slice::<u8, T>(bytes) {
            self.values.extend_from_slice(values);
        } else {
            let values = bytemuck::allocation::pod_collect_to_vec::<u8, T>(bytes);
            self.values.extend_from_slice(&values);
        }
        Ok(())
    }
    fn complete(&self) -> bool {
        self.values.len() == self.count
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self.values)
    }
}

#[derive(Default)]
pub(crate) struct BlobStore {
    buffers: Vec<Option<Box<dyn Destination>>>,
    total: usize,
}
impl BlobStore {
    fn declare<T: Pod + Send + Sync>(&mut self, id: u32, count: usize) -> Result<(), String> {
        let size = std::mem::size_of::<T>();
        let bytes = count
            .checked_mul(size)
            .ok_or("terrain declared buffer overflow")?;
        self.total = self
            .total
            .checked_add(bytes)
            .ok_or("terrain declared total overflow")?;
        if size == 0
            || size > PAGE_BYTES
            || self.total > MAX_BYTES
            || id as usize != self.buffers.len()
            || self.buffers.len() >= MAX_BUFFERS
        {
            return Err("invalid terrain buffer declaration ordinal or size".into());
        }
        self.buffers.push(Some(Box::new(TypedDestination::<T> {
            count,
            values: Vec::new(),
        })));
        Ok(())
    }
    pub(crate) fn append(&mut self, id: u32, offset: usize, bytes: &[u8]) -> Result<(), String> {
        self.buffers
            .get_mut(id as usize)
            .and_then(Option::as_mut)
            .ok_or("unknown or consumed terrain buffer")?
            .append(offset, bytes)
    }
    pub(crate) fn completed(&self, id: u32) -> bool {
        self.buffers
            .get(id as usize)
            .and_then(Option::as_ref)
            .is_some_and(|b| b.complete())
    }
    fn take<T: Pod + Send + Sync>(
        &mut self,
        id: u32,
        count: usize,
    ) -> Result<Option<Vec<T>>, String> {
        let slot = self
            .buffers
            .get_mut(id as usize)
            .ok_or("unknown terrain buffer")?;
        let destination = slot.as_ref().ok_or("terrain buffer already consumed")?;
        if !destination.complete() {
            return Ok(None);
        }
        let values = slot
            .take()
            .unwrap()
            .into_any()
            .downcast::<Vec<T>>()
            .map_err(|_| "terrain buffer type mismatch")?;
        if values.len() != count {
            return Err("terrain buffer count mismatch".into());
        }
        Ok(Some(*values))
    }
    pub(crate) fn all_consumed(&self) -> bool {
        self.buffers.iter().all(Option::is_none)
    }
    pub(crate) fn declared_bytes(&self) -> usize {
        self.total
    }
}

thread_local! {
    static ENCODING:RefCell<Option<Sources>>=const{RefCell::new(None)};
    static DECODING:RefCell<Option<BlobStore>>=const{RefCell::new(None)};
}
pub(crate) fn header_encoding_active() -> bool {
    ENCODING.with(|c| c.borrow().is_some())
}
pub(crate) fn header_decoding_active() -> bool {
    DECODING.with(|c| c.borrow().is_some())
}

impl<T: Pod + Send + Sync> Serialize for Bulk<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self::Owned(values) = self else {
            return Err(serde::ser::Error::custom(
                "cannot serialize unresolved terrain buffers",
            ));
        };
        let id = ENCODING
            .with(|context| {
                let mut context = context.borrow_mut();
                context
                    .as_mut()
                    .map(|sources| sources.register(values.clone()))
                    .transpose()
            })
            .map_err(serde::ser::Error::custom)?;
        if let Some(id) = id {
            (id, values.len() as u64).serialize(serializer)
        } else {
            super::packed::serialize(values, serializer)
        }
    }
}
impl<'de, T: Pod + Send + Sync> Deserialize<'de> for Bulk<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if DECODING.with(|c| c.borrow().is_some()) {
            let (id, count) = <(u32, u64)>::deserialize(deserializer)?;
            let count = usize::try_from(count).map_err(serde::de::Error::custom)?;
            DECODING
                .with(|c| c.borrow_mut().as_mut().unwrap().declare::<T>(id, count))
                .map_err(serde::de::Error::custom)?;
            Ok(Self::Pending { id, count })
        } else {
            super::packed::deserialize(deserializer).map(Self::from)
        }
    }
}

pub(crate) fn encode_header<T: Serialize>(
    job: u64,
    value: &T,
) -> Result<(Vec<u8>, PageSession), String> {
    if !cfg!(target_endian = "little") {
        return Err("terrain pages require little-endian targets".into());
    }
    ENCODING.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_some() {
            Err("nested terrain header encoding")
        } else {
            *c = Some(Sources::default());
            Ok(())
        }
    })?;
    let mut bytes = HEADER_MAGIC.to_vec();
    bytes.extend_from_slice(&job.to_le_bytes());
    let encoded = ciborium::into_writer(value, &mut bytes).map_err(|e| e.to_string());
    let sources = ENCODING.with(|c| c.borrow_mut().take().unwrap());
    encoded?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("terrain metadata header exceeds limit".into());
    }
    Ok((
        bytes,
        PageSession {
            job,
            sources,
            cursor: 0,
            offset: 0,
            in_flight: VecDeque::new(),
        },
    ))
}
pub(crate) fn decode_header<T: serde::de::DeserializeOwned>(
    job: u64,
    bytes: &[u8],
) -> Result<(T, BlobStore), String> {
    if !cfg!(target_endian = "little")
        || bytes.len() < 16
        || bytes.len() > 16 * 1024 * 1024
        || &bytes[..8] != HEADER_MAGIC
        || u64::from_le_bytes(bytes[8..16].try_into().unwrap()) != job
    {
        return Err("invalid terrain metadata header/job".into());
    }
    DECODING.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_some() {
            Err("nested terrain header decoding")
        } else {
            *c = Some(BlobStore::default());
            Ok(())
        }
    })?;
    let mut reader = std::io::Cursor::new(&bytes[16..]);
    let decoded = ciborium::from_reader(&mut reader).map_err(|e| e.to_string());
    let store = DECODING.with(|c| c.borrow_mut().take().unwrap());
    let value = decoded?;
    if reader.position() as usize != bytes.len() - 16 {
        return Err("trailing terrain metadata bytes".into());
    }
    Ok((value, store))
}

pub(crate) struct PageSession {
    job: u64,
    sources: Sources,
    cursor: usize,
    offset: usize,
    in_flight: VecDeque<(u32, u64)>,
}
impl PageSession {
    pub(crate) fn next_page(&mut self) -> Result<Option<Vec<u8>>, String> {
        if self.in_flight.len() >= 2 {
            return Ok(None);
        }
        while self.cursor < self.sources.buffers.len() {
            let source = self.sources.buffers[self.cursor].as_ref().unwrap();
            let bytes = source.bytes();
            if self.offset == bytes.len() {
                self.sources.buffers[self.cursor] = None;
                self.cursor += 1;
                self.offset = 0;
                continue;
            }
            let size = source.element_size();
            let len = (bytes.len() - self.offset).min(PAGE_BYTES - PAGE_BYTES % size);
            let mut page = Vec::with_capacity(32 + len);
            page.extend_from_slice(PAGE_MAGIC);
            page.extend_from_slice(&self.job.to_le_bytes());
            page.extend_from_slice(&(self.cursor as u32).to_le_bytes());
            page.extend_from_slice(&(self.offset as u64).to_le_bytes());
            page.extend_from_slice(&(len as u32).to_le_bytes());
            page.extend_from_slice(&bytes[self.offset..self.offset + len]);
            self.in_flight
                .push_back((self.cursor as u32, self.offset as u64));
            self.offset += len;
            return Ok(Some(page));
        }
        Ok(None)
    }
    pub(crate) fn acknowledge(&mut self, job: u64, id: u32, offset: u64) -> Result<(), String> {
        if job != self.job {
            return Err("stale terrain page acknowledgement".into());
        }
        let Some(i) = self.in_flight.iter().position(|p| *p == (id, offset)) else {
            return Err("unknown terrain page acknowledgement".into());
        };
        self.in_flight.remove(i);
        Ok(())
    }
    pub(crate) fn finished(&self) -> bool {
        self.cursor == self.sources.buffers.len() && self.in_flight.is_empty()
    }
    pub(crate) fn raw_bytes(&self) -> usize {
        self.sources.total
    }
}

pub(crate) fn append_packet(
    store: &mut BlobStore,
    job: u64,
    page: &[u8],
) -> Result<(u32, u64), String> {
    if page.len() < 32
        || &page[..8] != PAGE_MAGIC
        || u64::from_le_bytes(page[8..16].try_into().unwrap()) != job
    {
        return Err("invalid terrain page header/job".into());
    }
    let id = u32::from_le_bytes(page[16..20].try_into().unwrap());
    let offset = u64::from_le_bytes(page[20..28].try_into().unwrap());
    let len = u32::from_le_bytes(page[28..32].try_into().unwrap()) as usize;
    if len != page.len() - 32 {
        return Err("terrain page byte count mismatch".into());
    }
    store.append(
        id,
        usize::try_from(offset).map_err(|_| "terrain page offset overflow")?,
        &page[32..],
    )?;
    Ok((id, offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_windows_fit_the_paged_budget_and_oversized_declarations_still_fail() {
        let mut store = BlobStore::default();
        let wide_scene_bytes = 512 * 1024 * 1024 + PAGE_BYTES;
        store
            .declare::<u8>(0, wide_scene_bytes)
            .expect("wide outdoor scene");
        // Declarations retain no initialized bulk allocation until a page is
        // received. They must not reserve the complete advertised scene.
        assert_eq!(store.declared_bytes(), wide_scene_bytes);
        assert!(!store.completed(0));
        assert!(store.declare::<u8>(1, MAX_BYTES).is_err());
    }
    #[derive(Serialize, Deserialize)]
    struct Header {
        positions: Bulk<[u32; 3]>,
        indices: Bulk<u16>,
        empty: Bulk<u64>,
    }
    #[test]
    fn paged_buffers_preserve_bits_offsets_and_two_page_backpressure() {
        let values = (0..100_000)
            .map(|i| [i, 0x80000000, 0x7fc12345])
            .collect::<Vec<_>>();
        let header = Header {
            positions: values.clone().into(),
            indices: vec![0, 1, u16::MAX].into(),
            empty: Vec::new().into(),
        };
        let (bytes, mut session) = encode_header(7, &header).unwrap();
        assert!(bytes.len() < 256);
        let (mut decoded, mut store) = decode_header::<Header>(7, &bytes).unwrap();
        assert!(!decoded.positions.resolve(&mut store).unwrap());
        let a = session.next_page().unwrap().unwrap();
        let b = session.next_page().unwrap().unwrap();
        assert!(session.next_page().unwrap().is_none());
        assert!(a.len() <= PAGE_BYTES + 32 && b.len() <= PAGE_BYTES + 32);
        assert!(append_packet(&mut store, 8, &a).is_err());
        let ack = append_packet(&mut store, 7, &a).unwrap();
        assert!(append_packet(&mut store, 7, &a).is_err());
        session.acknowledge(7, ack.0, ack.1).unwrap();
        assert!(session.acknowledge(7, ack.0, ack.1).is_err());
        let ack = append_packet(&mut store, 7, &b).unwrap();
        session.acknowledge(7, ack.0, ack.1).unwrap();
        while let Some(page) = session.next_page().unwrap() {
            let ack = append_packet(&mut store, 7, &page).unwrap();
            session.acknowledge(7, ack.0, ack.1).unwrap();
        }
        assert!(session.finished());
        assert!(decoded.positions.resolve(&mut store).unwrap());
        assert!(decoded.indices.resolve(&mut store).unwrap());
        assert!(decoded.empty.resolve(&mut store).unwrap());
        assert_eq!(decoded.positions.into_vec().unwrap(), values);
        assert_eq!(decoded.indices.into_vec().unwrap(), vec![0, 1, u16::MAX]);
        assert!(decoded.empty.into_vec().unwrap().is_empty());
        assert!(store.all_consumed());
    }
    #[test]
    fn shared_owned_buffers_never_silently_copy_on_restore() {
        let a = Bulk::from(vec![1u32; 16]);
        let b = a.clone();
        assert!(a.into_vec().is_err());
        assert_eq!(b.into_vec().unwrap(), vec![1; 16]);
    }
}
