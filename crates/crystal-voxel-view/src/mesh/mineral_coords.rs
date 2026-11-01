//! Recover terrain coordinates independently of a connected map's paint basis.
use super::GridGeometry;
use bevy::prelude::Vec2;
use crystal_render_api::VisualSourceRegion;

pub(super) fn point(position:[f32;3],uv:[f32;2],g:&GridGeometry,origin:[i32;2],
    regions:&[VisualSourceRegion])->Vec2 {
    let origin=Vec2::new(origin[0] as f32,origin[1] as f32);
    let tile=Vec2::new(g.tile_width,g.tile_height);
    let placed=(Vec2::new(position[0],position[2])-Vec2::new(g.origin_x,g.origin_z))/tile+origin;
    let mut best=Vec2::ZERO;
    let mut distance=f32::INFINITY;
    for tag in [20000.,40000.,60000.] {
        let painted=Vec2::new(-10000.-uv[0],-tag-uv[1])/tile;
        // Some producers already register in the active map; native paint
        // on connected scenery instead uses its owning map's coordinates.
        for shift in std::iter::once(Vec2::ZERO).chain(regions.iter()
            .filter(|r|r.origin!=bevy::prelude::IVec2::ZERO).map(|r|r.origin.as_vec2())) {
            let candidate=painted+shift;
            let error=candidate.distance_squared(placed);
            if error<distance {best=candidate;distance=error;}
        }
    }
    best-origin
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2,UVec2};

    #[test]
    fn connected_and_active_paint_recover_the_same_shared_terrain_point() {
        for size in [8.,32.,64.] {
            let g=GridGeometry {width:116,height:114,tile_width:size,tile_height:size,
                origin_x:-58.*size,origin_z:-57.*size};
            let origin=[-38,-17];
            let world=Vec2::new(72.,40.);
            let local=world-Vec2::new(-38.,-17.);
            let original=Vec2::new(g.origin_x,g.origin_z)+local*size;
            let regions=[VisualSourceRegion {map_id:"neighbor".into(),
                origin:IVec2::new(60,-32),size:UVec2::new(40,80)}];
            for tag in [20000.,40000.,60000.] {for erosion in [Vec2::ZERO,Vec2::new(-0.7,0.4)*size] {
                let p=[original.x+erosion.x,16.,original.y+erosion.y];
                let active=world*size;
                let connected=(world-regions[0].origin.as_vec2())*size;
                let a=point(p,[-10000.-active.x,-tag-active.y],&g,origin,&regions);
                let b=point(p,[-10000.-connected.x,-tag-connected.y],&g,origin,&regions);
                assert!(a.distance(local)<0.001 && b.distance(local)<0.001,
                    "paint basis cannot separate a joined cap/wall sample: {a:?} {b:?}");
            }}
        }
    }

    #[test]
    fn material_tag_boundaries_do_not_move_distant_terrain() {
        let g=GridGeometry {width:12,height:12,tile_width:8.,tile_height:8.,
            origin_x:0.,origin_z:0.};
        for origin in [[0,1800],[0,-1800]] {for tag in [20000.,40000.,60000.] {
            let source=Vec2::new(6.5,7.25);
            let world=(source+Vec2::new(origin[0] as f32,origin[1] as f32))*8.;
            let result=point([source.x*8.,16.,source.y*8.],
                [-10000.-world.x,-tag-world.y],&g,origin,&[]);
            assert!(result.distance(source)<0.001);
        }}
    }
}
