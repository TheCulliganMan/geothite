//! Frame and audio fences shared by native and renderer-neutral field text.
use anyhow::Result;

use super::field_text_program::{self, TextEffect, TextProgram, TextTiming};
use super::{
    BevyRuntimeShell, RuntimeShellSnapshot, queue_visible_shell_sound_effect, visible_rival_name,
    visible_wait_sfx_finished,
};

pub(super) fn initialize(
    shell: &mut BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    pages: &[String],
) {
    let Some(reveal) = shell.field_text_reveal.as_ref() else {
        return;
    };
    let label = if shell.field_notice.is_some() {
        "field-notice"
    } else if shell.pc_notice.is_some() {
        "pc-notice"
    } else {
        snapshot
            .ui
            .text
            .as_ref()
            .map_or("special-text", |text| text.label.as_str())
    }
    .to_owned();
    if shell
        .field_text_timing
        .as_ref()
        .is_some_and(|timing| timing.identity == reveal.text && timing.label == label)
    {
        return;
    }
    let mut body = if shell.field_notice.is_none()
        && shell.pc_notice.is_none()
        && !shell.pc_hub_session_open
    {
        snapshot.ui.text.as_ref().and_then(|text| text.body.clone())
    } else {
        None
    };
    for _ in 0..8 {
        let far = body
            .as_ref()
            .and_then(|body| body.commands.iter().find(|c| c.command == "text_far"))
            .and_then(|c| c.args.first())
            .cloned();
        let Some(label) = far else {
            break;
        };
        body = shell
            .shell
            .text_snapshot(&label)
            .ok()
            .and_then(|text| text.body);
    }
    let program = body
        .map(|body| {
            field_text_program::compile(
                &body,
                &shell.shell.session().state().script_runtime.named_buffers,
                &snapshot.trainer.player_name,
                visible_rival_name(snapshot),
                snapshot.progression.time.day_of_week,
            )
        })
        .filter(|program| {
            program
                .pages
                .iter()
                .map(|page| page.text.as_str())
                .eq(pages.iter().map(String::as_str))
        })
        .unwrap_or_else(|| TextProgram::plain(pages.to_vec()));
    let identity = reveal.text.clone();
    let label_changed = shell
        .field_text_timing
        .as_ref()
        .is_some_and(|timing| timing.label != label);
    if label_changed && let Some(reveal) = shell.field_text_reveal.as_mut() {
        reveal.page_index = 0;
        reveal.visible_chars = 0;
        reveal.frames_until_next_char = 0;
    }
    let page_index = shell
        .field_text_reveal
        .as_ref()
        .map_or(0, |reveal| reveal.page_index);
    let mut timing = TextTiming::new(identity, program, page_index);
    timing.label = label;
    shell.field_text_timing = Some(timing);
}

pub(super) fn complete(shell: &BevyRuntimeShell) -> bool {
    shell.field_text_timing.as_ref().is_none_or(|timing| {
        shell
            .field_text_reveal
            .as_ref()
            .is_none_or(|reveal| reveal.text != timing.identity)
            || timing.complete()
    })
}

/// Returns true when a native delay or sound consumes this printer frame.
pub(super) fn advance(shell: &mut BevyRuntimeShell, accelerated: bool) -> Result<bool> {
    if shell
        .field_text_timing
        .as_ref()
        .is_some_and(|timing| timing.waiting_sound)
    {
        if !visible_wait_sfx_finished(shell) {
            return Ok(true);
        }
        if let Some(timing) = shell.field_text_timing.as_mut() {
            timing.waiting_sound = false;
        }
    }
    let Some(timing) = shell.field_text_timing.as_mut() else {
        return Ok(false);
    };
    if timing.frames_remaining > 0 {
        timing.frames_remaining -= 1;
        if timing.frames_remaining == 0 {
            timing.transition_active = false;
        }
        return Ok(true);
    }
    let visible_chars = shell
        .field_text_reveal
        .as_ref()
        .map_or(0, |reveal| reveal.visible_chars);
    let Some(page) = timing.program.pages.get(timing.page_index) else {
        return Ok(false);
    };
    while let Some(cue) = page
        .cues
        .get(timing.next_cue)
        .filter(|cue| cue.after_chars <= visible_chars)
    {
        timing.next_cue += 1;
        match cue.effect {
            TextEffect::Pause if !accelerated => {
                timing.frames_remaining = 29;
                return Ok(true);
            }
            TextEffect::Pause => {}
            TextEffect::Sound(id) => {
                timing.waiting_sound = true;
                queue_visible_shell_sound_effect(shell, id)?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub(super) fn reveal_limit(shell: &BevyRuntimeShell, requested: usize) -> usize {
    shell
        .field_text_timing
        .as_ref()
        .and_then(|timing| {
            timing
                .program
                .pages
                .get(timing.page_index)
                .and_then(|page| page.cues.get(timing.next_cue))
        })
        .map_or(requested, |cue| requested.min(cue.after_chars))
}

/// CONT scrolls two tile rows, spending five VBlanks at each intermediate pose.
pub(super) fn scroll_offset(shell: &BevyRuntimeShell) -> f32 {
    shell.field_text_timing.as_ref().map_or(0.0, |timing| {
        if timing.transition_active
            && timing.frames_remaining > 5
            && timing
                .program
                .pages
                .get(timing.page_index)
                .is_some_and(|page| page.transition == field_text_program::PageTransition::Scroll)
        {
            1.0
        } else {
            0.0
        }
    })
}

// The renderer-neutral driver retains the audio event for inspection but has no
// device to complete it. Native/browser frames wait on the actual backend.
pub(super) fn settle_sound(shell: &mut BevyRuntimeShell) {
    if let Some(timing) = shell.field_text_timing.as_mut() {
        timing.waiting_sound = false;
    }
}
