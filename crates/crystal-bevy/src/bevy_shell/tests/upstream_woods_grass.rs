//! Explicit local fixtures, then native walking and westward hop inputs only.
use super::*;
const BANK: &str = "MnMississippiUpstreamWoodsC01R00";

#[test]
#[ignore = "requires reviewed upper river pack and retained-save directory"]
fn upstream_woods_grass_native_both_grassy_eastbound_loops_and_dry_west_hops() {
    for y in [18, 19] {
        let (_, r, mut c) = fixture(BANK, (7, y), true, true, false);
        assert_eq!(terrain_path(&r, &mut c, (10, y), false, true), 3);
        step(&mut c, GameButton::Left, BANK, (9, y), "dry west-ledge lip");
        step(
            &mut c,
            GameButton::Left,
            BANK,
            (7, y),
            "native dry westward return",
        );
        for _ in 0..3 {
            c.press(GameButton::Right).expect("native reverse attempt");
            c.wait_frames(16).expect("settled reverse");
            settle(&mut c);
            assert_at(&mut c, BANK, (7, y), "native reverse remains blocked");
        }
        terrain_path(&r, &mut c, (16, 19), false, true);
        save(&mut c, &format!("upstream-woods-lane-{y}"));
    }
}
