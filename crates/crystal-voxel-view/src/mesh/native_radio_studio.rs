//! Complete studio panes, worktables and apparatus; striped fields stay floor.
use super::*;
#[path="native_radio_side.rs"]
mod side;
pub(super) fn side_missing_context(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,furnishings:&[Placement])->bool {
    side::missing_context(map,p,cells,g,furnishings)
}


fn tile(cells:&[&VisualTile],g:&GridGeometry,x:usize,y:usize,b:u16,c:u8,r:u8,t:u16)->bool {
    x<g.width&&y<g.height&&{
        let s=&cells[y*g.width+x].source;
        s.tileset_id.as_ref()=="radio_tower"&&s.metatile_id==b&&s.subtile_column==c
            &&s.subtile_row==r&&s.tile_index==t
    }
}

pub(super) fn panel(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !native_radio_room::is_map(map)||p.kind!=ModelKind::WindowWall||p.width!=2||p.base_pixels!=0.
        ||p.column+2>g.width||p.row+p.height>g.height||cells.len()!=g.width*g.height{return false;}
    let first=&cells[p.row*g.width+p.column].source;let ox=first.subtile_column;
    if first.metatile_id!=0x15||!matches!(ox,0|2)||first.subtile_row!=2{return false;}
    for y in 0..2 {for x in 0..2 {
        if !tile(cells,g,p.column+x,p.row+y,0x15,ox+x as u8,2+y as u8,
            [[0x30,0x31],[0x20,0x21]][y][x]){return false;}
    }}
    if p.row+4>g.height{return false;}
    let lower=&cells[(p.row+2)*g.width+p.column].source;
    let b=lower.metatile_id;
    match p.height {
        4=>{
            let art=match (b,ox) {
                (0x19,2)=>[[0x20,0x21],[0x25,0x26]],
                (0x33,2)|(0x36,2)=>[[0x20,0x21],[0x29,0x2a]],
                (0x36,0)=>[[0x41,0x21],[0x40,0x2a]],_=>return false,
            };
            (0..2).all(|y|(0..2).all(|x|tile(cells,g,p.column+x,p.row+2+y,b,
                ox+x as u8,y as u8,art[y][x])))
                &&(b!=0x36||ox!=0||(p.row+5<=g.height
                    &&tile(cells,g,p.column,p.row+4,0x36,0,2,0x40)))
        }
        3=>b==0x33&&ox==0&&(0..2).all(|y|(0..2).all(|x|
            tile(cells,g,p.column+x,p.row+2+y,b,x as u8,y as u8,
                [[0x20,0x21],[7,6]][y][x]))),
        2=>{
            if b!=0x19||ox!=0||p.row+6>g.height{return false;}
            (0..4).all(|y|(0..2).all(|x|tile(cells,g,p.column+x,p.row+2+y,b,
                x as u8,y as u8,[[0x22,0x23],[0x32,0x33],[0x32,0x33],[0x42,0x43]][y][x])))
        }
        _=>false,
    }
}

fn grille_panel(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    panel(map,p,cells,g)&&p.height==4&&cells[(p.row+2)*g.width+p.column].source.metatile_id==0x36
        &&cells[(p.row+2)*g.width+p.column].source.subtile_column==0
}
pub(super) fn grille_foot(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if p.kind!=ModelKind::WallPanel||p.width!=1||p.height!=1||p.base_pixels!=0.
        ||p.depth_pixels!=0.||p.height_pixels!=0.||p.row<4||cells.len()!=g.width*g.height
        ||!tile(cells,g,p.column,p.row,0x36,0,2,0x40){return false;}
    let q=Placement{kind:ModelKind::WindowWall,column:p.column,row:p.row-4,width:2,height:4,
        ground:p.ground,depth_pixels:6.,height_pixels:32.,base_pixels:0.,footing_pixels:None};
    grille_panel(map,&q,cells,g)
}
pub(super) fn grille_foot_ready(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,furnishings:&[Placement])->bool {
    grille_foot(map,p,cells,g)&&crate::interior_models::selected_model(ModelKind::WindowWall).is_some()
        &&furnishings.iter().any(|q|q.column==p.column&&q.row+4==p.row&&grille_panel(map,q,cells,g))
}
fn grille_ready(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,furnishings:&[Placement])->bool {
    grille_panel(map,p,cells,g)&&crate::interior_models::selected_model(ModelKind::WallPanel).is_some()
        &&furnishings.iter().any(|q|q.column==p.column&&q.row==p.row+4&&grille_foot(map,q,cells,g))
}

fn east_return(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,furnishings:&[Placement])->bool {
    if !panel(map,p,cells,g) || p.row+10>g.height {return false;}
    furnishings.iter().any(|anchor| {
        if !panel(map,anchor,cells,g) || anchor.row!=p.row || anchor.height!=4 || anchor.column<4
            || cells[(anchor.row+2)*g.width+anchor.column].source.metatile_id!=0x33
            || !table_ready(map,anchor.column-4,anchor.row+4,cells,g,furnishings)
            || !(0..2).all(|x|tile(cells,g,anchor.column+x,anchor.row+9,0x34,2+x as u8,3,0x42+x as u16)) {
            return false;
        }
        let mut east=anchor.column;
        while furnishings.iter().any(|q|q.column==east+2 && q.row==p.row && panel(map,q,cells,g)) {
            east+=2;
        }
        if east!=p.column || side::replaces_return(map) {return false;}
        // A cropped or user-owned continuation is still glass source. Do
        // not cap the run early just because its next model is unavailable.
        if east+2<g.width {
            let next=&cells[p.row*g.width+east+2].source;
            if next.tileset_id.as_ref()=="radio_tower" && next.metatile_id==0x15
                && next.subtile_row==2 && next.tile_index==0x30 {return false;}
        }
        true
    })
}

/// Already rendered glass owns these east-boundary spans. The generic room
/// enclosure must not place an opaque wall outside the same glass faces.
pub(super) fn perimeter_glass(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    furnishings:&[Placement],authored:&[Option<&'static str>],origin:[i32;2],map_width:i32)->Vec<(f32,f32)> {
    if authored.len()!=cells.len() {return vec![];}
    furnishings.iter().filter_map(|p| {
        if !p.indices(g.width).all(|i|authored[i]==Some(p.label())) {return None;}
        if side::matches(map,p,cells,g) && origin[0]+p.column as i32+1==map_width {
            Some((g.origin_z+p.row as f32*g.tile_height,
                g.origin_z+(p.row+p.height) as f32*g.tile_height))
        } else if east_return(map,p,cells,g,furnishings)
            && origin[0]+(p.column+p.width) as i32==map_width {
            Some((g.origin_z+(p.row+4) as f32*g.tile_height-3.*g.tile_height/8.,
                g.origin_z+(p.row+10) as f32*g.tile_height))
        } else {None}
    }).collect()
}

fn table_at(map:&str,cells:&[&VisualTile],g:&GridGeometry,column:usize,row:usize)->bool {
    if !native_radio_room::is_map(map)||row==0||column+4>g.width||row+5>g.height
        ||cells.len()!=g.width*g.height{return false;}
    let art=[[0x25,0x26,7,6],[0x35,0x36,0x19,9],[8,0x19,0x19,9],
        [0x4e,0x4f,0x19,9],[0x5e,0x5f,0x19,9],[0x15,0x18,0x18,0x16]];
    // Top two left cells are glass; only the cap on their right belongs to
    // this table. All six rows guard the complete neighboring composition.
    (0..6).all(|y|(0..4).all(|x| {
        let b=match (y<3,x<2) {(true,true)=>0x19,(true,false)=>0x33,
            (false,true)=>0x32,(false,false)=>0x34};
        let r=if y<3 {y+1}else{y-3};let c=if x<2 {x+2}else{x-2};
        tile(cells,g,column+x,row-1+y,b,c as u8,r as u8,art[y][x])
    }))
}

pub(super) fn table(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::DiningTable&&p.width==4&&p.height==5&&p.base_pixels==0.
        &&table_at(map,cells,g,p.column,p.row)
}
pub(super) fn cap(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::LabCounter&&p.width==2&&p.height==1&&p.column>=2&&p.base_pixels==0.&&p.height_pixels==0.&&p.depth_pixels==0.
        &&table_at(map,cells,g,p.column-2,p.row+1)
}

fn table_ready(map:&str,column:usize,row:usize,cells:&[&VisualTile],g:&GridGeometry,furnishings:&[Placement])->bool {
    crate::interior_models::selected_model(ModelKind::DiningTable).is_some_and(|m|m.has_native_dining_table_materials())
        &&furnishings.iter().any(|q|q.column==column&&q.row==row&&table(map,q,cells,g))
}
pub(super) fn cap_ready(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,furnishings:&[Placement])->bool {
    cap(map,p,cells,g)&&table_ready(map,p.column-2,p.row+1,cells,g,furnishings)
}

pub(super) fn gold_local(s:&VisualTileSource)->Option<(u8,u8)> {
    if s.tileset_id.as_ref()!="radio_tower"{return None;}
    let (ox,oy)=match s.metatile_id {0x32=>(0,0),0x33=>(2,2),0x34=>(2,0),_=>return None};
    if !(ox..ox+2).contains(&s.subtile_column)||!(oy..oy+2).contains(&s.subtile_row){return None;}
    let (x,y)=(s.subtile_column-ox,s.subtile_row-oy);
    (s.tile_index==[[0x27,0x28],[0x37,0x38]][y as usize][x as usize]).then_some((x,y))
}
fn gold(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    native_radio_room::is_map(map)&&p.kind==ModelKind::ArcadeStool&&p.width==2&&p.height==2&&p.base_pixels==0.
        &&p.column+2<=g.width&&p.row+2<=g.height&&cells.len()==g.width*g.height
        &&p.indices(g.width).enumerate().all(|(j,i)|gold_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8)))
}

pub(super) fn rack(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    native_radio_room::is_map(map)&&p.kind==ModelKind::BroadcastRack&&p.width==2&&p.height==4&&p.base_pixels==0.
        &&p.column+2<=g.width&&p.row+4<=g.height&&cells.len()==g.width*g.height
        &&p.indices(g.width).enumerate().all(|(j,i)|tile(cells,g,i%g.width,i/g.width,0x19,
            (j%2)as u8,(j/2)as u8,[0x22,0x23,0x32,0x33,0x32,0x33,0x42,0x43][j]))
}
pub(super) fn computer(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !native_radio_room::is_map(map)||p.kind!=ModelKind::Computer||p.width!=2||p.height!=3||p.base_pixels!=0.
        ||p.column+2>g.width||p.row+3>g.height||cells.len()!=g.width*g.height{return false;}
    let (b,ox,art)=match cells[p.row*g.width+p.column].source.metatile_id {
        0x1c=>(0x1c,2,[0x56,0x57,0x58,0x59,0x4c,0x4d]),
        0x12=>(0x12,0,[0x2e,0x2f,0x3e,0x3f,0x3a,0x3b]),_=>return false,
    };
    p.indices(g.width).enumerate().all(|(j,i)|tile(cells,g,i%g.width,i/g.width,b,
        ox+(j%2)as u8,1+(j/2)as u8,art[j]))
}
pub(super) fn console(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !native_radio_room::is_map(map)||p.kind!=ModelKind::BroadcastConsole||p.base_pixels!=0.||p.column+p.width>g.width
        ||p.row+p.height>g.height||cells.len()!=g.width*g.height{return false;}
    let (b,ox,oy,art):(u16,u8,u8,&[u16])=match (p.width,p.height,cells[p.row*g.width+p.column].source.metatile_id) {
        (4,3,0x16)=>(0x16,0,1,&[0x11,0x54,0x56,0x57,5,0x55,0x58,0x59,0x15,0x17,0x17,0x16]),
        (2,2,0x2b)=>(0x2b,2,2,&[0x56,0x57,0x58,0x59]),_=>return false,
    };
    p.indices(g.width).enumerate().all(|(j,i)|tile(cells,g,i%g.width,i/g.width,b,
        ox+(j%p.width)as u8,oy+(j/p.width)as u8,art[j]))
}
pub(super) fn equipment_size(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<(f32,f32)> {
    if computer(map,p,cells,g){Some((8.,16.))}
    else if rack(map,p,cells,g){Some((8.,24.))}
    else if console(map,p,cells,g){Some((6.,if p.height==2 {10.}else{18.}))}else{None}
}

pub(super) fn candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
    if !native_radio_room::is_map(map)||cells.len()!=g.width*g.height||claimed.len()!=cells.len(){return;}
    let Some(ground)=ground_sample(cells,map,"radio_tower",1)else{return;};
    let protected=claimed.to_vec();
    for (kind,w,h,d,rise) in [(ModelKind::WindowWall,2,4,6.,32.),(ModelKind::WindowWall,2,3,6.,32.),
        (ModelKind::WindowWall,2,2,6.,32.),(ModelKind::DiningTable,4,5,40.,8.),
        (ModelKind::LabCounter,2,1,0.,0.),(ModelKind::WallPanel,1,1,0.,0.),(ModelKind::ArcadeStool,2,2,8.,7.)] {
        if w>g.width||h>g.height{continue;}
        for row in 0..=g.height-h {for column in 0..=g.width-w {
            let p=Placement{kind,column,row,width:w,height:h,ground,depth_pixels:d,height_pixels:rise,
                base_pixels:0.,footing_pixels:None};
            let pane=panel(map,&p,cells,g);let table=table(map,&p,cells,g);let cap=cap(map,&p,cells,g);let foot=grille_foot(map,&p,cells,g);
            if !(pane||table||cap||foot||gold(map,&p,cells,g))||p.indices(g.width).any(|i|claimed[i]){continue;}
            let (gx,gy,gw,gh)=if pane {(column,row,2,if h==2 {6}else if grille_panel(map,&p,cells,g){5}else{4})}
                else if foot {(column,row-4,2,5)}else if table {(column,row-1,4,6)}else if cap {(column-2,row,4,6)}else{(column,row,w,h)};
            if (gy..gy+gh).any(|y|(gx..gx+gw).any(|x|protected[y*g.width+x])){continue;}
            for i in p.indices(g.width){claimed[i]=true;}out.push(p);
        }}
    }
    side::candidates(map,cells,g,claimed,out,ground);
}

pub(super) fn handled(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    panel(map,p,cells,g)||table(map,p,cells,g)||cap(map,p,cells,g)||grille_foot(map,p,cells,g)||gold(map,p,cells,g)||side::matches(map,p,cells,g)
}

pub(super) fn floor_sample(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,destination:usize)->Option<usize> {
    if side::matches(map,p,cells,g) && cells[destination].source.metatile_id!=0x3e {return None;}
    if !(side::matches(map,p,cells,g)||rack(map,p,cells,g)||table(map,p,cells,g)||cap(map,p,cells,g)||grille_foot(map,p,cells,g)||gold(map,p,cells,g)){return None;}
    let phase=cells[destination].source.subtile_column%2;let wanted=0x42+u16::from(phase);
    cells.iter().position(|t|t.source.tileset_id.as_ref()=="radio_tower"&&t.source.metatile_id==0x35
        &&t.source.subtile_column<4&&t.source.subtile_row<4&&t.source.subtile_column%2==phase&&t.source.tile_index==wanted)
}

pub(super) fn append(out:&mut SurfaceMeshData,map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,
    b:[f32;4],selected:&crate::interior_models::Model,furnishings:&[Placement]) {
    if cap(map,p,cells,g)||grille_foot(map,p,cells,g){return;}
    let scale=g.tile_height/8.;let [w,e,n,s]=b;
    if gold(map,p,cells,g) {
        let paint=native_source_swatches(p,g,[[8,5],[3,9]]);
        selected.append_native_stool(out,b,7.*scale,paint[0],paint[1]);return;
    }
    if side::matches(map,p,cells,g) {
        side::append(out,map,p,cells,g,furnishings);return;
    }
    if panel(map,p,cells,g) {
        let paint=native_source_swatches(p,g,[[0,0],[3,11],[3,6]]);
        let joined_grille=grille_ready(map,p,cells,g,furnishings);
        let first=out.positions.len();architecture_box(out,[w,e,0.,30.*scale,s-3.*scale,s],[1.;3]);
        out.uvs[first..].fill(paint[1]);
        // Header30/31 is the top cap; only its dark fascia faces forward.
        let first=out.positions.len();architecture_box(out,[w,e,30.*scale,32.*scale,n,s],[1.;3]);
        out.uvs[first..].fill(paint[2]);
        for x in 0..2 {append_top(out,[w+x as f32*(e-w)/2.,w+(x+1)as f32*(e-w)/2.,n,s],
            32.01*scale,g.uv(p.column+x,p.row));}
        for y in 1..4 {for x in 0..2 {
            let mic_row=y==3&&cells[(p.row+2)*g.width+p.column].source.metatile_id==0x19
                &&table_ready(map,p.column,p.row+4,cells,g,furnishings);
            let post_row=joined_grille&&x==0&&y>=2;
            let source_row=if y>=p.height||mic_row||post_row {p.row+1}else{p.row+y};
            let uv=g.uv(p.column+x,source_row);let a=w+(e-w)*x as f32/2.;let z=a+(e-w)/2.;
            let top=(30.-(y-1)as f32*10.)*scale;let bottom=top-10.*scale;
            append_quad(out,[[z,bottom,s+0.01*scale],[z,top,s+0.01*scale],[a,top,s+0.01*scale],[a,bottom,s+0.01*scale]],
                Vec3::Z.to_array(),[[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],TEXTURED_SHADE);
        }}
        let first=out.positions.len();
        architecture_box(out,[w,e,0.,scale,s-3.*scale,s+0.02*scale],[1.;3]);out.uvs[first..].fill(paint[0]);
        // The eastern end of the studio is a glass return, joining the
        // front pane to the room edge. Follow the complete connected pane
        // run beyond the terminal drawing; larger studios have more glass.
        if east_return(map,p,cells,g,furnishings) {
            let end = g.origin_z + (p.row+10) as f32 * g.tile_height;
            let start = s - 3.*scale;
            let first = out.positions.len();
            architecture_box(out,[e-3.*scale,e,0.,30.*scale,start,end],[1.;3]);
            out.uvs[first..].fill(paint[1]);
            let first = out.positions.len();
            architecture_box(out,[e-3.*scale,e,30.*scale,32.*scale,start,end],[1.;3]);
            out.uvs[first..].fill(paint[2]);
            // Both exposed faces retain the live glass drawing. East and
            // west views must not depend on a one-sided south-facing quad.
            for segment in 0..6 {
              let near = end - (segment+1) as f32*g.tile_height;
              let far = near + g.tile_height;
              for (a,b) in [(e+0.01*scale,1.),(e-3.01*scale,-1.)] {
                for band in 0..3 {
                    let uv = g.uv(p.column+segment%2,p.row+1);
                    let top = (30.-band as f32*10.)*scale;
                    let mut points = [[a,top-10.*scale,far],[a,top,far],
                        [a,top,near],[a,top-10.*scale,near]];
                    let mut uvs = [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]];
                    if b > 0. {points.reverse();uvs.reverse();}
                    append_quad(out,points,[b,0.,0.],uvs,TEXTURED_SHADE);
                }
              }
            }
        }
        if joined_grille {append_grille(out,p,g,[w,s]);}
        return;
    }
    if table(map,p,cells,g) {
        let uv=g.uv(p.column+2,p.row-1);
        let paint=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
        assert!(table_body(out,p,g,b,selected));
        microphone(out,[w,n],scale,native_source_swatches(p,g,[[0,0],[3,0]]));
        // The uncertain printed tabletop insert retains its exact live art.
        // It sits on a shallow closed support, not on the walking floor.
        let first=out.positions.len();
        architecture_box(out,[w,w+(e-w)*0.5,8.*scale,8.18*scale,s-16.*scale,s],[1.;3]);
        out.uvs[first..].fill(paint(3.,3.));
        for y in 0..2 {for x in 0..2 {
            append_top(out,[w+x as f32*(e-w)/4.,w+(x+1)as f32*(e-w)/4.,
                s-(2-y)as f32*8.*scale,s-(1-y)as f32*8.*scale],8.19*scale,
                g.uv(p.column+x,p.row+2+y));
        }}
    }
}

fn table_body(out:&mut SurfaceMeshData,p:&Placement,g:&GridGeometry,b:[f32;4],selected:&crate::interior_models::Model)->bool {
    let uv=g.uv(p.column+2,p.row-1);
    let point=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
    let gray=point(3.,3.);let white=point(1.,1.);let first=out.positions.len();
    if !selected.append_native_dining_table(out,b,8.*g.tile_height/8.,[white,gray]) {return false;}
    // Selected tables may put cloth or timber on top. The source studio has
    // one gray work surface; every upward surface follows that source field.
    for i in first..out.positions.len() {if out.normals[i][1]>0.5 {out.uvs[i]=gray;}}
    true
}

fn append_grille(out:&mut SurfaceMeshData,p:&Placement,g:&GridGeometry,origin:[f32;2]) {
    let scale=g.tile_height/8.;let [w,n]=origin;let e=w+8.*scale;let s=n+8.*scale;
    let uv=g.uv(p.column,p.row+2);
    let gray=[uv.0+(uv.1-uv.0)*3.5/8.,uv.2+(uv.3-uv.2)*3.5/8.];
    let first=out.positions.len();architecture_box(out,[w,e,0.,16.*scale,n,s],[1.;3]);out.uvs[first..].fill(gray);
    append_top(out,[w,e,n,s],16.01*scale,uv);
    for y in 0..2 {
        let uv=g.uv(p.column,p.row+3+y);let top=(16.-8.*y as f32)*scale;
        append_quad(out,[[e,top-8.*scale,s+0.01*scale],[e,top,s+0.01*scale],
            [w,top,s+0.01*scale],[w,top-8.*scale,s+0.01*scale]],Vec3::Z.to_array(),
            [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],TEXTURED_SHADE);
    }
}

fn round_part(out:&mut SurfaceMeshData,origin:[f32;3],axis:[f32;3],profile:&[[f32;2]],scale:f32,paint:[f32;2]) {
    let axis=Vec3::from_array(axis).normalize();let seed=if axis.x.abs()>0.9 {Vec3::Z}else{Vec3::X};
    let u=(seed-axis*axis.dot(seed)).normalize();let v=axis.cross(u);let origin=Vec3::from_array(origin);
    let point=|ring:[f32;2],j:usize| {
        let theta=j as f32*std::f32::consts::TAU/12.;
        (origin+scale*(axis*ring[0]+ring[1]*(u*theta.cos()+v*theta.sin()))).to_array()
    };
    for rings in profile.windows(2) {for j in 0..12 {
        let q=[point(rings[0],j),point(rings[0],(j+1)%12),point(rings[1],(j+1)%12),point(rings[1],j)];
        if rings[0][1]==0.||rings[1][1]==0. {
            let tri=if rings[0][1]==0. {[q[0],q[2],q[3]]}else{[q[0],q[1],q[2]]};
            let normal=(Vec3::from_array(tri[1])-Vec3::from_array(tri[0]))
                .cross(Vec3::from_array(tri[2])-Vec3::from_array(tri[0])).normalize();
            let first=out.positions.len()as u32;
            for p in tri {out.positions.push(p);out.normals.push(normal.to_array());out.uvs.push(paint);
                out.colors.push(crate::interior_models::authored_face_color([1.;4],normal));}
            out.indices.extend([first,first+1,first+2]);
        }else {
            let normal=(Vec3::from_array(q[1])-Vec3::from_array(q[0]))
                .cross(Vec3::from_array(q[2])-Vec3::from_array(q[0])).normalize();
            append_quad(out,q,normal.to_array(),[paint;4],crate::interior_models::authored_face_color([1.;4],normal));
        }
    }}
}
fn microphone(out:&mut SurfaceMeshData,origin:[f32;2],scale:f32,paint:[[f32;2];2]) {
    let [w,n]=origin;
    round_part(out,[w+8.*scale,8.*scale,n+12.*scale],[0.,1.,0.],
        &[[0.,0.],[0.,2.8],[0.2,3.3],[0.6,3.3],[0.8,2.8],[0.8,0.]],scale,paint[0]);
    round_part(out,[w+8.*scale,8.8*scale,n+12.*scale],[0.,1.,0.],
        &[[0.,0.],[0.,0.65],[5.2,0.65],[5.2,0.]],scale,paint[0]);
    round_part(out,[w+8.*scale,13.*scale,n+12.*scale],[0.,1.,-2.],
        &[[0.,0.],[0.2,0.9],[0.7,1.5],[1.2,1.7],[3.8,1.7],[4.3,1.5],[4.8,0.9],[5.,0.]],scale,paint[1]);
}

pub(super) fn append_computer(out:&mut SurfaceMeshData,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,b:[f32;4],selected:&crate::interior_models::Model) {
    let scale=g.tile_height/8.;let [w,e,n,s]=b;
    let rear=cells[p.row*g.width+p.column].source.metatile_id==0x12;
    let paint=native_source_swatches(p,g,if rear {[[3,0],[0,0],[3,1],[2,0],[0,0]]}
        else{[[2,0],[0,0],[2,1],[1,0],[0,0]]});
    let uv=g.uv(p.column,p.row);
    // The actual LCD is below the white case cap, and spans two atlas rows.
    let screen=[uv.0+(uv.1-uv.0)*if rear {4./8.}else{3./8.},uv.0+(uv.1-uv.0)*if rear {12./8.}else{13./8.},
        uv.2+(uv.3-uv.2)*5./8.,uv.2+(uv.3-uv.2)*10./8.];
    let mut upper=SurfaceMeshData::default();
    assert!(selected.append_native_radio_terminal_head(&mut upper,b,16.*scale,8.*scale,paint,screen));
    let first=out.positions.len();architecture_box(out,[w,e,0.,8.*scale,n,s],[1.;3]);out.uvs[first..].fill(paint[1]);
    for x in 0..2 {
        let uv=g.uv(p.column+x,p.row+2);let a=w+(e-w)*x as f32/2.;let z=a+(e-w)/2.;
        // Rear PC3a/3b contains upward keyboard art above a three-pixel plinth.
        // Its keys are modeled on top; only that bottom plinth faces forward.
        let top=if rear {3.}else{8.};let v=if rear {uv.2+(uv.3-uv.2)*5./8.}else{uv.2};
        append_quad(out,[[z,0.,s+0.01*scale],[z,top*scale,s+0.01*scale],
            [a,top*scale,s+0.01*scale],[a,0.,s+0.01*scale]],Vec3::Z.to_array(),
            [[uv.1,uv.3],[uv.1,v],[uv.0,v],[uv.0,uv.3]],TEXTURED_SHADE);
    }
    let offset=out.positions.len()as u32;
    out.positions.extend(upper.positions);out.normals.extend(upper.normals);out.uvs.extend(upper.uvs);out.colors.extend(upper.colors);
    out.indices.extend(upper.indices.into_iter().map(|i|i+offset));
}

pub(super) fn append_equipment(out:&mut SurfaceMeshData,p:&Placement,g:&GridGeometry,b:[f32;4],selected:&crate::interior_models::Model) {
    let paint=if p.kind==ModelKind::BroadcastRack {
        native_source_swatches(p,g,[[1,1],[3,8],[3,8],[0,0]])
    }else {
        let x=if p.width==4 {16}else{0};
        native_source_swatches(p,g,[[x,0],[x+1,0],[x+2,1],[x+2,0]])
    };
    let screen=if p.kind==ModelKind::BroadcastConsole {
        let x=p.column+if p.width==4 {2}else{0};let uv=g.uv(x,p.row);
        Some([uv.0+(uv.1-uv.0)*2./8.,uv.0+(uv.1-uv.0)*14./8.,
            uv.2+(uv.3-uv.2)*2./8.,uv.2+(uv.3-uv.2)*5./8.])
    }else{None};
    assert!(selected.append_native_radio_equipment(out,b,p.height_pixels*g.tile_height/8.,paint,screen));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_microphone_is_round_closed_angled_and_keeps_source_ink() {
        for scale in [0.5,1.,2.] {
            let paint=[[0.1,0.2],[0.3,0.4]];let mut mesh=SurfaceMeshData::default();
            microphone(&mut mesh,[0.,0.],scale,paint);
            let mut edges=std::collections::HashMap::new();let mut volume=0.;
            let key=|p:[f32;3]|p.map(|v|(v/scale*10000.).round()as i32);
            for t in mesh.indices.chunks_exact(3) {
                let [a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let cross=(b-a).cross(c-a);assert!(cross.length_squared()>0.0000001*scale.powi(4));
                assert!(cross.dot(Vec3::from_array(mesh.normals[t[0]as usize]))>0.);
                volume+=a.dot(b.cross(c))/6.;
                for (i,j) in [(0,1),(1,2),(2,0)] {
                    let mut pair=[key(mesh.positions[t[i]as usize]),key(mesh.positions[t[j]as usize])];pair.sort();
                    *edges.entry(pair).or_insert(0usize)+=1;
                }
            }
            assert!(volume>0.);assert!(edges.values().all(|&n|n==2),"every microphone part is closed");
            assert!(mesh.normals.iter().all(|n|(Vec3::from_array(*n).length_squared()-1.).abs()<0.001));
            assert!(mesh.uvs.iter().all(|uv|paint.contains(uv)));
            assert!(mesh.positions.iter().all(|p|p[1]>=8.*scale&&p[1]<=17.5*scale
                &&p[2]>=4.*scale&&p[2]<=16.*scale&&p[0]>=4.5*scale&&p[0]<=11.5*scale));
            let head:Vec<_>=mesh.positions.iter().zip(&mesh.uvs).filter(|(_,uv)|**uv==paint[1]).map(|(p,_)|p).collect();
            assert!(head.iter().any(|p|p[2]<8.*scale));
            let xs:std::collections::HashSet<_>=head.iter().map(|p|(p[0]/scale*10000.).round()as i32).collect();
            assert!(xs.len()>6,"head has a round section rather than box corners");
        }
    }
    #[test]
    fn external_radio_computers_cover_three_atomic_drawings_and_preserve_reception_ownership() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(path.is_file());let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
        let art=&pack.runtime_files()["data/tilesets/radio_tower_metatiles.bin"];
        for name in ["RadioTower1F","RadioTower2F","RadioTower3F","RadioTower4F","RadioTower5F","LavRadioTower1F"] {
            let data=&pack.data().maps[name];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"radio_tower".into(),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve(name,&refs,&g,None);let pcs=placements.iter().filter(|p|computer(name,p,&refs,&g)).collect::<Vec<_>>();
            assert_eq!(pcs.len(),usize::from(matches!(name,"RadioTower1F"|"RadioTower5F"|"LavRadioTower1F")),"{name}");
            let mut reception=vec![false;w*h];
            for q in crate::mesh::facility_radio::resolve(name,&refs,&g,[0,0],None,&reception.clone()) {
                for i in q.indices(w){reception[i]=true;}
            }
            for p in pcs {
                assert!(p.indices(w).all(|i|!reception[i]),"standalone PC cannot replace reception hardware");
                let mut mesh=TerrainMeshData{authored_cells:vec![None;w*h],footing_heights:vec![1.5;w*h],..Default::default()};
                let mut claimed=vec![false;w*h];assert!(super::super::append_with_tables(&mut mesh,name,&refs,&g,p,&mut claimed,[0,0],&placements));
                assert_eq!(claimed.iter().filter(|v|**v).count(),6);assert_eq!(mesh.footing_heights,vec![1.5;w*h]);
                for bad in p.indices(w) {
                    let mut changed=tiles.clone();changed[bad].source.tile_index^=128;
                    let changed=changed.iter().collect::<Vec<_>>();
                    assert!(!resolve(name,&changed,&g,None).iter().any(|q|q.column==p.column&&q.row==p.row&&q.kind==ModelKind::Computer));
                }
                let mut body=SurfaceMeshData::default();append_computer(&mut body,p,&refs,&g,[0.,16.,0.,8.],crate::interior_models::selected_model(p.kind).unwrap());
                if refs[p.row*w+p.column].source.metatile_id==0x12 {
                    let footer=g.uv(p.column,p.row+2);
                    assert!(!body.uvs.iter().any(|uv|uv[1]>=footer.2&&uv[1]<footer.2+(footer.3-footer.2)*5./8.-0.00001),
                        "rear keyboard artwork must not be painted onto the vertical plinth");
                }
                for warp in &data.events.warps {for dy in 0..2 {for dx in 0..2 {
                    assert!(!claimed[(warp.y as usize*2+dy)*w+warp.x as usize*2+dx]);
                }}}
            }
            if name=="RadioTower1F" {
                let pc:Vec<_>=refs.iter().enumerate().filter(|(_,t)|t.source.metatile_id==0x2b&&t.source.subtile_column>=2&&t.source.subtile_row>=2).collect();
                assert_eq!(pc.len(),4);assert!(pc.iter().all(|(i,_)|reception[*i]),"U fixture retains its own four monitor/keyboard cells");
            }
        }
    }
    #[test]
    fn external_radio_studios_keep_full_panels_tables_floor_and_claims() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(path.is_file(),"studio regression requires external pack");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
        let art=&pack.runtime_files()["data/tilesets/radio_tower_metatiles.bin"];
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/radio_tower.png"]).unwrap().to_luma8();
        for (name,panes) in [("RadioTower2F",4),("RadioTower4F",4),("RadioTower5F",8),("LavRadioTower1F",8)] {
            let data=&pack.data().maps[name];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"radio_tower".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve(name,&refs,&g,None);
            assert_eq!(placements.iter().filter(|p|east_return(name,p,&refs,&g,&placements)).count(),usize::from(name!="RadioTower5F"),
                "source side wall replaces the inferred east return in5F");
            if let Some(return_p)=placements.iter().find(|p|east_return(name,p,&refs,&g,&placements)) {
                assert_eq!(return_p.column,placements.iter().filter(|p|panel(name,p,&refs,&g))
                    .map(|p|p.column).max().unwrap(),"return closes the whole studio, beyond the desk and grille");
            } else {
                assert!(placements.iter().any(|p|side::matches(name,p,&refs,&g)&&p.column==w-1),
                    "5F east edge must use its complete authored side wall");
            }
            let padded_g=GridGeometry {width:w+8,height:h+8,origin_x:-32.,origin_z:-32.,..g};
            let padded=(0..padded_g.width*padded_g.height).map(|i| {
                let (x,y)=(i%padded_g.width,i/padded_g.width);
                let mut tile=if x>=4 && x<w+4 && y>=4 && y<h+4 {tiles[(y-4)*w+x-4].clone()}
                    else {let mut t=tiles[0].clone();t.source.metatile_id=0;t.source.tile_index=0;t};
                tile.column=x as u32;tile.row=y as u32;tile
            }).collect::<Vec<_>>();
            let padded_refs=padded.iter().collect::<Vec<_>>();
            let padded_placements=resolve(name,&padded_refs,&padded_g,None);
            assert_eq!(padded_placements.iter().filter(|p|east_return(name,p,&padded_refs,&padded_g,&padded_placements)).count(),usize::from(name!="RadioTower5F"),
                "production halo must not move or suppress the room edge");
            assert_eq!(placements.iter().filter(|p|panel(name,p,&refs,&g)).count(),panes,"{name} panes");
            assert_eq!(placements.iter().filter(|p|table(name,p,&refs,&g)).count(),1,"{name} table");
            assert_eq!(placements.iter().filter(|p|cap(name,p,&refs,&g)).count(),1,"{name} table cap");
            assert_eq!(placements.iter().filter(|p|gold(name,p,&refs,&g)).count(),3,"{name} gold stools");
            assert_eq!(placements.iter().filter(|p|rack(name,p,&refs,&g)).count(),1,"{name} rack");
            assert_eq!(placements.iter().filter(|p|computer(name,p,&refs,&g)).count(),usize::from(matches!(name,"RadioTower5F"|"LavRadioTower1F")),"complete CRT terminal");
            assert!(!placements.iter().any(|p|p.kind==ModelKind::BroadcastConsole&&refs[p.row*w+p.column].source.metatile_id==0x1c),"terminal is not a mixing desk");
            assert_eq!(placements.iter().filter(|p|grille_foot(name,p,&refs,&g)).count(),if panes==8 {2}else{0},"{name} grille posts");
            let sides=placements.iter().filter(|p|side::matches(name,p,&refs,&g)).collect::<Vec<_>>();
            let side_cells=match name {"RadioTower2F"|"RadioTower5F"=>10,"LavRadioTower1F"=>4,_=>0};
            assert_eq!(sides.iter().map(|p|p.height).sum::<usize>(),side_cells,"complete pale cap run");
            assert_eq!(padded_placements.iter().filter(|p|side::matches(name,p,&padded_refs,&padded_g))
                .map(|p|p.height).sum::<usize>(),side_cells,"side glass survives a shifted production halo");
            for p in sides {
                let mut body=SurfaceMeshData::default();
                side::append(&mut body,name,p,&refs,&g,&placements);
                assert!(body.positions.iter().any(|v|v[1]>32.));
                let cap=native_source_swatches(p,&g,[[3,0]])[0];
                assert!(body.positions.iter().zip(&body.normals).zip(&body.uvs)
                    .any(|((v,n),uv)|v[1]>32. && n[1]>0.99 && *uv==cap),"pale source cap stays on top");
                for ids in body.indices.chunks_exact(3) {
                    let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(body.positions[i as usize]));
                    let n=Vec3::from_array(body.normals[ids[0] as usize]);
                    assert!((b-a).cross(c-a).dot(n)>0.,"closed glass faces use outward winding");
                }
                for sign in [-1.,1.] {
                    assert!(body.positions.iter().zip(&body.normals).any(|(v,n)|
                        n[0]==sign && v[1]==30.),"both sides have full-height glass");
                }
                for bad in p.indices(w) {
                    if name=="RadioTower5F" {
                        let mut changed=tiles.clone();changed[bad].source.tile_index^=0x80;
                        let changed_refs=changed.iter().collect::<Vec<_>>();
                        assert!(!placements.iter().any(|q|east_return(name,q,&changed_refs,&g,&placements)),
                            "a changed side cap must not revive a wall across the studio doorway");
                    }
                    let mut reserved=vec![false;w*h];reserved[bad]=true;
                    let mut context=placements.iter().filter(|q|panel(name,q,&refs,&g)).cloned().collect::<Vec<_>>();
                    side::candidates(name,&refs,&g,&mut reserved,&mut context,p.ground);
                    assert!(!context.iter().any(|q|side::matches(name,q,&refs,&g)),"one custom cell protects the whole side run");
                }
                let mut fallback=TerrainMeshData {authored_cells:vec![None;w*h],footing_heights:vec![3.;w*h],..Default::default()};
                assert!(super::super::append_with_tables(&mut fallback,name,&refs,&g,p,&mut vec![false;w*h],[0,0],&[]));
                assert!(fallback.textured.positions.iter().all(|v|v[1]==0.),"missing glass context retains source cap drawing");
                assert_eq!(fallback.footing_heights,vec![3.;w*h]);
            }
            let mut owned=vec![false;w*h];let mut mesh=TerrainMeshData{authored_cells:vec![None;w*h],footing_heights:vec![2.5;w*h],..Default::default()};
            let mut claimed=vec![false;w*h];
            for p in placements.iter().filter(|p|handled(name,p,&refs,&g)||equipment_size(name,p,&refs,&g).is_some()) {
                let mut raised=p.clone();raised.base_pixels=1.;
                assert!(!handled(name,&raised,&refs,&g)&&equipment_size(name,&raised,&refs,&g).is_none(),"custom base has authority");
                let pixel=|pixel:[usize;2]| {
                    let source=&refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source;
                    png.get_pixel(source.tile_index as u32%16*8+(pixel[0]%8)as u32,
                        source.tile_index as u32/16*8+(pixel[1]%8)as u32).0[0]
                };
                if computer(name,p,&refs,&g) {
                    assert_eq!((p.depth_pixels,p.height_pixels),(8.,16.));
                    let rear=refs[p.row*w+p.column].source.metatile_id==0x12;
                    let probes=if rear {[([3,0],0),([0,0],170),([3,1],255),([2,0],85),([5,6],255),([6,6],85)]}
                        else {[([2,0],0),([0,0],170),([2,1],255),([1,0],85),([4,6],85),([7,8],85)]};
                    for (probe,gray) in probes {assert_eq!(pixel(probe),gray);}
                    let mut pc=SurfaceMeshData::default();
                    append_computer(&mut pc,p,&refs,&g,[0.,16.,0.,8.],crate::interior_models::selected_model(p.kind).unwrap());
                    assert!(pc.positions.iter().all(|v|v[0]>=0.&&v[0]<=16.&&v[1]>=0.&&v[1]<=16.001));
                    let foot=g.uv(p.column,p.row+2);
                    assert!(pc.positions.iter().zip(&pc.normals).zip(&pc.uvs)
                        .any(|((v,n),uv)|v[1]==0.&&v[2]>8.&&n[2]>0.5&&*uv==[foot.0,foot.3]),"closed native cabinet uses its own face");
                    assert!(pc.positions.iter().any(|v|*v==[16.,8.,8.]),"cabinet meets adjoining8px counter without an inset gap");
                }
                if table(name,p,&refs,&g) {
                    assert_eq!(pixel([0,0]),0);assert_eq!(pixel([3,0]),85);
                    let selected=crate::interior_models::selected_model(p.kind).unwrap();
                    let mut body=SurfaceMeshData::default();assert!(table_body(&mut body,p,&g,[0.,32.,0.,40.],selected));
                    let uv=g.uv(p.column+2,p.row-1);let gray=[uv.0+(uv.1-uv.0)*3.5/8.,uv.2+(uv.3-uv.2)*3.5/8.];
                    assert!(body.normals.iter().zip(&body.uvs).filter(|(n,_)|n[1]>0.5).all(|(_,uv)|*uv==gray),
                        "native gray worktop must not become a white cloth surface");
                    let mut original=SurfaceMeshData::default();
                    assert!(selected.append_native_dining_table(&mut original,[0.,32.,0.,40.],8.,[[0.,0.],[0.,0.]]));
                    assert_eq!(body.positions,original.positions);assert_eq!(body.normals,original.normals);assert_eq!(body.indices,original.indices);
                }
                if panel(name,p,&refs,&g) {
                    assert_eq!(pixel([3,11]),170,"glass continuation uses cyan field rather than white glare");
                    let mut pane=SurfaceMeshData::default();
                    append(&mut pane,name,p,&refs,&g,[0.,16.,0.,6.],crate::interior_models::selected_model(p.kind).unwrap(),&placements);
                    let area:f32=pane.indices.chunks_exact(3).filter(|ids|ids.iter().all(|&i| {
                        let v=pane.positions[i as usize];pane.normals[i as usize][2]>0.5&&v[2]>6.005&&v[2]<6.015
                    })).map(|ids| {
                        let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(pane.positions[i as usize]));
                        (b-a).cross(c-a).length()*0.5
                    }).sum();
                    assert!((area-16.*30.).abs()<0.01,"glass drawing covers the full front behind apparatus");
                    if east_return(name,p,&refs,&g,&placements) {
                        assert!(!east_return(name,p,&refs,&g,&[]),"missing studio context preserves the original boundary");
                        for ids in pane.indices.chunks_exact(3) {
                            let n=Vec3::from_array(pane.normals[ids[0] as usize]);
                            if n.x.abs()<0.9 {continue;}
                            let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(pane.positions[i as usize]));
                            assert!((b-a).cross(c-a).dot(n)>0.,"both glass faces use outward triangle winding");
                        }
                        for normal in [-1.,1.] {
                            assert!(pane.positions.iter().zip(&pane.normals).any(|(v,n)|
                                n[0]==normal && v[2]==(p.row+10) as f32*8. && v[1]==30.),
                                "east glass return reaches room edge and remains visible from either side");
                        }
                    }
                    let header=g.uv(p.column,p.row);
                    assert!(pane.positions.iter().zip(&pane.normals).zip(&pane.uvs)
                        .any(|((v,n),uv)|v[1]>32.&&n[1]>0.5&&*uv==[header.0,header.2]),"native cap belongs on top");
                    let shadow=native_source_swatches(p,&g,[[3,6]])[0];
                    assert!(pane.positions.iter().zip(&pane.normals).zip(&pane.uvs)
                        .filter(|((v,n),_)|v[1]>30.1&&n[2]>0.5).all(|(_,uv)|*uv==shadow),"forward fascia is thin gray");
                    if grille_panel(name,p,&refs,&g) {
                        assert_eq!(pixel([3,19]),170,"own grille cap gray");
                        let top=g.uv(p.column,p.row+2);
                        assert!(pane.positions.iter().zip(&pane.normals).zip(&pane.uvs)
                            .any(|((v,n),uv)|v[1]>16.&&v[1]<16.1&&n[1]>0.5&&*uv==[top.0,top.2]));
                        let foot=g.uv(p.column,p.row+4);
                        assert!(pane.positions.iter().zip(&pane.normals).zip(&pane.uvs)
                            .any(|((v,n),uv)|v[2]>14.&&v[1]<0.1&&n[2]>0.5&&*uv==[foot.0,foot.3]),"lower grille is upright");
                        let dirty=g.uv(p.column,p.row+2);
                        let painted=|surface:&SurfaceMeshData|surface.positions.iter().zip(&surface.normals).zip(&surface.uvs)
                            .any(|((v,n),uv)|v[2]<6.1&&n[2]>0.5&&*uv==[dirty.0,dirty.3]);
                        assert!(!painted(&pane),"grille cap must not remain painted on glass");
                        let context=placements.iter().filter(|q|!grille_foot(name,q,&refs,&g)).cloned().collect::<Vec<_>>();
                        let mut fallback=SurfaceMeshData::default();
                        append(&mut fallback,name,p,&refs,&g,[0.,16.,0.,6.],crate::interior_models::selected_model(p.kind).unwrap(),&context);
                        assert!(painted(&fallback),"missing grille context preserves original art");
                        let mut protected=vec![false;w*h];protected[(p.row+4)*w+p.column]=true;
                        let mut rejected=Vec::new();candidates(name,&refs,&g,&mut protected,&mut rejected);
                        assert!(!rejected.iter().any(|q|q.column==p.column&&(q.row==p.row||q.row==p.row+4)),"custom grille foot protects whole assembly");
                    }
                    if p.height==4&&refs[(p.row+2)*w+p.column].source.metatile_id==0x19 {
                        let dirty=g.uv(p.column,p.row+3);
                        assert!(!pane.uvs.contains(&[dirty.0,dirty.3]),"mic art must not remain painted on glass");
                        let mut fallback=SurfaceMeshData::default();
                        append(&mut fallback,name,p,&refs,&g,[0.,16.,0.,6.],crate::interior_models::selected_model(p.kind).unwrap(),&[]);
                        assert!(fallback.uvs.contains(&[dirty.0,dirty.3]),"retain projected mic when its table is absent");
                    }
                }
                for i in p.indices(w) {assert!(!owned[i],"overlapping studio claims");owned[i]=true;}
                assert!(super::super::append_with_tables(&mut mesh,name,&refs,&g,p,&mut claimed,[0,0],&placements));
                assert!(p.indices(w).all(|i|claimed[i]));
                if panel(name,p,&refs,&g) {
                    assert!(p.indices(w).all(|i|!matches!(refs[i].source.tile_index,0x42|0x43)),"floor is not glass");
                }
                // Every claimed cell participates in exact source authentication.
                for bad in p.indices(w) {
                    let mut altered=tiles.clone();altered[bad].source.tile_index^=0x80;
                    let altered=altered.iter().collect::<Vec<_>>();
                    assert!(!handled(name,p,&altered,&g)&&equipment_size(name,p,&altered,&g).is_none());
                }
                if cap(name,p,&refs,&g) {
                    assert!(cap_ready(name,p,&refs,&g,&placements));
                    assert!(!cap_ready(name,p,&refs,&g,&[]));
                    let mut fallback=TerrainMeshData{authored_cells:vec![None;w*h],..Default::default()};
                    assert!(super::super::append(&mut fallback,name,&refs,&g,p,&mut vec![false;w*h],[0,0]));
                    assert_eq!(fallback.textured.indices.len(),12,"orphan cap retains its two original cells");
                }
                if grille_foot(name,p,&refs,&g) {
                    assert!(grille_foot_ready(name,p,&refs,&g,&placements));assert!(!grille_foot_ready(name,p,&refs,&g,&[]));
                    let mut fallback=TerrainMeshData{authored_cells:vec![None;w*h],..Default::default()};
                    assert!(super::super::append(&mut fallback,name,&refs,&g,p,&mut vec![false;w*h],[0,0]));
                    assert_eq!(fallback.textured.indices.len(),6,"orphan grille foot retains original art");
                    assert_eq!(refs[p.row*w+p.column+1].source.tile_index,0x43);
                    assert!(!owned[p.row*w+p.column+1],"adjacent floor is never part of the grille claim");
                }
                let mut excluded=vec![false;w*h];excluded[p.row*w+p.column]=true;
                let mut rejected=Vec::new();candidates(name,&refs,&g,&mut excluded,&mut rejected);
                assert!(!rejected.iter().any(|q|q.column==p.column&&q.row==p.row&&q.kind==p.kind));
            }
            for warp in &data.events.warps {for dy in 0..2 {for dx in 0..2 {
                assert!(!owned[(warp.y as usize*2+dy)*w+warp.x as usize*2+dx],"{name}: studio leaves warp clear");
            }}}
            let spans=perimeter_glass(name,&refs,&g,&placements,&mesh.authored_cells,[0,0],w as i32);
            assert_eq!(spans.len(),1,"each rendered studio east edge owns one perimeter opening");
            assert!(perimeter_glass(name,&refs,&g,&placements,&[],[0,0],w as i32).is_empty(),
                "missing authored geometry cannot remove a room enclosure");
            let frame=VisualWorldFrame {map_id:name.into(),source_map_size_core_tiles:Some(bevy::prelude::UVec2::new(w as u32/2,h as u32/2)),..Default::default()};
            let mut perimeter=TerrainMeshData {authored_cells:mesh.authored_cells.clone(),..Default::default()};
            super::super::append_room_perimeter(&mut perimeter,&frame,&refs,&g,&placements,&[],None);
            for surface in [&perimeter.solid,&perimeter.textured] {for ids in surface.indices.chunks_exact(3) {
                let points=[ids[0],ids[1],ids[2]].map(|i|surface.positions[i as usize]);
                let z=points.iter().map(|p|p[2]).sum::<f32>()/3.;
                assert!(!points.iter().all(|p|p[0]>=w as f32*8.-0.001)
                    || !spans.iter().any(|(a,b)|z>a+0.1&&z<b-0.1),
                    "opaque outer wall must not hide rendered glass from the east");
            }}
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert!(mesh.textured.normals.iter().all(|n|(Vec3::from_array(*n).length_squared()-1.).abs()<0.001));
            assert!(has_seating_source_candidate(name,&refs,None));
        }
    }
}
