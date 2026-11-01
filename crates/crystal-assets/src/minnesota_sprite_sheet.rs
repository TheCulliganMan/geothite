//! Convert independently drawn ImageGen front/back views into native frames.
//! Conversion does not invent camera angles, animation or pack registration.
use std::io::Read;

use anyhow::{Context, Result, bail, ensure};
use flate2::Crc;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};

use super::minnesota_pokemon_pixel_art::{bitplanes, colored, png_rgb};

/// An opaque, noninterlaced RGB/RGBA8 PNG decoded without external image tools.
#[derive(Debug)]
pub struct MinnesotaSheet {
    pub width: usize,
    pub height: usize,
    pixels: Vec<[u8; 3]>,
    alpha: Vec<u8>,
}

/// A reviewed sheet cell and its maximum native silhouette size.
#[derive(Debug, Clone, Copy)]
pub struct SpriteCrop {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub fit_width: usize,
    pub fit_height: usize,
}

/// The separately authored battle view and its native Crystal canvas.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpriteView {
    Front,
    Back,
}

/// Whether native silhouette edges require independent source-black evidence.
#[derive(Debug, Default, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExteriorContour {
    /// Preserve the original source-evidence converter for existing artwork.
    #[default]
    SourceBacked,
    /// Ink covered boundaries and retain narrow true-black source contours.
    Solid,
}

/// How source ink is sampled inside each native pixel's footprint.
#[derive(Debug, Default, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpriteSampling {
    /// Preserve the existing area vote and authored highlight thresholds.
    #[default]
    AreaVote,
    /// Quantize the center source pixel without white-highlight priority.
    /// Intended for enlarged pixel art whose ink clusters already match the
    /// native grid. Exterior contours still follow `ExteriorContour`.
    SourceCenter,
}

/// Explicit conversion settings; existing callers retain the area-vote default.
#[derive(Debug, Clone, Copy)]
pub struct SpriteConversionOptions {
    /// Enclosed white priority threshold, in 64-sample units, for area voting.
    pub minimum_white_samples: u16,
    /// Interior black threshold, in 64-sample units, for area voting.
    pub minimum_black_samples: u16,
    /// Source-backed or covered-silhouette exterior contour policy.
    pub exterior_contour: ExteriorContour,
    /// Interior source sampling policy.
    pub sampling: SpriteSampling,
}

impl ExteriorContour {
    /// Identify the indexed-frame policy recorded by conversion and review.
    pub fn outline_profile(self) -> &'static str {
        match self {
            Self::SourceBacked => "source-contour-v1",
            Self::Solid => "solid-exterior-v1",
        }
    }
}

impl SpriteView {
    /// Native front frames use seven tiles per side; backs use six.
    pub fn canvas(self) -> usize {
        match self {
            Self::Front => 56,
            Self::Back => 48,
        }
    }

    /// File suffix identifying this authored view.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Back => "back",
        }
    }
}

/// One static frame from an explicitly selected, separately drawn source view.
#[derive(Debug, Serialize)]
pub struct MinnesotaSpriteCandidate {
    pub outline_profile: &'static str,
    pub sampling: SpriteSampling,
    pub view: SpriteView,
    pub size: [usize; 2],
    pub source_bounds: [usize; 4],
    pub palette_rgb555: [u16; 4],
    pub rows: Vec<String>,
    #[serde(skip)]
    indices: Vec<u8>,
}

impl MinnesotaSheet {
    /// Decode a CRC-checked, noninterlaced 8-bit RGB/RGBA PNG.
    /// Returns errors for unsupported formats, corrupt chunks, oversized images,
    /// invalid filters or a decompressed byte count inconsistent with the header.
    pub fn from_png(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "not a PNG");
        let mut position = 8;
        let mut header = None;
        let mut compressed = Vec::new();
        let mut ended = false;
        while position < bytes.len() {
            let chunk = bytes
                .get(position..position + 8)
                .context("truncated PNG chunk")?;
            let count = usize::try_from(u32::from_be_bytes(chunk[..4].try_into()?))?;
            let end = position
                .checked_add(count)
                .and_then(|n| n.checked_add(12))
                .context("PNG chunk overflow")?;
            let payload = bytes
                .get(position + 8..end - 4)
                .context("truncated PNG payload")?;
            let expected =
                u32::from_be_bytes(bytes.get(end - 4..end).context("PNG CRC")?.try_into()?);
            let mut crc = Crc::new();
            crc.update(&chunk[4..8]);
            crc.update(payload);
            ensure!(crc.sum() == expected, "PNG CRC mismatch");
            match &chunk[4..8] {
                b"IHDR" => {
                    ensure!(
                        header.is_none() && position == 8 && payload.len() == 13,
                        "invalid PNG header order"
                    );
                    let width = usize::try_from(u32::from_be_bytes(payload[..4].try_into()?))?;
                    let height = usize::try_from(u32::from_be_bytes(payload[4..8].try_into()?))?;
                    ensure!(
                        width > 0
                            && height > 0
                            && width <= 8192
                            && height <= 8192
                            && width * height <= 16_777_216,
                        "PNG size out of bounds"
                    );
                    ensure!(
                        payload[8] == 8
                            && matches!(payload[9], 2 | 6)
                            && payload[10..] == [0, 0, 0],
                        "requires noninterlaced RGB/RGBA8 PNG"
                    );
                    header = Some((width, height, if payload[9] == 2 { 3 } else { 4 }));
                }
                b"IDAT" => {
                    ensure!(
                        header.is_some() && compressed.len() + payload.len() <= 67_108_864,
                        "PNG IDAT size/order"
                    );
                    compressed.extend_from_slice(payload);
                }
                b"IEND" => {
                    ensure!(payload.is_empty(), "invalid IEND");
                    ended = true;
                    break;
                }
                _ => {}
            }
            position = end;
        }
        ensure!(ended, "PNG missing IEND");
        let (width, height, channels) = header.context("PNG missing header")?;
        let stride = width * channels;
        let expected = (stride + 1) * height;
        let mut filtered = Vec::new();
        ZlibDecoder::new(compressed.as_slice())
            .take(u64::try_from(expected + 1)?)
            .read_to_end(&mut filtered)?;
        ensure!(filtered.len() == expected, "PNG scanline count mismatch");
        let mut raw = vec![0_u8; height * stride];
        for y in 0..height {
            let row = &filtered[y * (stride + 1)..(y + 1) * (stride + 1)];
            for x in 0..stride {
                let index = y * stride + x;
                let left = if x >= channels {
                    raw[index - channels]
                } else {
                    0
                };
                let up = if y > 0 { raw[index - stride] } else { 0 };
                let corner = if y > 0 && x >= channels {
                    raw[index - stride - channels]
                } else {
                    0
                };
                let predictor = match row[0] {
                    0 => 0,
                    1 => left,
                    2 => up,
                    3 => ((u16::from(left) + u16::from(up)) / 2) as u8,
                    4 => paeth(left, up, corner),
                    _ => bail!("unsupported PNG filter"),
                };
                raw[index] = row[x + 1].wrapping_add(predictor);
            }
        }
        let alpha = raw
            .chunks_exact(channels)
            .map(|pixel| if channels == 4 { pixel[3] } else { 255 })
            .collect();
        let pixels = raw
            .chunks_exact(channels)
            .map(|pixel| {
                let alpha = if channels == 4 {
                    u32::from(pixel[3])
                } else {
                    255
                };
                std::array::from_fn(|i| {
                    ((u32::from(pixel[i]) * alpha + 255 * (255 - alpha) + 127) / 255) as u8
                })
            })
            .collect();
        Ok(Self {
            width,
            height,
            pixels,
            alpha,
        })
    }

    /// Read an in-bounds RGB pixel; out-of-image positions return `None`.
    pub fn pixel(&self, x: usize, y: usize) -> Option<[u8; 3]> {
        if x < self.width && y < self.height {
            Some(self.pixels[y * self.width + x])
        } else {
            None
        }
    }

    /// Read original source alpha; out-of-image positions return `None`.
    pub fn alpha(&self, x: usize, y: usize) -> Option<u8> {
        if x < self.width && y < self.height {
            Some(self.alpha[y * self.width + x])
        } else {
            None
        }
    }

    /// Save an unchanged source-resolution cell before palette conversion.
    /// Returns an error for empty or out-of-bounds crops.
    pub fn crop_png(&self, crop: SpriteCrop) -> Result<Vec<u8>> {
        ensure!(
            crop.width > 0
                && crop.height > 0
                && crop
                    .x
                    .checked_add(crop.width)
                    .is_some_and(|n| n <= self.width)
                && crop
                    .y
                    .checked_add(crop.height)
                    .is_some_and(|n| n <= self.height),
            "crop outside sheet"
        );
        let mut pixels = Vec::with_capacity(crop.width * crop.height);
        for y in crop.y..crop.y + crop.height {
            pixels.extend_from_slice(
                &self.pixels[y * self.width + crop.x..y * self.width + crop.x + crop.width],
            );
        }
        png_rgb(crop.width, crop.height, &pixels)
    }

    /// Convert an authored front view into a 56x56 palette-indexed frame.
    /// Returns errors for invalid crops, empty cells or invalid palettes.
    pub fn front_candidate(
        &self,
        crop: SpriteCrop,
        palette: [u16; 4],
    ) -> Result<MinnesotaSpriteCandidate> {
        self.candidate_with_highlight_threshold(crop, palette, SpriteView::Front, 16)
    }

    /// Convert a separately authored rear view into a native 48x48 frame.
    /// Returns errors for invalid crops, empty cells or invalid palettes.
    pub fn back_candidate(
        &self,
        crop: SpriteCrop,
        palette: [u16; 4],
    ) -> Result<MinnesotaSpriteCandidate> {
        self.candidate_with_highlight_threshold(crop, palette, SpriteView::Back, 16)
    }

    /// Area-vote a view with an explicit threshold for small white highlights.
    /// The ordinary threshold is 16 of 64 samples. A smaller authored value
    /// preserves tiny eye dots without changing other species. Returns errors
    /// for thresholds outside 1..=64, invalid crops or invalid palettes.
    /// Interior black requires a majority vote; source-backed exterior contours
    /// retain thin black rims separately from colored interior shading.
    pub fn candidate_with_highlight_threshold(
        &self,
        crop: SpriteCrop,
        palette: [u16; 4],
        view: SpriteView,
        minimum_white_samples: u16,
    ) -> Result<MinnesotaSpriteCandidate> {
        self.candidate_with_thresholds(crop, palette, view, minimum_white_samples, 32)
    }

    /// Area-vote a view with independently authored white-highlight and interior
    /// black thresholds, each expressed in 64-sample units. The ordinary black
    /// threshold is 32; a lower value can retain narrow structural shell seams.
    /// Exterior contour handling is independent of this interior threshold.
    /// Returns errors for thresholds outside 1..=64 or invalid crops/palettes.
    pub fn candidate_with_thresholds(
        &self,
        crop: SpriteCrop,
        palette: [u16; 4],
        view: SpriteView,
        minimum_white_samples: u16,
        minimum_black_samples: u16,
    ) -> Result<MinnesotaSpriteCandidate> {
        self.candidate_with_thresholds_and_contour(
            crop,
            palette,
            view,
            minimum_white_samples,
            minimum_black_samples,
            ExteriorContour::SourceBacked,
        )
    }

    /// Area-vote an authored frame with an explicit exterior contour policy.
    ///
    /// `Solid` inks only covered foreground pixels adjacent to exterior paper.
    /// Narrow neutral-black source seams also survive area voting; predominantly
    /// enclosed white regions keep their white ink rather than losing to those
    /// seams. It does not dilate the body, fill enclosed white features or cap
    /// the deliberately cropped bottom of a rear portrait. Existing artwork
    /// keeps its original conversion unless this policy is explicitly selected.
    /// Returns errors for thresholds outside 1..=64 or invalid crops/palettes.
    pub fn candidate_with_thresholds_and_contour(
        &self,
        crop: SpriteCrop,
        palette: [u16; 4],
        view: SpriteView,
        minimum_white_samples: u16,
        minimum_black_samples: u16,
        exterior_contour: ExteriorContour,
    ) -> Result<MinnesotaSpriteCandidate> {
        self.candidate_with_conversion(
            crop,
            palette,
            view,
            SpriteConversionOptions {
                minimum_white_samples,
                minimum_black_samples,
                exterior_contour,
                sampling: SpriteSampling::AreaVote,
            },
        )
    }

    /// Convert an authored view with explicit sampling and contour policies.
    /// `SourceCenter` reads the center of each native footprint, with integer
    /// floor rounding and no highlight override. It does not paint or relocate
    /// features. The contour pass still uses source coverage and black evidence.
    /// Returns errors for invalid crops, palettes, fits or thresholds outside
    /// 1..=64, including thresholds unused by source-center sampling.
    pub fn candidate_with_conversion(
        &self,
        crop: SpriteCrop,
        palette: [u16; 4],
        view: SpriteView,
        options: SpriteConversionOptions,
    ) -> Result<MinnesotaSpriteCandidate> {
        let SpriteConversionOptions {
            minimum_white_samples,
            minimum_black_samples,
            exterior_contour,
            sampling,
        } = options;
        ensure!(
            (1..=64).contains(&minimum_white_samples),
            "invalid white detail threshold"
        );
        ensure!(
            (1..=64).contains(&minimum_black_samples),
            "invalid interior black threshold"
        );
        let canvas = view.canvas();
        ensure!(
            crop.width > 0
                && crop.height > 0
                && crop
                    .x
                    .checked_add(crop.width)
                    .is_some_and(|n| n <= self.width)
                && crop
                    .y
                    .checked_add(crop.height)
                    .is_some_and(|n| n <= self.height),
            "crop outside sheet"
        );
        ensure!(
            (1..=canvas - 2).contains(&crop.fit_width)
                && (1..=canvas - if view == SpriteView::Back { 1 } else { 2 })
                    .contains(&crop.fit_height),
            "native fit must preserve top and side margins"
        );
        ensure!(
            palette[0] == 0x7fff
                && palette[3] == 0
                && palette.iter().all(|v| *v <= 0x7fff)
                && (0..4).all(|i| (0..i).all(|j| palette[i] != palette[j])),
            "invalid native palette"
        );
        let mut bounds = [self.width, self.height, 0, 0];
        for y in crop.y..crop.y + crop.height {
            for x in crop.x..crop.x + crop.width {
                if self.pixels[y * self.width + x].iter().any(|v| *v < 220) {
                    bounds[0] = bounds[0].min(x);
                    bounds[1] = bounds[1].min(y);
                    bounds[2] = bounds[2].max(x + 1);
                    bounds[3] = bounds[3].max(y + 1);
                }
            }
        }
        ensure!(
            bounds[2] > bounds[0] && bounds[3] > bounds[1],
            "empty sprite cell"
        );
        let source_width = bounds[2] - bounds[0];
        let source_height = bounds[3] - bounds[1];
        // White eyes and highlights must not lose a vote to their black rims.
        // Separate enclosed white ink from paper connected to the crop edge.
        let mut paper = vec![false; source_width * source_height];
        let mut pending = Vec::new();
        let is_paper = |index: usize| {
            let x = bounds[0] + index % source_width;
            let y = bounds[1] + index / source_width;
            self.pixels[y * self.width + x].iter().all(|v| *v >= 220)
        };
        for y in 0..source_height {
            for x in 0..source_width {
                let index = y * source_width + x;
                if (x == 0
                    || y == 0
                    || x + 1 == source_width
                    || (view == SpriteView::Front && y + 1 == source_height))
                    && is_paper(index)
                {
                    paper[index] = true;
                    pending.push(index);
                }
            }
        }
        while let Some(index) = pending.pop() {
            let x = index % source_width;
            let y = index / source_width;
            let neighbors = [
                (x > 0).then(|| index - 1),
                (x + 1 < source_width).then_some(index + 1),
                (y > 0).then(|| index - source_width),
                (y + 1 < source_height).then_some(index + source_width),
            ];
            for neighbor in neighbors.into_iter().flatten() {
                if !paper[neighbor] && is_paper(neighbor) {
                    paper[neighbor] = true;
                    pending.push(neighbor);
                }
            }
        }
        let exterior_paper = paper.clone();
        let scale = (crop.fit_width as f64 / source_width as f64)
            .min(crop.fit_height as f64 / source_height as f64);
        let width = ((source_width as f64 * scale).round() as usize).clamp(1, crop.fit_width);
        let height = ((source_height as f64 * scale).round() as usize).clamp(1, crop.fit_height);
        // Only small enclosed whites (eyes/highlights) take priority over black.
        // Large white wings still need black edge votes to keep their contours.
        let mut small_white_ink = vec![false; paper.len()];
        let protected_white_pixels = if exterior_contour == ExteriorContour::Solid {
            4
        } else {
            16
        };
        let maximum = source_width * source_height * protected_white_pixels / (width * height);
        for start in 0..paper.len() {
            if paper[start] || !is_paper(start) {
                continue;
            }
            let mut component = Vec::new();
            paper[start] = true;
            pending.push(start);
            while let Some(index) = pending.pop() {
                component.push(index);
                let x = index % source_width;
                let y = index / source_width;
                for neighbor in [
                    (x > 0).then(|| index - 1),
                    (x + 1 < source_width).then_some(index + 1),
                    (y > 0).then(|| index - source_width),
                    (y + 1 < source_height).then_some(index + source_width),
                ]
                .into_iter()
                .flatten()
                {
                    if !paper[neighbor] && is_paper(neighbor) {
                        paper[neighbor] = true;
                        pending.push(neighbor);
                    }
                }
            }
            if component.len() <= maximum {
                for index in component {
                    small_white_ink[index] = true;
                }
            }
        }
        let left = (canvas - width) / 2;
        let top = canvas - height - usize::from(view == SpriteView::Front);
        let rgb = colored(&[0, 1, 2, 3], palette);
        let mut indices = vec![0_u8; canvas * canvas];
        let mut foreground = vec![false; canvas * canvas];
        let mut edge_black = vec![0_u16; canvas * canvas];
        for y in 0..height {
            for x in 0..width {
                let mut votes = [0_u16; 4];
                let mut white_ink = 0;
                let mut enclosed_white_ink = 0;
                let mut ink_coverage = 0;
                let mut black_evidence = 0;
                // A 16x16 grid captures thin diagonal source contours. Public
                // white-detail settings remain expressed in 64-sample units.
                for dy in 0..16 {
                    for dx in 0..16 {
                        let sx = bounds[0]
                            + ((32 * x + 2 * dx + 1) * source_width / (32 * width))
                                .min(source_width - 1);
                        let sy = bounds[1]
                            + ((32 * y + 2 * dy + 1) * source_height / (32 * height))
                                .min(source_height - 1);
                        let pixel = self.pixels[sy * self.width + sx];
                        let nearest = rgb
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, color)| {
                                (0..3)
                                    .map(|i| (i32::from(pixel[i]) - i32::from(color[i])).pow(2))
                                    .sum::<i32>()
                            })
                            .context("four palette colors")?
                            .0;
                        votes[nearest] += 1;
                        let source_index = (sy - bounds[1]) * source_width + sx - bounds[0];
                        ink_coverage += u16::from(!exterior_paper[source_index]);
                        let maximum_channel = pixel[0].max(pixel[1]).max(pixel[2]);
                        let minimum_channel = pixel[0].min(pixel[1]).min(pixel[2]);
                        if nearest == 3
                            && maximum_channel <= 48
                            && maximum_channel - minimum_channel <= 20
                        {
                            black_evidence += 1;
                        }
                        if nearest == 0
                            && small_white_ink[(sy - bounds[1]) * source_width + sx - bounds[0]]
                        {
                            white_ink += 1;
                        }
                        if nearest == 0 && !exterior_paper[source_index] {
                            enclosed_white_ink += 1;
                        }
                    }
                }
                let nearest = if sampling == SpriteSampling::SourceCenter {
                    let sx = bounds[0]
                        + ((2 * x + 1) * source_width / (2 * width)).min(source_width - 1);
                    let sy = bounds[1]
                        + ((2 * y + 1) * source_height / (2 * height)).min(source_height - 1);
                    let pixel = self.pixels[sy * self.width + sx];
                    rgb.iter()
                        .enumerate()
                        .min_by_key(|(_, color)| {
                            (0..3)
                                .map(|i| (i32::from(pixel[i]) - i32::from(color[i])).pow(2))
                                .sum::<i32>()
                        })
                        .context("four palette colors")?
                        .0
                } else if white_ink >= minimum_white_samples * 4
                    || (exterior_contour == ExteriorContour::Solid
                        && enclosed_white_ink >= 128
                        && black_evidence < 48)
                {
                    0
                } else if votes[3] >= minimum_black_samples * 4
                    || (exterior_contour == ExteriorContour::Solid && black_evidence >= 48)
                {
                    3
                } else {
                    (0..3)
                        .max_by_key(|i| votes[*i])
                        .context("four palette votes")?
                };
                let index = (top + y) * canvas + left + x;
                indices[index] = u8::try_from(nearest)?;
                foreground[index] = ink_coverage >= 64 || black_evidence >= 32;
                edge_black[index] = black_evidence;
            }
        }
        // Keep source-backed exterior rims with weaker evidence than interior
        // shading. White foreground inside a source outline remains part of the
        // silhouette; eye/highlight priority cannot punch holes through its rim.
        for y in top..top + height {
            for x in left..left + width {
                let index = y * canvas + x;
                let boundary = !foreground[index - 1]
                    || !foreground[index + 1]
                    || !foreground[index - canvas]
                    || (y + 1 < canvas && !foreground[index + canvas]);
                if foreground[index]
                    && boundary
                    && (exterior_contour == ExteriorContour::Solid || edge_black[index] >= 8)
                {
                    indices[index] = 3;
                }
            }
        }
        ensure!(indices.iter().any(|v| *v != 0), "palette erased the sprite");
        let rows = indices
            .chunks_exact(canvas)
            .map(|row| {
                row.iter()
                    .map(|i| b".123"[usize::from(*i)] as char)
                    .collect()
            })
            .collect();
        Ok(MinnesotaSpriteCandidate {
            outline_profile: exterior_contour.outline_profile(),
            sampling,
            view,
            size: [canvas, canvas],
            source_bounds: bounds,
            palette_rgb555: palette,
            rows,
            indices,
        })
    }
}

impl MinnesotaSpriteCandidate {
    /// Replace the native grid and synchronize the PNG and tile indices.
    ///
    /// Returns an error without changing the frame for a different grid size
    /// or an ink outside `.123`.
    pub fn set_rows(&mut self, rows: Vec<String>) -> Result<()> {
        let canvas = self.view.canvas();
        ensure!(
            rows.len() == canvas && rows.iter().all(|row| row.len() == canvas),
            "refined native grid geometry mismatch"
        );
        let indices = rows
            .iter()
            .flat_map(|row| row.bytes())
            .map(|ink| match ink {
                b'.' => Ok(0),
                b'1'..=b'3' => Ok(ink - b'0'),
                _ => bail!("invalid refined native ink"),
            })
            .collect::<Result<Vec<_>>>()?;
        self.rows = rows;
        self.indices = indices;
        Ok(())
    }

    /// Reflect the native frame horizontally without changing source bounds
    /// or palette. The indexed JSON rows and tile pixels remain synchronized.
    pub fn mirror_horizontal(&mut self) {
        for row in self.indices.chunks_exact_mut(self.view.canvas()) {
            row.reverse();
        }
        for row in &mut self.rows {
            *row = row.chars().rev().collect();
        }
    }

    /// Borrow the native row-major palette indices.
    pub fn indices(&self) -> &[u8] {
        &self.indices
    }

    /// Encode 784 front bytes in row order or 576 back bytes in column order,
    /// matching the native Crystal front/back graphics layouts respectively.
    pub fn bitplanes(&self) -> Vec<u8> {
        let canvas = self.view.canvas();
        let rows = bitplanes(&self.indices, canvas, canvas);
        if self.view == SpriteView::Front {
            return rows;
        }
        let tiles = canvas / 8;
        let mut columns = Vec::with_capacity(rows.len());
        for x in 0..tiles {
            for y in 0..tiles {
                let offset = (y * tiles + x) * 16;
                columns.extend_from_slice(&rows[offset..offset + 16]);
            }
        }
        columns
    }

    /// Render with the supplied four-color palette at native size or integer zoom.
    /// Returns an error for zoom outside 1..=8 or a palette outside RGB555.
    pub fn png(&self, palette: [u16; 4], zoom: usize) -> Result<Vec<u8>> {
        ensure!(
            (1..=8).contains(&zoom) && palette.iter().all(|v| *v <= 0x7fff),
            "invalid review palette/zoom"
        );
        let pixels = colored(&self.indices, palette);
        let canvas = self.view.canvas();
        let mut enlarged = Vec::with_capacity(canvas * canvas * zoom * zoom);
        for y in 0..canvas * zoom {
            for x in 0..canvas * zoom {
                enlarged.push(pixels[(y / zoom) * canvas + x / zoom]);
            }
        }
        png_rgb(canvas * zoom, canvas * zoom, &enlarged)
    }
}

fn paeth(left: u8, up: u8, corner: u8) -> u8 {
    let prediction = i32::from(left) + i32::from(up) - i32::from(corner);
    let distances = [
        (prediction - i32::from(left)).abs(),
        (prediction - i32::from(up)).abs(),
        (prediction - i32::from(corner)).abs(),
    ];
    if distances[0] <= distances[1] && distances[0] <= distances[2] {
        left
    } else if distances[1] <= distances[2] {
        up
    } else {
        corner
    }
}

/// Encode existing native ink indices with a normal or shiny RGB555 palette.
/// Returns an error for a non-square frame, invalid index, or invalid palette.
/// This changes colors only; every authored pixel position is retained.
pub fn indexed_rows_png(rows: &[String], palette: [u16; 4]) -> Result<Vec<u8>> {
    let size = rows.len();
    ensure!(
        matches!(size, 48 | 56),
        "expected a native front or back frame"
    );
    ensure!(
        palette.iter().all(|color| *color <= 0x7fff),
        "invalid RGB555 palette"
    );
    let mut indices = Vec::with_capacity(size * size);
    for row in rows {
        ensure!(row.len() == size, "native frame is not square");
        for value in row.bytes() {
            ensure!((b'0'..=b'3').contains(&value), "invalid native ink index");
            indices.push(value - b'0');
        }
    }
    png_rgb(size, size, &colored(&indices, palette))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[test]
    fn rgba_source_keeps_opaque_white_separate_from_transparent_white() -> Result<()> {
        let png = crate::minnesota_pokemon_pixel_art::png_rgba(
            2,
            1,
            &[[255, 255, 255, 0], [255, 255, 255, 255]],
        )?;
        let source = super::MinnesotaSheet::from_png(&png)?;
        assert_eq!(source.pixel(0, 0), source.pixel(1, 0));
        assert_eq!(source.alpha(0, 0), Some(0));
        assert_eq!(source.alpha(1, 0), Some(255));
        assert_eq!(source.alpha(2, 0), None);
        Ok(())
    }
    use super::{
        ExteriorContour, MinnesotaSheet, SpriteConversionOptions, SpriteCrop, SpriteSampling,
        SpriteView,
    };
    use crate::minnesota_pokemon_pixel_art::colored;

    fn unoutlined_sheet() -> MinnesotaSheet {
        let mut pixels = vec![[255; 3]; 28 * 28];
        let color = colored(&[1], [0x7fff, 0x2b78, 0x3caa, 0])[0];
        for y in 3..25 {
            for x in 3..25 {
                pixels[y * 28 + x] = color;
            }
        }
        // An enclosed eye and a larger loop opening must remain white.
        for y in 7..9 {
            for x in 11..13 {
                pixels[y * 28 + x] = [255; 3];
            }
        }
        for y in 15..21 {
            for x in 15..21 {
                pixels[y * 28 + x] = [255; 3];
            }
        }
        MinnesotaSheet {
            width: 28,
            height: 28,
            pixels,
            alpha: vec![255; 28 * 28],
        }
    }

    #[test]
    fn native_refinements_keep_rows_png_and_tiles_synchronized() -> Result<()> {
        let palette = [0x7fff, 0x2b78, 0x3caa, 0];
        let mut frame = unoutlined_sheet().front_candidate(
            SpriteCrop {
                x: 0,
                y: 0,
                width: 28,
                height: 28,
                fit_width: 20,
                fit_height: 20,
            },
            palette,
        )?;
        let mut rows = frame.rows.clone();
        rows[0].replace_range(0..1, "3");
        frame.set_rows(rows.clone())?;
        assert_eq!(frame.indices()[0], 3);
        assert_eq!(frame.bitplanes()[0] & 0x80, 0x80);
        assert_eq!(frame.bitplanes()[1] & 0x80, 0x80);
        assert_eq!(
            MinnesotaSheet::from_png(&frame.png(palette, 1)?)?.pixels[0],
            [0; 3]
        );
        let indices = frame.indices().to_vec();
        let mut invalid = rows.clone();
        invalid[0].replace_range(0..1, "x");
        assert!(frame.set_rows(invalid).is_err());
        assert!(frame.set_rows(vec![".".to_string()]).is_err());
        assert_eq!(frame.rows, rows);
        assert_eq!(frame.indices(), indices);
        Ok(())
    }

    #[test]
    fn solid_contour_closes_only_the_existing_boundary() -> Result<()> {
        let sheet = unoutlined_sheet();
        let frame = sheet.candidate_with_thresholds_and_contour(
            SpriteCrop {
                x: 0,
                y: 0,
                width: 28,
                height: 28,
                fit_width: 11,
                fit_height: 11,
            },
            [0x7fff, 0x2b78, 0x3caa, 0],
            SpriteView::Front,
            16,
            32,
            ExteriorContour::Solid,
        )?;
        assert_eq!(frame.outline_profile, "solid-exterior-v1");
        for y in 0..56 {
            for x in 0..56 {
                let pixel = frame.indices()[y * 56 + x];
                if !(22..33).contains(&x) || !(44..55).contains(&y) {
                    assert_eq!(pixel, 0, "outline dilated outside the source silhouette");
                } else if x == 22 || x == 32 || y == 44 || y == 54 {
                    assert_eq!(pixel, 3, "solid exterior contour has a gap");
                }
            }
        }
        assert_eq!(frame.indices()[46 * 56 + 26], 0, "white eye was inked");
        assert_eq!(frame.indices()[51 * 56 + 29], 0, "loop opening was inked");
        assert_eq!(frame.indices()[48 * 56 + 25], 1, "body core changed");
        Ok(())
    }

    #[test]
    fn default_contour_and_open_back_crop_remain_unchanged() -> Result<()> {
        let sheet = unoutlined_sheet();
        let crop = SpriteCrop {
            x: 0,
            y: 0,
            width: 28,
            height: 28,
            fit_width: 11,
            fit_height: 11,
        };
        let palette = [0x7fff, 0x2b78, 0x3caa, 0];
        let ordinary = sheet.candidate_with_thresholds(crop, palette, SpriteView::Back, 16, 32)?;
        let explicit = sheet.candidate_with_thresholds_and_contour(
            crop,
            palette,
            SpriteView::Back,
            16,
            32,
            ExteriorContour::SourceBacked,
        )?;
        assert_eq!(ordinary.bitplanes(), explicit.bitplanes());
        let area_vote = sheet.candidate_with_conversion(
            crop,
            palette,
            SpriteView::Back,
            SpriteConversionOptions {
                minimum_white_samples: 16,
                minimum_black_samples: 32,
                exterior_contour: ExteriorContour::SourceBacked,
                sampling: SpriteSampling::default(),
            },
        )?;
        assert_eq!(ordinary.indices(), area_vote.indices());
        assert_eq!(ordinary.bitplanes(), area_vote.bitplanes());
        assert_eq!(ordinary.outline_profile, "source-contour-v1");
        let solid = sheet.candidate_with_thresholds_and_contour(
            crop,
            palette,
            SpriteView::Back,
            16,
            32,
            ExteriorContour::Solid,
        )?;
        assert_eq!(solid.indices()[47 * 48 + 23], 1, "rear crop was capped");
        assert_eq!(solid.indices()[47 * 48 + 18], 3, "rear side rim is open");
        Ok(())
    }

    #[test]
    fn source_center_preserves_pupil_and_tooth_separator_without_painting() -> Result<()> {
        let palette = [0x7fff, 0x2b78, 0x3caa, 0];
        let color = colored(&[1], palette)[0];
        let mut pixels = vec![[255; 3]; 60 * 60];
        for y in 5..55 {
            for x in 5..55 {
                pixels[y * 60 + x] = if x == 5 || x == 54 || y == 5 || y == 54 {
                    [0; 3]
                } else {
                    color
                };
            }
        }
        for y in 20..25 {
            for x in 20..25 {
                pixels[y * 60 + x] = [255; 3];
            }
        }
        for y in 21..24 {
            for x in 21..24 {
                pixels[y * 60 + x] = [0; 3];
            }
        }
        for y in 35..45 {
            for x in 20..35 {
                pixels[y * 60 + x] = if x == 27 { [0; 3] } else { [255; 3] };
            }
        }
        let sheet = MinnesotaSheet {
            width: 60,
            height: 60,
            pixels,
            alpha: vec![255; 60 * 60],
        };
        let crop = SpriteCrop {
            x: 0,
            y: 0,
            width: 60,
            height: 60,
            fit_width: 10,
            fit_height: 10,
        };
        let area = sheet.candidate_with_thresholds(crop, palette, SpriteView::Front, 16, 32)?;
        let center = sheet.candidate_with_conversion(
            crop,
            palette,
            SpriteView::Front,
            SpriteConversionOptions {
                minimum_white_samples: 16,
                minimum_black_samples: 32,
                exterior_contour: ExteriorContour::SourceBacked,
                sampling: SpriteSampling::SourceCenter,
            },
        )?;
        assert_eq!(
            area.indices()[48 * 56 + 26],
            0,
            "fixture lacks white override"
        );
        assert_eq!(center.indices()[48 * 56 + 26], 3, "source pupil vanished");
        for y in 51..53 {
            assert_eq!(area.indices()[y * 56 + 27], 0);
            assert_eq!(center.indices()[y * 56 + 26], 0, "left tooth was inked");
            assert_eq!(center.indices()[y * 56 + 27], 3, "tooth seam vanished");
            assert_eq!(center.indices()[y * 56 + 28], 0, "right tooth was inked");
        }
        assert_eq!(center.indices()[50 * 56 + 23], 3, "source rim vanished");
        assert_eq!(center.indices()[50 * 56 + 22], 0, "rim expanded into paper");
        Ok(())
    }

    #[test]
    fn solid_policy_retains_source_seams_and_broad_white_ink() -> Result<()> {
        let mut sheet = unoutlined_sheet();
        for y in 3..25 {
            sheet.pixels[y * 28 + 11] = [0; 3];
        }
        let palette = [0x7fff, 0x2b78, 0x3caa, 0];
        let crop = SpriteCrop {
            x: 0,
            y: 0,
            width: 28,
            height: 28,
            fit_width: 4,
            fit_height: 4,
        };
        let old = sheet.candidate_with_thresholds(crop, palette, SpriteView::Front, 16, 32)?;
        let solid = sheet.candidate_with_thresholds_and_contour(
            crop,
            palette,
            SpriteView::Front,
            16,
            32,
            ExteriorContour::Solid,
        )?;
        assert_eq!(old.indices()[53 * 56 + 27], 1);
        assert_eq!(
            solid.indices()[53 * 56 + 27],
            3,
            "thin source seam vanished"
        );

        for y in 3..25 {
            for x in 3..25 {
                sheet.pixels[y * 28 + x] = if x == 3 || x == 24 || y == 3 || y == 24 {
                    [0; 3]
                } else if x == 14 || x == 15 {
                    // Dark colored shading votes black in the output palette,
                    // but must not eat an adjacent large white body feature.
                    [50, 0, 0]
                } else {
                    [255; 3]
                };
            }
        }
        let crop = SpriteCrop {
            fit_width: 11,
            fit_height: 11,
            ..crop
        };
        let old = sheet.candidate_with_thresholds(crop, palette, SpriteView::Front, 16, 32)?;
        let solid = sheet.candidate_with_thresholds_and_contour(
            crop,
            palette,
            SpriteView::Front,
            16,
            32,
            ExteriorContour::Solid,
        )?;
        assert_eq!(old.indices()[48 * 56 + 27], 3);
        assert_eq!(solid.indices()[48 * 56 + 27], 0, "broad white ink eroded");
        Ok(())
    }
}
