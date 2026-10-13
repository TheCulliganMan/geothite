//! Incremental source-command clock. It advances only when the sound IRQ runs.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SourceChannelState {
    pub channel: u8,
    pub flags1: u8,
    pub flags2: u8,
    pub flags3: u8,
    pub note_flags: u8,
    pub duty: u8,
    pub envelope: u8,
    pub frequency: u16,
    pub pitch: u8,
    pub octave: u8,
    pub transpose: u8,
    pub duration: u8,
    pub duration_modifier: u8,
    pub note_length: u8,
    pub tempo: u16,
    pub duty_pattern: u8,
    pub vibrato_delay_count: u8,
    pub vibrato_delay: u8,
    pub vibrato_extent: u8,
    pub vibrato_rate: u8,
    pub pitch_offset: u16,
}

#[derive(Clone, Debug)]
pub struct SourceChannelStep {
    pub before: SourceChannelState,
    /// State at entry to ApplyPitchSlide, after parsing a new note if needed.
    pub parsed: SourceChannelState,
    /// State at entry to UpdateChannels (including shadowed music).
    pub after: SourceChannelState,
    pub commands: Vec<AudioCommand>,
    pub note_over: bool,
    pub shadowed: bool,
    pub hardware_write: bool,
    pub noise_delay_before: u8,
    pub noise_delay_after: u8,
    pub noise_sample_first_byte: Option<u8>,
}

#[derive(Clone)]
struct Channel {
    state: SourceChannelState,
    sources: BTreeMap<String, AudioSource>,
    source: String,
    pc: usize,
    returns: Vec<(String, usize)>,
    loops: BTreeMap<(String, usize), u8>,
    kit: Option<i32>,
    samples: Vec<NoiseNote>,
    sample: usize,
}

/// Eight source channels, including inaudible music underneath active SFX.
pub struct SourceAudioClock {
    channels: [Option<Channel>; 8],
    context: SynthContext,
    pub sfx_priority: bool,
    pub noise_delay: u8,
    pub music_drumkit: i32,
    pub sfx_drumkit: i32,
    pub updates: u64,
}

impl SourceAudioClock {
    pub fn new() -> Result<Self> {
        Ok(Self { channels: std::array::from_fn(|_| None), context: SynthContext::cartridge()?, sfx_priority: false, noise_delay: 0, music_drumkit: 0, sfx_drumkit: 0, updates: 0 })
    }

    pub fn stop_all(&mut self) {
        self.channels = std::array::from_fn(|_| None);
        self.noise_delay = 0;
        self.sfx_priority = false;
    }

    pub fn channels(&self) -> Vec<SourceChannelState> {
        self.channels.iter().filter_map(|c| c.as_ref().map(|c| c.state.clone())).collect()
    }

    pub fn play(&mut self, program: &Program) -> Result<()> {
        validate_references(&program.music_data)?;
        let sfx = program.music_data.channels.values().any(|c| c.number.is_some_and(|n| n > 4));
        let range = if sfx { 4..8 } else { 0..4 };
        for n in range { self.channels[n] = None; }
        for (name, source) in &program.music_data.channels {
            let n = source.number.context("source clock channel has no number")?;
            ensure!((1..=8).contains(&n), "invalid source clock channel {n}");
            let mut state = SourceChannelState { channel: n, flags1: 1 | if n > 4 { 8 } else { 0 }, note_length: 1, tempo: 256, ..Default::default() };
            if let Some(length) = program.cry_length { if n % 4 != 0 { state.tempo = length as u16; } state.flags1 |= 32; }
            if let Some(pitch) = program.cry_pitch { state.pitch_offset = pitch as u16; state.flags2 |= 16; }
            self.channels[(n - 1) as usize] = Some(Channel { state, sources: program.music_data.sources(), source: name.clone(), pc: 0, returns: vec![], loops: BTreeMap::new(), kit: None, samples: vec![], sample: 0 });
        }
        Ok(())
    }

    pub fn update_vblank(&mut self) -> Result<Vec<SourceChannelStep>> {
        let mut out = Vec::new();
        for index in 0..8 {
            let Some(mut channel) = self.channels[index].take() else { continue };
            if channel.state.flags1 & 1 == 0 { self.channels[index] = Some(channel); continue; }
            let before = channel.state.clone();
            let note_over = before.duration < 2;
            let mut commands = vec![];
            if note_over {
                channel.state.vibrato_delay_count = channel.state.vibrato_delay;
                channel.state.flags2 &= !2;
                self.parse(&mut channel, &mut commands)?;
            } else { channel.state.duration -= 1; }
            let parsed = channel.state.clone();
            ensure!(channel.state.flags2 & 2 == 0, "source clock pitch slide is not implemented");
            let mut frequency = channel.state.frequency;
            let mut current_envelope = channel.state.envelope;
            if channel.state.flags2 & 4 != 0 {
                channel.state.duty_pattern = channel.state.duty_pattern.rotate_left(2);
                channel.state.duty = channel.state.duty_pattern & 0xc0;
                channel.state.note_flags |= 1;
            }
            if channel.state.flags2 & 16 != 0 { frequency = frequency.wrapping_add(channel.state.pitch_offset); }
            if channel.state.flags2 & 1 != 0 {
                if channel.state.vibrato_delay_count != 0 { channel.state.vibrato_delay_count -= 1; }
                else if channel.state.vibrato_extent != 0 {
                    if channel.state.vibrato_rate & 15 != 0 { channel.state.vibrato_rate -= 1; }
                    else {
                        channel.state.vibrato_rate |= channel.state.vibrato_rate >> 4;
                        let low = frequency as u8;
                        let next = if channel.state.flags3 & 1 != 0 { channel.state.flags3 &= !1; low.saturating_sub(channel.state.vibrato_extent & 15) }
                            else { channel.state.flags3 |= 1; low.saturating_add(channel.state.vibrato_extent >> 4) };
                        frequency = (frequency & 0xff00) | next as u16;
                        channel.state.note_flags |= 64;
                    }
                }
            }
            let noise_delay_before = self.noise_delay;
            let mut noise_sample_first_byte = None;
            let noise_blocked = index < 4 && self.channels[7].as_ref().is_some_and(|c| c.state.flags1 & 17 == 17);
            if channel.state.flags1 & 16 != 0 && !noise_blocked {
                if self.noise_delay != 0 { self.noise_delay -= 1; }
                else if let Some(sample) = channel.samples.get(channel.sample) {
                    noise_sample_first_byte = Some(sample.length as u8);
                    self.noise_delay = (sample.length as u8 & 15) + 1;
                    current_envelope = envelope(sample.volume, sample.fade);
                    frequency = sample.frequency as u16;
                    channel.sample += 1;
                    channel.state.note_flags |= 16;
                } else if !channel.samples.is_empty() { noise_sample_first_byte = Some(255); }
            }
            let shadowed = index < 4 && self.channels[index + 4].as_ref().is_some_and(|c| c.state.flags1 & 1 != 0);
            if index < 4 && self.sfx_priority && self.channels[4..].iter().any(|c| c.as_ref().is_some_and(|c| c.state.flags1 & 1 != 0)) { channel.state.note_flags |= 32; }
            let mut after = channel.state.clone();
            after.frequency = frequency; after.envelope = current_envelope;
            out.push(SourceChannelStep { before, parsed, after, commands, note_over, shadowed, hardware_write: !shadowed, noise_delay_before, noise_delay_after: self.noise_delay, noise_sample_first_byte });
            channel.state.note_flags = 0;
            self.channels[index] = Some(channel);
        }
        self.updates += 1;
        Ok(out)
    }

    fn parse(&mut self, c: &mut Channel, commands: &mut Vec<AudioCommand>) -> Result<()> {
        for _ in 0..10000 {
            let Some(command) = c.sources.get(&c.source).and_then(|s| s.commands.get(c.pc)).cloned() else {
                c.state.flags1 &= !1; c.state.note_flags |= 32; return Ok(());
            };
            c.pc += 1;
            let op = command.command.as_str();
            if op == "label" || op == "assert" { continue; }
            commands.push(command.clone());
            match op {
                "sound_call" => { let target = resolve(&c.sources, &c.source, &command.args[0])?; c.returns.push((c.source.clone(), c.pc)); (c.source, c.pc) = target; c.state.flags1 |= 2; }
                "sound_ret" => { if let Some(target) = c.returns.pop() { (c.source, c.pc) = target; c.state.flags1 &= !2; } else { c.state.flags1 &= !1; c.state.note_flags |= 32; return Ok(()); } }
                "sound_jump" => { (c.source, c.pc) = resolve(&c.sources, &c.source, &command.args[0])?; }
                "sound_loop" => { let count = arg(&command, 0)? as u8; let key = (c.source.clone(), c.pc - 1); let left = c.loops.entry(key.clone()).or_insert(count.wrapping_sub(1)); if count == 0 || *left != 0 { if count != 0 { *left -= 1; } (c.source, c.pc) = resolve(&c.sources, &c.source, &command.args[1])?; c.state.flags1 |= 4; } else { c.loops.remove(&key); c.state.flags1 &= !4; } }
                "tempo" => { let tempo = arg(&command, 0)? as u16; c.state.tempo = tempo; let group = if c.state.channel <= 4 { 0..4 } else { 4..8 }; for n in group { if let Some(other) = &mut self.channels[n] { other.state.tempo = tempo; other.state.duration_modifier = 0; } } c.state.duration_modifier = 0; }
                "speed" | "note_type" | "drum_speed" => { c.state.note_length = arg(&command, 0)? as u8; if command.args.len() >= 3 { c.state.envelope = if c.state.channel % 4 == 3 { ((arg(&command, 1)? as u8) << 4) | arg(&command, 2)? as u8 } else { envelope(arg(&command, 1)?, arg(&command, 2)?) }; } }
                "octave" => c.state.octave = (8 - arg(&command, 0)?) as u8 & 7,
                "inc_octave" => c.state.octave = c.state.octave.wrapping_sub(1) & 7,
                "dec_octave" => c.state.octave = c.state.octave.wrapping_add(1) & 7,
                "transpose" => c.state.transpose = ((arg(&command, 0)? as u8) << 4) | arg(&command, 1)? as u8,
                "duty_cycle" => { c.state.duty = (arg(&command, 0)? as u8 & 3) << 6; c.state.note_flags |= 1; }
                "duty_cycle_pattern" => { let mut p = 0u8; for i in 0..4 { p |= (arg(&command, i)? as u8 & 3) << (6 - i * 2); } c.state.duty_pattern = p.rotate_right(2); c.state.flags2 |= 4; }
                "volume_envelope" | "channel_volume" => { c.state.envelope = envelope(arg(&command, 0)?, arg(&command, 1)?); }
                "vibrato" => { c.state.flags2 |= 1; c.state.flags3 &= !1; c.state.vibrato_delay = arg(&command, 0)? as u8; c.state.vibrato_delay_count = c.state.vibrato_delay; let packed = if command.args.len() == 2 { arg(&command, 1)? as u8 } else { ((arg(&command, 1)? as u8) << 4) | arg(&command, 2)? as u8 }; let extent = packed >> 4; c.state.vibrato_extent = extent.div_ceil(2) << 4 | extent / 2; c.state.vibrato_rate = (packed & 15) * 17; }
                "pitch_offset" => { c.state.pitch_offset = arg(&command, 0)? as u16; c.state.flags2 |= 16; }
                "sfx_priority_on" => self.sfx_priority = true,
                "sfx_priority_off" => self.sfx_priority = false,
                "toggle_sfx" => c.state.flags1 ^= 8,
                "toggle_noise" | "sfx_toggle_noise" => { c.state.flags1 ^= 16; if c.state.flags1 & 16 != 0 { let kit = arg(&command, 0)?; c.kit = Some(kit); if c.state.channel <= 4 { self.music_drumkit = kit; } else { self.sfx_drumkit = kit; } } }
                "pitch_sweep" | "sweep" => c.state.note_flags |= 8,
                "volume" | "stereo_panning" | "force_stereo_panning" | "toggle_perfect_pitch" => {},
                "rest" | "note" | "wave_note" | "square_note" | "noise_note" | "drum_note" => {
                    let ticks = match op { "rest" => arg(&command, 0)?, "square_note" | "noise_note" => arg(&command, 0)? + 1, _ => arg(&command, 1)? } as u8;
                    let scaled = c.state.note_length.wrapping_mul(ticks);
                    let duration = c.state.tempo.wrapping_mul(scaled as u16).wrapping_add(c.state.duration_modifier as u16);
                    c.state.duration = (duration >> 8) as u8;
                    c.state.duration_modifier = duration as u8;
                    if op == "rest" { c.state.note_flags |= 32; }
                    else if op == "square_note" || op == "noise_note" { c.state.envelope = envelope(arg(&command, 1)?, arg(&command, 2)?); c.state.frequency = arg(&command, 3)? as u16; c.state.note_flags |= 16; }
                    else if op == "drum_note" {
                        let drum_pitch = arg(&command, 0)? as u8;
                        if c.state.channel % 4 == 0 && c.state.flags1 & 16 != 0 && !(c.state.channel <= 4 && self.channels[7].as_ref().is_some_and(|s| s.state.flags1 & 1 != 0)) {
                            let kit = if c.state.channel <= 4 { self.music_drumkit } else { self.sfx_drumkit };
                            c.samples = self.context.drumkits.get(&kit).and_then(|k| k.get(&(drum_pitch as i32))).cloned().context("source clock missing drum sample")?;
                            c.sample = 0; self.noise_delay = 0;
                        }
                    } else { let mut state = State::default(); state.octave = c.state.octave as i32; state.transpose_octaves = (c.state.transpose >> 4) as i32; state.transpose_pitches = (c.state.transpose & 15) as i32; c.state.frequency = pitch(&command.args[0], &state)? as u16; c.state.pitch = ["C_","C#","D_","D#","E_","F_","F#","G_","G#","A_","A#","B_"].iter().position(|n| *n == command.args[0]).context("invalid note")? as u8 + 1; c.state.note_flags |= 16; }
                    return Ok(());
                }
                _ => bail!("source clock unsupported command {op} in {}", c.source),
            }
        }
        bail!("source clock command budget exhausted in {}", c.source)
    }
}

fn envelope(volume: i32, fade: i32) -> u8 {
    ((volume as u8 & 15) << 4) | if fade < 0 { 8 | (-fade as u8 & 7) } else { fade as u8 & 15 }
}

fn resolve(sources: &BTreeMap<String, AudioSource>, current: &str, target: &str) -> Result<(String, usize)> {
    if sources.contains_key(target) { return Ok((target.into(), 0)); }
    let scoped = if target.starts_with('.') { format!("{}{}", current.split('.').next().unwrap(), target) } else { target.into() };
    for (name, source) in sources { for (index, command) in source.commands.iter().enumerate() { if command.command == "label" { if let Some(label) = command.args.first() { if (name == current && label == target) || label == &scoped || (label.starts_with('.') && format!("{name}{label}") == scoped) { return Ok((name.clone(), index + 1)); } } } } }
    bail!("source clock unresolved target {target} in {current}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn program(channel: u8, commands: &[(&str, &[&str])]) -> Program {
        Program { profile: "cartridge".into(), cry_pitch: None, cry_length: None, music_data: MusicData { channel_count: 1, channels: BTreeMap::from([("Fixture".into(), AudioSource { number: Some(channel), commands: commands.iter().map(|(command,args)| AudioCommand { command: (*command).into(), args: args.iter().map(|a|(*a).into()).collect() }).collect() })]), subroutines: BTreeMap::new(), shared_sources: BTreeMap::new() } }
    }

    #[test]
    fn source_clock_keeps_parsing_shadowed_music() -> Result<()> {
        let mut clock = SourceAudioClock::new()?;
        clock.play(&program(1, &[("tempo", &["384"]), ("octave", &["4"]), ("note", &["C_", "1"]), ("note", &["D_", "1"]), ("sound_ret", &[])]))?;
        clock.play(&program(5, &[("square_note", &["8", "8", "8", "1000"]), ("sound_ret", &[])]))?;
        let first = clock.update_vblank()?;
        assert!(first[0].shadowed && !first[0].hardware_write);
        assert_eq!((first[0].parsed.duration, first[0].parsed.duration_modifier), (1,128));
        let second = clock.update_vblank()?;
        assert!(second[0].shadowed && second[0].note_over);
        assert_eq!((second[0].parsed.duration, second[0].parsed.duration_modifier), (2,0));
        assert_ne!(first[0].parsed.frequency, second[0].parsed.frequency);
        Ok(())
    }

    #[test]
    fn source_clock_vibrato_and_duty_advance_on_sustained_frames() -> Result<()> {
        let mut clock = SourceAudioClock::new()?;
        clock.play(&program(1, &[("vibrato", &["1", "3", "1"]), ("duty_cycle_pattern", &["0", "1", "2", "3"]), ("octave", &["4"]), ("note", &["C_", "8"]), ("sound_ret", &[])]))?;
        let first = clock.update_vblank()?.remove(0);
        assert_eq!(first.after.vibrato_delay_count,0);
        assert_eq!(first.after.vibrato_extent,0x21);
        let second = clock.update_vblank()?.remove(0);
        assert_eq!(second.after.vibrato_rate,0x10);
        assert_ne!(first.after.duty,second.after.duty);
        let third = clock.update_vblank()?.remove(0);
        assert_eq!(third.after.vibrato_rate,0x11);
        assert_eq!(third.after.frequency,third.parsed.frequency+2);
        assert_eq!(third.after.note_flags & 64,64);
        Ok(())
    }

    #[test]
    fn source_clock_executes_counted_loops_and_returns() -> Result<()> {
        let mut clock = SourceAudioClock::new()?;
        clock.play(&program(8, &[("label", &[".loop"]), ("noise_note", &["0", "8", "8", "33"]), ("sound_loop", &["3", ".loop"]), ("sound_ret", &[])]))?;
        for _ in 0..3 { let step=clock.update_vblank()?.remove(0); assert_eq!(step.parsed.frequency,33); assert_eq!(step.parsed.envelope,0x88); }
        let end=clock.update_vblank()?.remove(0);
        assert_eq!(end.after.flags1&1,0);
        assert_eq!(end.after.note_flags&32,32);
        assert!(clock.update_vblank()?.is_empty());
        Ok(())
    }
}




