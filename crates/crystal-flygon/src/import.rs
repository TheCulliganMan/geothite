//! Offline MaleCNS importer. All retained edges survive; no strength threshold.
use arrow_array::{types::*, *};
use arrow_ipc::reader::FileReader;
use crystal_flygon::{Cell, Metadata};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::Path,
};
const SOURCES: &[(&str, &str)] = &[
    (
        "annotations.feather",
        "2177e246113e4cfbf1e7772ec37c6da1955ff22e8063d0b1f833101f99a9a3b2",
    ),
    (
        "neurotransmitters.feather",
        "95c9289220663abeb3409f3ad9e5a7f8a53f8093f5139d15502cd08da8879621",
    ),
    (
        "edges.feather",
        "e35da783d1c686b2b58b3b87cd6a403ae43bfcfba8bff28e08ef752c1a56afc1",
    ),
];
fn verify_sources(source: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0u8; 1024 * 1024];
    for &(name, expected) in SOURCES {
        let mut file = File::open(source.join(name))?;
        let mut hash = Sha256::new();
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        let actual = format!("{:x}", hash.finalize());
        if actual != expected {
            return Err(format!("Source SHA-256 mismatch for {name}: {actual}").into());
        }
        eprintln!("Verified {name}: {actual}");
    }
    Ok(())
}
fn text(a: &dyn Array, i: usize) -> String {
    if a.is_null(i) {
        return String::new();
    }
    if let Some(x) = a.as_any().downcast_ref::<StringArray>() {
        return x.value(i).into();
    }
    if let Some(x) = a.as_any().downcast_ref::<LargeStringArray>() {
        return x.value(i).into();
    }
    macro_rules! dict {
        ($t:ty) => {
            if let Some(x) = a.as_any().downcast_ref::<DictionaryArray<$t>>() {
                return text(x.values().as_ref(), x.keys().value(i) as usize);
            }
        };
    }
    dict!(Int8Type);
    dict!(Int16Type);
    dict!(Int32Type);
    dict!(Int64Type);
    dict!(UInt8Type);
    dict!(UInt16Type);
    dict!(UInt32Type);
    macro_rules! number {
        ($t:ty) => {
            if let Some(x) = a.as_any().downcast_ref::<PrimitiveArray<$t>>() {
                return x.value(i).to_string();
            }
        };
    }
    number!(UInt64Type);
    number!(Int64Type);
    number!(UInt32Type);
    number!(Int32Type);
    number!(Float32Type);
    number!(Float64Type);
    String::new()
}
fn integer(a: &dyn Array, i: usize) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(text(a, i).parse()?)
}
fn scan(
    path: &Path,
    mut row: impl FnMut(
        &[ArrayRef],
        &HashMap<String, usize>,
        usize,
    ) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = FileReader::try_new(File::open(path)?, None)?;
    eprintln!("{} schema {:?}", path.display(), reader.schema().fields());
    let names = reader
        .schema()
        .fields()
        .iter()
        .enumerate()
        .map(|(i, f)| (f.name().clone(), i))
        .collect();
    for batch in reader {
        let batch = batch?;
        for i in 0..batch.num_rows() {
            row(batch.columns(), &names, i)?;
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(
            "Usage: flygon-import SOURCE_DIR OUTPUT_DIR [--verify-only|--metadata-only]".into(),
        );
    }
    let source = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    verify_sources(source)?;
    if args.get(3).is_some_and(|v| v == "--verify-only") {
        return Ok(());
    }
    fs::create_dir_all(out)?;
    let mut transmitters = HashMap::new();
    scan(&source.join("neurotransmitters.feather"), |a, n, i| {
        transmitters.insert(
            integer(a[n["body"]].as_ref(), i)?,
            text(a[n["consensus_nt"]].as_ref(), i).to_lowercase(),
        );
        Ok(())
    })?;
    let mut cells = Vec::new();
    scan(&source.join("annotations.feather"), |a, n, i| {
        let get = |key: &str| {
            n.get(key)
                .map(|&j| text(a[j].as_ref(), i))
                .unwrap_or_default()
        };
        let class = get("superclass");
        if class.is_empty() || get("status") == "Glia" {
            return Ok(());
        }
        let id = integer(a[n["bodyId"]].as_ref(), i)?;
        let position = n.get("somaLocation").and_then(|&j| {
            let list = a[j].as_any().downcast_ref::<ListArray>()?;
            if list.is_null(i) {
                return None;
            }
            let values = list.value(i);
            let values = values.as_any().downcast_ref::<Int64Array>()?;
            if values.len() != 3 {
                return None;
            }
            Some([
                values.value(0) as f32,
                values.value(1) as f32,
                values.value(2) as f32,
            ])
        });
        cells.push(Cell {
            id: id.to_string(),
            kind: get("type"),
            class,
            side: get("somaSide"),
            transmitter: transmitters.get(&id).cloned().unwrap_or_default(),
            position,
        });
        Ok(())
    })?;
    cells.sort_by_key(|c| c.id.parse::<u64>().unwrap());
    if args.get(3).is_some_and(|v| v == "--metadata-only") {
        let previous: Metadata = serde_json::from_reader(File::open(out.join("metadata.json"))?)?;
        if previous
            .cells
            .iter()
            .map(|c| &c.id)
            .ne(cells.iter().map(|c| &c.id))
        {
            return Err("Neuron identities changed".into());
        }
        serde_json::to_writer(
            BufWriter::new(File::create(out.join("metadata.json"))?),
            &Metadata {
                dataset: previous.dataset,
                cells,
                contacts: previous.contacts,
            },
        )?;
        return Ok(());
    }
    let ids = cells
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.parse::<u64>().unwrap(), i as u32))
        .collect::<HashMap<_, _>>();
    if ids.len() != cells.len() {
        return Err("Duplicate source identities".into());
    }
    let mut edges = Vec::<(u32, u32, u32)>::new();
    let mut contacts = 0u64;
    let mut excluded = 0u64;
    scan(&source.join("edges.feather"), |a, n, i| {
        let pre = integer(a[n["body_pre"]].as_ref(), i)?;
        let post = integer(a[n["body_post"]].as_ref(), i)?;
        let count = integer(a[n["weight"]].as_ref(), i)?;
        if count == 0 || count > u32::MAX as u64 {
            return Err("Invalid synapse count".into());
        }
        if let (Some(&pre), Some(&post)) = (ids.get(&pre), ids.get(&post)) {
            edges.push((pre, post, count as u32));
            contacts += count;
        } else {
            excluded += 1;
        }
        Ok(())
    })?;
    edges.sort_unstable_by_key(|e| (e.0, e.1));
    let mut offsets = vec![0u32; cells.len() + 1];
    for &(pre, _, _) in &edges {
        offsets[pre as usize + 1] += 1;
    }
    for i in 1..offsets.len() {
        offsets[i] += offsets[i - 1];
    }
    let mut w = BufWriter::new(File::create(out.join("graph.bin"))?);
    w.write_all(b"FLYGON01")?;
    for x in [cells.len() as u32, edges.len() as u32] {
        w.write_all(&x.to_le_bytes())?;
    }
    for x in offsets {
        w.write_all(&x.to_le_bytes())?;
    }
    for &(_, post, _) in &edges {
        w.write_all(&post.to_le_bytes())?;
    }
    for &(_, _, count) in &edges {
        w.write_all(&count.to_le_bytes())?;
    }
    w.flush()?;
    let metadata = Metadata {
        dataset: "MaleCNS v1.0; assigned superclass, excluding Glia; all retained pair edges"
            .into(),
        cells,
        contacts,
    };
    serde_json::to_writer(
        BufWriter::new(File::create(out.join("metadata.json"))?),
        &metadata,
    )?;
    eprintln!(
        "Imported {} neurons, {} edges, {} contacts; excluded {} source edge rows",
        metadata.cells.len(),
        edges.len(),
        contacts,
        excluded
    );
    Ok(())
}
