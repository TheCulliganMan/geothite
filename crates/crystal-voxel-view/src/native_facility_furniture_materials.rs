//! Original facility part names authenticate live gold/brown/light/ink finishes.
pub(super) fn expected(name: &str) -> bool {
    matches!(
        name,
        "facility_workbench"
            | "facility_square_document_table"
            | "facility_meeting_table"
            | "facility_document_side_desk"
            | "facility_square_back_chair"
    )
}
pub(super) fn role(model: &str, part: &str, material: &str) -> Option<u8> {
    if !expected(model) || !material.is_empty() {
        return None;
    }
    if model == "facility_workbench" {
        return match part {
            "Laboratory bench overhung paper laminate"
            | "Inset bench drawer"
            | "Left terminal rear casing"
            | "Right instrument faceted upright"
            | "Keyboard spacebar" => Some(0),
            "Continuous bench apron"
            | "Laboratory bench folded leg"
            | "Laboratory bench lower rear cross rail"
            | "Keyboard broad shallow case"
            | "Left display pedestal"
            | "Left terminal deep dark bezel"
            | "Right instrument dark front inset"
            | "Right instrument pale side facet"
            | "Right instrument raised foot"
            | "Right instrument upper face" => Some(1),
            "Laboratory bench folded front edge"
            | "Keyboard raised cap"
            | "Left terminal narrow pale lip"
            | "Right instrument narrow status marker" => Some(2),
            "Laboratory bench dark foot"
            | "Laboratory bench recessed back modesty panel"
            | "Left terminal live display recess"
            | "Right instrument lower bay"
            | "Slim drawer pull" => Some(3),
            _ => None,
        };
    }
    // The four table/chair kits share these named joinery and document parts.
    match part {
        "Complete folded tabletop"
        | "Inset ochre writing surface"
        | "Folded square seat"
        | "Backrest oak upright"
        | "Complete solid rear backrest"
        | "Pale recessed backrest front"
        | "Finished rear backrest inset"
        | "Shallow drawer box"
        | "Recessed drawer front" => Some(0),
        "Square folded oak leg"
        | "Square chair leg"
        | "Recessed full-width apron"
        | "Recessed side apron"
        | "Folio folded spine"
        | "Recessed tea surface"
        | "Inlaid center timber joint"
        | "Inset quiet linen seat" => Some(1),
        "Leg light outer facet"
        | "Square chair crest rail"
        | "Folio cream page edges"
        | "Top writing leaf"
        | "Faceted cup with true open interior"
        | "Cup handle upper return"
        | "Cup handle lower return"
        | "Cup handle open outer strap"
        | "Drawer slim brass grip" => Some(2),
        "Dark ferrule"
        | "Chair leg ferrule"
        | "Under-seat open-side front rail"
        | "Document folio dark cover" => Some(3),
        _ => None,
    }
}
