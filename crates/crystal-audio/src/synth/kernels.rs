use super::*;
pub(super) fn i16_trunc(v: f64) -> i16 {
    let nearest = v.round();
    let v = if (v - nearest).abs() < 1e-9 {
        nearest
    } else {
        v
    };
    v.trunc().clamp(-32768.0, 32767.0) as i16
}
fn integral(pattern: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0];
    let mut sum = 0.0_f32;
    for v in pattern {
        sum += v;
        out.push(sum);
    }
    out
}
fn lookup(p: f64, table: &[f32]) -> f64 {
    let i = (p.floor().max(0.0) as usize).min(table.len() - 1);
    let next = (i + 1).min(table.len() - 1);
    table[i] as f64 + (table[next] as f64 - table[i] as f64) * (p - i as f64)
}
fn kernel(count: usize, frequency: f64, phase: &mut f64, pattern: &[f32]) -> Vec<f32> {
    let table = integral(pattern);
    let len = pattern.len() as f64;
    let inc = (frequency * PREC / SAMPLE_RATE as f64).trunc();
    let factor = if inc == 0.0 {
        0.0
    } else {
        PREC / (inc * len) * 28000.0
    };
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let start = *phase * len / PREC;
        let end = (*phase + inc) * len / PREC;
        let value = if end >= len {
            lookup(len, &table) - lookup(start, &table) + lookup(end - len, &table)
                - lookup(0.0, &table)
        } else {
            lookup(end, &table) - lookup(start, &table)
        };
        out.push((value * factor) as f32);
        *phase += inc;
        if *phase >= PREC {
            *phase -= PREC;
        }
    }
    out
}
pub(super) fn pulse(count: usize, freq: f64, duty: i32, phase: &mut f64) -> Vec<f32> {
    let patterns = [
        [0., 0., 0., 0., 0., 0., 0., 1.],
        [1., 0., 0., 0., 0., 0., 0., 1.],
        [1., 0., 0., 0., 0., 1., 1., 1.],
        [0., 1., 1., 1., 1., 1., 1., 0.],
    ];
    kernel(count, freq, phase, &patterns[(duty & 3) as usize])
}
pub(super) fn wave(
    count: usize,
    freq: f64,
    instrument: i32,
    volume: i32,
    phase: &mut f64,
    context: &SynthContext,
) -> Result<Vec<f32>> {
    let shift = [4, 0, 1, 2][(volume & 3) as usize];
    if shift >= 4 {
        return Ok(vec![0.0; count]);
    }
    let index = context
        .wave_instrument_map
        .get(&instrument)
        .copied()
        .unwrap_or(instrument);
    let pattern = context
        .wave_samples
        .get(&index)
        .with_context(|| format!("missing wave instrument {instrument} sample {index}"))?;
    ensure!(pattern.len() == 32, "wave table length must be 32");
    let pattern = pattern
        .iter()
        .map(|v| ((((*v >> shift) & 15) as f64 / 15.0) * 2.0 - 1.0) as f32)
        .collect::<Vec<_>>();
    Ok(kernel(count, freq, phase, &pattern))
}
pub(super) fn envelope(count: usize, env: (i32, i32), offset: usize) -> Vec<f64> {
    let initial = env.0.clamp(0, 15);
    let fade = if env.1.abs() == 8 { 0 } else { env.1 };
    let direction = if fade < 0 { 1 } else { -1 };
    let period = fade.abs() & 7;
    if period == 0 {
        return vec![initial as f64 / 15.0; count];
    }
    let tick = ratio_floor(offset, 512, SAMPLE_RATE);
    let diff = (7 + 8 - tick % 8) % 8;
    let first = tick + if diff == 0 { 8 } else { diff } + (period as usize - 1) * 8;
    (0..count)
        .map(|i| {
            let t = ratio_floor(offset + i, 512, SAMPLE_RATE);
            let changes = if t < first {
                0
            } else {
                1 + (t - first) / (period as usize * 8)
            };
            (initial + direction * changes as i32).clamp(0, 15) as f64 / 15.0
        })
        .collect()
}
pub(super) fn apply_envelope(samples: &mut [f32], env: Option<(i32, i32)>, offset: usize) {
    if let Some(env) = env {
        for (sample, level) in {
            let levels = envelope(samples.len(), env, offset);
            samples.iter_mut().zip(levels)
        } {
            *sample = (*sample as f64 * level) as f32;
        }
    }
}
pub(super) fn noise(count: usize, note: &NoiseNote, state: &mut State, offset: usize) -> Vec<i16> {
    if note.volume <= 0 || count == 0 {
        return vec![0; count];
    }
    if offset == 0 {
        state.lfsr = 0x7fff;
        state.noise_acc = 0.0;
    }
    let shift = (note.frequency >> 4) & 15;
    let width = (note.frequency >> 3) & 1;
    let divisor = [8, 16, 32, 48, 64, 80, 96, 112][(note.frequency & 7) as usize];
    let period = (divisor << (shift + 1)) as f64;
    let cps = 524288.0 / SAMPLE_RATE as f64;
    let levels = envelope(count, (note.volume, note.fade), offset);
    let mut out = Vec::with_capacity(count);
    for level in levels {
        let amp = 28000.0 * level.clamp(0.0, 1.0);
        let mut current = if state.lfsr & 1 == 0 { amp } else { -amp };
        let remaining = period - state.noise_acc;
        if cps <= remaining {
            out.push(i16_trunc(current));
            state.noise_acc += cps;
            continue;
        }
        let mut clocks = cps;
        let mut integrated = 0.0;
        while clocks > 0.0 {
            let remaining = period - state.noise_acc;
            if clocks <= remaining {
                integrated += current * clocks;
                state.noise_acc += clocks;
                clocks = 0.0;
            } else {
                integrated += current * remaining;
                clocks -= remaining;
                state.noise_acc = 0.0;
                let fb = ((state.lfsr & 1) ^ ((state.lfsr >> 1) & 1)) & 1;
                state.lfsr = (state.lfsr >> 1) & 0x7fff;
                state.lfsr = (state.lfsr & !(1 << 14)) | (fb << 14);
                if width != 0 {
                    state.lfsr = (state.lfsr & !(1 << 6)) | (fb << 6);
                }
                current = if state.lfsr & 1 == 0 { amp } else { -amp };
            }
        }
        out.push(i16_trunc(integrated / cps));
    }
    out
}
pub(super) fn duty_advance(s: &mut State) {
    if let Some(pattern) = s.duty_pattern {
        let p = ((pattern << 2) | (pattern >> 6)) & 255;
        s.duty_pattern = Some(p);
        s.duty = (p >> 6) & 3;
    }
}
pub(super) fn vibrato(frames: usize, s: &mut State, base: i32) -> Vec<(usize, i32)> {
    let base = base.clamp(0, 2047);
    let up = (s.vib_extent + 1) / 2;
    let down = s.vib_extent / 2;
    let mut out: Vec<(usize, i32)> = Vec::new();
    if !(1..=2047).contains(&s.vib_latched) {
        s.vib_latched = base;
    }
    for _ in 0..frames {
        if s.vib_delay_count > 0 {
            s.vib_delay_count -= 1;
        } else if up > 0 || down > 0 {
            if s.vib_counter == 0 {
                s.vib_latched = (base & 0x700)
                    | if s.vib_up {
                        ((base & 255) - down).max(0)
                    } else {
                        ((base & 255) + up).min(255)
                    };
                s.vib_up = !s.vib_up;
                s.vib_counter = s.vib_rate & 15;
            } else {
                s.vib_counter = (s.vib_counter - 1) & 15;
            }
        }
        let reg = s.vib_latched.clamp(0, 2047);
        if let Some(last) = out.last_mut().filter(|last| last.1 == reg) {
            last.0 += 1;
        } else {
            out.push((1, reg));
        }
    }
    out
}
pub(super) fn slide(
    frames: usize,
    start: i32,
    target: i32,
    duration: usize,
) -> Vec<(usize, Option<i32>)> {
    let ramp = duration.max(1).min(frames.max(1));
    let direction = if target > start { 1 } else { -1 };
    let distance = (target - start).unsigned_abs() as usize;
    let step = (distance / ramp) as i32;
    let rem = distance % ramp;
    let mut acc = 0;
    let mut current = start;
    let mut done = start == target;
    let mut out: Vec<(usize, Option<i32>)> = Vec::new();
    for i in 0..frames {
        if let Some(last) = out.last_mut().filter(|last| last.1 == Some(current)) {
            last.0 += 1;
        } else {
            out.push((1, Some(current)));
        }
        if i >= ramp || done {
            continue;
        }
        let mut next = current + direction * step;
        acc += rem;
        if acc >= ramp {
            next += direction;
            acc -= ramp;
        }
        if (direction > 0 && next >= target) || (direction < 0 && next <= target) {
            current = target;
            done = true;
        } else {
            current = next.clamp(0, 2047);
        }
    }
    out
}
pub(super) fn sweep(
    frames: usize,
    base: i32,
    s: &mut State,
    channel: u8,
    offset: usize,
) -> Vec<(usize, Option<i32>)> {
    if channel != 1 || frames == 0 || base <= 0 || !s.sweep_enabled {
        return vec![(frames, Some(base))];
    }
    if !s.pulse_active {
        return vec![(frames, None)];
    }
    let shift = s.sweep & 7;
    let time = (s.sweep >> 4) & 7;
    if shift == 0 || time == 0 {
        return vec![(frames, Some(base))];
    }
    let direction = if s.sweep & 8 != 0 { -1 } else { 1 };
    let ticks = time as usize * 4;
    let start_tick = ratio_floor(offset, 512, SAMPLE_RATE);
    let diff = (3 + 4 - start_tick % 4) % 4;
    let first = if diff == 0 { 4 } else { diff };
    let mut rem = 0;
    let mut to_frames = |ticks: usize| {
        let total = ticks * FRAME_DEN + rem;
        rem = total % (70224 * 512);
        total / (70224 * 512)
    };
    let mut until = (to_frames(first) + to_frames(ticks)).max(1);
    let mut shadow = if s.sweep_shadow == 0 {
        base
    } else {
        s.sweep_shadow
    }
    .clamp(0, 2047);
    let mut remaining = frames;
    let mut out = Vec::new();
    while remaining > 0 {
        let take = remaining.min(until);
        out.push((take, Some(shadow)));
        remaining -= take;
        until -= take;
        if remaining == 0 {
            break;
        }
        if until > 0 {
            continue;
        }
        let next = shadow + direction * (shadow >> shift);
        if !(0..=2047).contains(&next) {
            s.pulse_active = false;
            s.sweep_shadow = shadow;
            out.push((remaining, None));
            break;
        }
        shadow = next;
        s.sweep_shadow = shadow;
        until = to_frames(ticks).max(1);
    }
    out
}
