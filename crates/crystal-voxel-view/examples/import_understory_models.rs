//! Import selected CC0 Quaternius plants into the texture-free terrain palette.
//! Usage: cargo run -p crystal-voxel-view --example import_understory_models --
//! <extracted Standard pack>/glTF <output directory>
//! Alpha silhouettes become real geometry, so no rectangular leaf cards remain.
use std::{collections::HashMap, path::Path};

#[derive(Clone, Copy)]
struct Vertex {
    p: [f32; 3],
    n: [f32; 3],
    uv: [f32; 2],
    c: [f32; 4],
}
impl Vertex {
    fn mix(self, b: Self, t: f32) -> Self {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Self {
            p: std::array::from_fn(|i| mix(self.p[i], b.p[i])),
            n: std::array::from_fn(|i| mix(self.n[i], b.n[i])),
            uv: std::array::from_fn(|i| mix(self.uv[i], b.uv[i])),
            c: std::array::from_fn(|i| mix(self.c[i], b.c[i])),
        }
    }
}
#[derive(Default)]
struct Mesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    vertices: HashMap<[i32; 10], u32>,
}
fn sample(image: &image::RgbaImage, uv: [f32; 2]) -> [f32; 4] {
    let x = (uv[0].clamp(0., 1.) * (image.width() - 1) as f32).round() as u32;
    let y = (uv[1].clamp(0., 1.) * (image.height() - 1) as f32).round() as u32;
    image.get_pixel(x, y).0.map(|c| c as f32 / 255.)
}
impl Mesh {
    fn triangle(&mut self, v: [Vertex; 3], image: &image::RgbaImage, depth: u8) {
        let mid = [
            v[0].mix(v[1], 0.5),
            v[1].mix(v[2], 0.5),
            v[2].mix(v[0], 0.5),
        ];
        let alpha = v.map(|v| sample(image, v.uv)[3]);
        let tests = [
            alpha[0],
            alpha[1],
            alpha[2],
            sample(image, mid[0].uv)[3],
            sample(image, mid[1].uv)[3],
            sample(image, mid[2].uv)[3],
            sample(image, v[0].mix(v[1], 0.5).mix(v[2], 1. / 3.).uv)[3],
        ];
        let extent = (0..3)
            .map(|i| {
                let a = v[i].uv;
                let b = v[(i + 1) % 3].uv;
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
            })
            .fold(0., f32::max);
        if tests.iter().all(|&a| a < 0.2) && extent < 0.035 {
            return;
        }
        let opaque = tests.iter().all(|&a| a >= 0.2);
        if depth < 5 && extent > if opaque { 0.25 } else { 0.04 } {
            for tri in [
                [v[0], mid[0], mid[2]],
                [mid[0], v[1], mid[1]],
                [mid[2], mid[1], v[2]],
                [mid[0], mid[1], mid[2]],
            ] {
                self.triangle(tri, image, depth + 1);
            }
            return;
        }
        let mut polygon = Vec::new();
        for i in 0..3 {
            let a = v[i];
            let b = v[(i + 1) % 3];
            let inside = alpha[i] >= 0.2;
            if inside {
                polygon.push(a);
            }
            if inside != (alpha[(i + 1) % 3] >= 0.2) {
                let (mut low, mut high) = (0., 1.);
                for _ in 0..10 {
                    let t = (low + high) * 0.5;
                    if (sample(image, a.mix(b, t).uv)[3] >= 0.2) == inside {
                        low = t;
                    } else {
                        high = t;
                    }
                }
                polygon.push(a.mix(b, (low + high) * 0.5));
            }
        }
        for i in 1..polygon.len().saturating_sub(1) {
            let tri = [polygon[0], polygon[i], polygon[i + 1]];
            // Separate backs preserve the artist's curved normals from either side.
            for back in [false, true] {
                let mut ids = [0; 3];
                for (i, v) in tri.into_iter().enumerate() {
                    let paint = sample(image, v.uv);
                    let n = v.n.map(|n| if back { -n } else { n });
                    let length = n.iter().map(|n| n * n).sum::<f32>().sqrt().max(1e-6);
                    let n = n.map(|n| n / length);
                    let linear = |v: f32| {
                        if v <= 0.04045 {
                            v / 12.92
                        } else {
                            ((v + 0.055) / 1.055).powf(2.4)
                        }
                    };
                    let c = std::array::from_fn(|i| {
                        if i == 3 {
                            1.
                        } else {
                            linear(paint[i]) * v.c[i]
                        }
                    });
                    let key: [i32; 10] = std::array::from_fn(|i| {
                        let f = if i < 3 {
                            v.p[i]
                        } else if i < 6 {
                            n[i - 3]
                        } else {
                            c[i - 6]
                        };
                        (f * 10000.).round() as i32
                    });
                    ids[i] = *self.vertices.entry(key).or_insert_with(|| {
                        let id = self.positions.len() as u32;
                        self.positions.push(v.p);
                        self.normals.push(n);
                        self.colors.push(c);
                        id
                    });
                }
                if back {
                    ids.swap(1, 2);
                }
                self.indices.extend(ids);
            }
        }
    }
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    assert_eq!(
        args.len(),
        2,
        "expected extracted glTF directory and output directory"
    );
    let source = Path::new(&args[0]);
    let output = Path::new(&args[1]);
    std::fs::create_dir_all(output).unwrap();
    for (name, target) in [
        ("Fern_1", "fern"),
        ("Plant_1_Big", "woodland_stem"),
        ("Plant_7_Big", "clover"),
        ("Flower_3_Single", "wildflower"),
    ] {
        let g = gltf::Gltf::open(source.join(format!("{name}.gltf"))).unwrap();
        let buffers = g
            .buffers()
            .map(|b| {
                let gltf::buffer::Source::Uri(uri) = b.source() else {
                    panic!("expected external source buffer")
                };
                std::fs::read(source.join(uri)).unwrap()
            })
            .collect::<Vec<_>>();
        let mut mesh = Mesh::default();
        for primitive in g.meshes().flat_map(|m| m.primitives()) {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let p = reader.read_positions().unwrap().collect::<Vec<_>>();
            let n = reader.read_normals().unwrap().collect::<Vec<_>>();
            let uv = reader
                .read_tex_coords(0)
                .unwrap()
                .into_f32()
                .collect::<Vec<_>>();
            let c = reader
                .read_colors(0)
                .map(|c| c.into_rgba_f32().collect::<Vec<_>>())
                .unwrap_or_else(|| vec![[1.; 4]; p.len()]);
            let indices = reader
                .read_indices()
                .unwrap()
                .into_u32()
                .collect::<Vec<_>>();
            let texture = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .unwrap();
            let gltf::image::Source::Uri { uri, .. } = texture.texture().source().source() else {
                panic!("expected source texture")
            };
            let image = image::open(source.join(uri)).unwrap().to_rgba8();
            for ids in indices.chunks_exact(3) {
                mesh.triangle(
                    std::array::from_fn(|i| {
                        let j = ids[i] as usize;
                        Vertex {
                            p: p[j],
                            n: n[j],
                            uv: uv[j],
                            c: c[j],
                        }
                    }),
                    &image,
                    0,
                );
            }
        }
        // Ground the source and give all species a unit horizontal radius.
        let bottom = mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let radius = mesh
            .positions
            .iter()
            .map(|p| (p[0] * p[0] + p[2] * p[2]).sqrt())
            .fold(0., f32::max);
        for p in &mut mesh.positions {
            *p = [p[0] / radius, (p[1] - bottom) / radius, p[2] / radius];
        }
        let json = serde_json::json!({"positions":mesh.positions,"normals":mesh.normals,
            "colors":mesh.colors,"indices":mesh.indices});
        std::fs::write(
            output.join(format!("{target}.mesh.json")),
            json.to_string() + "\n",
        )
        .unwrap();
        println!(
            "{name}: {} vertices, {} triangles",
            mesh.positions.len(),
            mesh.indices.len() / 3
        );
    }
}
