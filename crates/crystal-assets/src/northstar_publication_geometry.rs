//! Prevent story publication from reviving the retired North Shore layout.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Requires the compact Superior footprints and their reviewed connecting ports.
///
/// Story publishers preserve their parent's outdoor geometry. They must consume
/// the compact canonical geography before adding story content, rather than
/// copying terrain from historical story packs. This check does not change the
/// pack, migrate saves, or establish gameplay/visual acceptance.
///
/// # Errors
/// Rejects missing compact maps, obsolete dimensions, retired map modules, and
/// changed compact ports. Johto/Kanto source content is not modified.
pub fn verify_northstar_publication_geometry(pack: &CompiledGamePack) -> Result<()> {
    for (name, dimensions) in [
        ("MnR11C01R00", (4, 24)),
        ("MnR12C00R00", (32, 24)),
        ("MnR12C00R01", (32, 8)),
        ("MnGrandMarais", (20, 19)),
        ("MnMaraisCedarApproach", (20, 12)),
    ] {
        let map = pack.data.maps.get(name).with_context(|| {
            format!("compact canonical geography required before story publication: missing {name}")
        })?;
        ensure!(
            (map.attributes.width, map.attributes.height) == dimensions,
            "obsolete North Shore/Gunflint geometry in {name}; rebase the story on compact canonical geography before publishing"
        );
    }
    for name in [
        "MnMaraisInlandNeck",
        "MnR12C01R00",
        "MnR12C01R01",
        "MnR12C00R02",
        "MnR12C01R02",
    ] {
        ensure!(
            !pack.data.maps.contains_key(name),
            "retired outdoor module {name} cannot be republished by a story overlay"
        );
    }
    for (name, ports) in [
        (
            "MnR11C01R00",
            vec![
                ("south", "MnR11C01R01", 0),
                ("west", "MnR11C00R00", 0),
                ("east", "MnGrandMarais", 0),
            ],
        ),
        (
            "MnR12C00R00",
            vec![("south", "MnR12C00R01", 0), ("west", "MnBoundaryWaters", 2)],
        ),
        (
            "MnR12C00R01",
            vec![
                ("north", "MnR12C00R00", 0),
                ("east", "MnMaraisCedarApproach", -4),
            ],
        ),
        (
            "MnGrandMarais",
            vec![
                ("north", "MnMaraisCedarApproach", -4),
                ("west", "MnR11C01R00", 0),
            ],
        ),
        (
            "MnMaraisCedarApproach",
            vec![("west", "MnR12C00R01", 4), ("south", "MnGrandMarais", 4)],
        ),
    ] {
        let connections = &pack.data.maps[name].attributes.connections;
        ensure!(
            connections.len() == ports.len()
                && ports
                    .iter()
                    .all(|(direction, target, offset)| connections
                        .iter()
                        .any(|c| c.direction == *direction
                            && c.target_map == *target
                            && c.offset == *offset)),
            "compact connecting ports differ in {name}; historical story geometry must not replace canonical terrain"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify_northstar_publication_geometry;

    #[test]
    #[ignore = "requires frozen compact and historical story packs"]
    fn compact_publication_accepts_canonical_and_rejects_stale_story_or_ports() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs")
            .canonicalize()
            .expect("external pack directory");
        let mut compact = crate::read_verified_compiled_game_pack(
            root.join("minnesota-superior-compact-v03.crystalpack"),
        )
        .expect("compact canonical pack");
        verify_northstar_publication_geometry(&compact).expect("canonical geography");
        let stale = crate::read_verified_compiled_game_pack(
            root.join("minnesota-northstar-story-arc-v8.crystalpack"),
        )
        .expect("historical story pack");
        assert!(verify_northstar_publication_geometry(&stale).is_err());
        compact
            .data
            .maps
            .get_mut("MnR12C00R01")
            .expect("lower Gunflint")
            .attributes
            .connections[1]
            .offset = 0;
        assert!(verify_northstar_publication_geometry(&compact).is_err());
    }
}
