//! Encode the supplied native two-bit pixels; no image or exporter dependency.
use std::io::Write;

use anyhow::{Result, ensure};
use flate2::write::ZlibEncoder;
use flate2::{Compression, Crc};

pub(crate) fn from_2bpp(tiles: &[u8]) -> Result<Vec<u8>> {
    const WIDTH: usize = 128;
    const TILES_PER_ROW: usize = WIDTH / 8;
    ensure!(
        !tiles.is_empty() && tiles.len().is_multiple_of(16 * TILES_PER_ROW),
        "native tile sheet must contain complete rows"
    );
    let height = tiles.len() / (16 * TILES_PER_ROW) * 8;
    let mut pixels = Vec::with_capacity((WIDTH + 1) * height);
    for y in 0..height {
        pixels.push(0); // PNG scanline filter: none.
        for x in 0..WIDTH {
            let tile = (y / 8) * TILES_PER_ROW + x / 8;
            let row = tile * 16 + (y % 8) * 2;
            let bit = 7 - x % 8;
            let level = ((tiles[row] >> bit) & 1) | (((tiles[row + 1] >> bit) & 1) << 1);
            pixels.push([255, 170, 85, 0][usize::from(level)]);
        }
    }
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&pixels)?;
    let compressed = compressor.finish()?;
    let mut header = Vec::new();
    header.extend(u32::try_from(WIDTH)?.to_be_bytes());
    header.extend(u32::try_from(height)?.to_be_bytes());
    header.extend([8, 0, 0, 0, 0]); // Eight-bit grayscale, standard compression/filter, no interlace.
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &header)?;
    chunk(&mut png, b"IDAT", &compressed)?;
    chunk(&mut png, b"IEND", &[])?;
    Ok(png)
}

fn chunk(output: &mut Vec<u8>, kind: &[u8; 4], bytes: &[u8]) -> Result<()> {
    let mut crc = Crc::new();
    crc.update(kind);
    crc.update(bytes);
    output.extend(u32::try_from(bytes.len())?.to_be_bytes());
    output.extend(kind);
    output.extend(bytes);
    output.extend(crc.sum().to_be_bytes());
    Ok(())
}
