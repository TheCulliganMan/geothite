use bevy::prelude::*;
use crystal_render_api::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Binary float bits make signed zero and every finite source value exact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Frame {
    active: bool,
    map: String,
    source_size: Option<[u32; 2]>,
    boundary_warps: Vec<[i32; 2]>,
    ledges: Vec<[[i32; 2]; 3]>,
    datum: Option<([i32;2],[u32;2],super::blobs::Bulk<[u32;4]>,super::blobs::Bulk<[i32;2]>)>,
    #[serde(default)]
    open_water: Option<(u32,super::blobs::Bulk<[i32;2]>)>,
    revision: u64,
    origin: [i32; 2],
    atlas: u32,
    center: [u32; 2],
    #[serde(default)]
    view_center: [u32; 2],
    viewport: [u32; 2],
    tile_size: [u32; 2],
    grid_size: [u32; 2],
    tiles: super::records::Records<Tile>,
    actors: Vec<Actor>,
    texture_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Tile {
    position: [u32; 2],
    source: Source,
    texture: u32,
    family: Option<Vec<u32>>,
    priority: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Source {
    tileset: String,
    metatile: u16,
    subtile: [u8; 2],
    index: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ActorId {
    Player,
    Follower,
    Mount,
    Remote(u64),
    Object(u32),
    Grass,
    Ball(u8),
    Lamp(u8),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Actor {
    id: ActorId,
    source: String,
    palette: Option<[[[u8; 3]; 4]; 2]>,
    texture: u32,
    center: [u32; 2],
    size: [u32; 2],
    flip_x: bool,
    above_priority: bool,
    facing: Option<[u32; 2]>,
}

fn bits(v: Vec2) -> [u32; 2] {
    v.to_array().map(f32::to_bits)
}
fn vec2(v: [u32; 2]) -> Vec2 {
    Vec2::from_array(v.map(f32::from_bits))
}

/// Main-thread binding table. Only dense ordinals appear on the wire.
#[derive(Clone, Default)]
pub(crate) struct TextureBindings {
    handles: Vec<Handle<Image>>,
    slots: std::collections::HashMap<bevy::asset::AssetId<Image>, u32>,
}

impl TextureBindings {
    pub(crate) fn len(&self) -> u32 {
        self.handles.len() as u32
    }
    pub(crate) fn slot(&mut self, handle: &Handle<Image>) -> u32 {
        if let Some(slot) = self.slots.get(&handle.id()) {
            return *slot;
        }
        let slot = self.handles.len() as u32;
        self.handles.push(handle.clone());
        self.slots.insert(handle.id(), slot);
        slot
    }

    pub(crate) fn handle(&self, slot: u32) -> Result<Handle<Image>, String> {
        self.handles
            .get(slot as usize)
            .cloned()
            .ok_or_else(|| "invalid texture slot".into())
    }

    pub(crate) fn worker(count: u32) -> Result<Self, String> {
        if count > 262_144 {
            return Err("too many terrain texture slots".into());
        }
        let mut bindings = Self::default();
        for i in 0..count {
            // This namespace is local to the worker, never a main asset ID.
            bindings.slot(&Handle::Uuid(
                bevy::asset::uuid::Uuid::from_u128(
                    0x7465727261696e000000000000000000_u128 + u128::from(i) + 1,
                ),
                std::marker::PhantomData,
            ));
        }
        Ok(bindings)
    }
}

impl Frame {
    pub(crate) fn texture_count(&self) -> u32 {
        self.texture_count
    }
    pub(crate) fn capture(frame: &VisualWorldFrame) -> (Self, TextureBindings) {
        let mut bindings = TextureBindings::default();
        let wire = Self::capture_with_bindings(frame, &mut bindings);
        (wire, bindings)
    }

    pub(crate) fn capture_with_bindings(
        frame: &VisualWorldFrame,
        bindings: &mut TextureBindings,
    ) -> Self {
        let atlas = bindings.slot(&frame.map_texture);
        let tiles = frame
            .tiles
            .iter()
            .map(|tile| Tile {
                position: [tile.column, tile.row],
                source: Source {
                    tileset: tile.source.tileset_id.to_string(),
                    metatile: tile.source.metatile_id,
                    subtile: [tile.source.subtile_column, tile.source.subtile_row],
                    index: tile.source.tile_index,
                },
                texture: bindings.slot(&tile.texture),
                family: tile
                    .animation_frames
                    .as_ref()
                    .map(|family| family.iter().map(|h| bindings.slot(h)).collect()),
                priority: tile.priority,
            })
            .collect();
        let actors = frame
            .actors
            .iter()
            .map(|actor| Actor {
                id: match actor.id {
                    VisualActorId::Player => ActorId::Player,
                    VisualActorId::PartyFollower => ActorId::Follower,
                    VisualActorId::SurfMount => ActorId::Mount,
                    VisualActorId::RemotePlayer(id) => ActorId::Remote(id),
                    VisualActorId::Object(id) => ActorId::Object(id),
                    VisualActorId::Effect(VisualEffectId::GrassRustle) => ActorId::Grass,
                    VisualActorId::Effect(VisualEffectId::HealMachineBall(id)) => ActorId::Ball(id),
                    VisualActorId::Effect(VisualEffectId::HealMachineLamp(id)) => ActorId::Lamp(id),
                },
                source: actor.source_id.to_string(),
                palette: actor
                    .model_palette
                    .as_ref()
                    .map(|p| [p.normal, p.appearance]),
                texture: bindings.slot(&actor.texture),
                center: bits(actor.center),
                size: bits(actor.size),
                flip_x: actor.flip_x,
                above_priority: actor.above_priority,
                facing: actor.facing.map(bits),
            })
            .collect();
        Self {
            active: frame.active,
            map: frame.map_id.to_string(),
            source_size: frame.source_map_size_core_tiles.map(|v| v.to_array()),
            datum: frame.terrace_datum.as_ref().map(|d|(d.origin.to_array(),d.size.to_array(),d.levels.iter().map(|v|v.map(f32::to_bits)).collect(),d.water_barriers.iter().map(|p|p.to_array()).collect())),
            open_water:frame.terrace_datum.as_ref().and_then(|d|d.open_water.as_ref())
                .map(|r|(r.level.to_bits(),r.cells.iter().map(|p|p.to_array()).collect())),
            boundary_warps: frame.boundary_warps.iter().map(|v| v.to_array()).collect(),
            ledges: frame
                .jump_ledges
                .iter()
                .map(|l| [l.start.to_array(), l.end.to_array(), l.descent.to_array()])
                .collect(),
            revision: frame.terrain_revision,
            origin: frame.grid_origin.to_array(),
            atlas,
            center: bits(frame.center),
            view_center: bits(frame.view_center),
            viewport: bits(frame.viewport_size),
            tile_size: bits(frame.tile_size),
            grid_size: frame.grid_size.to_array(),
            tiles,
            actors,
            texture_count: bindings.handles.len() as u32,
        }
    }

    pub(crate) fn resolve(
        &mut self,
        blobs: &mut super::blobs::BlobStore,
        deadline: bevy::platform::time::Instant,
        max_elements: usize,
    ) -> Result<bool, String> {
        if let Some((_,size,levels,water))=&self.datum {
            let n=u64::from(size[0])*u64::from(size[1]);
            if n==0 || n>2_000_000 || levels.len()as u64!=n || water.len()as u64>n {
                return Err("invalid source-domain buffer lengths".into());
            }
        }
        let datum_ready=if let Some((_,_,levels,water))=self.datum.as_mut() {
            let levels_ready=levels.resolve(blobs)?;
            let water_ready=water.resolve(blobs)?;
            levels_ready && water_ready
        }else{true};
        let river_ready=if let Some((_,cells))=&mut self.open_water {
            if cells.len()>2_000_000 {return Err("invalid river source-domain length".into());}
            cells.resolve(blobs)?
        }else{true};
        Ok(self.tiles.resolve(blobs, deadline, max_elements)? && datum_ready && river_ready)
    }

    pub(crate) fn restore(&self, bindings: &TextureBindings) -> Result<VisualWorldFrame, String> {
        if let Some((_,size,levels,water))=&self.datum {
            let values=levels.as_slice()?;
            if size[0]==0 || size[1]==0 || u64::from(size[0])*u64::from(size[1])>2_000_000
                || values.len()as u64!=u64::from(size[0])*u64::from(size[1])
                || water.len()as u64>u64::from(size[0])*u64::from(size[1])
                || values.iter().flatten().any(|&bits|{let v=f32::from_bits(bits);!v.is_finite()||v<0.}) {
                return Err("invalid source-complete terrace datum".into());
            }
        }
        if self.open_water.is_some() && self.datum.is_none() {return Err("river requires complete terrace source bounds".into());}
        if self.tiles.len() > 262_144 || self.actors.len() > 65_536 || self.ledges.len() > 65_536
            || self.boundary_warps.len() > 65_536 {
            return Err("terrain frame exceeds element limits".into());
        }
        let frame = VisualWorldFrame {
            active: self.active,
            map_id: Arc::from(self.map.as_str()),
            source_map_size_core_tiles: self.source_size.map(UVec2::from_array),
            terrace_datum: self.datum.as_ref().map(|(origin,size,levels,water)|->Result<_,String>{
                let datum=VisualTerraceDatum {
                    origin:IVec2::from_array(*origin),size:UVec2::from_array(*size),
                    levels:levels.as_slice()?.iter().map(|v|v.map(f32::from_bits)).collect(),
                    open_water:self.open_water.as_ref().map(|(level,cells)|->Result<_,String>{
                        Ok(VisualOpenWaterDatum {level:f32::from_bits(*level),
                            cells:cells.as_slice()?.iter().copied().map(IVec2::from_array).collect()})
                    }).transpose()?,water_barriers:water.as_slice()?.iter().copied().map(IVec2::from_array).collect(),
                };
                if !datum.valid_water_barriers() || !datum.valid_open_water() {return Err("invalid source-complete water barrier domain".into());}
                Ok(Arc::new(datum))
            }).transpose()?,
            boundary_warps: self.boundary_warps.iter().copied().map(IVec2::from_array).collect(),
            jump_ledges: self
                .ledges
                .iter()
                .map(|l| VisualJumpLedge {
                    start: IVec2::from_array(l[0]),
                    end: IVec2::from_array(l[1]),
                    descent: IVec2::from_array(l[2]),
                })
                .collect(),
            terrain_revision: self.revision,
            grid_origin: IVec2::from_array(self.origin),
            map_texture: bindings.handle(self.atlas)?,
            center: vec2(self.center),
            view_center: vec2(self.view_center),
            viewport_size: vec2(self.viewport),
            tile_size: vec2(self.tile_size),
            grid_size: UVec2::from_array(self.grid_size),
            tiles: self
                .tiles
                .as_slice()?
                .iter()
                .map(|tile| {
                    Ok(VisualTile {
                        column: tile.position[0],
                        row: tile.position[1],
                        source: VisualTileSource {
                            tileset_id: Arc::from(tile.source.tileset.as_str()),
                            metatile_id: tile.source.metatile,
                            subtile_column: tile.source.subtile[0],
                            subtile_row: tile.source.subtile[1],
                            tile_index: tile.source.index,
                        },
                        texture: bindings.handle(tile.texture)?,
                        animation_frames: tile
                            .family
                            .as_ref()
                            .map(|f| {
                                f.iter()
                                    .map(|&slot| bindings.handle(slot))
                                    .collect::<Result<Arc<[_]>, String>>()
                            })
                            .transpose()?,
                        priority: tile.priority,
                    })
                })
                .collect::<Result<_, String>>()?,
            actors: self
                .actors
                .iter()
                .map(|actor| {
                    Ok(VisualActor {
                        id: match actor.id {
                            ActorId::Player => VisualActorId::Player,
                            ActorId::Follower => VisualActorId::PartyFollower,
                            ActorId::Mount => VisualActorId::SurfMount,
                            ActorId::Remote(id) => VisualActorId::RemotePlayer(id),
                            ActorId::Object(id) => VisualActorId::Object(id),
                            ActorId::Grass => VisualActorId::Effect(VisualEffectId::GrassRustle),
                            ActorId::Ball(id) => {
                                VisualActorId::Effect(VisualEffectId::HealMachineBall(id))
                            }
                            ActorId::Lamp(id) => {
                                VisualActorId::Effect(VisualEffectId::HealMachineLamp(id))
                            }
                        },
                        source_id: Arc::from(actor.source.as_str()),
                        model_palette: actor.palette.map(|p| VisualPokemonModelPalette {
                            normal: p[0],
                            appearance: p[1],
                        }),
                        texture: bindings.handle(actor.texture)?,
                        center: vec2(actor.center),
                        size: vec2(actor.size),
                        flip_x: actor.flip_x,
                        above_priority: actor.above_priority,
                        facing: actor.facing.map(vec2),
                    })
                })
                .collect::<Result<_, String>>()?,
        };
        frame
            .validate()
            .map_err(|e| format!("invalid terrain frame: {e:?}"))?;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_complete_terrace_field_uses_bounded_pages_and_preserves_all_corners() {
        let datum=Arc::new(VisualTerraceDatum {
            open_water:Some(VisualOpenWaterDatum {level:f32::from_bits(0x3f123456),
                cells:(0..512*256).step_by(3).map(|i|IVec2::new(-40+i%512,-80+i/512)).collect()}),
            water_barriers:(0..512*256).step_by(2).map(|i|IVec2::new(-40+i%512,-80+i/512)).collect(),
            origin:IVec2::new(-40,-80),size:UVec2::new(512,256),
            levels:vec![[0.,f32::from_bits(0x3f123456),2.,3.];512*256].into()});
        let frame=VisualWorldFrame {terrace_datum:Some(datum.clone()),..Default::default()};
        let (wire,bindings)=Frame::capture(&frame);
        let (header,mut pages)=super::super::blobs::encode_header(29,&wire).unwrap();
        assert!(header.len()<4096,"the source-complete raster must not become a large metadata header");
        let (mut decoded,mut store):(Frame,_)=super::super::blobs::decode_header(29,&header).unwrap();
        assert!(decoded.restore(&bindings).is_err(),"incomplete raster pages cannot become terrain evidence");
        while !pages.finished() {if let Some(page)=pages.next_page().unwrap() {
            let ack=super::super::blobs::append_packet(&mut store,29,&page).unwrap();
            pages.acknowledge(29,ack.0,ack.1).unwrap();
        }}
        assert!(decoded.resolve(&mut store,bevy::platform::time::Instant::now()+std::time::Duration::from_secs(1),4096).unwrap());
        assert_eq!(decoded.restore(&bindings).unwrap().terrace_datum,Some(datum));
    }

    #[test]
    fn visual_frame_roundtrip_rebinds_assets_and_preserves_float_bits() {
        let bindings = TextureBindings::worker(3).unwrap();
        let a = bindings.handle(0).unwrap();
        let b = bindings.handle(1).unwrap();
        let mut frame = VisualWorldFrame {
            active: true,
            map_id: Arc::from("NewBarkTown"),
            grid_size: UVec2::ONE,
            tile_size: Vec2::splat(8.),
            viewport_size: Vec2::splat(8.),
            center: Vec2::new(-0., f32::from_bits(0x3f123456)),
            view_center: Vec2::new(f32::from_bits(0x41412345), -0.),
            map_texture: a.clone(),
            source_map_size_core_tiles: Some(UVec2::ONE),
            terrace_datum:Some(Arc::new(VisualTerraceDatum {open_water:None,water_barriers:vec![IVec2::new(-8,4)].into(),
                origin:IVec2::new(-8,4),size:UVec2::ONE,
                levels:vec![[0.,f32::from_bits(0x3f123456),1.,2.]].into(),
            })),
            boundary_warps: vec![IVec2::new(3, 7)].into(),
            jump_ledges: vec![VisualJumpLedge {
                start: IVec2::ZERO,
                end: IVec2::ONE,
                descent: IVec2::Y,
            }]
            .into(),
            tiles: vec![VisualTile {
                column: 0,
                row: 0,
                source: VisualTileSource {
                    tileset_id: Arc::from("johto"),
                    metatile_id: 17,
                    subtile_column: 0,
                    subtile_row: 1,
                    tile_index: 33,
                },
                texture: a.clone(),
                animation_frames: Some(vec![a.clone(), b.clone()].into()),
                priority: true,
            }],
            ..Default::default()
        };
        for (i, id) in [
            VisualActorId::Player,
            VisualActorId::PartyFollower,
            VisualActorId::SurfMount,
            VisualActorId::RemotePlayer(u64::MAX),
            VisualActorId::Object(u32::MAX),
            VisualActorId::Effect(VisualEffectId::GrassRustle),
            VisualActorId::Effect(VisualEffectId::HealMachineBall(2)),
            VisualActorId::Effect(VisualEffectId::HealMachineLamp(3)),
        ]
        .into_iter()
        .enumerate()
        {
            frame.actors.push(VisualActor {
                id,
                source_id: Arc::from("player"),
                model_palette: Some(VisualPokemonModelPalette {
                    normal: [[1, 2, 3]; 4],
                    appearance: [[4, 5, 6]; 4],
                }),
                texture: b.clone(),
                center: Vec2::new(i as f32, -0.),
                size: Vec2::splat(16.),
                flip_x: true,
                above_priority: true,
                facing: Some(Vec2::NEG_Y),
            });
        }
        let (wire, original) = Frame::capture(&frame);
        assert_eq!(wire.texture_count, 2);
        let bytes = super::super::encode(&wire).unwrap();
        let decoded: Frame = super::super::decode(&bytes).unwrap();
        assert_eq!(wire, decoded);
        let (header, mut pages) = super::super::blobs::encode_header(17, &wire).unwrap();
        let (mut paged, mut store): (Frame, _) =
            super::super::blobs::decode_header(17, &header).unwrap();
        while !pages.finished() {
            if let Some(page) = pages.next_page().unwrap() {
                let ack = super::super::blobs::append_packet(&mut store, 17, &page).unwrap();
                pages.acknowledge(17, ack.0, ack.1).unwrap();
            }
        }
        assert!(
            paged
                .resolve(
                    &mut store,
                    bevy::platform::time::Instant::now() + std::time::Duration::from_secs(1),
                    100
                )
                .unwrap()
        );
        assert_eq!(wire, paged);
        assert!(store.all_consumed());
        assert_eq!(super::super::encode(&paged).unwrap(), bytes);
        assert_eq!(decoded.restore(&original).unwrap(), frame);
        let worker = TextureBindings::worker(decoded.texture_count).unwrap();
        let rebuilt = decoded.restore(&worker).unwrap();
        let (roundtrip, _) = Frame::capture(&rebuilt);
        assert_eq!(wire, roundtrip);
        assert_eq!(rebuilt.center.x.to_bits(), (-0.0f32).to_bits());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(super::super::decode::<Frame>(&trailing).is_err());
        let mut version = bytes;
        version[7] = b'0';
        assert!(super::super::decode::<Frame>(&version).is_err());
    }
}
