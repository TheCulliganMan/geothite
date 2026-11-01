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
    shared: Option<SharedFrames>,
}

/// Two private POSIX shared-memory slots, consumed/unlinked by the terminal.
/// Never overwrite an unread frame or queue cosmetics behind real inputs.
/// Raw RGB bypasses PNG encode/decode and megabytes of base64 in the PTY.
struct SharedFrames {
    #[cfg(unix)]
    names: [std::ffi::CString; 2],
    used: [bool; 2],
    consumed: bool,
    started: std::time::Instant,
}
impl SharedFrames {
    fn new() -> Option<Self> {
        #[cfg(unix)]
        {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .subsec_nanos();
            let name = |slot| {
                std::ffi::CString::new(format!("/geo-{:x}-{nonce:x}-{slot}", std::process::id()))
                    .ok()
            };
            Some(Self {
                names: [name(0)?, name(1)?],
                used: [false; 2],
                consumed: false,
                started: std::time::Instant::now(),
            })
        }
        #[cfg(not(unix))]
        {
            None
        }
    }
    fn frame(&mut self, slot: usize, image: &image::RgbImage) -> io::Result<Option<String>> {
        #[cfg(unix)]
        {
            use std::os::fd::FromRawFd;
            // SAFETY: names are owned NUL-terminated strings. O_EXCL prevents
            // replacing another object or a frame the terminal hasn't read.
            let fd = unsafe {
                libc::shm_open(
                    self.names[slot].as_ptr(),
                    libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                    0o600,
                )
            };
            if fd < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::AlreadyExists {
                    return Ok(None);
                }
                return Err(error);
            }
            self.consumed |= self.used[slot];
            self.used[slot] = true;
            // POSIX shared memory is mapped, not written through read/write
            // (macOS rejects those calls on shm descriptors).
            // SAFETY: File exclusively owns and closes this new descriptor.
            let file = unsafe { std::fs::File::from_raw_fd(fd) };
            file.set_len(image.as_raw().len() as u64)?;
            // SAFETY: the just-sized object covers this nonzero RGB length.
            // Check MAP_FAILED before copying; unmap once after writing.
            let length = image.as_raw().len();
            let map = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    length,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    fd,
                    0,
                )
            };
            if map == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }
            unsafe {
                std::ptr::copy_nonoverlapping(image.as_raw().as_ptr(), map.cast::<u8>(), length);
                libc::munmap(map, length);
            }
            Ok(Some(STANDARD.encode(self.names[slot].as_bytes())))
        }
        #[cfg(not(unix))]
        {
            let _ = (slot, image);
            Err(io::Error::other("shared memory unavailable"))
        }
    }
    fn cleanup(&mut self) {
        #[cfg(unix)]
        for (slot, name) in self.names.iter().enumerate() {
            if !self.used[slot] {
                continue;
            }
            // SAFETY: unlink only our two exact, exclusively created names.
            unsafe {
                libc::shm_unlink(name.as_ptr());
            }
            self.used[slot] = false;
        }
    }
    fn unsupported(&self) -> bool {
        !self.consumed && self.started.elapsed() > std::time::Duration::from_secs(2)
    }
}
impl Drop for SharedFrames {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn use_shared_memory(program: &str, term: &str, remote: bool, choice: &str) -> bool {
    choice == "shared"
        || (choice != "direct"
            && !remote
            && (program == "ghostty" || term == "xterm-ghostty" || term == "xterm-kitty"))
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
            shared: if enabled
                && use_shared_memory(
                    &env("TERM_PROGRAM"),
                    &env("TERM"),
                    !env("SSH_CONNECTION").is_empty()
                        || !env("SSH_TTY").is_empty()
                        || !env("SSH_CLIENT").is_empty(),
                    &env("GEOTHITE_TUI_TRANSPORT"),
                ) {
                SharedFrames::new()
            } else {
                None
            },
        }
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn clear(&mut self, output: &mut impl Write) -> io::Result<()> {
        self.signature = None;
        if let Some(shared) = self.shared.as_mut() {
            shared.cleanup();
        }
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
        let next = self.active.map_or(0, |i| 1 - i);
        if let Some(shared) = self.shared.as_mut() {
            match shared.frame(next, image) {
                // Fall back only after genuinely unread slots. Reduced motion
                // can leave a successfully consumed single frame idle for ages.
                Ok(None) if shared.unsupported() => self.shared = None,
                Ok(None) => return Ok(()), // Backpressure: no unbounded frame queue.
                Ok(Some(payload)) => {
                    let mut packet = Vec::new();
                    write!(
                        packet,
                        "\x1b7\x1b[{};{}H\x1b_Ga=T,f=24,t=s,s={},v={},i={},c={},r={},C=1,q=2,z=1;{}\x1b\\",
                        area.y + 1,
                        area.x + 1,
                        image.width(),
                        image.height(),
                        self.ids[next],
                        area.width,
                        area.height,
                        payload
                    )?;
                    if let Some(previous) = self.active {
                        write!(packet, "\x1b_Ga=d,d=I,i={},q=2\x1b\\", self.ids[previous])?;
                    }
                    packet.extend_from_slice(b"\x1b8");
                    output.write_all(&packet)?;
                    output.flush()?;
                    self.active = Some(next);
                    self.signature = Some(signature);
                    return Ok(());
                }
                Err(_) => self.shared = None, // Portable compressed fallback.
            }
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
        let id = self.ids[next];
        // Keep the previous image visible until this whole frame arrives.
        // q=2 prevents protocol responses from masquerading as game inputs;
        // C=1 prevents cursor movement/scrolling outside the art rectangle.
        let mut packet = Vec::with_capacity(payload.len() + payload.len() / 100);
        write!(packet, "\x1b7\x1b[{};{}H", area.y + 1, area.x + 1)?;
        let chunks: Vec<_> = payload.as_bytes().chunks(4096).collect();
        for (i, chunk) in chunks.iter().enumerate() {
            let more = usize::from(i + 1 != chunks.len());
            if i == 0 {
                write!(
                    packet,
                    "\x1b_Ga=T,f=100,t=d,i={id},c={},r={},C=1,q=2,z=1,m={more};",
                    area.width, area.height
                )?;
            } else {
                write!(packet, "\x1b_Gm={more};")?;
            }
            packet.write_all(chunk)?;
            packet.write_all(b"\x1b\\")?;
        }
        if let Some(previous) = self.active {
            write!(packet, "\x1b_Ga=d,d=I,i={},q=2\x1b\\", self.ids[previous])?;
        }
        packet.write_all(b"\x1b8")?;
        output.write_all(&packet)?;
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
    fn shared_transport_is_local_only_unless_explicitly_selected() {
        assert!(use_shared_memory("ghostty", "", false, ""));
        assert!(!use_shared_memory("ghostty", "", true, ""));
        assert!(!use_shared_memory("ghostty", "", false, "direct"));
        assert!(!use_shared_memory("", "xterm-256color", false, ""));
    }
    #[test]
    #[cfg(unix)]
    fn shared_rgb_is_exact_bounded_and_never_overwrites_unread_frames() {
        use std::os::fd::FromRawFd;
        let mut shared = SharedFrames::new().unwrap();
        let first = image::RgbImage::from_pixel(64, 32, image::Rgb([10, 40, 80]));
        let second = image::RgbImage::from_pixel(64, 32, image::Rgb([20, 50, 90]));
        shared.frame(0, &first).unwrap().unwrap();
        shared.frame(1, &second).unwrap().unwrap();
        assert!(
            shared.frame(0, &second).unwrap().is_none(),
            "Bounded queue; don't replace an unread image"
        );
        // SAFETY: open our exact object, own the returned descriptor, close it
        // with File, then simulate the terminal's protocol-defined unlink.
        let fd = unsafe { libc::shm_open(shared.names[0].as_ptr(), libc::O_RDONLY, 0) };
        assert!(fd >= 0);
        let _file = unsafe { std::fs::File::from_raw_fd(fd) };
        let length = first.as_raw().len();
        let map = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                length,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        assert_ne!(map, libc::MAP_FAILED);
        let received = unsafe { std::slice::from_raw_parts(map.cast::<u8>(), length) };
        assert_eq!(received, first.as_raw());
        unsafe {
            libc::munmap(map, length);
        }
        unsafe {
            libc::shm_unlink(shared.names[0].as_ptr());
        }
        shared.frame(0, &second).unwrap().unwrap();
        assert!(shared.consumed);
        let name = shared.names[0].clone();
        drop(shared);
        let fd = unsafe { libc::shm_open(name.as_ptr(), libc::O_RDONLY, 0) };
        assert!(fd < 0, "Owned shared-memory frames are cleaned on exit");
    }
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
            shared: None,
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
