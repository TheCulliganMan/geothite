//! Independent anatomy renderer. Owns no neural dynamics or learned weights.
//! The simulation supplies source positions once and measured activity thereafter.
use crate::*;

#[derive(Clone, Deserialize)]
#[serde(default)]
struct Style {
    background: [u8; 3],
    inactive: [u8; 3],
    spike: [u8; 3],
    dopamine: [u8; 3],
    penalty: [u8; 3],
    spike_radius: u8,
    soma_radius: u8,
    color_mode: String,
    show_activity_edges: bool,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            background: [8, 15, 22],
            inactive: [62, 92, 113],
            spike: [110, 255, 188],
            dopamine: [255, 115, 191],
            penalty: [255, 176, 72],
            spike_radius: 2,
            soma_radius: 0,
            color_mode: "anatomy".into(),
            show_activity_edges: true,
        }
    }
}
#[derive(Clone, Copy)]
struct Point {
    position: [f32; 3],
    index: u32,
    flags: u32,
}

/// Source anatomy in a small, separate WASM worker. Render and pick use precisely
/// the same perspective projection. Missing somas are omitted, never fabricated.
#[derive(Deserialize)]
struct Connection {
    source: u32,
    target: u32,
    contacts: u32,
}

#[wasm_bindgen]
pub struct AnatomyView {
    points: Vec<Point>,
    positions: Vec<Option<[f32; 3]>>,
    flags: Vec<u32>,
    activity: Vec<bool>,
    spike_counts: Vec<u32>,
    spike_onsets: Vec<f32>,
    connections: Vec<Connection>,
    activity_connections: Vec<Connection>,
    selected: Option<u32>,
    activity_age: f32,
    motion: bool,
}
#[wasm_bindgen]
impl AnatomyView {
    #[wasm_bindgen(constructor)]
    pub fn new(records: &[f32]) -> Result<AnatomyView, String> {
        if records.len() % 5 != 0
            || records.len() > 2_000_000
            || records.iter().any(|x| !x.is_finite())
        {
            return Err("Invalid anatomy records".into());
        }
        let mut points = Vec::with_capacity(records.len() / 5);
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for r in records.chunks_exact(5) {
            if r[3] < 0.0
                || r[3] > 400_000.0
                || r[3].fract() != 0.0
                || !(0.0..=1023.0).contains(&r[4])
                || r[4].fract() != 0.0
            {
                return Err("Invalid source index or population".into());
            }
            let position = [r[0], r[1], r[2]];
            for a in 0..3 {
                min[a] = min[a].min(position[a]);
                max[a] = max[a].max(position[a]);
            }
            points.push(Point {
                position,
                index: r[3] as u32,
                flags: r[4] as u32,
            });
        }
        let extent = (0..3).map(|a| max[a] - min[a]).fold(1.0, f32::max);
        for p in &mut points {
            for a in 0..3 {
                p.position[a] = (p.position[a] - (min[a] + max[a]) * 0.5) / extent;
            }
        }
        let n = points
            .iter()
            .map(|p| p.index as usize + 1)
            .max()
            .unwrap_or(0);
        let mut positions = vec![None; n];
        let mut flags = vec![0; n];
        for p in &points {
            positions[p.index as usize] = Some(p.position);
            flags[p.index as usize] = p.flags;
        }
        Ok(Self {
            positions,
            flags,
            points,
            activity: vec![false; n],
            spike_counts: vec![0; n],
            spike_onsets: vec![0.0; n],
            connections: Vec::new(),
            activity_connections: Vec::new(),
            selected: None,
            activity_age: 0.0,
            motion: false,
        })
    }
    pub fn update_activity(&mut self, indices: &[u32]) -> Result<(), String> {
        if indices.iter().any(|&i| i > 400_000) {
            return Err("Invalid activity index".into());
        }
        self.activity.fill(false);
        self.spike_counts.fill(0);
        self.spike_onsets.fill(0.0);
        for &i in indices {
            if let Some(active) = self.activity.get_mut(i as usize) {
                *active = true;
                self.spike_counts[i as usize] = 1;
            }
        }
        Ok(())
    }
    /// Interleaved source index/count records from one measured neural window.
    pub fn update_spikes(&mut self, samples: &[u32]) -> Result<(), String> {
        if samples.len() % 2 != 0 || samples.chunks_exact(2).any(|r| r[0] > 400_000) {
            return Err("Invalid spike samples".into());
        }
        self.activity.fill(false);
        self.spike_counts.fill(0);
        self.spike_onsets.fill(0.0);
        for sample in samples.chunks_exact(2) {
            if let Some(count) = self.spike_counts.get_mut(sample[0] as usize) {
                *count = sample[1];
                self.activity[sample[0] as usize] = sample[1] > 0;
            }
        }
        Ok(())
    }
    /// Index, count and measured last-spike phase (millionths of the window).
    /// Replay the last spike once over 800 presentation ms; counts remain the
    /// complete window total. This is not a replay of every spike in a burst.
    pub fn update_timed_spikes(&mut self, samples: &[u32]) -> Result<(), String> {
        if samples.len() % 3 != 0
            || samples
                .chunks_exact(3)
                .any(|r| r[0] > 400_000 || r[2] > 1_000_000)
        {
            return Err("Invalid timed spike samples".into());
        }
        self.activity.fill(false);
        self.spike_counts.fill(0);
        self.spike_onsets.fill(0.0);
        for r in samples.chunks_exact(3) {
            if let Some(count) = self.spike_counts.get_mut(r[0] as usize) {
                *count = r[1];
                self.activity[r[0] as usize] = r[1] > 0;
                self.spike_onsets[r[0] as usize] = r[2] as f32 * 0.8 / 1_000_000.0;
            }
        }
        Ok(())
    }
    pub fn update_activity_connections(&mut self, connections: &str) -> Result<(), String> {
        let edges: Vec<Connection> =
            serde_json::from_str(connections).map_err(|e| e.to_string())?;
        if edges.len() > 384
            || edges.iter().any(|e| {
                e.source as usize >= self.activity.len()
                    || e.target as usize >= self.activity.len()
                    || e.contacts == 0
            })
        {
            return Err("Invalid active connection view".into());
        }
        self.activity_connections = edges;
        Ok(())
    }
    pub fn select(&mut self, index: i32, connections: &str) -> Result<(), String> {
        let edges: Vec<Connection> =
            serde_json::from_str(connections).map_err(|e| e.to_string())?;
        if edges.len() > 256
            || edges.iter().any(|e| {
                e.source as usize >= self.activity.len()
                    || e.target as usize >= self.activity.len()
                    || e.contacts == 0
            })
        {
            return Err("Invalid connection view".into());
        }
        self.selected = (index >= 0).then_some(index as u32);
        self.connections = edges;
        Ok(())
    }
    /// Presentation time only: never advances or fabricates neural activity.
    pub fn animate(&mut self, activity_age: f32, motion: bool) -> Result<(), String> {
        if !activity_age.is_finite() || activity_age < 0.0 {
            return Err("Invalid presentation time".into());
        }
        self.activity_age = activity_age;
        self.motion = motion;
        Ok(())
    }
    pub fn render(
        &self,
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        group: &str,
        style: &str,
    ) -> Result<Vec<u8>, String> {
        let camera = Camera::new(width, height, yaw, pitch, zoom)?;
        let style: Style = serde_json::from_str(style).map_err(|e| e.to_string())?;
        if style.spike_radius > 3
            || style.soma_radius > 2
            || !["anatomy", "activity"].contains(&style.color_mode.as_str())
        {
            return Err("Invalid point radius".into());
        }
        let mask = group_mask(group)?;
        let mut pixels = vec![0; width as usize * height as usize * 4];
        for px in pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&[
                style.background[0],
                style.background[1],
                style.background[2],
                255,
            ]);
        }
        let mut depth = vec![f32::INFINITY; width as usize * height as usize];
        // Activity is an explicit luminous overlay, so deep active cells remain
        // visible. Anatomy itself is depth-tested. Never invent activity.
        for active_pass in [false, true] {
            for point in &self.points {
                let focused = mask == 0 || point.flags & mask != 0;
                let age = self.activity_age - self.spike_onsets[point.index as usize];
                let active =
                    self.activity[point.index as usize] && focused && (!self.motion || age >= 0.0);
                let flash = if self.motion {
                    (-age.max(0.0) * 1.8).exp()
                } else {
                    1.0
                };
                if active != active_pass {
                    continue;
                }
                let (x, y, z) = camera.project(point.position);
                let radius = if active {
                    style.spike_radius
                } else if point.flags & 768 != 0 && focused {
                    style.soma_radius.max(1)
                } else {
                    style.soma_radius
                } as i32;
                let base = if active {
                    style.spike
                } else if style.color_mode == "anatomy" {
                    population_color(point.flags, style.inactive)
                } else {
                    style.inactive
                };
                let shade = if active {
                    let strength =
                        (self.spike_counts[point.index as usize] as f32).ln_1p() / 16f32.ln_1p();
                    (0.35 + 0.65 * strength.min(1.0)) * (0.35 + 0.65 * flash)
                } else {
                    ((3.65 - z) * 0.65).clamp(0.30, 1.0) * if focused { 1.0 } else { 0.25 }
                };
                if active && flash > 0.02 {
                    glow(
                        &mut pixels,
                        width,
                        height,
                        (x, y),
                        radius as f32 + 4.0,
                        base,
                        flash * 0.28,
                    );
                }
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if radius > 1 && dx * dx + dy * dy > radius * radius {
                            continue;
                        }
                        let (sx, sy) = (x.round() as i32 + dx, y.round() as i32 + dy);
                        if sx < 0 || sy < 0 || sx >= width as i32 || sy >= height as i32 {
                            continue;
                        }
                        let i = sy as usize * width as usize + sx as usize;
                        if active || z < depth[i] {
                            if !active {
                                depth[i] = z;
                            }
                            for k in 0..3 {
                                pixels[i * 4 + k] = (base[k] as f32 * shade) as u8;
                            }
                        }
                    }
                }
            }
        }
        for edge in self
            .activity_connections
            .iter()
            .filter(|_| style.show_activity_edges)
            .chain(&self.connections)
        {
            if mask != 0
                && self.flags[edge.source as usize] & mask == 0
                && self.flags[edge.target as usize] & mask == 0
            {
                continue;
            }
            let source = self.positions[edge.source as usize];
            let target = self.positions[edge.target as usize];
            if let (Some(source), Some(target)) = (source, target) {
                // Curves are schematic: endpoints are the measured soma coordinates.
                // A direction-dependent bend separates reciprocal connections.
                let midpoint: [f32; 3] = std::array::from_fn(|i| (source[i] + target[i]) * 0.5);
                let delta = std::array::from_fn::<_, 3, _>(|i| target[i] - source[i]);
                let bend = [delta[1] * 0.22, -delta[0] * 0.22, delta[0] * 0.12];
                let control: [f32; 3] = std::array::from_fn(|i| midpoint[i] + bend[i]);
                let curve = |t: f32| {
                    let p = std::array::from_fn(|i| {
                        (1.0 - t).powi(2) * source[i]
                            + 2.0 * (1.0 - t) * t * control[i]
                            + t * t * target[i]
                    });
                    let (x, y, _) = camera.project(p);
                    (x, y)
                };
                let age = self.activity_age - self.spike_onsets[edge.source as usize];
                let active = self.activity[edge.source as usize] && (!self.motion || age >= 0.0);
                let flash = if self.motion {
                    (-age.max(0.0) * 1.8).exp()
                } else {
                    1.0
                };
                let signal = if self.flags[edge.source as usize] & 8 != 0 {
                    style.penalty
                } else if self.flags[edge.source as usize] & 2 != 0 {
                    style.dopamine
                } else {
                    style.spike
                };
                let color = if active {
                    signal
                } else if Some(edge.source) == self.selected {
                    [87, 185, 238]
                } else {
                    [185, 140, 244]
                };
                let strength = (edge.contacts as f32).ln_1p() / 8.0;
                let selected_link =
                    self.selected == Some(edge.source) || self.selected == Some(edge.target);
                let alpha = if selected_link {
                    0.22 + strength.min(1.0) * 0.38
                } else {
                    0.04 + strength.min(1.0) * 0.10
                };
                let mut previous = curve(0.0);
                for step in 1..=40 {
                    let next = curve(step as f32 / 40.0);
                    line(&mut pixels, width, height, previous, next, color, alpha);
                    if selected_link && edge.contacts >= 10 {
                        line(
                            &mut pixels,
                            width,
                            height,
                            (previous.0, previous.1 + 1.0),
                            (next.0, next.1 + 1.0),
                            color,
                            alpha * 0.25,
                        );
                    }
                    previous = next;
                }
                let tip = curve(0.78);
                let before = curve(0.75);
                let (dx, dy) = (tip.0 - before.0, tip.1 - before.1);
                let length = dx.hypot(dy);
                if selected_link && length > 0.1 {
                    let (ux, uy) = (dx / length, dy / length);
                    for side in [-1.0, 1.0] {
                        line(
                            &mut pixels,
                            width,
                            height,
                            tip,
                            (
                                tip.0 - ux * 6.0 - uy * side * 3.0,
                                tip.1 - uy * 6.0 + ux * side * 3.0,
                            ),
                            color,
                            0.85,
                        );
                    }
                }
                // One short traveling trace per measured activity window. Its
                // speed is illustrative, not measured transmission timing.
                if active && self.motion && age < 1.4 {
                    let head = (age / 1.1).min(1.0);
                    for i in 0..12 {
                        let t = head - i as f32 * 0.012;
                        if t < 0.0 {
                            break;
                        }
                        glow(
                            &mut pixels,
                            width,
                            height,
                            curve(t),
                            2.8,
                            signal,
                            (1.0 - i as f32 / 12.0) * flash,
                        );
                    }
                } else if active && !self.motion {
                    glow(&mut pixels, width, height, curve(0.5), 3.5, signal, 0.9);
                }
            }
        }
        if let Some(p) = self
            .selected
            .and_then(|index| self.points.iter().find(|p| p.index == index))
        {
            let (x, y, _) = camera.project(p.position);
            for i in 0..32 {
                let a = i as f32 * std::f32::consts::TAU / 32.0;
                let b = (i + 1) as f32 * std::f32::consts::TAU / 32.0;
                line(
                    &mut pixels,
                    width,
                    height,
                    (x + a.cos() * 7.0, y + a.sin() * 7.0),
                    (x + b.cos() * 7.0, y + b.sin() * 7.0),
                    [238, 249, 255],
                    1.0,
                );
            }
        }
        Ok(pixels)
    }
    /// Return an original simulation index; the simulation owns source metadata.
    pub fn pick(
        &self,
        width: u32,
        height: u32,
        x: f32,
        y: f32,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        group: &str,
    ) -> Result<i32, String> {
        let camera = Camera::new(width, height, yaw, pitch, zoom)?;
        let mask = group_mask(group)?;
        if !x.is_finite() || !y.is_finite() {
            return Err("Invalid cursor".into());
        }
        let mut best = None;
        let mut distance = 64.0;
        for p in &self.points {
            if mask != 0 && p.flags & mask == 0 {
                continue;
            }
            let (sx, sy, z) = camera.project(p.position);
            let d = (sx - x).powi(2) + (sy - y).powi(2);
            // Within one pixel choose the frontmost soma, otherwise nearest.
            if d < distance
                || (d - distance).abs() < 1.0 && best.is_some_and(|(_, depth)| z < depth)
            {
                distance = d;
                best = Some((p.index, z));
            }
        }
        Ok(best.map_or(-1, |(i, _)| i as i32))
    }
}
fn group_mask(group: &str) -> Result<u32, String> {
    match group {
        "all" => Ok(0),
        "KC" => Ok(1),
        "DAN" => Ok(2),
        "MBON" => Ok(4),
        "sensory" => Ok(16),
        "descending" => Ok(32),
        "optic" => Ok(64),
        "intrinsic" => Ok(128),
        "input" => Ok(256),
        "readout" => Ok(512),
        _ => Err("Unknown display population".into()),
    }
}
fn population_color(flags: u32, fallback: [u8; 3]) -> [u8; 3] {
    if flags & 256 != 0 {
        [70, 211, 235]
    } else if flags & 512 != 0 {
        [240, 151, 97]
    } else if flags & 16 != 0 {
        [71, 146, 180]
    } else if flags & 32 != 0 {
        [192, 140, 99]
    } else if flags & 64 != 0 {
        [91, 115, 157]
    } else if flags & 1 != 0 {
        [124, 157, 122]
    } else if flags & 4 != 0 {
        [182, 134, 193]
    } else if flags & 2 != 0 {
        [215, 117, 173]
    } else if flags & 128 != 0 {
        [131, 130, 168]
    } else {
        fallback
    }
}
struct Camera {
    width: f32,
    height: f32,
    scale: f32,
    cy: f32,
    sy: f32,
    cp: f32,
    sp: f32,
}
impl Camera {
    fn new(width: u32, height: u32, yaw: f32, pitch: f32, zoom: f32) -> Result<Self, String> {
        if width == 0
            || height == 0
            || width > 1600
            || height > 1600
            || !yaw.is_finite()
            || !pitch.is_finite()
            || !zoom.is_finite()
            || !(0.4..=4.0).contains(&zoom)
        {
            return Err("Invalid view".into());
        }
        Ok(Self {
            width: width as f32,
            height: height as f32,
            scale: width.min(height) as f32 * 0.82 * zoom,
            cy: yaw.cos(),
            sy: yaw.sin(),
            cp: pitch.cos(),
            sp: pitch.sin(),
        })
    }
    fn project(&self, p: [f32; 3]) -> (f32, f32, f32) {
        let x = p[0] * self.cy + p[2] * self.sy;
        let z = -p[0] * self.sy + p[2] * self.cy;
        let y = p[1] * self.cp - z * self.sp;
        let depth = 3.0 + p[1] * self.sp + z * self.cp;
        (
            self.width * 0.5 + x * self.scale * 3.0 / depth,
            self.height * 0.5 + y * self.scale * 3.0 / depth,
            depth,
        )
    }
}
#[wasm_bindgen]
impl Brain {
    pub fn view_anatomy(&self) -> Vec<f32> {
        let inputs: std::collections::BTreeSet<_> = self
            .wiring
            .as_ref()
            .map(|w| w.inputs.iter().copied().collect())
            .unwrap_or_default();
        let outputs: std::collections::BTreeSet<_> = self
            .wiring
            .as_ref()
            .map(|w| w.outputs.iter().copied().collect())
            .unwrap_or_default();
        let mut out = Vec::with_capacity(self.metadata.cells.len() * 5);
        for (i, c) in self.metadata.cells.iter().enumerate() {
            if let Some(p) = c.position {
                let flags = u32::from(self.kc[i])
                    | if c.transmitter == "dopamine" { 2 } else { 0 }
                    | if c.kind.starts_with("MBON") { 4 } else { 0 }
                    | if c.kind.starts_with("PPL") { 8 } else { 0 }
                    | if matches!(c.class.as_str(), "cb_sensory" | "visual_projection") {
                        16
                    } else {
                        0
                    }
                    | if c.class == "descending_neuron" {
                        32
                    } else {
                        0
                    }
                    | if c.class.starts_with("optic") { 64 } else { 0 }
                    | if c.class == "cb_intrinsic" { 128 } else { 0 }
                    | if inputs.contains(&i) { 256 } else { 0 }
                    | if outputs.contains(&i) { 512 } else { 0 };
                out.extend_from_slice(&[p[0], p[1], p[2], i as f32, flags as f32]);
            }
        }
        out
    }
    pub fn view_activity(&self) -> Vec<u32> {
        self.counts
            .iter()
            .enumerate()
            .filter_map(|(i, &n)| (n > 0).then_some(i as u32))
            .collect()
    }
    pub fn view_spikes(&self) -> Vec<u32> {
        self.counts
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .flat_map(|(i, &n)| [i as u32, n])
            .collect()
    }
    pub fn view_timed_spikes(&self) -> Vec<u32> {
        let start = self.tick.saturating_sub(self.last_window_steps as u64);
        let duration = u64::from(self.last_window_steps).max(1);
        self.counts
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .flat_map(|(i, &n)| {
                let offset = self.last_spike[i].saturating_sub(start).min(duration);
                [i as u32, n, (offset * 1_000_000 / duration) as u32]
            })
            .collect()
    }
    pub fn prepare_circuit(&mut self) -> Result<(), String> {
        if self.config.operant.is_some() {
            self.ensure_wiring()?;
        }
        Ok(())
    }
    /// Strongest outgoing anatomical links from neurons that fired in the measured window.
    pub fn view_activity_connections(&self) -> Result<String, String> {
        let mut edges = std::collections::BinaryHeap::new();
        for (source, &spikes) in self.counts.iter().enumerate() {
            if spikes == 0 || self.metadata.cells[source].position.is_none() {
                continue;
            }
            for edge in self.offsets[source] as usize..self.offsets[source + 1] as usize {
                let target = self.targets[edge] as usize;
                if self.metadata.cells[target].position.is_some() {
                    edges.push(std::cmp::Reverse((self.contacts[edge], source, target)));
                    if edges.len() > 384 {
                        edges.pop();
                    }
                }
            }
        }
        let mut edges: Vec<_> = edges.into_iter().map(|e| e.0).collect();
        edges.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        serde_json::to_string(&edges.into_iter().map(|(contacts, source, target)| {
            serde_json::json!({"source":source,"target":target,"contacts":contacts})
        }).collect::<Vec<_>>()).map_err(|e| e.to_string())
    }
}

/// Soft radial falloff keeps luminous cells legible without square sprites.
fn glow(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    center: (f32, f32),
    radius: f32,
    color: [u8; 3],
    alpha: f32,
) {
    let r = radius.ceil() as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let (x, y) = (center.0.round() as i32 + dx, center.1.round() as i32 + dy);
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }
            let falloff = (1.0 - (dx * dx + dy * dy) as f32 / (radius * radius))
                .max(0.0)
                .powi(2)
                * alpha;
            let j = (y as usize * width as usize + x as usize) * 4;
            for k in 0..3 {
                pixels[j + k] = (pixels[j + k] as f32 + color[k] as f32 * falloff).min(255.0) as u8;
            }
        }
    }
}

fn line(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    a: (f32, f32),
    b: (f32, f32),
    color: [u8; 3],
    alpha: f32,
) {
    let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil() as usize).clamp(1, 6400);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.0 + (b.0 - a.0) * t;
        let y = a.1 + (b.1 - a.1) * t;
        let (ix, iy) = (x.floor() as i32, y.floor() as i32);
        let (fx, fy) = (x - x.floor(), y - y.floor());
        // Subpixel coverage prevents thin paths shimmering during an orbit.
        for (dx, dy, coverage) in [
            (0, 0, (1.0 - fx) * (1.0 - fy)),
            (1, 0, fx * (1.0 - fy)),
            (0, 1, (1.0 - fx) * fy),
            (1, 1, fx * fy),
        ] {
            let (sx, sy) = (ix + dx, iy + dy);
            if sx >= 0 && sy >= 0 && sx < width as i32 && sy < height as i32 {
                let j = (sy as usize * width as usize + sx as usize) * 4;
                let opacity = alpha * coverage;
                for k in 0..3 {
                    pixels[j + k] =
                        (pixels[j + k] as f32 * (1.0 - opacity) + color[k] as f32 * opacity) as u8;
                }
            }
        }
    }
}
#[wasm_bindgen]
impl Brain {
    /// Strongest real soma-to-soma links in each direction, bounded for clarity.
    pub fn view_connections(&self, index: u32) -> Result<String, String> {
        let selected = index as usize;
        if selected >= self.metadata.cells.len() {
            return Err("Unknown neuron".into());
        }
        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();
        let mut omitted = 0;
        for source in 0..self.metadata.cells.len() {
            for e in self.offsets[source] as usize..self.offsets[source + 1] as usize {
                let target = self.targets[e] as usize;
                if source != selected && target != selected {
                    continue;
                }
                if self.metadata.cells[source].position.is_none()
                    || self.metadata.cells[target].position.is_none()
                {
                    omitted += 1;
                    continue;
                }
                let entry = (self.contacts[e], source, target, e);
                if source == selected {
                    outgoing.push(entry);
                } else {
                    incoming.push(entry);
                }
            }
        }
        let total_in = incoming.len();
        let total_out = outgoing.len();
        incoming.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        outgoing.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.2.cmp(&b.2)));
        let edges:Vec<_>=incoming.into_iter().take(48).chain(outgoing.into_iter().take(48)).map(|(contacts,source,target,e)|serde_json::json!({"source":source,"target":target,"contacts":contacts,"weight_mv":self.weights[e]})).collect();
        serde_json::to_string(&serde_json::json!({"index":index,"cell":self.metadata.cells[selected],"edges":edges,"incoming":total_in,"outgoing":total_out,"omitted_missing_somas":omitted,"limit_per_direction":48,"geometry":"schematic soma-to-soma; not reconstructed axons","highlight":"source neuron spiked in the measured window; not measured synaptic transmission"})).map_err(|e|e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renderer_accepts_emitted_population_flags() {
        // PPL dopamine cells emit 8|2, the flags that broke full-data startup.
        let mut view = AnatomyView::new(&[
            0., 0., 0., 0., 10., 1., 0., 0., 1., 272., 0., 1., 0., 2., 544.,
        ])
        .unwrap();
        view.update_spikes(&[0, 4, 2, 12]).unwrap();
        for group in ["all", "DAN", "input", "readout", "sensory", "descending"] {
            assert_eq!(
                view.render(80, 80, 0., 0., 1., group, "{}").unwrap().len(),
                80 * 80 * 4
            );
        }
    }
    fn fixture() -> AnatomyView {
        let mut view = AnatomyView::new(&[-1., 0., 0., 0., 1., 1., 0., 0., 1., 4.]).unwrap();
        view.select(
            0,
            r#"[{"source":0,"target":1,"contacts":24},{"source":1,"target":0,"contacts":8}]"#,
        )
        .unwrap();
        view
    }
    fn frame(view: &AnatomyView) -> Vec<u8> {
        view.render(240, 240, 0., 0., 1., "all", "{}").unwrap()
    }
    #[test]
    fn animation_requires_measured_activity_and_settles() {
        let mut view = fixture();
        view.animate(0.1, true).unwrap();
        let idle = frame(&view);
        view.animate(0.6, true).unwrap();
        assert_eq!(idle, frame(&view), "idle links must never invent spikes");
        view.update_activity(&[0]).unwrap();
        view.animate(0.1, true).unwrap();
        let early = frame(&view);
        view.animate(0.6, true).unwrap();
        assert_ne!(
            early,
            frame(&view),
            "measured source activity travels along its links"
        );
        view.update_activity(&[]).unwrap();
        assert_eq!(idle, frame(&view));
    }
    #[test]
    fn timed_activity_respects_order_and_reduced_motion() {
        let mut view = fixture();
        view.animate(0.0, true).unwrap();
        let idle = frame(&view);
        view.update_timed_spikes(&[0, 4, 500_000, 1, 2, 1_000_000])
            .unwrap();
        assert_eq!(idle, frame(&view), "no spikes before measured onset");
        view.animate(0.5, true).unwrap();
        assert_ne!(idle, frame(&view));
        view.animate(0.0, false).unwrap();
        let summary = frame(&view);
        view.animate(10.0, false).unwrap();
        assert_eq!(summary, frame(&view));
        assert!(view.update_timed_spikes(&[0, 1, 1_000_001]).is_err());
        assert_eq!(
            summary,
            frame(&view),
            "bad samples must not erase valid activity"
        );
        view.update_timed_spikes(&[]).unwrap();
        assert_eq!(idle, frame(&view));
    }
    #[test]
    fn reduced_motion_is_static_and_time_is_validated() {
        let mut view = fixture();
        view.update_activity(&[0]).unwrap();
        view.animate(0.1, false).unwrap();
        let still = frame(&view);
        view.animate(10., false).unwrap();
        assert_eq!(still, frame(&view));
        assert!(view.animate(f32::NAN, true).is_err());
        assert!(view.animate(-1., true).is_err());
    }
    #[test]
    fn animated_projection_keeps_picking_on_source_somas() {
        let mut view = fixture();
        view.animate(0.3, true).unwrap();
        for yaw in [0., 0.8, -1.2] {
            let camera = Camera::new(240, 240, yaw, 0.2, 1.4).unwrap();
            let (x, y, _) = camera.project(view.positions[0].unwrap());
            assert_eq!(view.pick(240, 240, x, y, yaw, 0.2, 1.4, "all").unwrap(), 0);
        }
    }
}
