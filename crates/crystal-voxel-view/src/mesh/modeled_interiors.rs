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
        TrashCan => (10.,8.),
        Stool | ArcadeStool => (12.0, 7.0),
        Chair | LinkSeat => (12.0, 14.0),
        Cushion => (15.0, 3.0),
        DiningTable => (source_depth, 10.0),
        LowTable => (source_depth, 6.0),
        CafeTable => (24.0, 16.0),
        BedRed | BedBlue | BedGreen | BedPink | BedFeathery | BedPolkadot | BedPikachu => {
            (source_depth, 11.0)
        }
        PottedPlant | PlantMagna | PlantTropic | PlantJumbo => (12.0, 23.0),
        WallDomestic | WallTraditional | WallClinical | WallAcoustic | ArcadeWall => {
            (3.0, source_depth.max(24.0))
        }
        PictureFrame => (2.0, source_depth),
        ArcadePrizeRack => (10.0, 28.0),
        OlivineStage => (64.0, 8.0),
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
fn kanto_retail_room(map:&str)->bool {
    matches!(map,"CeruleanMart"|"FuchsiaMart"|"LavenderMart"|"PewterMart"
        |"SaffronMart"|"VermilionMart"|"ViridianMart"
        |"CeladonDeptStore1F"|"CeladonDeptStore2F"|"CeladonDeptStore3F"
        |"CeladonDeptStore4F"|"CeladonDeptStore5F"|"CeladonDeptStore6F")
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
                            || kanto_native_floor_map(map)
                            || kanto_retail_room(map)
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
                    (12.0,7.0)
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
                (12.0,7.0)
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
    if map=="CeladonCafe" {
        add(grouped_flat_card_placements(cells,geometry,0x01,false,|s| {
            crate::cafe::celadon_table_local(map,s).map(|(x,y)|(x,y,4,4))
        }),&|_|ModelKind::CafeTable);
    }
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
        let native_player=kanto_native_room(map) && s.tileset_id.as_ref()=="players_house";
        if native_player && !native_player_dining_table_complete(cells,geometry,table.column,table.row) {continue;}
        let p = Placement {
            kind: if s.tileset_id.as_ref() == "traditional_house" {
                ModelKind::LowTable
            } else {
                ModelKind::DiningTable
            },
            column: table.column,
            row: table.row,
            width: 4,
            height: if native_player {5}else{4},
            ground,
            depth_pixels: 32.0,
            // The old source extrusion height is not the imported model's
            // scale. Use the same dining proportions as profile-resolved tables.
            height_pixels: if s.tileset_id.as_ref()=="traditional_house" {
                table.height_pixels
            } else {proportions(ModelKind::DiningTable,4,4).1},
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
    append_cafe_architecture_candidates(map,cells,geometry,&mut claimed,&mut out);
    append_cianwood_walls(map, cells, geometry, &mut claimed, &mut out);
    append_signature_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_stair_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_player_house_wall_fragments(map, cells, geometry, &mut claimed, &mut out);
    out
}

// These are the remaining gold wall courses beside the PC and stairs. Match
// the full drawing and phase, then reuse the same closed native wall geometry.
fn append_player_house_wall_fragments(map: &str, cells: &[&VisualTile],
    g: &GridGeometry, claimed: &mut [bool], out: &mut Vec<Placement>) {
    if !kanto_native_room(map) { return; }
    let Some(ground)=ground_sample(cells,map,"players_house",crate::players_house::FLOOR_TILE) else {return;};
    for (block,ox,oy,w,h) in [(0x0b,0,0,2,2),(0x10,0,0,4,1),(0x10,2,1,2,1)] {
        if w>g.width || h>g.height {continue;}
        for row in 0..=g.height-h {for column in 0..=g.width-w {
            if row<oy {continue;}
            let complete=(0..h).all(|y|(0..w).all(|x| {
                let i=(row+y)*g.width+column+x;let s=&cells[i].source;
                !claimed[i] && s.tileset_id.as_ref()=="players_house"
                    && s.metatile_id==block && s.tile_index==0x11
                    && s.subtile_column as usize==ox+x && s.subtile_row as usize==oy+y
            }));
            if !complete {continue;}
            let p=Placement{kind:ModelKind::WallDomestic,column,row,width:w,height:h,
                ground,depth_pixels:3.,height_pixels:h as f32*8.,
                base_pixels:(2-oy-h) as f32*8.,footing_pixels:None};
            for i in p.indices(g.width) {claimed[i]=true;}
            out.push(p);
        }}
    }
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

fn append_cafe_architecture_candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    claimed:&mut[bool],out:&mut Vec<Placement>) {
    if map!="CeladonCafe" {return;}
    let Some(ground)=ground_sample(cells,map,"game_corner",1) else{return;};
    for (i,tile) in cells.iter().enumerate() {
        let source=&tile.source;
        if source.tileset_id.as_ref()!="game_corner" || source.subtile_column!=0 || source.subtile_row!=0 {continue;}
        let (column,row)=(i%g.width,i/g.width);
        let mut piece=|kind,rect:[usize;4],expected:&[u16]| {
            let [x,y,w,h]=rect;if column+x+w>g.width||row+y+h>g.height {return;}
            if !(0..h).all(|dy|(0..w).all(|dx| {
                let index=(row+y+dy)*g.width+column+x+dx;let s=&cells[index].source;
                !claimed[index]&&s.tileset_id.as_ref()=="game_corner"&&s.metatile_id==source.metatile_id
                    &&s.subtile_column as usize==x+dx&&s.subtile_row as usize==y+dy
                    &&s.tile_index==expected[dy*w+dx]
            })) {return;}
            let wall=kind==ModelKind::ArcadeWall;
            let p=Placement{kind,column:column+x,row:row+y,width:w,height:h,ground,
                depth_pixels:if wall {3.}else{10.},height_pixels:if wall {h as f32*8.}else if kind==ModelKind::TrashCan {8.}else{28.},
                base_pixels:if wall {(2-y-h)as f32*8.}else{0.},footing_pixels:None};
            for index in p.indices(g.width) {claimed[index]=true;}out.push(p);
        };
        match source.metatile_id {
            0x04=>piece(ModelKind::ArcadePrizeRack,[0,0,4,3],&casino_drawing(4).unwrap()[..12]),
            0x29=>{piece(ModelKind::ArcadeWall,[0,0,4,2],&[2;8]);
                piece(ModelKind::TrashCan,[2,2,2,2],&[0x46,0x47,0x56,0x57]);},
            0x02|0x2d=>piece(ModelKind::ArcadeWall,[0,0,4,2],&[2;8]),
            0x13=>{piece(ModelKind::ArcadeWall,[0,0,2,1],&[2;2]);
                piece(ModelKind::ArcadeWall,[2,0,2,2],&[2;4]);},
            0x28=>piece(ModelKind::ArcadeWall,[0,0,4,2],&[2,2,6,7,2,2,8,9]),
            _=>{},
        }
    }
}

fn native_player_dining_table_complete(cells:&[&VisualTile],g:&GridGeometry,column:usize,row:usize)->bool {
    let object=built_in_profiles().objects.iter().find(|o|o.name=="PlayersHouse1F dining table").unwrap();
    let h=object.tiles.len();let w=object.tiles[0].len();
    if column+w>g.width || row+h>g.height {return false;}
    (0..h).all(|y|(0..w).all(|x| {
        let s=&cells[(row+y)*g.width+column+x].source;
        let ox=object.origin[0]as usize+x;let oy=object.origin[1]as usize+y;
        s.tileset_id.as_ref()=="players_house" && s.subtile_column as usize==ox%4
            && s.subtile_row as usize==oy%4 && s.tile_index==object.tiles[y][x]
            && s.metatile_id==object.metatiles.as_ref().unwrap()[oy/4][ox/4]
    }))
}

fn native_house_window_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::WindowWall && p.width==2 && p.height==2
        && INTERIOR_DRAWINGS.iter().any(|d|d.kind==ModelKind::WindowWall
            && matches!(d.tileset,"house"|"players_house") && d.size==[p.width,p.height]
            && p.indices(g.width).enumerate().all(|(j,i)| {
                let s=&cells[i].source;
                s.tileset_id.as_ref()==d.tileset && s.metatile_id==d.block
                    && s.subtile_column==d.origin[0]+(j%p.width)as u8
                    && s.subtile_row==d.origin[1]+(j/p.width)as u8 && s.tile_index==d.tiles[j]
            }))
}

fn native_house_television_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[usize;2];5]> {
    if p.kind!=ModelKind::Television || p.width!=2 || !(2..=3).contains(&p.height) {return None;}
    let source=&cells[p.row*g.width+p.column].source;
    if p.height==2 {
        return (p.indices(g.width).enumerate().all(|(j,i)|
            crate::players_house::tv_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8))))
            .then_some([[3,2],[3,3],[1,3],[3,11],[3,2]]);
    }
    let timber=match source.tileset_id.as_ref() {
        "house" if source.metatile_id==0x1e && p.indices(g.width).enumerate().all(|(j,i)|
            crate::house::upright_fixture_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8,2,3)))=>[3,23],
        "players_house" if source.metatile_id==0x11 && p.indices(g.width).enumerate().all(|(j,i)|
            crate::players_house::upright_fixture_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8,2,3)))=>[1,16],
        "players_house" if source.metatile_id==2 && p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="players_house" && s.metatile_id==2
                && s.subtile_column==2+(j%2)as u8 && s.subtile_row==1+(j/2)as u8
                && s.tile_index==[6,7,0x16,0x17,0x18,0x19][j]
        })=>[2,18],
        _=>return None,
    };
    Some([timber,[3,3],[1,3],[3,11],[3,2]])
}

fn native_house_dining_table(map:&str,cells:&[&VisualTile],g:&GridGeometry,p:&Placement)->bool {
    kanto_native_room(map) && p.kind==ModelKind::DiningTable && p.width==4
        && ((p.height==4 && p.indices(g.width).all(|i|cells[i].source.tileset_id.as_ref()=="house")
            && house_table_placements(cells,g).iter().any(|t|t.column==p.column && t.row==p.row))
            || (p.height==5 && native_player_dining_table_complete(cells,g,p.column,p.row)))
}

/// Append only after successful resolution. The original immutable ground
/// atlas supplies every vacated pixel, with no fabricated floor or collision.
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
    world_origin: [i32; 2],
) -> bool {
    if p.indices(geometry.width).any(|i| claimed[i]) {
        return false;
    }
    let Some(selected) = crate::interior_models::selected_model(p.kind) else {
        append_missing_source(mesh, geometry, p, claimed);
        return true;
    };
    let piece_source=&cells[p.row*geometry.width+p.column].source;
    if kanto_native_room(map) && native_computer_source(p,cells,geometry).is_some()
        && !selected.has_native_computer_materials() {
        append_missing_source(mesh,geometry,p,claimed);
        return true;
    }
    if kanto_native_room(map)
        && matches!(p.kind,ModelKind::Bookcase|ModelKind::LabBookcase)
        && matches!((piece_source.tileset_id.as_ref(),piece_source.tile_index),
            ("house",0x32)|("players_house",0x0e)|("lab",0x05)|("mansion",0x36))
        && p.width%2==0 && p.height>=2
        && selected.native_bookcase_structural_mask().is_none() {
        append_missing_source(mesh,geometry,p,claimed);
        return true;
    }
    if kanto_native_room(map) && native_house_television_source(p,cells,geometry).is_some()
        && !selected.has_native_television_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_house_dining_table(map,cells,geometry,p) && !selected.has_native_dining_table_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if p.kind==ModelKind::TrashCan && !selected.has_native_trashcan_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
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
    let piece_source = &cells[p.row * geometry.width + p.column].source;
    // The arcade's one- and two-row drawings are one north wall course.
    // Drawing height must not turn its panels into staggered freestanding posts.
    let arcade_poster = p.kind == ModelKind::PictureFrame
            && piece_source.tileset_id.as_ref() == "game_corner"
            && piece_source.metatile_id == 0x28;
    let arcade_wall_plane = p.kind == ModelKind::ArcadeWall || arcade_poster;
    let native_gate_wall=kanto_native_floor_map(map) && p.kind==ModelKind::WallClinical
        && piece_source.tileset_id.as_ref()=="gate" && (1..=3).contains(&p.height)
        && piece_source.subtile_row==0;
    let native_gate_doors=kanto_native_floor_map(map) && p.kind==ModelKind::GateDoorFrame
        && p.width==4 && p.height==3 && p.indices(geometry.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="gate" && s.metatile_id==4
                && s.subtile_column==j as u8%4 && s.subtile_row==j as u8/4
                && s.tile_index==[2,3,2,3,0x12,0x13,0x12,0x13,0x11,0x11,0x11,0x11][j]
        });
    let native_cafe_wall=map=="CeladonCafe"&&p.kind==ModelKind::ArcadeWall
        && piece_source.tileset_id.as_ref()=="game_corner"&&p.height<=2
        && matches!(piece_source.metatile_id,2|0x13|0x28|0x29|0x2d);
    let native_player_wall=kanto_native_room(map) && p.kind==ModelKind::WallDomestic
        && piece_source.tileset_id.as_ref()=="players_house"
        && p.indices(geometry.width).all(|i|cells[i].source.tile_index==0x11);
    let south_row = if native_cafe_wall {
        p.row-usize::from(piece_source.subtile_row)+2
    } else if native_house_dining_table(map,cells,geometry,p) && p.height==5 {
        p.row+4
    } else if native_player_wall {
        p.row-usize::from(piece_source.subtile_row)+2
    } else if native_gate_wall {
        // The authored upper fragments carry a 16px forward offset; two-row
        // planter fragments carry 8px. All join the same three-row course.
        p.row+3
    } else if arcade_wall_plane {
        p.row - usize::from(piece_source.subtile_row) + 1
    } else if p.kind==ModelKind::WallTraditional && piece_source.tileset_id.as_ref()=="tower"
        && piece_source.tile_index==0x41 && p.depth_pixels>p.height as f32*8. {
        p.row+(p.depth_pixels/8.) as usize
    } else {
        p.row + p.height
    };
    let poster_relief = if arcade_poster { 0.2 * geometry.tile_height / 8.0 } else { 0.0 };
    let south = geometry.origin_z + south_row as f32 * geometry.tile_height + poster_relief;
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
    // The guarded 2f/30 lower band is already covered by its complete upper
    // course. Retain its source claim/underlay without a duplicate wall volume.
    let repeats = if native_cafe_wall {1} else if p.kind == ModelKind::ArcadeWall && piece_source.subtile_row == 1 {
        debug_assert!(matches!(piece_source.metatile_id, 0x2f | 0x30));
        0
    } else if (matches!(
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
            let source_uv = |pixel: [usize; 2]| {
                let pixel = if source.tile_index == 0x41 { pixel } else {
                    match pixel {
                        [0, 0] => [0, 0], [0, 2] => [1, 3],
                        [0, 1] => [0, 1], [4, 0] => [2, 0],
                        _ => unreachable!("validated timber materials"),
                    }
                };
                let uv = geometry.uv(p.column, p.row);
                [uv.0 + (uv.1 - uv.0) * (pixel[0] as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * (pixel[1] as f32 + 0.5) / 8.]
            };
            if source.tile_index == 0x41 {
                let facing = if source.subtile_column < 2 { 1.0 } else { -1.0 };
                append_side_wall(&mut mesh.textured,
                    [left, right, north, south], p.height_pixels * geometry.tile_height / 8.0,
                    facing, geometry.tile_width, source_uv);
            } else {
                crate::dungeon_models::model(crate::dungeon_models::Kind::TimberWall).append_tower_timber(
                    &mut mesh.textured, [left, right, north, south],
                    p.height_pixels * geometry.tile_height / 8.0, source_uv);
            }
        } else if native_gate_wall || native_gate_doors {
            append_native_gate_wall(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if p.kind==ModelKind::TrashCan {
            let uv=geometry.uv(p.column,p.row);
            let swatches=[[6.,1.],[2.,2.],[1.,1.]].map(|v|[uv.0+(uv.1-uv.0)*(v[0]+0.5)/8.,uv.2+(uv.3-uv.2)*(v[1]+0.5)/8.]);
            let center=(left+right)*0.5;let half=5.*geometry.tile_width/8.;
            assert!(selected.append_native_trashcan(&mut mesh.textured,[center-half,center+half,north,south],8.*geometry.tile_height/8.,swatches),"native shared trashcan requires named source materials");
        } else if native_cafe_wall {
            assert!(p.height<=2 && p.base_pixels>=0.,"native cafe wall requires source course bounds");
            append_kanto_house_wall(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if kanto_native_room(map) && native_house_window_source(p,cells,geometry) {
            append_native_house_window(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if kanto_native_room(map) && p.kind==ModelKind::WallDomestic
            && (p.height==2 || native_player_wall) && p.indices(geometry.width).all(|i| {
                let s=&cells[i].source;
                matches!((s.tileset_id.as_ref(),s.tile_index),
                    ("house",0) | ("players_house",0x11|0x32|0x33|0x34))
            }) {
            append_kanto_house_wall(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if kanto_native_floor_map(map) && map.ends_with("Pokecenter1F")
            && p.kind==ModelKind::LinkSeat && p.width==2 && p.height==2
            && p.indices(geometry.width).enumerate().all(|(j,i)| {
                let source=&cells[i].source;
                source.tileset_id.as_ref()=="pokecenter"
                    && matches!(source.metatile_id,0x2e|0x2f)
                    && source.tile_index==[0x48,0x49,0x58,0x59][j]
            }) {
            append_native_center_lounge_seat(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,north,south]);
        } else if map=="CeladonCafe" && p.kind==ModelKind::ArcadeStool && p.width==2 && p.height==2
            && p.indices(geometry.width).enumerate().all(|(j,i)| {
                let s=&cells[i].source;
                s.tileset_id.as_ref()=="game_corner" && matches!(s.metatile_id,5|6)
                    && s.subtile_column==if s.metatile_id==5 {2+j as u8%2}else{j as u8%2}
                    && s.subtile_row==piece_source.subtile_row+j as u8/2
                    && s.tile_index==[0x0a,0x0b,0x1a,0x1b][j]
            }) {
            append_native_rectangular_stool(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,north,south]);
        } else if (kanto_native_room(map) || map=="CeladonCafe")
            && p.kind==ModelKind::Stool && p.width==2 && p.height==2
            && piece_source.tileset_id.as_ref()=="house"
            && p.indices(geometry.width).enumerate().all(|(j,i)| {
                crate::house::furniture_local(&cells[i].source)
                    ==Some(((j%2) as u8,(j/2) as u8,crate::house::FurnitureKind::Stool))
            }) {
            append_native_rectangular_stool(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,north,south]);
        } else if map=="SoulHouse" && p.kind==ModelKind::Memorial
            && p.width==2 && p.height==2 && p.indices(geometry.width).enumerate().all(|(j,i)| {
                crate::house::soul_house_bench_local(&cells[i].source)
                    ==Some(((j%2) as u8,(j/2) as u8))
            }) {
            // All three swatches belong to this complete memorial's upper-left
            // tile37. Its gold body, brown recess and pale trim remain distinct.
            let uv=geometry.uv(p.column,p.row);
            let sample=|pixel:[usize;2]|[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.];
            assert!(selected.append_native_memorial(&mut mesh.textured,
                [left+inset,right-inset,north,south],
                p.base_pixels*geometry.tile_height/8.,p.height_pixels*geometry.tile_height/8.,
                [sample([1,1]),sample([4,3]),sample([4,1])]));
        } else if p.width==2 && p.height==4 && piece_source.tileset_id.as_ref()=="mart"
            && ((p.kind==ModelKind::RetailShelf && piece_source.metatile_id==0x13)
                || (kanto_retail_room(map) && matches!((p.kind,piece_source.metatile_id,piece_source.tile_index),
                    (ModelKind::RetailShelf,0x12|0x2c,0x26)
                    |(ModelKind::RetailShelf,0x15|0x27,0x0c)
                    |(ModelKind::RetailRefrigerator,0x14,0x0c)))) {
            append_native_retail_display(&mut mesh.textured,geometry,p,
                [left,right,north,south],piece_source.tile_index==0x26);
        } else if p.kind == ModelKind::Cushion
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref()=="mart" {
            append_store_seat(&mut mesh.solid,[left+inset,right-inset,north,south],geometry.tile_height/8.0);
        } else if p.kind == ModelKind::DiningTable && p.width == 4 && p.height == 3
            && piece_source.tileset_id.as_ref() == "game_corner" && piece_source.metatile_id == 0x2f {
            append_arcade_game_table(&mut mesh.textured, geometry, p, [left,right,north,south]);
        } else if map=="CeladonCafe" && p.kind==ModelKind::CafeTable {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8) as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8) as f32+0.5)/8.]
            };
            let scale=geometry.tile_height/8.;
            assert!(selected.append_native_cafe_table(&mut mesh.textured,
                [left+inset,right-inset,north,south],16.*scale,
                [sample([4,6]),sample([12,26]),sample([14,22])]));
            append_cafe_basket(&mut mesh.textured,[(left+right)*0.5,(north+south)*0.5],
                scale,[sample([11,5]),sample([13,2]),sample([15,4])]);
        } else if kanto_native_room(map)
            && let Some(swatches)=native_house_television_source(p,cells,geometry) {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
            };
            let bounds=[left+inset,right-inset,north,south];
            let height=p.height_pixels*geometry.tile_height/8.;
            let appended=if p.height==2 {
                selected.append_native_television_screen(&mut mesh.textured,bounds,height,swatches.map(sample))
            } else {selected.append_native_television(&mut mesh.textured,bounds,height,swatches.map(sample))};
            assert!(appended,"native television requires cabinet and CRT material roles");
        } else if native_house_dining_table(map,cells,geometry,p) {
            // Verified table tile26: medium-gold timber and darker gold cloth.
            // Use the complete object's palette, retaining its selected geometry.
            let uv=geometry.uv(p.column,p.row);
            let sample=|pixel:[usize;2]|[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.];
            assert!(selected.append_native_dining_table(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                if piece_source.tileset_id.as_ref()=="players_house" {
                    [sample([2,1]),sample([2,2])]
                } else {[sample([7,2]),sample([5,2])]}),"native complete square dining table requires named wood and cloth roles");
        } else if p.kind == ModelKind::ArcadeStool {
            let cx = (left + right) * 0.5;
            let cz = geometry.origin_z + (p.row as f32 + p.height as f32 * 0.5) * geometry.tile_height;
            let radius_x = 4.0 * geometry.tile_width / 8.0;
            let radius_z = 4.0 * geometry.tile_height / 8.0;
            selected.append_fitted(&mut mesh.solid,
                [cx - radius_x, cx + radius_x, cz - radius_z, cz + radius_z],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if p.kind == ModelKind::ArcadePair {
            append_outward_arcade_pair(&mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if arcade_poster {
            // A sign is attached to a closed wall, not suspended over a hole.
            // Its exact casino drawing owns the backing without a new claim.
            let wall_south = south - poster_relief;
            model(ModelKind::ArcadeWall).append_fitted(&mut mesh.solid,
                [left, right, wall_south - 3.0 * geometry.tile_height / 8.0, wall_south],
                0.0, 28.0 * geometry.tile_height / 8.0);
            model(ModelKind::ArcadeWall).apply_arcade_wall_finish(&mut mesh.solid, first);
            selected.append_fitted(&mut mesh.solid,
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
        } else if kanto_native_room(map)
            && let Some(swatches)=native_computer_source(p,cells,geometry) {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*(pixel[0]%8)as f32/8.+(uv.1-uv.0)/16.,
                 uv.2+(uv.3-uv.2)*(pixel[1]%8)as f32/8.+(uv.3-uv.2)/16.]
            };
            assert!(selected.append_native_computer(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.base_pixels*geometry.tile_height/8.,
                p.height_pixels*geometry.tile_height/8.,
                swatches.map(sample)),
                "designated native computer source requires named material roles");
        } else if kanto_native_room(map)
            && matches!(p.kind, ModelKind::Bookcase | ModelKind::LabBookcase)
            && matches!((piece_source.tileset_id.as_ref(),piece_source.tile_index),
                ("house",0x32) | ("players_house",0x0e) | ("lab",0x05) | ("mansion",0x36))
            && p.width % 2 == 0 && p.height >= 2 {
            // Swatches belong to this repeated cabinet's top and front, not
            // to a global guessed wood palette or a neighboring floor tile.
            let uv_at = |pixel: [usize;2]| {
                let uv = geometry.uv(p.column + unit*2 + pixel[0]/8,
                    p.row + pixel[1]/8);
                [uv.0+(uv.1-uv.0)*(pixel[0]%8) as f32/8.+(uv.1-uv.0)/16.,
                 uv.2+(uv.3-uv.2)*(pixel[1]%8) as f32/8.+(uv.3-uv.2)/16.]
            };
            let light = if piece_source.tileset_id.as_ref()=="players_house" {[1,8]} else {[3,3]};
            let mut cabinet=SurfaceMeshData::default();
            let appended = selected.append_native_bookcase(&mut cabinet,
                [left+inset,right-inset,north,south],
                p.base_pixels*geometry.tile_height/8.,p.height_pixels*geometry.tile_height/8.,
                [uv_at([0,8]),uv_at([2,8]),uv_at(light)]);
            assert!(appended,"canonical cabinet must retain all named structural materials");
            let structural=selected.native_bookcase_structural_mask().unwrap();
            let mut remap=vec![u32::MAX;cabinet.positions.len()];
            let marked=vec![false;cabinet.positions.len()];
            // Only structural triangles sample the map atlas. Authored books
            // use the untextured material, preserving their individual colors.
            for (textured,target) in [(true,&mut mesh.textured),(false,&mut mesh.solid)] {
                let selected=select_surface_indices(&cabinet,&marked,&mut remap,
                    cabinet.indices.chunks_exact(3).filter(|tri| {
                        assert!(tri.iter().all(|i|structural[*i as usize]==structural[tri[0]as usize]));
                        structural[tri[0]as usize]==textured
                    }).flatten().copied());
                let offset=target.positions.len() as u32;
                target.positions.extend(selected.positions);target.normals.extend(selected.normals);
                target.uvs.extend(selected.uvs);target.colors.extend(selected.colors);
                target.indices.extend(selected.indices.into_iter().map(|i|i+offset));
            }
        } else {
            selected.append_fitted(
                &mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0,
            );
        }
        if p.kind == ModelKind::ArcadeWall && !native_cafe_wall {
            selected.apply_arcade_wall_finish(&mut mesh.solid, first);
        } else if matches!(p.kind, ModelKind::PrizeBooth | ModelKind::ArcadePrizeRack) {
            selected.apply_room_finish(&mut mesh.solid, first);
        }
        if p.kind == ModelKind::PottedPlant
            && matches!(cells[p.row * geometry.width + p.column].source.tileset_id.as_ref(), "mart" | "game_corner") {
            selected.apply_room_palette(&mut mesh.solid, first);
        }
        if cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "game_corner" {
            if p.kind == ModelKind::ArcadeMachine
                && !matches!(cells[p.row * geometry.width + p.column].source.metatile_id, 0x07 | 0x0b) {
                selected.apply_arcade_finish(&mut mesh.solid, first);
            } else if p.kind == ModelKind::ReceptionCounter {
                selected.apply_arcade_finish(&mut mesh.solid, first);
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

include!("cafe_table.rs");

/// Preserve the profile's raised upper/planter fragments and shared 22px crest.
/// Their full source guard is resolved before this native palette path runs.
fn append_native_gate_wall(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    b:[f32;4]) {
    let [w,e,n,s]=b;let scale=g.tile_height/8.;
    let crop=if p.height==1{0.}else{2.};
    let base=match p.height{1=>14.,2=>8.,3=>0.,_=>unreachable!()}*scale;
    let top=22.*scale;
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,base,top,n,s],[1.;3]);
    let uv=g.uv(p.column,p.row);
    let cap=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*1.5/8.];
    out.uvs[first..].fill(cap);
    out.indices.drain(indices+6..indices+12);
    for y in 0..p.height {for x in 0..p.width {
        let uv=g.uv(p.column+x,p.row+y);
        let clipped=if y==0{crop}else{0.};
        let high=top-(y as f32*8.-crop).max(0.)*scale;
        let low=top-((y+1) as f32*8.-crop)*scale;
        let left=w+x as f32*g.tile_width;let right=left+g.tile_width;
        let v0=uv.2+(uv.3-uv.2)*clipped/8.;
        append_quad(out,[[right,low,s],[right,high,s],[left,high,s],[left,low,s]],
            [0.,0.,1.],[[uv.1,uv.3],[uv.1,v0],[uv.0,v0],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }}
}

/// The lobby drawing is a low upholstered stool, not a backed link-club chair.
/// Four short feet and a closed cushion retain its own yellow/ochre palette;
/// only the cushion's upper drawing is mapped onto the horizontal top.
// Native stools are compact rectangular seats. The upper source row owns
// their cushion/frame; the lower row owns the two drawn feet and shadow.
fn native_computer_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[usize;2];3]> {
    if p.kind!=ModelKind::Computer {return None;}
    let source=&cells[p.row*g.width+p.column].source;
    if source.tileset_id.as_ref()=="players_room" && p.width==2 && p.height==3
        && p.indices(g.width).enumerate().all(|(j,i)| {
            if j<4 {crate::interior::player_room_pc_monitor_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8))}
            else {crate::interior::player_room_pc_keyboard_local(&cells[i].source)==Some(((j%2)as u8,0))}
        }) {return Some([[3,0],[0,0],[3,1]]);}
    let drawing=INTERIOR_DRAWINGS.iter().find(|d|d.kind==ModelKind::Computer
        && matches!(d.tileset,"house"|"players_house") && source.tileset_id.as_ref()==d.tileset
        && d.size==[p.width,p.height] && source.metatile_id==d.block
        && p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()==d.tileset && s.metatile_id==d.block
                && s.subtile_column==d.origin[0]+(j%p.width)as u8
                && s.subtile_row==d.origin[1]+(j/p.width)as u8 && s.tile_index==d.tiles[j]
        }))?;
    Some(if drawing.tiles[0]==0x11 {[[7,0],[0,0],[14,0]]}else{[[3,0],[0,0],[3,1]]})
}

fn append_native_rectangular_stool(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    bounds:[f32;4]) {
    let [left,right,north,south]=bounds;let scale=g.tile_height/8.;
    let [w,e,n,s]=[left+2.*scale,right-2.*scale,north+scale,south-scale];
    let texel=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8) as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8) as f32+0.5)/8.]
    };
    let foot=texel(6,9);let side=texel(4,2);
    for x in [w+scale,e-2.*scale] {for z in [n+scale,s-2.*scale] {
        let first=out.positions.len();
        architecture_box(out,[x,x+scale,0.,3.*scale,z,z+scale],[1.;3]);
        out.uvs[first..].fill(foot);
    }}
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,3.*scale,5.*scale,n,s],[1.;3]);
    out.uvs[first..].fill(side);out.indices.drain(indices..indices+6);
    // Crop only the seat drawing, keeping the atlas seam between02 and03.
    // The lower12/13 foot drawing does not get stretched across upholstery.
    for x in 0..2 {
        let sx0=(x*8).max(1);let sx1=((x+1)*8).min(15);
        let a=w+(e-w)*(sx0-1) as f32/14.;let b=w+(e-w)*(sx1-1) as f32/14.;
        let uv=g.uv(p.column+x,p.row);
        let u0=uv.0+(uv.1-uv.0)*(sx0-x*8) as f32/8.;
        let u1=uv.0+(uv.1-uv.0)*(sx1-x*8) as f32/8.;
        let v0=uv.2+(uv.3-uv.2)/8.;let v1=uv.2+(uv.3-uv.2)*7./8.;
        append_quad(out,[[a,5.*scale,n],[a,5.*scale,s],[b,5.*scale,s],[b,5.*scale,n]],
            [0.,1.,0.],[[u0,v0],[u0,v1],[u1,v1],[u1,v0]],[1.;4]);
    }
}

fn append_native_center_lounge_seat(out: &mut SurfaceMeshData,g: &GridGeometry,
    p: &Placement,bounds: [f32;4]) {
    let [w,e,n,s]=bounds;let scale=g.tile_height/8.;
    let texel=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8) as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8) as f32+0.5)/8.]
    };
    let foot=texel(7,12);let side=texel(1,3);
    for x in [w+scale,e-3.*scale] {for z in [n+scale,s-3.*scale] {
        let first=out.positions.len();
        architecture_box(out,[x,x+2.*scale,0.,3.*scale,z,z+2.*scale],[1.;3]);
        out.uvs[first..].fill(foot);
    }}
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,3.*scale,7.*scale,n,s],[1.;3]);
    out.uvs[first..].fill(side);
    // The original lower four pixels describe feet, not cushion upholstery.
    // Replace the box top with the cropped native upper pattern, conserving
    // source tile UVs independently on either side of each atlas boundary.
    out.indices.drain(indices..indices+6);
    for y in 0..2 {for x in 0..2 {
        let sx0=(x*8).max(1);let sx1=((x+1)*8).min(15);
        let sy0=(y*8).max(1);let sy1=((y+1)*8).min(11);
        let left=w+(e-w)*(sx0-1) as f32/14.;
        let right=w+(e-w)*(sx1-1) as f32/14.;
        let north=n+(s-n)*(sy0-1) as f32/10.;
        let south=n+(s-n)*(sy1-1) as f32/10.;
        let uv=g.uv(p.column+x,p.row+y);
        let u0=uv.0+(uv.1-uv.0)*(sx0-x*8) as f32/8.;
        let u1=uv.0+(uv.1-uv.0)*(sx1-x*8) as f32/8.;
        let v0=uv.2+(uv.3-uv.2)*(sy0-y*8) as f32/8.;
        let v1=uv.2+(uv.3-uv.2)*(sy1-y*8) as f32/8.;
        append_quad(out,[[left,7.*scale,north],[left,7.*scale,south],
            [right,7.*scale,south],[right,7.*scale,north]],[0.,1.,0.],
            [[u0,v0],[u0,v1],[u1,v1],[u1,v0]],[1.;4]);
    }}
}

/// Closed native domestic wall course, selected only from complete house art.
fn append_kanto_house_wall(out: &mut SurfaceMeshData,g: &GridGeometry,p: &Placement,
    bounds: [f32;4]) {
    let [west,east,north,south]=bounds;
    // The native two-row wall and adjacent window are one 16-pixel course.
    // Keep a closed volume behind its actual horizontal banded drawing.
    let base=p.base_pixels*g.tile_height/8.;
    let height=base+p.height as f32*g.tile_height;
    let first=out.positions.len();let index_first=out.indices.len();
    architecture_box(out,[west,east,base,height,north,south],[1.;3]);
    let uv=g.uv(p.column,p.row);
    let body=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
    out.uvs[first..].fill(body);
    // Replace the plain front with the source tiles, without coplanar faces.
    out.indices.drain(index_first+6..index_first+12);
    for y in 0..p.height {for x in 0..p.width {
        let left=west+x as f32*g.tile_width;let right=left+g.tile_width;
        let top=height-y as f32*g.tile_height;let bottom=top-g.tile_height;
        let uv=g.uv(p.column+x,p.row+y);
        append_quad(out,[[right,bottom,south],[right,top,south],
            [left,top,south],[left,bottom,south]],[0.,0.,1.],
            [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }}
}

fn append_native_house_window(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4]) {
    let [w,e,n,s]=b;let index=out.indices.len();
    // Reuse the closed domestic wall volume and replace only its front course.
    append_kanto_house_wall(out,g,p,[w,e,n,s]);
    out.indices.truncate(index+5*6);
    let (u0,_,v0,_)=g.uv(p.column,p.row);
    let (_,u1,_,v1)=g.uv(p.column+p.width-1,p.row+p.height-1);
    let height=p.height as f32*g.tile_height;
    let relief=g.tile_height*0.06;
    assert!(height>0. && e>w && s>n,"native curtain wall requires complete source bounds");
    let point=|x:f32,y:f32| {
        let inset=if x==0.||x==1.||y==0.||y==1. {0.}
            else {relief*(0.75+0.25*(x*std::f32::consts::TAU*4.).cos())};
        [w+(e-w)*x,height*(1.-y),s-inset]
    };
    // The original curtains and neighboring frame keep their exact UVs,
    // while recessed folds add depth without detaching from the wall course.
    native_source_surface_patches(out,&[0.,0.125,0.25,0.375,0.5,0.625,0.75,0.875,1.],
        &[0.,0.5,1.],point,|x,y|[u0+(u1-u0)*x,v0+(v1-v0)*y],|_,_|1.,false);
}

#[cfg(test)]
mod kanto_native_wall_tests {
    use super::*;
    #[test]
    fn kanto_native_house_wall_keeps_closed_source_course_and_shared_seams() {
        let g=GridGeometry{width:8,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=Placement{kind:ModelKind::WallDomestic,column:0,row:0,width:4,height:2,
            ground:0,depth_pixels:3.,height_pixels:24.,base_pixels:0.,footing_pixels:None};
        let mut wall=SurfaceMeshData::default();
        append_kanto_house_wall(&mut wall,&g,&p,[0.,32.,13.,16.]);
        assert_eq!(wall.indices.len(),(5+8)*6,"five closed sides and eight source front tiles");
        for ids in wall.indices.chunks_exact(3) {
            let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(wall.positions[i as usize]));
            let normal=Vec3::from_array(wall.normals[ids[0]as usize]);
            assert!((b-a).cross(c-a).dot(normal)>0.,"outward closed faces");
        }
        assert!(wall.positions.iter().all(|v|v[1]>=0. && v[1]<=16.));
        assert_eq!(proportions(ModelKind::WindowWall,2,2).1,16.);
        let mut adjacent=p.clone();adjacent.column=4;
        let mut other=SurfaceMeshData::default();
        append_kanto_house_wall(&mut other,&g,&adjacent,[32.,64.,13.,16.]);
        let seam=|mesh:&SurfaceMeshData| {
            let mut vertices=mesh.positions.iter().filter(|v|v[0]==32.).map(|v|(v[1]as i32,v[2]as i32)).collect::<Vec<_>>();
            vertices.sort();vertices.dedup();vertices
        };
        assert_eq!(seam(&wall),seam(&other));
        assert!(wall.uvs.iter().any(|uv|uv[0]>0. && uv[1]>0.));
    }
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

// A closed display case with native glass/equipment fronts and recessed bays.
// Complete source drawings choose its original merchandise, not a generic book model.
fn append_native_retail_display(mesh:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],blue_case:bool) {
    let [w,e,n,s]=b;let height=p.height_pixels*g.tile_height/8.;
    let sample=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]
    };
    let first=mesh.positions.len();let index=mesh.indices.len();
    architecture_box(mesh,[w,e,0.,height,n,s],[1.;3]);
    // Paint the closed casing from this display, with its native front split
    // at the original cell boundaries. No book merchandise is substituted.
    let casing=if blue_case {
        // Blue aisle units have a mid-blue color at the retained source anchor;
        // electronics keep their original light cap. The native texture still
        // paints each individual glass/front section over the closed geometry.
        sample(6,3)
    } else {sample(1,1)};
    for i in first..mesh.positions.len() {mesh.uvs[i]=casing;}
    let faces=mesh.indices.split_off(index);
    mesh.indices.extend(faces.chunks_exact(3)
        .filter(|tri|mesh.normals[tri[0]as usize][2]<0.5).flatten().copied());
    // Four recessed bays meet raised shelf lips and a beveled outer frame.
    // All shared knots coincide: the inset front seals the same casing rather
    // than putting floating graphics or intersecting boxes in front of it.
    let xs=[0.,0.06,0.12,0.5,0.88,0.94,1.];
    let ys:&[f32]=if p.kind==ModelKind::RetailRefrigerator {
        &[0.,0.04,0.20,0.25,0.29,0.5,0.75,0.95,1.]
    } else {&[0.,0.04,0.20,0.25,0.29,0.45,0.5,0.54,0.70,0.75,0.79,0.95,1.]};
    let (u0,_,v0,_)=g.uv(p.column,p.row);
    let (_,u1,_,v1)=g.uv(p.column+p.width-1,p.row+p.height-1);
    let point=|x:f32,y:f32| {
        let recessed=if p.kind==ModelKind::RetailRefrigerator {y>0.25 && y<1.}
            else {![0.,0.25,0.5,0.75,1.].contains(&y)};
        let inset=if x>=0.12 && x<=0.88 && recessed {0.8*g.tile_height/8.} else {0.};
        [w+(e-w)*x,height*(1.-y),s-inset]
    };
    for y in 0..ys.len()-1 {for x in 0..xs.len()-1 {
        let corners=[[xs[x],ys[y]],[xs[x],ys[y+1]],
            [xs[x+1],ys[y+1]],[xs[x+1],ys[y]]];
        for tri in [[0,1,2],[0,2,3]] {
            let coords=tri.map(|i|corners[i]);let points=coords.map(|[x,y]|point(x,y));
            let normal=(Vec3::from_array(points[1])-Vec3::from_array(points[0]))
                .cross(Vec3::from_array(points[2])-Vec3::from_array(points[0])).normalize();
            let first=mesh.positions.len()as u32;
            mesh.positions.extend(points);mesh.normals.extend([normal.to_array();3]);
            mesh.uvs.extend(coords.map(|[x,y]|[u0+(u1-u0)*x,v0+(v1-v0)*y]));
            mesh.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
            mesh.indices.extend([first,first+1,first+2]);
        }
    }}
}

/// Reserve missing furniture against legacy extrusion, displaying its exact
/// source atlas cells. This makes missing coverage visible without changing
/// actor footing, gameplay state or claiming an authored model label.
fn append_missing_source(mesh:&mut TerrainMeshData,g:&GridGeometry,p:&Placement,claimed:&mut [bool]) {
    for i in p.indices(g.width) {
        append_top(&mut mesh.textured,g.bounds(i%g.width,i/g.width).into(),0.,g.uv(i%g.width,i/g.width));
        claimed[i]=true;
        mesh.authored_cells[i]=None;
    }
}

fn append_store_seat(mesh:&mut SurfaceMeshData,b:[f32;4],scale:f32) {
    let [left,right,north,south]=b;
    let cx=(left+right)*0.5;let cz=(north+south)*0.5;
    let hx=(right-left)*0.425;let hz=(south-north)*0.425;
    let [w,e,n,s]=[cx-hx,cx+hx,cz-hz,cz+hz];
    let first=mesh.colors.len();
    model(ModelKind::Cushion).append_fitted(mesh,[w,e,n,s],5.0*scale,2.0*scale);
    // Named upholstery primitives own their paint and weave; never infer a
    // material from its color. Unknown primitives retain authored placement.
    model(ModelKind::Cushion).apply_room_palette(mesh, first);
    model(ModelKind::Cushion).apply_room_finish(mesh, first);
    let x=(e-w)*0.10;let z=(s-n)*0.12;
    architecture_box(mesh,[w+x,e-x,4.3*scale,5.0*scale,n+z,s-z],[0.28,0.29,0.30]);
    for cx in [w+x,e-x-x*0.65] {for cz in [n+z,s-z-z*0.65] {
        architecture_box(mesh,[cx,cx+x*0.65,0.0,4.3*scale,cz,cz+z*0.65],[0.32,0.34,0.35]);
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
    fn kanto_native_cafe_stools_share_rectangular_seats_with_their_own_gray_art() {
        for name in ["Shared game corner stool 5 0","Shared game corner stool 5 2",
            "Shared game corner stool 6 0","Shared game corner stool 6 2"] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("CeladonCafe",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::ArcadeStool).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,"CeladonCafe",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=5.));
            assert_eq!(mesh.footing_heights,vec![1.75;tiles.len()]);
            let mut arcade=TerrainMeshData::default();
            assert!(append(&mut arcade,"CeladonGameCorner",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(!arcade.solid.positions.is_empty());
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::ArcadeStool));
        }
    }
    #[test]
    fn kanto_native_selected_house_bookcase_uses_wood_source_and_keeps_authored_books() {
        let mut object=profile("Shared facility paired bookcases 06");
        object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
        object.tileset="house".into();object.metatile=4;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![0x32,0x33,0x32,0x33],vec![0x30,0x31,0x0e,0x0f],
            vec![0x0e,0x0f,0x0e,0x0f],vec![0x3c,0x3b,0x1e,0x1f]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("BluesHouse",&refs,&g,None);
        let books=placements.iter().filter(|p|p.kind==ModelKind::Bookcase).collect::<Vec<_>>();
        assert!(!books.is_empty());
        let mut mesh=TerrainMeshData{authored_cells:vec![None;tiles.len()],footing_heights:vec![2.5;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        for p in books {
            assert!(append(&mut mesh,"BluesHouse",&refs,&g,p,&mut claimed,[0,0]));
            assert!(p.indices(g.width).all(|i|mesh.authored_cells[i]==Some("interior/bookcase")));
        }
        assert!(!mesh.solid.positions.is_empty(),"selected books retain authored materials");
        assert!(!mesh.textured.positions.is_empty(),"cabinet uses its source wood swatch");
        assert!(mesh.solid.colors.iter().any(|c|c[0]!=c[1] && c[1]!=c[2]));
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
    }
    #[test]
    fn kanto_native_bedroom_pc_keeps_complete_source_claim_and_own_palette() {
        let mut object=profile("Player bedroom computer feet");
        object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
        object.tileset="players_house".into();object.metatile=0x10;object.origin=[0,1];object.ground=1;
        object.tiles=vec![vec![0x20,0x21],vec![0x30,0x31],vec![0x0c,0x0d]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("RedsHouse2F",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::Computer).unwrap();
        let mut mesh=TerrainMeshData{authored_cells:vec![None;tiles.len()],footing_heights:vec![2.5;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"RedsHouse2F",&refs,&g,p,&mut claimed,[0,0]));
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
        assert!(p.indices(g.width).all(|i|claimed[i] && mesh.authored_cells[i]==Some("interior/computer")));
        for uv in mesh.textured.uvs.iter().skip(4*p.width*p.height) {
            assert!(uv[0]<2./g.width as f32 && uv[1]<3./g.height as f32);
        }
        let mut clipped=tiles.clone();clipped[0].source.tile_index=0x0a;
        assert!(!resolve("RedsHouse2F",&clipped.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::Computer));
    }
    #[test]
    fn missing_selected_interior_keeps_source_atlas_and_prevents_legacy_extrusion() {
        let g=GridGeometry{width:3,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=Placement{kind:ModelKind::Chair,column:1,row:0,width:2,height:2,
            ground:0,depth_pixels:16.,height_pixels:12.,base_pixels:0.,footing_pixels:None};
        let mut mesh=TerrainMeshData{footing_heights:vec![3.;6],authored_cells:vec![None;6],..Default::default()};
        let mut claimed=vec![false;6];
        append_missing_source(&mut mesh,&g,&p,&mut claimed);
        assert_eq!(claimed,vec![false,true,true,false,true,true]);
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![3.;6]);
        assert_eq!(mesh.authored_cells,vec![None;6]);
        assert_eq!(mesh.textured.positions.len(),16);
        for (face,i) in p.indices(g.width).enumerate() {
            let (u0,u1,v0,v1)=g.uv(i%g.width,i/g.width);
            assert_eq!(&mesh.textured.uvs[face*4..face*4+4],&[[u0,v0],[u0,v1],[u1,v1],[u1,v0]]);
        }
    }
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
    fn kanto_native_lab_cabinet_routes_structure_to_atlas_and_respects_source_ownership() {
        let mut object=profile("Shared lab paired bookcases 14");
        let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("OaksLab",&refs,&g,None);
        let cabinet=placements.iter().find(|p|p.kind==ModelKind::LabBookcase).unwrap();
        let build=|map| {
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,cabinet,&mut vec![false;tiles.len()],[0,0]));
            mesh
        };
        let native=build("OaksLab");let ordinary=build("ElmsLab");
        assert!(native.textured.indices.len()>ordinary.textured.indices.len());
        assert!(native.solid.indices.len()<ordinary.solid.indices.len());
        assert_eq!(native.textured.indices.len()+native.solid.indices.len(),
            ordinary.textured.indices.len()+ordinary.solid.indices.len());
        assert_eq!(native.footing_heights,ordinary.footing_heights);
        assert!(native.solid.uvs.iter().all(|uv|*uv==[0.,0.]),"books must retain untextured paint");
        object.name="User cabinet".into();
        assert!(!resolve("OaksLab",&refs,&g,Some(&Document{objects:vec![object],atmosphere:None}))
            .iter().any(|p|p.kind==ModelKind::LabBookcase));
        tiles[0].source.tile_index=0xffff;
        assert!(!resolve("OaksLab",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::LabBookcase && p.column==0));
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
        assert!(append(&mut mesh,"GoldenrodGameCorner",&cells,&g,&p[0],&mut claimed,[0,0]));
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
            assert!(append(&mut mesh,map,&refs,&g,&plants[0],&mut vec![false;tiles.len()],[0,0]));
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
    fn store_electronics_preserve_both_native_fronts_on_closed_cabinets() {
        let mut object=profile("Shared department store cushion 2");
        object.metatile=0x13;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![0x40,0x41,0x42,0x2b],vec![0x50,0x51,0x52,0x45],
            vec![0x43,0x44,0x5c,0x5d],vec![0x53,0x54,0x1f,0x55]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["GoldenrodDeptStore3F","CeladonDeptStore3F"] {
            let displays=resolve(map,&refs,&g,None);
            let cabinets=displays.iter().filter(|p|p.kind==ModelKind::RetailShelf).collect::<Vec<_>>();
            assert_eq!(cabinets.len(),2);
            for p in cabinets {
                let mut mesh=SurfaceMeshData::default();
                append_native_retail_display(&mut mesh,&g,p,[0.,16.,0.,10.],false);
                let mut volume=0.;let mut front_area=0.;
                for tri in mesh.indices.chunks_exact(3) {
                    let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                    let cross=(points[1]-points[0]).cross(points[2]-points[0]);
                    assert!(cross.dot(Vec3::from_array(mesh.normals[tri[0]as usize]))>0.);
                    volume+=points[0].dot(points[1].cross(points[2]))/6.;
                    if mesh.normals[tri[0]as usize][2]>0.5 {front_area+=cross.z/2.;}
                }
                assert!(volume>16.*10.*30.*0.90 && volume<16.*10.*30.);
                assert!((front_area-16.*30.).abs()<0.01);
                assert!(mesh.positions.iter().any(|p|p[2]>9. && p[2]<10.));
                for y in 0..4 {for x in 0..2 {
                    let uv=g.uv(p.column+x,p.row+y);
                    assert!(mesh.uvs.contains(&[uv.0,uv.2]));
                    assert!(mesh.uvs.contains(&[uv.1,uv.3]));
                }}
            }
        }
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
            assert_eq!((seats[0].depth_pixels,seats[0].height_pixels),(12.,7.));
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
            assert_eq!(seat.height_pixels,7.0);
            assert_eq!(seat.depth_pixels,12.0);
            let mut mesh=TerrainMeshData {footing_heights:vec![2.5;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,"GoldenrodDeptStore1F",&cells,&g,seat,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            let top=mesh.solid.positions.iter().map(|p|p[1]).fold(0.0,f32::max);
            assert!((top-7.0).abs()<0.001);
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
    fn kanto_native_cafe_trashcan_requires_complete_drawing_and_keeps_footing() {
        let mut object=profile("Shared lab round stool");object.tileset="game_corner".into();
        object.metatile=0x29;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![2;4],vec![2;4],vec![1,1,0x46,0x47],vec![1,1,0x56,0x57]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("CeladonCafe",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::TrashCan).unwrap();
        assert_eq!((p.column,p.row,p.width,p.height),(2,2,2,2));
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];assert!(append(&mut mesh,"CeladonCafe",&refs,&g,&p,&mut claimed,[0,0]));
        assert!(mesh.solid.positions.is_empty());assert!(mesh.textured.positions.iter().any(|p|p[1]>7.));
        assert!(mesh.textured.positions.iter().all(|p|p[0]>=16.&&p[0]<=32.&&p[2]>=16.&&p[2]<=32.&&p[1]<=8.+1e-5));
        assert!(p.indices(g.width).all(|i|claimed[i]));assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
        assert!(resolve("OlivineCafe",&refs,&g,None).iter().all(|p|p.kind!=ModelKind::TrashCan));
        tiles[2*g.width+2].source.tile_index=0xffff;
        assert!(resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::TrashCan));
    }

    #[test]
    fn kanto_native_cafe_uses_shared_complete_display_and_connected_gold_wall() {
        for (block,drawing,kind) in [
            (4,vec![vec![0x16,0x17,0x17,0x18],vec![0x26,0x27,0x27,0x28],vec![0x36,0x37,0x37,0x38]],ModelKind::ArcadePrizeRack),
            (2,vec![vec![2;4],vec![2;4]],ModelKind::ArcadeWall),
            (0x28,vec![vec![2,2,6,7],vec![2,2,8,9]],ModelKind::ArcadeWall),
        ] {
            let mut object=profile("Shared lab round stool");object.tileset="game_corner".into();
            object.metatile=block;object.metatiles=None;object.origin=[0,0];object.ground=1;object.tiles=drawing;
            let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("CeladonCafe",&refs,&g,None);let p=placements.iter().find(|p|p.kind==kind).unwrap();
            assert_eq!(p.width,4);assert_eq!(p.height,if block==4 {3}else{2});
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"CeladonCafe",&refs,&g,p,&mut claimed,[0,0]));
            assert!(p.indices(g.width).all(|i|claimed[i]));assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
            if kind==ModelKind::ArcadeWall {
                assert!(mesh.solid.positions.is_empty());
                assert!(mesh.textured.positions.iter().all(|p|p[0]>=0.&&p[0]<=32.&&p[1]<=16.));
                assert_eq!(mesh.textured.indices.len(),4*2*6+(5+4*2)*6,"one closed course, no repeated overlapping panels");
            } else {assert!(!mesh.solid.indices.is_empty(),"complete shelf must be a real shared cabinet");}
            tiles[0].source.tile_index=0xffff;
            assert!(resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=kind));
            let mut other=Vec::new();
            append_cafe_architecture_candidates("OlivineCafe",&tiles.iter().collect::<Vec<_>>(),&g,&mut vec![false;tiles.len()],&mut other);
            assert!(other.is_empty());
        }
    }

    #[test]
    fn kanto_native_house_windows_keep_curtain_source_and_closed_wall_course() {
        for (atlas,block,origin,drawing,map) in [
            ("house",5,[2,0],vec![vec![0x24,0x4a],vec![0x34,0x2c]],"BluesHouse"),
            ("players_house",1,[0,0],vec![vec![0x2c,0x2d],vec![0x11,0x11]],"RedsHouse1F"),
        ] {
            let mut object=profile("Shared lab round stool");object.tileset=atlas.into();
            object.metatile=block;object.metatiles=None;object.origin=origin;object.ground=1;
            object.tiles=drawing;let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind==ModelKind::WindowWall).unwrap();
            assert!(native_house_window_source(p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty(),"aqua substitute cannot remain");
            assert!(mesh.textured.positions.iter().all(|p|p[1]>=0.&&p[1]<=16.));
            assert!(mesh.textured.positions.iter().any(|p|p[2]>15.&&p[2]<16.),"curtain relief must remain inside closed wall");
            for ids in mesh.textured.indices[4*6..].chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let normal=Vec3::from_array(mesh.textured.normals[ids[0]as usize]);
                assert!((b-a).cross(c-a).dot(normal)>0.);
            }
            let (u0,_,v0,_)=g.uv(p.column,p.row);
            let (_,u1,_,v1)=g.uv(p.column+1,p.row+1);
            assert!(mesh.textured.uvs.contains(&[u0,v0]));assert!(mesh.textured.uvs.contains(&[u1,v1]));
            assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::WindowWall));
        }
    }

    #[test]
    fn kanto_native_televisions_use_own_blue_crt_and_gold_stand_source() {
        for (atlas,block,origin,tileset_tiles,map) in [
            ("house",0x1e,[0,1],vec![vec![6,7],vec![0x16,0x17],vec![0x1e,0x1f]],"BluesHouse"),
            ("players_house",2,[2,1],vec![vec![6,7],vec![0x16,0x17],vec![0x18,0x19]],"RedsHouse2F"),
            ("players_house",0x11,[0,1],vec![vec![6,7],vec![0x16,0x17],vec![8,9]],"RedsHouse1F"),
            ("players_house",0x13,[2,0],vec![vec![6,7],vec![0x16,0x17]],"RedsHouse2F"),
        ] {
            let mut object=profile("Shared lab round stool");object.tileset=atlas.into();
            object.metatile=block;object.metatiles=None;object.origin=origin;object.ground=1;
            object.tiles=tileset_tiles;let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind==ModelKind::Television).unwrap();
            let swatches=native_house_television_source(p,&refs,&g).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty(),"imported pale wood TV must not remain");
            for (role,pixel) in swatches.into_iter().enumerate() {
                if p.height==2 && role==0 {continue;}
                let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                let sample=[uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                    uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.];
                assert!(mesh.textured.uvs[2*p.height*4..].contains(&sample));
            }
            assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::Television));
        }
    }

    #[test]
    fn kanto_native_red_dining_table_claims_complete_feet_and_keeps_square_top() {
        let object=profile("PlayersHouse1F dining table");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve("RedsHouse1F",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
        assert_eq!((p.width,p.height),(4,5));
        let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"RedsHouse1F",&refs,&g,p,&mut claimed,[0,0]));
        assert!(p.indices(g.width).all(|i|claimed[i]));
        assert!(mesh.solid.positions.is_empty());
        let vertices=&mesh.textured.positions[4*5*4..];
        let min=vertices.iter().fold([f32::INFINITY;3],|a,p|std::array::from_fn(|i|a[i].min(p[i])));
        let max=vertices.iter().fold([f32::NEG_INFINITY;3],|a,p|std::array::from_fn(|i|a[i].max(p[i])));
        assert!((max[2]-min[2]-32.).abs()<0.001,"source feet cannot elongate the physical top");
        assert!(max[2]<=32. && min[2]>=0.);
        assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
        tiles[4*g.width].source.tile_index=0xffff;
        assert!(resolve("RedsHouse1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::DiningTable));
        let mut clipped=g.clone();clipped.height=4;
        assert!(!native_player_dining_table_complete(&tiles.iter().collect::<Vec<_>>(),&clipped,0,0));
    }

    #[test]
    fn kanto_native_dining_table_uses_shared_model_scale_instead_of_legacy_extrusion() {
        let object=profile("Neighbor dining table");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("BluesHouse",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
        assert_eq!((p.width,p.height),(4,4));assert_eq!(p.height_pixels,10.);
        let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        assert!(append(&mut mesh,"BluesHouse",&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
        let table_vertices=&mesh.textured.positions[4*4*4..];
        let min=table_vertices.iter().fold([f32::INFINITY;3],|a,p|std::array::from_fn(|i|a[i].min(p[i])));
        let max=table_vertices.iter().fold([f32::NEG_INFINITY;3],|a,p|std::array::from_fn(|i|a[i].max(p[i])));
        assert!((max[1]-min[1]-10.).abs()<0.001);
        assert!((max[0]-min[0]).min(max[2]-min[2])>31.,"complete table cannot collapse into a narrow strip");
        assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
        assert!(mesh.solid.positions.is_empty(),"pink imported finish cannot remain beside native geometry");
        let uv=g.uv(p.column,p.row);
        for pixel in [[7,2],[5,2]] {
            let sample=[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.];
            assert!(mesh.textured.uvs[4*4*4..].contains(&sample));
        }
        tiles[0].source.tile_index=0xffff;
        assert!(resolve("BluesHouse",&tiles.iter().collect::<Vec<_>>(),&g,None).iter()
            .all(|p|p.kind!=ModelKind::DiningTable),"altered table must remain source art");
    }

    #[test]
    fn kanto_native_player_wall_fragments_join_course_without_claiming_pc_or_stairs() {
        for block in [0x0b,0x10] {
            let mut object=profile("Shared lab round stool");
            object.tileset="players_house".into();object.metatile=block;
            object.metatiles=None;object.origin=[0,0];object.ground=crate::players_house::FLOOR_TILE;
            object.tiles=if block==0x10 {vec![vec![0x11;4],vec![0x20,0x21,0x11,0x11],
                vec![0x30,0x31,1,1],vec![0x0c,0x0d,1,1]]}
                else {vec![vec![0x11,0x11,0x50,0x51],vec![0x11,0x11,0x60,0x61],vec![1;4],vec![1;4]]};
            let (mut tiles,g)=fixture(&object);
            let mut claimed=vec![false;tiles.len()];let mut placements=Vec::new();
            append_player_house_wall_fragments("RedsHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,&mut claimed,&mut placements);
            assert_eq!(placements.len(),if block==0x10 {2}else{1});
            assert_eq!(claimed.iter().filter(|c|**c).count(),if block==0x10 {6}else{4});
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            let mut rendered=vec![false;tiles.len()];
            for p in &placements {
                assert!(append(&mut mesh,"RedsHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,p,&mut rendered,[0,0]));
            }
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.positions.iter().any(|v|v[1]==16. && v[2]==16.));
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert!(!claimed[2*g.width],"PC and staircase cells remain independently owned");
            let p=&placements[0];tiles[p.row*g.width+p.column].source.tile_index=0xffff;
            let mut altered=Vec::new();
            append_player_house_wall_fragments("RedsHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,&mut vec![false;tiles.len()],&mut altered);
            assert_eq!(altered.len(),placements.len()-1,"altered source cannot acquire a wall");
            let mut johto=Vec::new();
            append_player_house_wall_fragments("PlayersHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,&mut vec![false;tiles.len()],&mut johto);
            assert!(johto.is_empty());
        }
    }

    #[test]
    fn kanto_native_house_stools_keep_rectangular_source_seat_and_low_authored_height() {
        let mut object=profile("Shared lab round stool");
        object.name="Source house stool".into();object.tileset="house".into();
        object.metatile=0x0c;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.map=None;object.maps=None;object.parts.clear();
        object.tiles=vec![vec![2,3],vec![0x12,0x13]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["BillsHouse","TrainerHouse1F"] {
            let placements=resolve(map,&refs,&g,None);
            let seat=placements.iter().find(|p|p.kind==ModelKind::Stool).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,seat,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty(),"native stool must not retain coral upholstery");
            assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
            assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
            assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=5.));
            let top=mesh.textured.indices.chunks_exact(3).filter(|tri|
                tri.iter().all(|i|mesh.textured.positions[*i as usize][1]==5.)).map(|tri| {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                (points[1]-points[0]).cross(points[2]-points[0]).y*0.5
            }).sum::<f32>();
            assert!((top-11.28*10.).abs()<0.01,"seat must close its compact rectangular footprint");
            for x in 0..2 {let uv=g.uv(seat.column+x,seat.row);
                assert!(mesh.textured.uvs.iter().any(|p|p[0]>=uv.0 && p[0]<=uv.1
                    && p[1]==uv.2+(uv.3-uv.2)*7./8.));
            }
        }
        let seat=resolve("BillsHouse",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Stool).unwrap();
        let mut other=TerrainMeshData::default();
        assert!(append(&mut other,"EcruteakHouse",&refs,&g,&seat,&mut vec![false;tiles.len()],[0,0]));
        assert!(!other.solid.indices.is_empty(),"unrelated stool styling remains untouched");
        let custom=Document{objects:vec![object.clone()],atmosphere:None};
        assert!(!resolve("BillsHouse",&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Stool));
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("BillsHouse",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::Stool));
    }

    #[test]
    fn kanto_native_soul_house_memorials_keep_own_gold_palette_and_separate_claims() {
        for name in ["Soul House memorial 28 0","Soul House memorial 28 2",
            "Soul House memorial 2a 0","Soul House memorial 2a 2",
            "Soul House memorial 2b 0","Soul House memorial 2b 2"] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("SoulHouse",&refs,&g,Some(built_in_profiles()));
            let memorial=placements.iter().find(|p|p.kind==ModelKind::Memorial).unwrap();
            assert_eq!((memorial.width,memorial.height),(2,2));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,"SoulHouse",&refs,&g,memorial,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert_eq!(claims.iter().filter(|&&c|c).count(),4);
            let uv=g.uv(memorial.column,memorial.row);
            for pixel in [[1,1],[4,3],[4,1]] {
                assert!(mesh.textured.uvs.contains(&[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                    uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.]));
            }
            let mut other=TerrainMeshData::default();
            assert!(append(&mut other,"OtherHouse",&refs,&g,memorial,
                &mut vec![false;tiles.len()],[0,0]));
            let ground_vertices=other.textured.positions.len();
            let ground_indices=other.textured.indices.len();
            assert_eq!(mesh.textured.positions[..ground_vertices],other.textured.positions);
            assert_eq!(mesh.textured.positions[ground_vertices..],other.solid.positions);
            assert_eq!(mesh.textured.normals[ground_vertices..],other.solid.normals);
            assert_eq!(mesh.textured.indices[ground_indices..].iter()
                .map(|i|*i-ground_vertices as u32).collect::<Vec<_>>(),other.solid.indices);
            let mut changed=object.clone();changed.name="Custom memorial".into();
            let custom=Document{objects:vec![changed],atmosphere:None};
            assert!(!resolve("SoulHouse",&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Memorial));
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("SoulHouse",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::Memorial));
        }
    }

    #[test]
    fn kanto_native_retail_displays_keep_blue_glass_and_equipment_sources_without_generic_goods() {
        let mut blue=profile("Shared department store cushion 2");
        blue.name="Custom blue display".into();blue.metatile=0x12;blue.metatiles=None;
        blue.origin=[2,0];blue.ground=1;blue.map=None;blue.maps=None;blue.parts.clear();
        blue.tiles=vec![vec![0x26,0x27],vec![0x36,0x37],vec![0x28,0x29],vec![0x38,0x39]];
        let mut crossing=blue.clone();crossing.metatile=0x2c;crossing.origin=[2,2];
        crossing.metatiles=Some(vec![vec![0x2c],vec![0x2d]]);
        for (object,kind) in [(blue,ModelKind::RetailShelf),(crossing,ModelKind::RetailShelf),
            (profile("Shared Mart Ecruteak Mart refrigerator 0"),ModelKind::RetailRefrigerator),
            (profile("Shared Mart Ecruteak Mart shelf 0"),ModelKind::RetailShelf)] {
            let (mut tiles,g)=fixture(&object);
            // Department stores use tile 01 beneath these shared Mart fixtures;
            // the canonical ordinary-Mart profile supplies tile 48 instead.
            tiles.last_mut().unwrap().source.tile_index=crate::mart::FLOOR_TILE;
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("CeladonDeptStore2F",&refs,&g,Some(built_in_profiles()));
            let cabinet=placements.iter().find(|p|p.kind==kind)
                .unwrap_or_else(||panic!("complete native display missing: {}",object.name));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,"CeladonDeptStore2F",&refs,&g,cabinet,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty(),"native source fronts replace generic merchandise paint");
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert_eq!(claims.iter().filter(|&&c|c).count(),8);
            for y in 0..4 {for x in 0..2 {
                let uv=g.uv(x,y);
                assert!(mesh.textured.uvs.contains(&[uv.0,uv.2]));
                assert!(mesh.textured.uvs.contains(&[uv.1,uv.3]));
            }}
            let front=(cabinet.row+cabinet.height) as f32*g.tile_height;
            assert!(mesh.textured.positions.iter().any(|p|p[2]>front-1. && p[2]<front),"recessed front volume");
            let mut johto=TerrainMeshData::default();
            assert!(append(&mut johto,"EcruteakMart",&refs,&g,cabinet,&mut vec![false;tiles.len()],[0,0]));
            assert!(!johto.solid.indices.is_empty(),"unrelated model treatments stay unchanged");
            let mut changed=object.clone();changed.name="New customized retail object".into();
            changed.map=None;changed.maps=None;
            let custom=Document{objects:vec![changed],atmosphere:None};
            assert!(!resolve("CeladonDeptStore2F",&refs,&g,Some(&custom)).iter().any(|p|p.kind==kind));
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("CeladonDeptStore2F",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==kind));
            tiles[0].source.tile_index^=1;
            let mut clipped=g.clone();clipped.height=3;tiles.truncate(g.width*3);
            assert!(!resolve("CeladonDeptStore2F",&tiles.iter().collect::<Vec<_>>(),&clipped,None)
                .iter().any(|p|p.kind==kind));
        }
    }

    #[test]
    fn kanto_native_cafe_table_uses_complete_source_and_closed_basket_without_fallback() {
        let mut object=profile("PlayersHouse1F dining table");
        object.name="Customized cafe table".into();object.tileset="game_corner".into();
        object.map=None;object.maps=None;object.metatiles=None;object.metatile=0x2e;
        object.origin=[0,0];object.ground=1;object.parts.clear();object.footing_pixels=None;
        object.tiles=vec![vec![0x1c,0x20,0x21,0x1d],vec![0x1c,0x30,0x31,0x1d],
            vec![0x3e,0x50,0x50,0x3f],vec![0xa8,0x4e,0x4f,0xa9]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("CeladonCafe",&refs,&g,Some(built_in_profiles()));
        let table=placements.iter().find(|p|p.kind==ModelKind::CafeTable).unwrap();
        assert_eq!(table.indices(g.width).count(),16);
        let mut mesh=TerrainMeshData{footing_heights:vec![0.75;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"CeladonCafe",&refs,&g,table,&mut claimed,[0,0]));
        assert_eq!(claimed.iter().filter(|&&c|c).count(),16);
        assert!(mesh.footing_heights.iter().all(|h|*h==0.75));
        assert!(mesh.solid.indices.is_empty());
        assert!(mesh.textured.positions.iter().any(|p|p[1]>18.));
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. &&p[1]<=19.31));
        let mut basket=SurfaceMeshData::default();
        append_cafe_basket(&mut basket,[0.,0.],1.,[[0.1,0.2],[0.3,0.4],[0.5,0.6]]);
        for tri in basket.indices.chunks_exact(3) {
            let [a,b,c]=[0,1,2].map(|j|Vec3::from_array(basket.positions[tri[j] as usize]));
            assert!((b-a).cross(c-a).dot(Vec3::from_array(basket.normals[tri[0] as usize]))>0.);
        }
        let key=|p:[f32;3]|p.map(|v|(v*10000.).round()as i32);
        let mut edges=std::collections::HashMap::new();
        for tri in basket.indices.chunks_exact(3) {for (a,b) in [(0,1),(1,2),(2,0)] {
            let mut edge=[key(basket.positions[tri[a]as usize]),key(basket.positions[tri[b]as usize])];
            edge.sort();*edges.entry(edge).or_insert(0)+=1;
        }}
        assert!(edges.values().all(|count|*count==2),"basket and contents are closed");
        let custom=Document{objects:vec![object.clone()],atmosphere:None};
        assert!(!resolve("CeladonCafe",&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::CafeTable));
        assert!(!resolve("GoldenrodGameCorner",&refs,&g,None).iter().any(|p|p.kind==ModelKind::CafeTable));
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::CafeTable));
        assert_eq!(crate::cafe::shape("CeladonCafe",&tiles[0].source),None);
    }

    #[test]
    fn kanto_native_gate_double_doors_keep_blue_faces_and_join_the_wall_course() {
        let object=profile("Shared gate north double doors");
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("Route5SaffronGate",&refs,&g,Some(built_in_profiles()));
        let doors=placements.iter().find(|p|p.kind==ModelKind::GateDoorFrame).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"Route5SaffronGate",&refs,&g,doors,&mut claimed,[0,0]));
        assert!(mesh.solid.indices.is_empty(),"closed blue doors use their native atlas");
        assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
        assert_eq!(claimed.iter().filter(|&&v|v).count(),12);
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=22.));
        assert!(mesh.textured.positions.iter().any(|p|p[1]==22. && p[2]==24.));
        let front=mesh.textured.indices.chunks_exact(3).filter(|t|
            mesh.textured.normals[t[0] as usize][2]>0.9).map(|t| {
                let p=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                (p[1]-p[0]).cross(p[2]-p[0]).z*0.5
            }).sum::<f32>();
        assert!((front-32.*22.).abs()<0.01,"door panels close the entire wall course");
        for y in 0..3 {for x in 0..4 {
            let uv=g.uv(x,y);
            assert!(mesh.textured.uvs.contains(&[uv.1,uv.3]));
        }}
        let mut other=TerrainMeshData::default();
        assert!(append(&mut other,"Route31VioletGate",&refs,&g,doors,
            &mut vec![false;tiles.len()],[0,0]));
        assert!(!other.solid.indices.is_empty(),"unrelated gate treatment is preserved");
        let mut customized=object.clone();customized.name="Custom gate double doors".into();
        let custom=Document{objects:vec![customized],atmosphere:None};
        assert!(!resolve("Route5SaffronGate",&refs,&g,Some(&custom))
            .iter().any(|p|p.kind==ModelKind::GateDoorFrame));
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("Route5SaffronGate",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::GateDoorFrame));
    }

    #[test]
    fn kanto_native_gate_wall_fragments_keep_source_palette_and_authored_course_offsets() {
        for (name,base) in [("Shared gate wall variant 2d",0.),
            ("Shared gate counter junction upper wall",14.),
            ("Shared gate wall above planters",8.)] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("Route5SaffronGate",&refs,&g,Some(built_in_profiles()));
            let wall=placements.iter().find(|p|p.kind==ModelKind::WallClinical
                && p.column==0 && p.row==0 && p.width==object.tiles[0].len()
                && p.height==object.tiles.len()).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"Route5SaffronGate",&refs,&g,wall,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),wall.width*wall.height);
            let vertices=mesh.textured.indices.chunks_exact(3).filter(|t| {
                mesh.textured.normals[t[0] as usize][1]<0.9
            }).flatten().map(|i|mesh.textured.positions[*i as usize]).collect::<Vec<_>>();
            assert!(vertices.iter().all(|p|p[1]>=base && p[1]<=22.));
            assert!(vertices.iter().any(|p|p[1]==22.));
            assert!(vertices.iter().any(|p|p[2]==24.),"all fronts meet the same native course");
            let mut johto=TerrainMeshData::default();
            assert!(append(&mut johto,"Route31VioletGate",&refs,&g,wall,&mut vec![false;tiles.len()],[0,0]));
            assert!(!johto.solid.indices.is_empty());
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("Route5SaffronGate",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::WallClinical && p.column==0 &&p.row==0
                    && p.width==object.tiles[0].len() && p.height==object.tiles.len()));
        }
    }

    #[test]
    fn kanto_native_center_lounge_seat_keeps_low_cushion_palette_and_source_ownership() {
        let object=profile("Shared Pokecenter seat 2e");
        let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("PewterPokecenter1F",&refs,&g,None);
        let seat=placements.iter().find(|p|p.kind==ModelKind::LinkSeat).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,seat,&mut claimed,[0,0]));
        assert!(mesh.solid.indices.is_empty(),"all lounge surfaces use its own source palette");
        assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
        assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=7.));
        assert!(mesh.textured.positions.iter().any(|p|p[1]==7.));
        assert!(mesh.textured.normals.iter().any(|n|n[1]==-1.),"closed cushion and feet");
        let mut johto=TerrainMeshData::default();
        assert!(append(&mut johto,"CherrygrovePokecenter1F",&refs,&g,seat,&mut vec![false;tiles.len()],[0,0]));
        assert!(!johto.solid.indices.is_empty(),"other rooms retain their existing treatment");
        drop(refs);
        tiles[0].source.tile_index^=1;
        assert!(!resolve("PewterPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::LinkSeat),"changed cushion art is not replaced");
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
        assert!(append(&mut mesh, "CherrygrovePokecenter1F", &refs, &g, machine, &mut claimed, [0, 0]));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 16);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            16
        );
        assert!(mesh.solid.positions.len() > 100);
        assert_eq!(mesh.textured.quad_count(), 16);
        let size = mesh.solid.positions.len();
        assert!(!append(&mut mesh, "CherrygrovePokecenter1F", &refs, &g, machine, &mut claimed, [0, 0]));
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
