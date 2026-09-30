// Source-complete second-pass interior drawings, individually inspected for role.
// This contains selected matcher signatures, not a duplicated content catalog.
struct AuthoredInteriorDrawing {
    tileset: &'static str,
    block: u16,
    origin: [u8; 2],
    size: [usize; 2],
    tiles: &'static [u16],
    ground: u16,
    kind: ModelKind,
}
const INTERIOR_DRAWINGS: &[AuthoredInteriorDrawing] = &[
    // house 03 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x03,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // house 05 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x05,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x00, 0x00, 0x00, 0x00],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // house 05 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x05,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x24, 0x4a, 0x34, 0x2c],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // house 08 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x08,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x00, 0x00, 0x00, 0x00],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // house 08 2,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x08,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x24, 0x4a, 0x34, 0x2c],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // house 0e 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x0e,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x00, 0x00, 0x00, 0x00],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // house 0e 2,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x0e,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x2d, 0x2e, 0x3d, 0x3e],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // house 16 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x16,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x24, 0x4a, 0x34, 0x2c],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // house 16 2,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x16,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x00, 0x00, 0x00, 0x00],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // house 1d 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x1d,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x00, 0x00, 0x00, 0x00],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // house 1e 2,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x1e,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x2d, 0x2e, 0x3d, 0x3e],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // house 10 0,0 Computer
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x10,
        origin: [0, 0],
        size: [2, 4],
        tiles: &[0x40, 0x41, 0x20, 0x21, 0x42, 0x43, 0x1e, 0x1f],
        ground: 0x01,
        kind: ModelKind::Computer,
    },
    // house 1c 2,1 Computer
    AuthoredInteriorDrawing {
        tileset: "house",
        block: 0x1c,
        origin: [2, 1],
        size: [2, 3],
        tiles: &[0x40, 0x41, 0x20, 0x21, 0x42, 0x43],
        ground: 0x01,
        kind: ModelKind::Computer,
    },
    // players_house 04 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x04,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_house 07 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x07,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x33, 0x32, 0x32, 0x34, 0x11, 0x11, 0x11, 0x11],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_house 20 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x20,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_house 01 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x01,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2c, 0x2d, 0x11, 0x11],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // players_house 01 2,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x01,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x11, 0x11, 0x11, 0x11],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_house 02 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x02,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x11, 0x11, 0x11, 0x11],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_house 02 2,1 Television
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x02,
        origin: [2, 1],
        size: [2, 3],
        tiles: &[0x06, 0x07, 0x16, 0x17, 0x18, 0x19],
        ground: 0x01,
        kind: ModelKind::Television,
    },
    // players_house 10 0,1 Computer
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x10,
        origin: [0, 1],
        size: [2, 3],
        tiles: &[0x20, 0x21, 0x30, 0x31, 0x0c, 0x0d],
        ground: 0x01,
        kind: ModelKind::Computer,
    },
    // players_house 18 2,1 Computer
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x18,
        origin: [2, 1],
        size: [2, 3],
        tiles: &[0x20, 0x21, 0x30, 0x31, 0x0c, 0x0d],
        ground: 0x01,
        kind: ModelKind::Computer,
    },
    // players_house 1f 2,1 Computer
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x1f,
        origin: [2, 1],
        size: [2, 3],
        tiles: &[0x11, 0x11, 0x26, 0x27, 0x36, 0x37],
        ground: 0x01,
        kind: ModelKind::Computer,
    },
    // players_house 1a 0,2 Radio
    AuthoredInteriorDrawing {
        tileset: "players_house",
        block: 0x1a,
        origin: [0, 2],
        size: [2, 2],
        tiles: &[0x36, 0x37, 0x08, 0x09],
        ground: 0x01,
        kind: ModelKind::Radio,
    },
    // players_room 01 2,2 Radio
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x01,
        origin: [2, 2],
        size: [2, 2],
        tiles: &[0x42, 0x43, 0x52, 0x53],
        ground: 0x02,
        kind: ModelKind::Radio,
    },
    // players_room 03 0,1 Television
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x03,
        origin: [0, 1],
        size: [2, 3],
        tiles: &[0x3b, 0x3c, 0x4b, 0x4c, 0x5b, 0x5c],
        ground: 0x02,
        kind: ModelKind::Television,
    },
    // players_room 03 2,0 Bookcase
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x03,
        origin: [2, 0],
        size: [2, 4],
        tiles: &[0x05, 0x06, 0x15, 0x16, 0x25, 0x26, 0x35, 0x36],
        ground: 0x02,
        kind: ModelKind::Bookcase,
    },
    // players_room 04 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x04,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 08 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x08,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 0b 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x0b,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 0e 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x0e,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 11 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x11,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 14 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x14,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 17 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x17,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 02 0,0 WallDomestic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x02,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02],
        ground: 0x01,
        kind: ModelKind::WallDomestic,
    },
    // players_room 20 2,0 PlantMagna
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x20,
        origin: [2, 0],
        size: [2, 3],
        tiles: &[0x27, 0x28, 0x37, 0x38, 0x46, 0x56],
        ground: 0x01,
        kind: ModelKind::PlantMagna,
    },
    // players_room 21 2,0 PlantTropic
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x21,
        origin: [2, 0],
        size: [2, 3],
        tiles: &[0x47, 0x48, 0x57, 0x58, 0x46, 0x56],
        ground: 0x01,
        kind: ModelKind::PlantTropic,
    },
    // players_room 22 2,0 PlantJumbo
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x22,
        origin: [2, 0],
        size: [2, 4],
        tiles: &[0x07, 0x08, 0x17, 0x18, 0x17, 0x18, 0x46, 0x56],
        ground: 0x01,
        kind: ModelKind::PlantJumbo,
    },
    // players_room 1f 0,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x1f,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x44, 0x45, 0x54, 0x55],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // players_room 23 0,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x23,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x4e, 0x4f, 0x5e, 0x5f],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // players_room 24 0,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x24,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1e, 0x1f],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // players_room 25 0,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "players_room",
        block: 0x25,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2e, 0x2f, 0x3e, 0x3f],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // traditional_house 03 0,0 WallTraditional
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x03,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
        ground: 0x50,
        kind: ModelKind::WallTraditional,
    },
    // traditional_house 25 0,0 WallTraditional
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x25,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
        ground: 0x01,
        kind: ModelKind::WallTraditional,
    },
    // traditional_house 2d 0,0 WallTraditional
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x2d,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x4e, 0x4f, 0x4e, 0x4f, 0x5e, 0x5e, 0x5e, 0x5e],
        ground: 0x50,
        kind: ModelKind::WallTraditional,
    },
    // traditional_house 14 0,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x14,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x47, 0x48, 0x49, 0x4a, 0x57, 0x58, 0x59, 0x5a],
        ground: 0x50,
        kind: ModelKind::PictureFrame,
    },
    // traditional_house 1a 0,0 Bookcase
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x1a,
        origin: [0, 0],
        size: [2, 4],
        tiles: &[0x23, 0x24, 0x3e, 0x3f, 0x3e, 0x3f, 0x18, 0x19],
        ground: 0x50,
        kind: ModelKind::Bookcase,
    },
    // traditional_house 1a 2,0 Bookcase
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x1a,
        origin: [2, 0],
        size: [2, 4],
        tiles: &[0x23, 0x24, 0x3e, 0x3f, 0x3e, 0x3f, 0x18, 0x19],
        ground: 0x50,
        kind: ModelKind::Bookcase,
    },
    // traditional_house 01 0,2 Radio
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x01,
        origin: [0, 2],
        size: [2, 2],
        tiles: &[0x0a, 0x0b, 0x1a, 0x1b],
        ground: 0x50,
        kind: ModelKind::Radio,
    },
    // traditional_house 23 0,2 Radio
    AuthoredInteriorDrawing {
        tileset: "traditional_house",
        block: 0x23,
        origin: [0, 2],
        size: [2, 2],
        tiles: &[0x0a, 0x0b, 0x1a, 0x1b],
        ground: 0x50,
        kind: ModelKind::Radio,
    },
    // mansion 04 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x04,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12],
        ground: 0x01,
        kind: ModelKind::WallClinical,
    },
    // mansion 07 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x07,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x03, 0x03, 0x03, 0x03, 0x13, 0x13, 0x13, 0x13],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // mansion 08 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x08,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x03, 0x03, 0x03, 0x03, 0x13, 0x13, 0x13, 0x13],
        ground: 0x01,
        kind: ModelKind::WallClinical,
    },
    // mansion 05 2,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x05,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x12, 0x12, 0x12, 0x12],
        ground: 0x01,
        kind: ModelKind::WallClinical,
    },
    // mansion 06 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x06,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x12, 0x12, 0x12, 0x12],
        ground: 0x01,
        kind: ModelKind::WallClinical,
    },
    // mansion 0f 0,1 Bookcase
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x0f,
        origin: [0, 1],
        size: [2, 3],
        tiles: &[0x36, 0x38, 0x22, 0x23, 0x32, 0x33],
        ground: 0x01,
        kind: ModelKind::Bookcase,
    },
    // mansion 0f 2,0 PendulumClock
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x0f,
        origin: [2, 0],
        size: [2, 4],
        tiles: &[0x36, 0x38, 0x24, 0x25, 0x34, 0x35, 0x32, 0x33],
        ground: 0x01,
        kind: ModelKind::PendulumClock,
    },
    // mansion 11 0,1 Bookcase
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x11,
        origin: [0, 1],
        size: [2, 3],
        tiles: &[0x36, 0x38, 0x22, 0x23, 0x32, 0x33],
        ground: 0x11,
        kind: ModelKind::Bookcase,
    },
    // mansion 11 2,2 PottedPlant
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x11,
        origin: [2, 2],
        size: [2, 2],
        tiles: &[0x2e, 0x2f, 0x5e, 0x5f],
        ground: 0x11,
        kind: ModelKind::PottedPlant,
    },
    // mansion 17 2,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "mansion",
        block: 0x17,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x30, 0x31],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // lab 05 0,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x05,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x08, 0x09, 0x18, 0x19],
        ground: 0x10,
        kind: ModelKind::PictureFrame,
    },
    // lab 05 2,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x05,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x08, 0x09, 0x18, 0x19],
        ground: 0x10,
        kind: ModelKind::PictureFrame,
    },
    // lab 09 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x09,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x30, 0x31],
        ground: 0x10,
        kind: ModelKind::WindowWall,
    },
    // lab 09 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x09,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x22, 0x23, 0x32, 0x33],
        ground: 0x10,
        kind: ModelKind::WindowWall,
    },
    // lab 20 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x20,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x30, 0x31],
        ground: 0x10,
        kind: ModelKind::WindowWall,
    },
    // lab 20 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x20,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x30, 0x31],
        ground: 0x10,
        kind: ModelKind::WindowWall,
    },
    // lab 1a 0,0 LabRestorationMachine
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x1a,
        origin: [0, 0],
        size: [2, 4],
        tiles: &[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10],
        ground: 0x10,
        kind: ModelKind::LabRestorationMachine,
    },
    // lab 1b 0,0 LabRestorationMachine
    AuthoredInteriorDrawing {
        tileset: "lab",
        block: 0x1b,
        origin: [0, 0],
        size: [2, 4],
        tiles: &[0x4c, 0x4d, 0x5c, 0x5d, 0x4e, 0x4f, 0x5e, 0x5f],
        ground: 0x10,
        kind: ModelKind::LabRestorationMachine,
    },
    // radio_tower 04 0,0 WallAcoustic
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x04,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x10, 0x10],
        ground: 0x01,
        kind: ModelKind::WallAcoustic,
    },
    // radio_tower 02 0,0 WallAcoustic
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x02,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x11, 0x11, 0x10, 0x10],
        ground: 0x01,
        kind: ModelKind::WallAcoustic,
    },
    // radio_tower 02 2,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x02,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x03, 0x04, 0x13, 0x14],
        ground: 0x01,
        kind: ModelKind::PictureFrame,
    },
    // radio_tower 0a 0,0 Bookcase
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x0a,
        origin: [0, 0],
        size: [2, 4],
        tiles: &[0x30, 0x31, 0x1a, 0x1b, 0x1a, 0x1b, 0x0a, 0x0b],
        ground: 0x01,
        kind: ModelKind::Bookcase,
    },
    // radio_tower 0a 2,0 Bookcase
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x0a,
        origin: [2, 0],
        size: [2, 4],
        tiles: &[0x30, 0x31, 0x1a, 0x1b, 0x1a, 0x1b, 0x0a, 0x0b],
        ground: 0x01,
        kind: ModelKind::Bookcase,
    },
    // radio_tower 0b 0,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x0b,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x46, 0x47, 0x48, 0x49],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 0b 2,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x0b,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x46, 0x47, 0x48, 0x49],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 0b 0,2 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x0b,
        origin: [0, 2],
        size: [2, 2],
        tiles: &[0x44, 0x45, 0x4a, 0x4b],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 0b 2,2 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x0b,
        origin: [2, 2],
        size: [2, 2],
        tiles: &[0x44, 0x45, 0x4a, 0x4b],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 14 0,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x14,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2c, 0x2d, 0x3c, 0x3d],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 2c 0,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x2c,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2c, 0x2d, 0x3c, 0x3d],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 2d 0,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x2d,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2c, 0x2d, 0x3c, 0x3d],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 39 0,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x39,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2c, 0x2d, 0x3c, 0x3d],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 3a 0,0 ArcadeStool
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x3a,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x2c, 0x2d, 0x3c, 0x3d],
        ground: 0x01,
        kind: ModelKind::ArcadeStool,
    },
    // radio_tower 11 0,1 Chair
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x11,
        origin: [0, 1],
        size: [2, 2],
        tiles: &[0x52, 0x53, 0x50, 0x51],
        ground: 0x01,
        kind: ModelKind::Chair,
    },
    // radio_tower 11 2,1 Chair
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x11,
        origin: [2, 1],
        size: [2, 2],
        tiles: &[0x52, 0x34, 0x50, 0x51],
        ground: 0x01,
        kind: ModelKind::Chair,
    },
    // radio_tower 24 0,1 Chair
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x24,
        origin: [0, 1],
        size: [2, 2],
        tiles: &[0x52, 0x53, 0x50, 0x51],
        ground: 0x01,
        kind: ModelKind::Chair,
    },
    // radio_tower 24 2,0 Bookcase
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x24,
        origin: [2, 0],
        size: [2, 4],
        tiles: &[0x30, 0x31, 0x1a, 0x1b, 0x1a, 0x1b, 0x0a, 0x0b],
        ground: 0x01,
        kind: ModelKind::Bookcase,
    },
    // radio_tower 16 0,1 BroadcastConsole
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x16,
        origin: [0, 1],
        size: [4, 3],
        tiles: &[
            0x11, 0x54, 0x56, 0x57, 0x05, 0x55, 0x58, 0x59, 0x15, 0x17, 0x17, 0x16,
        ],
        ground: 0x01,
        kind: ModelKind::BroadcastConsole,
    },
    // radio_tower 1c 2,1 BroadcastConsole
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x1c,
        origin: [2, 1],
        size: [2, 3],
        tiles: &[0x56, 0x57, 0x58, 0x59, 0x4c, 0x4d],
        ground: 0x01,
        kind: ModelKind::BroadcastConsole,
    },
    // radio_tower 2b 2,2 BroadcastConsole
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x2b,
        origin: [2, 2],
        size: [2, 2],
        tiles: &[0x56, 0x57, 0x58, 0x59],
        ground: 0x01,
        kind: ModelKind::BroadcastConsole,
    },
    // radio_tower 19 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x19,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x25, 0x26],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // radio_tower 1a 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x1a,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x29, 0x2a],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // radio_tower 18 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x18,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x29, 0x2a],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // radio_tower 19 2,2 PottedPlant
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x19,
        origin: [2, 2],
        size: [2, 2],
        tiles: &[0x35, 0x36, 0x08, 0x19],
        ground: 0x01,
        kind: ModelKind::PottedPlant,
    },
    // radio_tower 1e 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x1e,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x29, 0x2a],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // radio_tower 1e 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "radio_tower",
        block: 0x1e,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x20, 0x21, 0x29, 0x2a],
        ground: 0x01,
        kind: ModelKind::WindowWall,
    },
    // battle_tower_inside 07 0,0 LinkSeat
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x07,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1e, 0x1f],
        ground: 0x11,
        kind: ModelKind::LinkSeat,
    },
    // battle_tower_inside 07 2,0 LinkSeat
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x07,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1e, 0x1f],
        ground: 0x11,
        kind: ModelKind::LinkSeat,
    },
    // battle_tower_inside 05 0,0 Computer
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x05,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0c, 0x0d, 0x1c, 0x1d],
        ground: 0x11,
        kind: ModelKind::Computer,
    },
    // battle_tower_inside 06 0,2 LinkSeat
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x06,
        origin: [0, 2],
        size: [2, 2],
        tiles: &[0x08, 0x09, 0x18, 0x19],
        ground: 0x11,
        kind: ModelKind::LinkSeat,
    },
    // battle_tower_inside 02 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x02,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x06, 0x0a, 0x0a, 0x0a, 0x16, 0x1a, 0x1a, 0x1a],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // battle_tower_inside 03 0,0 ReceptionCounter
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x03,
        origin: [0, 0],
        size: [4, 2],
        tiles: &[0x54, 0x55, 0x54, 0x55, 0x50, 0x51, 0x50, 0x51],
        ground: 0x11,
        kind: ModelKind::TowerReception,
    },
    // battle_tower_inside 1b 0,0 ReceptionCounter
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x1b,
        origin: [0, 0],
        size: [4, 3],
        tiles: &[
            0x43, 0x45, 0x44, 0x45, 0x53, 0x55, 0x54, 0x55, 0x50, 0x51, 0x50, 0x51,
        ],
        ground: 0x11,
        kind: ModelKind::TowerReception,
    },
    // battle_tower_inside 1d 0,0 ReceptionCounter
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x1d,
        origin: [0, 0],
        size: [4, 3],
        tiles: &[
            0x44, 0x45, 0x44, 0x45, 0x54, 0x55, 0x54, 0x55, 0x50, 0x51, 0x50, 0x51,
        ],
        ground: 0x11,
        kind: ModelKind::TowerReception,
    },
    // battle_tower_inside 1e 0,0 ReceptionCounter
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x1e,
        origin: [0, 0],
        size: [4, 3],
        tiles: &[
            0x44, 0x45, 0x44, 0x45, 0x48, 0x49, 0x4a, 0x55, 0x50, 0x51, 0x50, 0x51,
        ],
        ground: 0x11,
        kind: ModelKind::TowerReception,
    },
    // battle_tower_inside 04 0,0 GateTerminal
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x04,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x54, 0x55, 0x50, 0x51],
        ground: 0x11,
        kind: ModelKind::GateTerminal,
    },
    // battle_tower_inside 04 2,0 PictureFrame
    AuthoredInteriorDrawing {
        tileset: "battle_tower_inside",
        block: 0x04,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x02, 0x03, 0x12, 0x13],
        ground: 0x11,
        kind: ModelKind::PictureFrame,
    },
    // mart 01 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x01,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 03 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x03,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 03 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x03,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 08 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x08,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 08 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x08,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x2a, 0x2b],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 09 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x09,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 09 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x09,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 0a 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x0a,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x2a, 0x2b],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 0a 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x0a,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 17 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x17,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 18 2,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x18,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 26 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x26,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // mart 2f 0,0 WindowWall
    AuthoredInteriorDrawing {
        tileset: "mart",
        block: 0x2f,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x0e, 0x0f, 0x1c, 0x1d],
        ground: 0x48,
        kind: ModelKind::WindowWall,
    },
    // pokecenter 02 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x02,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x04, 0x05, 0x14, 0x15],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 02 2,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x02,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 03 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x03,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 03 2,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x03,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x0f],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 08 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x08,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 08 2,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x08,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x20, 0x21],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 13 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x13,
        origin: [0, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // pokecenter 13 2,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "pokecenter",
        block: 0x13,
        origin: [2, 0],
        size: [2, 2],
        tiles: &[0x02, 0x02, 0x02, 0x02],
        ground: 0x11,
        kind: ModelKind::WallClinical,
    },
    // gate 02 0,0 WallClinical
    AuthoredInteriorDrawing {
        tileset: "gate",
        block: 0x02,
        origin: [0, 0],
        size: [4, 3],
        tiles: &[
            0x5c, 0x5d, 0x5c, 0x5d, 0x10, 0x10, 0x10, 0x10, 0x11, 0x11, 0x11, 0x11,
        ],
        ground: 0x01,
        kind: ModelKind::WallClinical,
    },
];

fn append_signature_candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    for drawing in INTERIOR_DRAWINGS {
        let Some(ground) = ground_sample(cells, map, drawing.tileset, drawing.ground) else {
            continue;
        };
        let [w, h] = drawing.size;
        if w > g.width || h > g.height {
            continue;
        }
        for row in 0..=g.height - h {
            for column in 0..=g.width - w {
                let complete = (0..h).all(|y| {
                    (0..w).all(|x| {
                        let i = (row + y) * g.width + column + x;
                        let s = &cells[i].source;
                        !claimed[i]
                            && s.tileset_id.as_ref() == drawing.tileset
                            && s.metatile_id == drawing.block
                            && s.subtile_column as usize == drawing.origin[0] as usize + x
                            && s.subtile_row as usize == drawing.origin[1] as usize + y
                            && s.tile_index == drawing.tiles[y * w + x]
                    })
                });
                if !complete {
                    continue;
                }
                let (depth_pixels, height_pixels) = proportions(drawing.kind, w, h);
                let p = Placement {
                    kind: drawing.kind,
                    column,
                    row,
                    width: w,
                    height: h,
                    ground,
                    depth_pixels,
                    height_pixels,
                    base_pixels: 0.0,
                    footing_pixels: None,
                };
                for i in p.indices(g.width) {
                    claimed[i] = true;
                }
                out.push(p);
            }
        }
    }
}
fn append_stair_candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    let groups = grouped_flat_card_placements(cells, g, 0x01, false, |s| {
        house_stair_local(map, s).map(|(x, y, _)| (x, y, 2, 2))
    });
    for group in groups {
        let source = &cells[group.row * g.width + group.column].source;
        let Some((_, _, kind)) = house_stair_local(map, source) else {
            continue;
        };
        let ground_id = if source.tileset_id.as_ref() == "traditional_house" {
            0x50
        } else {
            0x01
        };
        let Some(ground) = ground_sample(cells, map, &source.tileset_id, ground_id) else {
            continue;
        };
        let p = Placement {
            kind: ModelKind::StairFlight,
            column: group.column,
            row: group.row,
            width: 2,
            height: 2,
            ground,
            depth_pixels: 16.0,
            height_pixels: 22.4,
            base_pixels: if kind == crate::players_house::StairKind::DownWest {
                -16.0
            } else {
                0.0
            },
            footing_pixels: None,
        };
        if p.indices(g.width).any(|i| claimed[i]) {
            continue;
        }
        for i in p.indices(g.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
}
/// Preserve the exact live picture. Only its frame and backing become modeled;
/// the source subject is never replaced by a generic invented geometric icon.
fn append_framed_source_art(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    bounds: [f32; 4],
    repeats: usize,
) {
    let [west, east, north, south] = bounds;
    let columns = p.width / repeats;
    let bottom =
        p.base_pixels * g.tile_height / 8.0 + p.height_pixels * g.tile_height / 8.0 * 0.067;
    let top = p.base_pixels * g.tile_height / 8.0 + p.height_pixels * g.tile_height / 8.0 * 0.927;
    let z = south - (south - north) * 0.14;
    for unit in 0..repeats {
        let left = west + (east - west) * unit as f32 / repeats as f32;
        let right = west + (east - west) * (unit + 1) as f32 / repeats as f32;
        let x0 = left + (right - left) * 0.085;
        let x1 = right - (right - left) * 0.085;
        for y in 0..p.height {
            for x in 0..columns {
                let left = x0 + (x1 - x0) * x as f32 / columns as f32;
                let right = x0 + (x1 - x0) * (x + 1) as f32 / columns as f32;
                let y_top = top - (top - bottom) * y as f32 / p.height as f32;
                let y_bottom = top - (top - bottom) * (y + 1) as f32 / p.height as f32;
                let (u0, u1, v0, v1) = g.uv(p.column + unit * columns + x, p.row + y);
                append_quad(
                    &mut mesh.textured,
                    [
                        [right, y_bottom, z],
                        [right, y_top, z],
                        [left, y_top, z],
                        [left, y_bottom, z],
                    ],
                    [0.0, 0.0, 1.0],
                    [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
                    TEXTURED_SHADE,
                );
            }
        }
    }
}

#[cfg(test)]
mod signature_tests {
    use super::*;
    use std::sync::Arc;
    fn source_fixture(d: &AuthoredInteriorDrawing) -> (Vec<VisualTile>, GridGeometry) {
        let width = d.size[0] + 2;
        let height = d.size[1] + 2;
        let mut tiles = Vec::new();
        for row in 0..height {
            for column in 0..width {
                tiles.push(VisualTile {
                    column: column as u32,
                    row: row as u32,
                    texture: Handle::default(),
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: Arc::from(d.tileset),
                        metatile_id: 0,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: d.ground,
                    },
                });
            }
        }
        for y in 0..d.size[1] {
            for x in 0..d.size[0] {
                tiles[y * width + x].source = VisualTileSource {
                    tileset_id: Arc::from(d.tileset),
                    metatile_id: d.block,
                    subtile_column: d.origin[0] + x as u8,
                    subtile_row: d.origin[1] + y as u8,
                    tile_index: d.tiles[y * d.size[0] + x],
                };
            }
        }
        (
            tiles,
            GridGeometry {
                width,
                height,
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: 0.0,
                origin_z: 0.0,
            },
        )
    }
    #[test]
    fn selected_signatures_are_complete_bounded_drawings() {
        assert_eq!(INTERIOR_DRAWINGS.len(), 128);
        for d in INTERIOR_DRAWINGS {
            assert_eq!(d.tiles.len(), d.size[0] * d.size[1]);
            assert!(d.origin[0] as usize + d.size[0] <= 4);
            assert!(d.origin[1] as usize + d.size[1] <= 4);
            assert!(d.size.iter().all(|&v| v > 0));
        }
    }
    #[test]
    fn live_bedroom_plant_changes_select_three_distinct_sculptures() {
        for (block, kind) in [
            (0x20, ModelKind::PlantMagna),
            (0x21, ModelKind::PlantTropic),
            (0x22, ModelKind::PlantJumbo),
        ] {
            let d = INTERIOR_DRAWINGS
                .iter()
                .find(|d| d.tileset == "players_room" && d.block == block)
                .unwrap();
            let (mut tiles, g) = source_fixture(d);
            let refs: Vec<_> = tiles.iter().collect();
            let p = resolve("PlayersHouse2F", &refs, &g, None);
            assert!(p.iter().any(|p| p.kind == kind));
            tiles[0].source.tile_index ^= 1;
            let refs: Vec<_> = tiles.iter().collect();
            assert!(
                !resolve("PlayersHouse2F", &refs, &g, None)
                    .iter()
                    .any(|p| p.kind == kind)
            );
        }
    }
    #[test]
    fn modeled_picture_retains_live_subject_on_recessed_textured_surface() {
        let d = INTERIOR_DRAWINGS
            .iter()
            .find(|d| d.tileset == "players_room" && d.block == 0x23)
            .unwrap();
        let (tiles, g) = source_fixture(d);
        let refs: Vec<_> = tiles.iter().collect();
        let p = resolve("PlayersHouse2F", &refs, &g, None)
            .into_iter()
            .find(|p| p.kind == ModelKind::PictureFrame)
            .unwrap();
        let mut mesh = TerrainMeshData::default();
        let mut claims = vec![false; tiles.len()];
        assert!(append(&mut mesh, &refs, &g, &p, &mut claims));
        // Four original-floor quads plus four unaltered live picture quads.
        assert_eq!(mesh.textured.quad_count(), 8);
        assert!(
            mesh.textured
                .normals
                .chunks_exact(4)
                .filter(|n| n[0] == [0.0, 0.0, 1.0])
                .count()
                == 4
        );
        assert!(mesh.solid.positions.len() > 100);
    }
}
