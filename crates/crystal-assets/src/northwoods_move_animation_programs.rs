//! Native object choreography; no donor aliases or background palette flashes.
use super::IMPORTED_MOVES;

pub(super) fn programs() -> Vec<(&'static str, Vec<String>)> {
    let mut water = Script::new();
    water.sound("SFX_BUBBLEBEAM");
    for lane in [0, -6, 6] {
        for (x, y) in [(64, 82), (84, 73), (106, 64), (130, 55)] {
            water.pose(&[("WAVE", x, y + lane)], 3);
        }
    }
    water.hit(10);

    let mut jet = Script::new();
    jet.pose(&[("WATER", 52, 84), ("WAKE", 36, 88)], 9);
    jet.sound("SFX_WATER_GUN");
    for (x, y) in [(64, 80), (84, 72), (108, 63), (130, 55)] {
        jet.pose(&[("WATER", x, y), ("WAKE", x - 20, y + 8)], 2);
    }
    jet.pose(&[("WAVE", 130, 55), ("WATER", 138, 62)], 9);
    jet.hit(9);

    let mut ice = Script::new();
    ice.pose(&[("SHARD", 54, 83)], 8);
    ice.sound("SFX_SHARPEN");
    for (x, y) in [(72, 76), (92, 68), (112, 60), (132, 53)] {
        ice.pose(&[("SHARD", x, y)], 3);
    }
    ice.pose(&[("SHARD", 124, 46), ("SHARD", 141, 63)], 6);
    ice.pose(&[("FROST", 127, 45), ("FROST", 140, 66)], 7);
    ice.hit(9);

    let mut energy = Script::new();
    for (left, right, y) in [(40, 62, 79), (48, 54, 67), (62, 40, 82)] {
        energy.pose(
            &[("ORB", 52, 80), ("LEAF", left, y), ("LEAF", right, 160 - y)],
            5,
        );
    }
    energy.sound("SFX_PSYCHIC");
    for (x, y) in [(66, 78), (82, 72), (100, 65), (118, 59), (133, 54)] {
        energy.pose(&[("ORB", x, y), ("LEAF", x - 5, y + 5)], 4);
    }
    energy.pose(&[("LEAF", 119, 42), ("LEAF", 144, 66), ("ORB", 132, 54)], 8);
    energy.hit(8);

    let mut drain = Script::new();
    drain.pose(&[("FIST", 54, 84)], 7);
    drain.sound("SFX_MEGA_PUNCH");
    for (x, y) in [(78, 74), (104, 64), (132, 54)] {
        drain.pose(&[("FIST", x, y)], 3);
    }
    drain.hit(7);
    drain.sound("SFX_GIGA_DRAIN");
    for (x, y) in [(126, 57), (108, 64), (88, 72), (69, 80), (51, 87)] {
        drain.pose(&[("MOTE", x, y), ("MOTE", x + 7, y - 9)], 4);
    }
    drain.pose(&[("ORB", 51, 87)], 6);

    let mut signal = Script::new();
    signal.pose(&[("BEAM_GREEN", 54, 83)], 8);
    signal.sound("SFX_PSYBEAM");
    for (step, (x, y)) in [(68, 73), (84, 80), (100, 60), (116, 67), (134, 49)]
        .into_iter()
        .enumerate()
    {
        let (front, back) = if step % 2 == 0 {
            ("BEAM_BLUE", "BEAM_GREEN")
        } else {
            ("BEAM_GREEN", "BEAM_BLUE")
        };
        signal.pose(&[(front, x, y), (back, x - 17, y + 9)], 4);
    }
    signal.pose(&[("BEAM_GREEN", 128, 47), ("BEAM_BLUE", 136, 64)], 7);
    signal.hit(10);

    let mut iron = Script::new();
    for y in [86, 82, 86] {
        iron.pose(&[("RAM", 52, y), ("STAR", 46, y - 16)], 5);
    }
    iron.sound("SFX_HEADBUTT");
    for (x, y) in [(76, 76), (104, 64), (130, 54)] {
        iron.pose(&[("RAM", x, y), ("METAL", x + 7, y)], 3);
    }
    iron.hit(8);
    iron.pose(&[("STAR", 119, 43), ("STAR", 142, 67)], 7);
    iron.hit(7);

    let mut scissor = Script::new();
    scissor.pose(&[("CUT_LEFT", 58, 81), ("CUT_RIGHT", 59, 81)], 8);
    scissor.sound("SFX_CUT");
    for (x, y) in [(100, 38), (116, 48), (134, 60)] {
        scissor.pose(&[("CUT_LEFT", x, y)], 4);
    }
    scissor.sound("SFX_CUT");
    for (x, y) in [(104, 77), (119, 64), (134, 52)] {
        scissor.pose(&[("CUT_RIGHT", x, y), ("CUT_LEFT", 133, 55)], 4);
    }
    scissor.pose(&[("CUT_LEFT", 132, 54), ("CUT_RIGHT", 132, 54)], 8);
    scissor.hit(8);

    let mut wind = Script::new();
    wind.sound("SFX_RAZOR_WIND");
    for (x, y) in [(57, 83), (73, 65), (91, 72), (109, 48), (130, 54)] {
        wind.pose(
            &[
                ("CURL_LEFT", x, y),
                ("CURL_RIGHT", x - 8, y - 17),
                ("STAR", x + 7, y + 12),
            ],
            6,
        );
    }
    wind.pose(&[("CURL_LEFT", 137, 42), ("CURL_RIGHT", 125, 65)], 8);
    wind.hit(8);

    let mut voice = Script::new();
    voice.pose(&[("VOICE", 52, 84), ("STAR", 62, 82)], 8);
    voice.sound("SFX_SING");
    for (x, y) in [(72, 77), (95, 69), (119, 60), (134, 54)] {
        voice.pose(
            &[
                ("VOICE", x, y - 8),
                ("VOICE", x - 6, y + 10),
                ("STAR", x + 5, y),
            ],
            5,
        );
    }
    voice.pose(&[("WAVE", 133, 55), ("STAR", 133, 55)], 8);
    voice.hit(9);

    let mut gleam = Script::new();
    gleam.pose(&[("STAR", 132, 55)], 10);
    gleam.sound("SFX_PSYCHIC");
    for distance in [8, 14, 18, 12] {
        gleam.pose(
            &[
                ("STAR", 132 - distance, 55),
                ("STAR", 132 + distance, 55),
                ("STAR", 132, 55 - distance),
                ("STAR", 132, 55 + distance),
            ],
            6,
        );
    }
    gleam.pose(&[("CRESCENT", 121, 46), ("CRESCENT", 143, 64)], 7);
    gleam.hit(9);

    let mut rough = Script::new();
    rough.pose(&[("PALM", 56, 85)], 8);
    rough.sound("SFX_COMET_PUNCH");
    for (first, second, x, y) in [
        ("PALM", "FIST", 123, 43),
        ("FIST", "PALM", 142, 62),
        ("PALM", "FIST", 125, 68),
    ] {
        rough.pose(&[(first, x, y), (second, 265 - x, 110 - y)], 7);
    }
    rough.hit(7);
    rough.pose(&[("PALM", 119, 54), ("FIST", 143, 54)], 6);
    rough.hit(8);

    let mut moon = Script::new();
    moon.pose(&[("CRESCENT", 53, 66), ("STAR", 54, 65)], 12);
    moon.sound("SFX_MOONLIGHT");
    moon.pose(&[("MOON", 54, 65)], 7);
    for (x, y) in [(70, 52), (88, 38), (106, 34), (123, 41), (133, 54)] {
        moon.pose(&[("MOON", x, y)], 5);
    }
    moon.pose(
        &[
            ("CRESCENT", 120, 43),
            ("CRESCENT", 142, 66),
            ("STAR", 132, 54),
        ],
        8,
    );
    moon.hit(10);

    let mut poison = Script::new();
    poison.pose(&[("VENOM", 54, 83)], 9);
    poison.sound("SFX_POISON_STING");
    for (x, y) in [(111, 65), (122, 59), (134, 54)] {
        poison.pose(&[("VENOM", x, y)], 4);
    }
    poison.pose(
        &[("VENOM", 136, 54), ("DROP", 124, 42), ("DROP", 141, 68)],
        8,
    );
    poison.sound("SFX_POISON");
    poison.pose(&[("DROP", 118, 36), ("DROP", 145, 73)], 8);
    poison.hit(8);

    IMPORTED_MOVES
        .into_iter()
        .zip(
            [
                water, jet, ice, energy, drain, signal, iron, scissor, wind, voice, gleam, rough,
                moon, poison,
            ]
            .map(Script::finish),
        )
        .collect()
}

struct Script(Vec<String>);
impl Script {
    fn new() -> Self {
        Self(vec![
            "anim_1gfx BATTLE_ANIM_GFX_HIT".to_string(),
            "anim_wait 6".to_string(),
        ])
    }
    fn pose(&mut self, objects: &[(&str, i32, i32)], frames: u8) {
        self.0.push("anim_clearobjs".to_string());
        for (name, x, y) in objects {
            self.0.push(format!(
                "anim_obj BATTLE_ANIM_OBJ_MN_IMPORT_{name}, {x}, {y}, 0"
            ));
        }
        self.0.push(format!("anim_wait {frames}"));
    }
    fn sound(&mut self, id: &str) {
        self.0.push(format!("anim_sound 0, 1, {id}"));
    }
    fn hit(&mut self, frames: u8) {
        self.0.push("anim_clearobjs".to_string());
        self.sound("SFX_COMET_PUNCH");
        self.0
            .push("anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 133, 55, 0".to_string());
        self.0.push(format!("anim_wait {frames}"));
    }
    fn finish(mut self) -> Vec<String> {
        self.0.extend([
            "anim_clearobjs".to_string(),
            "anim_wait 7".to_string(),
            "anim_ret".to_string(),
        ]);
        self.0
    }
}
