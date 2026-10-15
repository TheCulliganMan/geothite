// Source-proven architectural finishes. These are surfaces, not object coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InteriorFloor {
    Oak,
    NativeRoom,
    AmberTimber,
    TowerDepth,
    Tatami,
    Ceramic,
    StoreCeramic,
    PoolCeramic,
    VioletBrick,
    MahoganyIce,
    MahoganyGrip,
    BlackthornTile,
    OlivineTile,
    OlivineRunner,
    ArcadeChecker,
    Stone,
    Marble,
    LighthouseSlate,
    LighthouseChecker,
    GymRose,
    GymGreen,
    GymTimber,
    FuchsiaNative,
    Ship(ShipFloor),
}
impl InteriorFloor {
    fn palette(self) -> [f32; 3] {
        match self {
            Self::Oak => [0.52, 0.40, 0.27],
            Self::AmberTimber => [0.58, 0.34, 0.10],
            Self::TowerDepth => [0.20, 0.14, 0.09],
            Self::Tatami => [0.60, 0.59, 0.37],
            Self::StoreCeramic => [1.0, 1.0, 0.040915],
            Self::PoolCeramic => [0.60, 0.60, 0.61],
            Self::VioletBrick => [0.25, 0.35, 0.64],
            Self::MahoganyIce => [0.48, 0.76, 0.83],
            Self::BlackthornTile => [0.262251, 0.332452, 1.0],
            Self::Ceramic => [0.66, 0.72, 0.69],
            Self::Stone => [0.58, 0.60, 0.57],
            Self::Marble => [0.72, 0.69, 0.61],
            Self::LighthouseSlate => [0.38, 0.43, 0.44],
            Self::LighthouseChecker => [0.54, 0.56, 0.51],
            Self::NativeRoom | Self::GymRose | Self::GymGreen | Self::GymTimber | Self::FuchsiaNative
                | Self::MahoganyGrip | Self::OlivineTile | Self::OlivineRunner | Self::ArcadeChecker =>
                unreachable!("live Gym floor artwork has no generic palette substitute"),
            Self::Ship(style) => style.palette(),
        }
    }
}
fn store_floor_map(map:&str)->bool {
    map.strip_prefix("GoldenrodDeptStore").or_else(||map.strip_prefix("CeladonDeptStore"))
        .is_some_and(|floor|matches!(floor,"1F"|"2F"|"3F"|"4F"|"5F"|"6F"|"Elevator"))
}
// Reviewed Kanto residential rooms and the two Johto player-home floors.
// Palette is read through the room atlas, never guessed from the map name.
fn reviewed_native_room(map: &str) -> bool {
    matches!(map,
        "PlayersHouse1F" | "PlayersHouse2F" | "BillsHouse"
        | "BillsOlderSistersHouse"
        | "BluesHouse"
        | "CeladonMansion1F"
        | "CeladonMansion2F"
        | "CeladonMansion3F"
        | "CeladonMansionRoof"
        | "CeladonMansionRoofHouse"
        | "CeruleanGymBadgeSpeechHouse"
        | "CeruleanPoliceStation"
        | "CeruleanTradeSpeechHouse"
        | "CopycatsHouse1F"
        | "CopycatsHouse2F"
        | "DayOfWeekSiblingsHouse"
        | "LavenderNameRater"
        | "LavenderSpeechHouse"
        | "MrFujisHouse"
        | "MrPsychicsHouse"
        | "OaksLab"
        | "PewterNidoranSpeechHouse"
        | "PewterSnoozeSpeechHouse"
        | "PokemonFanClub"
        | "RedsHouse1F"
        | "RedsHouse2F"
        | "Route12SuperRodHouse"
        | "Route16FuchsiaSpeechHouse"
        | "Route26HealHouse"
        | "Route27SandstormHouse"
        | "Route28SteelWingHouse"
        | "Route2NuggetHouse"
        | "Route5CleanseTagHouse"
        | "SafariZoneWardensHome"
        | "SoulHouse"
        | "TrainerHouse1F"
        | "VermilionDiglettsCaveSpeechHouse"
        | "VermilionFishingSpeechHouse"
        | "VermilionMagnetTrainSpeechHouse"
        | "ViridianNicknameSpeechHouse"
    )
}
pub(crate) fn kanto_native_floor_map(map: &str) -> bool {
    reviewed_native_room(map) || super::gate_counters::reviewed_gate_map(map) || matches!(map,
        "CeladonCafe" | "MountMoonGiftShop" | "SafariZoneMainOffice" | "UndergroundPath" | "VermilionPortPassage" |
        "AzaleaPokecenter1F" | "BlackthornPokecenter1F" | "CherrygrovePokecenter1F" | "CianwoodPokecenter1F" | "EcruteakPokecenter1F" | "GoldenrodPokecenter1F" | "MahoganyPokecenter1F" | "OlivinePokecenter1F" | "Route32Pokecenter1F" | "SilverCavePokecenter1F" | "VioletPokecenter1F" | "CeladonPokecenter1F" | "CeladonPokecenter2FBeta"
        | "CeruleanPokecenter1F" | "CeruleanPokecenter2FBeta"
        | "CinnabarPokecenter1F" | "CinnabarPokecenter2FBeta"
        | "FuchsiaPokecenter1F" | "FuchsiaPokecenter2FBeta"
        | "IndigoPlateauPokecenter1F"
        | "LavenderPokecenter1F" | "LavenderPokecenter2FBeta"
        | "PewterPokecenter1F" | "PewterPokecenter2FBeta"
        | "Route10Pokecenter1F" | "Route10Pokecenter2FBeta"
        | "SaffronPokecenter1F" | "SaffronPokecenter2FBeta"
        | "VermilionPokecenter1F" | "VermilionPokecenter2FBeta"
        | "ViridianPokecenter1F" | "ViridianPokecenter2FBeta"
        | "BlackthornMart" | "CherrygroveMart" | "EcruteakMart" | "OlivineMart" | "VioletMart"
        | "AzaleaMart" | "CeruleanMart" | "FuchsiaMart" | "LavenderMart" | "PewterMart"
        | "SaffronMart" | "VermilionMart" | "ViridianMart"
        | "Route15FuchsiaGate" | "Route16Gate" | "Route17Route18Gate"
        | "Route19FuchsiaGate" | "Route2Gate" | "Route5SaffronGate"
        | "Route5UndergroundPathEntrance" | "Route6SaffronGate"
        | "Route6UndergroundPathEntrance" | "Route7SaffronGate"
        | "Route8SaffronGate" | "SafariZoneFuchsiaGateBeta" | "VictoryRoadGate"
        | "PowerPlant" | "SilphCo1F" | "TrainerHouseB1F"
        | "LavRadioTower1F" | "SaffronMagnetTrainStation")
}
fn cafe_floor_source(source: &VisualTileSource) -> bool {
    if crate::casino::is_floor_cell(source) { return true; }
    if source.tileset_id.as_ref() != "game_corner"
        || source.subtile_column >= 4 || source.subtile_row >= 4 { return false; }
    // Cafe shelf, table and counter blocks reuse the same floor quartet in
    // different strips from the casino. These masks follow the packed layout.
    let mask: u16 = match source.metatile_id {
        0x16 => 0xcccc, 0x17 => 0x00cc, 0x18 => 0x00ff,
        0x19 => 0xff00, 0x29 => 0x3300, 0x2c => 0x0fff, 0x2d => 0x0f00,
        _ => return false,
    };
    let phase = source.subtile_row * 4 + source.subtile_column;
    mask & (1 << phase) != 0
        && source.tile_index == [0x01, 0x10, 0x01, 0x10, 0x11, 0x12, 0x11, 0x12][usize::from(phase % 8)]
}

fn interior_floor(map: &str, source: &VisualTileSource) -> Option<InteriorFloor> {
    use InteriorFloor::*;
    if kanto_native_floor_map(map) && source.tileset_id.as_ref()=="pokecenter"
        && source.tile_index==1 && source.subtile_column<4
        && ((source.subtile_row==2 && (matches!(source.metatile_id,5|6|7) || source.metatile_id==0x38 && source.subtile_column<2
            || map.ends_with("Pokecenter1F") && match source.metatile_id {
                2=>source.subtile_column>=2,3=>source.subtile_column<3,
                8=>source.subtile_column<2,19=>true,42=>source.subtile_column>=1,
                _=>false,
            })) || map.ends_with("Pokecenter1F")&&source.metatile_id==12
                &&source.subtile_row==0&&source.subtile_column>=2) {
        return matches!(shape_for_source_on_map(map,source),CellShape::Flat|CellShape::PlaneAt{height:0.})
            .then_some(NativeRoom);
    }
    if underground_native_floor(map,source) {return Some(NativeRoom);}
    if traditional_room::floor_map(map)&&source.tileset_id.as_ref()=="traditional_house"
        &&source.tile_index==1&&source.subtile_column<4&&source.subtile_row<4 {
        let mask:u16=match source.metatile_id {
            5=>0xfff0,6=>0x33f0,7=>0xccf0,0x2a=>0xffc0,_=>0,
        };
        if mask&(1<<(source.subtile_row*4+source.subtile_column))!=0 {
            return matches!(shape_for_source_on_map(map,source),CellShape::Flat|CellShape::PlaneAt{height:0.})
                .then_some(NativeRoom);
        }
    }
    if traditional_wood_wall::is_map(map)&&source.tileset_id.as_ref()=="traditional_house"
        &&source.tile_index==0x50&&source.subtile_column<4&&source.subtile_row<4
        &&source.subtile_row>=2&&(source.metatile_id==8
            ||source.metatile_id==1&&source.subtile_column>=2) {
        return matches!(shape_for_source_on_map(map,source),CellShape::Flat|CellShape::PlaneAt{height:0.})
            .then_some(NativeRoom);
    }
    if crate::casino::safari_office_floor(map,source) {return Some(NativeRoom);}
    if map=="MahoganyMart1F"&&source.tileset_id.as_ref()=="traditional_house"
        &&source.tile_index==1&&source.subtile_column<4&&source.subtile_row<4
        &&((source.metatile_id==5&&source.subtile_row>=1)||matches!(source.metatile_id,32..=34)) {
        return matches!(shape_for_source_on_map(map,source),CellShape::Flat|CellShape::PlaneAt{height:0.})
            .then_some(NativeRoom);
    }
    if map=="MountMoonGiftShop"&&source.tileset_id.as_ref()=="traditional_house" {
        let valid=source.subtile_column<4&&source.subtile_row<4&&match source.metatile_id {
            4=>source.tile_index==[[68,69,69,70],[84,85,85,86],[69,70,68,69],[85,86,84,85]]
                [source.subtile_row as usize][source.subtile_column as usize],
            16=>source.tile_index==[[68,69,2,3],[84,85,18,19],[69,70,68,69],[85,86,84,85]]
                [source.subtile_row as usize][source.subtile_column as usize]&&!(source.subtile_column>=2&&source.subtile_row<2),
            18=>source.subtile_column>=2&&source.tile_index==[[35,36,69,70],[66,67,85,86],[66,67,68,69],[51,52,84,85]]
                [source.subtile_row as usize][source.subtile_column as usize],
            5=>source.subtile_row>=1&&source.tile_index==1,
            32..=34=>source.tile_index==1,_=>false,
        };
        return (valid&&matches!(shape_for_source_on_map(map,source),CellShape::Flat|CellShape::PlaneAt{height:0.}))
            .then_some(NativeRoom);
    }
    if map == "CeladonCafe" {
        return (cafe_floor_source(source)
            && matches!(shape_for_source_on_map(map, source),
                CellShape::Flat | CellShape::PlaneAt { height: 0. }))
            .then_some(NativeRoom);
    }
    if kanto_native_floor_map(map)
        && matches!((source.tileset_id.as_ref(),source.tile_index),
            ("house"|"players_house"|"players_room",0x01) | ("mansion",0x01|0x11)
            | ("lab",0x10|0x24) | ("pokecenter",0x11) | ("mart",0x01|0x48)
            | ("gate",0x01) | ("facility",0x01|0x26)
            | ("radio_tower",0x01) | ("train_station",0x3d) | ("game_corner",0x01)) {
        return (source.subtile_column<4 && source.subtile_row<4
            && matches!(shape_for_source_on_map(map,source),
                CellShape::Flat|CellShape::PlaneAt{height:0.}))
            .then_some(NativeRoom);
    }
    if map=="FuchsiaGym" {return enhanced_room_floor_source(map,source);}
    // This room's live blue patterned tile is already on its physical floor.
    // A shared station stone finish would discard both its palette and motif,
    // including the verified blue donor beneath the guarded maze sections.
    if map == "ViridianGym" && source.tileset_id.as_ref() == "train_station"
        && source.tile_index == crate::viridian_gym::FLOOR_TILE {
        return None;
    }
    // Exact elevator floors already keep their live yellow/orange motif on a
    // closed slab. The general store ceramic pass must not repaint that lid.
    if matches!(map, "GoldenrodDeptStoreElevator" | "CeladonDeptStoreElevator")
        && source.tileset_id.as_ref() == "mart" && source.tile_index == 0x01 {
        return None;
    }
    let style = match (source.tileset_id.as_ref(), source.tile_index) {
        _ if traditional_room::tatami_source(map, source) => Tatami,
        _ if gym_floor_source(map, source).is_some() => gym_floor_source(map, source)?,
        _ if ship_floor_source(map, source).is_some() => Ship(ship_floor_source(map, source)?),
        _ if enhanced_room_floor_source(map, source).is_some() => enhanced_room_floor_source(map, source)?,
        ("players_room" | "players_house" | "house", 0x01) => Oak,
        ("traditional_house", 0x50 | 0x44 | 0x45 | 0x54 | 0x55) => Tatami,
        ("traditional_house", 0x01) => Oak,
        ("lab", 0x10 | 0x24) => Oak,
        ("mart", 0x01) if store_floor_map(map) => StoreCeramic,
        ("mart", 0x01 | 0x48) | ("pokecenter", 0x11) => Ceramic,
        ("mansion", 0x01 | 0x11) => Marble,
        ("gate", 0x01)
        | ("facility", 0x01 | 0x26)
        | ("radio_tower", 0x01)
        | ("train_station", 0x3d)
        | ("battle_tower_inside", 0x11)
        | ("pokecom_center", 0x3d | 0x11) => Stone,
        ("game_corner", 0x01) => {
            if crate::cafe::is_cafe_map(map) {
                Oak
            } else {
                Stone
            }
        }
        ("lighthouse", _) if lighthouse_floor_source(map, source) => LighthouseSlate,
        ("lighthouse", _) if lighthouse_chamber_floor_source(map, source) => LighthouseChecker,
        _ => return None,
    };
    matches!(
        shape_for_source_on_map(map, source),
        CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
    )
    .then_some(style)
}
fn material_quad(mesh: &mut SurfaceMeshData, bounds: [f32; 4], height: f32, color: [f32; 3]) {
    let [x0, x1, z0, z1] = bounds;
    if x1 <= x0 || z1 <= z0 {
        return;
    }
    append_quad(
        mesh,
        [
            [x0, height, z0],
            [x0, height, z1],
            [x1, height, z1],
            [x1, height, z0],
        ],
        [0.0, 1.0, 0.0],
        [[0.0, 0.0]; 4],
        [color[0], color[1], color[2], 1.0],
    );
}
fn tint(color: [f32; 3], amount: f32) -> [f32; 3] {
    color.map(|v| (v + amount).clamp(0.0, 1.0))
}
// Explicit indoor paint coordinates. Only verified store/arcade tile and tower wood
// emit this reserved range; atlas art and other scenery keep their own UVs.
fn room_material_quad(
    mesh: &mut SurfaceMeshData, bounds: [f32; 4], height: f32,
    color: [f32; 3], cell: [f32; 4], world: [i32; 2], timber: bool,
) {
    let start = mesh.positions.len();
    material_quad(mesh, bounds, height, color);
    let width = cell[1] - cell[0];
    let depth = cell[3] - cell[2];
    for i in start..mesh.positions.len() {
        let p = mesh.positions[i];
        mesh.uvs[i] = [
            80000.0 + world[0] as f32 * 8.0 + (p[0] - cell[0]) / width * 8.0,
            if timber { 2000.0 } else { 1000.0 }
                + world[1] as f32 * 8.0 + (p[2] - cell[2]) / depth * 8.0,
        ];
    }
}
fn floor_cell(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    style: InteriorFloor,
    world: [i32; 2],
) {
    assert_ne!(style,InteriorFloor::NativeRoom,"Kanto room floors require authenticated source UVs");
    assert_ne!(style, InteriorFloor::MahoganyIce, "ice requires its validated source UVs");
    assert_ne!(style, InteriorFloor::MahoganyGrip, "stop patches require their validated source UVs");
    assert!(!matches!(style,InteriorFloor::OlivineTile|InteriorFloor::OlivineRunner),
        "Olivine floors require their validated source UVs");
    assert_ne!(style,InteriorFloor::ArcadeChecker,"arcade floors require native source UVs");
    assert_ne!(style,InteriorFloor::TowerDepth,"tower grating requires native source UVs");
    let [x0, x1, z0, z1] = b;
    let w = x1 - x0;
    let d = z1 - z0;
    let [column, row] = world;
    let palette = style.palette();
    if let InteriorFloor::Ship(style) = style {
        ship_floor_cell(mesh, b, height, style, world);
    } else if style == InteriorFloor::StoreCeramic {
        // Interlocking glazed pavers keep the native yellow/amber angular
        // motif. Every colored region partitions the same physical floor lid.
        let amber = [1., 0.23074, 0.002428];
        let cream = [0.930111, 0.799103, 0.672443];
        for (rect, color) in [
            ([0., 0.375, 0., 0.375], palette),
            ([0.375, 0.5, 0., 0.375], amber),
            ([0., 0.5, 0.375, 0.5], amber),
            ([0.5, 1., 0., 0.5], palette),
            ([0., 0.5, 0.5, 1.], palette),
            ([0.5, 1., 0.5, 0.625], amber),
            ([0.5, 0.625, 0.625, 1.], amber),
            ([0.625, 1., 0.625, 0.75], cream),
            ([0.625, 0.75, 0.75, 1.], cream),
            ([0.75, 1., 0.75, 1.], palette),
        ] {
            room_material_quad(mesh,
                [x0 + w * rect[0], x0 + w * rect[1], z0 + d * rect[2], z0 + d * rect[3]],
                height, color, b, world, false);
        }
    } else if style == InteriorFloor::BlackthornTile {
        // Source-blue staggered brick courses, partitioned on one floor plane.
        // Mortar and brick never overlap; the sliding/walking datum is retained.
        let seam = w.min(d) * 0.04;
        let mortar = [0.930111, 0.799103, 0.672443];
        for course in 0..2 {
            let north = z0 + d * course as f32 * 0.5;
            let south = north + d * 0.5;
            let joint = if (row * 2 + course) % 2 == 0 { x0 } else { x0 + w * 0.5 };
            material_quad(mesh, [x0,x1,north,north+seam], height, mortar);
            material_quad(mesh, [x0,joint,north+seam,south], height, palette);
            material_quad(mesh, [joint,joint+seam,north+seam,south], height, mortar);
            material_quad(mesh, [joint+seam,x1,north+seam,south], height, palette);
        }
    } else if style == InteriorFloor::VioletBrick {
        // Blue brick belongs to Violet's exact floor drawing. Two staggered
        // courses retain its pale mortar while remaining at the native datum.
        let seam = w.min(d) * 0.035;
        for course in 0..2 {
            let north = z0 + d * course as f32 * 0.5;
            let south = north + d * 0.5;
            let mortar = [0.75,0.78,0.84];
            let joint = if (row * 2 + course) % 2 == 0 {x0} else {x0 + w * 0.5};
            // Partition the original floor instead of layering coincident
            // mortar and brick triangles, which flicker in low battle shots.
            material_quad(mesh,[x0,x1,north,north+seam],height,mortar);
            material_quad(mesh,[x0,x1,south-seam,south],height,mortar);
            if joint > x0 {
                material_quad(mesh,[x0,joint,north+seam,south-seam],height,palette);
            }
            material_quad(mesh,[joint,joint+seam,north+seam,south-seam],height,mortar);
            material_quad(mesh,[joint+seam,x1,north+seam,south-seam],height,palette);
        }
    } else if style==InteriorFloor::PoolCeramic {
        // Keep the small gray squares and warm grout of the pool-side source.
        let seam = w.min(d) * 0.06;
        material_quad(mesh, [x0, x1, z0, z0 + seam], height, [0.91, 0.87, 0.79]);
        material_quad(mesh, [x0, x0 + seam, z0 + seam, z1], height, [0.91, 0.87, 0.79]);
        material_quad(mesh, [x1 - seam, x1, z0 + seam, z1], height, [0.36, 0.37, 0.38]);
        material_quad(mesh, [x0 + seam, x1 - seam, z1 - seam, z1], height, [0.36, 0.37, 0.38]);
        material_quad(mesh, [x0 + seam, x1 - seam, z0 + seam, z1 - seam], height, palette);
    } else if gym_floor_style(style) {
        unreachable!("native Gym floors require their verified source UVs");
    } else if style == InteriorFloor::LighthouseChecker {
        lighthouse_chamber_floor_cell(mesh, b, height, world, [false; 4]);
    } else if style == InteriorFloor::LighthouseSlate {
        lighthouse_floor_cell(mesh, b, height, world);
    } else if matches!(style, InteriorFloor::Oak | InteriorFloor::AmberTimber) {
        // Wide boards with staggered end joints. The low-contrast palette and
        // sparse joints avoid the old high-frequency striped source texture.
        for half in 0..2 {
            let plank = row * 2 + half;
            let za = z0 + d * half as f32 * 0.5;
            let zb = za + d * 0.5;
            let offset = (plank.rem_euclid(2)) * 2;
            let board = (column - offset).div_euclid(4);
            let variation = ((board * 13 + plank * 7).rem_euclid(7) as f32 - 3.0) * 0.007;
            let seam = d * 0.018;
            let joint = if (column - offset).rem_euclid(4) == 0 {
                w * 0.018
            } else {
                0.0
            };
            let color = tint(palette, variation);
            let dark = tint(palette, -0.10);
            material_quad(mesh, [x0, x1, za, za + seam], height, dark);
            if joint > 0.0 {
                material_quad(mesh, [x0, x0 + joint, za + seam, zb], height, dark);
            }
            if style==InteriorFloor::AmberTimber {
                room_material_quad(mesh,[x0+joint,x1,za+seam,zb],height,color,b,world,true);
            } else {
            material_quad(mesh, [x0 + joint, x1, za + seam, zb], height, color);
            }
        }
    } else if style == InteriorFloor::Tatami {
        let mat_x = column.div_euclid(2);
        let mat_z = row.div_euclid(4);
        let variation = ((mat_x + mat_z).rem_euclid(2) as f32 - 0.5) * 0.035;
        let color = tint(palette, variation);
        let hem = [0.29, 0.35, 0.25];
        let west = if column.rem_euclid(2) == 0 {
            w * 0.06
        } else {
            0.0
        };
        let north = if row.rem_euclid(4) == 0 {
            d * 0.06
        } else {
            0.0
        };
        if north > 0.0 {
            material_quad(mesh, [x0, x1, z0, z0 + north], height, hem);
        }
        if west > 0.0 {
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, hem);
        }
        for line in 0..4 {
            let a = z0 + north + (d - north) * line as f32 / 4.0;
            let b = z0 + north + (d - north) * (line + 1) as f32 / 4.0;
            material_quad(
                mesh,
                [x0 + west, x1, a, b],
                height,
                tint(color, if line % 2 == 0 { 0.005 } else { -0.005 }),
            );
        }
    } else {
        let variation =
            ((column.div_euclid(2) * 3 + row.div_euclid(2) * 5).rem_euclid(5) as f32 - 2.0) * 0.008;
        let color = tint(palette, variation);
        let grout = tint(palette, -0.085);
        let west = if column.rem_euclid(2) == 0 {
            w * 0.025
        } else {
            0.0
        };
        let north = if row.rem_euclid(2) == 0 {
            d * 0.025
        } else {
            0.0
        };
        if north > 0.0 {
            material_quad(mesh, [x0, x1, z0, z0 + north], height, grout);
        }
        if west > 0.0 {
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, grout);
        }
        material_quad(mesh, [x0 + west, x1, z0 + north, z1], height, color);
    }
}
// Uses precisely the same grid phase and palette as the established room floor.
// Physical ice slabs retain the complete live source drawing and palette.
// A shallow bevel follows each 16-pixel slab perimeter; the inner 8-pixel
// quadrant boundaries remain seamless. The authoritative sliding datum stays
// at height, with all decorative geometry below it.
pub(super) fn native_beveled_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32;4], height: f32,
    world: [i32;2], uv: (f32,f32,f32,f32),
) {
    let [w,e,n,s] = b;
    let (u0,u1,v0,v1) = uv;
    let west = world[0].rem_euclid(2) == 0;
    let north = world[1].rem_euclid(2) == 0;
    let xs = if west {[0.,0.06,1.]} else {[0.,0.94,1.]};
    let zs = if north {[0.,0.06,1.]} else {[0.,0.94,1.]};
    let depth = (e-w).min(s-n) * 0.006;
    let point = |x: f32,z: f32| {
        let edge = x == if west {0.} else {1.} || z == if north {0.} else {1.};
        [w+(e-w)*x, height-if edge {depth} else {0.}, n+(s-n)*z]
    };
    let tex = |x: f32,z: f32| [u0+(u1-u0)*x,v0+(v1-v0)*z];
    native_source_surface_patches(mesh,&xs,&zs,point,tex,|_,_|1.,false);
}

// Share the surface topology between original source slabs and room material
// finishes. Source colors stay in the atlas; detail is a neutral facet tint.
pub(super) fn native_source_surface_patches(mesh:&mut SurfaceMeshData,xs:&[f32],zs:&[f32],
    point:impl Fn(f32,f32)->[f32;3],tex:impl Fn(f32,f32)->[f32;2],
    shade:impl Fn(f32,f32)->f32,swatch:bool) {
    for z in 0..zs.len()-1 {for x in 0..xs.len()-1 {
        let corners=[[xs[x],zs[z]],[xs[x],zs[z+1]],
            [xs[x+1],zs[z+1]],[xs[x+1],zs[z]]];
        let centre=[(xs[x]+xs[x+1])*0.5,(zs[z]+zs[z+1])*0.5];
        for edge in 0..4 {
            let coords=[corners[edge],corners[(edge+1)%4],centre];
            let points=coords.map(|[x,z]|point(x,z));
            let normal=(Vec3::from_array(points[1])-Vec3::from_array(points[0]))
                .cross(Vec3::from_array(points[2])-Vec3::from_array(points[0])).normalize();
            let first=mesh.positions.len()as u32;
            mesh.positions.extend(points);mesh.normals.extend([normal.to_array();3]);
            // A material swatch stays constant over each face, avoiding
            // interpolated UVs that cross unrelated source pixels.
            if swatch {let sample=tex(centre[0],centre[1]);mesh.uvs.extend([sample;3]);}
            else {mesh.uvs.extend(coords.map(|[x,z]|tex(x,z)));}
            for [x,z] in coords {
                let tint=shade(x,z);mesh.colors.push(crate::interior_models::authored_face_color(
                    [tint,tint,tint,1.],normal));
            }
            mesh.indices.extend([first,first+1,first+2]);
        }
    }}
}

fn native_cafe_material_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    world: [i32; 2], uv: (f32, f32, f32, f32), source_tile: u16,
) {
    let (u0, u1, v0, v1) = uv;
    // Each authenticated quadrant has the same cream field, but its dark
    // outline occupies a different corner. Sample those live source pixels;
    // the original bitmap remains available to the classic renderer.
    let grout_pixel = match source_tile {
        0x01 => [1, 1],
        0x10 => [6, 1],
        0x11 => [1, 6],
        0x12 => [6, 6],
        _ => unreachable!("cafe finish requires an authenticated floor quadrant"),
    };
    let sample = |pixel: [usize; 2]| [
        u0 + (u1 - u0) * (pixel[0] as f32 + 0.5) / 8.,
        v0 + (v1 - v0) * (pixel[1] as f32 + 0.5) / 8.,
    ];
    native_source_ceramic_floor_cell(mesh, b, height, world, sample([0, 0]), sample(grout_pixel));
}

fn native_station_material_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,
    world:[i32;2],uv:(f32,f32,f32,f32)) {
    let (u0,u1,v0,v1)=uv;
    let sample=|x:f32,y:f32|[u0+(u1-u0)*(x+0.5)/8.,v0+(v1-v0)*(y+0.5)/8.];
    // Tile3d's broad blue field and darker border become ceramic and grout.
    // Retain live palette ownership rather than magnifying its tiny glyph.
    native_source_ceramic_floor_cell(mesh,b,height,world,sample(2.,2.),sample(0.,0.));
}

fn native_gate_material_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    _world: [i32; 2], uv: (f32, f32, f32, f32),
) {
    let (u0, u1, v0, v1) = uv;
    let sample = |pixel: [usize; 2]| [
        u0 + (u1 - u0) * (pixel[0] as f32 + 0.5) / 8.,
        v0 + (v1 - v0) * (pixel[1] as f32 + 0.5) / 8.,
    ];
    // One source8px cell contains a complete diamond, not half of a large
    // square checker. Keep that rhythm and both authenticated pigments.
    let gray=sample([0,3]);let cream=sample([0,0]);
    let [w,e,n,s]=b;
    let centre=Vec3::new((w+e)*0.5,height,(n+s)*0.5);
    let corners=[[w,height,n],[e,height,n],[e,height,s],[w,height,s]].map(Vec3::from_array);
    for i in 0..4 {
        let paint=if i%2==0 {cream}else{gray};
        let start=mesh.positions.len()as u32;
        // Source cream triangles meet from north/south; gray meets east/west.
        // Adjacent cells join into diamonds centered on their shared edges.
        mesh.positions.extend([centre,corners[(i+1)%4],corners[i]].map(|p|p.to_array()));
        mesh.normals.extend([[0.,1.,0.];3]);mesh.uvs.extend([paint;3]);
        mesh.colors.extend([TEXTURED_SHADE;3]);mesh.indices.extend([start,start+1,start+2]);
    }
}

fn native_source_ceramic_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    world: [i32; 2], field_uv: [f32; 2], grout_uv: [f32; 2],
) {
    let [w, e, n, s] = b;
    let west = world[0].rem_euclid(2) == 0;
    let north = world[1].rem_euclid(2) == 0;
    let border = 0.025;
    let xs = if west { [0., border, 1.] } else { [0., 1. - border, 1.] };
    let zs = if north { [0., border, 1.] } else { [0., 1. - border, 1.] };
    let point = |x: f32, z: f32| {
        let edge = x == if west { 0. } else { 1. }
            || z == if north { 0. } else { 1. };
        [w + (e - w) * x,
            height - if edge { (e - w).min(s - n) * 0.006 } else { 0. },
            n + (s - n) * z]
    };
    let tex = |x: f32, z: f32| {
        let grout = (if west { x < border } else { x > 1. - border })
            || (if north { z < border } else { z > 1. - border });
        if grout { grout_uv } else { field_uv }
    };
    native_source_surface_patches(mesh, &xs, &zs, point, tex, |_, _| 1., true);
}

fn native_gift_shop_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,world:[i32;2],uv:(f32,f32,f32,f32),tile:u16) {
    if tile==1 {native_house_material_floor_cell(mesh,b,height,world,uv,true);return;}
    let [w,e,n,s]=b;let (u0,u1,v0,v1)=uv;
    let sample=|x:f32,y:f32|[u0+(u1-u0)*(x+0.5)/8.,v0+(v1-v0)*(y+0.5)/8.];
    let field=sample(1.,0.);let light=sample(1.,1.);let edge=sample(0.,0.);
    let xs=[0.,0.04,0.24,0.26,0.49,0.51,0.74,0.76,0.96,1.];let zs=[0.,0.04,0.96,1.];
    let border=|x:f32,z:f32|(world[0].rem_euclid(4)==0&&x<0.04)||(world[0].rem_euclid(4)==3&&x>0.96)
        ||(world[1].rem_euclid(4)==0&&z<0.04)||(world[1].rem_euclid(4)==3&&z>0.96);
    let point=|x:f32,z:f32|[w+(e-w)*x,height-if border(x,z){0.02}else{0.},n+(s-n)*z];
    let tex=|x:f32,z:f32|if border(x,z){edge}else if (x-0.25).abs()<0.01||(x-0.5).abs()<0.01||(x-0.75).abs()<0.01{light}else{field};
    native_source_surface_patches(mesh,&xs,&zs,point,tex,|_,_|1.,true);
}

pub(super) fn underground_native_floor(map:&str,s:&VisualTileSource)->bool {
    if !matches!(map,"UndergroundPath"|"VermilionPortPassage")||s.tileset_id.as_ref()!="underground"||s.tile_index!=16
        ||s.subtile_column>=4||s.subtile_row>=4 {return false;}
    let mask:u16=match s.metatile_id {3 if map=="VermilionPortPassage"=>0xff33,
        37|38 if map=="VermilionPortPassage"=>0xff00,41 if map=="VermilionPortPassage"=>0xf333,
        42 if map=="VermilionPortPassage"=>0xfccc,2=>0xff33,8=>0xcc00,9=>0xff00,10=>0x3300,
        12=>0xcccc,13=>0xffff,14=>0x3333,16=>0x00cc,17=>0x00ff,18=>0x0033,_=>0};
    mask&(1<<(s.subtile_row*4+s.subtile_column))!=0
        &&matches!(shape_for_source_on_map(map,s),CellShape::Flat|CellShape::PlaneAt{height:0.})
}

pub(super) fn native_underground_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,uv:(f32,f32,f32,f32)) {
    let [w,e,n,s]=b;let center=[(w+e)*0.5,height,(n+s)*0.5];
    let corners=[[w,height,n],[e,height,n],[e,height,s],[w,height,s]];
    let (u0,u1,v0,v1)=uv;
    let swatches=[[u0+(u1-u0)*0.5/8.,v0+(v1-v0)*0.5/8.],
        [u0+(u1-u0)*7.5/8.,v0+(v1-v0)*0.5/8.]];
    // Tile10 contains opposed gray triangles meeting at the center, not
    // stepped diamond boundaries. Preserve its orientation across every cell.
    for j in 0..4 {let first=mesh.positions.len()as u32;
        mesh.positions.extend_from_slice(&[center,corners[(j+1)%4],corners[j]]);
        mesh.normals.extend_from_slice(&[[0.,1.,0.];3]);
        mesh.uvs.extend_from_slice(&[swatches[j%2];3]);mesh.colors.extend_from_slice(&[TEXTURED_SHADE;3]);
        mesh.indices.extend_from_slice(&[first,first+1,first+2]);
    }
}

fn native_safari_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,world:[i32;2],white:[f32;2],joint:[f32;2],gray:[f32;2]) {
    let [w,e,n,s]=b;let xphase=world[0].rem_euclid(2)as f32;let zphase=world[1].rem_euclid(2)as f32;
    let cuts=|phase:f32|if phase==0. {[0.,0.125,0.25,0.375,1.]}else{[0.,0.625,0.75,0.875,1.]};
    let xs=cuts(xphase);let zs=cuts(zphase);
    let tex=|x:f32,z:f32| {
        let x=(xphase+x)/2.;let z=(zphase+z)/2.;
        let inside=x>0.0625&&x<0.9375&&z>0.0625&&z<0.9375;
        if inside&&(x<0.125||x>0.875||z<0.125||z>0.875) {joint}
        else if x>=0.5&&x<0.8125&&z>=0.1875&&z<0.5
            ||x>=0.1875&&x<0.5&&z>=0.5&&z<0.8125 {gray}else{white}
    };
    native_source_surface_patches(mesh,&xs,&zs,|x,z|[w+(e-w)*x,height,n+(s-n)*z],tex,|_,_|1.,true);
}

fn native_center_material_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,
    world:[i32;2],uv:(f32,f32,f32,f32)) {
    let (u0,u1,v0,v1)=uv;
    let sample=|[x,y]:[usize;2]|[u0+(u1-u0)*(x as f32+0.5)/8.,v0+(v1-v0)*(y as f32+0.5)/8.];
    // Tile11's pink field/light joint and tile01's darker service apron keep
    // their separate live palettes. Replace magnified pixel diamonds with
    // the same physical ceramic seams used by other authored floors.
    native_source_ceramic_floor_cell(mesh,b,height,world,sample([2,0]),sample([0,0]));
}
fn native_mart_material_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    world: [i32; 2], uv: (f32, f32, f32, f32),
) {
    let (u0, u1, v0, v1) = uv;
    let sample = |pixel: [usize; 2]| [
        u0 + (u1 - u0) * (pixel[0] as f32 + 0.5) / 8.,
        v0 + (v1 - v0) * (pixel[1] as f32 + 0.5) / 8.,
    ];
    // Keep the standard mart floor's live greens instead of its pixel lattice.
    let dark = sample([0, 0]);
    let light = sample([3, 3]);
    // The source has one continuous green field with small light details;
    // alternating entire tiles invents a large checkerboard absent from it.
    native_source_ceramic_floor_cell(mesh, b, height, world, dark, light);
}

fn native_silph_material_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    world: [i32; 2], uv: (f32, f32, f32, f32), source_tile: u16,
) {
    let (u0, u1, v0, v1) = uv;
    let sample = |x: f32| [u0 + (u1 - u0) * (x + 0.5) / 8.,
        v0 + (v1 - v0) * 0.5 / 8.];
    // Facility01 alternates palette indices 1/0 at each pixel; facility26
    // is the plain index-zero field. Keep the source blue/cream arrangement
    // without carrying that one-pixel dither into the angled walking surface.
    let field = sample(0.);
    let joint = sample(if source_tile == 0x01 { 1. } else { 0. });
    native_source_ceramic_floor_cell(mesh, b, height, world, field, joint);
}

fn native_deptstore_material_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    world: [i32; 2], uv: (f32, f32, f32, f32),
) {
    let [w, e, n, s] = b;
    let (u0, u1, v0, v1) = uv;
    let sample = |pixel: [usize; 2]| [
        u0 + (u1 - u0) * (pixel[0] as f32 + 0.5) / 8.,
        v0 + (v1 - v0) * (pixel[1] as f32 + 0.5) / 8.,
    ];
    let yellow = sample([0, 0]);
    let amber = sample([3, 0]);
    let cream = sample([5, 5]);
    // Preserve the angular three-color drawing at a sixteen-pixel slab scale.
    // Clip its partitions into each eight-pixel cell rather than painting over
    // neighboring furnishings or changing the walking datum.
    let ox = world[0].rem_euclid(2) as f32 * 0.5;
    let oz = world[1].rem_euclid(2) as f32 * 0.5;
    for (r, color) in [
        ([0., 0.375, 0., 0.375], yellow),
        ([0.375, 0.5, 0., 0.375], amber),
        ([0., 0.5, 0.375, 0.5], amber),
        ([0.5, 1., 0., 0.5], yellow),
        ([0., 0.5, 0.5, 1.], yellow),
        ([0.5, 1., 0.5, 0.625], amber),
        ([0.5, 0.625, 0.625, 1.], amber),
        ([0.625, 1., 0.625, 0.75], cream),
        ([0.625, 0.75, 0.75, 1.], cream),
        ([0.75, 1., 0.75, 1.], yellow),
    ] {
        let r: [f32; 4] = r;
        let left = r[0].max(ox);
        let right = r[1].min(ox + 0.5);
        let north = r[2].max(oz);
        let south = r[3].min(oz + 0.5);
        if left >= right || north >= south { continue; }
        append_top(mesh, [w + (e - w) * (left - ox) * 2.,
            w + (e - w) * (right - ox) * 2.,
            n + (s - n) * (north - oz) * 2.,
            n + (s - n) * (south - oz) * 2.], height,
            (color[0], color[0], color[1], color[1]));
    }
}

pub(super) fn native_house_material_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,
    world:[i32;2],uv:(f32,f32,f32,f32),wood:bool) {
    let [w,e,n,s]=b;let (u0,u1,v0,v1)=uv;
    assert!([w,e,n,s,height].iter().all(|v|v.is_finite()) && w<e && n<s,
        "native house material floor requires finite source bounds");
    let sample=|pixel:[usize;2]|[u0+(u1-u0)*(pixel[0]as f32+0.5)/8.,
        v0+(v1-v0)*(pixel[1]as f32+0.5)/8.];
    native_house_floor_fields(mesh,b,height,world,wood,[sample([0,0]),sample([1,0])]);
}
pub(super) fn native_house_floor_fields(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,
    world:[i32;2],wood:bool,fields:[[f32;2];2]) {
    let [w,e,n,s]=b;
    let west=if wood {(world[0]+2*world[1].rem_euclid(2)).rem_euclid(4)==0}
        else {world[0].rem_euclid(2)==0};
    let east=if wood {(world[0]+2*world[1].rem_euclid(2)).rem_euclid(4)==3}
        else {world[0].rem_euclid(2)==1};
    let north=wood || world[1].rem_euclid(2)==0;
    let south=wood || world[1].rem_euclid(2)==1;
    let border=if wood {0.045}else{0.025};
    let xs=if west {[0.,border,1.]}else if east {[0.,1.-border,1.]}else {[0.,0.5,1.]};
    let zs=if wood {vec![0.,0.045,0.25,0.5,0.75,0.955,1.]}
        else if north {vec![0.,0.025,1.]}else {vec![0.,0.975,1.]};
    let point=|x:f32,z:f32|[w+(e-w)*x,height-if (west && x==0.)||(east && x==1.)
        ||(north && z==0.)||(south && z==1.) {(e-w).min(s-n)*0.006}else{0.},n+(s-n)*z];
    let checker=(world[0].div_euclid(2)+world[1].div_euclid(2)).rem_euclid(2);
    // Verified residential swatches: wood uses medium gold; ceramic keeps the
    // light/pink pair from house tile01 or Mansion tile11 in its live atlas.
    let tex=|_:f32,_:f32|fields[usize::from(!wood&&checker!=0)];
    let shade=|x:f32,z:f32| {
        let column=if wood {(world[0]+2*world[1].rem_euclid(2)).div_euclid(4)}else{world[0].div_euclid(2)};
        let seed=(column*3+world[1]*5).rem_euclid(7)as f32;
        if wood {
            let grain=((world[0]as f32+x)*2.7+(world[1]as f32+z)*23.).sin();
            let body=0.93+seed*0.004+grain*0.045;
            if (west&&x<0.045)||(east&&x>0.955)||z<0.045||z>0.955 {body*0.68}else{body}
        } else {0.98+seed*0.002}
    };
    native_source_surface_patches(mesh,&xs,&zs,point,tex,shade,true);
}

fn native_rough_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,uv:(f32,f32,f32,f32)) {
    let [w,e,n,s]=b;let (u0,u1,v0,v1)=uv;
    // Small recessed facets give the stop patch a rough surface. Its source
    // speckles and gray palette remain intact, with no raised walking datum.
    let point=|x:usize,z:usize| {
        let recess=if x==0 || x==4 || z==0 || z==4 {0.}
            else {((x*7+z*11)%3)as f32*(e-w).min(s-n)*0.003};
        [w+(e-w)*x as f32/4.,height-recess,n+(s-n)*z as f32/4.]
    };
    for z in 0..4 {for x in 0..4 {
        let corners=[[x,z],[x,z+1],[x+1,z+1],[x+1,z]];
        for tri in [[0,1,2],[0,2,3]] {
            let coords=tri.map(|i|corners[i]);let points=coords.map(|[x,z]|point(x,z));
            let normal=(Vec3::from_array(points[1])-Vec3::from_array(points[0]))
                .cross(Vec3::from_array(points[2])-Vec3::from_array(points[0])).normalize();
            let first=mesh.positions.len()as u32;
            mesh.positions.extend(points);mesh.normals.extend([normal.to_array();3]);
            mesh.uvs.extend(coords.map(|[x,z]|[u0+(u1-u0)*x as f32/4.,v0+(v1-v0)*z as f32/4.]));
            mesh.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
            mesh.indices.extend([first,first+1,first+2]);
        }
    }}
}

pub(super) fn cable_club_floor_cell(mesh: &mut SurfaceMeshData, bounds: [f32; 4], world: [i32; 2]) {
    floor_cell(mesh, bounds, 0.0, InteriorFloor::Ceramic, world);
}
pub(super) fn arcade_floor_cell(mesh: &mut SurfaceMeshData, bounds: [f32; 4], phase: [i32; 2],
    uv:(f32,f32,f32,f32)) {
    native_beveled_floor_cell(mesh,bounds,0.,phase,uv);
}
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.0001
}

/// Replace only exact flat-floor atlas quads, including verified floor restored
/// beneath objects. Source pictures, unknown strips, stair treads and walls keep
/// their own art. Geometry heights and all actor footing remain unchanged.
/// Returns finished floor quads; deliberately does not mark them as objects.
pub(super) fn finish_surfaces(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
) -> usize {
    finish_surfaces_excluding(mesh, map, cells, g, world_origin, &[])
}
/// Live source profiles have first refusal over specialized material passes, including
/// profiles whose drawings consist entirely of a recognized native floor.
pub(super) fn finish_surfaces_with_profiles(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
    profiles: Option<&Document>,
) -> usize {
    if (!gym_floor_map(map) && !traditional_room::floor_map(map) && !ship_floor_map(map)
        && !enhanced_room_floor_map(map) && !store_floor_map(map) && !kanto_native_floor_map(map))
        || g.width.checked_mul(g.height) != Some(cells.len())
    {
        return finish_surfaces(mesh, map, cells, g, world_origin);
    }
    let mut excluded = vec![false; cells.len()];
    // Canonical fixtures have already been replaced by the authored models;
    // their verified native backing should receive the room finish too.
    // Only user-owned overrides exclude backing in the enhanced rooms.
    let custom = (enhanced_room_floor_map(map) || store_floor_map(map) || kanto_native_floor_map(map)
        || traditional_room::floor_map(map)).then(|| Document {
        objects: profiles.map_or_else(Vec::new, |d| d.objects.iter()
            .filter(|o| !built_in_profiles().objects.iter().any(|canonical| canonical == *o))
            .cloned().collect()),
        atmosphere: None,
    });
    let selected = custom.as_ref().or(profiles);
    for placement in live::resolve(cells, g.width, g.height, map, selected) {
        for index in placement.indices(g.width) {
            excluded[index] = true;
        }
    }
    finish_surfaces_excluding(mesh, map, cells, g, world_origin, &excluded)
}

fn tower_grating_cell(mesh: &mut SurfaceMeshData, bounds: [f32;4], height: f32,
    tile: u16, uv: (f32,f32,f32,f32)) {
    assert!(matches!(tile,0x0a|0x0b|0x1a|0x1b), "verified tower grating drawing");
    let [w,e,n,s]=bounds;let px=(e-w)/8.;let pz=(s-n)/8.;
    let scale=px.min(pz);
    let point = |x:f32,y:f32| [uv.0+(uv.1-uv.0)*(x+0.5)/8.,
        uv.2+(uv.3-uv.2)*(y+0.5)/8.];
    let first=mesh.positions.len();
    append_top(mesh,bounds,height-2.*scale,uv);
    mesh.uvs[first..].fill(point(0.,2.));
    let mut rail = |x0:f32,x1:f32,z0:f32,z1:f32, sample:[f32;2]| {
        let first=mesh.positions.len();
        architecture_box(mesh,[w+x0*px,w+x1*px,height-0.6*scale,height,
            n+z0*pz,n+z1*pz],[1.;3]);
        mesh.uvs[first..].fill(point(sample[0],sample[1]));
    };
    // Two native gold courses over the dark opening. Corner drawings retain
    // their terminated rails; repeated cells meet exactly across source seams.
    rail(if tile==0x1a {4.} else {0.},8.,1.,2.,[4.,1.]);
    rail(if matches!(tile,0x0a|0x1a) {2.} else {0.},8.,5.,7.,[4.,5.]);
    if matches!(tile,0x0b|0x1b) {rail(1.,2.,0.,8.,[1.,3.]);}
}

fn finish_surfaces_excluding(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
    excluded: &[bool],
) -> usize {
    if g.width.checked_mul(g.height) != Some(cells.len())
        || cells.is_empty()
        || (!excluded.is_empty() && excluded.len() != cells.len())
    {
        return 0;
    }
    let gym_ground = gym_floor_underlays(mesh, map, cells, g, world_origin, excluded);
    let ship_ground = ship_floor_underlays(mesh, map, cells, g, excluded);
    let tatami_floor = traditional_room::tatami_floor_mask(map, cells, g, world_origin, excluded);
    let source_styles: Vec<_> = cells
        .iter()
        .map(|tile| interior_floor(map, &tile.source))
        .collect();
    let floor_trace=map=="UndergroundPath"&&std::env::var_os("CRYSTAL_INTERIOR_FLOOR_TRACE").is_some();
    if floor_trace {
        let sample=cells.iter().find(|t|t.source.tileset_id.as_ref()=="underground"&&t.source.tile_index==16);
        eprintln!("interior-floor map={map} grid={}x{} source16={} native={} sample={:?}",g.width,g.height,
            cells.iter().filter(|t|t.source.tileset_id.as_ref()=="underground"&&t.source.tile_index==16).count(),
            source_styles.iter().filter(|&&s|s==Some(InteriorFloor::NativeRoom)).count(),sample.map(|t|&t.source));
    }
    if source_styles.iter().all(Option::is_none) {
        return 0;
    }
    let original = std::mem::take(&mut mesh.textured.indices);
    if floor_trace {
        let mut counts=[0usize;5];let mut sample=None;
        for q in original.chunks(6) {
            if q.len()!=6||q.iter().any(|&i|i as usize>=mesh.textured.positions.len()) {continue;}
            let ps=q.iter().map(|&i|mesh.textured.positions[i as usize]).collect::<Vec<_>>();
            let x=ps.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min);
            let z=ps.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min);
            let col=((x-g.origin_x)/g.tile_width).round()as i32;let row=((z-g.origin_z)/g.tile_height).round()as i32;
            if col<0||row<0||col>=g.width as i32||row>=g.height as i32
                ||source_styles[row as usize*g.width+col as usize]!=Some(InteriorFloor::NativeRoom) {continue;}
            let width=ps.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max)-x;
            let depth=ps.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max)-z;
            let stage=if q[0]!=q[3]||q[2]!=q[4] {0}
                else if q.iter().any(|&i|mesh.textured.normals[i as usize]!=[0.,1.,0.]) {1}
                else if !near(width,g.tile_width)||!near(depth,g.tile_height) {2}
                else {3};
            counts[stage]+=1;counts[4]+=1;
            if stage<3&&sample.is_none(){sample=Some((col,row,width,depth,stage));}
        }
        eprintln!("interior-floor input-groups irregular/normal/size/cell/total={counts:?} first={sample:?}");
    }
    let mut finished = 0;
    for quad in original.chunks(6) {
        let candidate = (|| {
            if quad.len() != 6 || quad[0] != quad[3] || quad[2] != quad[4] {
                return None;
            }
            let ids = [
                quad[0] as usize,
                quad[1] as usize,
                quad[2] as usize,
                quad[5] as usize,
            ];
            if ids.iter().any(|&i| {
                i >= mesh.textured.positions.len() || mesh.textured.normals[i] != [0.0, 1.0, 0.0]
            }) {
                return None;
            }
            let points = ids.map(|i| mesh.textured.positions[i]);
            let uvs = ids.map(|i| mesh.textured.uvs[i]);
            let height = points[0][1];
            if points.iter().any(|p| !near(p[1], height)) {
                return None;
            }
            let x0 = points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
            let x1 = points
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max);
            let z0 = points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
            let z1 = points
                .iter()
                .map(|p| p[2])
                .fold(f32::NEG_INFINITY, f32::max);
            if !near(x1 - x0, g.tile_width) || !near(z1 - z0, g.tile_height) {
                return None;
            }
            let column = ((x0 - g.origin_x) / g.tile_width).round() as i32;
            let row = ((z0 - g.origin_z) / g.tile_height).round() as i32;
            if column < 0 || row < 0 || column >= g.width as i32 || row >= g.height as i32 {
                return None;
            }
            if !near(x0, g.origin_x + column as f32 * g.tile_width)
                || !near(z0, g.origin_z + row as f32 * g.tile_height)
            {
                return None;
            }
            // Store floor samples can be redirected by backing/merged fixtures.
            // Resolve the actual destination floor first so the ceramic grid
            // covers every eligible native floor cell with one world phase.
            let dest=row as usize*g.width+column as usize;
            if store_floor_map(map) && near(height,0.0) && excluded.get(dest)!=Some(&true)
                && interior_floor(map,&cells[dest].source)==Some(InteriorFloor::StoreCeramic) {
                return Some((dest,[x0,x1,z0,z1],height,InteriorFloor::StoreCeramic,
                    [column+world_origin[0],row+world_origin[1]]));
            }
            let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
            let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
            let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
            let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
            let sc = ((min_u + max_u) * 0.5 * g.width as f32).floor() as usize;
            let sr = ((min_v + max_v) * 0.5 * g.height as f32).floor() as usize;
            if sc >= g.width || sr >= g.height {
                return None;
            }
            let (u0, u1, v0, v1) = g.uv(sc, sr);
            if !near(min_u, u0) || !near(max_u, u1) || !near(min_v, v0) || !near(max_v, v1) {
                return None;
            }
            // Bounds alone also accept a clipped/distorted sample whose other
            // corners still touch the original limits. Require all four actual
            // atlas corners before replacing source artwork with a finish.
            if [[u0, v0], [u0, v1], [u1, v0], [u1, v1]]
                .iter()
                .any(|corner| {
                    !uvs.iter()
                        .any(|uv| near(uv[0], corner[0]) && near(uv[1], corner[1]))
                })
            {
                return None;
            }
            let destination = row as usize * g.width + column as usize;
            let source_index = sr * g.width + sc;
            if (enhanced_room_floor_map(map) || store_floor_map(map) || kanto_native_floor_map(map)
                || traditional_room::floor_map(map))
                && (excluded.get(destination) == Some(&true)
                    || excluded.get(source_index) == Some(&true))
            {
                return None;
            }
            if traditional_room::floor_map(map)
                && (traditional_room::tatami_source(map,&cells[destination].source)
                    || traditional_room::tatami_source(map,&cells[source_index].source))
                && (destination != source_index || !tatami_floor[destination] || !near(height, 0.0))
            {
                return None;
            }
            let style = source_styles[sr * g.width + sc].or_else(|| {
                // Old grouped fixtures used a wallpaper sample for masking.
                // It is floor backing only under an actually appended model,
                // never permission to paint over an unknown wall/prop cell.
                let source = &cells[sr * g.width + sc].source;
                let modeled = mesh
                    .authored_cells
                    .get(destination)
                    .is_some_and(Option::is_some);
                if !modeled {
                    return None;
                }
                match (source.tileset_id.as_ref(), source.tile_index) {
                    ("players_house",0x11) | ("players_room",0x02) if reviewed_native_room(map)
                        && source_styles.iter().any(|style|*style==Some(InteriorFloor::NativeRoom))
                        => Some(InteriorFloor::NativeRoom),
                    ("players_room", 0x02) | ("players_house", 0x11) => Some(InteriorFloor::Oak),
                    ("traditional_house",0x11) if map=="MountMoonGiftShop"=>Some(InteriorFloor::NativeRoom),
                    ("traditional_house", 0x11) => Some(InteriorFloor::Tatami),
                    _ => None,
                }
            })?;
            if style==InteriorFloor::FuchsiaNative
                && (source_index!=destination || !near(height,0.)
                    || !crate::fuchsia_gym::floor_source(&cells[destination].source)) {
                return None;
            }
            if matches!(style,InteriorFloor::MahoganyGrip|InteriorFloor::OlivineTile
                |InteriorFloor::OlivineRunner|InteriorFloor::ArcadeChecker|InteriorFloor::TowerDepth)
                && (source_index!=destination || !near(height,0.)
                    || enhanced_room_floor_source(map,&cells[destination].source)!=Some(style)) {
                return None;
            }
            if gym_floor_style(style)
                && !gym_floor_destination(
                    mesh,
                    map,
                    cells,
                    destination,
                    sr * g.width + sc,
                    style,
                    &gym_ground,
                    excluded,
                )
            {
                return None;
            }
            if let InteriorFloor::Ship(ship_style) = style {
                if !ship_floor_destination(
                    mesh,
                    map,
                    cells,
                    destination,
                    source_index,
                    ship_style,
                    &ship_ground,
                    excluded,
                ) {
                    return None;
                }
            }
            if style == InteriorFloor::LighthouseSlate
                && !lighthouse_floor_destination(mesh, map, cells, destination)
            {
                return None;
            }
            if style == InteriorFloor::LighthouseChecker
                && !lighthouse_chamber_floor_destination(mesh, map, cells, destination)
            {
                return None;
            }
            Some((
                destination,
                [x0, x1, z0, z1],
                height,
                style,
                [column + world_origin[0], row + world_origin[1]],
            ))
        })();
        if let Some((destination, bounds, height, style, world)) = candidate {
            if style==InteriorFloor::NativeRoom {
                let ids=[quad[0],quad[1],quad[2],quad[5]];
                let u=ids.map(|i|mesh.textured.uvs[i as usize]);
                let source_column=(((u[0][0]+u[2][0])*0.5*g.width as f32).floor() as usize).min(g.width-1);
                let source_row=(((u[0][1]+u[2][1])*0.5*g.height as f32).floor() as usize).min(g.height-1);
                let source_index=source_row*g.width+source_column;
                let mut donor_index = source_index;
                let native_uv=if source_styles[source_index]==Some(InteriorFloor::NativeRoom) {
                    (u.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min),
                     u.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max),
                     u.iter().map(|v|v[1]).fold(f32::INFINITY,f32::min),
                     u.iter().map(|v|v[1]).fold(f32::NEG_INFINITY,f32::max))
                } else {
                    // An already-authored object's historical wallpaper
                    // underlay receives an actual same-tileset floor donor.
                    let Some(donor)=source_styles.iter().enumerate().find_map(|(i,style)|
                        (*style==Some(InteriorFloor::NativeRoom)
                          && cells[i].source.tileset_id==cells[source_index].source.tileset_id
                          && (!(matches!(map,"MountMoonGiftShop"|"MahoganyMart1F")
                            ||traditional_room::floor_map(map))||cells[i].source.tile_index==1)).then_some(i))
                        else {mesh.textured.indices.extend_from_slice(quad);continue;};
                    donor_index = donor;
                    g.uv(donor%g.width,donor/g.width)
                };
                let atlas=cells[source_index].source.tileset_id.as_ref();
                if traditional_wood_wall::is_map(map)&&atlas=="traditional_house"
                    &&cells[donor_index].source.tile_index==0x50 {
                    let field=[native_uv.0+(native_uv.1-native_uv.0)*0.5/8.,
                        native_uv.2+(native_uv.3-native_uv.2)*2.5/8.];
                    native_house_floor_fields(&mut mesh.textured,bounds,height,world,true,[field;2]);
                } else if matches!(map,"MountMoonGiftShop"|"MahoganyMart1F")
                    ||traditional_room::floor_map(map)&&cells[donor_index].source.tile_index==1 {
                    native_gift_shop_floor_cell(&mut mesh.textured,bounds,height,world,native_uv,cells[donor_index].source.tile_index);
                } else if matches!(map,"UndergroundPath"|"VermilionPortPassage")&&atlas=="underground" {
                    native_underground_floor_cell(&mut mesh.textured,bounds,height,native_uv);
                } else if map=="SafariZoneMainOffice"&&atlas=="game_corner" {
                    let uv=g.uv(cells[donor_index].column as usize,cells[donor_index].row as usize);
                    // Every quarter uses the same white field/gray joint donor.
                    let donor=cells.iter().position(|t|crate::casino::safari_office_floor(map,&t.source)&&t.source.tile_index==1);
                    let uv=donor.map_or(uv,|i|g.uv(cells[i].column as usize,cells[i].row as usize));
                    let sample=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
                    let gray_donor=cells.iter().position(|t|crate::casino::safari_office_floor(map,&t.source)&&t.source.tile_index==16);
                    let gray=gray_donor.map_or(sample(0.,0.),|i| {
                        let u=g.uv(cells[i].column as usize,cells[i].row as usize);
                        [u.0+(u.1-u.0)*0.5/8.,u.2+(u.3-u.2)*3.5/8.]
                    });
                    native_safari_floor_cell(&mut mesh.textured,bounds,height,world,sample(0.,0.),sample(1.,3.),gray);
                } else if map == "CeladonCafe" {
                    native_cafe_material_floor_cell(&mut mesh.textured, bounds, height,
                        world, native_uv, cells[donor_index].source.tile_index);
                } else if map=="SaffronMagnetTrainStation" && atlas=="train_station"
                    && cells[donor_index].source.tile_index==0x3d {
                    native_station_material_floor_cell(&mut mesh.textured,bounds,height,world,native_uv);
                } else if atlas=="pokecenter" && matches!(cells[donor_index].source.tile_index,1|0x11) {
                    native_center_material_floor_cell(&mut mesh.textured,bounds,height,world,native_uv);
                } else if atlas == "gate" {
                    native_gate_material_floor_cell(&mut mesh.textured, bounds, height, world, native_uv);
                } else if atlas == "mart"
                    && cells[donor_index].source.tile_index == crate::mart::STANDARD_MART_FLOOR_TILE {
                    native_mart_material_floor_cell(&mut mesh.textured, bounds, height, world, native_uv);
                } else if matches!(map,"SilphCo1F"|"TrainerHouseB1F") && atlas == "facility" {
                    native_silph_material_floor_cell(&mut mesh.textured, bounds, height,
                        world, native_uv, cells[donor_index].source.tile_index);
                } else if reviewed_native_room(map) && matches!(atlas,"house"|"players_house"|"players_room") {
                    native_house_material_floor_cell(&mut mesh.textured,bounds,height,world,native_uv,matches!(atlas,"players_house"|"players_room"));
                } else if reviewed_native_room(map) && map.starts_with("CeladonMansion")
                    && atlas=="mansion" && cells[donor_index].source.tile_index==0x11 {
                    // Tile11's white/medium pair is the native pink checker.
                    // Fit physical ceramic courses instead of enlarged pixels.
                    native_house_material_floor_cell(&mut mesh.textured,bounds,height,world,native_uv,false);
                } else {native_beveled_floor_cell(&mut mesh.textured,bounds,height,world,native_uv);}

            } else if style == InteriorFloor::StoreCeramic && store_floor_map(map) {
                let ids = [quad[0], quad[1], quad[2], quad[5]];
                let u = ids.map(|i| mesh.textured.uvs[i as usize]);
                let sc = (((u[0][0] + u[2][0]) * 0.5 * g.width as f32).floor() as usize).min(g.width - 1);
                let sr = (((u[0][1] + u[2][1]) * 0.5 * g.height as f32).floor() as usize).min(g.height - 1);
                let uv = if source_styles[sr * g.width + sc] == Some(InteriorFloor::StoreCeramic) {
                    (u.iter().map(|v| v[0]).fold(f32::INFINITY, f32::min),
                     u.iter().map(|v| v[0]).fold(f32::NEG_INFINITY, f32::max),
                     u.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min),
                     u.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max))
                } else { g.uv(destination % g.width, destination / g.width) };
                native_deptstore_material_floor_cell(&mut mesh.textured, bounds, height,
                    world, uv);
            } else if gym_floor_style(style) {
                let native_uv =
                    // Backing quads may use an authenticated floor donor. Keep
                    // those exact UVs instead of sampling the planter drawing.
                    { let ids=[quad[0],quad[1],quad[2],quad[5]];
                      let u=ids.map(|i|mesh.textured.uvs[i as usize]);
                      (u.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min),
                       u.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max),
                       u.iter().map(|v|v[1]).fold(f32::INFINITY,f32::min),
                       u.iter().map(|v|v[1]).fold(f32::NEG_INFINITY,f32::max)) };
                gym_native_floor_cell(&mut mesh.textured,bounds,height,style,world,native_uv);
            } else if style == InteriorFloor::TowerDepth {
                tower_grating_cell(&mut mesh.textured, bounds, height,
                    cells[destination].source.tile_index,
                    g.uv(destination % g.width, destination / g.width));
            } else if style == InteriorFloor::FuchsiaNative {
                gym_native_floor_cell(&mut mesh.textured,bounds,height,
                    InteriorFloor::GymTimber,world,g.uv(destination%g.width,destination/g.width));
            } else if matches!(style,InteriorFloor::MahoganyIce|InteriorFloor::OlivineRunner|InteriorFloor::ArcadeChecker|InteriorFloor::TowerDepth) {
                native_beveled_floor_cell(&mut mesh.textured, bounds, height, world,
                    g.uv(destination % g.width,destination / g.width));
            } else if matches!(style,InteriorFloor::MahoganyGrip|InteriorFloor::OlivineTile) {
                native_rough_floor_cell(&mut mesh.textured,bounds,height,
                    g.uv(destination % g.width,destination / g.width));
            } else if style == InteriorFloor::Tatami
                && traditional_room::tatami_source(map, &cells[destination].source)
            {
                // Keep the native green weave and alternating mat borders.
                // These complete, zero-height drawings are floor, not cushions.
                append_top(&mut mesh.textured, bounds.into(), height,
                    g.uv(destination % g.width, destination / g.width));
            } else if style == InteriorFloor::LighthouseChecker {
                let edges = lighthouse_chamber_floor_edges(mesh, map, cells, g, destination);
                lighthouse_chamber_floor_cell(&mut mesh.solid, bounds, height, world, edges);
            } else {
                floor_cell(&mut mesh.solid, bounds, height, style, world);
            }
            finished += 1;
        } else {
            mesh.textured.indices.extend_from_slice(quad);
        }
    }
    if finished > 0 {
        compact_surface(&mut mesh.textured);
    }
    append_known_room_backing(mesh, map, cells, g, world_origin, excluded);
    if floor_trace {eprintln!("interior-floor map={map} finished={finished}");}
    finished
}
fn compact_surface(mesh: &mut SurfaceMeshData) {
    let mut rebuilt = SurfaceMeshData::default();
    let mut mapping = vec![u32::MAX; mesh.positions.len()];
    let mut used=vec![false;mesh.positions.len()];
    for &i in &mesh.indices {used[i as usize]=true;}
    let mut owner=vec![None;mesh.positions.len()];
    for (group,range) in mesh.cutaway_ranges.iter().enumerate() {
        for i in range.clone() {
            assert!(owner[i].replace(group).is_none(),"fade groups must have exclusive ownership");
        }
    }
    // Pack opaque vertices first, then each complete fade owner. Group order
    // remains stable for the terrain's cross-domain cutaway links.
    let mut copy=|old:usize,out:&mut SurfaceMeshData| {
        mapping[old]=out.positions.len()as u32;
        out.positions.push(mesh.positions[old]);out.normals.push(mesh.normals[old]);
        out.uvs.push(mesh.uvs[old]);out.colors.push(mesh.colors[old]);
    };
    for i in 0..used.len() {if used[i]&&owner[i].is_none() {copy(i,&mut rebuilt);}}
    for (group,range) in mesh.cutaway_ranges.iter().enumerate() {
        let first=rebuilt.positions.len();
        for i in range.clone() {if used[i]&&owner[i]==Some(group) {copy(i,&mut rebuilt);}}
        rebuilt.cutaway_ranges.push(first..rebuilt.positions.len());
    }
    rebuilt.indices=mesh.indices.iter().map(|&i|mapping[i as usize]).collect();
    *mesh = rebuilt;
}

#[cfg(test)]
#[test]
fn kanto_floor_compaction_preserves_complete_fade_owners_and_triangle_attributes() {
    let mut mesh=SurfaceMeshData::default();
    for x in [0.,8.,16.,24.] {
        append_quad(&mut mesh,[[x,2.,1.],[x,2.,5.],[x+4.,2.,5.],[x+4.,2.,1.]],
            [0.,1.,0.],[[0.,0.],[0.,1.],[1.,1.],[1.,0.]],TEXTURED_SHADE);
    }
    // Interleave triangle references; the third quad is unused scratch.
    mesh.indices=vec![4,5,6,0,1,2,12,13,14,4,6,7,0,2,3,12,14,15];
    mesh.cutaway_ranges=vec![4..8,12..16];
    let (opaque,groups)=crate::occluder_fade::prepare(mesh.clone());
    let before=crate::occluder_fade::geometry_signature(&opaque,groups);
    compact_surface(&mut mesh);
    assert_eq!(mesh.positions.len(),12);
    assert_eq!(mesh.cutaway_ranges,vec![4..8,8..12]);
    let (opaque,groups)=crate::occluder_fade::prepare(mesh);
    assert_eq!(groups.len(),2,"one owner per complete object");
    assert_eq!(opaque.indices.len(),6,"floor remains opaque");
    assert_eq!(crate::occluder_fade::geometry_signature(&opaque,groups),before);
}
pub(super) fn architecture_box(mesh: &mut SurfaceMeshData, b: [f32; 6], color: [f32; 3]) {
    let [x0, x1, y0, y1, z0, z1] = b;
    for (p, n, shade) in [
        (
            [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]],
            [0.0, 1.0, 0.0],
            1.0,
        ),
        (
            [[x1, y0, z1], [x1, y1, z1], [x0, y1, z1], [x0, y0, z1]],
            [0.0, 0.0, 1.0],
            0.88,
        ),
        (
            [[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]],
            [0.0, 0.0, -1.0],
            0.74,
        ),
        (
            [[x0, y0, z1], [x0, y1, z1], [x0, y1, z0], [x0, y0, z0]],
            [-1.0, 0.0, 0.0],
            0.80,
        ),
        (
            [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]],
            [1.0, 0.0, 0.0],
            0.91,
        ),
        (
            [[x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]],
            [0.0, -1.0, 0.0],
            0.60,
        ),
    ] {
        append_quad(
            mesh,
            p,
            n,
            [[0.0, 0.0]; 4],
            [color[0] * shade, color[1] * shade, color[2] * shade, 1.0],
        );
    }
}
fn bedroom_course(cells: &[&VisualTile], g: &GridGeometry) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    if g.width < 16 || g.height < 4 {
        return out;
    }
    for row in 0..=g.height - 4 {
        for column in 0..=g.width - 16 {
            let complete = (0..4).all(|y| {
                (0..16).all(|x| {
                    let s = &cells[(row + y) * g.width + column + x].source;
                    s.tileset_id.as_ref() == "players_room"
                        && s.subtile_column as usize == x % 4
                        && s.subtile_row as usize == y
                        && match x / 4 {
                            0 => matches!(
                                s.metatile_id,
                                0x04 | 0x08 | 0x0b | 0x0e | 0x11 | 0x14 | 0x17
                            ),
                            1 => s.metatile_id == 0x01,
                            2 => s.metatile_id == 0x03,
                            3 => matches!(s.metatile_id, 0x02 | 0x1f | 0x23 | 0x24 | 0x25),
                            _ => false,
                        }
                })
            });
            if complete {
                out.push((column, row, 16));
            }
        }
    }
    out
}
fn native_bedroom_backing_source(map:&str,cells:&[&VisualTile],g:&GridGeometry,column:usize,row:usize)->bool {
    if map!="PlayersHouse2F" || column+16>g.width || row+4>g.height {return false;}
    const DRAWINGS:[[u16;16];3]=[
        [2,2,2,2,2,2,2,2,1,1,1,1,1,1,1,1],
        [2,2,2,2,11,12,2,2,27,28,66,67,43,44,82,83],
        [2,2,5,6,59,60,21,22,75,76,37,38,91,92,53,54],
    ];
    (0..4).all(|y|(0..16).all(|x| {
        let source=&cells[(row+y)*g.width+column+x].source;let block=x/4;
        if source.tileset_id.as_ref()!="players_room" || source.subtile_column as usize!=x%4
            || source.subtile_row as usize!=y {return false;}
        if block<3 {
            source.metatile_id==[4,1,3][block] && source.tile_index==DRAWINGS[block][y*4+x%4]
        } else if y>=2 {source.tile_index==1 && matches!(source.metatile_id,2|0x1f|0x23..=0x25)}
        else if x%4>=2 {crate::interior::player_room_stair_local(source).is_some()}
        else if source.metatile_id==2 {source.tile_index==2}
        else {INTERIOR_DRAWINGS.iter().any(|drawing|drawing.tileset=="players_room"
            && drawing.kind==ModelKind::PictureFrame && drawing.block==source.metatile_id
            && drawing.origin==[0,0] && drawing.size==[2,2]
            && source.tile_index==drawing.tiles[y*2+x%4])}
    }))
}
fn native_living_room_backing_source(map:&str,cells:&[&VisualTile],g:&GridGeometry,column:usize,row:usize)->bool {
    if map!="PlayersHouse1F" || column+20>g.width || row+4>g.height {return false;}
    const BLOCKS:[u16;5]=[7,15,17,5,10];
    const DRAWINGS:[[u16;16];5]=[
        [51,50,50,52,17,17,17,17,80,81,67,69,82,83,24,25],
        [17,17,37,53,10,11,37,53,26,27,37,53,42,43,37,53],
        [17,17,17,17,6,7,17,17,22,23,14,15,8,9,58,59],
        [17,17,17,17,17,17,17,17,1,1,1,1,1,1,1,1],
        [17,17,76,77,17,17,92,93,1,1,1,1,1,1,1,1],
    ];
    (0..4).all(|y|(0..20).all(|x| {
        let source=&cells[(row+y)*g.width+column+x].source;
        source.tileset_id.as_ref()=="players_house" && source.metatile_id==BLOCKS[x/4]
            && source.subtile_column as usize==x%4 && source.subtile_row as usize==y
            && source.tile_index==DRAWINGS[x/4][y*4+x%4]
    }))
}
fn append_native_home_backing(mesh:&mut TerrainMeshData,g:&GridGeometry,column:usize,row:usize,width:usize,donor:[usize;2]) {
    let th=g.tile_height;let height=4.*th;let front=g.origin_z+(row as f32+1.8)*th;
    let uv=g.uv(column+donor[0],row+donor[1]);
    let sample=|x:usize,y:usize|[uv.0+(uv.1-uv.0)*(x as f32+0.5)/8.,uv.2+(uv.3-uv.2)*(y as f32+0.5)/8.];
    // Both home atlases draw the same plank: body at(1,0), light west edge,
    // dark east joint, and a small light grain mark at(3,2).
    let paint=sample(0,0);let field=sample(1,0);let joint=sample(7,0);let grain=sample(3,2);
    for x in 0..width {
        let w=g.origin_x+(column+x) as f32*g.tile_width;let e=w+g.tile_width;
        let first=mesh.textured.positions.len();let indices=mesh.textured.indices.len();
        architecture_box(&mut mesh.textured,[w,e,0.,height,front-th*0.16,front],[1.;3]);
        mesh.textured.uvs[first..].fill(field);
        mesh.textured.indices.drain(indices+6..indices+12);
        let xs=[0.,0.025,0.34,0.43,0.95,0.975,1.];
        let ys=[0.,0.20,0.30,0.65,0.75,1.];
        let point=|x:f32,y:f32|[w+(e-w)*x,height*(1.-y),
            front-if x==0.975 {th*0.025}else{0.}];
        let texture=|x:f32,y:f32|if x>0.95 {joint}else if x<0.025 {paint}
            else if x>0.34 && x<0.43 && ((y>0.20 && y<0.30)||(y>0.65 && y<0.75)) {grain}else{field};
        native_source_surface_patches(&mut mesh.textured,&xs,&ys,point,texture,|_,_|1.,true);
    }
    // The last two columns retain the complete native downstairs opening.
    let first=mesh.textured.positions.len();
    architecture_box(&mut mesh.textured,[g.origin_x+column as f32*g.tile_width,g.origin_x+(column+width) as f32*g.tile_width,
        height,height+th*0.10,front-th*0.21,front+th*0.22],[1.;3]);
    mesh.textured.uvs[first..].fill(paint);
}

fn append_known_room_backing(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
    excluded: &[bool],
) {
    // These rooms have closed, source-owned wall/window/cabinet volumes.
    // The older whole-course scaffold adds a second, taller cream wall and
    // also accepts block metadata after its actual wall art has changed.
    if matches!(map,"PlayersHouse1F"|"PlayersHouse2F") {
        let width=if map=="PlayersHouse2F" {16}else{20};
        for row in 0..g.height.saturating_sub(3) {for column in 0..g.width.saturating_sub(width-1) {
            if world_origin[0]+column as i32!=0 || world_origin[1]+row as i32!=0 {continue;}
            if (0..4).any(|y|(0..width).any(|x|excluded.get((row+y)*g.width+column+x)==Some(&true))) {continue;}
            if native_bedroom_backing_source(map,cells,g,column,row) {
                append_native_home_backing(mesh,g,column,row,14,[0,0]);
            } else if native_living_room_backing_source(map,cells,g,column,row) {
                append_native_home_backing(mesh,g,column,row,18,[4,0]);
            }
        }}
        return;
    }
    if reviewed_native_room(map) { return; }
    if traditional_wood_wall::is_map(map) {
        traditional_wood_wall::append_backing(mesh,map,cells,g,world_origin,excluded);
        return;
    }
    // Existing complete native wall-course matchers are the room blueprint.
    // This makes no wall from collision, an arbitrary flat cell or a viewport edge.
    let mut courses: Vec<_> = ordinary_house::course_origins(cells, g)
        .into_iter()
        .map(|(x, y, w)| (x, y, w, ModelKind::WallDomestic))
        .collect();
    courses.extend(
        players_house::course_origins(cells, g)
            .into_iter()
            .map(|(x, y, w)| (x, y, w, ModelKind::WallDomestic)),
    );
    courses.extend(
        traditional_house::course_origins(cells, g)
            .into_iter()
            .map(|(x, y, w, _)| (x, y, w, ModelKind::WallTraditional)),
    );
    courses.extend(
        bedroom_course(cells, g)
            .into_iter()
            .map(|(x, y, w)| (x, y, w, ModelKind::WallDomestic)),
    );
    for (column, row, width, kind) in courses {
        let tw = g.tile_width;
        let th = g.tile_height;
        let x0 = g.origin_x + column as f32 * tw;
        let x1 = x0 + width as f32 * tw;
        let front = g.origin_z + (row as f32 + 1.80) * th;
        let height = 4.0 * th;
        let mut opening = vec![false; width];
        for x in 0..width {
            opening[x] = (0..4).any(|y| {
                house_stair_local(map, &cells[(row + y) * g.width + column + x].source).is_some()
            });
        }
        // Group matching pairs, preserving the actual stair opening. Existing
        // complete window, shelf and picture models stand in front of this wall.
        let mut x = 0;
        while x < width {
            if opening[x] {
                x += 1;
                continue;
            }
            let w = if x + 1 < width && !opening[x + 1] {
                2
            } else {
                1
            };
            model(kind).append_fitted(
                &mut mesh.solid,
                [
                    x0 + x as f32 * tw,
                    x0 + (x + w) as f32 * tw,
                    front - th * 0.16,
                    front,
                ],
                0.0,
                height,
            );
            x += w;
        }
        // One uninterrupted lintel/cornice connects the panel kit visually.
        architecture_box(
            &mut mesh.solid,
            [
                x0,
                x1,
                height - th * 0.06,
                height + th * 0.10,
                front - th * 0.21,
                front + th * 0.22,
            ],
            [0.45, 0.34, 0.24],
        );
        architecture_box(
            &mut mesh.solid,
            [
                x0,
                x1,
                height + th * 0.10,
                height + th * 0.17,
                front - th * 0.20,
                front + th * 0.20,
            ],
            [0.62, 0.49, 0.33],
        );
        // A shallow below-floor cutaway section is safe only for the complete
        // bedroom blueprint, never for a cropped viewport or a guessed room.
        if map == "PlayersHouse2F"
            && column == 0
            && row == 0
            && width == 16
            && g.width == 16
            && g.height == 12
            && world_origin == [0, 0]
        {
            let north = g.origin_z + 2.0 * th;
            let south = g.origin_z + 12.0 * th;
            let color = [0.35, 0.28, 0.22];
            let depth = th * 0.38;
            for (a, b, c, d) in [
                (x0, x0 + tw * 0.045, north, south),
                (x1 - tw * 0.045, x1, north, south),
                (x0, x1, south - th * 0.045, south),
            ] {
                architecture_box(&mut mesh.solid, [a, b, -depth, -0.001, c, d], color);
            }
        }
    }
}

#[cfg(test)]
mod surface_finish_tests {
    use super::*;
    use std::sync::Arc;
    fn floor() -> (Vec<VisualTile>, GridGeometry) {
        (
            vec![VisualTile {
                animation_frames: None,
                column: 0,
                row: 0,
                texture: Handle::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("players_room"),
                    metatile_id: 0x05,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: 0x01,
                },
            }],
            GridGeometry {
                width: 1,
                height: 1,
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: 0.0,
                origin_z: 0.0,
            },
        )
    }
    #[test]
    fn tower_grating_retains_gold_rails_dark_depth_exact_datum_and_ownership() {
        for scale in [0.5,1.,2.] {
            for (tile,x,y,rails) in [(0x0a,0,0,2),(0x0b,1,0,3),(0x1a,0,1,2),(0x1b,1,1,3)] {
                let (mut tiles,mut g)=floor();g.tile_width=8.*scale;g.tile_height=8.*scale;
                tiles[0].source=VisualTileSource {tileset_id:"tower".into(),metatile_id:7,
                    subtile_column:x,subtile_row:y,tile_index:tile};
                let refs=tiles.iter().collect::<Vec<_>>();
                let build=|| {let mut mesh=TerrainMeshData {footing_heights:vec![7.],..Default::default()};
                    append_top(&mut mesh.textured,g.bounds(0,0).into(),0.,g.uv(0,0));mesh};
                let mut mesh=build();
                assert_eq!(finish_surfaces_excluding(&mut mesh,"SproutTower2F",&refs,&g,[0,0],&[false]),1);
                assert_eq!(mesh.textured.indices.len(),6+rails*36);
                assert_eq!(mesh.footing_heights,vec![7.]);assert!(mesh.solid.positions.is_empty());
                assert!(mesh.textured.positions.iter().all(|p| p[0]>=0.&&p[0]<=8.*scale
                    &&p[2]>=0.&&p[2]<=8.*scale&&p[1]<=0.&&p[1]>=-2.*scale));
                for uv in [[0.5/8.,2.5/8.],[4.5/8.,1.5/8.],[4.5/8.,5.5/8.]] {
                    assert!(mesh.textured.uvs.contains(&uv));
                }
                let mut custom=build();let before=custom.textured.positions.clone();
                assert_eq!(finish_surfaces_excluding(&mut custom,"SproutTower2F",&refs,&g,[0,0],&[true]),0);
                assert_eq!(custom.textured.positions,before);
                let mut stale=tiles.clone();stale[0].source.tile_index=0x77;
                let mut original=build();
                assert_eq!(finish_surfaces_excluding(&mut original,"SproutTower2F",&stale.iter().collect::<Vec<_>>(),&g,[0,0],&[false]),0);
                assert_eq!(original.textured.indices.len(),6);
            }
        }
    }
    #[test]
    fn fuchsia_gray_floor_rejects_lab_wood_substitution_and_keeps_hidden_barrier_height() {
        let (original,mut g)=floor();g.width=4;g.height=4;
        let tiles:Vec<_>=(0..16).map(|i| {
            let mut t=original[0].clone();t.column=(i%4) as u32;t.row=(i/4) as u32;
            t.source.tileset_id="lab".into();t.source.metatile_id=0x1d;
            t.source.subtile_column=(i%4) as u8;t.source.subtile_row=(i/4) as u8;
            t.source.tile_index=if i%4<2 {0x37} else {0x10};t
        }).collect();
        let refs=tiles.iter().collect::<Vec<_>>();
        let build=|| {
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],..Default::default()};
            for i in 0..16 {append_top(&mut mesh.textured,g.bounds(i%4,i/4).into(),0.,g.uv(i%4,i/4));}
            mesh
        };
        let mut mesh=build();
        assert_eq!(finish_surfaces_excluding(&mut mesh,"FuchsiaGym",&refs,&g,[0,0],&[false;16]),16);
        assert!(mesh.solid.positions.is_empty(),"Fuchsia must not inherit the beige lab floor");
        assert_eq!(mesh.footing_heights,vec![0.;16]);
        assert_eq!(mesh.textured.indices.len(),16*108);
        assert!(mesh.textured.positions.iter().all(|p|p[1]<=0.&&p[1]>=-0.05));
        assert!(mesh.textured.uvs.iter().all(|uv|uv.iter().all(|v|(0. ..=1.).contains(v))));
        let mut reserved=[false;16];reserved[0]=true;
        let mut custom=build();
        assert_eq!(finish_surfaces_excluding(&mut custom,"FuchsiaGym",&refs,&g,[0,0],&reserved),15);
        let mut bad=tiles.clone();bad[5].source.metatile_id=0x2d;
        assert_eq!(interior_floor("FuchsiaGym",&bad[5].source),None);
        let mut changed=build();
        assert_eq!(finish_surfaces_excluding(&mut changed,"FuchsiaGym",&bad.iter().collect::<Vec<_>>(),&g,[0,0],&[false;16]),15);
        let mut raised=build();
        for p in &mut raised.textured.positions {p[1]=4.;}
        assert_eq!(finish_surfaces_excluding(&mut raised,"FuchsiaGym",&refs,&g,[0,0],&[false;16]),0);
    }
    #[test]
    fn violet_brick_floor_covers_each_point_once_at_the_original_datum() {
        for row in [0, 1] {
            let mut mesh = SurfaceMeshData::default();
            floor_cell(&mut mesh, [0., 8., 0., 8.], 3.25, InteriorFloor::VioletBrick, [0, row]);
            let area: f32 = mesh.indices.chunks_exact(3).map(|tri| {
                let a = Vec3::from_array(mesh.positions[tri[0] as usize]);
                let b = Vec3::from_array(mesh.positions[tri[1] as usize]);
                let c = Vec3::from_array(mesh.positions[tri[2] as usize]);
                let area = (b - a).cross(c - a).length() * 0.5;
                assert!(area > 0., "floor must not contain degenerate triangles");
                area
            }).sum();
            assert!((area - 64.).abs() < 0.0001, "coplanar floor paint overlaps: {area}");
            assert!(mesh.positions.iter().all(|p| p[1] == 3.25));
        }
    }

    #[test]
    fn store_lift_live_floor_survives_final_surface_finish() {
        let (mut tiles, g) = floor();
        tiles[0].source.tileset_id = Arc::from("mart");
        tiles[0].source.metatile_id = 6;
        let refs = tiles.iter().collect::<Vec<_>>();
        for map in ["GoldenrodDeptStoreElevator", "CeladonDeptStoreElevator"] {
            let mut mesh = TerrainMeshData { footing_heights: vec![0.], ..Default::default() };
            append_top(&mut mesh.textured, [0., 8., 0., 8.], 0., g.uv(0, 0));
            let original = mesh.textured.clone();
            assert_eq!(finish_surfaces_excluding(&mut mesh, map, &refs, &g, [0, 0], &[false]), 0);
            assert_eq!(mesh.textured, original);
            assert!(mesh.solid.indices.is_empty());
            assert_eq!(mesh.footing_heights, vec![0.]);
        }
        assert_eq!(interior_floor("GoldenrodDeptStore1F", &tiles[0].source), Some(InteriorFloor::StoreCeramic));
    }
    #[test]
    fn native_department_floor_preserves_source_pigment_area_at_larger_slab_scale() {
        let mut mesh=SurfaceMeshData::default();
        for y in 0..2 {for x in 0..2 {
            native_deptstore_material_floor_cell(&mut mesh,
                [x as f32*8.,(x+1)as f32*8.,y as f32*8.,(y+1)as f32*8.],
                2.5,[x,y],(0.,1.,0.,1.));
        }}
        let samples=[[0.5/8.,0.5/8.],[3.5/8.,0.5/8.],[5.5/8.,5.5/8.]];
        let mut areas=[0.;3];
        for tri in mesh.indices.chunks_exact(3) {
            let p=tri.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).collect::<Vec<_>>();
            let role=samples.iter().position(|uv|*uv==mesh.uvs[tri[0]as usize]).unwrap();
            areas[role]+=(p[1]-p[0]).cross(p[2]-p[0]).length()*0.5;
        }
        for (area,pixels) in areas.into_iter().zip([45.,14.,5.]) {
            assert!((area-256.*pixels/64.).abs()<0.001,"native pigment area changed");
        }
        assert!(mesh.positions.iter().all(|p|p[1]==2.5));
    }
    #[test]
    fn department_store_floor_finishes_every_source_cell_and_respects_reservations() {
        let (tile,mut g)=floor();g.width=2;g.height=2;
        let tiles=(0..4).map(|i| {
            let mut t=tile[0].clone();t.column=(i%2) as u32;t.row=(i/2) as u32;
            t.source.tileset_id=Arc::from("mart");t.source.metatile_id=4;
            t.source.subtile_column=(i%2) as u8;t.source.subtile_row=(i/2) as u8;t
        }).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {footing_heights:vec![0.0;4],authored_cells:vec![None;4],..Default::default()};
        for i in 0..4 {
            // Fixture backing may address another atlas cell; the destination
            // floor still has to finish in the same tile phase without gaps.
            append_top(&mut mesh.textured,g.bounds(i%2,i/2).into(),0.0,g.uv(0,0));
        }
        assert_eq!(finish_surfaces_excluding(&mut mesh,"GoldenrodDeptStore1F",&refs,&g,[0,0],&[true,false,false,false]),3);
        assert_eq!(mesh.footing_heights,vec![0.0;4]);
        assert_eq!(mesh.authored_cells,vec![None;4]);
        assert_eq!(&mesh.textured.indices[..6],&[0,1,2,0,2,3]);
        let area=mesh.textured.indices.chunks_exact(3).map(|tri| {
            let p=tri.iter().map(|&i|Vec3::from_array(mesh.textured.positions[i as usize])).collect::<Vec<_>>();
            (p[1]-p[0]).cross(p[2]-p[0]).length()*0.5
        }).sum::<f32>();
        assert!((area-256.).abs()<0.001,"source floor coverage must have no gaps or overlaps");
        assert!(mesh.textured.positions.iter().all(|p|p[1]==0.));
        assert!(mesh.solid.positions.iter().all(|p|p[1]==0.0));
    }
    #[test]
    fn store_floor_angular_glazed_pavers_keep_contrast_area_and_walking_height() {
        for scale in [0.5, 1., 1.5] {
            let mut mesh = SurfaceMeshData::default();
            floor_cell(&mut mesh, [0., 8.*scale, 0., 8.*scale], 2.5,
                InteriorFloor::StoreCeramic, [-3, 5]);
            assert!(mesh.positions.iter().all(|p| p[1] == 2.5));
            assert!(mesh.normals.iter().all(|n| *n == [0., 1., 0.]));
            assert!(mesh.colors.iter().any(|c| c[0] > 0.99 && c[1] > 0.99 && c[2] < 0.05));
            assert!(mesh.colors.iter().any(|c| c[0] > 0.99 && c[1] < 0.24 && c[2] < 0.01));
            assert!(mesh.colors.iter().any(|c| c[2] > 0.65));
            let rectangles = mesh.positions.chunks_exact(4).map(|q| {
                [q.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min),
                 q.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max),
                 q.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
                 q.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max)]
            }).collect::<Vec<_>>();
            let area: f32 = rectangles.iter().map(|r| (r[1]-r[0])*(r[3]-r[2])).sum();
            assert!((area - 64.*scale*scale).abs() < 0.001);
            for (i, a) in rectangles.iter().enumerate() {
                for b in &rectangles[i+1..] {
                    let overlap = (a[1].min(b[1])-a[0].max(b[0])).max(0.)
                        * (a[3].min(b[3])-a[2].max(b[2])).max(0.);
                    assert!(overlap < 0.00001, "overlapping colored pavers");
                }
            }
        }
    }
    #[test]
    fn blackthorn_blue_brick_floor_partitions_preserve_area_and_walking_height() {
        for scale in [0.5,1.,1.5] {
            let mut mesh = SurfaceMeshData::default();
            floor_cell(&mut mesh, [0.,8.*scale,0.,8.*scale], 2.5,
                InteriorFloor::BlackthornTile, [-3,5]);
            assert!(mesh.positions.iter().all(|p| p[1] == 2.5));
            assert!(mesh.normals.iter().all(|n| *n == [0.,1.,0.]));
            let rectangles = mesh.positions.chunks_exact(4).map(|quad| {
                [quad.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),
                 quad.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max),
                 quad.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min),
                 quad.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max)]
            }).collect::<Vec<_>>();
            let area: f32 = rectangles.iter().map(|r|(r[1]-r[0])*(r[3]-r[2])).sum();
            assert!((area-64.*scale*scale).abs() < 0.001);
            for (i,a) in rectangles.iter().enumerate() {
                for b in &rectangles[i+1..] {
                    let overlap = (a[1].min(b[1])-a[0].max(b[0])).max(0.)
                        * (a[3].min(b[3])-a[2].max(b[2])).max(0.);
                    assert!(overlap < 0.00001);
                }
            }
            assert!(mesh.colors.iter().any(|c| c[2] == 1. && c[0] < 0.3));
        }
    }
    #[test]
    fn authored_floor_keeps_height_footing_and_object_evidence_separate() {
        let (tiles, g) = floor();
        let refs: Vec<_> = tiles.iter().collect();
        for map in ["PlayersHouse2F","UnreviewedHome2F"] {
            let mut mesh = TerrainMeshData::default();
            append_top(&mut mesh.textured, [0.0, 8.0, 0.0, 8.0], 3.5, g.uv(0, 0));
            mesh.footing_heights = vec![3.5];
            mesh.authored_cells = vec![None];
            assert_eq!(finish_surfaces(&mut mesh,map,&refs,&g,[4,6]),1);
            assert_eq!(mesh.footing_heights,vec![3.5]);
            assert_eq!(mesh.authored_cells,vec![None]);
            if map=="PlayersHouse2F" {
                assert!(!mesh.textured.indices.is_empty());
                assert!(mesh.solid.indices.is_empty());
                assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.5 && p[1]>=3.4),
                    "source wood relief stays below the unchanged actor datum");
            } else {
                assert!(mesh.textured.indices.is_empty());
                assert!(mesh.solid.positions.iter().all(|p|near(p[1],3.5)));
            }
        }
    }
    #[test]
    fn unknown_prop_strip_and_upright_art_are_not_floor_materials() {
        let (mut tiles, g) = floor();
        tiles[0].source.tile_index = 0x30;
        let refs: Vec<_> = tiles.iter().collect();
        let mut mesh = TerrainMeshData::default();
        append_top(&mut mesh.textured, [0.0, 8.0, 0.0, 8.0], 0.0, g.uv(0, 0));
        let before = mesh.textured.clone();
        assert_eq!(
            finish_surfaces(&mut mesh, "PlayersHouse2F", &refs, &g, [0, 0]),
            0
        );
        assert_eq!(mesh.textured, before);
    }
    #[test]
    fn indoor_material_coordinates_keep_world_phase_and_floor_area() {
        for style in [InteriorFloor::StoreCeramic, InteriorFloor::AmberTimber] {
            let mut a = SurfaceMeshData::default();
            let mut b = SurfaceMeshData::default();
            floor_cell(&mut a,[0.,8.,0.,8.],0.,style,[7,9]);
            floor_cell(&mut b,[32.,40.,48.,56.],0.,style,[7,9]);
            assert_eq!(a.uvs,b.uvs);
            assert_eq!(a.colors,b.colors);
            assert!(a.uvs.iter().any(|uv| (80000.0..81000.0).contains(&uv[0])));
            assert!(a.positions.iter().all(|p| p[1] == 0.));
            let area: f32 = a.indices.chunks_exact(3).map(|ids| {
                let p = ids.iter().map(|i| Vec3::from_array(a.positions[*i as usize])).collect::<Vec<_>>();
                (p[1]-p[0]).cross(p[2]-p[0]).length()*0.5
            }).sum();
            assert!((area-64.).abs()<0.001);
        }
        let mut ordinary = SurfaceMeshData::default();
        floor_cell(&mut ordinary,[0.,8.,0.,8.],0.,InteriorFloor::Oak,[7,9]);
        assert!(ordinary.uvs.iter().all(|uv| *uv == [0.,0.]));
    }
    #[test]
    fn plank_material_world_phase_is_stable_across_viewport_origins() {
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        floor_cell(
            &mut a,
            [0.0, 8.0, 0.0, 8.0],
            0.0,
            InteriorFloor::Oak,
            [7, 9],
        );
        floor_cell(
            &mut b,
            [32.0, 40.0, 48.0, 56.0],
            0.0,
            InteriorFloor::Oak,
            [7, 9],
        );
        assert_eq!(a.colors, b.colors);
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.positions.len(), b.positions.len());
        for (a, b) in a.positions.iter().zip(&b.positions) {
            assert!(near(a[0] + 32.0, b[0]) && near(a[2] + 48.0, b[2]));
        }
    }
}

include!("lighthouse_floor.rs");

include!("gym_floor.rs");

include!("ship_floor.rs");


#[cfg(test)]
mod kanto_room_floor_tests {
    use super::*;
    fn fixture() -> (Vec<VisualTile>,GridGeometry,TerrainMeshData) {
        let g=GridGeometry{width:2,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=(0..4).map(|i|VisualTile{
            column:(i%2)as u32,row:(i/2)as u32,texture:Default::default(),
            animation_frames:None,priority:false,
            source:VisualTileSource{tileset_id:"house".into(),metatile_id:0,
                subtile_column:(i%2)as u8,subtile_row:(i/2)as u8,tile_index:1},
        }).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData{footing_heights:vec![3.25;4],authored_cells:vec![None;4],..Default::default()};
        for i in 0..4 {append_top(&mut mesh.textured,g.bounds(i%2,i/2).into(),3.25,g.uv(i%2,i/2));}
        (tiles,g,mesh)
    }
    #[test]
    fn kanto_native_house_materials_share_seams_and_keep_source_swatches() {
        let uv=(0.,0.5,0.,0.5);let sample=[0.5/16.,0.5/16.];
        let mut first=SurfaceMeshData::default();let mut adjacent=SurfaceMeshData::default();
        native_house_material_floor_cell(&mut first,[0.,8.,0.,8.],3.25,[0,0],uv,true);
        native_house_material_floor_cell(&mut adjacent,[8.,16.,0.,8.],3.25,[1,0],uv,true);
        assert!(first.uvs.iter().all(|u|*u==sample));
        assert!(first.positions.iter().all(|p|p[1]<=3.25 && p[1]>=3.202));
        assert!(first.colors.iter().all(|c|c[0]==c[1]&&c[1]==c[2]));
        assert!(first.colors.windows(2).any(|c|c[0]!=c[1]),"subtle grain must remain visible");
        let seam=|mesh:&SurfaceMeshData| {
            let mut points=mesh.positions.iter().filter(|p|p[0]==8.).map(|p|[p[1].to_bits(),p[2].to_bits()]).collect::<Vec<_>>();
            points.sort();points.dedup();points
        };
        assert_eq!(seam(&first),seam(&adjacent));
        for wood in [false,true] {for col in [-3,-2,-1,0,1,2,3] {
            let mut mesh=SurfaceMeshData::default();
            native_house_material_floor_cell(&mut mesh,[0.,8.,0.,8.],0.,[col,-3],uv,wood);
            for ids in mesh.indices.chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let normal=Vec3::from_array(mesh.normals[ids[0]as usize]);
                assert!((b-a).cross(c-a).dot(normal)>0.);
            }
            assert!(mesh.uvs.iter().all(|u|*u==sample||(!wood&&*u==[1.5/16.,0.5/16.])));
        }}
    }
    #[test]
    fn kanto_native_center_ceramic_keeps_local_palette_footing_and_service_apron_guards() {
        let (mut tiles,g,mut mesh)=fixture();
        for tile in &mut tiles {tile.source.tileset_id="pokecenter".into();
            tile.source.metatile_id=4;tile.source.tile_index=0x11;}
        assert_eq!(finish_surfaces(&mut mesh,"CinnabarPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,[0,0]),4);
        assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
        assert!(mesh.solid.indices.is_empty());
        assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25 && p[1]>=3.202));
        for uv in &mesh.textured.uvs {
            let x=(uv[0]*16.).rem_euclid(8.);let y=(uv[1]*16.).rem_euclid(8.);
            assert!(near(x,0.5)||near(x,2.5));assert!(near(y,0.5));
        }
        let mut apron=tiles[0].source.clone();apron.tile_index=1;apron.metatile_id=5;apron.subtile_row=2;
        assert_eq!(interior_floor("CinnabarPokecenter1F",&apron),Some(InteriorFloor::NativeRoom));
        apron.metatile_id=4;
        assert_ne!(interior_floor("CinnabarPokecenter1F",&apron),Some(InteriorFloor::NativeRoom));
        assert_ne!(interior_floor("UnreviewedPokecenter1F",&tiles[0].source),Some(InteriorFloor::NativeRoom));
    }

    #[test]
    fn kanto_moon_gift_floor_uses_native_green_and_gold_without_raised_mats() {
        let (mut tiles,g,mut mesh)=fixture();
        for (i,tile) in tiles.iter_mut().enumerate() {tile.source.tileset_id="traditional_house".into();
            tile.source.metatile_id=4;tile.source.tile_index=[68,69,84,85][i];}
        let refs=tiles.iter().collect::<Vec<_>>();
        assert!(traditional_house_cushion_placements("MountMoonGiftShop",&refs,&g).is_empty());
        assert_eq!(finish_surfaces(&mut mesh,"MountMoonGiftShop",&refs,&g,[0,0]),4);
        assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
        assert!(mesh.solid.positions.is_empty());assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25&&p[1]>=3.23));
        for uv in &mesh.textured.uvs {let x=(uv[0]*16.).rem_euclid(8.);let y=(uv[1]*16.).rem_euclid(8.);
            assert!((near(x,0.5)||near(x,1.5))&&(near(y,0.5)||near(y,1.5)));}
        let mut altered=tiles[0].source.clone();altered.tile_index=2;
        assert_ne!(interior_floor("MountMoonGiftShop",&altered),Some(InteriorFloor::NativeRoom));
        altered=tiles[0].source.clone();altered.metatile_id=16;altered.subtile_column=2;
        assert_ne!(interior_floor("MountMoonGiftShop",&altered),Some(InteriorFloor::NativeRoom));
        assert_ne!(interior_floor("MahoganyMart1F",&tiles[0].source),Some(InteriorFloor::NativeRoom));
    }
    #[test]
    fn traditional_entrance_floor_keeps_gold_without_repainting_borders_or_door_mat() {
        let (mut tiles,g,mut mesh)=fixture();
        for t in &mut tiles {t.source.tileset_id="traditional_house".into();
            t.source.metatile_id=5;t.source.subtile_row+=1;t.source.tile_index=1;}
        let refs=tiles.iter().collect::<Vec<_>>();
        assert_eq!(finish_surfaces(&mut mesh,"EcruteakItemfinderHouse",&refs,&g,[0,12]),4);
        assert_eq!(mesh.footing_heights,vec![3.25;4]);
        assert_eq!(mesh.authored_cells,vec![None;4]);assert!(mesh.solid.positions.is_empty());
        for map in ["DanceTheater","KurtsHouse","EcruteakItemfinderHouse",
            "EcruteakLugiaSpeechHouse","MahoganyMart1F","VioletKylesHouse",
            "VioletNicknameSpeechHouse"] {
            for (block,mask) in [(5,0xfff0u16),(6,0x33f0),(7,0xccf0),(0x2a,0xffc0)] {
                for y in 0..4 {for x in 0..4 {
                    let mut s=tiles[0].source.clone();s.metatile_id=block;
                    s.subtile_column=x;s.subtile_row=y;
                    assert_eq!(interior_floor(map,&s)==Some(InteriorFloor::NativeRoom),
                        mask&(1<<(y*4+x))!=0,"{map} {block} {x} {y}");
                    s.tile_index=4;
                    assert_ne!(interior_floor(map,&s),Some(InteriorFloor::NativeRoom));
                }}
            }
        }
        assert_ne!(interior_floor("UnreviewedTraditionalHouse",&tiles[0].source),Some(InteriorFloor::NativeRoom));
    }
    #[test]
    fn mahogany_shop_gold_floor_preserves_source_footing_and_rejects_other_drawings() {
        let (mut tiles,g,mut mesh)=fixture();
        for t in &mut tiles {t.source.tileset_id="traditional_house".into();
            t.source.metatile_id=32;t.source.tile_index=1;}
        let refs=tiles.iter().collect::<Vec<_>>();
        assert_eq!(finish_surfaces(&mut mesh,"MahoganyMart1F",&refs,&g,[0,0]),4);
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
        assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25));
        for block in [32,33,34,5] {for row in 0..4 {for col in 0..4 {
            let mut s=tiles[0].source.clone();s.metatile_id=block;s.subtile_row=row;s.subtile_column=col;
            assert_eq!(interior_floor("MahoganyMart1F",&s)==Some(InteriorFloor::NativeRoom),
                block!=5||row>=1);
            assert_ne!(interior_floor("MahoganyMart1FAnnex",&s),Some(InteriorFloor::NativeRoom));
            s.tile_index=0xffff;
            assert_ne!(interior_floor("MahoganyMart1F",&s),Some(InteriorFloor::NativeRoom));
        }}}
        let mut invalid=tiles[0].source.clone();invalid.metatile_id=31;
        assert_ne!(interior_floor("MahoganyMart1F",&invalid),Some(InteriorFloor::NativeRoom));
    }
    #[test]
    fn kanto_safari_floor_retains_gray_panel_quadrants_and_flat_datum() {
        for (world,expected) in [([0,0],0.),([1,0],25.),([0,1],25.),([1,1],0.)] {
            let mut surface=SurfaceMeshData::default();
            native_safari_floor_cell(&mut surface,[0.,8.,0.,8.],3.25,world,[0.,0.],[0.5,0.],[1.,0.]);
            let mut gray_area=0.;
            for tri in surface.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(surface.positions[i as usize]));
                let cross=(p[1]-p[0]).cross(p[2]-p[0]);assert!(cross.y>0.);
                if surface.uvs[tri[0]as usize]==[1.,0.] {gray_area+=cross.length()*0.5;}
            }
            assert!((gray_area-expected).abs()<0.001);
            assert!(surface.positions.iter().all(|p|p[1]==3.25));
        }
        let (mut tiles,g,mut mesh)=fixture();
        for (i,t) in tiles.iter_mut().enumerate() {t.source.tileset_id="game_corner".into();
            t.source.metatile_id=1;t.source.tile_index=[1,16,17,18][i];}
        let refs=tiles.iter().collect::<Vec<_>>();
        assert_eq!(finish_surfaces(&mut mesh,"SafariZoneMainOffice",&refs,&g,[0,0]),4);
        assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
        assert!(mesh.textured.positions.iter().all(|p|p[1]==3.25));assert!(mesh.solid.positions.is_empty());
    }
    #[test]
    fn kanto_underground_floor_keeps_native_weave_colors_and_excludes_stairs_and_walls() {
        let mut surface=SurfaceMeshData::default();
        native_underground_floor_cell(&mut surface,[0.,8.,0.,8.],3.25,(0.,1.,0.,1.));
        assert_eq!(surface.indices.len(),12);let mut areas=[0.;2];
        for (j,tri) in surface.indices.chunks_exact(3).enumerate() {
            let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(surface.positions[i as usize]));
            let cross=(p[1]-p[0]).cross(p[2]-p[0]);assert!(cross.y>0.);
            areas[j%2]+=cross.length()*0.5;
        }
        assert_eq!(areas,[32.,32.]);assert!(surface.positions.iter().all(|p|p[1]==3.25));
        let (mut tiles,g,mut mesh)=fixture();
        for t in &mut tiles {t.source.tileset_id="underground".into();t.source.metatile_id=13;t.source.tile_index=16;}
        let refs=tiles.iter().collect::<Vec<_>>();
        assert_eq!(finish_surfaces(&mut mesh,"UndergroundPath",&refs,&g,[0,0]),4);
        assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
        let mut s=tiles[0].source.clone();
        for block in [2,8,9,10,12,13,14,16,17,18] {for row in 0..4 {for col in 0..4 {
            s.metatile_id=block;s.subtile_row=row;s.subtile_column=col;
            let eligible=underground_native_floor("UndergroundPath",&s);
            assert!(!underground_native_floor("GoldenrodUnderground",&s));
            if eligible {assert_eq!(interior_floor("UndergroundPath",&s),Some(InteriorFloor::NativeRoom));}
            s.tile_index=42;assert!(!underground_native_floor("UndergroundPath",&s));s.tile_index=16;
        }}}
        s.metatile_id=2;s.subtile_row=0;s.subtile_column=2;
        assert!(!underground_native_floor("UndergroundPath",&s));
        s.metatile_id=12;s.subtile_column=0;assert!(!underground_native_floor("UndergroundPath",&s));
    }
    #[test]
    fn kanto_underground_weave_reaches_the_complete_terrain_builder() {
        use bevy::prelude::{Assets,Image,Vec2,UVec2};
        use crystal_render_api::VisualWorldFrame;
        let (mut tiles,g,_)=fixture();
        let mut images=Assets::<Image>::default();let texture=images.add(Image::default());
        for t in &mut tiles {t.texture=texture.clone();t.source.tileset_id="underground".into();
            t.source.metatile_id=13;t.source.tile_index=16;}
        let frame=VisualWorldFrame{active:true,map_id:"UndergroundPath".into(),map_texture:texture,
            tile_size:Vec2::splat(8.),viewport_size:Vec2::splat(16.),grid_size:UVec2::new(g.width as u32,g.height as u32),
            tiles,..Default::default()};
        let samples=TerrainImageSamples::default();
        for images in [None,Some(&samples)] {
        let mesh=super::super::build_terrain_mesh_internal(&frame,images,true,Some(built_in_profiles()),None,None).unwrap();
        assert_eq!(mesh.footing_heights,vec![0.;4]);
        assert_eq!(mesh.textured.indices.len(),48,"four native triangles for each of four real floor cells");
        for t in mesh.textured.indices.chunks_exact(3) {
            assert!(t.iter().all(|&i|mesh.textured.uvs[i as usize]==mesh.textured.uvs[t[0]as usize]),
                "the full builder must consume pixel-interpolated floor artwork");
        }
        }
    }
    #[test]
    fn kanto_clinic_north_service_aprons_keep_palette_datum_and_counter_boundary() {
        for (block,column,row) in [(2,2,2),(2,3,2),(3,0,2),(3,1,2),(3,2,2),(8,0,2),(8,1,2),
            (19,0,2),(19,1,2),(19,2,2),(19,3,2),(42,1,2),(42,2,2),(42,3,2),(12,2,0),(12,3,0)] {
            let (mut tiles,g,mut mesh)=fixture();
            for tile in &mut tiles {tile.source.tileset_id="pokecenter".into();
                tile.source.metatile_id=block;tile.source.tile_index=1;
                tile.source.subtile_column=column;tile.source.subtile_row=row;}
            let refs=tiles.iter().collect::<Vec<_>>();
            assert_eq!(finish_surfaces(&mut mesh,"IndigoPlateauPokecenter1F",&refs,&g,[-3,7]),4);
            assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
            assert!(mesh.solid.positions.is_empty());
            for (position,uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
                assert!(position[1]<=3.25&&position[1]>=3.202);
                let pixel=[(uv[0]*16.).rem_euclid(8.),(uv[1]*16.).rem_euclid(8.)];
                assert!((near(pixel[0],0.5)||near(pixel[0],2.5))&&near(pixel[1],0.5));
            }
            let reserved=vec![true;4];let (_,_,mut protected)=fixture();
            assert_eq!(finish_surfaces_excluding(&mut protected,"IndigoPlateauPokecenter1F",&refs,&g,[-3,7],&reserved),0);
            let mut changed=tiles[0].source.clone();changed.tile_index=15;
            assert_ne!(interior_floor("IndigoPlateauPokecenter1F",&changed),Some(InteriorFloor::NativeRoom));
            changed.tile_index=1;changed.subtile_row=1;
            assert_ne!(interior_floor("IndigoPlateauPokecenter1F",&changed),Some(InteriorFloor::NativeRoom));
            assert_ne!(interior_floor("UnreviewedPokecenter1F",&tiles[0].source),Some(InteriorFloor::NativeRoom));
        }
        let (mut tiles,_,_)=fixture();let s=&mut tiles[0].source;
        s.tileset_id="pokecenter".into();s.tile_index=1;s.subtile_row=2;
        for (block,column) in [(2,0),(2,1),(3,3),(8,2),(8,3),(42,0),(4,0)] {
            s.metatile_id=block;s.subtile_column=column;
            assert_ne!(interior_floor("IndigoPlateauPokecenter1F",s),Some(InteriorFloor::NativeRoom));
        }
    }
    #[test]
    fn kanto_native_room_backing_cannot_add_a_second_tall_cream_wall() {
        let (mut tiles,mut g,_)=fixture();g.width=16;g.height=4;
        let base=tiles.remove(0);
        tiles=(0..64).map(|i|{let mut t=base.clone();
            t.column=(i%16)as u32;t.row=(i/16)as u32;
            t.source.metatile_id=[0x04,0x1e,0x05,0x1d][i%16/4];
            t.source.subtile_column=(i%4)as u8;t.source.subtile_row=(i/16)as u8;t
        }).collect();
        let refs=tiles.iter().collect::<Vec<_>>();
        assert!(!ordinary_house::course_origins(&refs,&g).is_empty(),"old scaffold recognizes this block course");
        let mut mesh=TerrainMeshData::default();
        append_known_room_backing(&mut mesh,"BluesHouse",&refs,&g,[0,0],&[]);
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty(),
            "closed authored courses must not receive another wall from block metadata");
    }
    #[test]
    fn kanto_native_mansion_checker_keeps_live_colors_claims_and_walking_datum() {
        for map in ["CeladonMansion1F","CeladonMansion2F","CeladonMansion3F","CeladonMansionRoofHouse"] {
            let (mut tiles,g,mut mesh)=fixture();
            for tile in &mut tiles {tile.source.tileset_id="mansion".into();tile.source.tile_index=0x11;}
            let refs=tiles.iter().collect::<Vec<_>>();
            assert_eq!(finish_surfaces(&mut mesh,map,&refs,&g,[-3,7]),4);
            assert!(mesh.solid.positions.is_empty());
            assert_eq!(mesh.footing_heights,vec![3.25;4]);
            assert_eq!(mesh.authored_cells,vec![None;4]);
            assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25 && p[1]>=3.202));
            for uv in &mesh.textured.uvs {
                let pixel=[(uv[0]*16.).rem_euclid(8.),(uv[1]*16.).rem_euclid(8.)];
                assert!((near(pixel[0],0.5)||near(pixel[0],1.5)) && near(pixel[1],0.5),
                    "only the native light/pink source pixels may paint the checker");
            }
            let (mut tiles,g,mut legacy)=fixture();
            for tile in &mut tiles {tile.source.tileset_id="mansion".into();tile.source.tile_index=1;}
            let refs=tiles.iter().collect::<Vec<_>>();
            assert_eq!(finish_surfaces(&mut legacy,map,&refs,&g,[0,0]),4);
            assert!(legacy.textured.uvs.iter().any(|uv|!near((uv[1]*16.).rem_euclid(8.),0.5)),
                "the separate gold ornament drawing must retain its full live artwork");
        }
    }
    #[test]
    fn johto_player_bedroom_backing_requires_complete_art_and_keeps_stair_open() {
        let (sample,_,_)=fixture();
        let g=GridGeometry{width:16,height:12,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let drawings=[
            [2,2,2,2,2,2,2,2,1,1,1,1,1,1,1,1],
            [2,2,2,2,11,12,2,2,27,28,66,67,43,44,82,83],
            [2,2,5,6,59,60,21,22,75,76,37,38,91,92,53,54],
            [2,2,64,65,2,2,80,81,1,1,1,1,1,1,1,1],
        ];
        let mut tiles=(0..192).map(|i| {
            let mut tile=sample[0].clone();let x=i%16;let y=i/16;
            tile.column=x as u32;tile.row=y as u32;tile.source.tileset_id="players_room".into();
            tile.source.metatile_id=[4,1,3,2][x/4];tile.source.subtile_column=(x%4)as u8;
            tile.source.subtile_row=(y%4)as u8;
            tile.source.tile_index=if y<4 {drawings[x/4][y*4+x%4]}else{1};tile
        }).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();
        assert!(native_bedroom_backing_source("PlayersHouse2F",&refs,&g,0,0));
        let mut mesh=TerrainMeshData{footing_heights:vec![2.5;192],authored_cells:vec![None;192],..Default::default()};
        append_known_room_backing(&mut mesh,"PlayersHouse2F",&refs,&g,[0,0],&[]);
        assert!(mesh.solid.positions.is_empty());assert!(!mesh.textured.positions.is_empty());
        assert!(mesh.textured.positions.iter().all(|p|p[0]<=112.));
        assert_eq!(mesh.footing_heights,vec![2.5;192]);assert_eq!(mesh.authored_cells,vec![None;192]);
        for i in 0..64 {let old=tiles[i].source.tile_index;tiles[i].source.tile_index=0xffff;
            assert!(!native_bedroom_backing_source("PlayersHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,0,0));
            tiles[i].source.tile_index=old;
        }
        for y in 0..2 {for x in 12..14 {
            tiles[y*16+x].source.metatile_id=0x1f;
            tiles[y*16+x].source.tile_index=[0x44,0x45,0x54,0x55][y*2+x-12];
        }}
        for y in 0..4 {for x in 12..16 {tiles[y*16+x].source.metatile_id=0x1f;}}
        assert!(native_bedroom_backing_source("PlayersHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,0,0));
        // Production publishes a padded world grid, not a map-sized grid.
        let pg=GridGeometry{width:36,height:32,origin_x:-144.,origin_z:-128.,..g.clone()};
        let mut padded=vec![sample[0].clone();pg.width*pg.height];
        for tile in &mut padded {tile.source.tileset_id="outside_room".into();}
        for y in 0..12 {for x in 0..16 {padded[(y+6)*pg.width+x+8]=tiles[y*16+x].clone();}}
        let refs=padded.iter().collect::<Vec<_>>();
        assert!(native_bedroom_backing_source("PlayersHouse2F",&refs,&pg,8,6));
        let mut room=TerrainMeshData::default();
        append_known_room_backing(&mut room,"PlayersHouse2F",&refs,&pg,[-8,-6],&[]);
        assert!(!room.textured.positions.is_empty());
        assert!(room.textured.positions.iter().all(|p|p[0]<=32.));
        let mut excluded=vec![false;padded.len()];excluded[6*pg.width+8]=true;
        let mut custom=TerrainMeshData::default();
        append_known_room_backing(&mut custom,"PlayersHouse2F",&refs,&pg,[-8,-6],&excluded);
        assert!(custom.textured.positions.is_empty(),"custom course owner must not get another wall");
    }

    #[test]
    fn johto_home_backing_has_source_planks_without_changing_volume_or_footing() {
        let (_,g,mut mesh)=fixture();mesh.footing_heights=vec![2.5;4];
        append_native_home_backing(&mut mesh,&g,0,0,2,[0,0]);
        assert_eq!(mesh.footing_heights,vec![2.5;4]);
        assert_eq!(mesh.authored_cells,vec![None;4]);
        let uv=g.uv(0,0);let source=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
        for donor in [source(1.,0.),source(0.,0.),source(7.,0.),source(3.,2.)] {
            assert!(mesh.textured.uvs.contains(&donor));
        }
        let front=g.origin_z+1.8*g.tile_height;let mut area=0.;
        for tri in mesh.textured.indices.chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
            let cross=(points[1]-points[0]).cross(points[2]-points[0]);
            assert!(cross.dot(normal)>0.);
            if normal.z>0. && points.iter().all(|p|p.y<=4.*g.tile_height) {area+=cross.z*0.5;}
        }
        let expected=2.*g.tile_width*4.*g.tile_height;
        assert!((area-expected).abs()<0.01,"one joined front, no layered wall cards: area={area}, expected={expected}");
        assert!(mesh.textured.positions.iter().any(|p|p[2]<front && p[2]>front-g.tile_height*0.1));
        assert!(mesh.textured.positions.iter().all(|p|p[0]>=g.origin_x && p[0]<=g.origin_x+2.*g.tile_width));
    }

    #[test]
    fn johto_all_first_floor_clinics_use_source_pink_with_unchanged_footing() {
        for map in ["AzaleaPokecenter1F","BlackthornPokecenter1F","CherrygrovePokecenter1F","CianwoodPokecenter1F","EcruteakPokecenter1F","GoldenrodPokecenter1F","MahoganyPokecenter1F","OlivinePokecenter1F","Route32Pokecenter1F","SilverCavePokecenter1F","VioletPokecenter1F"] {
            let (mut tiles,g,mut mesh)=fixture();
            for tile in &mut tiles {tile.source.tileset_id="pokecenter".into();
                tile.source.metatile_id=4;tile.source.tile_index=0x11;}
            assert_eq!(finish_surfaces(&mut mesh,map,&tiles.iter().collect::<Vec<_>>(),&g,[0,0]),4);
            assert_eq!(mesh.footing_heights,vec![3.25;4]);
            assert_eq!(mesh.authored_cells,vec![None;4]);
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25 && p[1]>=3.202));
        }
        assert!(!kanto_native_floor_map("UnreviewedPokecenter1F"));
        assert!(!kanto_native_floor_map("CherrygrovePokecenter2FBeta"));
    }

    #[test]
    fn johto_standard_mart_floor_field_does_not_invent_large_checker_squares() {
        let uv=(0.,1.,0.,1.);
        let field=[0.5/8.,0.5/8.];
        for world in [[0,0],[2,0],[0,2],[2,2]] {
            let mut mesh=SurfaceMeshData::default();
            native_mart_material_floor_cell(&mut mesh,[0.,8.,0.,8.],0.,world,uv);
            let mut area=0.;
            for tri in mesh.indices.chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let a=(points[1]-points[0]).cross(points[2]-points[0]).length()*0.5;
                if a>5. {
                    assert!(tri.iter().all(|i|mesh.uvs[*i as usize]==field));
                    area+=a;
                }
            }
            assert!(area>60.,"continuous field covers the floor, retaining narrow seams");
        }
    }

    #[test]
    fn johto_standard_mart_floors_use_green_source_swatches_without_footing_changes() {
        for map in ["AzaleaMart","BlackthornMart","CherrygroveMart","EcruteakMart","OlivineMart","VioletMart"] {
        let (mut tiles,g,mut mesh)=fixture();
        for tile in &mut tiles {tile.source.tileset_id="mart".into();tile.source.tile_index=crate::mart::STANDARD_MART_FLOOR_TILE;}
        assert_eq!(finish_surfaces(&mut mesh,map,&tiles.iter().collect::<Vec<_>>(),&g,[0,0]),4);
        assert!(mesh.solid.positions.is_empty());assert_eq!(mesh.footing_heights,vec![3.25;4]);
        for uv in &mesh.textured.uvs {
            let x=(uv[0]*16.).rem_euclid(8.);let y=(uv[1]*16.).rem_euclid(8.);
            assert!((near(x,0.5)&&near(y,0.5))||(near(x,3.5)&&near(y,3.5)));
        }
        }
        assert!(!kanto_native_floor_map("UnreviewedJohtoMart"));
    }

    #[test]
    fn johto_player_home_floor_uses_live_wood_without_changing_footing() {
        for (map,atlas) in [("PlayersHouse1F","players_house"),("PlayersHouse2F","players_room")] {
            let (mut tiles,g,mut mesh)=fixture();
            for tile in &mut tiles {tile.source.tileset_id=atlas.into();tile.source.tile_index=1;}
            assert_eq!(finish_surfaces(&mut mesh,map,&tiles.iter().collect::<Vec<_>>(),&g,[0,0]),4);
            assert!(mesh.solid.positions.is_empty());
            assert_eq!(mesh.footing_heights,vec![3.25;4]);
            assert_eq!(mesh.authored_cells,vec![None;4]);
            for uv in &mesh.textured.uvs {
                assert!(near((uv[0]*16.).rem_euclid(8.),0.5));
                assert!(near((uv[1]*16.).rem_euclid(8.),0.5));
            }
        }
    }

    #[test]
    fn kanto_native_room_floor_keeps_atlas_palette_datum_and_original_claims() {
        let (tiles,g,mut mesh)=fixture();let refs=tiles.iter().collect::<Vec<_>>();
        assert_eq!(finish_surfaces(&mut mesh,"BluesHouse",&refs,&g,[-3,7]),4);
        assert!(mesh.solid.indices.is_empty(),"no beige replacement paint");
        assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
        assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25 && p[1]>=3.202));
        for uv in &mesh.textured.uvs {
            let pixel_x=(uv[0]*16.).rem_euclid(8.);
            let pixel_y=(uv[1]*16.).rem_euclid(8.);
            assert!(near(pixel_x,0.5)||near(pixel_x,1.5));assert!(near(pixel_y,0.5));
        }
    }
    #[test]
    fn kanto_native_service_floor_retains_live_sources_without_spilling_into_other_rooms() {
        for (map,atlas,tile) in [
            ("PewterPokecenter1F","pokecenter",0x11),
            ("PewterPokecenter2FBeta","pokecenter",0x11),
            ("PowerPlant","facility",0x01),
            ("LavRadioTower1F","radio_tower",0x01),
            ("SaffronMagnetTrainStation","train_station",0x3d),
        ] {
            let (mut tiles,g,mut mesh)=fixture();
            for cell in &mut tiles {cell.source.tileset_id=atlas.into();cell.source.tile_index=tile;}
            let refs=tiles.iter().collect::<Vec<_>>();
            assert_eq!(finish_surfaces(&mut mesh,map,&refs,&g,[0,0]),4,"{map}");
            assert!(mesh.solid.positions.is_empty(),"source palette must not acquire generic paint");
            assert_eq!(mesh.footing_heights,vec![3.25;4]);
            for (p,uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
                if atlas=="pokecenter" {
                    let pixel=[(uv[0]*16.).rem_euclid(8.),(uv[1]*16.).rem_euclid(8.)];
                    assert!((near(pixel[0],0.5)||near(pixel[0],2.5)) && near(pixel[1],0.5),
                        "live tile11 field/joint only");
                } else if map=="SaffronMagnetTrainStation" {
                    let pixel=[(uv[0]*16.).rem_euclid(8.),(uv[1]*16.).rem_euclid(8.)];
                    assert!((near(pixel[0],0.5)&&near(pixel[1],0.5))
                        ||(near(pixel[0],2.5)&&near(pixel[1],2.5)),"live tile3d field/grout only");
                }else{assert!(near(uv[0],p[0]/16.) && near(uv[1],p[2]/16.));}
            }
            assert_ne!(interior_floor("UnreviewedServiceRoom",&tiles[0].source),Some(InteriorFloor::NativeRoom));
            tiles[0].source.subtile_row=4;
            assert_eq!(interior_floor(map,&tiles[0].source),None);
        }
        assert!(!kanto_native_floor_map("UnreviewedPokecenter1F"));
        assert!(!kanto_native_floor_map("GoldenrodMagnetTrainStation"));
        assert!(!kanto_native_floor_map("CeladonDeptStore1F"));
        assert!(!kanto_native_floor_map("PewterMartAnnex"));
    }
    #[test]
    fn cafe_ceramic_finish_preserves_footing_and_rejects_unproven_quadrants() {
        let (mut tiles, g, mut mesh) = fixture();
        for (cell, tile) in tiles.iter_mut().zip([0x01, 0x10, 0x11, 0x12]) {
            cell.source.tileset_id = "game_corner".into();
            cell.source.metatile_id = 1;
            cell.source.tile_index = tile;
            assert_eq!(interior_floor("CeladonCafe", &cell.source), Some(InteriorFloor::NativeRoom));
        }
        let original_tiles = tiles.clone();
        let refs = tiles.iter().collect::<Vec<_>>();
        assert_eq!(finish_surfaces(&mut mesh, "CeladonCafe", &refs, &g, [0, 0]), 4);
        assert_eq!(tiles, original_tiles);
        assert_eq!(mesh.footing_heights, vec![3.25; 4]);
        assert_eq!(mesh.authored_cells, vec![None; 4]);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.iter().all(|p| p[1] <= 3.25 && p[1] >= 3.202));
        for cell in &mut tiles {
            cell.source.tile_index ^= 0x80;
            assert_eq!(interior_floor("CeladonCafe", &cell.source), None);
        }
    }
    #[test]
    fn johto_gate_floor_has_one_closed_diamond_per_source_cell_at_every_world_phase() {
        for world in [[0,0],[1,0],[2,1],[-3,7]] {
            let mut mesh=SurfaceMeshData::default();
            native_gate_material_floor_cell(&mut mesh,[0.,8.,0.,8.],3.25,world,(0.,1.,0.,1.));
            let mut areas=[0.;2];
            for tri in mesh.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let area=(p[1]-p[0]).cross(p[2]-p[0]).y*0.5;
                assert!(area>0.);assert!(p.iter().all(|p|p.y==3.25));
                let uv=mesh.uvs[tri[0]as usize];
                assert!(tri.iter().all(|i|mesh.uvs[*i as usize]==uv));
                let role=if uv==[0.5/8.,3.5/8.] {0}else{assert_eq!(uv,[0.5/8.,0.5/8.]);1};
                areas[role]+=area;
            }
            assert_eq!(areas,[32.,32.]);
            assert!(mesh.positions.contains(&[4.,3.25,4.]));
            assert_eq!(mesh.indices.len(),12);
            for edge in 0..4 {
                let expected=if edge%2==0 {[0.5/8.,0.5/8.]}else{[0.5/8.,3.5/8.]};
                assert_eq!(mesh.uvs[edge*3],expected,"north/south cream, east/west gray follows source phase");
            }
        }
    }

    #[test]
    fn johto_reviewed_gate_floors_keep_native_source_and_authoritative_footing() {
        for map in crate::mesh::gate_counters::JOHTO_GATE_ROOMS {
            let (mut tiles,g,mut mesh)=fixture();
            for tile in &mut tiles {tile.source.tileset_id="gate".into();tile.source.tile_index=1;}
            let before=tiles.clone();
            assert_eq!(finish_surfaces(&mut mesh,map,&tiles.iter().collect::<Vec<_>>(),&g,[-3,7]),4);
            assert_eq!(tiles,before);assert_eq!(mesh.footing_heights,vec![3.25;4]);
            assert_eq!(mesh.authored_cells,vec![None;4]);
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.iter().all(|p|p[1]<=3.25 && p[1]>=3.202));
        }
        assert!(!kanto_native_floor_map("UnreviewedJohtoGate"));
        assert!(!kanto_native_floor_map("Route31VioletGateAnnex"));
    }

    #[test]
    fn kanto_ceramic_finishes_keep_source_and_footing_without_spilling_into_johto() {
        for (map, atlas, johto, tile) in [("Route5SaffronGate", "gate", "UnreviewedJohtoGate", 1),
            ("PewterMart", "mart", "GoldenrodMart", crate::mart::STANDARD_MART_FLOOR_TILE),
            ("SilphCo1F", "facility", "RuinsOfAlphResearchCenter", 0x01),
            ("SilphCo1F", "facility", "RuinsOfAlphResearchCenter", 0x26)] {
            let (mut tiles, g, mut mesh) = fixture();
            for cell in &mut tiles {
                cell.source.tileset_id = atlas.into();
                cell.source.tile_index = tile;
            }
            let original_tiles = tiles.clone();
            let refs = tiles.iter().collect::<Vec<_>>();
            assert_eq!(finish_surfaces(&mut mesh, map, &refs, &g, [-3, 7]), 4);
            assert_eq!(tiles, original_tiles);
            assert_eq!(mesh.footing_heights, vec![3.25; 4]);
            assert_eq!(mesh.authored_cells, vec![None; 4]);
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.iter().all(|p| p[1] <= 3.25 && p[1] >= 3.202));
            assert_ne!(interior_floor(johto, &tiles[0].source), Some(InteriorFloor::NativeRoom));
            tiles[0].source.tile_index = 0x81;
            assert_eq!(interior_floor(map, &tiles[0].source), None);
        }
    }
    #[test]
    fn kanto_trainer_basement_floor_uses_source_fields_without_pixel_dither() {
        for tile in [1,38] {
            let (mut tiles,g,mut mesh)=fixture();
            for t in &mut tiles {t.source.tileset_id="facility".into();t.source.metatile_id=48;t.source.tile_index=tile;}
            let refs=tiles.iter().collect::<Vec<_>>();
            assert_eq!(finish_surfaces(&mut mesh,"TrainerHouseB1F",&refs,&g,[0,0]),4);
            assert_eq!(mesh.footing_heights,vec![3.25;4]);assert_eq!(mesh.authored_cells,vec![None;4]);
            for (p,uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
                assert!(p[1]<=3.25&&p[1]>=3.202);
                let px=(uv[0]*16.).rem_euclid(8.);let py=(uv[1]*16.).rem_euclid(8.);
                assert!(near(px,0.5)||(tile==1&&near(px,1.5)));
                assert!(near(py,0.5),"each finish samples a source color rather than stretching its dither");
            }
            let (_,_,mut protected)=fixture();
            assert_eq!(finish_surfaces_excluding(&mut protected,"TrainerHouseB1F",&refs,&g,[0,0],&[true;4]),0);
        }
    }
    #[test]
    fn kanto_department_pavers_cover_every_phase_without_raising_the_floor() {
        for row in -3..3 {
            for column in -3..3 {
                let mut mesh = SurfaceMeshData::default();
                native_deptstore_material_floor_cell(&mut mesh, [0., 8., 0., 8.],
                    3.25, [column, row], (0.2, 0.4, 0.3, 0.7));
                let area: f32 = mesh.indices.chunks_exact(3).map(|ids| {
                    let [a, b, c] = [ids[0], ids[1], ids[2]]
                        .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                    (b - a).cross(c - a).length() * 0.5
                }).sum();
                assert!((area - 64.).abs() < 0.001);
                assert!(mesh.positions.iter().all(|p|
                    p[1] == 3.25 && (0.0..=8.0).contains(&p[0]) && (0.0..=8.0).contains(&p[2])));
                assert!(mesh.uvs.iter().all(|uv|
                    (0.2..0.4).contains(&uv[0]) && (0.3..0.7).contains(&uv[1])));
            }
        }
    }
    #[test]
    fn kanto_native_room_floor_yields_to_custom_source_profiles_and_changed_art() {
        let (mut tiles,g,mut mesh)=fixture();let refs=tiles.iter().collect::<Vec<_>>();
        let custom=Document{objects:vec![crate::live_profiles::Object{
            name:"User pink floor".into(),tileset:"house".into(),map:Some("BluesHouse".into()),
            maps:None,metatile:0,metatiles:None,origin:[0,0],tiles:vec![vec![1]],ground:1,
            top_pixels:0,depth_pixels:0.,footing_pixels:None,mask:Default::default(),parts:vec![],
        }],atmosphere:None};
        assert_eq!(finish_surfaces_with_profiles(&mut mesh,"BluesHouse",&refs,&g,[0,0],Some(&custom)),3);
        assert_eq!(mesh.textured.indices.len(),6+3*48);
        tiles[0].source.subtile_column=4;
        assert_eq!(interior_floor("BluesHouse",&tiles[0].source),None);
        tiles[0].source.subtile_column=0;tiles[0].source.tile_index=0x24;
        assert_ne!(interior_floor("BluesHouse",&tiles[0].source),Some(InteriorFloor::NativeRoom));
        tiles[0].source.tile_index=1;
        assert_eq!(interior_floor("NewBarkTown",&tiles[0].source),Some(InteriorFloor::Oak));
        assert_eq!(interior_floor("BluesHouseAnnex",&tiles[0].source),Some(InteriorFloor::Oak));
    }
}
