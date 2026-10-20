//! Continuous field locomotion; all cell entries use the existing gameplay loop.
use super::*;
use crate::core::world::free_movement::FreeMovement;

#[derive(Default)]
pub(super) struct NaturalMovement {
    pub enabled: bool,
    position: FreeMovement,
    anchor: Option<(String, TilePosition)>,
    pending: Option<Direction>,
}

impl NaturalMovement {
    pub fn displacement(&self, map: &str, tile: TilePosition) -> Vec2 {
        if !self.enabled || self.anchor.as_ref().is_none_or(|(m,t)| m != map || *t != tile) {
            return Vec2::ZERO;
        }
        Vec2::new(self.position.offset[0], -self.position.offset[1])
            * (TILE_SIZE * f32::from(RENDER_TILES_PER_RUNTIME_TILE))
    }
}

pub(super) fn keyboard_vector(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let axis = |positive, negative| i32::from(keys.pressed(positive)) - i32::from(keys.pressed(negative));
    Vec2::new(axis(KeyCode::ArrowRight,KeyCode::ArrowLeft) as f32,
        axis(KeyCode::ArrowDown,KeyCode::ArrowUp) as f32).clamp_length_max(1.0)
}

/// Called only after the production input owner releases field movement.
pub(super) fn prepare(shell: &mut BevyRuntimeShell, vector: Vec2, buttons: &mut Vec<GameButton>) -> Result<bool> {
    if !shell.natural_movement.enabled || shell.player_walk_frame_ticks > 0
        || shell.visible_ledge_jump.is_some()
        || shell.shell.session().overworld().forced_movement_direction().is_some() {
        shell.natural_movement = NaturalMovement { enabled: shell.natural_movement.enabled, ..Default::default() };
        return Ok(false);
    }
    let world = shell.shell.session().overworld();
    let anchor = (world.map.map_name.clone(), world.player.tile);
    if shell.natural_movement.anchor.as_ref() != Some(&anchor) {
        shell.natural_movement.position = FreeMovement::default();
        shell.natural_movement.anchor = Some(anchor);
    }
    let occupied = world.occupied_tiles_checked()?;
    let mut open = [false;4];
    for (index,direction) in [Direction::Left,Direction::Right,Direction::Up,Direction::Down].into_iter().enumerate() {
        let mut player = world.player;
        let result = crate::core::world::movement::attempt_step_with_occupied_tiles(
            &mut player,direction,&world.map,&world.tileset,
            crate::core::world::movement::StepOptions { force_step_after_turn:true,..Default::default() },&occupied);
        open[index] = matches!(result,StepOutcome::Moved {..});
        // Ledges retain their authored jump and landing controller.
        if !open[index] {
            let mut player = world.player;
            open[index] = matches!(crate::core::world::movement::attempt_ledge_jump_with_occupied_tiles(
                &mut player,direction,&world.map,&world.tileset,
                crate::core::world::movement::StepOptions { force_step_after_turn:true,..Default::default() },&occupied),
                LedgeJumpOutcome::Jumped {..});
        }
    }
    let speed = if buttons.contains(&GameButton::B) { 0.125 } else { 0.0625 }
        * f32::from(world.player.mode.speed_multiplier());
    let next = shell.natural_movement.position.advance(vector.to_array(),speed,open);
    shell.natural_movement.pending = next;
    buttons.retain(|button| !is_direction_button(*button));
    if let Some(direction) = next {
        buttons.insert(0, match direction { Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
            Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right });
    }
    Ok(true)
}

pub(super) fn finish(shell: &mut BevyRuntimeShell, frame: &RuntimeShellFrame) {
    let _ = (shell,frame);
}
