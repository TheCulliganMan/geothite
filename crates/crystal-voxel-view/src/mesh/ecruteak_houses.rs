//! Ecruteak's source-selected wooden houses: cedar courses, pale infill and
//! narrow lattice openings. All features stay inside the existing source plot.
use super::*;
#[path = "ecruteak_roof.rs"]
mod roof;
#[path = "ecruteak_ruin_roof.rs"]
mod ruin_roof;
#[path = "ecruteak_timber.rs"]
mod timber;
#[path = "ecruteak_eaves.rs"]
mod eaves;

const WOOD: [f32; 3] = [0.30, 0.18, 0.075];
const EDGE: [f32; 3] = [0.43, 0.28, 0.12];
const ROOF: [f32; 3] = [0.17, 0.14, 0.085];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FacadePaint {
    pub wall: [f32;3],
    pub timber: [f32;3],
    pub shadow: [f32;3],
    pub window: [f32;3],
}

fn finish_facade(mesh: &mut SurfaceMeshData, first: usize, b: [f32;4], t: f32,
    paint: Option<FacadePaint>) {
    let Some(paint)=paint else {return;};
    let light=Vec3::new(-0.35,0.85,0.4).normalize();
    for i in first..mesh.positions.len() {
        // The separately painted ceramic courses own their existing tags.
        if crate::scenery_finish::town_paint_kind(mesh.uvs[i]).is_some() {continue;}
        if crate::scenery_finish::town_timber_coordinates(mesh.uvs[i]).is_some() {continue;}
        if crate::city_lights::source_window_tag(mesh.uvs[i]) {continue;}
        let p=mesh.positions[i];let n=Vec3::from_array(mesh.normals[i]);
        let shade=0.62+0.38*n.dot(light).max(0.);
        let plaster=(0..3).all(|c|(mesh.colors[i][c]-paint.wall[c]*shade).abs()<0.00001);
        mesh.uvs[i]=if plaster {
            let (u,v)=if n.y.abs()>0.5 {((p[0]-b[0])/t,(p[2]-b[2])/t)}
                else if n.x.abs()>n.z.abs() {((p[2]-b[2])/t,p[1]/t)}else {((p[0]-b[0])/t,p[1]/t)};
            [u,crate::scenery_finish::TOWN_PAINT_BASE+32.+v]
        } else {crate::scenery_finish::SOURCE_PAINT_UV};
    }
}

fn window(mesh: &mut SurfaceMeshData, b: [f32;6], color: [f32;3]) {
    let first=mesh.positions.len();
    block(mesh,b,color);
    crate::city_lights::mark_window(mesh,first..mesh.positions.len(),first,true);
}

fn block(mesh: &mut SurfaceMeshData, b: [f32; 6], color: [f32; 3]) {
    super::city_facades::block(mesh, b, color);
}

// The source facades have a continuous timber base with narrow upright
// boards. Keep it below the windows and omit the authored doorway span.
fn timber_base(mesh: &mut SurfaceMeshData, left: f32, right: f32, front: f32, t: f32, facade: Option<FacadePaint>) {
    let timber_block=|mesh: &mut SurfaceMeshData, member, color| timber::block(mesh,member,color,t,[left,right,front,front],facade.is_some());
    let wood=facade.map(|p|p.timber.map(|v|v*0.65)).unwrap_or(WOOD);
    let edge=facade.map(|p|p.timber).unwrap_or(EDGE);
    timber_block(mesh, [left, right, t * 0.15, t * 0.88, front, front + t * 0.06], wood);
    let boards = ((right - left) / (t * 0.25)).ceil() as usize;
    for i in 0..boards {
        let x = left + (right - left) * i as f32 / boards as f32;
        timber_block(mesh, [x, (x + t * 0.06).min(right), t * 0.15, t * 0.88,
            front + t * 0.06, front + t * 0.09], edge);
    }
    timber_block(mesh, [left, right, t * 0.88, t * 0.99, front, front + t * 0.10], edge);
}

fn face(mesh: &mut SurfaceMeshData, p: &[[f32; 3]], color: [f32; 3]) {
    let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
        .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
        .normalize();
    let shade = 0.62 + 0.38 * normal.dot(Vec3::new(-0.35, 0.85, 0.4).normalize()).max(0.);
    let first = mesh.positions.len() as u32;
    for &v in p {
        mesh.positions.push(v);
        mesh.normals.push(normal.to_array());
        mesh.uvs.push([0., 0.]);
        mesh.colors
            .push([color[0] * shade, color[1] * shade, color[2] * shade, 1.]);
    }
    for i in 1..p.len() - 1 {
        mesh.indices
            .extend([first, first + i as u32, first + i as u32 + 1]);
    }
}

fn timber_gable(mesh: &mut SurfaceMeshData, x: f32, back: f32, front: f32,
    eave: f32, ridge: f32, t: f32, color: [f32;3], outward: f32) {
    let middle=(back+front)*0.5;
    let height=|z:f32|eave+(ridge-eave)*(1.-((z-middle)/((front-back)*0.5)).abs());
    let count=((front-back)/(t*0.4)).ceil() as usize;
    for i in 0..count {
        let a=back+(front-back)*i as f32/count as f32+t*0.008;
        let b=back+(front-back)*(i+1) as f32/count as f32-t*0.008;
        let mut outline=vec![(a,eave),(b,eave),(b,height(b))];
        if a<middle && b>middle {outline.push((middle,ridge));}
        outline.push((a,height(a)));
        let n=outline.len();let mut points=Vec::new();
        for plane in [x-outward*t*0.004,x+outward*t*0.025] {
            points.extend(outline.iter().map(|&(z,y)|Vec3::new(plane,y,z)));
        }
        let center=points.iter().copied().sum::<Vec3>()/points.len() as f32;
        let mut faces=vec![(0..n).collect::<Vec<_>>(),(n..n*2).collect::<Vec<_>>()];
        for j in 0..n {faces.push(vec![j,(j+1)%n,(j+1)%n+n,j+n]);}
        let color=color.map(|v|v*(0.96+(i%5) as f32*0.02));
        for ids in faces {
            let mut p=ids.iter().map(|&j|points[j]).collect::<Vec<_>>();
            if (p[1]-p[0]).cross(p[2]-p[0]).dot(p.iter().copied().sum::<Vec3>()/p.len() as f32-center)<0. {p.reverse();}
            face(mesh,&p.iter().map(|p|p.to_array()).collect::<Vec<_>>(),color);
        }
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, door: Option<f32>, roof_palette: Option<[[f32;4];3]>, facade: Option<FacadePaint>) {
    let timber_block=|mesh: &mut SurfaceMeshData, member, color| timber::block(mesh,member,color,t,b,facade.is_some());
    let first=mesh.positions.len();
    let wood=facade.map(|p|p.timber.map(|v|v*0.65)).unwrap_or(WOOD);
    let edge=facade.map(|p|p.timber).unwrap_or(EDGE);
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.28, e - t * 0.28, n + t * 0.38, s - t * 0.46);
    let (base, eave, ridge) = (t * 0.18, t * 2.7, t * 4.2);
    let middle = (back + front) * 0.5;
    for z in [back - t * 0.08, front] {
        timber_block(
            mesh,
            [left, right, t * 2.40, t * 2.52, z, z + t * 0.08],
            wood,
        );
    }
    for x in [left - t * 0.08, right] {
        timber_block(
            mesh,
            [x, x + t * 0.08, t * 2.40, t * 2.52, back, front],
            wood,
        );
    }
    block(
        mesh,
        [left, right, 0., base, back, front],
        facade.map(|p|p.shadow).unwrap_or([0.38, 0.36, 0.29]),
    );
    block(
        mesh,
        [left, right, base, eave, back, front],
        facade.map(|p|p.wall).unwrap_or([0.77, 0.79, 0.62]),
    );
    // Close the gables independently of the overlapping roof courses.
    face(
        mesh,
        &[
            [left, eave, back],
            [left, eave, front],
            [left, ridge, middle],
        ],
        wood,
    );
    face(
        mesh,
        &[
            [right, eave, front],
            [right, eave, back],
            [right, ridge, middle],
        ],
        wood,
    );
    let (rn, rs) = (n + t * 0.10, s - t * 0.16);
    timber_gable(mesh,left,back,front,eave,ridge,t,wood,-1.);
    timber_gable(mesh,right,back,front,eave,ridge,t,wood,1.);
    roof::append(mesh,[w+t*0.06,e-t*0.06,rn,rs],t,eave,ridge,roof_palette);
    eaves::append(mesh,[w+t*0.06,e-t*0.06,rn,rs],[left,right,back,front],
        t,eave,ridge,edge,facade.is_some());
    timber_block(
        mesh,
        [left, right, eave - t * 0.12, eave, front, front + t * 0.10],
        edge,
    );
    // Wood wainscot wraps every side; seams are geometry, rather than paint.
    for i in 0..28 {
        let xa = left + (right - left) * i as f32 / 28.;
        let xb = left + (right - left) * (i + 1) as f32 / 28. - t * 0.018;
        let color = edge.map(|v| v * (0.90 + (i % 4) as f32 * 0.055));
        for z in [back - t * 0.045, front] {
            timber_block(mesh, [xa, xb, base, t * 0.90, z, z + t * 0.045], color);
        }
    }
    for i in 0..18 {
        let za = back + (front - back) * i as f32 / 18.;
        let zb = back + (front - back) * (i + 1) as f32 / 18. - t * 0.018;
        for x in [left - t * 0.045, right] {
            timber_block(mesh, [x, x + t * 0.045, base, t * 0.90, za, zb], edge);
        }
    }
    for x in [left, right - t * 0.12] {
        for z in [back, front - t * 0.12] {
            timber_block(mesh, [x, x + t * 0.12, base, eave, z, z + t * 0.12], wood);
        }
    }
    let dx = door.unwrap_or((w + e) * 0.5);
    let half = t * 0.45;
    let dz = front + t * 0.055;
    timber_block(
        mesh,
        [dx - half, dx + half, base, t * 2.05, front, dz],
        wood,
    );
    window(
        mesh,
        [
            dx - half + t * 0.08,
            dx + half - t * 0.08,
            t * 0.34,
            t * 1.92,
            dz,
            dz + t * 0.015,
        ],
        facade.map(|p|p.window).unwrap_or([0.12, 0.11, 0.065]),
    );
    for i in 0..5 {
        let x = dx - half + t * 0.12 + i as f32 * t * 0.16;
        timber_block(
            mesh,
            [
                x,
                x + t * 0.035,
                t * 0.34,
                t * 1.92,
                dz + t * 0.02,
                dz + t * 0.05,
            ],
            edge,
        );
    }
    for y in [t * 0.8, t * 1.35, t * 1.88] {
        timber_block(
            mesh,
            [
                dx - half,
                dx + half,
                y,
                y + t * 0.035,
                dz + t * 0.02,
                dz + t * 0.05,
            ],
            edge,
        );
    }
    for x in [
        left + (dx - half - left) * 0.46,
        dx + half + (right - dx - half) * 0.54,
    ] {
        window(
            mesh,
            [
                x - t * 0.18,
                x + t * 0.18,
                t * 1.55,
                t * 2.30,
                front,
                front + t * 0.07,
            ],
            facade.map(|p|p.window).unwrap_or(wood),
        );
        for slit in [-0.10, 0.04] {
            block(
                mesh,
                [
                    x + t * slit,
                    x + t * (slit + 0.055),
                    t * 1.63,
                    t * 2.21,
                    front + t * 0.075,
                    front + t * 0.09,
                ],
                facade.map(|p|p.timber).unwrap_or([0.69, 0.68, 0.45]),
            );
        }
    }
    block(
        mesh,
        [dx - t * 0.62, dx + t * 0.62, 0., base, front, s],
        facade.map(|p|p.shadow).unwrap_or([0.48, 0.45, 0.34]),
    );
    finish_facade(mesh,first,b,t,facade);
}

// The town's shrine gate and ruined tower are separate source drawings.
// Keep their timber silhouettes distinct from the ordinary house and pagoda.
pub(super) fn append_gate(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, roof_palette: Option<[[f32;4];3]>, facade: Option<FacadePaint>) {
    let timber_block=|mesh: &mut SurfaceMeshData, member, color| timber::block(mesh,member,color,t,b,facade.is_some());
    let first=mesh.positions.len();
    let wood=facade.map(|p|p.timber.map(|v|v*0.65)).unwrap_or(WOOD);
    let edge=facade.map(|p|p.timber).unwrap_or(EDGE);
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.30, e - t * 0.30, n + t * 0.30, s - t * 0.35);
    let h = t * 3.6;
    let infill = facade.map(|p|p.wall).unwrap_or([0.76, 0.77, 0.59]);
    block(
        mesh,
        [left, right, 0., t * 0.15, back, front],
        facade.map(|p|p.shadow).unwrap_or([0.41, 0.35, 0.22]),
    );
    block(
        mesh,
        [left, right, t * 0.15, h, back, back + t * 0.12],
        infill,
    );
    block(
        mesh,
        [left, right, t * 0.15, h, front - t * 0.12, front],
        infill,
    );
    block(
        mesh,
        [left, left + t * 0.12, t * 0.15, h, back, front],
        infill,
    );
    // This blank-front drawing is shared by ordinary houses and the shrine
    // gate. It does not prove an open passage through every east wall. The
    // gate's actual east warp tiles lie beyond this inset building envelope.
    block(
        mesh,
        [right - t * 0.12, right, t * 0.15, h, back, front],
        infill,
    );
    for x in [
        left,
        left + (right - left) * 0.33,
        left + (right - left) * 0.66,
        right - t * 0.12,
    ] {
        timber_block(
            mesh,
            [x, x + t * 0.12, 0., h, front, front + t * 0.06],
            wood,
        );
    }
    for x in [
        left + (right - left) * 0.20,
        left + (right - left) * 0.50,
        left + (right - left) * 0.80,
    ] {
        window(
            mesh,
            [
                x - t * 0.11,
                x + t * 0.11,
                t * 2.0,
                t * 2.85,
                front,
                front + t * 0.05,
            ],
            facade.map(|p|p.window).unwrap_or([0.08, 0.065, 0.03]),
        );
        timber_block(
            mesh,
            [
                x - t * 0.025,
                x + t * 0.025,
                t * 2.0,
                t * 2.85,
                front + t * 0.055,
                front + t * 0.07,
            ],
            edge,
        );
    }
    timber_base(mesh, left, right, front, t, facade);
    low_roof(mesh, b, t, h, t * 4.0, false, roof_palette, Some(wood));
    eaves::append(mesh,[w+t*0.10,e-t*0.10,n+t*0.10,s-t*0.12],
        [left,right,back,front],t,h,t*4.0,edge,facade.is_some());
    finish_facade(mesh,first,b,t,facade);
}

pub(super) fn append_temple(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, door: Option<f32>, roof_palette: Option<[[f32;4];3]>, facade: Option<FacadePaint>) {
    let timber_block=|mesh: &mut SurfaceMeshData, member, color| timber::block(mesh,member,color,t,b,facade.is_some());
    let first=mesh.positions.len();
    let wood=facade.map(|p|p.timber.map(|v|v*0.65)).unwrap_or(WOOD);
    let edge=facade.map(|p|p.timber).unwrap_or(EDGE);
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.30, e - t * 0.30, n + t * 0.30, s - t * 0.35);
    let h = t * 3.9;
    let dx = door
        .unwrap_or((w + e) * 0.5)
        .clamp(left + t * 1.1, right - t * 1.1);
    let infill = facade.map(|p|p.wall).unwrap_or([0.76, 0.77, 0.59]);
    block(
        mesh,
        [left, right, 0., t * 0.15, back, front],
        facade.map(|p|p.shadow).unwrap_or([0.41, 0.35, 0.22]),
    );
    block(
        mesh,
        [left, right, t * 0.15, h, back, back + t * 0.12],
        infill,
    );
    for x in [left, right - t * 0.12] {
        block(mesh, [x, x + t * 0.12, t * 0.15, h, back, front], infill);
    }
    for (a, b) in [(left, dx - t), (dx + t, right)] {
        block(mesh, [a, b, t * 0.15, h, front - t * 0.12, front], infill);
    }
    timber_block(
        mesh,
        [dx - t, dx + t, t * 3.5, h, front - t * 0.12, front],
        wood,
    );
    block(
        mesh,
        [
            dx - t,
            dx + t,
            t * 0.15,
            t * 3.5,
            front - t * 0.40,
            front - t * 0.30,
        ],
        [0.06, 0.045, 0.024],
    );
    for x in [left, right - t * 0.12, dx - t, dx + t - t * 0.12] {
        timber_block(
            mesh,
            [x, x + t * 0.12, t * 0.15, h, front, front + t * 0.06],
            wood,
        );
    }
    for x in [w + t * 1.5, w + t * 7.2, e - t * 1.5] {
        if (x - dx).abs() < t * 1.2 {
            continue;
        }
        window(
            mesh,
            [
                x - t * 0.12,
                x + t * 0.12,
                t * 2.0,
                t * 3.1,
                front,
                front + t * 0.05,
            ],
            facade.map(|p|p.window).unwrap_or([0.08, 0.065, 0.03]),
        );
        timber_block(
            mesh,
            [
                x - t * 0.025,
                x + t * 0.025,
                t * 2.0,
                t * 3.1,
                front + t * 0.055,
                front + t * 0.07,
            ],
            edge,
        );
    }
    for bar in 0..6 {
        let x = dx - t * 0.84 + bar as f32 * t * 0.32;
        timber_block(
            mesh,
            [
                x,
                x + t * 0.06,
                t * 0.22,
                t * 3.38,
                front - t * 0.28,
                front - t * 0.20,
            ],
            edge,
        );
    }
    timber_base(mesh, left, dx - t, front, t, facade);
    timber_base(mesh, dx + t, right, front, t, facade);
    low_roof(mesh, b, t, h, t * 4.35, false, roof_palette, Some(wood));
    eaves::append(mesh,[w+t*0.10,e-t*0.10,n+t*0.10,s-t*0.12],
        [left,right,back,front],t,h,t*4.35,edge,facade.is_some());
    finish_facade(mesh,first,b,t,facade);
}

pub(super) fn append_burned(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, door: Option<f32>, facade: Option<FacadePaint>) {
    let first=mesh.positions.len();
    let timber_block=|mesh: &mut SurfaceMeshData, member, color| timber::block(mesh,member,color,t,b,facade.is_some());
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.30, e - t * 0.30, n + t * 0.30, s - t * 0.40);
    let h = t * 3.5;
    let ash=facade.map(|p|super::kanto_craft::mix(p.window,p.timber,0.04)).unwrap_or([0.055,0.046,0.033]);
    let wall=facade.map(|p|p.wall).unwrap_or([0.40,0.28,0.105]);
    let edge=facade.map(|p|p.timber).unwrap_or(EDGE);
    let dx = door
        .unwrap_or(w + (e - w) * 0.30)
        .clamp(left + t * 0.70, right - t * 0.70);
    block(
        mesh,
        [left, right, 0., t * 0.15, back, front],
        facade.map(|p|super::kanto_craft::mix(p.window,p.wall,0.12)).unwrap_or([0.28,0.25,0.18]),
    );
    block(
        mesh,
        [left, right, t * 0.15, h, back, back + t * 0.12],
        wall,
    );
    block(
        mesh,
        [left, left + t * 0.12, t * 0.15, h, back, front],
        wall,
    );
    block(
        mesh,
        [right - t * 0.12, right, t * 0.15, h, back, front],
        wall,
    );
    for (a, b) in [(left, dx - t * 0.55), (dx + t * 0.55, right)] {
        block(mesh, [a, b, t * 0.15, h, front - t * 0.12, front], wall);
    }
    block(
        mesh,
        [
            dx - t * 0.55,
            dx + t * 0.55,
            t * 3.0,
            h,
            front - t * 0.12,
            front,
        ],
        wall,
    );
    block(
        mesh,
        [
            dx - t * 0.55,
            dx + t * 0.55,
            t * 0.15,
            t * 3.0,
            front - t * 0.40,
            front - t * 0.30,
        ],
        facade.map(|p|p.window).unwrap_or([0.025,0.022,0.016]),
    );
    for i in 0..9 {
        let x = left + (right - left) * i as f32 / 8.;
        if (x - dx).abs() > t * 0.58 {
            timber_block(
                mesh,
                [
                    x - t * 0.04,
                    x + t * 0.04,
                    t * 0.15,
                    h,
                    front,
                    front + t * 0.05,
                ],
                ash,
            );
        }
    }
    for x in [left + t * 0.70, right - t * 0.70] {
        block(
            mesh,
            [
                x - t * 0.13,
                x + t * 0.13,
                t * 1.6,
                t * 2.65,
                front,
                front + t * 0.06,
            ],
            facade.map(|p|p.window).unwrap_or([0.035,0.031,0.020]),
        );
    }
    // Dark interior remains behind the missing roof courses, not white infill.
    block(
        mesh,
        [left, right, h - t * 0.22, h - t * 0.12, back, front],
        ash,
    );
    // Source plaster sits behind surviving cedar framing. Each member is
    // a closed chamfered solid; the doorway remains its original clear span.
    for (a,z) in [(left,dx-t*0.55),(dx+t*0.55,right)] {
        timber_base(mesh,a,z,front,t,facade);
    }
    for x in [dx-t*0.61,dx+t*0.55] {
        timber_block(mesh,[x,x+t*0.06,t*0.15,t*3.0,front,front+t*0.09],edge);
    }
    timber_block(mesh,[dx-t*0.61,dx+t*0.61,t*3.0,t*3.10,front,front+t*0.09],edge);
    timber_block(mesh,[left,right,h-t*0.20,h-t*0.08,front,front+t*0.09],edge);
    finish_facade(mesh,first,b,t,facade);
    low_roof(mesh, b, t, h, t * 3.95, true, None, None);
}

fn low_roof(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, eave: f32, ridge: f32, burned: bool, roof_palette: Option<[[f32;4];3]>, gable_wood: Option<[f32;3]>) {
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.10, e - t * 0.10, n + t * 0.10, s - t * 0.12);
    let middle = (back + front) * 0.5;
    let height =
        |z: f32| eave + (ridge - eave) * (1. - ((z - middle) / ((front - back) * 0.5)).abs());
    if !burned {roof::append(mesh,[left,right,back,front],t,eave,ridge,roof_palette);}
    else {ruin_roof::append(mesh,[left,right,back,front],t,eave,ridge);}
    for (x, reverse) in [(left, false), (right, true)] {
        let mut p = vec![
            [x, eave - t * 0.08, back],
            [x, eave - t * 0.08, front],
            [x, ridge, middle],
        ];
        if reverse {
            p.reverse();
        }
        let wood=gable_wood.unwrap_or(WOOD);
        face(mesh, &p, if burned { [0.06, 0.05, 0.03] } else { wood });
        if !burned {timber_gable(mesh,x,back,front,eave-t*0.08,ridge,t,wood,
            if reverse {1.}else{-1.});}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_front_houses_have_complete_side_walls_inside_the_warp_boundary() {
        for t in [8., 16.] {
            let b = [-4. * t, 4. * t, -3. * t, 3. * t];
            let mut mesh = SurfaceMeshData::default();
            append_gate(&mut mesh, b, t, None, None);
            let right = b[1] - t * 0.30;
            for y in [0.3, 1.2, 2.5, 3.4] {
                for z in [-2.5, -1., 0., 1., 2.5] {
                    let point = Vec3::new(right, y * t, z * t);
                    let covered = mesh.indices.chunks_exact(3).any(|ids| {
                        let [a, b, c] = [ids[0], ids[1], ids[2]]
                            .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                        if (a.x - right).abs() > 0.0001 || (b.x - right).abs() > 0.0001
                            || (c.x - right).abs() > 0.0001 { return false; }
                        let n = (b - a).cross(c - a);
                        n.x > 0. && [(b - a).cross(point - a), (c - b).cross(point - b),
                            (a - c).cross(point - c)].iter().all(|edge| edge.dot(n) >= -0.0001)
                    });
                    assert!(covered, "missing east plaster at height {y}, depth {z}");
                }
            }
            assert!(mesh.positions.iter().all(|p| p[0] < b[1]),
                "the east-side warp remains beyond the inset building envelope");
        }
    }

    #[test]
    fn timber_gable_planks_are_closed_and_stay_inside_the_original_triangle() {
        for t in [8.,16.] {for side in [-1.,1.] {
            let mut mesh=SurfaceMeshData::default();let x=side*4.*t;
            let back=-3.*t;let front=3.*t;let eave=2.7*t;let ridge=4.2*t;
            timber_gable(&mut mesh,x,back,front,eave,ridge,t,WOOD,side);
            let mut edges=std::collections::BTreeMap::new();let mut volume=0.;
            for p in &mesh.positions {
                let roof=eave+(ridge-eave)*(1.-(p[2]/(3.*t)).abs());
                assert!(p[2]>=back && p[2]<=front && p[1]>=eave && p[1]<=roof+0.0001);
                assert!((p[0]-x).abs()<=t*0.026);
            }
            for ids in mesh.indices.chunks_exact(3) {
                let p=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                volume+=p[0].dot(p[1].cross(p[2]))/6.;
                assert!((p[1]-p[0]).cross(p[2]-p[0]).length_squared()>0.);
                for e in 0..3 {let mut edge=[p[e].to_array().map(f32::to_bits),p[(e+1)%3].to_array().map(f32::to_bits)];
                    edge.sort();*edges.entry(edge).or_insert(0)+=1;}
            }
            assert!(volume>0.);assert!(edges.values().all(|&count|count==2));
            assert!(mesh.indices.len()/3<300);
        }}
    }

    #[test]
    fn ruined_facade_uses_live_plaster_and_grained_joinery_without_repainting_its_roof() {
        let b=[-32.,32.,-32.,32.];
        for paint in [
            FacadePaint {wall:[0.7,0.95,0.7],timber:[0.4,0.23,0.02],shadow:[0.04;3],window:[0.;3]},
            FacadePaint {wall:[0.18,0.15,0.45],timber:[0.06,0.02,0.03],shadow:[0.006;3],window:[0.;3]},
        ] {
            let mut original=SurfaceMeshData::default();
            block(&mut original,[-80.,-72.,0.,8.,-80.,-72.],[0.2,0.3,0.4]);
            let mut plain=original.clone();let mut painted=original.clone();
            append_burned(&mut plain,b,8.,Some(-8.),None);
            append_burned(&mut painted,b,8.,Some(-8.),Some(paint));
            assert_eq!(plain.positions,painted.positions);assert_eq!(plain.normals,painted.normals);
            assert_eq!(plain.indices,painted.indices);
            let prefix=original.positions.len();
            assert_eq!(&painted.colors[..prefix],original.colors.as_slice());
            assert_eq!(&painted.uvs[..prefix],original.uvs.as_slice());
            let mut plaster=0;let mut timber=0;let mut roof=0;
            for i in prefix..painted.positions.len() {
                let uv=painted.uvs[i];
                if uv==crate::scenery_finish::SOURCE_PAINT_UV
                    && (painted.positions[i][1]-(8.*3.5-8.*0.12)).abs()<0.0001
                    && painted.normals[i][1]>0.99 {
                    assert!(painted.colors[i][..3].iter().all(|v|*v<0.02),
                        "the burned interior must stay dark beneath the roof openings");
                }
                if crate::scenery_finish::town_paint_kind(uv)==Some(1) {
                    plaster+=1;
                    let c=painted.colors[i];assert!((c[0]/c[1]-paint.wall[0]/paint.wall[1]).abs()<0.00001);
                } else if crate::scenery_finish::town_timber_coordinates(uv).is_some() {timber+=1;}
                else if uv==[0.,0.] {roof+=1;assert_eq!(painted.colors[i],plain.colors[i]);}
                else {assert_eq!(uv,crate::scenery_finish::SOURCE_PAINT_UV);}
            }
            assert!(plaster>=24 && timber>200 && roof>1000);
        }
    }

    #[test]
    fn live_facade_paint_preserves_roof_geometry_and_preceding_scenery() {
        let paint=FacadePaint {wall:[0.6,0.9,0.6],timber:[0.4,0.2,0.02],
            shadow:[0.04;3],window:[0.;3]};
        let roof=Some([[0.04;4],[0.1;4],[0.3,0.2,0.01,1.]]);
        for kind in 0..3 {
            let mut views=Vec::new();
            for facade in [None,Some(paint)] {
                let mut mesh=SurfaceMeshData::default();
                block(&mut mesh,[-90.,-80.,0.,8.,-90.,-80.],[0.2,0.3,0.4]);
                let b=[-64.,64.,-48.,48.];
                match kind {
                    0=>append(&mut mesh,b,16.,Some(-16.),roof,facade),
                    1=>append_gate(&mut mesh,b,16.,roof,facade),
                    _=>append_temple(&mut mesh,b,16.,Some(-16.),roof,facade),
                }
                views.push(mesh);
            }
            assert_eq!(views[0].positions,views[1].positions);
            assert_eq!(views[0].normals,views[1].normals);
            assert_eq!(views[0].indices,views[1].indices);
            assert_eq!(&views[0].colors[..24],&views[1].colors[..24]);
            assert_eq!(&views[0].uvs[..24],&views[1].uvs[..24]);
            let mut plaster=0;let mut joinery=0;let mut carved=0;
            for i in 24..views[0].positions.len() {
                let before=views[0].uvs[i];let after=views[1].uvs[i];
                if crate::scenery_finish::town_paint_kind(before)==Some(0) {
                    assert_eq!(before,after);assert_eq!(views[0].colors[i],views[1].colors[i]);
                } else if crate::scenery_finish::town_paint_kind(after)==Some(1) {
                    let c=views[1].colors[i];assert!((c[0]/c[1]-paint.wall[0]/paint.wall[1]).abs()<0.00001);plaster+=1;
                } else {assert!(after==crate::scenery_finish::SOURCE_PAINT_UV
                    || crate::scenery_finish::town_timber_coordinates(after).is_some()
                    || crate::city_lights::source_window_tag(after));
                    if crate::scenery_finish::town_timber_coordinates(after).is_some() {carved+=1;}
                    joinery+=1;}
                assert_eq!(views[1].colors[i][3],views[0].colors[i][3]);
            }
            assert!(plaster>=24 && joinery>100 && carved>100);
        }
    }

    #[test]
    fn intact_ecruteak_roofs_follow_live_paint_without_repainting_facades_or_changing_geometry() {
        let blue=[[0.05,0.10,0.40,1.],[0.08,0.16,0.64,1.],[0.10,0.20,0.80,1.]];
        let red=[[0.40,0.10,0.05,1.],[0.64,0.16,0.08,1.],[0.80,0.20,0.10,1.]];
        for kind in 0..3 {
            let mut views=Vec::new();
            for palette in [blue,red] {
                let mut mesh=SurfaceMeshData::default();let b=[-64.,64.,-48.,48.];
                match kind {
                    0=>append(&mut mesh,b,16.,Some(-16.),Some(palette),None),
                    1=>append_gate(&mut mesh,b,16.,Some(palette),None),
                    _=>append_temple(&mut mesh,b,16.,Some(-16.),Some(palette),None),
                }
                views.push(mesh);
            }
            assert_eq!(views[0].positions,views[1].positions);
            assert_eq!(views[0].normals,views[1].normals);
            assert_eq!(views[0].indices,views[1].indices);
            assert_eq!(views[0].uvs,views[1].uvs);
            let mut roof=0;let mut facade=0;
            for i in 0..views[0].positions.len() {
                let a=views[0].colors[i];let b=views[1].colors[i];
                if crate::scenery_finish::town_paint_kind(views[0].uvs[i])==Some(0) {
                    assert!(a[2]>a[0]*7. && b[0]>b[2]*7.);roof+=1;
                } else {assert_eq!(a,b);facade+=1;}
            }
            assert!(roof>1000 && facade>100);
        }
    }

    #[test]
    fn shrine_and_ruin_keep_source_plots_and_compact_closed_roofs() {
        for t in [8., 16.] {
            for kind in 0..3 {
                let b = if kind == 0 {
                    [-4. * t, 4. * t, -4. * t, 4. * t]
                } else {
                    [-6. * t, 6. * t, -3. * t, 3. * t]
                };
                let mut m = SurfaceMeshData::default();
                match kind {
                    0 => append_burned(&mut m, b, t, Some(-t), None),
                    1 => append_gate(&mut m, b, t, None, None),
                    _ => append_temple(&mut m, b, t, Some(-t), None, None),
                }
                assert!(m.positions.iter().all(|p| p[0] >= b[0]
                    && p[0] <= b[1]
                    && p[2] >= b[2]
                    && p[2] <= b[3]
                    && p[1] >= 0.
                    && p[1] <= t * 4.5));
                for tri in m.indices.chunks_exact(3) {
                    let p =
                        [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                    assert!(
                        (p[1] - p[0])
                            .cross(p[2] - p[0])
                            .dot(Vec3::from_array(m.normals[tri[0] as usize]))
                            > 0.
                    );
                }
            }
        }
    }

    #[test]
    fn wooden_house_stays_inside_source_plot_with_outward_faces() {
        for t in [8., 16.] {
            let b = [-4. * t, 4. * t, -3. * t, 3. * t];
            let mut mesh = SurfaceMeshData::default();
            append(&mut mesh, b, t, Some(-t), None, None);
            assert!(mesh.positions.iter().all(|p| p[0] >= b[0]
                && p[0] <= b[1]
                && p[2] >= b[2]
                && p[2] <= b[3]
                && p[1] >= 0.));
            for tri in mesh.indices.chunks_exact(3) {
                let p =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = Vec3::from_array(mesh.normals[tri[0] as usize]);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.);
            }
        }
    }
}
