//! Explicit position/inventory/badge fixtures; all travel and shop dialogue
//! after setup use the production controller. These do not earn regional access.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::systems::field_moves::{CANOE_ACTIVE_MEMORY, CANOE_ITEM};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::interior_life::{collision, fixture, talk};
use super::r10_gallery::{continue_save, press};

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_CANOE_TEST_PACK").expect("published canoe pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified canoe pack"),
            )
            .expect("canoe runtime");
            (root, runtime)
        })
        .clone()
}

fn quantity(c: &VisibleShellController, runtime: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[CANOE_ITEM])
}

#[test]
#[ignore = "requires CRYSTAL_CANOE_TEST_PACK"]
fn canoe_controller_shop_badge_gift_dialogue_repeat_and_continue() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnStPaulAnglerHouse", (5, 4));
    let before = talk(&mut c, (5, 3));
    assert!(before.contains("CAPITOL"));
    assert_eq!(quantity(&c, &runtime), 0);
    // Local earned-badge fixture. The actual delivery still runs native scripts.
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("Capitol Badge fixture");
    let text = talk(&mut c, (5, 3));
    assert!(text.contains("portage"), "{text}");
    assert_eq!(quantity(&c, &runtime), 1);
    talk(&mut c, (5, 3));
    assert_eq!(quantity(&c, &runtime), 1);
    let mut c = capture(c, "shop-native-v01.png");
    let mut loaded = continue_save(root, runtime.clone(), &mut c);
    talk(&mut loaded, (5, 3));
    assert_eq!(quantity(&loaded, &runtime), 1);
    super::interior_life::walk(&mut loaded, &runtime, (11, 6));
    let display_text = talk(&mut loaded, (11, 5));
    assert!(display_text.contains("cedar"), "{display_text}");
    capture(loaded, "shop-display-native-v01.png");
}

#[test]
#[ignore = "requires CRYSTAL_CANOE_TEST_PACK"]
fn canoe_controller_automatic_launch_portage_and_water_continue() {
    let (root, runtime) = load();
    let map = &runtime.data().maps["MnStPaul"];
    let shore = (2..i16::try_from(map.attributes.height * 2).expect("height") - 2)
        .flat_map(|y| {
            (2..i16::try_from(map.attributes.width * 2).expect("width") - 2).map(move |x| (x, y))
        })
        .find(|&(x, y)| {
            collision(&runtime, map, (x, y)) == Some("FLOOR")
                && collision(&runtime, map, (x + 1, y)) == Some("WATER")
                && map.objects.iter().all(|o| {
                    (i32::from(o.x) - i32::from(x)).abs() + (i32::from(o.y) - i32::from(y)).abs()
                        > 8
                })
                && map
                    .events
                    .coord_events
                    .iter()
                    .all(|e| e.x != (x + 1) as u16 || e.y != y as u16)
        })
        .expect("real St Paul open east-facing shoreline");
    let mut c = fixture(&root, &runtime, "MnStPaul", shore);
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::Right);
    assert_eq!(c.snapshot().expect("no canoe").overworld.tile.x, shore.0);
    let item = &runtime.data().items[CANOE_ITEM];
    assert!(
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(item, 1)
            .expect("owned canoe fixture")
    );
    let party = c.snapshot().expect("before paddling").party;
    press(&mut c, GameButton::Right);
    let launched = c.snapshot().expect("launched");
    assert_eq!(
        (launched.overworld.tile.x, launched.overworld.tile.y),
        (shore.0 + 1, shore.1)
    );
    assert_eq!(launched.overworld.mode, MovementMode::Surf);
    assert_eq!(
        launched
            .script_events
            .memory
            .get(CANOE_ACTIVE_MEMORY)
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(launched.party, party);
    let mut c = capture(c, "canoe-native-v01.png");
    let mut c = continue_save(root, runtime.clone(), &mut c);
    assert_eq!(
        c.snapshot()
            .expect("continued canoe")
            .script_events
            .memory
            .get(CANOE_ACTIVE_MEMORY)
            .map(String::as_str),
        Some("1")
    );
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::Left);
    let landed = c.snapshot().expect("portage landing");
    assert_eq!((landed.overworld.tile.x, landed.overworld.tile.y), shore);
    assert_eq!(landed.overworld.mode, MovementMode::Normal);
    assert!(
        !landed
            .script_events
            .memory
            .contains_key(CANOE_ACTIVE_MEMORY)
    );
    assert_eq!(quantity(&c, &runtime), 1);
}

// Compose the production 2D Sprite/Image outputs with the existing overworld audit
// rasterizer. MinimalPlugins opens no window and requires no GPU adapter.
fn capture(mut c: VisibleShellController, name: &str) -> VisibleShellController {
    use bevy::prelude::{Assets, Image, Mut};
    let Some(directory) = std::env::var_os("CRYSTAL_CANOE_RENDER_DIR") else {
        return c;
    };
    c.wait_frames(16).expect("capture settlement");
    let snapshot = c.snapshot().expect("capture position");
    println!(
        "{name}: {} at {:?}, facing {:?}",
        snapshot.overworld.map_name, snapshot.overworld.tile, snapshot.overworld.facing
    );
    let mut app = super::menu_render_test_app(c.shell);
    app.update();
    app.update();
    let world = app.world_mut();
    assert!(
        world
            .resource::<super::super::BevyRuntimeShell>()
            .last_error
            .is_none()
    );
    let image = world.resource_scope(|world, images: Mut<Assets<Image>>| {
        super::pokemon_overworld_context_canvas(world, &images)
    });
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).expect("render directory");
    let path = directory.join(name);
    assert!(!path.exists(), "retain earlier canoe renders");
    image.save(path).expect("native sprite capture");
    VisibleShellController {
        shell: world
            .remove_resource::<super::super::BevyRuntimeShell>()
            .expect("rendered session"),
    }
}
