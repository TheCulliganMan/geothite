//! Output transport for the same Rust dot canvas. No gameplay or asset logic.
use base64::{Engine, engine::general_purpose::STANDARD};
use image::{
    ImageEncoder,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};
use ratatui::layout::Rect;
use std::io::{self, Write};

pub struct TerminalCanvas {
    enabled: bool,
    ids: [u32; 2],
    active: Option<usize>,
    signature: Option<u64>,
}

/// Bound raster allocation without changing the terminal's scene aspect ratio.
pub fn terminal_canvas_size(width: u32, height: u32) -> (u32, u32) {
    let longest = width.max(height).max(2048);
    let scale = |value| ((u64::from(value) * 2048) / u64::from(longest)).max(1) as u32;
    (scale(width), scale(height))
}

fn supports_graphics(program: &str, term: &str, multiplexed: bool, choice: &str) -> bool {
    if choice == "off" {
        return false;
    }
    if choice == "kitty" {
        return true;
    }
    !multiplexed && (program == "ghostty" || term == "xterm-ghostty" || term == "xterm-kitty")
}

impl TerminalCanvas {
    pub fn detect() -> Self {
        let env = |name| std::env::var(name).unwrap_or_default();
        let enabled = supports_graphics(
            &env("TERM_PROGRAM"),
            &env("TERM"),
            !env("TMUX").is_empty() || !env("STY").is_empty(),
            &env("GEOTHITE_TUI_GRAPHICS"),
        );
        // Owned IDs only; never clear another application's terminal images.
        let base = 0x47000000 | (std::process::id() & 0x00fffffe);
        Self {
            enabled,
            ids: [base, base + 1],
            active: None,
            signature: None,
        }
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn clear(&mut self, output: &mut impl Write) -> io::Result<()> {
        self.signature = None;
        if self.active.take().is_some() {
            for id in self.ids {
                write!(output, "\x1b_Ga=d,d=I,i={id},q=2\x1b\\")?;
            }
            output.flush()?;
        }
        Ok(())
    }
    pub fn paint(
        &mut self,
        output: &mut impl Write,
        area: Rect,
        image: &image::RgbImage,
    ) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (
            area.x,
            area.y,
            area.width,
            area.height,
            image.width(),
            image.height(),
            image.as_raw(),
        )
            .hash(&mut hasher);
        let signature = hasher.finish();
        if self.signature == Some(signature) {
            return Ok(());
        }
        let mut png = Vec::new();
        // Lossless row filtering matters over SSH: an unfiltered fast PNG can
        // send nearly raw RGB on every cosmetic frame and delay real inputs.
        PngEncoder::new_with_quality(&mut png, CompressionType::Default, FilterType::Adaptive)
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::Rgb8,
            )
            .map_err(io::Error::other)?;
        let payload = STANDARD.encode(png);
        let next = self.active.map_or(0, |i| 1 - i);
        let id = self.ids[next];
        // Keep the previous image visible until this whole frame arrives.
        // q=2 prevents protocol responses from masquerading as game inputs;
        // C=1 prevents cursor movement/scrolling outside the art rectangle.
        write!(output, "\x1b7\x1b[{};{}H", area.y + 1, area.x + 1)?;
        let chunks: Vec<_> = payload.as_bytes().chunks(4096).collect();
        for (i, chunk) in chunks.iter().enumerate() {
            let more = usize::from(i + 1 != chunks.len());
            if i == 0 {
                write!(
                    output,
                    "\x1b_Ga=T,f=100,t=d,i={id},c={},r={},C=1,q=2,z=1,m={more};",
                    area.width, area.height
                )?;
            } else {
                write!(output, "\x1b_Gm={more};")?;
            }
            output.write_all(chunk)?;
            output.write_all(b"\x1b\\")?;
        }
        if let Some(previous) = self.active {
            write!(output, "\x1b_Ga=d,d=I,i={},q=2\x1b\\", self.ids[previous])?;
        }
        output.write_all(b"\x1b8")?;
        output.flush()?;
        self.active = Some(next);
        self.signature = Some(signature);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn large_canvases_keep_their_aspect_ratio_and_bound_allocation() {
        assert_eq!(terminal_canvas_size(864, 288), (864, 288));
        assert_eq!(terminal_canvas_size(4096, 2048), (2048, 1024));
        assert_eq!(terminal_canvas_size(2048, 8192), (512, 2048));
        assert_eq!(terminal_canvas_size(0, 0), (1, 1));
    }
    #[test]
    fn ordinary_and_multiplexed_terminals_use_text_without_graphics_queries() {
        assert!(supports_graphics("ghostty", "xterm-256color", false, ""));
        assert!(!supports_graphics(
            "Apple_Terminal",
            "xterm-256color",
            false,
            ""
        ));
        assert!(!supports_graphics("ghostty", "screen", true, ""));
        assert!(!supports_graphics("ghostty", "xterm-ghostty", false, "off"));
    }
    #[test]
    fn graphics_transport_is_chunked_quiet_bounded_and_cleans_only_owned_images() {
        let mut canvas = TerminalCanvas {
            enabled: true,
            ids: [41, 42],
            active: None,
            signature: None,
        };
        let mut output = Vec::new();
        let image = image::RgbImage::from_fn(96, 96, |x, y| {
            image::Rgb([(x * 13) as u8, (y * 17) as u8, (x * y) as u8])
        });
        canvas
            .paint(&mut output, Rect::new(2, 3, 40, 12), &image)
            .unwrap();
        let commands = String::from_utf8(output.clone()).unwrap();
        assert!(
            commands
                .starts_with("\x1b7\x1b[4;3H\x1b_Ga=T,f=100,t=d,i=41,c=40,r=12,C=1,q=2,z=1,m=1;")
        );
        let mut encoded = String::new();
        for part in commands.split("\x1b_G").skip(1) {
            let (header, tail) = part.split_once(';').unwrap();
            let payload = tail.split("\x1b\\").next().unwrap();
            assert!(payload.len() <= 4096);
            if !header.ends_with("m=0") {
                assert_eq!(payload.len() % 4, 0);
            }
            encoded.push_str(payload);
        }
        let decoded = image::load_from_memory(&STANDARD.decode(encoded).unwrap())
            .unwrap()
            .to_rgb8();
        assert_eq!(decoded, image);
        output.clear();
        canvas
            .paint(&mut output, Rect::new(2, 3, 40, 12), &image)
            .unwrap();
        assert!(
            output.is_empty(),
            "Unchanged/reduced-motion art is not retransmitted"
        );
        canvas
            .paint(&mut output, Rect::new(3, 3, 40, 12), &image)
            .unwrap();
        assert!(
            String::from_utf8(output.clone())
                .unwrap()
                .contains("a=d,d=I,i=41,q=2")
        );
        output.clear();
        canvas.clear(&mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "\x1b_Ga=d,d=I,i=41,q=2\x1b\\\x1b_Ga=d,d=I,i=42,q=2\x1b\\"
        );
    }
}
