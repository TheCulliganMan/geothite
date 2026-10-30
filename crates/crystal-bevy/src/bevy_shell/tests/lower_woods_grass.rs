//! Explicit local fixture, followed exclusively by production walking inputs.
use super::*;
const BANK: &str = "MnMississippiLowerWoodsC01R01";

#[test]
#[ignore = "requires reviewed lower river pack and retained-save directory"]
fn lower_woods_grass_native_grassy_climb_dry_hop_and_reverse_denial() {
    let (_, r, mut c) = fixture(BANK, (27, 32), true, true, false);
    assert_eq!(terrain_path(&r, &mut c, (27, 29), false, true), 3);
    step(
        &mut c,
        GameButton::Down,
        BANK,
        (27, 30),
        "dry pine-bank lip",
    );
    step(
        &mut c,
        GameButton::Down,
        BANK,
        (27, 32),
        "native dry bank return",
    );
    for _ in 0..3 {
        c.press(GameButton::Up).expect("native reverse attempt");
        c.wait_frames(16).expect("settled reverse");
        settle(&mut c);
        assert_at(&mut c, BANK, (27, 32), "native reverse remains blocked");
    }
    terrain_path(&r, &mut c, (32, 33), false, true);
    save(&mut c, "lower-woods-pine-bank");
}
