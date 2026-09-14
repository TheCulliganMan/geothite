//! Shared interpreter for bundled Pokémon main and idle picture programs.
//!
//! This state contains no textures, scene entities, or audio device handles.
//! A renderer selects art for `frame`; its presentation controller owns program
//! selection, cry cues, and transitions between main and idle animations.

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrontpicAnimation {
    pub species_id: String,
    pub speed: u16,
    pub pointer: usize,
    pub repeat: u16,
    pub wait: u16,
    pub frame: u16,
}

/// Advance one source interpreter call, returning true at endanim.
pub fn step_frontpic_animation(
    animation: &mut FrontpicAnimation,
    program: &crystal_core::models::frontpic_anim::FrontpicAnimProgram,
) -> Result<bool> {
    anyhow::ensure!(
        animation.speed <= 255 && animation.wait <= 255 && animation.repeat <= 255,
        "frontpic animation byte state is out of range"
    );
    anyhow::ensure!(
        !program.commands.is_empty() && program.commands.len() <= 256,
        "frontpic animation must fit its source byte command pointer"
    );
    if animation.wait > 0 {
        animation.wait -= 1;
        return Ok(false);
    }
    // PokeAnim_DoAnimScript may consume setrepeat and a taken dorepeat in
    // this call. A zero/exhausted dorepeat returns until the next frame.
    for _ in 0..program.commands.len() * 256 {
        let command = program
            .commands
            .get(animation.pointer)
            .context("frontpic animation reached missing command instead of endanim")?;
        animation.pointer = (animation.pointer + 1) & 0xff;
        let byte = |value: Option<u16>, field: &str| -> Result<u16> {
            let value =
                value.with_context(|| format!("frontpic {} requires {field}", command.kind))?;
            anyhow::ensure!(
                value <= 255,
                "frontpic {} {field} exceeds a source byte",
                command.kind
            );
            Ok(value)
        };
        match command.kind.as_str() {
            "setrepeat" => animation.repeat = byte(command.count, "count")?,
            "dorepeat" => {
                let target = usize::from(byte(command.target, "target")?);
                anyhow::ensure!(
                    target < program.commands.len(),
                    "frontpic repeat target is absent"
                );
                if animation.repeat == 0 {
                    return Ok(false);
                }
                animation.repeat -= 1;
                if animation.repeat == 0 {
                    return Ok(false);
                }
                animation.pointer = target;
            }
            "endanim" => return Ok(true),
            "frame" => {
                animation.frame = byte(command.frame, "frame")?;
                anyhow::ensure!(
                    animation.frame < 253,
                    "frontpic frame uses a reserved command byte"
                );
                let duration = byte(command.duration, "duration")?;
                // PokeAnim_GetDuration returns an 8-bit value. RunAnim falls
                // through WaitAnim, decrementing it on the frame-load call.
                let duration = (duration + duration * animation.speed / 16) as u8;
                animation.wait = u16::from(duration.wrapping_sub(1));
                return Ok(false);
            }
            other => anyhow::bail!("unknown frontpic animation command {other}"),
        }
    }
    anyhow::bail!("frontpic animation has no yielding command within its byte repeat range")
}
