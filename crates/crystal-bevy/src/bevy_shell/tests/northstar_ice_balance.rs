//! Intended-level fixtures; every turn still uses production battle input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{finish, load, press};

const WON: &str = "EVENT_MN_ICE_ENFORCER_DEFEATED";

fn setup(
    r: &CrystalRuntime,
    root: &crystal_assets::AssetRoot,
    starter: &str,
    mixed: bool,
) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnMinneapolisDispatchOffice".into(),
            tile_x: 9,
            tile_y: 9,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit intended-level fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let mut party = vec![(starter, 23)];
    if mixed {
        // Include an ordinary Fire/Rock answer to road slush. Augrub alongside
        // the bird and Grass starter left three members weak to the same Ice
        // coverage, which is not a prepared mixed party.
        party.extend([("GILLPUP", 22), ("CINDORE", 22), ("NOTCHBILL", 22)]);
    }
    let state = c.shell.shell.session_mut().state_mut();
    for (species, level) in party {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    // Midrange DVs and no trained stat experience. Default DVs
                    // are all zero, an extreme rather than an ordinary party.
                    .create_pokemon(species, level, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("actual regional fixture partner")
            )
        );
    }
    state.sync_party_from_storage();
    state.badges.johto = [true, true, false, false, false, false, false, false];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..2] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("prior badge setup");
    }
    state.last_spawn_map_constant = Some(
        r.data()
            .maps
            .values()
            .find(|m| m.id.contains("Minneapolis") && m.id.ends_with("Pokecenter1F"))
            .expect("ordinary blackout clinic")
            .attributes
            .map_constant
            .clone()
            .expect("clinic constant"),
    );
    if mixed {
        assert!(
            state
                .bag
                .add_item(&r.data().items["SUPER_POTION"], 2)
                .expect("two ordinary shop supplies")
        );
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("settled fixture");
    // No party, move, HP, RNG, flag or result writes beyond setup.
    c
}

#[test]
#[ignore = "requires the published regional faction teams; native intended-level balance course"]
fn ice_enforcer_mixed_regional_parties_can_win_at_intended_levels() {
    let (root, r) = load();
    for starter in ["SLIPPERINA", "RIMEPELT", "PORTABEAU"] {
        let mut c = setup(&r, &root, starter, true);
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        assert!(
            super::rochester_earned::try_battle(&mut c, &r, "MN_ICE_ENFORCER", WON),
            "{starter}: a prepared four-member level-22/23 party must have a fair win"
        );
        finish(&mut c);
        let after = c.snapshot().expect("actual balance outcome");
        assert!(
            after
                .party
                .slots
                .iter()
                .any(|slot| slot.pokemon.hp < slot.pokemon.max_hp),
            "the themed boss must put real pressure on {starter}'s party"
        );
        eprintln!(
            "ICE balance mixed {starter}: {:?}",
            after
                .party
                .slots
                .iter()
                .map(|slot| (
                    &slot.pokemon.species.id,
                    slot.pokemon.hp,
                    slot.pokemon.max_hp
                ))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
#[ignore = "requires the published regional faction teams; native solo sweep challenge"]
fn ice_enforcer_does_not_allow_a_healthy_single_starter_sweep() {
    let (root, r) = load();
    for starter in ["SLIPPERINA", "RIMEPELT", "PORTABEAU"] {
        let mut c = setup(&r, &root, starter, false);
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        let won = super::rochester_earned::try_battle(&mut c, &r, "MN_ICE_ENFORCER", WON);
        let after = c.snapshot().expect("actual sweep challenge");
        let partner = &after.party.slots[0].pokemon;
        assert!(
            !won || partner.hp * 2 < partner.max_hp,
            "{starter} rolled the whole ICE boss with more than half its HP"
        );
        eprintln!(
            "ICE balance solo {starter}: won={won}, HP={}/{}",
            partner.hp, partner.max_hp
        );
    }
}
