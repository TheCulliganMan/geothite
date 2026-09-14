use super::*;
impl Renderer<'_> {
    pub(super) fn scan_tempo(&mut self, stream: &[AudioCommand]) -> Result<()> {
        let mut s = State::default();
        let mut frame = 0;
        for c in stream {
            match c.command.as_str() {
                "tempo" => {
                    s.tempo = arg(c, 0)?;
                    self.tempos.insert(frame, (s.tempo, s.remainder));
                }
                "speed" | "note_type" | "drum_speed" => {
                    s.note_length = arg(c, 0)?.max(1);
                    s.default_length = Some(s.note_length);
                }
                "note" | "wave_note" | "drum_note" => {
                    let ticks = if c.args.len() > 1 {
                        arg(c, 1)?
                    } else {
                        s.default_length.unwrap_or(4)
                    };
                    let tempo = s.tempo;
                    frame += note_frames(ticks, &mut s, tempo);
                }
                "rest" | "square_note" | "noise_note" => {
                    let ticks = arg(c, 0)? + if c.command == "rest" { 0 } else { 1 };
                    let tempo = s.tempo;
                    frame += note_frames(ticks, &mut s, tempo);
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn tempo(&self, channel: u8, frame: usize, default: i32) -> i32 {
        if channel == self.primary {
            default
        } else {
            self.tempos
                .range(..=frame)
                .next_back()
                .map(|(_, v)| v.0)
                .unwrap_or(default)
        }
    }
    pub(super) fn channel(&mut self, channel: u8, stream: &[AudioCommand]) -> Result<Vec<Segment>> {
        let is_wave = channel == 3 || channel == 7;
        let is_noise = channel == 4 || channel == 8;
        let is_pulse = !is_wave && !is_noise;
        let mut s = State::default();
        if let Some((_, (tempo, rem))) = self.tempos.first_key_value() {
            s.tempo = *tempo;
            s.remainder = *rem;
        }
        if let Some(pitch) = self.program.cry_pitch {
            s.pitch_offset = (pitch & 65535) as i32;
        }
        if let Some(length) = self.program.cry_length {
            if !is_noise {
                s.tempo = (length & 65535) as i32;
                s.remainder = 0;
            }
        }
        if self.perfect_pitch {
            s.instrument_pitch = 1;
        }
        let mut frame = 0;
        let mut offset = 0;
        let mut kit = None;
        let mut pulse_phase = 0.0;
        let mut out = Vec::new();
        for c in stream {
            let op = c.command.as_str();
            match op {
                "__loop_point__" => {
                    self.loops.insert(channel, (frame, offset));
                    continue;
                }
                "volume" => {
                    if channel == self.primary {
                        self.volume
                            .insert(offset, [arg(c, 0)?.clamp(0, 7), arg(c, 1)?.clamp(0, 7)]);
                    }
                    continue;
                }
                "tempo" => {
                    s.tempo = arg(c, 0)?;
                    if self.tempo_channel.is_none() {
                        self.tempo_channel = Some(channel);
                    }
                    if self.tempo_channel == Some(channel) {
                        self.tempos.insert(frame, (s.tempo, s.remainder));
                    }
                    continue;
                }
                "speed" => {
                    s.note_length = arg(c, 0)?.max(1);
                    s.default_length = Some(s.note_length);
                    continue;
                }
                "duty_cycle" => {
                    s.duty = arg(c, 0)? & 3;
                    continue;
                }
                "duty_cycle_pattern" => {
                    let mut packed = 0;
                    for i in 0..4 {
                        packed |= if i < c.args.len() {
                            arg(c, i)?.clamp(0, 3)
                        } else {
                            0
                        } << (6 - i * 2);
                    }
                    let p = ((packed >> 2) | ((packed & 3) << 6)) & 255;
                    s.duty_pattern = Some(p);
                    s.duty = (p >> 6) & 3;
                    continue;
                }
                "pitch_offset" => {
                    s.pitch_offset = arg(c, 0)?;
                    continue;
                }
                "pitch_slide" => {
                    if channel == 1 {
                        let mut tmp = s.clone();
                        tmp.octave = (8 - arg(c, 1)?).clamp(0, 7);
                        let target =
                            pitch(c.args.get(2).context("pitch_slide missing target")?, &tmp)?;
                        let tempo = self.tempo(channel, frame, s.tempo);
                        let frames = note_frames(arg(c, 0)?, &mut tmp, tempo);
                        s.slide = Some((target, frames.max(1)));
                    }
                    continue;
                }
                "vibrato" => {
                    s.vib_delay = arg(c, 0)?.max(0);
                    let (extent, rate) = if c.args.len() == 2 {
                        let p = arg(c, 1)?;
                        ((p >> 4) & 15, p & 15)
                    } else if c.args.len() > 2 {
                        (arg(c, 1)?, arg(c, 2)?)
                    } else {
                        (0, 0)
                    };
                    s.vib_delay_count = s.vib_delay;
                    s.vib_extent = extent.max(0);
                    s.vib_rate = rate.max(0);
                    s.vib_counter = s.vib_rate;
                    s.vib_up = false;
                    continue;
                }
                "pitch_sweep" | "sweep" => {
                    if channel == 1 {
                        let value = if c.args.len() == 1 {
                            arg(c, 0)? & 255
                        } else {
                            let shift = arg(c, 1)?;
                            ((arg(c, 0)? & 15) << 4)
                                | (if shift < 0 { 8 } else { 0 })
                                | (shift.abs() & 7)
                        };
                        s.sweep = value;
                        s.sweep_enabled = ((value >> 4) & 7) > 0 && (value & 7) > 0;
                    }
                    continue;
                }
                "toggle_perfect_pitch" => {
                    self.perfect_pitch = !self.perfect_pitch;
                    s.instrument_pitch = i32::from(self.perfect_pitch);
                    continue;
                }
                "octave" => {
                    s.octave = (8 - arg(c, 0)?).clamp(0, 7);
                    continue;
                }
                "inc_octave" => {
                    s.octave = (s.octave - 1).rem_euclid(8);
                    continue;
                }
                "dec_octave" => {
                    s.octave = (s.octave + 1) % 8;
                    continue;
                }
                "transpose" => {
                    s.transpose_octaves = arg(c, 0)?;
                    s.transpose_pitches = arg(c, 1)?;
                    continue;
                }
                "stereo_panning" | "force_stereo_panning" => {
                    if c.args.len() >= 2
                        && c.args[..2].iter().all(|a| {
                            matches!(a.as_str(), "TRUE" | "FALSE" | "true" | "false" | "0" | "1")
                        })
                    {
                        s.pan = [
                            matches!(c.args[0].as_str(), "TRUE" | "true" | "1"),
                            matches!(c.args[1].as_str(), "TRUE" | "true" | "1"),
                        ];
                    } else {
                        let p = arg(c, 0)?;
                        s.pan = [p & 16 != 0, p & 1 != 0];
                    }
                    continue;
                }
                "note_type" | "drum_speed" => {
                    s.note_length = arg(c, 0)?.max(1);
                    s.default_length = Some(s.note_length);
                    if op == "note_type" && c.args.len() > 1 {
                        if is_wave {
                            s.wave_volume = arg(c, 1)?;
                            if c.args.len() > 2 {
                                s.wave_instrument = arg(c, 2)?;
                            }
                        } else {
                            s.envelope =
                                Some((arg(c, 1)?, if c.args.len() > 2 { arg(c, 2)? } else { 0 }));
                        }
                    }
                    continue;
                }
                "volume_envelope" => {
                    if is_wave {
                        s.wave_volume = arg(c, 0)?;
                        if c.args.len() > 1 {
                            s.wave_instrument = arg(c, 1)?;
                        }
                    } else {
                        s.envelope =
                            Some((arg(c, 0)?, if c.args.len() > 1 { arg(c, 1)? } else { 0 }));
                    }
                    continue;
                }
                "channel_volume" => {
                    if is_wave {
                        s.wave_volume = arg(c, 0)?;
                    } else {
                        s.envelope = Some((arg(c, 0)?, 0));
                    }
                    continue;
                }
                "fade_wave" => {
                    if is_wave {
                        s.wave_instrument = arg(c, 0)?;
                    } else {
                        s.envelope = Some((s.envelope.map(|e| e.0).unwrap_or(15), arg(c, 0)?));
                    }
                    continue;
                }
                "load_wave" => {
                    let mode = c.args.first().context("load_wave empty")?;
                    let values = if mode == "db" || mode == "dn" {
                        &c.args[1..]
                    } else {
                        &c.args[..]
                    };
                    let mut samples = Vec::new();
                    for token in values {
                        let value = number(token)?;
                        if mode == "db" {
                            ensure!((0..=255).contains(&value), "invalid wave byte");
                            samples.extend([(value >> 4) & 15, value & 15]);
                        } else {
                            ensure!((0..=15).contains(&value), "invalid wave nibble");
                            samples.push(value);
                        }
                    }
                    ensure!(samples.len() == 32, "inline wave requires 32 nibbles");
                    let index = self
                        .context
                        .wave_samples
                        .keys()
                        .next_back()
                        .copied()
                        .unwrap_or(15)
                        .max(15)
                        + 1;
                    let instrument = self
                        .context
                        .wave_instrument_map
                        .keys()
                        .next_back()
                        .copied()
                        .unwrap_or(15)
                        .max(15)
                        + 1;
                    self.context.wave_samples.insert(index, samples);
                    self.context.wave_instrument_map.insert(instrument, index);
                    s.wave_instrument = instrument;
                    continue;
                }
                "toggle_noise" | "sfx_toggle_noise" => {
                    if is_noise {
                        s.noise_enabled = !s.noise_enabled;
                        if s.noise_enabled && !c.args.is_empty() {
                            kit = Some(arg(c, 0)?);
                        }
                    }
                    continue;
                }
                "assert" | "db" | "sfx_priority_on" | "sfx_priority_off" | "toggle_sfx" => continue,
                "rest" | "note" | "square_note" | "wave_note" | "noise_note" | "drum_note" => {}
                _ => bail!("unsupported audio command {op} on channel {channel}"),
            }
            if op == "drum_note" && (!s.noise_enabled || kit.is_none()) {
                continue;
            }
            let ticks = match op {
                "rest" => arg(c, 0)?,
                "square_note" | "noise_note" => arg(c, 0)? + 1,
                _ => {
                    if c.args.len() > 1 {
                        arg(c, 1)?
                    } else {
                        s.default_length.unwrap_or(4)
                    }
                }
            };
            let tempo = self.tempo(channel, frame, s.tempo);
            let frames = note_frames(ticks, &mut s, tempo);
            let count = sample_count(frames, &mut self.sample_remainder);
            ensure!(
                offset + count <= SAMPLE_RATE * 900,
                "audio channel exceeds 900 seconds"
            );
            let audio = match op {
                "rest" => {
                    if is_pulse {
                        for _ in 0..frames {
                            duty_advance(&mut s);
                        }
                    }
                    vec![0; count]
                }
                "note" => {
                    let base = pitch(c.args.first().context("note missing pitch")?, &s)?;
                    s.vib_delay_count = s.vib_delay;
                    s.vib_latched = base;
                    let mut audio = Vec::new();
                    if is_wave {
                        let segments = vibrato(frames, &mut s, base);
                        let mut phase = PREC / 32.0;
                        let mut used = 0;
                        for (i, (f, reg)) in segments.iter().enumerate() {
                            let take = if i + 1 == segments.len() {
                                count.saturating_sub(used)
                            } else {
                                ratio_rounded(*f, count, frames.max(1))
                            };
                            if take == 0 {
                                continue;
                            }
                            used += take;
                            audio.extend(wave(
                                take,
                                frequency(*reg, true),
                                s.wave_instrument,
                                s.wave_volume,
                                &mut phase,
                                &self.context,
                            )?);
                        }
                    } else {
                        let slide_target = s.slide.take();
                        if channel == 1 {
                            s.sweep_shadow = base;
                            s.pulse_active = true;
                            if s.sweep_enabled {
                                let shift = s.sweep & 7;
                                let next = base
                                    + if s.sweep & 8 != 0 {
                                        -(base >> shift)
                                    } else {
                                        base >> shift
                                    };
                                if shift > 0 && !(0..=2047).contains(&next) {
                                    s.pulse_active = false;
                                }
                            }
                        }
                        pulse_phase = 6.0 * PREC / 8.0;
                        let segments = if let Some((target, duration)) =
                            slide_target.filter(|_| channel == 1)
                        {
                            slide(frames, base, target, duration)
                        } else {
                            sweep(frames, base, &mut s, channel, offset)
                        };
                        let mut used = 0;
                        for (i, (f, reg)) in segments.iter().enumerate() {
                            let take = if i + 1 == segments.len() {
                                count.saturating_sub(used)
                            } else {
                                ratio_rounded(*f, count, frames.max(1))
                            };
                            if take == 0 {
                                continue;
                            }
                            used += take;
                            let Some(reg) = reg else {
                                audio.extend(vec![0.0; take]);
                                continue;
                            };
                            let vibratos = vibrato(*f, &mut s, *reg);
                            let mut vib_used = 0;
                            for (j, (vf, vr)) in vibratos.iter().enumerate() {
                                let vt = if j + 1 == vibratos.len() {
                                    take.saturating_sub(vib_used)
                                } else {
                                    ratio_rounded(*vf, take, (*f).max(1))
                                };
                                if vt == 0 {
                                    continue;
                                }
                                vib_used += vt;
                                self.pulse_chunks(
                                    &mut audio,
                                    vt,
                                    *vf,
                                    frequency(*vr, false),
                                    &mut s,
                                    &mut pulse_phase,
                                );
                            }
                        }
                    }
                    audio.resize(count, 0.0);
                    apply_envelope(&mut audio, s.envelope, offset);
                    audio.into_iter().map(|v| i16_trunc(v as f64)).collect()
                }
                "square_note" => {
                    let reg = (arg(c, 3)? + s.pitch_offset + s.instrument_pitch) & 2047;
                    let mut audio = Vec::new();
                    self.pulse_chunks(
                        &mut audio,
                        count,
                        frames,
                        frequency(reg, false),
                        &mut s,
                        &mut pulse_phase,
                    );
                    apply_envelope(&mut audio, Some((arg(c, 1)?, arg(c, 2)?)), offset);
                    audio.into_iter().map(|v| i16_trunc(v as f64)).collect()
                }
                "wave_note" => {
                    let reg = (TABLE[arg(c, 2)?.clamp(0, 24) as usize]
                        + s.pitch_offset
                        + s.instrument_pitch)
                        & 2047;
                    let mut phase = PREC / 32.0;
                    let mut audio = wave(
                        count,
                        frequency(reg, true) * 0.5,
                        arg(c, 0)?,
                        s.wave_volume,
                        &mut phase,
                        &self.context,
                    )?;
                    apply_envelope(&mut audio, s.envelope, offset);
                    audio.into_iter().map(|v| i16_trunc(v as f64)).collect()
                }
                "noise_note" => noise(
                    count,
                    &NoiseNote {
                        length: arg(c, 0)?,
                        volume: arg(c, 1)?,
                        fade: arg(c, 2)?,
                        frequency: arg(c, 3)?,
                    },
                    &mut s,
                    offset,
                ),
                "drum_note" => {
                    let kit = kit.unwrap();
                    let instrument = arg(c, 0)?;
                    let notes = self
                        .context
                        .drumkits
                        .get(&kit)
                        .and_then(|k| k.get(&instrument))
                        .with_context(|| {
                            format!("missing drumkit {kit} instrument {instrument}")
                        })?;
                    let mut audio = Vec::new();
                    let mut remainder = 0;
                    for (i, note) in notes.iter().enumerate() {
                        let f = if note.length <= 0 {
                            64
                        } else if note.length < 64 {
                            64 - note.length
                        } else {
                            note.length
                        };
                        let take = sample_count(f as usize, &mut remainder);
                        let start = if i == 0 { 0 } else { offset + audio.len() };
                        audio.extend(noise(take, note, &mut s, start));
                    }
                    audio.resize(count, 0);
                    audio
                }
                _ => unreachable!(),
            };
            out.push(Segment { audio, pan: s.pan });
            frame += frames;
            offset += count;
        }
        self.frames.insert(channel, frame);
        self.samples.insert(channel, offset);
        Ok(out)
    }
    fn pulse_chunks(
        &self,
        out: &mut Vec<f32>,
        count: usize,
        frames: usize,
        freq: f64,
        s: &mut State,
        phase: &mut f64,
    ) {
        if s.duty_pattern.is_some() && frames > 0 {
            for i in 0..frames {
                let take = ratio_floor(i + 1, count, frames) - ratio_floor(i, count, frames);
                out.extend(pulse(take, freq, s.duty, phase));
                duty_advance(s);
            }
        } else {
            out.extend(pulse(count, freq, s.duty, phase));
        }
    }
    fn ratio(&self, frames: usize, channel: u8) -> usize {
        let pf = self.frames[&self.primary];
        let ps = self.samples[&self.primary];
        if pf == 0 || ps == 0 {
            return ratio_floor(frames, FRAME_NUM, FRAME_DEN);
        }
        let f = self.frames[&channel].max(1);
        let s = self.samples[&channel];
        let ratio = ratio_ceil(frames, s, f);
        if ratio > 0 {
            ratio
        } else {
            ratio_ceil(frames, ps, pf)
        }
    }
    pub(super) fn sync(&mut self) {
        let max_frames = self.frames.values().copied().max().unwrap_or(0);
        if max_frames == 0 {
            return;
        }
        let numbers = self.channels.keys().copied().collect::<Vec<_>>();
        let mut max_samples = self.samples.values().copied().max().unwrap_or(0);
        for n in &numbers {
            let current = self.frames[n];
            if current >= max_frames {
                continue;
            }
            let mut segs = self.channels[n].clone();
            let mut remaining = max_frames - current;
            if let Some((loop_frame, loop_sample)) =
                self.loops.get(n).copied().filter(|(f, _)| *f < current)
            {
                let body = suffix(&segs, loop_sample);
                let length = current - loop_frame;
                if !body.is_empty() {
                    while remaining > 0 {
                        if remaining >= length {
                            segs.extend(body.clone());
                            remaining -= length;
                        } else {
                            segs.extend(prefix(&body, self.ratio(remaining, *n)));
                            remaining = 0;
                        }
                    }
                }
            }
            if remaining > 0 {
                segs.push(Segment {
                    audio: vec![0; self.ratio(remaining, *n)],
                    pan: [true, true],
                });
            }
            self.frames.insert(*n, max_frames);
            let count = segs.iter().map(|s| s.audio.len()).sum::<usize>();
            self.samples.insert(*n, count);
            max_samples = max_samples.max(count);
            self.channels.insert(*n, segs);
        }
        let target = max_samples.max(ratio_floor(max_frames, FRAME_NUM, FRAME_DEN));
        for n in numbers {
            let segs = self.channels.get_mut(&n).unwrap();
            let count = segs.iter().map(|s| s.audio.len()).sum::<usize>();
            if count < target {
                segs.push(Segment {
                    audio: vec![0; target - count],
                    pan: [true, true],
                });
            }
            self.samples.insert(n, target);
        }
    }
    pub(super) fn mix(&self) -> Vec<i16> {
        let count = self.samples.values().copied().max().unwrap_or(0);
        let mut stereo = vec![0_i32; count * 2];
        for segs in self.channels.values() {
            let mut offset = 0;
            for seg in segs {
                for (i, v) in seg.audio.iter().enumerate() {
                    let master = self
                        .volume
                        .range(..=offset + i)
                        .next_back()
                        .map(|(_, v)| *v)
                        .unwrap_or([7, 7]);
                    for c in 0..2 {
                        let gain = if seg.pan[c] {
                            master[c] as f64 / 7.0
                        } else {
                            0.0
                        };
                        stereo[(offset + i) * 2 + c] += (*v as f64 * gain).round_ties_even() as i32;
                    }
                }
                offset += seg.audio.len();
            }
        }
        let alpha = (1.0 / SAMPLE_RATE as f64)
            / (1.0 / (2.0 * std::f64::consts::PI * 4300.0) + 1.0 / SAMPLE_RATE as f64);
        for channel in 0..2 {
            let mut cap = 0.0;
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                let input = stereo[i * 2 + channel] as f64;
                let output = input - cap;
                cap = input - output * 0.999958;
                values.push(output);
            }
            for _ in 0..2 {
                if let Some(&first) = values.first() {
                    let mut prev = first;
                    for v in values.iter_mut().skip(1) {
                        prev = alpha * *v + (1.0 - alpha) * prev;
                        *v = prev;
                    }
                }
            }
            for (i, v) in values.into_iter().enumerate() {
                stereo[i * 2 + channel] = v.round_ties_even() as i32;
            }
        }
        let peak = stereo.iter().map(|v| v.abs()).max().unwrap_or(0);
        if peak > 32767 {
            let scale = 32767.0 / peak as f64;
            for v in &mut stereo {
                *v = (*v as f64 * scale).round_ties_even() as i32;
            }
        }
        stereo
            .into_iter()
            .map(|v| v.clamp(-32767, 32767) as i16)
            .collect()
    }
}
fn prefix(segs: &[Segment], mut count: usize) -> Vec<Segment> {
    let mut out = Vec::new();
    for s in segs {
        let take = count.min(s.audio.len());
        if take > 0 {
            out.push(Segment {
                audio: s.audio[..take].to_vec(),
                pan: s.pan,
            });
            count -= take;
        }
        if count == 0 {
            break;
        }
    }
    out
}
fn suffix(segs: &[Segment], mut skip: usize) -> Vec<Segment> {
    let mut out = Vec::new();
    for s in segs {
        if skip >= s.audio.len() {
            skip -= s.audio.len();
        } else {
            out.push(Segment {
                audio: s.audio[skip..].to_vec(),
                pan: s.pan,
            });
            skip = 0;
        }
    }
    out
}
