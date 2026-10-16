//! Named parts of the authored radio desks; no colour-based role guessing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RadioFurniturePart { Rail, Return, Desk, Telephone, Terminal, Memo, Corner }

pub(super) fn expected(name:&str)->Option<u8> {
    use RadioFurniturePart::*;
    let mask=|parts:&[RadioFurniturePart]|parts.iter().fold(0,|bits,p|bits|1<<(*p as u8));
    Some(match name {
        "radio_phone_desk"|"radio_counter_extension"=>mask(&[Desk,Telephone]),
        "radio_memo_desk"=>mask(&[Desk,Telephone,Memo]),
        "radio_reception_u"=>mask(&[Rail,Return,Telephone,Terminal,Corner]),
        "radio_reception_l"=>mask(&[Rail,Return,Telephone,Memo,Corner]),
        _=>return None,
    })
}

/// Gray worktop/case, dark fascia, white trim, and black recess respectively.
pub(super) fn role(name:&str)->Option<(RadioFurniturePart,u8)> {
    use RadioFurniturePart::*;
    let (part,detail)=if let Some(s)=name.strip_prefix("Continuous service rail ") {(Rail,s)}
        else if let Some(s)=name.strip_prefix("West service return ").or_else(||name.strip_prefix("East service return ")) {(Return,s)}
        else if name=="West corner continuous folded stile"||name=="East corner continuous folded stile" {return Some((Corner,2));}
        else if let Some(s)=name.strip_prefix("Office desk ") {(Desk,s)}
        else if name=="Offset right stationery drawer" {return Some((Desk,0));}
        else if name=="Drawer recessed front" {return Some((Desk,1));}
        else if name=="Drawer pull" {return Some((Desk,2));}
        else if let Some(s)=name.strip_prefix("Telephone ") {
            return Some((Telephone,match s {
                "dark foot"|"black keypad well"|"receiver rounded end"|"handset bridge"=>3,
                "faceted pale casing"|"physical key"=>2,"receiver rim"=>1,_=>return None,
            }));
        } else if let Some(s)=name.strip_prefix("Broadcast terminal ") {
            return Some((Terminal,match s {
                "narrow floor foot"|"upper drawer"|"dark bezel"|"live display recess"=>3,
                "pedestal"|"raised key"=>1,"drawer apron"|"rear casing"=>0,"key shelf"=>2,
                _=>return None,
            }));
        } else if name=="Small original square memo support" {return Some((Memo,2));}
        else if name=="Memo folded corner" {return Some((Memo,1));}
        else {return None;};
    let paint=match detail {
        "closed folded cabinet"|"overhung paper laminate"=>0,
        "inset south fascia"|"finished rear inset"=>1,
        "narrow south return bead"|"folded front edge"=>2,
        "continuous recessed toe"|"folded leg"|"dark foot"|"recessed back modesty panel"|"lower rear cross rail"=>3,
        _=>return None,
    };
    Some((part,paint))
}
