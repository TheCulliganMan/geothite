//! Independent anatomy renderer. Owns no neural dynamics or learned weights.
//! The simulation supplies source positions once and measured activity thereafter.
use crate::*;

#[derive(Clone, Copy, Deserialize)]
#[serde(default)]
struct Style {
    background: [u8; 3],
    inactive: [u8; 3],
    spike: [u8; 3],
    dopamine: [u8; 3],
    spike_radius: u8,
    soma_radius: u8,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            background: [8, 15, 22],
            inactive: [62, 92, 113],
            spike: [110, 255, 188],
            dopamine: [255, 115, 191],
            spike_radius: 2,
            soma_radius: 0,
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
    activity: Vec<bool>,
    connections: Vec<Connection>,
    selected: Option<u32>,
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
                || !(0.0..=7.0).contains(&r[4])
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
        for p in &points {
            positions[p.index as usize] = Some(p.position);
        }
        Ok(Self {
            positions,
            points,
            activity: vec![false; n],
            connections: Vec::new(),
            selected: None,
        })
    }
    pub fn update_activity(&mut self, indices: &[u32]) -> Result<(), String> {
        if indices.iter().any(|&i| i > 400_000) {
            return Err("Invalid activity index".into());
        }
        self.activity.fill(false);
        for &i in indices {
            if let Some(active) = self.activity.get_mut(i as usize) {
                *active = true;
            }
        }
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
        if style.spike_radius > 3 || style.soma_radius > 2 {
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
                let active = self.activity[point.index as usize] && focused;
                if active != active_pass {
                    continue;
                }
                let (x, y, z) = camera.project(point.position);
                let radius = if active {
                    style.spike_radius
                } else {
                    style.soma_radius
                } as i32;
                let base = if active {
                    if point.flags & 2 != 0 {
                        style.dopamine
                    } else {
                        style.spike
                    }
                } else {
                    style.inactive
                };
                let shade = if active {
                    1.0
                } else {
                    ((3.65 - z) * 0.65).clamp(0.30, 1.0) * if focused { 1.0 } else { 0.25 }
                };
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
        for edge in &self.connections {
            let source = self.positions[edge.source as usize];
            let target = self.positions[edge.target as usize];
            if let (Some(source), Some(target)) = (source, target) {
                let (ax, ay, _) = camera.project(source);
                let (bx, by, _) = camera.project(target);
                let active = self.activity[edge.source as usize];
                let color = if active {
                    [110, 255, 188]
                } else if Some(edge.source) == self.selected {
                    [87, 185, 238]
                } else {
                    [185, 140, 244]
                };
                let alpha = if active { 0.85 } else { 0.32 };
                line(&mut pixels, width, height, (ax, ay), (bx, by), color, alpha);
                let (dx, dy) = (bx - ax, by - ay);
                let length = dx.hypot(dy);
                if length > 14.0 {
                    let (ux, uy) = (dx / length, dy / length);
                    let (tx, ty) = (ax + dx * 0.7, ay + dy * 0.7);
                    for side in [-1.0, 1.0] {
                        line(
                            &mut pixels,
                            width,
                            height,
                            (tx, ty),
                            (
                                tx - ux * 5.0 - uy * side * 3.0,
                                ty - uy * 5.0 + ux * side * 3.0,
                            ),
                            color,
                            alpha,
                        );
                    }
                }
                if active && edge.contacts >= 10 {
                    line(
                        &mut pixels,
                        width,
                        height,
                        (ax + 1.0, ay),
                        (bx + 1.0, by),
                        color,
                        0.25,
                    );
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
        _ => Err("Unknown display population".into()),
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
        let mut out = Vec::with_capacity(self.metadata.cells.len() * 5);
        for (i, c) in self.metadata.cells.iter().enumerate() {
            if let Some(p) = c.position {
                let flags = u32::from(self.kc[i])
                    | if c.transmitter == "dopamine" { 2 } else { 0 }
                    | if c.kind.starts_with("MBON") { 4 } else { 0 };
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
        let x = (a.0 + (b.0 - a.0) * t).round() as i32;
        let y = (a.1 + (b.1 - a.1) * t).round() as i32;
        if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
            let j = (y as usize * width as usize + x as usize) * 4;
            for k in 0..3 {
                pixels[j + k] =
                    (pixels[j + k] as f32 * (1.0 - alpha) + color[k] as f32 * alpha) as u8;
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
