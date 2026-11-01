/// A camera-relative walk may leave the preceding field pose off the discrete
/// step axis. Authenticate that actual published pose before allowing the
/// already committed step to publish its own centered field frame. This does
/// not turn the preceding pose into battle anchors or extend the entry window.
#[cfg(any(test, feature = "voxel-view"))]
fn continuous_encounter_frame_can_wait(
    source: &crate::core::world::session::OverworldSnapshot,
    from: TilePosition,
    map_size: UVec2,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> bool {
    let Some(facing) = rendered.visual_player_facing else {
        return false;
    };
    if !facing.is_finite() || (facing.length_squared() - 1.0).abs() > 0.0001 {
        return false;
    }
    let Some(cardinal) = rendered.player_sprite_facing else {
        return false;
    };
    let mut previous = source.clone();
    previous.tile = from;
    previous.facing = cardinal;
    capture_visible_field_source_frame(&previous, Some(map_size), rendered, frame, facing).is_some()
}
