//! Adapt three CC0 Quaternius CommonTree models for the solid terrain renderer.
//! Usage: import_deciduous_models <Standard pack>/glTF <output directory>
//! Preserve authored trunks, forks and crown placement; replace square leaf
//! cards with closed, cupped leaves. No custom leaf shader or texture is shipped.
use bevy::prelude::Vec3;
use std::path::Path;
#[derive(Default)]
struct Mesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}
fn linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn paint(image: &image::RgbaImage, uv: [f32; 2], color: [f32; 4]) -> [f32; 4] {
    let x = (uv[0].rem_euclid(1.) * (image.width() - 1) as f32).round() as u32;
    let y = (uv[1].rem_euclid(1.) * (image.height() - 1) as f32).round() as u32;
    let p = image.get_pixel(x, y).0;
    std::array::from_fn(|i| {
        if i == 3 {
            1.
        } else {
            linear(p[i] as f32 / 255.) * color[i]
        }
    })
}
fn leaf(mesh: &mut Mesh, corners: [Vec3; 4], color: [f32; 4]) {
    let center = corners.into_iter().sum::<Vec3>() * 0.25;
    let u = (corners[1] - corners[0] + corners[2] - corners[3]) * 0.25;
    let v = (corners[3] - corners[0] + corners[2] - corners[1]) * 0.25;
    let normal = u.cross(v).normalize();
    let size = u.length().min(v.length());
    let mut points = [center; 12];
    for (i, p) in points[..10].iter_mut().enumerate() {
        let angle = i as f32 * std::f32::consts::TAU / 10.;
        // Soft shoulders, a tapered tip and a cupped midrib give the individual
        // leaves a solid outline from above, below and oblique camera angles.
        let along = angle.sin();
        let across = angle.cos();
        let taper = 1. - 0.20 * along.max(0.).powi(2);
        *p = center
            + u * across * 0.94 * taper
            + v * along
            + normal * size * (0.045 * along * along - 0.025 * across * across);
    }
    points[10] = center + normal * size * 0.14;
    points[11] = center - normal * size * 0.028;
    let first = mesh.positions.len() as u32;
    let mut normals = [Vec3::ZERO; 12];
    for edge in 0..10 {
        for apex in [10, 11] {
            let mut ids = [apex, edge, (edge + 1) % 10];
            let mut n = (points[ids[1]] - points[ids[0]]).cross(points[ids[2]] - points[ids[0]]);
            if n.dot((points[ids[0]] + points[ids[1]] + points[ids[2]]) / 3. - center) < 0. {
                ids.swap(1, 2);
                n = -n;
            }
            mesh.indices.extend(ids.map(|i| first + i as u32));
            for i in ids {
                normals[i] += n;
            }
        }
    }
    for (i, (p, n)) in points.into_iter().zip(normals).enumerate() {
        mesh.positions.push(p.to_array());
        mesh.normals.push(n.normalize().to_array());
        let light = if i == 10 { 1.035 } else { 0.99 };
        mesh.colors
            .push([color[0] * light, color[1] * light, color[2] * light, 1.]);
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
        ("CommonTree_1", "spreading"),
        ("CommonTree_2", "forked"),
        ("CommonTree_4", "upright"),
    ] {
        let g = gltf::Gltf::open(source.join(format!("{name}.gltf"))).unwrap();
        assert!(g.nodes().all(|n|n.transform().matrix()==bevy::prelude::Mat4::IDENTITY.to_cols_array_2d()),"expected untransformed authored model");
        let buffers = g
            .buffers()
            .map(|b| {
                let gltf::buffer::Source::Uri(uri) = b.source() else {
                    panic!("expected source buffer")
                };
                std::fs::read(source.join(uri)).unwrap()
            })
            .collect::<Vec<_>>();
        let mut mesh = Mesh::default();
        let mut foliage_start = None;
        for primitive in g.meshes().flat_map(|m| m.primitives()) {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let p = reader
                .read_positions()
                .unwrap()
                .map(Vec3::from_array)
                .collect::<Vec<_>>();
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
            if primitive.material().name().unwrap().starts_with("Leaves") {
                foliage_start = Some(mesh.positions.len());
                assert_eq!(indices.len() % 6, 0);
                for quad in indices.chunks_exact(6) {
                    let mut edges = std::collections::BTreeMap::new();
                    for tri in quad.chunks_exact(3) {
                        for i in 0..3 {
                            let (a, b) = (tri[i], tri[(i + 1) % 3]);
                            *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                        }
                    }
                    let edges = edges
                        .into_iter()
                        .filter_map(|(e, count)| (count == 1).then_some(e))
                        .collect::<Vec<_>>();
                    assert_eq!(edges.len(), 4, "each authored leaf card must be a quad");
                    let mut ring = [edges[0].0; 4];
                    ring[1] = edges[0].1;
                    for i in 2..4 {
                        ring[i] = edges
                            .iter()
                            .find_map(|&(a, b)| {
                                if a == ring[i - 1] && b != ring[i - 2] {
                                    Some(b)
                                } else if b == ring[i - 1] && a != ring[i - 2] {
                                    Some(a)
                                } else {
                                    None
                                }
                            })
                            .unwrap();
                    }
                    let leaf_uv: [f32; 2] = std::array::from_fn(|j| {
                        ring.iter().map(|&i| uv[i as usize][j]).sum::<f32>() * 0.25
                    });
                    let leaf_tint: [f32; 4] = std::array::from_fn(|j| {
                        ring.iter().map(|&i| c[i as usize][j]).sum::<f32>() * 0.25
                    });
                    // The source card's corners are transparent black. Sample
                    // its painted center, retaining the artist's per-leaf tint.
                    leaf(
                        &mut mesh,
                        ring.map(|i| p[i as usize]),
                        paint(&image, leaf_uv, leaf_tint),
                    );
                }
            } else {
                assert!(foliage_start.is_none(), "trunk primitives precede leaves");
                let first = mesh.positions.len() as u32;
                mesh.positions.extend(p.iter().map(|p| p.to_array()));
                mesh.normals.extend(n);
                mesh.colors
                    .extend(uv.into_iter().zip(c).map(|(uv, c)| paint(&image, uv, c)));
                mesh.indices.extend(indices.iter().map(|i| first + i));
            }
        }
        let bottom = mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let radius = mesh
            .positions
            .iter()
            .map(|p| p[0].hypot(p[2]))
            .fold(0., f32::max);
        for p in &mut mesh.positions {
            *p = [p[0] / radius, (p[1] - bottom) / radius, p[2] / radius];
        }
        let json = serde_json::json!({"positions":mesh.positions,"normals":mesh.normals,"colors":mesh.colors,"indices":mesh.indices,"foliage_start":foliage_start.unwrap()});
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
