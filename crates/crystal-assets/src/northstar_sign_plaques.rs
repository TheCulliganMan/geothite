//! Readable notices on the Cedar Inlet pine and the Minneapolis Game Corner wall.

use anyhow::{Context, Result, ensure};

use crate::northstar_trail_caches::{self, Landmark};
use crate::{CompiledGamePack, authored_town_publication};

const ART: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/trail-signs-v01/plaques.json");

struct Plaque {
    map: &'static str,
    script: &'static str,
    square: [u16; 2],
    painted_tiles: [usize; 2],
    first_glyph: u8,
}

const PLAQUES: [Plaque; 2] = [
    Plaque {
        map: "MnR08C00R01",
        script: "MN_R08_SIGN_02",
        square: [22, 6],
        painted_tiles: [4, 5],
        first_glyph: 0xc8,
    },
    Plaque {
        map: "MnMinneapolis",
        script: "MnCornerSign",
        square: [55, 51],
        painted_tiles: [10, 11],
        first_glyph: 0xf9,
    },
];

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let art: [Landmark; 2] = serde_json::from_slice(ART)?;
    for (plaque, design) in PLAQUES.iter().zip(&art) {
        let [x, y] = plaque.square;
        let mut room = pack.data.maps.get(plaque.map).context(plaque.map)?.clone();
        ensure!(
            room.attributes.tileset_name == design.tileset
                && room.events.bg_events.iter().any(|e| {
                    e.script == plaque.script
                        && e.event_type == "BGEVENT_READ"
                        && e.x == x
                        && e.y == y
                }),
            "missing printed surface for {}",
            plaque.script
        );
        let index = usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2);
        let block = room.blocks.get_mut(index).context("sign metatile")?;
        ensure!(
            *block == u16::from(design.background)
                && design.collision_from == design.background
                && design.foreground.is_none(),
            "unexpected background, permissions or palette for {}",
            plaque.script
        );
        design.validate_rows()?;
        for i in 0..16 {
            ensure!(
                design.painted_glyph(i, [0; 16]).is_some() == plaque.painted_tiles.contains(&i),
                "notice crossed its mounting surface at {}",
                plaque.script
            );
        }
        let graphics = pack
            .runtime_files
            .get(&format!("gfx/tilesets/{}.2bpp", design.tileset))
            .context("sign tile atlas")?;
        let start = usize::from(plaque.first_glyph) * 16;
        ensure!(
            graphics.len() == 256 * 16 && graphics[start..start + 32].iter().all(|&byte| byte == 0),
            "expected two spare native glyphs for {}",
            plaque.script
        );
        northstar_trail_caches::install_art(
            pack,
            &design.tileset,
            std::slice::from_ref(design),
            plaque.first_glyph,
        )?;
        *block = u16::from(design.block);
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
