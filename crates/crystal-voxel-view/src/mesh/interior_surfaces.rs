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
// Exact Kanto residential rooms, selected from the pack's landmark region.
// Palette is read through the room atlas, never guessed from the map name.
fn kanto_native_room(map: &str) -> bool {
    matches!(map,
        "BillsHouse"
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
fn kanto_native_floor_map(map: &str) -> bool {
    kanto_native_room(map) || matches!(map,
        "CeladonCafe" |
        "CeladonPokecenter1F" | "CeladonPokecenter2FBeta"
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
        | "CeruleanMart" | "FuchsiaMart" | "LavenderMart" | "PewterMart"
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
    if map == "CeladonCafe" {
        return (cafe_floor_source(source)
            && matches!(shape_for_source_on_map(map, source),
                CellShape::Flat | CellShape::PlaneAt { height: 0. }))
            .then_some(NativeRoom);
    }
    if kanto_native_floor_map(map)
        && matches!((source.tileset_id.as_ref(),source.tile_index),
            ("house"|"players_house",0x01) | ("mansion",0x01|0x11)
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
            material_quad(mesh, [x0,x1,north,south], height, [0.75,0.78,0.84]);
            let joint = if (row * 2 + course) % 2 == 0 {x0} else {x0 + w * 0.5};
            material_quad(mesh,[x0,x1,north+seam,south-seam],height,palette);
            material_quad(mesh,[joint,joint+seam,north+seam,south-seam],height,[0.75,0.78,0.84]);
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
fn native_source_surface_patches(mesh:&mut SurfaceMeshData,xs:&[f32],zs:&[f32],
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

fn native_gate_material_floor_cell(
    mesh: &mut SurfaceMeshData, b: [f32; 4], height: f32,
    world: [i32; 2], uv: (f32, f32, f32, f32),
) {
    let (u0, u1, v0, v1) = uv;
    let sample = |pixel: [usize; 2]| [
        u0 + (u1 - u0) * (pixel[0] as f32 + 0.5) / 8.,
        v0 + (v1 - v0) * (pixel[1] as f32 + 0.5) / 8.,
    ];
    // Source tile01 contains cream and gray diamonds. Retain both pigments
    // on larger ceramic slabs so the walking lane is readable at an angle.
    let gray = sample([0, 3]);
    let cream = sample([0, 0]);
    let checker = (world[0].div_euclid(2) + world[1].div_euclid(2)).rem_euclid(2);
    native_source_ceramic_floor_cell(mesh, b, height, world,
        if checker == 0 { cream } else { gray }, gray);
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
    let checker = (world[0].div_euclid(2) + world[1].div_euclid(2)).rem_euclid(2);
    native_source_ceramic_floor_cell(mesh, b, height, world,
        if checker == 0 { light } else { dark }, dark);
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
        ([0., 0.475, 0., 0.475], yellow),
        ([0.475, 0.5, 0., 0.475], amber),
        ([0., 0.5, 0.475, 0.5], amber),
        ([0.5, 1., 0., 0.5], yellow),
        ([0., 0.5, 0.5, 1.], yellow),
        ([0.5, 1., 0.5, 0.525], amber),
        ([0.5, 0.525, 0.525, 1.], amber),
        ([0.525, 1., 0.525, 0.55], cream),
        ([0.525, 0.55, 0.55, 1.], cream),
        ([0.55, 1., 0.55, 1.], yellow),
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

fn native_house_material_floor_cell(mesh:&mut SurfaceMeshData,b:[f32;4],height:f32,
    world:[i32;2],uv:(f32,f32,f32,f32),wood:bool) {
    let [w,e,n,s]=b;let (u0,u1,v0,v1)=uv;
    assert!([w,e,n,s,height].iter().all(|v|v.is_finite()) && w<e && n<s,
        "native house material floor requires finite source bounds");
    let sample=|pixel:[usize;2]|[u0+(u1-u0)*(pixel[0]as f32+0.5)/8.,
        v0+(v1-v0)*(pixel[1]as f32+0.5)/8.];
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
    // Both swatches were checked against packed tile01. Wood uses the medium
    // gold; ceramic retains the original pink/light checker palette.
    let tex=|_:f32,_:f32|sample(if wood || checker==0 {[0,0]}else{[1,0]});
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
    let custom = (enhanced_room_floor_map(map) || store_floor_map(map) || kanto_native_floor_map(map)).then(|| Document {
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
    if source_styles.iter().all(Option::is_none) {
        return 0;
    }
    let original = std::mem::take(&mut mesh.textured.indices);
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
            if (enhanced_room_floor_map(map) || store_floor_map(map) || kanto_native_floor_map(map))
                && (excluded.get(destination) == Some(&true)
                    || excluded.get(source_index) == Some(&true))
            {
                return None;
            }
            if traditional_room::floor_map(map)
                && (cells[destination].source.metatile_id == 0x04
                    || cells[source_index].source.metatile_id == 0x04)
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
                    ("players_house",0x11) if kanto_native_room(map)
                        && source_styles.iter().any(|style|*style==Some(InteriorFloor::NativeRoom))
                        => Some(InteriorFloor::NativeRoom),
                    ("players_room", 0x02) | ("players_house", 0x11) => Some(InteriorFloor::Oak),
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
                          && cells[i].source.tileset_id==cells[source_index].source.tileset_id).then_some(i))
                        else {mesh.textured.indices.extend_from_slice(quad);continue;};
                    donor_index = donor;
                    g.uv(donor%g.width,donor/g.width)
                };
                let atlas=cells[source_index].source.tileset_id.as_ref();
                if map == "CeladonCafe" {
                    native_cafe_material_floor_cell(&mut mesh.textured, bounds, height,
                        world, native_uv, cells[donor_index].source.tile_index);
                } else if atlas == "gate" {
                    native_gate_material_floor_cell(&mut mesh.textured, bounds, height, world, native_uv);
                } else if atlas == "mart"
                    && cells[donor_index].source.tile_index == crate::mart::STANDARD_MART_FLOOR_TILE {
                    native_mart_material_floor_cell(&mut mesh.textured, bounds, height, world, native_uv);
                } else if map == "SilphCo1F" && atlas == "facility" {
                    native_silph_material_floor_cell(&mut mesh.textured, bounds, height,
                        world, native_uv, cells[donor_index].source.tile_index);
                } else if kanto_native_room(map) && matches!(atlas,"house"|"players_house") {
                    native_house_material_floor_cell(&mut mesh.textured,bounds,height,world,native_uv,atlas=="players_house");
                } else {native_beveled_floor_cell(&mut mesh.textured,bounds,height,world,native_uv);}

            } else if style == InteriorFloor::StoreCeramic && map.starts_with("CeladonDeptStore") {
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
    append_known_room_backing(mesh, map, cells, g, world_origin);
    finished
}
fn compact_surface(mesh: &mut SurfaceMeshData) {
    let mut rebuilt = SurfaceMeshData::default();
    let mut mapping = vec![u32::MAX; mesh.positions.len()];
    for &old in &mesh.indices {
        let old = old as usize;
        if mapping[old] == u32::MAX {
            mapping[old] = rebuilt.positions.len() as u32;
            rebuilt.positions.push(mesh.positions[old]);
            rebuilt.normals.push(mesh.normals[old]);
            rebuilt.uvs.push(mesh.uvs[old]);
            rebuilt.colors.push(mesh.colors[old]);
        }
        rebuilt.indices.push(mapping[old]);
    }
    *mesh = rebuilt;
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
fn append_known_room_backing(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
) {
    // These rooms have closed, source-owned wall/window/cabinet volumes.
    // The older whole-course scaffold adds a second, taller cream wall and
    // also accepts block metadata after its actual wall art has changed.
    if kanto_native_room(map) { return; }
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
        assert_eq!(mesh.textured.indices.len(),6);
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
        let mut mesh = TerrainMeshData::default();
        append_top(&mut mesh.textured, [0.0, 8.0, 0.0, 8.0], 3.5, g.uv(0, 0));
        mesh.footing_heights = vec![3.5];
        mesh.authored_cells = vec![None];
        assert_eq!(
            finish_surfaces(&mut mesh, "PlayersHouse2F", &refs, &g, [4, 6]),
            1
        );
        assert!(mesh.textured.indices.is_empty());
        assert_eq!(mesh.footing_heights, vec![3.5]);
        assert_eq!(mesh.authored_cells, vec![None]);
        assert!(mesh.solid.positions.iter().all(|p| near(p[1], 3.5)));
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
        append_known_room_backing(&mut mesh,"BluesHouse",&refs,&g,[0,0]);
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty(),
            "closed authored courses must not receive another wall from block metadata");
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
                assert!(near(uv[0],p[0]/16.) && near(uv[1],p[2]/16.));
            }
            assert_ne!(interior_floor("UnreviewedServiceRoom",&tiles[0].source),Some(InteriorFloor::NativeRoom));
            tiles[0].source.subtile_row=4;
            assert_eq!(interior_floor(map,&tiles[0].source),None);
        }
        assert!(!kanto_native_floor_map("GoldenrodPokecenter1F"));
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
    fn kanto_ceramic_finishes_keep_source_and_footing_without_spilling_into_johto() {
        for (map, atlas, johto, tile) in [("Route5SaffronGate", "gate", "Route35GoldenrodGate", 1),
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
