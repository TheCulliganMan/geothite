//! Stateful PlayRadioShow/PrintRadioLine/RadioScroll ownership.
//! Explicit source waits are modeled; CPU work crossing VBlank is not.
use super::radio_program::{
    RadioProgramContext, RadioProgramEffect, RadioProgramError, RadioProgramText,
    RadioProgramTextSource, radio_program_step,
};
use super::radio_text::{
    RADIO_SCROLL, RadioLinePrinter, RadioScrollState, RadioTextEnvironment, RadioTextError,
    RadioTextWindow,
};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RadioPlaybackError {
    #[error(transparent)]
    Program(#[from] RadioProgramError),
    #[error(transparent)]
    Text(#[from] RadioTextError),
    #[error("radio host: {0}")]
    Host(String),
}

/// The owner cannot skip an effect. A suspended source call (such as WaitBgMap)
/// returns false until complete; later effects and text remain queued.
pub trait RadioPlaybackHost: RadioTextEnvironment {
    fn effect(
        &mut self,
        effect: &RadioProgramEffect,
        window: &mut RadioTextWindow,
    ) -> Result<bool, RadioPlaybackError>;
    fn text(&mut self, source: &RadioProgramTextSource) -> Result<Vec<u8>, RadioPlaybackError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioPlayback {
    pub state: RadioScrollState,
    pub window: RadioTextWindow,
    text_buffer: Vec<u8>,
    printer: Option<RadioLinePrinter>,
    effects: VecDeque<RadioProgramEffect>,
    pending_text: Option<RadioProgramText>,
    executing: bool,
}

impl RadioPlayback {
    /// Station loading supplies the existing textbox, including its frame.
    pub fn new(line: u8, window: RadioTextWindow) -> Self {
        Self {
            state: RadioScrollState {
                current_line: line,
                next_line: 0,
                delay: 0,
                printed: 0,
            },
            window,
            text_buffer: Vec::new(),
            printer: None,
            effects: VecDeque::new(),
            pending_text: None,
            executing: false,
        }
    }

    pub fn printing(&self) -> bool {
        self.printer.is_some()
    }

    pub fn call_suspended(&self) -> bool {
        self.printer.is_some() || self.executing
    }

    /// One outer radio call, or one frame resuming a suspended source call.
    /// The host owns external state, formatting, audio and LCD waits. The
    /// supplied context is sampled only when a new program call begins.
    pub fn advance_frame(
        &mut self,
        mut context: RadioProgramContext<'_>,
        rockets_in_tower: bool,
        in_johto: bool,
        held_ab: bool,
        random: &mut impl FnMut(bool) -> Result<u8, RadioProgramError>,
        host: &mut impl RadioPlaybackHost,
    ) -> Result<(), RadioPlaybackError> {
        if self.printer.is_some() {
            return self.advance_printer(host, held_ab);
        }
        if !self.executing {
            // PlayRadioShow applies takeover only at station entry, never to
            // a continuation, the scroll routine, or the three special songs.
            if self.state.current_line < 8 && rockets_in_tower && in_johto {
                self.state.current_line = 7;
            }
            if self.state.current_line == RADIO_SCROLL {
                self.state.step(&mut self.window);
                return Ok(());
            }
            context.printed = self.state.printed;
            let oak = context.oak.copied().map(|mut oak| {
                oak.delay = self.state.delay;
                oak
            });
            let step = radio_program_step(
                self.state.current_line,
                RadioProgramContext {
                    oak: oak.as_ref(),
                    ..context
                },
                random,
            )?;
            self.effects = step.effects.into();
            self.pending_text = step.text;
            self.executing = true;
        }
        while let Some(effect) = self.effects.front() {
            match effect {
                RadioProgramEffect::SetCurrentLine(value) => self.state.current_line = *value,
                RadioProgramEffect::SetNextLine(value) => self.state.next_line = *value,
                RadioProgramEffect::SetRadioDelay(value) => self.state.delay = *value,
                RadioProgramEffect::SetPrintedLines(value) => self.state.printed = *value,
                _ => {
                    if !host.effect(effect, &mut self.window)? {
                        return Ok(());
                    }
                }
            }
            self.effects.pop_front();
        }
        if let Some(text) = self.pending_text.as_ref() {
            if text.source != RadioProgramTextSource::RetainedBuffer {
                self.text_buffer = host.text(&text.source)?;
            }
            // PrintRadioLine changes wRadioText itself, which matters when
            // Oak's source overflow branch prints that same buffer again.
            if self.state.printed < 2 {
                *self
                    .text_buffer
                    .get_mut(1)
                    .ok_or(RadioTextError::Truncated)? = 0;
            }
            self.printer = Some(RadioLinePrinter::new(
                &self.text_buffer,
                &mut self.state,
                text.next_line,
            )?);
            self.pending_text = None;
        }
        self.executing = false;
        self.advance_printer(host, held_ab)
    }

    fn advance_printer(
        &mut self,
        host: &impl RadioPlaybackHost,
        held_ab: bool,
    ) -> Result<(), RadioPlaybackError> {
        if let Some(printer) = self.printer.as_mut() {
            if printer.advance_frame(&mut self.window, host, held_ab)? {
                self.printer = None;
                self.state.finish_print();
            }
        }
        Ok(())
    }
}
