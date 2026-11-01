//! Coarse listening triage, never an aesthetic or recognizability test.

pub(super) struct Voice {
    pub id: String,
    pub family: String,
    spectrum: Vec<f64>,
    envelope: Vec<f64>,
}

impl Voice {
    pub fn measure(id: &str, family: &str, pcm: &[i16]) -> Self {
        let mono = pcm
            .as_chunks::<2>()
            .0
            .iter()
            .map(|frame| (f64::from(frame[0]) + f64::from(frame[1])) / 65536.0)
            .collect::<Vec<_>>();
        let mut spectrum = vec![0.0; 32];
        for window in mono.as_chunks::<256>().0 {
            for (band, power) in spectrum.iter_mut().enumerate() {
                let hz = 90.0 * (8000.0_f64 / 90.0).powf(band as f64 / 31.0);
                let coefficient = 2.0 * (std::f64::consts::TAU * hz / 22050.0).cos();
                let (mut a, mut b) = (0.0, 0.0);
                for (index, sample) in window.iter().enumerate() {
                    let taper = 0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / 255.0).cos();
                    let next = sample * taper + coefficient * a - b;
                    b = a;
                    a = next;
                }
                *power += (a * a + b * b - coefficient * a * b).max(0.0);
            }
        }
        for power in &mut spectrum {
            *power = power.sqrt();
        }
        let mut envelope = vec![0.0; 16];
        for (index, sample) in mono.iter().enumerate() {
            envelope[index * 16 / mono.len()] += sample * sample;
        }
        for energy in &mut envelope {
            *energy = energy.sqrt();
        }
        normalize(&mut spectrum);
        normalize(&mut envelope);
        Self {
            id: id.into(),
            family: family.into(),
            spectrum,
            envelope,
        }
    }
}

fn normalize(values: &mut [f64]) {
    let norm = values.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm > 0.0 {
        for value in values {
            *value /= norm;
        }
    }
}

pub(super) fn report(voices: &[Voice], label: &str) {
    let mut pairs = Vec::new();
    for (index, a) in voices.iter().enumerate() {
        for b in &voices[index + 1..] {
            if a.family == b.family {
                continue;
            }
            let spectral = a
                .spectrum
                .iter()
                .zip(&b.spectrum)
                .map(|(a, b)| a * b)
                .sum::<f64>();
            let temporal = a
                .envelope
                .iter()
                .zip(&b.envelope)
                .map(|(a, b)| a * b)
                .sum::<f64>();
            pairs.push((0.7 * spectral + 0.3 * temporal, &a.id, &b.id));
        }
    }
    pairs.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!(
        "{label} coarse similarity: {} unrelated pairs >= 0.95; listening candidates (not a quality verdict):",
        pairs.iter().filter(|p| p.0 >= 0.95).count()
    );
    for (similarity, a, b) in pairs.iter().take(16) {
        println!("  {a} / {b}: {similarity:.4}");
    }
}
