//! Exact full-region proof through the production bounded-page transport.
use super::*;

fn same_bytes(label: &str, expected: &[u8], actual: &[u8]) {
    assert!(
        actual == expected,
        "{label}: lengths {} / {}, first different byte {:?}",
        actual.len(),
        expected.len(),
        actual.iter().zip(expected).position(|(a, b)| a != b)
    );
}

pub(super) fn verify(
    job: u64,
    map: &str,
    frame: &crystal_render_api::VisualWorldFrame,
    submitted: &SubmittedRequest,
    direct: crate::BuiltTerrain,
) {
    let (_, mut bindings) = request::Frame::capture(frame);
    // Keep the independent borrowed capture as the oracle, but never assemble
    // an oversized single message. Drop its source meshes before worker build.
    let expected = response::Response::capture(job, &direct, frame, &mut bindings).unwrap();
    drop(direct);
    let (expected_header, mut expected_pages) = blobs::encode_header(job, &expected).unwrap();
    let started = std::time::Instant::now();
    let mut producer = begin_paged_cpu_job(&submitted.bytes, false).unwrap();
    let worker_ms = started.elapsed().as_secs_f64() * 1000.;
    same_bytes(map, &expected_header, producer.header());
    assert!(PagedTerrainRestore::from_header(job + 1, producer.header(), submitted).is_err());
    let mut receiver = PagedTerrainRestore::from_header(job, producer.header(), submitted).unwrap();
    let mut pages = 0usize;
    let mut turns = 0usize;
    let mut max_append_ms = 0f64;
    let mut max_restore_ms = 0f64;
    loop {
        // Exercise production's two-page backpressure and bounded restoration.
        for _ in 0..2 {
            let actual = producer.next_page().unwrap();
            let oracle = expected_pages.next_page().unwrap();
            assert_eq!(
                actual.is_some(),
                oracle.is_some(),
                "{map}: page availability"
            );
            if let (Some(actual), Some(oracle)) = (actual, oracle) {
                assert!(actual.len() <= blobs::PAGE_BYTES + 32);
                same_bytes(map, &oracle, &actual);
                let started = std::time::Instant::now();
                let ack = receiver.append_page(&actual).unwrap();
                max_append_ms = max_append_ms.max(started.elapsed().as_secs_f64() * 1000.);
                producer.acknowledge(ack.0, ack.1).unwrap();
                expected_pages.acknowledge(job, ack.0, ack.1).unwrap();
                pages += 1;
            }
        }
        let started = bevy::platform::time::Instant::now();
        let complete = receiver
            .advance(started + std::time::Duration::from_millis(4), 8192)
            .unwrap();
        max_restore_ms = max_restore_ms.max(started.elapsed().as_secs_f64() * 1000.);
        turns += 1;
        if complete && producer.finished() && expected_pages.finished() {
            break;
        }
        assert!(turns < 1_000_000, "{map}: bounded restore made no progress");
    }
    let raw_bytes = producer.raw_bytes();
    let restored = receiver.finish().unwrap();
    let recaptured =
        response::Response::capture_owned(job, restored, frame, &mut bindings).unwrap();
    let (actual_header, mut actual_pages) = blobs::encode_header(job, &recaptured).unwrap();
    same_bytes(map, &expected_header, &actual_header);
    let (oracle_header, mut oracle_pages) = blobs::encode_header(job, &expected).unwrap();
    same_bytes(map, &expected_header, &oracle_header);
    let mut restored_pages = 0;
    loop {
        let actual = actual_pages.next_page().unwrap();
        let oracle = oracle_pages.next_page().unwrap();
        assert_eq!(
            actual.is_some(),
            oracle.is_some(),
            "{map}: restored page availability"
        );
        let (Some(actual), Some(oracle)) = (actual, oracle) else {
            break;
        };
        same_bytes(map, &oracle, &actual);
        let buffer = u32::from_le_bytes(actual[16..20].try_into().unwrap());
        let offset = u64::from_le_bytes(actual[20..28].try_into().unwrap());
        actual_pages.acknowledge(job, buffer, offset).unwrap();
        oracle_pages.acknowledge(job, buffer, offset).unwrap();
        restored_pages += 1;
    }
    assert!(actual_pages.finished() && oracle_pages.finished());
    assert_eq!(restored_pages, pages, "{map}: retain every restored page");
    eprintln!("terrain_worker_paged map={map} tiles={} request_bytes={} header_bytes={} raw_bytes={raw_bytes} pages={pages} turns={turns} worker_ms={worker_ms:.3} max_append_ms={max_append_ms:.3} max_restore_ms={max_restore_ms:.3}",
        frame.tiles.len(), submitted.bytes.len(), expected_header.len());
}
