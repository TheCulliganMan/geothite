//! Complete-source furnishings, independent of wall pixels and collision data.
//!
//! Resolve against immutable source cells before any legacy/live masking. Every
//! reserved object has a verified ground sample. Unknown, altered and clipped
//! drawings remain unclaimed and retain the existing rendering path.
use super::*;
use crate::interior_models::{ModelKind, model};
use crate::live_profiles::{Document, Object};
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub(super) struct Placement {
    pub(super) kind: ModelKind,
    pub(super) column: usize,
    pub(super) row: usize,
    pub(super) width: usize,
    pub(super) height: usize,
    ground: usize,
    depth_pixels: f32,
    height_pixels: f32,
    base_pixels: f32,
    footing_pixels: Option<f32>,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| (self.row + y) * width + self.column + x)
        })
    }
    pub(super) fn label(&self) -> &'static str {
        self.kind.label()
    }
}
fn built_in_profiles() -> &'static Document {
    static DOCUMENT: OnceLock<Document> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .expect("validated source profiles")
    })
}

/// Map specific existing source-profile roles onto independently authored art.
/// No prefix/keyword guesses: a new or edited profile does not acquire a model.
fn profile_kind(object: &Object) -> Option<ModelKind> {
    use ModelKind::*;
    let kind = match (object.tileset.as_str(), object.name.as_str()) {
        ("lab", "Lab cabinet 04 0")
        | ("lab", "Lab cabinet 04 2")
        | ("lab", "Lab cabinet 14 0")
        | ("lab", "Lab cabinet 14 2")
        | ("lab", "Shared lab paired bookcases 14") => LabBookcase,
        ("lab", "Lab wall 08") | ("lab", "Lab wall 0d") | ("lab", "Lab wall 17") => WallPanel,
        ("lab", "Lab windows 09") => WindowWall,
        ("lab", "Lab restoration machine") => LabRestorationMachine,
        ("lab", "Shared lab computer workstation") => LabWorkstation,
        ("lab", "Lab central desk assembly") => LabCounter,
        ("house", "New Bark house plant 06") | ("house", "New Bark house plant 07") => PottedPlant,
        ("players_house", "PlayersHouse1F dining table") => DiningTable,
        ("house", "Neighbor dining table") => DiningTable,
        ("players_room", "Player bedroom table") => DiningTable,
        ("pokecenter", "Pokecenter upstairs complete PC")
        | ("pokecenter", "Shared Pokecenter complete PC") => Computer,
        ("pokecenter", "Pokecenter upstairs wall displays 2b")
        | ("pokecenter", "Pokecenter upstairs wall displays 0a") => PictureFrame,
        ("pokecenter", "Pokecenter upstairs seat 2d 0")
        | ("pokecenter", "Pokecenter upstairs seat 2d 2")
        | ("pokecenter", "Pokecenter upstairs seat 29 0")
        | ("pokecenter", "Shared Pokecenter seat 2e")
        | ("pokecenter", "Shared Pokecenter seat 2f") => LinkSeat,
        ("pokecenter", "Shared Pokecenter healing machine") => HealingMachine,
        ("mart", "Shared Mart Ecruteak Mart refrigerator 0")
        | ("mart", "Shared Mart Ecruteak Mart refrigerator 2") => RetailRefrigerator,
        ("mart", "Shared Mart Ecruteak Mart shelf 0")
        | ("mart", "Shared Mart Ecruteak Mart shelf 2")
        | ("mart", "Shared Mart Ecruteak Mart rear shelf 2") => RetailShelf,
        ("facility", "Shared facility paired bookcases 06") => Bookcase,
        ("facility", "Ruins research instrument cabinet") => LabRestorationMachine,
        ("mart", "Shared department store cushion 0")
        | ("mart", "Shared department store cushion 2") => Cushion,
        ("train_station", "Shared station seat 0")
        | ("train_station", "Shared station seat 2")
        | ("train_station", "Shared station column seat 0")
        | ("train_station", "Shared station column seat 2") => Chair,
        ("game_corner", "Shared game corner rear machine 0")
        | ("game_corner", "Shared game corner rear machine 2") => ArcadeMachine,
        ("game_corner", "Shared game row cabinet 7 0")
        | ("game_corner", "Shared game row cabinet 7 2")
        | ("game_corner", "Shared game row cabinet 11 0") => ArcadePair,
        ("game_corner", "Shared game corner stool 5 0")
        | ("game_corner", "Shared game corner stool 5 2")
        | ("game_corner", "Shared game corner stool 6 0")
        | ("game_corner", "Shared game corner stool 6 2") => ArcadeStool,
        ("radio_tower", "Shared radio tower PC") => Computer,
        ("traditional_house", "Traditional house north clock") => PendulumClock,
        ("traditional_house", "Traditional house north cabinet")
        | ("traditional_house", "Traditional house low cabinet") => Cabinet,
        ("traditional_house", "Traditional house plain north wall") => WallPanel,
        ("traditional_house", "Traditional house north wall with radio") => RadioWall,
        ("traditional_house", "Traditional house drawer cabinet") => DrawerCabinet,
        ("traditional_house", "Traditional cushion 10")
        | ("traditional_house", "Traditional cushion 1b")
        | ("traditional_house", "Traditional cushion 19") => Cushion,
        ("gate", "Shared gate north double doors") => GateDoorFrame,
        ("gate", "Shared park gate terminal") => GateTerminal,
        ("gate", "Shared gate wall variant 29")
        | ("gate", "Shared gate wall variant 2e")
        | ("gate", "Shared gate wall variant 2d")
        | ("gate", "Shared park gate terminal adjacent wall")
        | ("gate", "Shared gate counter junction wall")
        | ("gate", "Shared gate counter junction upper wall")
        | ("gate", "Shared gate mirrored counter wall")
        | ("gate", "Shared gate mirrored counter upper wall")
        | ("gate", "Shared gate wall above planters") => WallPanel,
        ("gate", "Shared gate planter 0")
        | ("gate", "Shared gate planter 2")
        | ("gate", "Shared gate corridor planter") => PottedPlant,
        ("radio_tower", "Shared radio tower glass machine") => BroadcastRack,
        ("gate", "Shared gate long planter") => LongPlanter,
        ("facility", "Shared facility potted plant 0d")
        | ("facility", "Shared facility potted plant 0e") => PottedPlant,
        ("lab", "Shared lab round stool") => Stool,
        ("lab", "Shared lab display table") => LabDisplayTable,
        ("house", "Shared house flower stand 30 0")
        | ("house", "Shared house flower stand 30 2")
        | ("house", "Shared house flower stand 31 0")
        | ("house", "Shared house flower stand 31 2") => FlowerStand,
        ("house", "Soul House memorial 28 0")
        | ("house", "Soul House memorial 28 2")
        | ("house", "Soul House memorial 2a 0")
        | ("house", "Soul House memorial 2a 2")
        | ("house", "Soul House memorial 2b 0")
        | ("house", "Soul House memorial 2b 2") => Memorial,
        _ => return None,
    };
    Some(if kind == ModelKind::WallPanel {
        match object.tileset.as_str() {
            "traditional_house" => ModelKind::WallTraditional,
            "lab" | "gate" | "pokecenter" => ModelKind::WallClinical,
            _ => kind,
        }
    } else {
        kind
    })
}

fn cable_club_fixture(object: &Object, map: &str) -> bool {
    super::cable_club::is_map(map) && object.tileset == "pokecenter"
        && matches!(object.name.as_str(), "Pokecenter upstairs complete PC"
            | "Pokecenter upstairs wall displays 2b" | "Pokecenter upstairs wall displays 0a")
}

fn proportions(kind: ModelKind, width: usize, height: usize) -> (f32, f32) {
    use ModelKind::*;
    let source_depth = height as f32 * 8.0;
    let (depth, rise): (f32, f32) = match kind {
        Stool | ArcadeStool => (12.0, 7.0),
        Chair | LinkSeat => (12.0, 14.0),
        Cushion => (15.0, 3.0),
        DiningTable => (source_depth, 10.0),
        LowTable => (source_depth, 6.0),
        CafeTable => (source_depth, 16.0),
        BedRed | BedBlue | BedGreen | BedPink | BedFeathery | BedPolkadot | BedPikachu => {
            (source_depth, 11.0)
        }
        PottedPlant | PlantMagna | PlantTropic | PlantJumbo => (12.0, 23.0),
        WallDomestic | WallTraditional | WallClinical | WallAcoustic | ArcadeWall => {
            (3.0, source_depth.max(24.0))
        }
        PictureFrame => (2.0, source_depth),
        CarpetCloth => (source_depth, 0.18),
        StairFlight => (16.0, 22.4),
        OfficePhone => (10.0, 6.0),
        BroadcastRack => (9.0, 30.0),
        TowerReception => (12.0, 18.0),
        LongPlanter => (source_depth, 17.0),
        FlowerStand => (14.0, 16.0),
        Bookcase | LabBookcase => (9.0, 29.0),
        Cabinet | DrawerCabinet => (8.0, 22.0),
        PendulumClock => (7.0, 30.0),
        Computer | LabWorkstation => (12.0, 22.0),
        Television => (9.0, 20.0),
        Radio => (7.0, 12.0),
        Keyboard => (7.0, 1.5),
        GameConsole => (13.0, 6.0),
        Refrigerator | RetailRefrigerator | RetailShelf | GiftShelf | VendingMachine => {
            (10.0, 30.0)
        }
        KitchenStove | KitchenSink => (8.0, 11.0),
        ReceptionCounter | LabCounter => (8.0, 10.0),
        HealingMachine => (12.0, 30.0),
        HealingConsole => (13.0, 13.0),
        ClinicalPartition | TimberPartition => (source_depth, 20.0),
        LabRestorationMachine => (12.0, 22.0),
        LabDisplayTable => (source_depth, 13.0),
        ArcadeMachine => (10.0, 29.0),
        ArcadePair => (14.0, 21.0),
        PrizeBooth => (8.0, 32.0),
        StationBench => (14.0, 14.0),
        StationTurnstile => (18.0, 16.0),
        GateTerminal => (9.0, 21.0),
        BroadcastConsole => (14.0, 19.0),
        WallPanel | WindowWall | RadioWall | GateDoorFrame => (3.0, source_depth),
        OpenBook => (13.0, 4.0),
        Memorial => (5.0, 14.0),
    };
    // Every axis stays within the source drawing, even for compact variants.
    (
        depth.min(source_depth).max(1.0),
        rise.min(40.0).max(if width == 0 || kind == ModelKind::CarpetCloth { 0.0 } else { 1.0 }),
    )
}
fn mart_display_model(source: &VisualTileSource) -> Option<ModelKind> {
    // These roles follow the complete, tile-verified drawings in mart.rs.
    // Unknown drawings never become merchandise shelves by default.
    crate::mart::display_rack_local(source)?;
    match source.metatile_id {
        0x0b => Some(ModelKind::PottedPlant),
        0x07 | 0x20 => Some(ModelKind::Cushion),
        0x21 => Some(ModelKind::VendingMachine),
        0x12 | 0x13 | 0x2c | 0x2d => Some(ModelKind::RetailShelf),
        _ => None,
    }
}
fn ground_sample(
    cells: &[&VisualTile],
    map: &str,
    tileset: &str,
    tile_index: u16,
) -> Option<usize> {
    cells.iter().position(|tile| {
        tile.source.tileset_id.as_ref() == tileset
            && tile.source.tile_index == tile_index
            && matches!(
                shape_for_source_on_map(map, &tile.source),
                CellShape::Flat | CellShape::Water | CellShape::PlaneAt { height: 0.0 }
            )
    })
}

/// Resolve claims on original cells, before the live-profile or legacy passes.
/// The document is used only to respect customized profile ownership: changed
/// rows or geometry retain the user's live renderer instead of being replaced.
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    profiles: Option<&Document>,
) -> Vec<Placement> {
    let mut out = Vec::new();
    let mut claimed = vec![false; cells.len()];
    // A user's altered live object retains ownership over all matching cells,
    // including overlaps with older compiled fixture classifiers.
    if let Some(active) = profiles {
        let customized = Document {
            objects: active
                .objects
                .iter()
                .filter(|object| {
                    built_in_profiles()
                        .objects
                        .iter()
                        .any(|canonical| canonical.name == object.name && canonical != *object)
                        || ((super::cable_club::is_map(map)
                            || crate::casino::is_game_corner_map(map)
                            || map == "CianwoodGym")
                            && !built_in_profiles().objects.iter().any(|canonical| canonical == *object))
                })
                .cloned()
                .collect(),
            atmosphere: None,
        };
        for placement in live::resolve(
            cells,
            geometry.width,
            geometry.height,
            map,
            Some(&customized),
        ) {
            for index in placement.indices(geometry.width) {
                claimed[index] = true;
            }
        }
    }
    append_prize_booth_candidate(map, cells, geometry, &mut claimed, &mut out);
    for object in &built_in_profiles().objects {
        let Some(kind) = profile_kind(object) else {
            continue;
        };
        let cable_fixture = cable_club_fixture(object, map);
        if !cable_fixture && (object.map.as_deref().is_some_and(|id| id != map)
            || object
                .maps
                .as_ref()
                .is_some_and(|ids| !ids.iter().any(|id| id == map)))
        {
            continue;
        }
        if let Some(active) =
            profiles.and_then(|d| d.objects.iter().find(|o| o.name == object.name))
        {
            if active != object {
                continue;
            }
        }
        let ground_tile = if object.tileset == "mart"
            && matches!(kind, ModelKind::RetailRefrigerator | ModelKind::RetailShelf)
        {
            crate::mart::ground_tile_for_map(map)
        } else {
            object.ground
        };
        let Some(ground) = ground_sample(cells, map, &object.tileset, ground_tile) else {
            continue;
        };
        let (w, h) = (object.tiles[0].len(), object.tiles.len());
        if w > geometry.width || h > geometry.height {
            continue;
        }
        for row in 0..=geometry.height - h {
            for column in 0..=geometry.width - w {
                let complete = (0..h).all(|y| {
                    (0..w).all(|x| {
                        let index = (row + y) * geometry.width + column + x;
                        let s = &cells[index].source;
                        !claimed[index]
                            && s.tileset_id.as_ref() == object.tileset
                            && s.metatile_id
                                == object.metatiles.as_ref().map_or(object.metatile, |blocks| {
                                    blocks[(usize::from(object.origin[1]) + y) / 4]
                                        [(usize::from(object.origin[0]) + x) / 4]
                                })
                            && usize::from(s.subtile_column)
                                == (usize::from(object.origin[0]) + x) % 4
                            && usize::from(s.subtile_row) == (usize::from(object.origin[1]) + y) % 4
                            && s.tile_index == object.tiles[y][x]
                    })
                });
                if !complete {
                    continue;
                }
                let (depth_pixels, height_pixels) = proportions(kind, w, h);
                let (depth_pixels,height_pixels)=if kind==ModelKind::Cushion && object.tileset=="mart" {
                    (12.0,12.0)
                } else {(depth_pixels,height_pixels)};
                let p = Placement {
                    kind,
                    column,
                    row,
                    // Each paired profile is a sign next to a real DOOR.
                    // Keep the full source guard, but give PictureFrame only
                    // the sign half. cable_club supplies the open door frame.
                    width: if cable_fixture && kind == ModelKind::PictureFrame { 2 } else { w },
                    height: h,
                    ground,
                    depth_pixels,
                    height_pixels,
                    base_pixels: 0.0,
                    footing_pixels: object.footing_pixels,
                };
                for i in p.indices(geometry.width) {
                    claimed[i] = true;
                }
                out.push(p);
            }
        }
    }
    // Older grouped matchers encode exact block phase and source tile identity.
    // These objects are independent from the live catalog and take only free cells.
    let mut add = |groups: Vec<TreePlacement>,
                   kind_for: &dyn Fn(&VisualTileSource) -> ModelKind| {
        for group in groups {
            let source = &cells[group.row * geometry.width + group.column].source;
            let kind = kind_for(source);
            let Some(ground) =
                ground_sample(cells, map, &source.tileset_id, group.ground_tile_index)
            else {
                continue;
            };
            let (depth_pixels, height_pixels) = if kind == ModelKind::Cushion
                && source.tileset_id.as_ref() == "mart" {
                (12.0,12.0)
            } else { proportions(kind, group.width, group.height) };
            let p = Placement {
                kind,
                column: group.column,
                row: group.row,
                width: group.width,
                height: group.height,
                ground,
                depth_pixels,
                height_pixels,
                base_pixels: 0.0,
                footing_pixels: None,
            };
            if p.indices(geometry.width).any(|i| claimed[i]) {
                continue;
            }
            for i in p.indices(geometry.width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    };
    add(house_plant_placements(cells, geometry), &|_| {
        ModelKind::PottedPlant
    });
    add(house_bookcase_placements(cells, geometry), &|_| {
        ModelKind::Bookcase
    });
    add(house_upright_fixture_placements(cells, geometry), &|s| {
        if s.metatile_id == 0x1e {
            ModelKind::Television
        } else {
            ModelKind::Radio
        }
    });
    add(players_house_bookcase_placements(cells, geometry), &|_| {
        ModelKind::Bookcase
    });
    // Player-home fixture material uses its original north-course floor sample.
    let mut player_fixtures = players_house_upright_fixture_placements(cells, geometry);
    for p in &mut player_fixtures {
        p.ground_tile_index = crate::players_house::FLOOR_TILE;
    }
    add(
        player_fixtures,
        &|s| match (s.metatile_id, s.subtile_column) {
            (0x07, 0 | 1) => ModelKind::KitchenStove,
            (0x07, _) => ModelKind::KitchenSink,
            (0x0f, _) => ModelKind::Refrigerator,
            (0x11, 0 | 1) => ModelKind::Television,
            _ => ModelKind::Cabinet,
        },
    );
    add(players_house_tv_placements(cells, geometry), &|_| {
        ModelKind::Television
    });
    add(players_house_console_placements(cells, geometry), &|_| {
        ModelKind::GameConsole
    });
    add(players_house_bed_placements(cells, geometry), &|_| {
        ModelKind::BedRed
    });
    add(
        player_bed_placements(cells, geometry),
        &|s| match s.metatile_id {
            0x1c => ModelKind::BedPink,
            0x1d => ModelKind::BedPolkadot,
            0x1e => ModelKind::BedPikachu,
            _ => ModelKind::BedFeathery,
        },
    );
    add(
        traditional_gift_shop_shelf_placements(map, cells, geometry),
        &|_| ModelKind::GiftShelf,
    );
    add(
        traditional_house_cushion_placements(map, cells, geometry),
        &|_| ModelKind::Cushion,
    );
    add(soul_house_bench_placements(cells, geometry), &|_| {
        ModelKind::Memorial
    });
    add(house_open_book_placements(cells, geometry), &|_| {
        ModelKind::OpenBook
    });
    add(house_display_table_placements(cells, geometry), &|_| {
        ModelKind::FlowerStand
    });
    add(
        mr_pokemon_work_counter_placements(map, cells, geometry),
        &|_| ModelKind::LabCounter,
    );
    let displays = mart_display_rack_placements(map, cells, geometry)
        .into_iter().filter(|p| {
            mart_display_model(&cells[p.row * geometry.width + p.column].source).is_some()
        }).collect();
    add(displays, &|s| mart_display_model(s).expect("explicitly classified Mart drawing"));
    add(
        pokecenter_healing_console_placements(map, cells, geometry),
        &|_| ModelKind::HealingConsole,
    );
    add(
        pokecenter_link_floor_seat_placements(map, cells, geometry),
        &|_| ModelKind::LinkSeat,
    );
    add(
        pokecom_workstation_placements(map, cells, geometry),
        &|_| ModelKind::Computer,
    );
    add(pokecom_plant_placements(map, cells, geometry), &|_| {
        ModelKind::PottedPlant
    });
    add(pokecom_chair_placements(map, cells, geometry), &|_| {
        ModelKind::Chair
    });
    add(train_station_seat_placements(map, cells, geometry), &|_| {
        ModelKind::Chair
    });
    add(
        train_station_planter_placements(map, cells, geometry),
        &|_| ModelKind::LongPlanter,
    );
    add(train_station_gate_placements(map, cells, geometry), &|_| {
        ModelKind::StationTurnstile
    });
    if crate::casino::is_game_corner_map(map) {
        add(casino_terminal_placements(cells, geometry), &|_| {
            ModelKind::ArcadeMachine
        });
        add(casino_slot_machine_placements(cells, geometry), &|_| {
            ModelKind::ArcadeMachine
        });
        add(casino_plant_placements(cells, geometry), &|_| {
            ModelKind::PottedPlant
        });
    }
    // The player's computer is one screen-plus-keyboard assembly; the separate
    // source rows are never interpreted as two full computer towers.
    add(
        grouped_flat_card_placements(cells, geometry, 0x02, false, |s| {
            crate::interior::player_room_pc_monitor_local(s)
                .map(|(x, y)| (x, y, 2, 3))
                .or_else(|| {
                    crate::interior::player_room_pc_keyboard_local(s).map(|(x, _)| (x, 2, 2, 3))
                })
        }),
        &|_| ModelKind::Computer,
    );
    add(
        grouped_flat_card_placements(cells, geometry, 0x01, false, |s| {
            crate::house::furniture_local(s).and_then(|(x, y, k)| {
                (k == crate::house::FurnitureKind::Stool).then_some((x, y, 2, 2))
            })
        }),
        &|_| ModelKind::Stool,
    );
    drop(add);
    // Complete dining drawings that cross metatile boundaries are already
    // validated by the established table matcher (including native corners).
    for table in house_table_placements(cells, geometry) {
        let s = &cells[table.row * geometry.width + table.column].source;
        let Some(ground) = ground_sample(cells, map, &s.tileset_id, table.ground_tile_index) else {
            continue;
        };
        let p = Placement {
            kind: if s.tileset_id.as_ref() == "traditional_house" {
                ModelKind::LowTable
            } else {
                ModelKind::DiningTable
            },
            column: table.column,
            row: table.row,
            width: 4,
            height: 4,
            ground,
            depth_pixels: 32.0,
            height_pixels: table.height_pixels,
            base_pixels: 0.0,
            footing_pixels: None,
        };
        if p.indices(geometry.width).any(|i| claimed[i]) {
            continue;
        }
        for i in p.indices(geometry.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
    append_counter_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_casino_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_cianwood_walls(map, cells, geometry, &mut claimed, &mut out);
    append_signature_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_stair_candidates(map, cells, geometry, &mut claimed, &mut out);
    out
}

fn append_counter_candidates(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    // Exact source strips that already have native top+front semantics. End
    // pieces stay separate so L-shaped access and service openings stay open.
    for (index, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        let (ox, oy, w, ground, kind) = match (s.tileset_id.as_ref(), s.metatile_id) {
            ("pokecenter", 0x05 | 0x06 | 0x07) if s.subtile_column == 0 && s.subtile_row == 0 => {
                (0, 0, 4, 0x11, ModelKind::ReceptionCounter)
            }
            ("pokecenter", 0x38) if s.subtile_column == 0 && s.subtile_row == 0 => {
                (0, 0, 2, 0x11, ModelKind::ReceptionCounter)
            }
            ("mart", 0x22) if s.subtile_column == 0 && s.subtile_row == 0 => (
                0,
                0,
                2,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            ("mart", 0x0c) if s.subtile_column == 2 && s.subtile_row == 0 => (
                2,
                0,
                2,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            ("mart", 0x0d) if s.subtile_column == 0 && s.subtile_row == 0 => (
                0,
                0,
                4,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            ("mart", 0x0e) if s.subtile_column == 0 && s.subtile_row == 0 => (
                0,
                0,
                2,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            _ => continue,
        };
        let (column, row) = (index % geometry.width, index / geometry.width);
        if column + w > geometry.width || row + 2 > geometry.height {
            continue;
        }
        let Some(ground) = ground_sample(cells, map, &s.tileset_id, ground) else {
            continue;
        };
        let complete = (0..2).all(|y| {
            (0..w).all(|x| {
                let i = (row + y) * geometry.width + column + x;
                let t = &cells[i].source;
                if claimed[i]
                    || t.tileset_id != s.tileset_id
                    || t.metatile_id != s.metatile_id
                    || t.subtile_column as usize != ox + x
                    || t.subtile_row as usize != oy + y
                {
                    return false;
                }
                let shape = if t.tileset_id.as_ref() == "pokecenter" {
                    crate::pokecenter::shape(map, t)
                } else {
                    crate::mart::shape(t)
                };
                match (y, shape) {
                    (
                        0,
                        Some(CellShape::RaisedTop {
                            height: 8.0,
                            solid: SolidKind::Prop,
                        }),
                    ) => true,
                    (
                        1,
                        Some(CellShape::LedgeBand {
                            face: LedgeFace::South,
                            band_count: 1,
                            height: 8.0,
                            ..
                        }),
                    ) => true,
                    _ => false,
                }
            })
        });
        if !complete {
            continue;
        }
        let p = Placement {
            kind,
            column,
            row,
            width: w,
            height: 2,
            ground,
            depth_pixels: 8.0,
            height_pixels: 8.0,
            base_pixels: 0.0,
            footing_pixels: None,
        };
        for i in p.indices(geometry.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
}

/// Append only after successful resolution. The original immutable ground
/// atlas supplies every vacated pixel, with no fabricated floor or collision.
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
    world_origin: [i32; 2],
) -> bool {
    if p.indices(geometry.width).any(|i| claimed[i]) {
        return false;
    }
    let source = &cells[p.ground];
    let ground_uv = geometry.uv(source.column as usize, source.row as usize);
    for i in p.indices(geometry.width) {
        let (x0, x1, z0, z1) = geometry.bounds(i % geometry.width, i / geometry.width);
        append_top(
            &mut mesh.textured,
            [x0, x1, z0, z1],
            if p.kind == ModelKind::StairFlight {
                p.base_pixels * geometry.tile_height / 8.0
            } else {
                0.0
            },
            ground_uv,
        );
    }
    let west = geometry.origin_x + p.column as f32 * geometry.tile_width;
    let east = west + p.width as f32 * geometry.tile_width;
    let south = geometry.origin_z + (p.row + p.height) as f32 * geometry.tile_height;
    let north = south - p.depth_pixels * geometry.tile_height / 8.0;
    let inset = if matches!(
        p.kind,
        ModelKind::CarpetCloth
            | ModelKind::PrizeBooth
            | ModelKind::WallPanel
            | ModelKind::WindowWall
            | ModelKind::GateDoorFrame
            | ModelKind::RadioWall
            | ModelKind::ClinicalPartition
            | ModelKind::TimberPartition
            | ModelKind::ReceptionCounter
            | ModelKind::LabCounter
            | ModelKind::WallDomestic
            | ModelKind::WallTraditional
            | ModelKind::WallClinical
            | ModelKind::WallAcoustic
            | ModelKind::ArcadeWall
    ) {
        0.0
    } else {
        geometry.tile_width * 0.045
    };
    // Adjacent native window/bookcase units retain their own proportions.
    // Whole-drawing claims do not license stretching one window across a wall.
    let repeats = if (matches!(
        p.kind,
        ModelKind::WindowWall | ModelKind::Bookcase | ModelKind::LabBookcase | ModelKind::ArcadeWall
    ) || (p.kind == ModelKind::PictureFrame
        && cells[p.row * geometry.width + p.column]
            .source
            .tileset_id
            .as_ref()
            == "pokecenter"))
        && p.width % 2 == 0
    {
        p.width / 2
    } else {
        1
    };
    for unit in 0..repeats {
        let left = west + (east - west) * unit as f32 / repeats as f32;
        let right = west + (east - west) * (unit + 1) as f32 / repeats as f32;
        let first = mesh.solid.positions.len();
        if p.kind == ModelKind::WallTraditional
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "tower" {
            let source = &cells[p.row * geometry.width + p.column].source;
            if source.tile_index == 0x41 {
                let facing = if source.subtile_column < 2 { 1.0 } else { -1.0 };
                append_side_wall(&mut mesh.solid,
                    [left, right, north, south], p.height_pixels * geometry.tile_height / 8.0,
                    facing, geometry.tile_width);
            } else {
                crate::dungeon_models::model(crate::dungeon_models::Kind::TimberWall).append(
                    &mut mesh.solid, [left, right, north, south], 0.0,
                    p.height_pixels * geometry.tile_height / 8.0);
            }
        } else if p.kind == ModelKind::Cushion
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref()=="mart" {
            append_store_seat(&mut mesh.solid,[left+inset,right-inset,north,south],geometry.tile_height/8.0);
        } else if p.kind == ModelKind::ArcadeStool {
            let cx = (left + right) * 0.5;
            let cz = geometry.origin_z + (p.row as f32 + p.height as f32 * 0.5) * geometry.tile_height;
            let radius_x = 4.0 * geometry.tile_width / 8.0;
            let radius_z = 4.0 * geometry.tile_height / 8.0;
            model(p.kind).append_fitted(&mut mesh.solid,
                [cx - radius_x, cx + radius_x, cz - radius_z, cz + radius_z],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if p.kind == ModelKind::ArcadePair {
            append_outward_arcade_pair(&mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if p.kind == ModelKind::ArcadeMachine
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "game_corner"
            && matches!(cells[p.row * geometry.width + p.column].source.metatile_id, 0x07 | 0x0b) {
            let source = &cells[p.row * geometry.width + p.column].source;
            append_side_arcade(&mut mesh.solid, [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0, 21.0 * geometry.tile_height / 8.0,
                if source.subtile_column < 2 { -1.0 } else { 1.0 });
        } else {
            model(p.kind).append_fitted(
                &mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0,
            );
        }
        if p.kind == ModelKind::ArcadeWall {
            model(p.kind).apply_arcade_wall_finish(&mut mesh.solid, first);
        } else if p.kind == ModelKind::PrizeBooth {
            model(p.kind).apply_room_finish(&mut mesh.solid, first);
        }
        if p.kind == ModelKind::PottedPlant
            && matches!(cells[p.row * geometry.width + p.column].source.tileset_id.as_ref(), "mart" | "game_corner") {
            model(p.kind).apply_room_palette(&mut mesh.solid, first);
        }
        if cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "game_corner" {
            if p.kind == ModelKind::ArcadeMachine
                && !matches!(cells[p.row * geometry.width + p.column].source.metatile_id, 0x07 | 0x0b) {
                model(p.kind).apply_arcade_finish(&mut mesh.solid, first);
            } else if p.kind == ModelKind::ReceptionCounter {
                model(p.kind).apply_arcade_finish(&mut mesh.solid, first);
            }
        }
        if cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "gate" {
            if p.kind == ModelKind::Chair {
                let block = cells[p.row * geometry.width + p.column].source.metatile_id;
                let facing = match block { 0x32 => -1.0, 0x35 => 1.0, _ => 0.0 };
                turn_counter_chair(&mut mesh.solid, first,
                    [(left + right) * 0.5, (north + south) * 0.5],
                    [(right - left) * 0.5 - inset, (south - north) * 0.5], facing);
            }
            vary_gate_furnishing(&mut mesh.solid, first, p.kind,
                [world_origin[0] + p.column as i32, world_origin[1] + p.row as i32],
                [(left + right) * 0.5, (north + south) * 0.5]);
        }
    }
    if p.kind == ModelKind::PrizeBooth { append_prize_booth_art(mesh, cells, geometry, p); }
    if p.kind == ModelKind::CarpetCloth {
        append_live_carpet_art(mesh, geometry, p);
    }
    if p.kind == ModelKind::PictureFrame {
        append_framed_source_art(mesh, geometry, p, [west, east, north, south], repeats);
    }
    for i in p.indices(geometry.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.label());
        }
        if let Some(height) = p.footing_pixels {
            if mesh.footing_heights.len() == cells.len() {
                mesh.footing_heights[i] = height * geometry.tile_height / 8.0;
            }
        }
    }
    true
}

/// The native row is a back-to-back bank: seats lie west and east, so
/// each cabinet's screen and controls face the seat on its own side.
fn append_outward_arcade_pair(mesh: &mut SurfaceMeshData, bounds: [f32; 4], base: f32, rise: f32) {
    let [west, east, north, south] = bounds;
    let half = (east - west) * 0.5;
    for (unit, facing) in [(0, -1.0), (1, 1.0)] {
        append_side_arcade(mesh, [west + half * unit as f32, west + half * (unit + 1) as f32, north, south], base, rise, facing);
    }
}

// Store seating is raised upholstery on a frame, not a floor cushion.
fn append_store_seat(mesh:&mut SurfaceMeshData,b:[f32;4],scale:f32) {
    let [w,e,n,s]=b;
    let first=mesh.colors.len();
    model(ModelKind::Cushion).append_fitted(mesh,b,6.0*scale,6.0*scale);
    // Named upholstery primitives own their paint and weave; never infer a
    // material from its color. Unknown primitives retain authored placement.
    model(ModelKind::Cushion).apply_room_palette(mesh, first);
    model(ModelKind::Cushion).apply_room_finish(mesh, first);
    let x=(e-w)*0.10;let z=(s-n)*0.12;
    architecture_box(mesh,[w+x,e-x,4.8*scale,6.0*scale,n+z,s-z],[0.28,0.29,0.30]);
    for cx in [w+x,e-x-x*0.65] {for cz in [n+z,s-z-z*0.65] {
        architecture_box(mesh,[cx,cx+x*0.65,0.0,4.8*scale,cz,cz+z*0.65],[0.32,0.34,0.35]);
    }}
}

fn append_side_arcade(mesh: &mut SurfaceMeshData, bounds: [f32; 4], base: f32, rise: f32, facing: f32) {
    let [west, east, north, south] = bounds;
    let width = east - west;
    let depth = south - north;
    let center = [(west + east) * 0.5, (north + south) * 0.5];
    let first = mesh.positions.len();
    // Fit before rotating so rectangular source cells keep their footprint.
    model(ModelKind::ArcadeMachine).append_fitted(mesh,
        [-depth * 0.5, depth * 0.5, -width * 0.5, width * 0.5], base, rise);
    model(ModelKind::ArcadeMachine).apply_arcade_finish(mesh, first);
    for p in &mut mesh.positions[first..] {
        let [x, y, z] = *p;
        *p = [center[0] + facing * z, y, center[1] - facing * x];
    }
    for (n, color) in mesh.normals[first..].iter_mut().zip(&mut mesh.colors[first..]) {
        let [x, y, z] = *n;
        let old_shade = crate::interior_models::authored_face_color([1.0;4], Vec3::from_array(*n))[0];
        *n = [facing * z, y, -facing * x];
        let new_shade = crate::interior_models::authored_face_color([1.0;4], Vec3::from_array(*n))[0];
        for channel in &mut color[..3] { *channel *= new_shade / old_shade; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn arcade_bank_screens_face_the_west_and_east_seats() {
        let mut mesh = SurfaceMeshData::default();
        // Deliberately rectangular footprint catches rotations after fitting.
        append_outward_arcade_pair(&mut mesh, [10.0, 42.0, 20.0, 34.0], 0.0, 21.0);
        assert!(mesh.positions.iter().all(|p|
            p[0] >= 9.999 && p[0] <= 42.001 && p[2] >= 19.999 && p[2] <= 34.001));
        let mut screens = [0, 0];
        for ((p, n), color) in mesh.positions.iter().zip(&mesh.normals).zip(&mesh.colors) {
            assert!((Vec3::from_array(*n).length() - 1.0).abs() < 0.001);
            if (color[0] / color[1] - 0.11 / 0.27).abs() < 0.001
                && (color[1] / color[2] - 0.27 / 0.29).abs() < 0.001 {
                let side = usize::from(p[0] > 26.0);
                assert!(if side == 0 { p[0] < 18.0 } else { p[0] > 34.0 }, "screen faces the bank interior");
                if n[0] * (if side == 0 { -1.0 } else { 1.0 }) > 0.99 { screens[side] += 1; }
            }
        }
        assert!(screens.iter().all(|&count| count > 0));
    }
    fn profile(name: &str) -> Object {
        built_in_profiles()
            .objects
            .iter()
            .find(|o| o.name == name)
            .unwrap()
            .clone()
    }
    #[test]
    fn arcade_stool_seat_is_smaller_than_its_native_plot() {
        let object=profile("Shared game corner stool 5 0");
        let (tiles,g)=fixture(&object);
        let cells:Vec<_>=tiles.iter().collect();
        let p=resolve("GoldenrodGameCorner",&cells,&g,None);
        assert_eq!(p.len(),1);
        let mut mesh=TerrainMeshData {footing_heights:vec![0.0;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,&cells,&g,&p[0],&mut claimed,[0,0]));
        let min=mesh.solid.positions.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min);
        let max=mesh.solid.positions.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max);
        assert!((max-min-8.0).abs()<0.001);
        assert_eq!(claimed.iter().filter(|&&x|x).count(),4);
        assert_eq!(mesh.footing_heights,vec![0.0;tiles.len()]);
    }
    #[test]
    fn new_user_casino_profiles_keep_priority_over_bundled_cabinets() {
        let mut object=profile("Shared game row cabinet 7 0");
        let (tiles,g)=fixture(&object);
        object.name="User bespoke cabinet".into();
        let active=Document {objects:vec![object],atmosphere:None};
        assert!(resolve("GoldenrodGameCorner",&tiles.iter().collect::<Vec<_>>(),&g,Some(&active)).is_empty());
    }
    #[test]
    fn department_store_plant_is_not_a_merchandise_fallback() {
        let mut object=profile("Shared department store cushion 2");
        object.name="Reviewed department store west plant".into();
        object.metatile=0x0b;
        object.metatiles=None;
        object.origin=[0,0];
        object.tiles=vec![vec![0x4a,0x4b],vec![0x08,0x09],vec![0x89,0x8a],vec![0xa7,0xa8]];
        object.ground=0x01;
        let (mut tiles,g)=fixture(&object);
        for map in ["GoldenrodDeptStore1F","CeladonDeptStore1F"] {
            let refs=tiles.iter().collect::<Vec<_>>();
            let plants=resolve(map,&refs,&g,None);
            assert_eq!(plants.len(),1);
            assert_eq!(plants[0].kind,ModelKind::PottedPlant);
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,&refs,&g,&plants[0],&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.iter().any(|p| p[1]>20.));
            assert_eq!(mesh.footing_heights,vec![0.;tiles.len()]);
            for index in [0,1,g.width,g.width+1,2*g.width,2*g.width+1,3*g.width,3*g.width+1] {
                let old=tiles[index].source.tile_index;
                tiles[index].source.tile_index=0xffff;
                assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).is_empty());
                tiles[index].source.tile_index=old;
            }
        }
        let mut unknown=tiles[0].source.clone();
        unknown.metatile_id=0xffff;
        assert_eq!(mart_display_model(&unknown),None);
    }
    #[test]
    fn sixth_floor_right_seats_match_the_left_seating_height() {
        let mut object=profile("Shared department store cushion 2");
        object.metatile=0x20;
        object.metatiles=None;
        object.origin=[2,0];
        object.tiles=vec![vec![0x22,0x23],vec![0x32,0x33],vec![0x24,0x25],vec![0x34,0x35]];
        object.ground=0x01;
        let (mut tiles,g)=fixture(&object);
        for map in ["GoldenrodDeptStore6F","CeladonDeptStore6F"] {
            let seats=resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(seats.len(),1);
            assert_eq!(seats[0].kind,ModelKind::Cushion);
            assert_eq!((seats[0].depth_pixels,seats[0].height_pixels),(12.,12.));
            let old=tiles[0].source.tile_index;
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).is_empty());
            tiles[0].source.tile_index=old;
        }
    }
    #[test]
    fn department_store_seating_uses_native_height_and_keeps_support() {
        for object in built_in_profiles().objects.iter().filter(|o| o.tileset=="mart" && o.name.starts_with("Shared department store cushion")) {
            let (mut tiles,g)=fixture(object);
            let cells=tiles.iter().collect::<Vec<_>>();
            let p=resolve("GoldenrodDeptStore1F",&cells,&g,None);
            let seat=p.iter().find(|p|p.kind==ModelKind::Cushion).unwrap();
            assert_eq!(seat.height_pixels,12.0);
            assert_eq!(seat.depth_pixels,12.0);
            let mut mesh=TerrainMeshData {footing_heights:vec![2.5;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,&cells,&g,seat,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            let top=mesh.solid.positions.iter().map(|p|p[1]).fold(0.0,f32::max);
            assert!((top-12.0).abs()<0.001);
            assert!(mesh.solid.positions.iter().any(|p|p[1]==0.0));
            tiles[0].source.tile_index=0xffff;
            assert!(resolve("GoldenrodDeptStore1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::Cushion));
        }
    }
    fn fixture(object: &Object) -> (Vec<VisualTile>, GridGeometry) {
        let width = object.tiles[0].len() + 2;
        let height = object.tiles.len() + 2;
        let geometry = GridGeometry {
            width,
            height,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut tiles = Vec::new();
        for row in 0..height {
            for column in 0..width {
                tiles.push(VisualTile {
                    animation_frames: None,
                    column: column as u32,
                    row: row as u32,
                    texture: Handle::default(),
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: Arc::from(object.tileset.as_str()),
                        metatile_id: 0,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: object.ground,
                    },
                });
            }
        }
        for y in 0..object.tiles.len() {
            for x in 0..object.tiles[0].len() {
                let source = &mut tiles[y * width + x].source;
                source.metatile_id = object.metatiles.as_ref().map_or(object.metatile, |blocks| {
                    blocks[(usize::from(object.origin[1]) + y) / 4]
                        [(usize::from(object.origin[0]) + x) / 4]
                });
                source.subtile_column = ((usize::from(object.origin[0]) + x) % 4) as u8;
                source.subtile_row = ((usize::from(object.origin[1]) + y) % 4) as u8;
                source.tile_index = object.tiles[y][x];
            }
        }
        (tiles, geometry)
    }
    fn resolve_fixture(
        tiles: &[VisualTile],
        geometry: &GridGeometry,
        profiles: Option<&Document>,
    ) -> Vec<Placement> {
        resolve(
            "CherrygrovePokecenter1F",
            &tiles.iter().collect::<Vec<_>>(),
            geometry,
            profiles,
        )
    }
    #[test]
    fn complete_machine_claims_only_its_drawing_and_appends_original_ground() {
        let object = profile("Shared Pokecenter healing machine");
        let (tiles, g) = fixture(&object);
        let refs: Vec<_> = tiles.iter().collect();
        let placements = resolve_fixture(&tiles, &g, None);
        let machine = placements
            .iter()
            .find(|p| p.kind == ModelKind::HealingMachine)
            .unwrap();
        assert_eq!(machine.indices(g.width).count(), 16);
        let mut claimed = vec![false; tiles.len()];
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; tiles.len()];
        mesh.footing_heights = vec![0.0; tiles.len()];
        assert!(append(&mut mesh, &refs, &g, machine, &mut claimed, [0, 0]));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 16);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            16
        );
        assert!(mesh.solid.positions.len() > 100);
        assert_eq!(mesh.textured.quad_count(), 16);
        let size = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &refs, &g, machine, &mut claimed, [0, 0]));
        assert_eq!(mesh.solid.positions.len(), size);
        assert!(mesh.footing_heights.iter().all(|&v| v == 0.0));
    }
    #[test]
    fn department_fourth_floor_fixtures_use_their_actual_floor_sample() {
        for name in ["Shared Mart Ecruteak Mart refrigerator 0",
            "Shared Mart Ecruteak Mart refrigerator 2",
            "Shared Mart Ecruteak Mart shelf 0", "Shared Mart Ecruteak Mart shelf 2"] {
            let object = profile(name);
            let expected = profile_kind(&object).unwrap();
            let (mut tiles, geometry) = fixture(&object);
            for tile in &mut tiles {
                if tile.source.metatile_id == 0 {
                    tile.source.tile_index = 0x01;
                }
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            for map in ["CeladonDeptStore4F", "GoldenrodDeptStore4F"] {
                let placements = resolve(map, &cells, &geometry, None);
                let placement = placements.iter().find(|p| p.kind == expected).unwrap();
                assert_eq!(cells[placement.ground].source.tile_index, 0x01);
            }
        }
    }
    #[test]
    fn changed_phase_wrong_art_and_missing_ground_are_unclaimed() {
        let object = profile("Shared Pokecenter healing machine");
        let (tiles, g) = fixture(&object);
        let mut wrong = tiles.clone();
        wrong[0].source.subtile_column = 1;
        assert!(
            resolve_fixture(&wrong, &g, None)
                .iter()
                .all(|p| p.kind != ModelKind::HealingMachine)
        );
        let mut wrong = tiles.clone();
        wrong[0].source.tile_index = u16::MAX;
        assert!(
            resolve_fixture(&wrong, &g, None)
                .iter()
                .all(|p| p.kind != ModelKind::HealingMachine)
        );
        let mut missing = tiles.clone();
        for tile in &mut missing {
            if tile.source.tile_index == object.ground {
                tile.source.tile_index = u16::MAX;
            }
        }
        assert!(resolve_fixture(&missing, &g, None).is_empty());
        let (_, mut clipped) = fixture(&object);
        clipped.width = 3;
        clipped.height = 3;
        let clipped_tiles: Vec<_> = tiles.iter().take(9).cloned().collect();
        assert!(
            resolve_fixture(&clipped_tiles, &clipped, None)
                .iter()
                .all(|p| p.kind != ModelKind::HealingMachine)
        );
    }
    #[test]
    fn changed_live_profile_keeps_user_geometry_ownership() {
        let mut object = profile("Shared Pokecenter complete PC");
        let (tiles, g) = fixture(&object);
        object.depth_pixels += 1.0;
        let custom = Document {
            objects: vec![object],
            atmosphere: None,
        };
        assert!(
            resolve_fixture(&tiles, &g, Some(&custom))
                .iter()
                .all(|p| p.kind != ModelKind::Computer)
        );
    }
    #[test]
    fn all_bound_profiles_have_specific_materialed_asset_roles() {
        let profiles: Vec<_> = built_in_profiles()
            .objects
            .iter()
            .filter(|p| profile_kind(p).is_some())
            .collect();
        assert!(profiles.len() >= 80);
        for p in profiles {
            let kind = profile_kind(p).unwrap();
            assert!(kind.label().starts_with("interior/"));
            let (depth, rise) = proportions(kind, p.tiles[0].len(), p.tiles.len());
            assert!(depth > 0.0 && depth <= p.tiles.len() as f32 * 8.0);
            assert!(rise > 0.0 && rise.is_finite());
        }
    }
}

#[cfg(test)]
#[path = "cable_fixture_tests.rs"]
mod cable_fixture_tests;

include!("interior_signatures.rs");

include!("interior_surfaces.rs");
include!("enhanced_room_floor.rs");
include!("casino_interiors.rs");
include!("cianwood_walls.rs");

include!("lighthouse_chamber_floor.rs");

include!("bedroom_carpet.rs");

include!("interior_variation.rs");

include!("prize_booth.rs");
