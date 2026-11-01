//! Exact conversion of byte source palettes, shared across frames and maps.
use std::sync::OnceLock;

pub(crate) fn linear(value: u8) -> f32 {
    static TABLE: OnceLock<[f32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| std::array::from_fn(|index| convert(index as u8)))[value as usize]
}

fn convert(value: u8) -> f32 {
    let value = f32::from(value) / 255.;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_channel_preserves_exact_linear_bits() {
        for value in 0..=255_u8 {
            assert_eq!(linear(value).to_bits(), convert(value).to_bits());
        }
    }

    #[test]
    #[ignore = "opt-in CPU microbenchmark; no renderer or window"]
    fn palette_conversion_timing() {
        use std::{hint::black_box, time::Instant};
        let _ = linear(0);
        for (name, convert) in [("formula", convert as fn(u8) -> f32), ("lookup", linear)] {
            let start = Instant::now();
            for _ in 0..100_000 {
                for value in 0..=255_u8 {
                    black_box(convert(black_box(value)));
                }
            }
            println!("{name}: {:?}", start.elapsed());
        }
    }
}
