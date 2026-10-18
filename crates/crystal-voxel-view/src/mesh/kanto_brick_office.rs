//! Shared brick building: 20/54/21 roof, repeated 68/7f/69 stories, native base.
//! The same source drawing serves Pallet, Celadon, Saffron and Fuchsia.
use super::*;
use super::kanto_craft::{block, mix};

// A repeated story facade can begin at the map edge without a pictured roof.
// Its adjacent $53/$3a entrance bays distinguish it from service buildings.
pub(super) fn double_entrance_candidate(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement)->bool {
    p.height>=4 && cells.get(p.row*g.width+p.column).is_some_and(|t|
        t.source.tileset_id.as_ref()=="kanto" && t.source.metatile_id==0x68)
        && (0..p.width).step_by(4).any(|x|cells.get((p.row+p.height-4)*g.width+p.column+x)
            .is_some_and(|t|t.source.metatile_id==0x53))
}
pub(super) fn matches(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> bool {
    if !(8..=20).contains(&p.width) || p.width % 4 != 0
        || !(8..=32).contains(&p.height) || p.height % 4 != 0 || p.roof_rows != 4
        || p.column + p.width > g.width || p.row + p.height > g.height { return false; }
    let twin=double_entrance_candidate(cells,g,p);
    if twin && p.height>28 {return false;}
    let at = |x: usize, y: usize| cells.get((p.row+y)*g.width+p.column+x).map(|c| &c.source);
    let base_y = p.height-4;
    let mut doors = Vec::new();
    let mut twin_bay=None;
    for x in (0..p.width).step_by(4) {
        let roof = if x==0 {0x20} else if x+4==p.width {0x21} else {0x54};
        let upper = if x==0 {0x68} else if x+4==p.width {0x69} else {0x7f};
        let Some(base) = at(x,base_y).map(|s|s.metatile_id) else {return false;};
        let retail=x+4==p.width && base==0x73 && p.height>8 && !twin;
        if (x==0 && !matches!(base,0x37|0x7c)) || (x+4==p.width && base!=0x7e && !retail)
            || (x>0 && x+4<p.width && !matches!(base,0x3a|0x7d) && !(twin&&base==0x53)) {return false;}
        if retail && ![(0,2,0x44),(1,2,0x45),(0,3,0x4a),(1,3,0x4a)]
            .into_iter().all(|(dx,dy,tile)|at(x+dx,base_y+dy).is_some_and(|s|s.tile_index==tile)) {return false;}
        if matches!(base,0x7c|0x3a|0x53) {
            if base==0x53 {if twin_bay.replace(x).is_some() {return false;}}
            doors.push(x+if matches!(base,0x7c|0x53) {2} else {0});
        }
        for y in 0..p.height {for dx in 0..4 {
            if !at(x+dx,y).is_some_and(|s|s.tileset_id.as_ref()=="kanto"
                && s.metatile_id==if !twin && y<4 {roof} else if y<base_y {upper} else {base}
                && s.subtile_row as usize==y%4 && s.subtile_column as usize==dx) {return false;}
        }}
    }
    if twin {
        if doors.len()!=2 || doors[1]!=doors[0]+2 || twin_bay.map(|x|x+2)!=doors.first().copied() {return false;}
    } else if doors.len()>1 {return false;}
    for &x in &doors {
        for (dx,dy,tile) in [(0,0,0x0b),(1,0,0x0c),(0,1,0x1b),(1,1,0x1c)] {
            if !at(x+dx,p.height-2+dy).is_some_and(|s|s.tile_index==tile) {return false;}
        }
    }
    (0..p.width).filter(|&x|at(x,p.height-1).is_some_and(|s|s.tile_index==0x1b)).count()==doors.len()
}

#[derive(Clone)]
pub(super) struct Paint {
    source_height: usize,
    extra_doors: Vec<f32>,
    retail_sign: bool,
    roof: [[f32; 3]; 2],
    brick: [f32; 3],
    mortar: [f32; 3],
    glass: [f32; 3],
    pub(super) windows: Vec<[f32; 3]>,
    lit_windows: Vec<bool>,
    pane_cells: Vec<[usize;2]>,
    door: [f32; 3],
    ink: [f32; 3],
}
fn linear(c: [u8; 3]) -> [f32; 3] {
    c.map(|v| {
        let x = f32::from(v) / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    })
}
fn mark_source_light(out: &mut SurfaceMeshData, first: usize, lit: bool) {
    // $0a is an authored lamp and $32 is deliberately dark glass. Retain that
    // identity even when daylight or dusk makes the sampled pane dark; the
    // shared RTC material controls when the retained lamp actually turns on.
    if lit {
        crate::city_lights::mark_window(out,first..out.positions.len(),first,true);
    }
}
pub(super) fn paint(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
    images: &TerrainImageSamples,
) -> Option<Paint> {
    let sample = |x: usize, y: usize, tile: u16| {
        let cell = cells.get((p.row + y) * g.width + p.column + x)?;
        if cell.source.tileset_id.as_ref() != "kanto" || cell.source.tile_index != tile {
            return None;
        }
        let data = tile_rgba(images, cell).ok()?;
        (data.len() == 256 && data.chunks_exact(4).all(|p| p[3] == 255)).then_some(data)
    };
    let two = |rgba: &[u8]| -> Option<[[f32; 3]; 2]> {
        let colors = rgba
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2]])
            .collect::<std::collections::BTreeSet<_>>();
        if colors.len() != 2 {
            return None;
        }
        let mut c = colors.into_iter().collect::<Vec<_>>();
        c.sort_by_key(|c| u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722);
        Some([linear(c[0]), linear(c[1])])
    };
    let pixel = |rgba: &[u8], x: usize, y: usize| {
        linear(
            rgba[(y * 8 + x) * 4..(y * 8 + x) * 4 + 3]
                .try_into()
                .unwrap(),
        )
    };
    let wall = two(sample(2, p.height-3, 0x4b)?)?;
    let twin=double_entrance_candidate(cells,g,p);
    let roof=if twin {
        // Native gray edge stone supplies the missing cap's slate pigment.
        let rgba=sample(0,0,0x0f)?;
        let mut colors=rgba.chunks_exact(4).map(|c|[c[0],c[1],c[2]])
            .collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        colors.sort_by_key(|c|u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722);
        if !(2..=4).contains(&colors.len()) {return None;}
        [linear(colors[colors.len()-2]),linear(colors[colors.len()-1])]
    } else {two(sample(5, 1, 0x5e)?)?};
    let mut windows=Vec::new();
    let mut lit_windows=Vec::new();
    let mut pane_cells=Vec::new();
    let mut ink=None;
    for y in (if twin {0}else{4}..p.height).step_by(2) {
        for x in 1..p.width-1 {
            let index=cells[(p.row+y)*g.width+p.column+x].source.tile_index;
            if matches!(index,0x0a|0x32) {
                let rgba=sample(x,y,index)?;
                windows.push(pixel(rgba,3,3));lit_windows.push(index==0x0a);pane_cells.push([x,y+if twin {4}else{0}]);
                ink.get_or_insert_with(||pixel(rgba,0,0));
            } else if y<p.height-2 && index!=0x4b {return None;}
        }
    }
    let door_cells=(1..p.width-1).filter(|&x|cells[(p.row+p.height-1)*g.width+p.column+x].source.tile_index==0x1b).collect::<Vec<_>>();
    let door_x=door_cells.first().copied();
    Some(Paint {
        source_height:p.height+if twin {4}else{0},
        extra_doors:door_cells.iter().skip(1).map(|&x|(x-door_x.unwrap())as f32).collect(),
        retail_sign:p.height>8 && cells[(p.row+p.height-4)*g.width+p.column+p.width-4].source.metatile_id==0x73,
        roof,brick:wall[0],mortar:wall[1],
        glass:if let Some(x)=door_x {pixel(sample(x,p.height-2,0x0b)?,6,5)} else {*windows.first()?},
        door:if let Some(x)=door_x {pixel(sample(x,p.height-1,0x1b)?,5,5)} else {wall[0]},
        windows,lit_windows,pane_cells,ink:ink?,
    })
}
// Front masonry is cut around the same native panes used by the glass. Keep
// physical reveals instead of placing opaque glass panels over an intact wall.
fn without_openings(rect: [f32; 4], openings: &[[f32; 4]]) -> Vec<[f32; 4]> {
    let mut parts=vec![rect];
    for &[a,b,c,d] in openings.iter().filter(|&&[a,b,c,d]|
        rect[0]<b && rect[1]>a && rect[2]<d && rect[3]>c) {
        let mut next=Vec::new();
        for [w,e,lo,hi] in parts {
            let (l,r,bottom,top)=(w.max(a),e.min(b),lo.max(c),hi.min(d));
            if l>=r || bottom>=top {next.push([w,e,lo,hi]);continue;}
            for part in [[w,l,lo,hi],[r,e,lo,hi],[l,r,lo,bottom],[l,r,top,hi]] {
                if part[1]-part[0]>0.00001 && part[3]-part[2]>0.00001 {next.push(part);}
            }
        }
        parts=next;
    }
    parts
}
fn pane_geometry(source: [usize;2], source_height: usize, unit: f32) -> [f32;4] {
    let [x,row]=source;
    let clerestory=row==source_height-4;
    let y=if clerestory {3.83} else if row==source_height-2 {1.40}
        else {5.35+((source_height-4-row)/2-1) as f32*1.90};
    [unit*(x as f32+0.5),unit*y,
        unit*if clerestory {0.78}else{0.80},unit*if clerestory {0.82}else{1.25}]
}
// Clip each diagonal slate to the inner coping. Boundary pieces are real
// closed solids too; clamping a rotated box would leave collapsed triangles.
fn clipped_slate(mut polygon: Vec<Vec2>, bounds: [f32;4]) -> Vec<Vec2> {
    for (axis,edge,sign) in [(0,bounds[0],1.),(0,bounds[1],-1.),
        (1,bounds[2],1.),(1,bounds[3],-1.)] {
        let mut next=Vec::new();
        for i in 0..polygon.len() {
            let (a,b)=(polygon[i],polygon[(i+1)%polygon.len()]);
            let (da,db)=((a[axis]-edge)*sign,(b[axis]-edge)*sign);
            if da>=0. {next.push(a);}
            if (da>=0.)!=(db>=0.) {next.push(a+(b-a)*(da/(da-db)));}
        }
        next.dedup_by(|a,b|a.distance_squared(*b)<0.00000001);
        if next.len()>1 && next[0].distance_squared(*next.last().unwrap())<0.00000001 {next.pop();}
        polygon=next;
    }
    polygon
}
fn append_slates(out: &mut SurfaceMeshData, bounds: [f32;4], unit: f32,
    lift: f32, roof: [f32;3], material_unit: f32) {
    let [w,e,n,s]=bounds;
    let step=unit*1.2;
    for row in -1..=((s-n)/(step*0.5)).ceil() as i32+1 {
        for col in -1..=((e-w)/step).ceil() as i32+1 {
            let center=Vec2::new(w+col as f32*step+row.rem_euclid(2) as f32*step*0.5,
                n+row as f32*step*0.5);
            let half=step*0.493;
            let polygon=clipped_slate(vec![center+Vec2::X*half,center+Vec2::Y*half,
                center-Vec2::X*half,center-Vec2::Y*half],bounds);
            let area=(0..polygon.len()).map(|i|polygon[i].perp_dot(polygon[(i+1)%polygon.len()]))
                .sum::<f32>().abs()*0.5;
            if polygon.len()<3 || area<unit*unit*0.00001 {continue;}
            let center=polygon.iter().copied().sum::<Vec2>()/polygon.len() as f32;
            let lo=unit*(5.41+lift);let hi=unit*(5.47+lift);
            let rings=[polygon.iter().map(|p|Vec3::new(p.x,lo,p.y)).collect::<Vec<_>>(),
                polygon.iter().map(|p|Vec3::new(p.x,hi-unit*0.018,p.y)).collect(),
                polygon.iter().map(|p| {let q=center+(*p-center)*0.98;Vec3::new(q.x,hi,q.y)}).collect()];
            let middle=Vec3::new(center.x,(lo+hi)*0.5,center.y);
            let mut seed=(row as u32).wrapping_mul(0x9e3779b9)
                ^(col as u32).wrapping_mul(0x85ebca6b);
            seed^=seed>>16;seed=seed.wrapping_mul(0x7feb352d);seed^=seed>>15;
            let paint=roof.map(|v|v*(0.88+(seed&65535) as f32/65535.*0.20));
            super::kanto_craft::face(out,rings[0].clone(),middle,paint,4.,material_unit);
            super::kanto_craft::face(out,rings[2].clone(),middle,paint,4.,material_unit);
            for pair in rings.windows(2) {for i in 0..polygon.len() {
                let j=(i+1)%polygon.len();
                super::kanto_craft::face(out,vec![pair[0][i],pair[0][j],pair[1][j],pair[1][i]],
                    middle,paint,4.,material_unit);
            }}
        }
    }
}
// One low weather-covered ventilator repeats across the shared native roof.
// Native roof/trim pigment owns its paint; no city or landmark variant is made.
fn append_roof_vents(out: &mut SurfaceMeshData, bounds: [f32;4], unit: f32, lift: f32,
    roof: [f32;3], stone: [f32;3], material_unit: f32) {
    let [w,e,n,s]=bounds;
    let count=if e-w>=unit*14. {3}else{2};
    let z=(n+s)*0.5;
    let dark=roof.map(|v|v*0.28);
    let metal=mix(roof,stone,0.44);
    for i in 0..count {
        let x=w+(e-w)*(i as f32+0.5)/count as f32;
        let mut part=|b:[f32;6],bevel,color,kind|block(out,
            [x+b[0]*unit,x+b[1]*unit,(b[2]+lift)*unit,(b[3]+lift)*unit,
                z+b[4]*unit,z+b[5]*unit],bevel*unit,color,kind,material_unit);
        part([-0.58,0.58,5.41,5.55,-0.78,0.78],0.025,stone,3.);
        part([-0.48,0.48,5.54,5.84,-0.62,0.62],0.025,roof,1.);
        part([-0.60,0.60,5.83,5.94,-0.74,0.74],0.035,metal,1.);
        for side in [-1.0_f32,1.] {
            let a=side*0.623;let b=side*0.635;
            part([-0.39,0.39,5.58,5.80,a.min(b),a.max(b)],0.,dark,-1.);
            for y in [5.61,5.68,5.75] {
                let a=side*0.636;let b=side*0.68;
                part([-0.40,0.40,y,y+0.025,a.min(b),a.max(b)],0.004,metal,1.);
            }
        }
    }
}

fn append_roof(out: &mut SurfaceMeshData, bounds: [f32;4], unit: f32, lift: f32,
    roof: [f32;3], stone: [f32;3], material_unit: f32) {
    let [w,e,n,roof_front]=bounds;
    let mut box_at=|b,bevel,color,kind|block(out,b,bevel,color,kind,material_unit);
    // Roof deck, physical slate flags and coping remain within the native plot.
    box_at(
        [
            w + unit * 0.08,
            e - unit * 0.08,
            unit * (5.22+lift),
            unit * (5.42+lift),
            n + unit * 0.10,
            roof_front,
        ],
        unit * 0.04,
        roof,
        4.,
    );
    drop(box_at);
    append_slates(out,[w+unit*0.30,e-unit*0.30,n+unit*0.32,roof_front-unit*0.22],
        unit,lift,roof,material_unit);
    let mut box_at=|b,bevel,color,kind|block(out,b,bevel,color,kind,material_unit);
    for (a, z) in [
        (n + unit * 0.10, n + unit * 0.30),
        (roof_front - unit * 0.20, roof_front),
    ] {
        box_at(
            [
                w + unit * 0.08,
                e - unit * 0.08,
                unit * (5.40+lift),
                unit * (5.70+lift),
                a,
                z,
            ],
            unit * 0.045,
            stone,
            3.,
        );
    }
    for (a, z) in [
        (w + unit * 0.08, w + unit * 0.28),
        (e - unit * 0.28, e - unit * 0.08),
    ] {
        box_at(
            [
                a,
                z,
                unit * (5.40+lift),
                unit * (5.70+lift),
                n + unit * 0.28,
                roof_front - unit * 0.18,
            ],
            unit * 0.045,
            stone,
            3.,
        );
    }
    append_roof_vents(out,bounds,unit,lift,roof,stone,material_unit);
}
pub(super) fn side_passage(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement)->bool {
    p.height==8 && p.column>=4 && p.column+p.width+4<=g.width
        && super::outdoor_entry_mat::kanto_approach(cells,g,p.column-4,p.row+4,true)
        && super::outdoor_entry_mat::kanto_approach(cells,g,p.column+p.width,p.row+4,false)
}
pub(super) fn side_passage_ground(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement)->Option<usize> {
    // The complete east approach contains two plain street columns beyond
    // its mat. Continue that proven local ground through the gate corridor.
    side_passage(cells,g,p).then_some((p.row+4)*g.width+p.column+p.width+2)
}
pub(super) fn append(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, door: Option<f32>, p: Paint) {
    append_with_passage(out,b,unit,door,p,false);
}

fn append_retail_sign(out:&mut SurfaceMeshData,b:[f32;4],unit:f32,p:&Paint,material_unit:f32) {
    let cx=b[1]-unit*3.;let front=b[3];
    let frame=mix(p.roof[1],p.mortar,0.28);
    for x in [cx-unit*0.63,cx+unit*0.63] {
        block(out,[x-unit*0.07,x+unit*0.07,0.,unit*1.05,
            front-unit*0.44,front-unit*0.28],unit*0.012,frame,3.,material_unit);
    }
    block(out,[cx-unit*0.97,cx+unit*0.97,unit*0.75,unit*1.74,
        front-unit*0.48,front-unit*0.18],unit*0.035,frame,3.,material_unit);
    block(out,[cx-unit*0.84,cx+unit*0.84,unit*0.88,unit*1.60,
        front-unit*0.19,front-unit*0.12],unit*0.015,p.mortar,3.,material_unit);
    // Original closed 3x5 strokes retain the compact MART inscription without
    // stretching its native pixels or introducing another store model.
    for (letter,rows) in [[5u8,7,7,5,5],[2,5,7,5,5],[6,5,6,5,5],[7,2,2,2,2]]
        .into_iter().enumerate() {
        for (row,bits) in rows.into_iter().enumerate() {for col in 0..3 {
            if bits&(1<<(2-col))==0 {continue;}
            let x=cx+unit*(-0.77+letter as f32*0.40+col as f32*0.105);
            let y=unit*(1.43-row as f32*0.12);
            block(out,[x,x+unit*0.108,y,y+unit*0.102,front-unit*0.12,front-unit*0.08],
                0.,p.ink,-1.,material_unit);
        }}
    }
}

pub(super) fn append_with_passage(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32,
    door: Option<f32>, p: Paint, passage:bool) {
    let passage=passage && door.is_none() && p.source_height==8;
    // Source paint must stay inside its material tag domain at every viewport
    // origin and story count. Author locally; geometry still uses native units.
    let first=out.positions.len();
    let origin=Vec3::new(b[0],0.,b[2]);
    let b=[0.,b[1]-b[0],0.,b[3]-b[2]];
    let door=door.map(|x|x-origin.x);
    let mut door_positions=door.into_iter().collect::<Vec<_>>();
    if let Some(first)=door {door_positions.extend(p.extra_doors.iter().map(|&offset|first+offset*unit));}
    let [w, e, n, s] = b;
    let extra_bands = (p.source_height-8)/2;
    let lift = extra_bands as f32 * 1.90;
    let wall_top = 5.20+lift;
    // Only the primitive's final unit controls procedural UVs, not geometry.
    // Tall/deep source drawings need the same bounded domain as low offices.
    let material_unit=unit*((e-w)/(unit*15.)).max((s-n)/(unit*7.5))
        .max((5.70+lift)/7.5).max(1.);
    let front = s - unit * if passage {0.24}else{0.72};
    let back = n + unit * 0.48;
    // Extra native window courses encode height, not a full-height warehouse
    // extending through every row. A low rear wing retains the occupied plot;
    // the upper volume uses the same eight-cell depth as the ordinary office.
    let tower_back=back+(s-n-unit*8.).max(0.);
    // A deep lip hides recessed clerestory glass from the raised game camera.
    // Keep a modest coping overhang above the masonry and clear that sightline.
    let roof_front=front+unit*0.18;
    let stone = mix(p.roof[1], p.mortar, 0.28);
    let trim = mix(p.mortar, p.ink, 0.18);
    let panes=p.pane_cells.iter().map(|&cell|pane_geometry(cell,p.source_height,unit)).collect::<Vec<_>>();
    let openings=panes.iter().map(|&[x,y,width,height]|
        [x-width*0.5-unit*0.055,x+width*0.5+unit*0.055,
            y-unit*0.055,y+height+unit*0.055]).collect::<Vec<_>>();
    let mut box_at = |b: [f32; 6], bevel: f32, c: [f32; 3], kind: f32| {
        let first=out.indices.len();
        if passage {
            for [n,s,lo,hi] in without_openings([b[4],b[5],b[2],b[3]],
                &[[unit*4.10,unit*7.50,0.,unit*3.40]]) {
                block(out,[b[0],b[1],lo,hi,n,s],bevel,c,kind,material_unit);
            }
        } else {block(out,b,bevel,c,kind,material_unit);}
        if kind==2. {
            // Only these bricks have a solid masonry backing. Omit the
            // buried rear faces to pay for the new visible window reveals.
            // The shared closed-block primitive remains closed for other kits.
            let (axis,sign)=if ((b[4]+b[5])*0.5-front).abs()<unit*0.001 {(2,1.)}
                else if ((b[4]+b[5])*0.5-back).abs()<unit*0.001
                    || ((b[4]+b[5])*0.5-tower_back).abs()<unit*0.001 {(2,-1.)}
                else if ((b[0]+b[1])*0.5-(w+unit*0.45)).abs()<unit*0.001 {(0,-1.)}
                else {(0,1.)};
            let end=out.indices.len();let mut write=first;
            for read in (first..end).step_by(3) {
                if out.normals[out.indices[read] as usize][axis]*sign < -0.999 {continue;}
                out.indices.copy_within(read..read+3,write);write+=3;
            }
            out.indices.truncate(write);
        }
    };
    box_at(
        [
            w + unit * 0.38,
            e - unit * 0.38,
            0.,
            unit * 0.42,
            back,
            front + unit * 0.12,
        ],
        unit * 0.06,
        stone,
        3.,
    );
    box_at(
        [
            w + unit * 0.45,
            e - unit * 0.45,
            unit * 0.40,
            unit * 5.20,
            back,
            front-unit*0.28,
        ],
        unit * 0.02,
        p.mortar,
        3.,
    );
    if lift>0. {
        box_at([w+unit*0.45,e-unit*0.45,unit*5.20,unit*wall_top,
            tower_back,front-unit*0.28],unit*0.02,p.mortar,3.);
    }
    for [l,r,lo,hi] in without_openings(
        [w+unit*0.45,e-unit*0.45,unit*0.40,unit*wall_top],&openings) {
        box_at([l,r,lo,hi,front-unit*0.28,front],0.,p.mortar,3.);
    }
    // Running-bond brickwork on every exposed wall. The high window band
    // stays above the main brick field, as in the source clerestory facade.
    for side in 0..4 {
        for row in 0..if extra_bands==0 {7} else {((wall_top-0.53)/0.43).floor() as usize} {
            let lo = unit * (0.53 + row as f32 * 0.43);
            let hi = lo + unit * 0.395;
            let rear=if lo>=unit*5.20 {tower_back}else{back};
            let (a, z, span) = if side < 2 {
                (
                    w + unit * 0.48,
                    if side == 0 { front } else { rear },
                    e - w - unit * 0.96,
                )
            } else {
                (
                    rear,
                    if side == 2 {
                        w + unit * 0.45
                    } else {
                        e - unit * 0.45
                    },
                    front - rear,
                )
            };
            let count = (span / (unit * 0.73)).ceil() as usize;
            let step = span / count as f32;
            for col in 0..=count {
                let start = (col as f32 - if row % 2 == 0 { 0. } else { 0.5 }) * step;
                let l = a + start.max(0.) + unit * 0.012;
                let r = a + (start + step).min(span) - unit * 0.012;
                if r - l < unit * 0.05 {
                    continue;
                }
                // Kiln variation belongs to each brick, rather than repeating
                // in diagonal bands across the whole facade. Local course
                // coordinates keep its native hue and paint stable on scroll.
                let mut kiln=(row as u32).wrapping_mul(0x9e3779b9)
                    ^(col as u32).wrapping_mul(0x85ebca6b)
                    ^(side as u32).wrapping_mul(0xc2b2ae35);
                kiln^=kiln>>16;
                kiln=kiln.wrapping_mul(0x7feb352d);
                kiln^=kiln>>15;
                let color=p.brick.map(|v|v*(0.92+(kiln&31) as f32*(0.14/31.)));
                if side==0 {
                    for [l,r,lo,hi] in without_openings([l,r,lo,hi],&openings) {
                        box_at([l,r,lo,hi,z-unit*0.035,z+unit*0.035],unit*0.018,color,2.);
                    }
                } else {
                    let bounds = if side == 1 {
                        [l, r, lo, hi, z - unit * 0.035, z + unit * 0.035]
                    } else {
                        [z - unit * 0.035, z + unit * 0.035, lo, hi, l, r]
                    };
                    box_at(bounds, unit * 0.018, color, 2.);
                }
            }
        }
    }
    for x in [w + unit * 0.45, e - unit * 0.45] {
        let corners=if tower_back>back {vec![(back,5.24),(front,5.24+lift),(tower_back,5.24+lift)]}
            else {vec![(back,5.24+lift),(front,5.24+lift)]};
        for (z,top) in corners {
            box_at(
                [
                    x - unit * 0.09,
                    x + unit * 0.09,
                    unit * 0.40,
                    unit * top,
                    z - unit * 0.09,
                    z + unit * 0.09,
                ],
                unit * 0.025,
                trim,
                1.,
            );
        }
    }
    for y in [unit * 0.50, unit * 3.63, unit * (4.83+lift), unit * (5.12+lift)] {
        box_at(
            [
                w + unit * 0.36,
                e - unit * 0.36,
                y,
                y + unit * 0.12,
                front - unit * 0.04,
                front + unit * 0.15,
            ],
            unit * 0.02,
            trim,
            1.,
        );
    }
    // Narrow stone floor belts return around all four walls of stacked drawings.
    for band in 0..extra_bands {
        let y=unit*(5.03+band as f32*1.90);
        let rear=if band==0 {back}else{tower_back};
        for bounds in [
            [w+unit*0.36,e-unit*0.36,y,y+unit*0.12,front-unit*0.04,front+unit*0.15],
            [w+unit*0.36,e-unit*0.36,y,y+unit*0.12,rear-unit*0.15,rear+unit*0.04],
            [w+unit*0.30,w+unit*0.49,y,y+unit*0.12,rear,front],
            [e-unit*0.49,e-unit*0.30,y,y+unit*0.12,rear,front],
        ] {box_at(bounds,unit*0.02,trim,1.);}
    }
    let roof = mix(p.roof[0], p.roof[1], 0.55);
    drop(box_at);
    if passage {
        // Two native side approaches share one open corridor. Masonry and
        // foundation are cut together; the lintel and jambs frame real depth.
        for [a,b] in [[w+unit*0.25,w+unit*0.68],[e-unit*0.68,e-unit*0.25]] {
            for bounds in [
                [a,b,0.,unit*3.4,unit*3.94,unit*4.10],
                [a,b,0.,unit*3.4,unit*7.50,unit*7.66],
                [a,b,unit*3.4,unit*3.66,unit*3.94,unit*7.66],
            ] {block(out,bounds,unit*0.025,stone,3.,material_unit);}
        }
    }
    append_roof(out,[w,e,tower_back-unit*0.48,roof_front],unit,lift,roof,stone,material_unit);
    if tower_back>back {
        append_roof(out,[w,e,n,tower_back+unit*0.10],unit,0.,roof,stone,material_unit);
    }
    // One recessed framed pane for each native light, including repeated stories.
    let mut window = |x: f32, y: f32, width: f32, height: f32, glass: [f32; 3], lit: bool| {
        block(
            out,
            [
                x - width * 0.5 - unit * 0.055,
                x + width * 0.5 + unit * 0.055,
                y - unit * 0.055,
                y + height + unit * 0.055,
                front - unit * 0.21,
                front - unit * 0.13,
            ],
            unit * 0.02,
            p.ink,
            -1.,
            material_unit,
        );
        let light_first=out.positions.len();
        block(
            out,
            [
                x - width * 0.5,
                x + width * 0.5,
                y,
                y + height,
                front - unit * 0.10,
                front - unit * 0.08,
            ],
            unit * 0.008,
            glass.map(|v| v * 0.84),
            -1.,
            material_unit,
        );
        crate::scenery_finish::mark_source_glass(out,light_first..out.positions.len(),
            [x-width*0.5,x+width*0.5,y,y+height],lit);
        for xx in [x - width * 0.5, x + width * 0.5] {
            block(
                out,
                [
                    xx - unit * 0.025,
                    xx + unit * 0.025,
                    y,
                    y + height,
                    front - unit * 0.13,
                    front + unit * 0.07,
                ],
                unit * 0.009,
                trim,
                1.,
                material_unit,
            );
        }
        for yy in [y,y+height] {
            block(out,[x-width*0.5-unit*0.025,x+width*0.5+unit*0.025,
                yy-unit*0.025,yy+unit*0.025,front-unit*0.13,front+unit*0.07],
                0.,trim,1.,material_unit);
        }
        block(
            out,
            [
                x - width * 0.5 - unit * 0.07,
                x + width * 0.5 + unit * 0.07,
                y - unit * 0.08,
                y - unit * 0.015,
                front - unit * 0.12,
                front + unit * 0.22,
            ],
            unit * 0.012,
            stone,
            3.,
            material_unit,
        );
        block(
            out,
            [
                x - width * 0.35,
                x + width * 0.15,
                y + height * 0.77,
                y + height * 0.80,
                front - unit * 0.079,
                front - unit * 0.07,
            ],
            0.,
            glass,
            -1.,
            material_unit,
        );
    };
    for ((&[x,y,width,height],glass),&lit) in panes.iter().zip(&p.windows).zip(&p.lit_windows) {
        window(w+x,y,width,height,*glass,lit);
    }
    drop(window);
    // The 7d facade is an uninterrupted masonry wall below its ten lights.
    // It has no doorway, lower panes or entrance step in the source.
    for door in door_positions.iter().copied() {
    block(
        out,
        [
            door - unit * 0.98,
            door + unit * 0.98,
            unit * 0.38,
            unit * 3.31,
            front + unit * 0.10,
            front + unit * 0.30,
        ],
        unit * 0.045,
        trim,
        1.,
        material_unit,
    );
    block(
        out,
        [
            door - unit * 0.80,
            door + unit * 0.80,
            unit * 0.40,
            unit * 3.13,
            front + unit * 0.30,
            front + unit * 0.34,
        ],
        unit * 0.02,
        p.door,
        -1.,
        material_unit,
    );
    block(
        out,
        [
            door - unit * 0.61,
            door + unit * 0.61,
            unit * 1.85,
            unit * 2.91,
            front + unit * 0.34,
            front + unit * 0.36,
        ],
        unit * 0.015,
        p.ink,
        -1.,
        material_unit,
    );
    let light_first=out.positions.len();
    block(
        out,
        [
            door - unit * 0.51,
            door + unit * 0.51,
            unit * 1.95,
            unit * 2.81,
            front + unit * 0.36,
            front + unit * 0.38,
        ],
        unit * 0.012,
        p.glass,
        -1.,
        material_unit,
    );
    mark_source_light(out,light_first,true);
    block(
        out,
        [
            door + unit * 0.55,
            door + unit * 0.62,
            unit * 1.12,
            unit * 1.62,
            front + unit * 0.35,
            front + unit * 0.46,
        ],
        unit * 0.015,
        stone,
        -1.,
        material_unit,
    );
    }
    if let (Some(&first),Some(&last))=(door_positions.first(),door_positions.last()) {
    block(
        out,
        [
            first - unit * 1.05,
            last + unit * 1.05,
            0.,
            unit * 0.28,
            front + unit * 0.18,
            s,
        ],
        unit * 0.035,
        stone,
        3.,
        material_unit,
    );
    }
    if p.retail_sign {append_retail_sign(out,b,unit,&p,material_unit);}
    for position in &mut out.positions[first..] {
        *position=(Vec3::from_array(*position)+origin).to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kanto_side_gate_corridor_clears_both_native_lanes_and_keeps_the_shared_windows() {
        let mut paint=colors();paint.pane_cells=(1..11).map(|x|[x,4]).collect();
        let mut closed=SurfaceMeshData::default();append(&mut closed,[0.,96.,0.,64.],8.,None,paint.clone());
        let mut open=SurfaceMeshData::default();append_with_passage(&mut open,[0.,96.,0.,64.],8.,None,paint.clone(),true);
        let blocked=|mesh:&SurfaceMeshData,y:f32,z:f32|mesh.indices.chunks_exact(3).any(|tri| {
            let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            let q=p.map(|p|Vec2::new(p.y,p.z));
            let (a,b,d)=(q[1]-q[0],q[2]-q[0],Vec2::new(y,z)-q[0]);
            let area=a.perp_dot(b);if area.abs()<0.000001 {return false;}
            let (u,v)=(d.perp_dot(b)/area,a.perp_dot(d)/area);
            u>=0. && v>=0. && u+v<=1.
        });
        for z in [40.,56.] {for y in [4.,12.,20.,24.] {
            assert!(blocked(&closed,y,z),"the ordinary office stays enclosed");
            assert!(!blocked(&open,y,z),"the gate corridor must clear both native approach lanes");
        }}
        assert_eq!(crate::city_lights::collect(&open).len(),10);
        assert!(open.positions.iter().all(|p|p[0]>=0. && p[0]<=96. && p[2]>=0. && p[2]<=64.),
            "gate trim, coping and sills stay inside the native plot");
        for tri in open.indices.chunks_exact(3) {
            let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(open.positions[i as usize]));
            let normal=(p[1]-p[0]).cross(p[2]-p[0]);
            assert!(normal.length_squared()>0.00000001);
            assert!(normal.dot(Vec3::from_array(open.normals[tri[0]as usize]))>0.);
        }
        let mut standard=SurfaceMeshData::default();append(&mut standard,[0.,96.,0.,64.],8.,Some(40.),paint.clone());
        let mut guarded=SurfaceMeshData::default();append_with_passage(&mut guarded,[0.,96.,0.,64.],8.,Some(40.),paint,true);
        assert_eq!(standard,guarded,"front-door buildings retain their original producer");
    }
    #[test]
    fn kanto_side_gate_requires_both_complete_adjacent_approaches() {
        let g=GridGeometry {width:16,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let sources=(0..128).map(|i| {
            let (x,y)=(i%16,i/16);let west=x<4;
            let mut s=crate::mesh::tests::source_with_tile(1,(x%4)as u8,(y%4)as u8,0x23);
            s.tileset_id="kanto".into();
            if y>=4 && (x<4 || x>=12) {
                let j=(y%4)*4+x%4;s.metatile_id=if west {0x17}else{0x16};
                s.tile_index=if (x%4>=2)==west {0x4f}
                    else if west && j==4 || !west && j==14 {0x39}else{0x23};
            } s
        }).collect();
        let mut tiles=crate::mesh::tests::frame(16,8,sources).tiles;
        let p=BuildingPlacement {column:4,row:0,width:8,height:8,roof_rows:4,ground_tile_index:0x2c};
        assert!(side_passage(&tiles.iter().collect::<Vec<_>>(),&g,p));
        assert_eq!(side_passage_ground(&tiles.iter().collect::<Vec<_>>(),&g,p),Some(78));
        assert_eq!(tiles[78].source.tile_index,0x23);
        for index in [64,76] {
            tiles[index].source.tile_index^=1;
            assert!(!side_passage(&tiles.iter().collect::<Vec<_>>(),&g,p));
            assert_eq!(side_passage_ground(&tiles.iter().collect::<Vec<_>>(),&g,p),None);
            tiles[index].source.tile_index^=1;
        }
    }
    fn colors() -> Paint {
        Paint {
            source_height:8,
            extra_doors:Vec::new(),
            retail_sign:false,
            roof: [[0.12; 3], [0.45; 3]],
            brick: [0.4, 0.22, 0.04],
            mortar: [0.8, 0.9, 0.8],
            glass: [0.7, 0.9, 0.7],
            windows: vec![[0.7, 0.9, 0.7]; 10],
            lit_windows: vec![true; 10],
            pane_cells: vec![[1,4],[2,4],[3,4],[4,4],[7,4],[8,4],[9,4],[10,4],[6,6],[7,6]],
            door: [0.4, 0.2, 0.03],
            ink: [0.04; 3],
        }
    }
    #[test]
    fn shared_roof_vents_stay_seated_and_keep_grilles_recessed_in_every_size() {
        for width in [8.,12.,16.,20.] {for lift in [0.,15.2] {
            let unit=8.;let bounds=[0.,width*unit,0.,8.*unit];
            let roof=[0.15,0.18,0.20];let stone=[0.55,0.60,0.52];
            let mut mesh=SurfaceMeshData::default();
            append_roof_vents(&mut mesh,bounds,unit,lift,roof,stone,unit*4.);
            assert!(mesh.indices.len()/3<2000,"bounded shared roof detail");
            assert!(mesh.positions.iter().all(|p|p[0]>bounds[0]+unit*0.3
                && p[0]<bounds[1]-unit*0.3 && p[2]>bounds[2]+unit*0.3
                && p[2]<bounds[3]-unit*0.3 && p[1]>=unit*(5.41+lift)-0.0001
                && p[1]<=unit*(5.94+lift)+0.0001));
            let count=if width>=14. {3}else{2};
            for i in 0..count {
                let x=width*unit*(i as f32+0.5)/count as f32;
                for side in [-1.0_f32,1.] {
                    let well=unit*(4.+side*0.635);
                    let slat=unit*(4.+side*0.68);
                    assert!(side*(slat-well)>unit*0.04,"physical reveal in front of the dark grille");
                    for (z,lo,hi) in [(well,5.58,5.80),(slat,5.60,5.79)] {
                        assert!(mesh.positions.iter().any(|p|(p[2]-z).abs()<0.0001
                            && p[1]>=unit*(lo+lift)-0.0001 && p[1]<=unit*(hi+lift)+0.0001
                            && (p[0]-x).abs()<unit*0.5));
                    }
                }
            }
            let mut repainted=SurfaceMeshData::default();
            append_roof_vents(&mut repainted,bounds,unit,lift,[0.02,0.04,0.09],[0.12,0.16,0.23],unit*4.);
            assert_eq!(mesh.positions,repainted.positions);assert_eq!(mesh.indices,repainted.indices);
            assert_eq!(mesh.normals,repainted.normals);assert_eq!(mesh.uvs,repainted.uvs);
            assert_ne!(mesh.colors,repainted.colors,"the same vent follows native roof/trim pigment");
        }}
    }

    #[test]
    fn assembled_roof_has_diagonal_closed_slates_clipped_inside_its_coping() {
        for width in [8.,12.,20.] {
            let unit=8.;let bounds=[0.,width*unit,0.,7.5*unit];
            let mut mesh=SurfaceMeshData::default();
            append_roof(&mut mesh,bounds,unit,0.,[0.3;3],[0.5;3],unit*2.);
            let mut top_area=0.;let mut diagonals=0;
            for tri in mesh.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let normal=(p[1]-p[0]).cross(p[2]-p[0]);
                assert!(normal.length_squared()>0.00000001,"clipped pieces cannot collapse");
                assert!(normal.dot(Vec3::from_array(mesh.normals[tri[0] as usize]))>0.,"outward winding");
                if p.iter().all(|p|(p.y-unit*5.47).abs()<0.0001) {
                    assert!(tri.iter().all(|&i|crate::scenery_finish::town_paint_kind(mesh.uvs[i as usize])==Some(4)),
                        "the physical slate field must not inherit ceramic glaze or glass");
                    top_area+=normal.length()*0.5;
                    for i in 0..3 {let d=p[(i+1)%3]-p[i];
                        if d.x.abs()>0.001 && (d.x.abs()-d.z.abs()).abs()<0.0001 {diagonals+=1;}}
                    assert!(p.iter().all(|p|p.x>=unit*0.30 && p.x<=bounds[1]-unit*0.30
                        && p.z>=unit*0.32 && p.z<=bounds[3]-unit*0.22));
                }
            }
            let area=(bounds[1]-unit*0.60)*(bounds[3]-unit*0.54);
            assert!(top_area>area*0.82 && top_area<area*0.97,"continuous slate field with narrow joints: {top_area}/{area}");
            assert!(diagonals>30,"native diagonal joints must be visible geometry");
        }
    }

    #[test]
    fn tall_brick_offices_keep_a_closed_low_wing_under_a_compact_upper_volume() {
        let hit=|m:&SurfaceMeshData,axis:usize,query:Vec2| {
            let mut result=None::<f32>;
            for tri in m.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(m.positions[i as usize]));
                let q=p.map(|p|Vec2::new(p[(axis+1)%3],p[(axis+2)%3]));
                let (a,b,d)=(q[1]-q[0],q[2]-q[0],query-q[0]);
                let area=a.perp_dot(b);if area.abs()<0.000001 {continue;}
                let (u,v)=(d.perp_dot(b)/area,a.perp_dot(d)/area);
                if u<0. || v<0. || u+v>1. {continue;}
                let at=p[0][axis]*(1.-u-v)+p[1][axis]*u+p[2][axis]*v;
                result=Some(result.map_or(at,|old|old.max(at)));
            }
            result.expect("the original occupied plot must remain a closed building")
        };
        for height in [8,12,24,32] {
            let mut p=colors();p.source_height=height;
            let mut mesh=SurfaceMeshData::default();let unit=8.;
            append(&mut mesh,[0.,96.,0.,height as f32*unit],unit,Some(40.),p);
            let rear=hit(&mesh,1,Vec2::new(unit*2.,unit*6.));
            assert!(rear>unit*5.3 && rear<unit*6.,"solid rear roof, not a freed invisible wall plot");
            let top=hit(&mesh,1,Vec2::new(unit*(height as f32-4.),unit*6.));
            let lift=(height-8) as f32*0.95;
            assert!(top>unit*(5.3+lift) && top<unit*(5.75+lift));
            let side=hit(&mesh,0,Vec2::new(unit*2.,unit*2.));
            assert!(side>unit*11.4 && side<unit*12.,"rear wing keeps its closed side wall");
            if height>8 {
                assert!(mesh.positions.iter().all(|p|p[1]<=unit*6. || p[2]>=unit*(height as f32-8.)),
                    "upper stories and their roof must fit the compact front volume");
            }
        }
    }
    #[test]
    fn native_brick_windows_are_visible_inside_real_masonry_openings() {
        // Trace the assembled facade, including its wall and running-bond
        // bricks: glass behind an uncut wall must fail this visibility check.
        let front_hit=|m:&SurfaceMeshData,query:Vec2,slope:f32| {
            let mut hit=None::<(f32,usize)>;
            for tri in m.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(m.positions[i as usize]));
                let q=p.map(|p|Vec2::new(p.x,p.y-slope*p.z));
                let (a,b,d)=(q[1]-q[0],q[2]-q[0],query-q[0]);
                let area=a.perp_dot(b);if area.abs()<0.000001 {continue;}
                let (u,v)=(d.perp_dot(b)/area,a.perp_dot(d)/area);
                if u<0. || v<0. || u+v>1. {continue;}
                let z=p[0].z*(1.-u-v)+p[1].z*u+p[2].z*v;
                if hit.is_none_or(|(old,_)|z>old) {hit=Some((z,tri[0] as usize));}
            }
            hit.expect("the recessed window must close the facade")
        };
        for height in [8,24] {
            let mut p=colors();p.source_height=height;p.lit_windows[2]=false;
            let mut mesh=SurfaceMeshData::default();let unit=8.;
            let front=height as f32*unit-unit*0.72;
            append(&mut mesh,[0.,96.,0.,height as f32*unit],unit,Some(40.),p.clone());
            // Foreground buildings are viewed more steeply than the orbit
            // camera's central ray. Their own roof must not hide the glass.
            for slope in [0.,2.] {for (i,&[x,row]) in p.pane_cells.iter().enumerate() {
                let clerestory=row==height-4;
                let bottom=if clerestory {3.83} else if row==height-2 {1.40}
                    else {5.35+((height-4-row)/2-1) as f32*1.90};
                let query=Vec2::new((x as f32+0.5)*unit,
                    (bottom+if clerestory {0.82}else{1.25}*0.45)*unit
                        -slope*(front-unit*0.082));
                let (z,index)=front_hit(&mesh,query,slope);
                assert!(z>front-unit*0.09 && z<front-unit*0.075,
                    "native pane {x},{row} is exposed behind the wall, including its small bevel");
                assert_eq!(crate::scenery_finish::source_glass_coordinates(mesh.uvs[index]).map(|(_,lit)|lit),Some(p.lit_windows[i]));
                let half=if clerestory {0.39}else{0.40};
                let (rim,_)=front_hit(&mesh,query+Vec2::X*unit*(half+0.012),slope);
                assert!(rim-z>unit*0.14,"the four-sided frame must stand in front of the glass");
                let (wall,_)=front_hit(&mesh,query+Vec2::X*unit*(half+0.10),slope);
                assert!(wall>=front-0.0001,"masonry beside each aperture must stay closed");
            }}
        }
    }
    #[test]
    fn every_brick_material_stays_native_at_all_viewport_origins_and_story_counts() {
        for (width,height,door) in [(8,8,None),(12,8,Some(5.)),(16,24,Some(5.)),(20,32,None)] {
            let mut p=colors();p.source_height=height;
            let bounds=[0.,width as f32*8.,0.,height as f32*8.];
            let mut local=SurfaceMeshData::default();
            append(&mut local,bounds,8.,door.map(|x|x*8.),p.clone());
            assert!(local.uvs.iter().all(|&uv|uv==crate::scenery_finish::SOURCE_PAINT_UV
                || crate::city_lights::source_window_tag(uv)
                || crate::scenery_finish::source_glass_coordinates(uv).is_some()
                || crate::scenery_finish::town_paint_kind(uv).is_some()),
                "no roof/wall/trim vertex can receive a second night tint");
            for origin in [Vec3::new(512.,0.,-768.),Vec3::new(-992.,0.,1440.)] {
                let mut moved=SurfaceMeshData::default();
                append(&mut moved,[origin.x,bounds[1]+origin.x,origin.z,bounds[3]+origin.z],8.,
                    door.map(|x|x*8.+origin.x),p.clone());
                assert_eq!(local.uvs,moved.uvs);assert_eq!(local.colors,moved.colors);
                assert_eq!(local.normals,moved.normals);assert_eq!(local.indices,moved.indices);
                for (a,b) in local.positions.iter().zip(&moved.positions) {
                    assert_eq!(*b,(Vec3::from_array(*a)+origin).to_array());
                }
            }
        }
    }

    #[test]
    fn doorless_brick_variant_has_ten_upper_lights_without_a_false_entrance() {
        let mut paint=colors();paint.pane_cells=(1..11).map(|x|[x,4]).collect();
        let mut mesh=SurfaceMeshData::default();
        append(&mut mesh,[0.,96.,0.,64.],8.,None,paint.clone());
        let panes=crate::city_lights::collect(&mesh);
        assert_eq!(panes.len(),10);
        assert!(mesh.positions.iter().zip(&mesh.uvs).filter(|(_,uv)|crate::scenery_finish::source_glass_coordinates(**uv).is_some())
            .all(|(p,_)|p[1]>3.8*8.));
        assert!(mesh.positions.iter().all(|p|p[1]>=3.4*8. || p[2]<(64.-8.*0.72)+8.*0.31));
        let mut changed=SurfaceMeshData::default();paint.windows[4]=[0.02,0.04,0.09];
        append(&mut changed,[0.,96.,0.,64.],8.,None,paint.clone());
        assert_eq!(mesh.positions,changed.positions);assert_eq!(mesh.indices,changed.indices);
        assert_ne!(mesh.colors,changed.colors);
        assert_eq!(mesh.uvs,changed.uvs,"daylight paint cannot remove a retained lamp");
        for i in [0,4,7,9] {paint.windows[i]=[0.12,0.08,0.24];paint.lit_windows[i]=false;}
        let mut night=SurfaceMeshData::default();append(&mut night,[0.,96.,0.,64.],8.,None,paint.clone());
        let lit=crate::city_lights::collect(&night);
        assert_eq!(lit.len(),6,"only the six source-lit panes emit");
    }

    #[test]
    fn closed_office_parts_have_outward_winding_and_no_zero_area_faces() {
        for (bounds, kind, bevel) in [
            ([0., 2., 0., 3., 0., 1.], 1., 0.03),
            ([0., 2., 0., 0.1, 0., 0.01], -1., 0.),
            ([0., 2., 0., 1., 0., 0.07], 2., 0.018),
            ([0., 0.07, 0., 1., 0., 2.], 2., 0.018),
        ] {
            let mut m = SurfaceMeshData::default();
            block(&mut m, bounds, bevel, [0.5; 3], kind, 1.);
            let mut edges = std::collections::BTreeMap::new();
            let mut volume = 0.;
            for t in m.indices.chunks_exact(3) {
                let p = [t[0], t[1], t[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                assert!((p[1] - p[0]).cross(p[2] - p[0]).length() > 0.000001);
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
                for i in 0..3 {
                    let mut e = [
                        p[i].to_array().map(|v| (v * 100000.).round() as i32),
                        p[(i + 1) % 3]
                            .to_array()
                            .map(|v| (v * 100000.).round() as i32),
                    ];
                    e.sort();
                    *edges.entry(e).or_insert(0) += 1;
                }
            }
            assert!(volume > 0.);
            assert!(edges.values().all(|&n| n == 2));
        }
    }
    #[test]
    fn corner_posts_do_not_bury_the_masonry_on_any_of_the_four_walls() {
        let mut m = SurfaceMeshData::default();
        append(&mut m, [0., 96., 0., 64.], 8., Some(40.), colors());
        for (axis, positive, along) in [
            (2, true, 2.8 * 8.),
            (2, false, 2.8 * 8.),
            (0, true, 3.404 * 8.),
            (0, false, 3.404 * 8.),
        ] {
            let query = bevy::prelude::Vec2::new(along, 2.03 * 8.);
            let mut hit = None::<(f32, usize)>;
            for tri in m.indices.chunks_exact(3) {
                let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                let project =
                    |p: Vec3| bevy::prelude::Vec2::new(if axis == 2 { p.x } else { p.z }, p.y);
                let q = p.map(project);
                let a = q[1] - q[0];
                let b = q[2] - q[0];
                let d = query - q[0];
                let area = a.perp_dot(b);
                if area.abs() < 0.000001 {
                    continue;
                }
                let u = d.perp_dot(b) / area;
                let v = a.perp_dot(d) / area;
                if u < 0. || v < 0. || u + v > 1. {
                    continue;
                }
                let depth = p[0][axis] * (1. - u - v) + p[1][axis] * u + p[2][axis] * v;
                if hit.is_none_or(|(old, _)| if positive { depth > old } else { depth < old }) {
                    hit = Some((depth, tri[0] as usize));
                }
            }
            let (_, index) = hit.expect("closed facade");
            assert_eq!(
                crate::scenery_finish::town_paint_kind(m.uvs[index]),
                Some(2),
                "wall axis{axis}, positive{positive}"
            );
        }
    }

    #[test]
    fn shared_office_stays_in_its_native_plot_and_has_bounded_geometry() {
        let mut m = SurfaceMeshData::default();
        append(&mut m, [24., 120., 40., 104.], 8., Some(64.), colors());
        assert!(
            m.indices.len() / 3 < 12000,
            "{} triangles",
            m.indices.len() / 3
        );
        assert!(m.indices.len() / 3 > 5000);
        assert!(m.positions.iter().all(|p| p.iter().all(|v| v.is_finite())
            && (24. ..=120.).contains(&p[0])
            && (40. ..=104.).contains(&p[2])
            && p[1] >= 0.));
        assert_eq!(m.positions.len(), m.colors.len());
        assert_eq!(m.positions.len(), m.uvs.len());
        assert!(
            m.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        let mut changed = SurfaceMeshData::default();
        let mut night = colors();
        night.windows[0] = [0.1, 0.03, 0.08];
        append(&mut changed, [24., 120., 40., 104.], 8., Some(64.), night);
        assert_eq!(m.positions, changed.positions);
        assert_eq!(m.indices, changed.indices);
        assert_ne!(m.colors, changed.colors);
    }
}
