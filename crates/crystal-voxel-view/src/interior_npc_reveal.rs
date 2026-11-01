//! Source-backed indoor people reveal explicitly registered depth-aware objects.
use super::*;

pub(super) fn source_object(group: &OccluderGroup) -> bool {
    // Coverage is a rendering choice, not ownership. Authored smooth timber
    // and facility partitions opt into the same depth-aware range policy as
    // cabinets, with their source floor outside the owned body range.
    group.camera_wall.is_none() && group.depth_coverage_fade
        && group.fade.residual_opacity == RESIDUAL_OPACITY
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Person {
    pub(super) body: CutawayUniform,
    pub(super) rays: [Vec3;9],
}

impl Person {
    pub(super) fn obscured(&self, blocks: impl FnMut(Vec3)->bool) -> bool {
        // A counter naturally hides shoes. Reveal furniture for an NPC's
        // torso/head, preserving low counters when only legs are behind them.
        self.rays[3..].iter().copied().any(blocks)
    }
}

// One material aperture encloses every body blocked by this source owner,
// including the player when needed. Two staff sharing a bay
// must not make the selected aperture jump between them.
pub(super) fn merge(aperture: &mut Option<CutawayUniform>, other: CutawayUniform) {
    let Some(old)=*aperture else {*aperture=Some(other);return;};
    let a=old.bottom_radius;let b=other.bottom_radius;
    let middle=(a.truncate()+b.truncate())*0.5;
    let radius=a.w.max(b.w)+Vec2::new(a.x-b.x,a.z-b.z).length()*0.5;
    *aperture=Some(CutawayUniform {
        bottom_radius:Vec4::new(middle.x,a.y.min(b.y),middle.z,radius),
        top_feather:Vec4::new(middle.x,old.top_feather.y.max(other.top_feather.y),middle.z,
            old.top_feather.w.max(other.top_feather.w)),..Default::default()
    });
}

pub(super) fn targets(frame: &crystal_render_api::VisualWorldFrame, heights: &[f32], camera: Mat4) -> Vec<Person> {
    use crystal_render_api::VisualActorId;
    if frame.map_id.is_empty() || crate::cave_water::outdoor(&frame.map_id)
        || !frame.tile_size.is_finite() || frame.tile_size.min_element() <= 0. { return Vec::new(); }
    let mut people = frame.actors.iter().filter_map(|actor| {
        let VisualActorId::Object(id) = actor.id else { return None; };
        let height = crate::new_bark_actors::modeled_character_height(frame, actor)?;
        let ground = crate::footing::actor_ground_point(frame, actor);
        crate::footing::tile_at_visual_point(frame, ground)?;
        let support = crate::resolved_footing_height(frame, ground, heights)?;
        let foot = crate::visual_point_to_voxel(ground, support + 0.04);
        let mut body = CutawayUniform::for_player(foot, frame.tile_size.y * 2.)?;
        body.top_feather = (foot + Vec3::Y * height).extend(body.top_feather.w);
        // Include the lower torso above the legs; a chest-only sample can
        // graze over a cabinet while the character remains mostly hidden.
        let (_, rays) = body_rays(body, camera, [0.12,0.40,0.92])?;
        Some((id, Person {body,rays}))
    }).collect::<Vec<_>>();
    // Original maps have at most fifteen objects. A stable bound also keeps
    // custom crowds from multiplying blocker work without limit.
    people.sort_by_key(|(id, _)| *id);
    people.into_iter().take(16).map(|(_, person)| person).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn frame() -> crystal_render_api::VisualWorldFrame {
        use crystal_render_api::{VisualActor, VisualActorId};
        crystal_render_api::VisualWorldFrame { map_id:"PewterMart".into(),grid_size:UVec2::splat(16),
            tile_size:Vec2::splat(8.), viewport_size:Vec2::splat(128.),
            actors:vec![VisualActor {id:VisualActorId::Object(1),source_id:"clerk".into(),
                model_palette:None,texture:default(),center:Vec2::new(12.,-8.),size:Vec2::splat(16.),
                flip_x:false,above_priority:false,facing:Some(Vec2::X)}],..default() }
    }
    fn camera() -> Mat4 {
        Transform::from_xyz(-20.,60.,128.).looking_at(Vec3::Y*12.,Vec3::Y).to_matrix()
    }
    #[test]
    fn indoor_people_use_actual_support_and_stature_and_ignore_props_and_outdoors() {
        use crystal_render_api::VisualActorId;
        let mut f=frame();let heights=vec![0.;256];let original=f.actors[0].clone();
        let first=targets(&f,&heights,camera());assert_eq!(first.len(),1);
        f.actors.extend([
            crystal_render_api::VisualActor{id:VisualActorId::Object(2),source_id:"rock".into(),..original.clone()},
            crystal_render_api::VisualActor{id:VisualActorId::RemotePlayer(1),..original.clone()},
            crystal_render_api::VisualActor{id:VisualActorId::Player,..original.clone()},
            crystal_render_api::VisualActor{id:VisualActorId::Object(3),center:Vec2::splat(f32::NAN),..original.clone()},
        ]);
        assert_eq!(targets(&f,&heights,camera()),first);
        assert!(targets(&f,&[],camera()).is_empty());
        assert!(targets(&f,&heights,Mat4::NAN).is_empty());
        let lifted=targets(&f,&vec![4.;256],camera());
        for (a,b) in first.iter().zip(lifted) {assert!((b.body.bottom_radius.y-a.body.bottom_radius.y-4.).abs()<0.0001);}
        f.actors[0].center.x+=1.;assert_ne!(targets(&f,&heights,camera()),first);
        for map in ["SaffronCity","Route32","IlexForest",""] {
            f.map_id=map.into();assert!(targets(&f,&heights,camera()).is_empty());
        }
    }
    #[test]
    fn indoor_people_are_stably_bounded_and_cover_every_original_object_budget() {
        use crystal_render_api::VisualActorId;
        let mut f=frame();let actor=f.actors[0].clone();f.actors.clear();
        for id in (1..=20).rev() {f.actors.push(crystal_render_api::VisualActor {
            id:VisualActorId::Object(id),center:Vec2::new(id as f32-8.,-8.),..actor.clone()});}
        let rays=targets(&f,&vec![0.;256],camera());assert_eq!(rays.len(),16);
        f.actors.reverse();assert_eq!(targets(&f,&vec![0.;256],camera()),rays);
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack")).unwrap();
        assert!(loaded.pack().data().maps.values().all(|m|m.objects.len()<=16),
            "the bound must not omit an original map object");
    }
    #[test]
    fn furniture_reveal_uses_npc_only_rays_and_restores_after_the_character_moves() {
        source_reveal_uses_npc_rays_and_restores(true);
    }

    #[test]
    fn smooth_partition_reveal_uses_npc_rays_without_changing_its_blend_policy() {
        source_reveal_uses_npc_rays_and_restores(false);
    }

    fn source_reveal_uses_npc_rays_and_restores(coverage: bool) {
        let mut app=App::new();app.add_plugins((MinimalPlugins,bevy::transform::TransformPlugin))
            .init_resource::<Assets<Mesh>>().init_resource::<Assets<VoxelMaterial>>()
            .init_resource::<TerrainRevisionCache>().insert_resource(VoxelViewStatus{active:true,..default()})
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f64(1./60.)));
        register(&mut app);let f=frame();let player=CutawayUniform::for_player(Vec3::new(-42.,0.04,8.),16.).unwrap();
        let pose=Transform::from_matrix(camera());
        let mut material=crate::solid_terrain_material();material.extension.cutaway=player;
        let handle=app.world_mut().resource_mut::<Assets<VoxelMaterial>>().add(material);
        let mut data=SurfaceMeshData::default();
        // A real source-like cabinet front blocks the clerk, clear of every player ray.
        data.positions=vec![[0.,0.,30.],[32.,0.,30.],[32.,32.,30.],[0.,32.,30.]];
        data.normals=vec![[0.,0.,1.];4];data.colors=vec![[1.;4];4];data.uvs=vec![[0.;2];4];
        data.indices=vec![0,1,2,0,2,3];data.cutaway_ranges=vec![0..4];
        let (eye,rays)=target_rays(player,camera()).unwrap();let tree=TriangleTree::new(&data);
        assert!(!rays.into_iter().any(|end|tree.blocks(eye,end)));
        assert!(targets(&f,&vec![0.;256],camera()).into_iter().flat_map(|p|p.rays).any(|end|tree.blocks(eye,end)));
        let (_,mut groups)=prepare(data);use_source_depth_coverage_fade(&mut groups,&[0..4]);
        set_coverage_fade(&mut groups,coverage);
        let root=app.world_mut().spawn((Transform::default(),Visibility::Inherited)).id();
        app.world_mut().resource_scope(|world,mut meshes:Mut<Assets<Mesh>>| {
            world.resource_scope(|world,mut materials:Mut<Assets<VoxelMaterial>>| {
                spawn_groups(&mut world.commands(),&mut meshes,&mut materials,root,groups,handle.clone(),None);
            });
        });app.world_mut().flush();
        app.world_mut().spawn((Camera3d::default(),Msaa::Off,pose,VoxelWorldCamera));
        {let mut cache=app.world_mut().resource_mut::<TerrainRevisionCache>();cache.solid_material=Some(handle);
            cache.built_frame=Some(f.clone());cache.footing_heights=vec![0.;256];}
        app.insert_resource(f);
        for _ in 0..60 {app.update();}
        let entity=app.world_mut().query_filtered::<Entity,With<OccluderGroup>>().single(app.world()).unwrap();
        let group=app.world().get::<OccluderGroup>(entity).unwrap();assert!(source_object(group)&&group.blocked&&group.fade.opacity<0.21);
        assert_eq!(group.coverage_fade,coverage,"source blend style must not change");
        let aperture=group.furniture_aperture.unwrap();
        assert_eq!(aperture,targets(app.world().resource::<crystal_render_api::VisualWorldFrame>(),&vec![0.;256],camera())[0].body);
        let material=app.world().resource::<Assets<VoxelMaterial>>().get(&group.transparent).unwrap();
        assert_eq!(material.extension.smooth_depth_fade,!coverage);
        assert_eq!(material.extension.cutaway.bottom_radius,aperture.bottom_radius);
        assert_eq!(material.extension.cutaway.top_feather,aperture.top_feather);
        app.world_mut().resource_mut::<crystal_render_api::VisualWorldFrame>().actors[0].center.x=-48.;
        for _ in 0..120 {app.update();}
        let group=app.world().get::<OccluderGroup>(entity).unwrap();assert!(!group.blocked);assert_eq!(group.fade.opacity,1.);
        // The same registered object must clear a blocked player body too,
        // retaining its existing smooth/coverage recipe and allocated leaves.
        let meshes=app.world().resource::<Assets<Mesh>>().len();
        let materials=app.world().resource::<Assets<VoxelMaterial>>().len();
        let handle=app.world().resource::<TerrainRevisionCache>().solid_material.clone().unwrap();
        let blocked_player=CutawayUniform::for_player(Vec3::new(12.,0.04,8.),16.).unwrap();
        app.world_mut().resource_mut::<Assets<VoxelMaterial>>().get_mut(&handle).unwrap().extension.cutaway=blocked_player;
        for _ in 0..60 {app.update();}
        let group=app.world().get::<OccluderGroup>(entity).unwrap();
        assert!(group.blocked);assert_eq!(group.furniture_aperture,Some(blocked_player));
        let material=app.world().resource::<Assets<VoxelMaterial>>().get(&group.transparent).unwrap();
        assert_eq!(material.extension.cutaway.bottom_radius,blocked_player.bottom_radius);
        assert_eq!(material.extension.smooth_depth_fade,!coverage);
        app.world_mut().resource_mut::<Assets<VoxelMaterial>>().get_mut(&handle).unwrap().extension.cutaway=player;
        for _ in 0..120 {app.update();}
        let group=app.world().get::<OccluderGroup>(entity).unwrap();
        assert!(!group.blocked);assert_eq!(group.fade.opacity,1.);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(),meshes);
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(),materials);
        // The same NPC behind an ordinary wall must not clear that wall.
        app.world_mut().get_mut::<OccluderGroup>(entity).unwrap().depth_coverage_fade=false;
        app.world_mut().resource_mut::<crystal_render_api::VisualWorldFrame>().actors[0].center.x=12.;
        for _ in 0..60 {app.update();}
        assert!(!app.world().get::<OccluderGroup>(entity).unwrap().blocked);
        assert!(app.world().get::<OccluderGroup>(entity).unwrap().furniture_aperture.is_none());
        // Camera walls retain their separate outside-room rule, even if they
        // were once registered as source furniture.
        {let mut group=app.world_mut().get_mut::<OccluderGroup>(entity).unwrap();
            group.depth_coverage_fade=true;group.camera_wall=Some([1.,0.,-500.]);}
        app.world_mut().resource_mut::<crystal_render_api::VisualWorldFrame>().actors[0].center.x=13.;
        for _ in 0..60 {app.update();}
        assert!(!app.world().get::<OccluderGroup>(entity).unwrap().blocked);
        {let mut group=app.world_mut().get_mut::<OccluderGroup>(entity).unwrap();group.camera_wall=None;}
        // A destination frame must never use the previous room's support.
        app.world_mut().resource_mut::<crystal_render_api::VisualWorldFrame>().map_id="FuchsiaMart".into();
        for _ in 0..60 {app.update();}
        assert!(!app.world().get::<OccluderGroup>(entity).unwrap().blocked);
    }
}

#[cfg(test)]
#[test]
fn furniture_aperture_encloses_both_characters_without_changing_their_source_bodies() {
    let a=CutawayUniform::for_player(Vec3::new(-8.,0.,12.),16.).unwrap();
    let b=CutawayUniform::for_player(Vec3::new(12.,4.,20.),24.).unwrap();
    let original=[a,b];let mut combined=None;merge(&mut combined,a);merge(&mut combined,b);
    let union=combined.unwrap();assert_eq!([a,b],original);
    for body in original {
        let offset=Vec2::new(body.bottom_radius.x-union.bottom_radius.x,body.bottom_radius.z-union.bottom_radius.z).length();
        assert!(offset+body.bottom_radius.w<=union.bottom_radius.w+0.0001);
        assert!(union.bottom_radius.y<=body.bottom_radius.y&&union.top_feather.y>=body.top_feather.y);
    }
    let mut reversed=None;merge(&mut reversed,b);merge(&mut reversed,a);assert_eq!(combined,reversed);
}

#[cfg(test)]
#[test]
fn low_counter_hiding_only_npc_legs_does_not_request_a_reveal() {
    let f=tests::frame();let camera=Transform::from_xyz(-20.,20.,128.)
        .looking_at(Vec3::Y*12.,Vec3::Y).to_matrix();
    let person=targets(&f,&vec![0.;256],camera)[0];
    let mut counter=SurfaceMeshData::default();
    counter.positions=vec![[0.,0.,30.],[32.,0.,30.],[32.,9.,30.],[0.,9.,30.]];
    counter.indices=vec![0,1,2,0,2,3];let tree=TriangleTree::new(&counter);
    let eye=camera.w_axis.truncate();
    assert!(person.rays[..3].iter().any(|&end|tree.blocks(eye,end)));
    assert!(!person.obscured(|end|tree.blocks(eye,end)));
}

#[cfg(test)]
#[test]
fn cabinet_top_grazing_the_chest_still_reveals_the_lower_torso() {
    let f=tests::frame();let camera=Transform::from_xyz(-20.,60.,128.)
        .looking_at(Vec3::Y*12.,Vec3::Y).to_matrix();
    let person=targets(&f,&vec![0.;256],camera)[0];
    let mut cabinet=SurfaceMeshData::default();
    cabinet.positions=vec![[0.,0.,30.],[32.,0.,30.],[32.,19.5,30.],[0.,19.5,30.]];
    cabinet.indices=vec![0,1,2,0,2,3];let tree=TriangleTree::new(&cabinet);
    let (eye,chest_rays)=target_rays(person.body,camera).unwrap();
    assert!(!chest_rays[3..].iter().any(|&end|tree.blocks(eye,end)));
    assert!(person.obscured(|end|tree.blocks(eye,end)));
}
