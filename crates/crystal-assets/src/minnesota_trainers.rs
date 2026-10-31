//! Additional hand-placed Minnesota opponents on existing native terrain.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerPartyPokemon;
use serde::Deserialize;
use serde_json::json;

use crate::authored_route_design::RouteTrainer;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule, authored_town_publication};

#[path = "minnesota_trainers_geography.rs"]
pub(crate) mod geography;

const MANIFEST: &str = "minnesota-trainer-expansion-v2";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expansion {
    schema_version: u8,
    trainers: Vec<Placement>,
}

#[derive(Deserialize)]
struct Placement {
    map: String,
    approach: [u16; 2],
    #[serde(default)]
    template_class: Option<String>,
    #[serde(default)]
    movement: Option<String>,
    #[serde(default)]
    patrol_size: Option<[u16; 2]>,
    #[serde(default)]
    optional_battle: bool,
    #[serde(default)]
    class_name: Option<String>,
    #[serde(flatten)]
    trainer: RouteTrainer,
}

/// Adds regional trainers and enables level, evolution and move scaling for
/// Minnesota battles. Existing terrain, parties, events and connections survive.
///
/// # Errors
/// Rejects occupied identities, unsafe placements, blocked approaches, missing
/// native species/art, stale pre-compact Superior geometry, malformed programs
/// and invalid packs.
pub fn build_minnesota_trainers_modpack(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    build_trainers(base, source, MANIFEST, true)
}

/// Adds the authored Minneapolis cycling opponents with bounded native patrols.
///
/// # Errors
/// Rejects other maps, non-bicycle art, obstructed patrols, blocked access,
/// malformed encounters and invalid packs. Existing content is preserved.
pub fn build_minneapolis_cyclists_modpack(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    let expansion: Expansion = serde_json::from_str(source)?;
    let original = base
        .data
        .maps
        .get("MnMinneapolis")
        .context("Minneapolis missing")?;
    for placement in &expansion.trainers {
        ensure!(
            placement.map == "MnMinneapolis"
                && matches!(
                    placement.trainer.sprite.as_str(),
                    "SPRITE_CHRIS_BIKE" | "SPRITE_KRIS_BIKE"
                )
                && placement.optional_battle
                && placement.movement.as_deref() == Some("patrol_clockwise"),
            "Minneapolis bicycle riders require optional battles and city loops"
        );
        let [x, y] = placement.trainer.tile;
        let [width, height] = placement.patrol_size.context("bicycle loop dimensions")?;
        ensure!(
            width > 0 && height > 0 && width + height >= 16,
            "city bicycle loop must be at least 32 tiles long"
        );
        let right = x.checked_add(width).context("loop width overflow")?;
        let bottom = y.checked_add(height).context("loop height overflow")?;
        for cy in y..=bottom {
            for cx in x..=right {
                if cx != x && cx != right && cy != y && cy != bottom {
                    continue;
                }
                let tile = [cx, cy];
                ensure!(
                    crate::authored_town_geometry::kind(base, original, tile)? == "FLOOR"
                        && !original.objects.iter().any(|o| [o.x, o.y] == tile)
                        && !original.events.warps.iter().any(|e| [e.x, e.y] == tile)
                        && !original
                            .events
                            .coord_events
                            .iter()
                            .any(|e| [e.x, e.y] == tile)
                        && !original.events.bg_events.iter().any(|e| [e.x, e.y] == tile),
                    "bicycle loop obstructed at {tile:?}"
                );
            }
        }
    }
    build_trainers(
        base,
        source,
        "minneapolis-cyclists-v1",
        base.data.minnesota_trainer_scaling,
    )
}

pub(crate) fn build_trainers(
    base: &CompiledGamePack,
    source: &str,
    manifest: &str,
    enable_scaling: bool,
) -> Result<CompiledGamePack> {
    geography::require_compact_superior(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|manifest| manifest == "northstar-minnesota-only-v1"),
        "bind the parent to the active Minnesota roster before adding trainers"
    );
    let source: Expansion = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1 && !source.trainers.is_empty(),
        "trainer source required"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == manifest),
        "trainers already installed"
    );
    let policy = crate::northstar_regional_roster::RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )?;
    for trainer in base
        .data
        .trainers
        .trainers
        .values()
        .filter(|trainer| trainer.trainer_id.starts_with("MN_"))
    {
        ensure!(
            trainer
                .party
                .iter()
                .all(|member| policy.allowed.contains(&member.species)),
            "existing Minnesota team uses an out-of-region or retired species: {}",
            trainer.trainer_id
        );
    }
    for species in &policy.allowed {
        if let Some(evolutions) = base.data.evolutions.0.get(species) {
            ensure!(
                evolutions
                    .iter()
                    .all(|entry| policy.allowed.contains(&entry.species)),
                "Minnesota evolution leaves the active regional roster: {species}"
            );
        }
    }
    let mut pack = base.clone();
    let mut maps = BTreeMap::new();
    for (index, placement) in source.trainers.iter().enumerate() {
        let t = &placement.trainer;
        ensure!(
            placement.map.starts_with("Mn") && t.id.starts_with("MN_EXTRA_"),
            "Minnesota placement required"
        );
        ensure!(
            (1..=3).contains(&t.sight_range),
            "short, readable trainer sight required"
        );
        let mut map = maps
            .remove(&placement.map)
            .or_else(|| pack.data.maps.get(&placement.map).cloned())
            .context("trainer map missing")?;
        validate_placement(&pack, &map, t.tile, placement.approach)?;
        ensure!(
            !pack.data.trainers.trainers.contains_key(&t.id),
            "trainer identity occupied: {}",
            t.id
        );
        let mut trainer = pack
            .data
            .trainers
            .trainers
            .values()
            .find(|trainer| {
                trainer.trainer_class
                    == placement
                        .template_class
                        .as_deref()
                        .unwrap_or(&t.trainer_class)
            })
            .context("native trainer class missing")?
            .clone();
        let dvs = trainer.party.first().context("native trainer DVs")?.dvs;
        trainer.trainer_id.clone_from(&t.id);
        trainer.trainer_class.clone_from(&t.trainer_class);
        trainer.name = t.display_name.to_ascii_uppercase();
        ensure!(
            !t.party.is_empty() && t.party.len() <= 6,
            "legal party size required"
        );
        trainer.party = t
            .party
            .iter()
            .map(|member| {
                ensure!(
                    (1..=100).contains(&member.level),
                    "legal trainer level required"
                );
                ensure!(
                    policy.allowed.contains(&member.species),
                    "new trainers require original Minnesota species"
                );
                let pokemon = pack
                    .data
                    .create_pokemon(&member.species, member.level, dvs)?;
                Ok(TrainerPartyPokemon {
                    species: member.species.clone(),
                    level: member.level,
                    item: None,
                    moves: pokemon.moves,
                    dvs,
                })
            })
            .collect::<Result<_>>()?;
        trainer.win_quote.clear();
        trainer.lose_quote.clear();
        pack.data.trainers.insert(trainer)?;
        let global = &mut pack.data.story_event_script_constants.global;
        ensure!(
            !global.contains_key(&t.id)
                && !global.contains_key(&t.defeated_flag)
                && t.defeated_flag.starts_with("EVENT_MN_EXTRA_"),
            "trainer constants occupied"
        );
        let next = global
            .iter()
            .filter(|(name, _)| name.starts_with("EVENT_"))
            .map(|(_, value)| *value)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .context("event flag overflow")?;
        global.insert(t.defeated_flag.clone(), next);
        global.insert(t.id.clone(), i64::try_from(index + 1)?);
        let [seen, won, lost, after] =
            ["Seen", "Won", "Lost", "After"].map(|suffix| format!("{}{suffix}", t.id));
        for (label, lines) in [
            (&seen, &t.before_lines),
            (&won, &t.after_lines),
            (&lost, &t.loss_lines),
        ] {
            ensure!(
                !lines.is_empty()
                    && lines
                        .iter()
                        .all(|line| !line.is_empty() && line.chars().count() <= 18),
                "native dialogue required"
            );
            authoring::text(
                &mut map,
                label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
        map.scripts.insert(
            t.id.clone(),
            json!([authoring::command(
                "trainer",
                &[
                    &t.trainer_class,
                    &t.id,
                    &t.defeated_flag,
                    &seen,
                    &won,
                    &lost,
                    &after
                ]
            )]),
        );
        map.scripts.insert(
            after,
            json!([
                authoring::command("endifjustbattled", &[]),
                authoring::command("jumptextfaceplayer", &[&won])
            ]),
        );
        if placement.optional_battle {
            let declined = format!("{}Declined", t.id);
            let c = authoring::command;
            map.scripts.insert(
                t.id.clone(),
                json!([
                    c("faceplayer", &[]),
                    c("checkevent", &[&t.defeated_flag]),
                    c("iftrue", &[&format!("{}After", t.id)]),
                    c("opentext", &[]),
                    c("writetext", &[&seen]),
                    c("yesorno", &[]),
                    c("iffalse", &[&declined]),
                    c("closetext", &[]),
                    c("winlosstext", &[&won, &lost]),
                    c("loadtrainer", &[&t.trainer_class, &t.id]),
                    c("startbattle", &[]),
                    c("reloadmapafterbattle", &[]),
                    c("setevent", &[&t.defeated_flag]),
                    c("end", &[])
                ]),
            );
            map.scripts
                .insert(declined, json!([c("closetext", &[]), c("end", &[])]));
        }
        let sprite = format!(
            "gfx/sprites/{}.png",
            t.sprite.trim_start_matches("SPRITE_").to_ascii_lowercase()
        );
        ensure!(
            pack.runtime_files.contains_key(&sprite),
            "native trainer sprite missing"
        );
        authoring::npc(
            &pack,
            &mut map,
            &t.id,
            &t.sprite,
            i16::try_from(t.tile[0])?,
            i16::try_from(t.tile[1])?,
        )?;
        let object = map.objects.last_mut().context("new trainer object")?;
        object.object_identifier = Some(t.id.clone());
        object.object_type = if placement.optional_battle {
            "OBJECTTYPE_SCRIPT"
        } else {
            "OBJECTTYPE_TRAINER"
        }
        .into();
        object.spritemovedata = t.facing.movement().into();
        match placement.movement.as_deref() {
            None => {}
            Some("walk_x") => {
                object.spritemovedata = "SPRITEMOVEDATA_WALK_LEFT_RIGHT".into();
                object.move_range_x = 2;
            }
            Some("walk_y") => {
                object.spritemovedata = "SPRITEMOVEDATA_WALK_UP_DOWN".into();
                object.move_range_y = 2;
            }
            Some("patrol_clockwise") => {
                let [width, height] = placement.patrol_size.context("patrol dimensions")?;
                object.spritemovedata = "SPRITEMOVEDATA_PATROL_CLOCKWISE".into();
                object.move_range_x = width;
                object.move_range_y = height;
            }
            Some(other) => anyhow::bail!("unsupported trainer patrol {other}"),
        }
        object.radius = if placement.optional_battle {
            0
        } else {
            u16::from(t.sight_range)
        };
        maps.insert(placement.map.clone(), map);
    }
    let map_names = pack.data.map_name_by_constant_from_attributes()?;
    for (_, mut map) in maps {
        authoring::finish(&pack, &mut map)?;
        map.script_map_commands =
            crate::parse_script_map_commands(&map.id, &map.scripts, &map_names)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.trainer_scripts = crate::parse_trainer_scripts(&map.id, &map.scripts)?;
        map.scripted_trainer_battles =
            crate::parse_scripted_trainer_battles(&map.id, &map.scripts)?;
        authored_town_publication::replace(&mut pack, map)?;
    }
    pack.data.minnesota_trainer_scaling = enable_scaling;
    pack.report.manifests.push(manifest.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

/// Installs one original NPC's six-pose overworld bank, battle art and encounter.
///
/// Existing runtime files and trainers cannot be replaced. The source is the
/// ordinary Minnesota trainer placement schema, with an explicit template_class
/// for the new class's battle behavior. Art must use the class's lowercase stem.
///
/// # Errors
/// Rejects multiple placements, occupied identities, incomplete native art,
/// mismatched palettes, and any invalid or unreachable trainer placement.
pub fn build_original_npc_modpack(
    base: &CompiledGamePack,
    source: &str,
    art: &BTreeMap<String, Vec<u8>>,
) -> Result<CompiledGamePack> {
    let expansion: Expansion = serde_json::from_str(source)?;
    ensure!(
        expansion.trainers.len() == 1,
        "finish one playable NPC at a time"
    );
    let placement = &expansion.trainers[0];
    let class = &placement.trainer.trainer_class;
    ensure!(
        class.starts_with("MN_") && class.bytes().all(|b| b.is_ascii_uppercase() || b == b'_'),
        "original NPC class required"
    );
    let stem = class.to_ascii_lowercase();
    ensure!(
        placement.trainer.sprite == format!("SPRITE_{class}")
            && placement.template_class.is_some()
            && !base.data.trainer_class_names.contains_key(class)
            && !base
                .data
                .sprite_palette_defaults
                .contains_key(&placement.trainer.sprite),
        "new original class, matching sprite and native behavior template required"
    );
    verify_original_npc_bundle(art, &stem)?;
    let mut parent = base.clone();
    for (path, bytes) in art {
        ensure!(
            !parent.runtime_files.contains_key(path),
            "preserve existing art: {path}"
        );
        parent.runtime_files.insert(path.clone(), bytes.clone());
    }
    parent
        .data
        .sprite_palette_defaults
        .insert(placement.trainer.sprite.clone(), 0);
    let class_name = placement
        .class_name
        .clone()
        .unwrap_or_else(|| class.trim_start_matches("MN_").replace('_', " "));
    ensure!(
        !class_name.is_empty() && class_name.chars().count() <= 18,
        "native class label required"
    );
    parent
        .data
        .trainer_class_names
        .insert(class.clone(), class_name);
    parent.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        parent.format_version,
        &parent.data,
        &parent.audio_manifest,
        &parent.runtime_files,
        &parent.report,
    )?;
    build_trainers(
        &parent,
        source,
        &format!("original-npc-{stem}-v1"),
        base.data.minnesota_trainer_scaling,
    )
}

/// Replaces one registered original NPC's overworld pixels and native tiles.
///
/// Requires the complete native bundle for validation, but preserves its palette,
/// battle art, all gameplay data, audio and every unrelated runtime file.
///
/// # Errors
/// Rejects unknown/mismatched NPC identities, changed battle art or palettes,
/// malformed native bundles, no-op revisions and invalid resulting packs.
pub fn revise_original_npc_overworld(
    base: &CompiledGamePack,
    source: &str,
    art: &BTreeMap<String, Vec<u8>>,
) -> Result<CompiledGamePack> {
    geography::require_compact_superior(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let expansion: Expansion = serde_json::from_str(source)?;
    ensure!(
        expansion.schema_version == 1 && expansion.trainers.len() == 1,
        "revise one original NPC at a time"
    );
    let placement = &expansion.trainers[0];
    let class = &placement.trainer.trainer_class;
    ensure!(
        class.starts_with("MN_")
            && class
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && placement.trainer.sprite == format!("SPRITE_{class}")
            && base.data.trainer_class_names.contains_key(class)
            && base
                .data
                .sprite_palette_defaults
                .contains_key(&placement.trainer.sprite),
        "registered original NPC class and matching sprite required"
    );
    let trainer = base
        .data
        .trainers
        .trainers
        .get(&placement.trainer.id)
        .context("original NPC trainer is missing")?;
    ensure!(
        trainer.trainer_class == *class
            && base
                .data
                .trainers
                .trainers
                .values()
                .filter(|trainer| trainer.trainer_class == *class)
                .count()
                == 1,
        "original NPC must own its class"
    );
    let map = base
        .data
        .maps
        .get(&placement.map)
        .context("original NPC map is missing")?;
    ensure!(
        map.objects
            .iter()
            .any(|object| object.script == placement.trainer.id
                && object.sprite == placement.trainer.sprite
                && [object.x, object.y] == placement.trainer.tile),
        "original NPC placement differs from the existing encounter"
    );
    let stem = class.to_ascii_lowercase();
    verify_original_npc_bundle(art, &stem)?;
    let mutable_paths = [
        format!("gfx/sprites/{stem}.png"),
        format!("gfx/sprites/{stem}.2bpp"),
    ];
    let mut revised = base.clone();
    let mut changed = false;
    for (path, bytes) in art {
        let previous = base
            .runtime_files
            .get(path)
            .context("registered NPC art is missing")?;
        if mutable_paths.contains(path) {
            changed |= previous != bytes;
            revised.runtime_files.insert(path.clone(), bytes.clone());
        } else {
            ensure!(previous == bytes, "preserve battle art and palette: {path}");
        }
    }
    ensure!(changed, "overworld revision must change native pixels");
    revised.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        revised.format_version,
        &revised.data,
        &revised.audio_manifest,
        &revised.runtime_files,
        &revised.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&revised)?;
    Ok(revised)
}

pub(crate) fn verify_original_npc_bundle(
    art: &BTreeMap<String, Vec<u8>>,
    stem: &str,
) -> Result<()> {
    let expected: BTreeSet<_> = ["sprites", "trainers"]
        .into_iter()
        .flat_map(|kind| {
            let stem = &stem;
            ["png", "2bpp", "gbcpal"].map(|extension| format!("gfx/{kind}/{stem}.{extension}"))
        })
        .collect();
    ensure!(
        art.keys().cloned().collect::<BTreeSet<_>>() == expected,
        "exact native art bundle required"
    );
    ensure!(
        art[&format!("gfx/sprites/{stem}.2bpp")].len() == 384
            && art[&format!("gfx/trainers/{stem}.2bpp")].len() == 784,
        "six 16x16 overworld frames and 56x56 battle art required"
    );
    let palette = &art[&format!("gfx/sprites/{stem}.gbcpal")];
    ensure!(
        palette.len() == 8 && palette == &art[&format!("gfx/trainers/{stem}.gbcpal")],
        "matched native palettes required"
    );
    verify_original_npc_art(art, stem)?;
    Ok(())
}

fn verify_original_npc_art(art: &BTreeMap<String, Vec<u8>>, stem: &str) -> Result<()> {
    use crate::minnesota_pokemon_pixel_art::bitplanes;
    use crate::minnesota_sprite_sheet::MinnesotaSheet;

    let palette_bytes = &art[&format!("gfx/trainers/{stem}.gbcpal")];
    let mut palette = [0_u16; 4];
    for (slot, bytes) in palette.iter_mut().zip(palette_bytes.as_chunks::<2>().0) {
        *slot = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
    ensure!(
        palette[0] == 32767 && palette[3] == 0 && palette.iter().all(|v| *v <= 32767),
        "invalid native palette"
    );
    ensure!(
        (0..4).all(|i| (0..i).all(|j| palette[i] != palette[j])),
        "duplicate native palette inks"
    );
    let rgb = palette.map(|v| {
        [0, 5, 10].map(|shift| {
            let c = ((v >> shift) & 31) as u8;
            (c << 3) | (c >> 2)
        })
    });
    for (kind, width, height) in [("sprites", 16, 96), ("trainers", 56, 56)] {
        let image = MinnesotaSheet::from_png(&art[&format!("gfx/{kind}/{stem}.png")])?;
        ensure!(
            image.width == width && image.height == height,
            "native PNG dimensions differ"
        );
        let mut indices = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let pixel = image.pixel(x, y).context("missing native pixel")?;
                let colors = if kind == "sprites" {
                    [[255; 3], [170; 3], [85; 3], [0; 3]]
                } else {
                    rgb
                };
                let index = colors
                    .iter()
                    .position(|c| *c == pixel)
                    .context("PNG pixel is outside exact native palette")?
                    as u8;
                if kind == "sprites" {
                    ensure!(
                        image.alpha(x, y) == Some(if index == 0 { 0 } else { 255 }),
                        "overworld alpha differs from native ink"
                    );
                }
                indices.push(index);
            }
        }
        ensure!(indices.iter().any(|v| *v != 0), "blank native bank");
        ensure!(
            bitplanes(&indices, width, height) == art[&format!("gfx/{kind}/{stem}.2bpp")],
            "PNG and tile bank disagree"
        );
        if kind == "sprites" {
            ensure!(
                indices
                    .as_chunks::<256>()
                    .0
                    .iter()
                    .all(|frame| frame.iter().any(|v| *v != 0)),
                "blank native pose"
            );
            for direction in 0..3 {
                ensure!(
                    indices[direction * 256..(direction + 1) * 256]
                        != indices[(direction + 3) * 256..(direction + 4) * 256],
                    "identical walk pair"
                );
            }
        }
    }
    Ok(())
}

fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

fn validate_placement(
    pack: &CompiledGamePack,
    map: &MapModule,
    tile: [u16; 2],
    approach: [u16; 2],
) -> Result<()> {
    let occupied: BTreeSet<_> = map.objects.iter().map(|o| [o.x, o.y]).collect();
    let floor = |tile: [u16; 2]| -> bool {
        tile[0] < map.attributes.width * 2
            && tile[1] < map.attributes.height * 2
            && crate::authored_route_modpack::collision(
                pack,
                &map.blocks,
                map.attributes.width,
                tile,
                &map.attributes.tileset_name,
            )
            .is_ok_and(|collision| matches!(collision, "FLOOR" | "TALL_GRASS"))
    };
    ensure!(
        floor(tile)
            && floor(approach)
            && !occupied.contains(&tile)
            && !occupied.contains(&approach)
            && neighbors(tile).any(|n| n == approach),
        "trainer and adjacent approach must be walkable and empty: {} {tile:?}",
        map.id
    );
    ensure!(
        tile[0] >= 3
            && tile[1] >= 3
            && tile[0] + 3 < map.attributes.width * 2
            && tile[1] + 3 < map.attributes.height * 2,
        "trainer must leave connection arrivals clear"
    );
    ensure!(
        !map.events
            .warps
            .iter()
            .any(|w| w.x.abs_diff(tile[0]) + w.y.abs_diff(tile[1]) < 3)
            && !map
                .events
                .coord_events
                .iter()
                .any(|e| e.x == tile[0] && e.y == tile[1])
            && !map
                .events
                .bg_events
                .iter()
                .any(|e| e.x == tile[0] && e.y == tile[1]),
        "trainer overlaps a map event"
    );
    // Removing a non-articulation tile cannot split an existing walking path.
    // Include previous new actors, so several placements cannot form a wall.
    let adjacent: Vec<_> = neighbors(tile)
        .filter(|n| floor(*n) && !occupied.contains(n))
        .collect();
    let mut reached = BTreeSet::from([approach]);
    let mut queue = VecDeque::from([approach]);
    while let Some(next) = queue.pop_front() {
        for n in neighbors(next) {
            if n != tile && floor(n) && !occupied.contains(&n) && reached.insert(n) {
                queue.push_back(n);
            }
        }
    }
    ensure!(
        adjacent.len() >= 2 && adjacent.iter().all(|n| reached.contains(n)),
        "trainer blocks a walking lane: {} {tile:?}",
        map.id
    );
    Ok(())
}

#[cfg(test)]
#[path = "minnesota_trainers_tests.rs"]
mod tests;
