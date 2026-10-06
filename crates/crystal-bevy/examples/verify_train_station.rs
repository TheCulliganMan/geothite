//! Public-API, headless train verification. Fixture saves are prepared before
//! controls start; every gameplay action then uses VisibleShellController.
use anyhow::{Context, Result, bail, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_core::{
    input::GameButton,
    systems::map_context::{SpawnMemoryUpdate, commit_overworld_snapshot},
    world::map::{Direction, TilePosition},
};
use crystal_runtime::{CrystalRuntime, RuntimeGameShell, RuntimeShellPhase, RuntimeShellSnapshot};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

const STATIONS: [&str; 2] = ["GoldenrodMagnetTrainStation", "SaffronMagnetTrainStation"];

// FIXTURE ONLY. This lower-level shell is never ticked or used to play a trip.
fn fixture_save(
    runtime: &CrystalRuntime,
    root: &AssetRoot,
    path: &Path,
    map: &str,
    tile: (i16, i16),
    powered: bool,
    pass: bool,
) -> Result<()> {
    let mut seed = RuntimeGameShell::new_game_at_runtime_tile(
        root.clone(),
        runtime.clone(),
        runtime.title_new_game_spawn_identifier()?,
        map,
        tile.0,
        tile.1,
    )?;
    let player_id = seed.snapshot()?.trainer.player_id;
    seed.set_trainer_identity("CHRIS", player_id)?;
    let pass_definition = runtime
        .data()
        .items
        .get("PASS")
        .context("authentic PASS item")?
        .clone();
    {
        let (state, overworld) = seed.session_mut().state_and_overworld_mut();
        state
            .flags
            .set_event_flag("EVENT_RESTORED_POWER_TO_KANTO", powered)?;
        if pass {
            ensure!(
                state
                    .bag
                    .add_item(&pass_definition, 1)
                    .map_err(anyhow::Error::msg)?,
                "fixture PASS insertion"
            );
        }
        overworld.player.facing = Direction::Up;
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
    }
    seed.save(path)?;
    let saved = runtime.load_save(path)?;
    ensure!(
        saved
            .flags
            .is_event_flag_set("EVENT_RESTORED_POWER_TO_KANTO")?
            == powered,
        "fixture flag verification"
    );
    ensure!(
        saved.bag.quantity(&pass_definition) == u16::from(pass),
        "fixture PASS verification"
    );
    Ok(())
}
fn observe(
    c: &mut VisibleShellController,
    events: &mut Vec<Value>,
    action: &str,
) -> Result<RuntimeShellSnapshot> {
    let s = c.snapshot()?;
    events.push(json!({"action":action,"frame":s.overworld.frame,"map":s.overworld.map_name,
        "tile":[s.overworld.tile.x,s.overworld.tile.y],"facing":format!("{:?}",s.overworld.facing),
        "phase":format!("{:?}",s.phase),"text_label":s.ui.text.as_ref().map(|t|&t.label),
        "text":s.ui.text.as_ref().and_then(|t|t.asm_text.as_deref()),"yes_no":s.ui.pending_yes_no.is_some(),
        "yes_no_identity":s.ui.pending_yes_no.as_ref().map(|p|json!({"source_script":p.source_script,"command_index":p.command_index})),
        "special":s.script_events.last_special_routine,"pending_script":c.has_pending_script_work(),
        "officer_positions":s.visible_object_runtime_tiles.iter().filter(|(id,_)|id.ends_with("_OFFICER")).map(|(id,p)|json!({"id":id,"tile":[p.x,p.y]})).collect::<Vec<_>>() }));
    Ok(s)
}
fn press(
    c: &mut VisibleShellController,
    events: &mut Vec<Value>,
    button: GameButton,
) -> Result<RuntimeShellSnapshot> {
    c.press(button).with_context(|| {
        format!(
            "controller {button:?}; last observation {:?}",
            events.last()
        )
    })?;
    observe(c, events, &format!("press {button:?}"))
}
fn idle(c: &VisibleShellController, s: &RuntimeShellSnapshot) -> bool {
    s.phase == RuntimeShellPhase::Overworld
        && s.ui.text.is_none()
        && s.ui.menu.is_none()
        && s.ui.pending_yes_no.is_none()
        && !c.has_pending_script_work()
}
fn tile(s: &RuntimeShellSnapshot, map: &str, x: i16, y: i16) -> Result<()> {
    ensure!(
        s.overworld.map_name == map && s.overworld.tile == TilePosition::new(x, y),
        "expected {map} ({x},{y}), got {} {:?}",
        s.overworld.map_name,
        s.overworld.tile
    );
    Ok(())
}
fn label_seen(events: &[Value], label: &str) -> bool {
    events
        .iter()
        .any(|e| e["text_label"].as_str() == Some(label))
}
fn move_one(
    c: &mut VisibleShellController,
    events: &mut Vec<Value>,
    button: GameButton,
    map: &str,
    x: i16,
    y: i16,
) -> Result<()> {
    for _ in 0..4 {
        let s = press(c, events, button)?;
        if s.overworld.map_name == map && s.overworld.tile == TilePosition::new(x, y) {
            return Ok(());
        }
    }
    bail!(
        "controller approach blocked: expected {map} ({x},{y}) after {button:?}; {:?}",
        events.last()
    )
}
fn open_officer(c: &mut VisibleShellController, events: &mut Vec<Value>, map: &str) -> Result<()> {
    let s = observe(c, events, "start officer interaction")?;
    tile(&s, map, 9, 10)?;
    ensure!(
        s.overworld.facing == Direction::Up,
        "fixture facing changed on load"
    );
    let s = press(c, events, GameButton::A)?;
    ensure!(
        s.ui.text.is_some() || s.ui.pending_yes_no.is_some(),
        "A did not start officer dialogue"
    );
    // Dialogue owns movement even when reached through the renderer-neutral API.
    let position = s.overworld.tile;
    let locked = press(c, events, GameButton::Left)?;
    ensure!(
        locked.overworld.tile == position && locked.overworld.map_name == map,
        "dialogue leaked movement input"
    );
    Ok(())
}
fn denial(
    c: &mut VisibleShellController,
    events: &mut Vec<Value>,
    map: &str,
    powered: bool,
) -> Result<()> {
    open_officer(c, events, map)?;
    let expected = if powered {
        format!(
            "{map}Officer{}",
            if map == STATIONS[0] {
                "YouDontHaveARailPassText"
            } else {
                "YouDontHaveAPassText"
            }
        )
    } else {
        format!(
            "{map}Officer{}",
            if map == STATIONS[0] {
                "TheTrainHasntComeInText"
            } else {
                "TrainIsntOperatingText"
            }
        )
    };
    // The same core prompt remains pending while presentation advances its
    // authored pages. Count its source identity once, not every observation.
    let mut prompts = BTreeSet::new();
    for _ in 0..64 {
        let s = observe(c, events, "denial boundary")?;
        if idle(c, &s) {
            tile(&s, map, 9, 10)?;
            ensure!(
                label_seen(events, &expected),
                "missing authentic denial {expected}"
            );
            ensure!(
                prompts.len() == usize::from(powered),
                "wrong power/PASS prompt ordering"
            );
            ensure!(
                !events
                    .iter()
                    .any(|e| e["special"].as_str() == Some("MagnetTrain")),
                "denial launched train special"
            );
            return Ok(());
        }
        if let Some(prompt) = s.ui.pending_yes_no.as_ref() {
            prompts.insert((prompt.source_script.clone(), prompt.command_index));
        }
        press(c, events, GameButton::A)?; // answer the real default YES prompt
    }
    bail!(
        "denial did not release input within 64 actions: {:?}",
        events.last()
    )
}
fn journey(
    c: &mut VisibleShellController,
    events: &mut Vec<Value>,
    from: &str,
    to: &str,
) -> Result<()> {
    let begin = events.len();
    open_officer(c, events, from)?;
    // The same core prompt remains pending while presentation advances its
    // authored pages. Count its source identity once, not every observation.
    let mut prompts = BTreeSet::new();
    let mut exited_door = false;
    let arrival_label = format!(
        "{to}OfficerArrivedIn{}Text",
        if to == STATIONS[0] {
            "Goldenrod"
        } else {
            "Saffron"
        }
    );
    for _ in 0..128 {
        let s = observe(c, events, "journey boundary")?;
        if s.overworld.map_name == to && idle(c, &s) {
            if s.overworld.tile == TilePosition::new(11, 5) {
                ensure!(
                    !exited_door,
                    "train repeatedly returned to the arrival door"
                );
                press(c, events, GameButton::Down)?;
                exited_door = true;
                continue;
            }
            tile(&s, to, 9, 10)?;
            ensure!(prompts.len() == 1, "expected one real boarding YES prompt");
            ensure!(
                label_seen(&events[begin..], &format!("{from}OfficerRightThisWayText")),
                "missing PASS-approved boarding text"
            );
            ensure!(
                label_seen(&events[begin..], &arrival_label),
                "arrival coordinate event and dialogue did not complete"
            );
            ensure!(
                events[begin..]
                    .iter()
                    .any(|e| e["special"].as_str() == Some("MagnetTrain")),
                "MagnetTrain special was never observed"
            );
            let officer = format!("{}_OFFICER", to.to_ascii_uppercase());
            ensure!(
                s.visible_object_runtime_tiles.get(&officer) == Some(&TilePosition::new(9, 9)),
                "arrival officer did not return to the original gate"
            );
            ensure!(
                s.bag
                    .key_items
                    .iter()
                    .any(|i| i.item_id == "PASS" && i.quantity == 1),
                "trip consumed or duplicated PASS"
            );
            // A normal step after the arrival dialogue proves the controller
            // released all script/animation ownership, then returns by input.
            move_one(c, events, GameButton::Down, to, 9, 11)?;
            move_one(c, events, GameButton::Up, to, 9, 10)?;
            return Ok(());
        }
        if let Some(prompt) = s.ui.pending_yes_no.as_ref() {
            prompts.insert((prompt.source_script.clone(), prompt.command_index));
            press(c, events, GameButton::A)?;
        } else if s.ui.text.is_some() {
            press(c, events, GameButton::A)?;
        } else {
            c.wait_frames(1)?;
            observe(c, events, "wait one controller frame")?;
        }
    }
    bail!(
        "journey {from} -> {to} did not finish within 128 bounded controller actions: {:?}",
        events.last()
    )
}
fn west_approach(c: &mut VisibleShellController, events: &mut Vec<Value>, map: &str) -> Result<()> {
    // Independent platform fixture. Do not step into the warp or treat this
    // fixture as travel; successful arrival clears the east door in journey.
    tile(&observe(c, events, "platform fixture")?, map, 9, 7)?;
    for x in (6..9).rev() {
        move_one(c, events, GameButton::Left, map, x, 7)?;
    }
    move_one(c, events, GameButton::Up, map, 6, 6)?;
    move_one(c, events, GameButton::Down, map, 6, 7)?;
    for x in 7..=9 {
        move_one(c, events, GameButton::Right, map, x, 7)?;
    }
    Ok(())
}

// Real production mesh path, including all other station models. The 3D art is
// not collision authority; both mesh aperture and controller movement matter.
fn mesh_clearance(runtime: &CrystalRuntime, map: &str) -> Result<Value> {
    use bevy::{
        prelude::{Assets, Handle, Image, UVec2, Vec2},
        render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        },
    };
    use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldFrame};
    let m = &runtime.data().maps[map];
    let ts = &m.attributes.tileset_name;
    let w = usize::from(m.attributes.width) * 4;
    let h = usize::from(m.attributes.height) * 4;
    let layout = runtime
        .runtime_file(&format!("data/tilesets/{ts}_metatiles.bin"))
        .context("packed metatiles")?;
    let sheet = image::load_from_memory(
        runtime
            .runtime_file(&format!("gfx/tilesets/{ts}.png"))
            .context("packed atlas")?,
    )?
    .to_rgba8();
    let mut images = Assets::<Image>::default();
    let mut handles = Vec::<Handle<Image>>::new();
    for y in (0..sheet.height()).step_by(8) {
        for x in (0..sheet.width()).step_by(8) {
            let crop = image::imageops::crop_imm(&sheet, x, y, 8, 8).to_image();
            handles.push(images.add(Image::new(
                Extent3d {
                    width: 8,
                    height: 8,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                crop.into_raw(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD,
            )));
        }
    }
    let mut tiles = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let block = m.blocks[(y / 4) * (w / 4) + x / 4];
            let index = u16::from(layout[usize::from(block) * 16 + (y % 4) * 4 + x % 4]);
            let palette = runtime.data().tilesets[ts]
                .palette_map
                .get(usize::from(index))
                .copied()
                .unwrap_or(0);
            let sample = crystal_bevy::bevy_shell::resolve_tileset_tile_index(
                handles.len(),
                usize::from(index),
                (palette >> 3) & 1,
            );
            tiles.push(VisualTile {
                column: x as u32,
                row: y as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from(ts.as_str()),
                    metatile_id: block,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: index,
                },
                texture: handles[sample].clone(),
                animation_frames: None,
                priority: false,
            });
        }
    }
    let frame = VisualWorldFrame {
        active: true,
        map_id: Arc::from(map),
        map_texture: handles[0].clone(),
        tile_size: Vec2::splat(8.),
        grid_size: UVec2::new(w as u32, h as u32),
        viewport_size: Vec2::new(w as f32 * 8., h as f32 * 8.),
        tiles,
        ..Default::default()
    };
    let mesh = crystal_voxel_view::audit_terrain_mesh_with_images(&frame, &images)
        .map_err(|e| anyhow::anyhow!("production station mesh: {e:?}"))?;
    ensure!(
        mesh.authored_cells
            .iter()
            .filter(|label| **label == Some("train:stationary-body"))
            .count()
            == 60,
        "stationary train body is absent or partially bound"
    );
    for x in [6usize, 11] {
        for cy in 10..12 {
            for cx in 2 * x..2 * x + 2 {
                let i = cy * w + cx;
                ensure!(
                    mesh.authored_cells[i].is_none(),
                    "live native door cell was swallowed"
                );
                ensure!(mesh.footing_heights[i] == 0., "door footing changed");
            }
        }
        let left = x as f32 * 16. - w as f32 * 4.;
        let right = left + 16.;
        let north = 80. - h as f32 * 4.;
        let south = north + 16.;
        for (name, surface) in [
            ("solid", &mesh.solid),
            ("animated solid", &mesh.animated_solid),
            ("textured", &mesh.textured),
            ("animated textured", &mesh.animated_textured),
        ] {
            for triangle in surface.indices.chunks_exact(3) {
                let p = triangle
                    .iter()
                    .map(|&i| surface.positions[i as usize])
                    .collect::<Vec<_>>();
                ensure!(
                    p.iter().all(|p| p[0] <= left + 0.001)
                        || p.iter().all(|p| p[0] >= right - 0.001)
                        || p.iter().all(|p| p[2] <= north + 0.001)
                        || p.iter().all(|p| p[2] >= south - 0.001)
                        || p.iter().all(|p| p[1] <= 0.001),
                    "{name} triangle obstructs real boarding aperture ({x},5): {p:?}"
                );
            }
        }
    }
    Ok(
        json!({"map":map,"authored_body_cells":60,"live_door_cells":8,"apertures":[[6,5],[11,5]],"zero_footing":true,"production_mesh_clear":true}),
    )
}
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: verify_train_station PACK OUTPUT_DIRECTORY")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("output directory required")?);
    ensure!(args.next().is_none(), "unexpected arguments");
    std::fs::create_dir_all(&output)?;
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack)?,
    )?;
    let mut results = Vec::new();
    let mut failures = 0;
    for map in STATIONS {
        let result = mesh_clearance(&runtime, map);
        if result.is_err() {
            failures += 1;
        }
        results.push(match result{Ok(detail)=>json!({"case":"production_mesh_clearance","map":map,"pass":true,"detail":detail}),Err(e)=>json!({"case":"production_mesh_clearance","map":map,"pass":false,"error":format!("{e:#}")})});
        for (kind, powered, pass, start) in [
            ("power_denial", false, true, (9, 10)),
            ("pass_denial", true, false, (9, 10)),
            ("round_trip", true, true, (9, 10)),
            ("west_door_approach", true, true, (9, 7)),
        ] {
            let path = output.join(format!("fixture-{map}-{kind}.crystalsave"));
            let mut events = Vec::new();
            let run = (|| -> Result<()> {
                fixture_save(&runtime, &root, &path, map, start, powered, pass)?;
                let mut controller = VisibleShellController::load_save(
                    root.clone(),
                    runtime.clone(),
                    path.clone(),
                    None,
                )?;
                controller.set_runtime_journal_enabled(false);
                observe(
                    &mut controller,
                    &mut events,
                    "loaded authentic fixture save",
                )?;
                match kind {
                    "power_denial" | "pass_denial" => {
                        denial(&mut controller, &mut events, map, powered)?
                    }
                    "west_door_approach" => west_approach(&mut controller, &mut events, map)?,
                    _ => {
                        let other = if map == STATIONS[0] {
                            STATIONS[1]
                        } else {
                            STATIONS[0]
                        };
                        journey(&mut controller, &mut events, map, other)?;
                        journey(&mut controller, &mut events, other, map)?;
                    }
                }
                controller.save(output.join(format!("result-{map}-{kind}.crystalsave")))?;
                Ok(())
            })();
            if run.is_err() {
                failures += 1;
            }
            let result = json!({"case":kind,"map":map,"fixture_only":{"tile":[start.0,start.1],"power_restored":powered,"pass":pass},"actions_use":"VisibleShellController::press / wait_frames","pass":run.is_ok(),"error":run.err().map(|e|format!("{e:#}")),"observations":events});
            eprintln!(
                "{} {map} {kind}",
                if result["pass"].as_bool() == Some(true) {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            results.push(result);
            std::fs::write(
                output.join("train-controller-report.json"),
                serde_json::to_vec_pretty(
                    &json!({"pack":pack,"results":results,"failures":failures}),
                )?,
            )?;
        }
    }
    ensure!(
        failures == 0,
        "{failures} station verification cases failed; see {}",
        output.join("train-controller-report.json").display()
    );
    println!(
        "PASS both stations: denial ordering, genuine reciprocal controller travel/arrival, both mesh apertures and native footing, ordinary west-door approaches; native image/frame review remains separate"
    );
    Ok(())
}
