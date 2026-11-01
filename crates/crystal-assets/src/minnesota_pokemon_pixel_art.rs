//! Original native-size pixel source compiler, independent of pack publication.
//!
//! Source grids are directly authored palette indices, never sampled or reduced
//! from the concept sheets. Generated PNG/2bpp files belong in ignored output.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use anyhow::{Context, Result, ensure};
use crystal_core::models::{FrontpicAnimCommand, FrontpicAnimProgram};
use flate2::write::ZlibEncoder;
use flate2::{Compression, Crc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MinnesotaPixelSource {
    pub schema_version: u16,
    pub id: String,
    pub runtime_token: String,
    pub front_frames: [MinnesotaPixelFrame; 2],
    pub back: MinnesotaPixelFrame,
    pub icon_frames: [MinnesotaPixelFrame; 2],
}

/// Each character is an original source pixel: `.` white, `1` light, `2` dark,
/// `3` black. Short crop rows are padded with white to the canvas edge.
#[derive(Debug, Deserialize)]
pub struct MinnesotaPixelFrame {
    pub size: [usize; 2],
    pub origin: [usize; 2],
    pub rows: Vec<String>,
}

#[derive(Debug)]
pub struct MinnesotaNativeArt {
    /// Runtime-relative files only. No source pack or audio is read or mutated.
    pub files: BTreeMap<String, Vec<u8>>,
    pub preview_png: Vec<u8>,
    /// Actual typed main/idle candidates, separate from pack registration.
    pub animation_programs: BTreeMap<String, FrontpicAnimProgram>,
}

impl MinnesotaPixelSource {
    /// Parse and validate original authored grids and the native frame geometry.
    /// Returns errors for malformed JSON, bad palette symbols, clipping, empty
    /// art, a non-white border, identical animation frames, or wrong canvas sizes.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let source: Self = serde_json::from_slice(bytes).context("parse native pixel source")?;
        source.validate()?;
        Ok(source)
    }

    fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 1, "unsupported pixel source version");
        ensure!(
            self.id.len() == 6
                && self.id.starts_with("MN_")
                && self.id[3..]
                    .parse::<u16>()
                    .is_ok_and(|index| (1..=100).contains(&index)),
            "invalid MN identity"
        );
        ensure!(
            !self.runtime_token.is_empty()
                && self
                    .runtime_token
                    .bytes()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_'),
            "invalid native runtime token"
        );
        for (frame, size) in self.front_frames.iter().zip([[56, 56]; 2]) {
            ensure!(frame.size == size, "front canvas must be 56x56");
            frame.pixels()?;
        }
        ensure!(self.back.size == [48, 48], "back canvas must be 48x48");
        self.back.pixels()?;
        for frame in &self.icon_frames {
            ensure!(frame.size == [16, 16], "icon frame must be 16x16");
            frame.pixels()?;
        }
        ensure!(
            self.front_frames[0].pixels()? != self.front_frames[1].pixels()?,
            "front animation needs two authored frames"
        );
        ensure!(
            self.icon_frames[0].pixels()? != self.icon_frames[1].pixels()?,
            "icon animation needs two authored frames"
        );
        Ok(())
    }

    /// Encode real native PNGs, bitplanes, palette data and an animation bank.
    /// Returns an error for invalid palettes or source geometry. It neither
    /// registers species nor writes files; the caller owns ignored export paths.
    pub fn compile(&self, normal: [u16; 4], shiny: [u16; 4]) -> Result<MinnesotaNativeArt> {
        self.validate()?;
        for palette in [normal, shiny] {
            ensure!(
                palette[0] == 0x7fff
                    && palette[3] == 0
                    && palette.iter().all(|c| *c < 0x8000)
                    && palette.iter().collect::<BTreeSet<_>>().len() == 4,
                "palettes must use RGB555 with white/black endpoints"
            );
        }
        let front = self
            .front_frames
            .iter()
            .map(MinnesotaPixelFrame::pixels)
            .collect::<Result<Vec<_>>>()?;
        let back = self.back.pixels()?;
        let icons = self
            .icon_frames
            .iter()
            .map(MinnesotaPixelFrame::pixels)
            .collect::<Result<Vec<_>>>()?;
        let root = format!("gfx/pokemon/{}", self.runtime_token.to_ascii_lowercase());
        let mut files = BTreeMap::new();
        let mut put = |name: &str, bytes: Vec<u8>| {
            files.insert(format!("{root}/{name}"), bytes);
        };
        let front_pixels = front.concat();
        put(
            "front.png",
            png_rgb(56, 112, &colored(&front_pixels, normal))?,
        );
        put("back.png", png_rgb(48, 48, &colored(&back, normal))?);
        put(
            "front.2bpp",
            front
                .iter()
                .flat_map(|frame| bitplanes(frame, 56, 56))
                .collect(),
        );
        put("back.2bpp", bitplanes(&back, 48, 48));
        put("front.dimensions", vec![0x77]);
        put(
            "normal.gbcpal",
            normal
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect(),
        );
        let mut shiny_text = String::new();
        for value in &shiny[1..3] {
            shiny_text.push_str(&format!(
                "RGB {}, {}, {}\n",
                value & 31,
                (value >> 5) & 31,
                (value >> 10) & 31
            ));
        }
        put("shiny.pal", shiny_text.into_bytes());
        let mut bank: Vec<Vec<u8>> = Vec::new();
        let mut tilemap = Vec::new();
        for frame in &front {
            for tile in bitplanes(frame, 56, 56).as_chunks::<16>().0 {
                let index = if let Some(index) = bank
                    .iter()
                    .position(|entry| entry.as_slice() == tile.as_slice())
                {
                    index
                } else {
                    bank.push(tile.to_vec());
                    bank.len() - 1
                };
                tilemap.push(u8::try_from(index).context("animation bank exceeds 256 tiles")?);
            }
        }
        put("front.animated.2bpp", bank.concat());
        put("front.animated.tilemap", tilemap);
        let icon_pixels = icons.concat();
        let gray = [[255, 255, 255], [170, 170, 170], [85, 85, 85], [0, 0, 0]];
        let icon_rgb: Vec<[u8; 3]> = icon_pixels
            .iter()
            .map(|index| gray[usize::from(*index)])
            .collect();
        put("icon.png", png_rgb(16, 32, &icon_rgb)?);
        let icon_2bpp: Vec<u8> = icons
            .iter()
            .flat_map(|frame| bitplanes(frame, 16, 16))
            .collect();
        put("icon.2bpp", icon_2bpp.clone());
        files.insert(
            format!("gfx/icons/{}.2bpp", self.id.to_ascii_lowercase()),
            icon_2bpp,
        );
        let preview_png = review_png(&front, &back, &icons, normal, shiny)?;
        let animation_programs = [
            (self.runtime_token.clone(), [4, 10, 6]),
            (format!("{}_IDLE", self.runtime_token), [20, 6, 6]),
        ]
        .into_iter()
        .map(|(key, durations)| {
            let mut commands: Vec<FrontpicAnimCommand> = [0, 1, 0]
                .into_iter()
                .zip(durations)
                .map(|(frame, duration)| FrontpicAnimCommand {
                    kind: "frame".to_string(),
                    frame: Some(frame),
                    duration: Some(duration),
                    ..FrontpicAnimCommand::default()
                })
                .collect();
            commands.push(FrontpicAnimCommand {
                kind: "endanim".to_string(),
                ..FrontpicAnimCommand::default()
            });
            (key, FrontpicAnimProgram { commands })
        })
        .collect();
        Ok(MinnesotaNativeArt {
            files,
            preview_png,
            animation_programs,
        })
    }
}

impl MinnesotaPixelFrame {
    /// Expand an original cropped glyph into its native canvas without scaling.
    /// Returns an error for invalid symbols, clipping or artwork on the border.
    pub fn pixels(&self) -> Result<Vec<u8>> {
        let [width, height] = self.size;
        ensure!(
            width > 0 && height > 0 && width <= 56 && height <= 56,
            "invalid source canvas"
        );
        let [left, top] = self.origin;
        ensure!(
            top.checked_add(self.rows.len())
                .is_some_and(|bottom| bottom < height),
            "source crop clips bottom border"
        );
        let mut pixels = vec![0; width * height];
        for (row, text) in self.rows.iter().enumerate() {
            ensure!(
                left.checked_add(text.len())
                    .is_some_and(|right| right < width),
                "source crop clips right border"
            );
            for (column, glyph) in text.bytes().enumerate() {
                let index = match glyph {
                    b'.' => 0,
                    b'1' => 1,
                    b'2' => 2,
                    b'3' => 3,
                    _ => anyhow::bail!("invalid palette symbol {:?}", char::from(glyph)),
                };
                pixels[(top + row) * width + left + column] = index;
            }
        }
        ensure!(
            pixels.iter().filter(|index| **index != 0).count() >= 12,
            "empty or negligible native drawing"
        );
        ensure!(
            (0..width).all(|x| pixels[x] == 0 && pixels[(height - 1) * width + x] == 0)
                && (0..height)
                    .all(|y| pixels[y * width] == 0 && pixels[y * width + width - 1] == 0),
            "sprite needs a white colorkey border"
        );
        Ok(pixels)
    }
}

pub(crate) fn bitplanes(pixels: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(width * height / 4);
    for tile_y in 0..height / 8 {
        for tile_x in 0..width / 8 {
            for y in 0..8 {
                let mut lo = 0;
                let mut hi = 0;
                for x in 0..8 {
                    let value = pixels[(tile_y * 8 + y) * width + tile_x * 8 + x];
                    lo |= (value & 1) << (7 - x);
                    hi |= ((value >> 1) & 1) << (7 - x);
                }
                bytes.extend([lo, hi]);
            }
        }
    }
    bytes
}

pub(crate) fn colored(indices: &[u8], palette: [u16; 4]) -> Vec<[u8; 3]> {
    indices
        .iter()
        .map(|index| {
            let value = palette[usize::from(*index)];
            [0, 5, 10].map(|shift| {
                let component = ((value >> shift) & 31) as u8;
                (component << 3) | (component >> 2)
            })
        })
        .collect()
}

fn review_png(
    front: &[Vec<u8>],
    back: &[u8],
    icons: &[Vec<u8>],
    normal: [u16; 4],
    shiny: [u16; 4],
) -> Result<Vec<u8>> {
    // Top: 4x nearest front A / front B / true back / two icons.
    // Bottom: matching shiny views. Source-size views sit at the right edge.
    let width = 880;
    let height = 492;
    let mut rgb = vec![[235, 232, 220]; width * height];
    for (row, palette) in [normal, shiny].into_iter().enumerate() {
        for (column, (pixels, w, h, scale)) in [
            (&front[0][..], 56, 56, 4),
            (&front[1][..], 56, 56, 4),
            (back, 48, 48, 4),
            (&icons[0][..], 16, 16, 4),
            (&icons[1][..], 16, 16, 4),
        ]
        .into_iter()
        .enumerate()
        {
            let left = [8, 240, 472, 676, 748][column];
            let top = row * 244 + 8;
            let colors = if column < 3 {
                colored(pixels, palette)
            } else {
                pixels.iter().map(|i| [255 - i * 85; 3]).collect()
            };
            for y in 0..h * scale {
                for x in 0..w * scale {
                    rgb[(top + y) * width + left + x] = colors[(y / scale) * w + x / scale];
                }
            }
        }
        let colors = colored(&front[0], palette);
        for y in 0..56 {
            for x in 0..56 {
                rgb[(row * 244 + 172 + y) * width + 808 + x] = colors[y * 56 + x];
            }
        }
    }
    png_rgb(width, height, &rgb)
}

pub(crate) fn png_rgb(width: usize, height: usize, pixels: &[[u8; 3]]) -> Result<Vec<u8>> {
    ensure!(pixels.len() == width * height, "PNG pixel count mismatch");
    let mut scanlines = Vec::with_capacity(height * (width * 3 + 1));
    for row in pixels.chunks_exact(width) {
        scanlines.push(0);
        for rgb in row {
            scanlines.extend(rgb);
        }
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&scanlines)?;
    let mut header = Vec::new();
    header.extend(u32::try_from(width)?.to_be_bytes());
    header.extend(u32::try_from(height)?.to_be_bytes());
    header.extend([8, 2, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png_chunk(&mut png, b"IHDR", &header)?;
    png_chunk(&mut png, b"IDAT", &encoder.finish()?)?;
    png_chunk(&mut png, b"IEND", &[])?;
    Ok(png)
}

/// Encode supplied native RGBA pixels without flattening their transparency.
pub(crate) fn png_rgba(width: usize, height: usize, pixels: &[[u8; 4]]) -> Result<Vec<u8>> {
    ensure!(
        width > 0 && pixels.len() == width * height,
        "RGBA pixel count mismatch"
    );
    let mut scanlines = Vec::with_capacity(height * (width * 4 + 1));
    for row in pixels.chunks_exact(width) {
        scanlines.push(0);
        for rgba in row {
            scanlines.extend(rgba);
        }
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&scanlines)?;
    let mut header = Vec::new();
    header.extend(u32::try_from(width)?.to_be_bytes());
    header.extend(u32::try_from(height)?.to_be_bytes());
    header.extend([8, 6, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png_chunk(&mut png, b"IHDR", &header)?;
    png_chunk(&mut png, b"IDAT", &encoder.finish()?)?;
    png_chunk(&mut png, b"IEND", &[])?;
    Ok(png)
}

fn png_chunk(output: &mut Vec<u8>, kind: &[u8; 4], bytes: &[u8]) -> Result<()> {
    let mut crc = Crc::new();
    crc.update(kind);
    crc.update(bytes);
    output.extend(u32::try_from(bytes.len())?.to_be_bytes());
    output.extend(kind);
    output.extend(bytes);
    output.extend(crc.sum().to_be_bytes());
    Ok(())
}
