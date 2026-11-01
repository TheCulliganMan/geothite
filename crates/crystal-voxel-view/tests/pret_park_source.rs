// Source-backed fixture coverage; no generated source content is stored here.
#[path = "../src/mesh/park_scenery_source.rs"]
mod park_source;

fn source_cells(
    blocks: &[u8],
    tiles: &[u8],
    block_width: usize,
    block_height: usize,
) -> Vec<park_source::Identity<'static>> {
    assert_eq!(blocks.len(), block_width * block_height);
    let mut cells = Vec::new();
    for y in 0..block_height * 4 {
        for x in 0..block_width * 4 {
            let block = blocks[(y / 4) * block_width + x / 4];
            cells.push(park_source::Identity {
                tileset: "park",
                metatile: u16::from(block),
                column: (x % 4) as u8,
                row: (y % 4) as u8,
                tile: u16::from(tiles[usize::from(block) * 16 + (y % 4) * 4 + x % 4]),
            });
        }
    }
    cells
}

#[test]
fn external_national_park_layout_resolves_every_authored_fixture() {
    let Some(root) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
        eprintln!("external source test skipped: CRYSTAL_PRET_SOURCE_ROOT unset");
        return;
    };
    let root = std::path::Path::new(&root);
    let constants = std::fs::read_to_string(root.join("constants/map_constants.asm")).unwrap();
    for name in ["NATIONAL_PARK", "NATIONAL_PARK_BUG_CONTEST"] {
        let line = constants
            .lines()
            .find(|line| line.trim_start().starts_with(&format!("map_const {name},")))
            .unwrap();
        let args: Vec<_> = line
            .split(';')
            .next()
            .unwrap()
            .split(',')
            .map(str::trim)
            .collect();
        assert_eq!((args[1], args[2]), ("20", "27"));
    }
    let blocks = std::fs::read(root.join("maps/NationalPark.blk")).unwrap();
    let tiles = std::fs::read(root.join("data/tilesets/park_metatiles.bin")).unwrap();
    assert_eq!(blocks.len(), 20 * 27);
    assert_eq!(tiles.len(), 64 * 16);
    let width = 20 * 4;
    let height = 27 * 4;
    let cells = source_cells(&blocks, &tiles, 20, 27);
    let blocked = vec![false; cells.len()];
    let ordinary = park_source::resolve("NationalPark", &cells, width, height, [0, 0], &blocked);
    let contest = park_source::resolve(
        "NationalParkBugContest",
        &cells,
        width,
        height,
        [0, 0],
        &blocked,
    );
    assert_eq!(ordinary.len(), 4);
    assert_eq!(contest.len(), ordinary.len());
    for kind in [
        park_source::Kind::LitterBin,
        park_source::Kind::PedestalFountain,
        park_source::Kind::PondBasin,
    ] {
        let expected = match kind {
            park_source::Kind::LitterBin => 2,
            _ => 1,
        };
        assert_eq!(ordinary.iter().filter(|p| p.kind == kind).count(), expected);
    }
    for (a, b) in ordinary.iter().zip(&contest) {
        assert_eq!((a.column, a.row, a.kind), (b.column, b.row, b.kind));
        assert!(a.coherent(&cells, width, height));
        let mut occupied = blocked.clone();
        occupied[a.guard_indices(width).last().unwrap()] = true;
        let resolved =
            park_source::resolve("NationalPark", &cells, width, height, [0, 0], &occupied);
        assert!(
            !resolved
                .iter()
                .any(|p| (p.column, p.row) == (a.column, a.row)),
            "reserved backing/spray cells must reject the whole fixture"
        );
    }
    let safari_blocks = std::fs::read(root.join("maps/SafariZoneBeta.blk")).unwrap();
    let safari_cells = source_cells(&safari_blocks, &tiles, 10, 18);
    let safari = park_source::resolve(
        "SafariZoneBeta",
        &safari_cells,
        40,
        72,
        [0, 0],
        &vec![false; safari_cells.len()],
    );
    assert_eq!(safari.len(), 3);
    assert_eq!(
        safari
            .iter()
            .filter(|p| p.kind == park_source::Kind::LitterBin)
            .count(),
        1
    );
    assert_eq!(
        safari
            .iter()
            .filter(|p| p.kind == park_source::Kind::PedestalFountain)
            .count(),
        2
    );
    assert!(
        !safari
            .iter()
            .any(|p| p.kind == park_source::Kind::PondBasin),
        "Safari's similar pond source must retain its distinct treatment"
    );
}
