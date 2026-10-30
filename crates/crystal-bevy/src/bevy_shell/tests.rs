include!("tests/shell_basics.rs");
#[path = "tests/dialogue_clock_presentation.rs"]
mod dialogue_clock_presentation;
#[path = "tests/dialogue_native_transitions.rs"]
mod dialogue_native_transitions;
#[path = "tests/dialogue_overhaul.rs"]
mod dialogue_overhaul;
#[path = "tests/dialogue_save_prompt.rs"]
mod dialogue_save_prompt;
#[path = "tests/dialogue_scene_fixtures.rs"]
mod dialogue_scene_fixtures;
#[path = "tests/grand_marais_partner_retry.rs"]
mod grand_marais_partner_retry;
#[path = "tests/minnesota_castorock.rs"]
mod minnesota_castorock;
#[path = "tests/minnesota_fossils.rs"]
mod minnesota_fossils;
#[path = "tests/minnesota_starter_return.rs"]
mod minnesota_starter_return;
#[path = "tests/native_city_ports.rs"]
mod native_city_ports;
#[path = "tests/northstar_dispatch_course.rs"]
mod northstar_dispatch_course;
#[cfg(feature = "location-tester")]
include!("tests/party_follower.rs");
include!("tests/multiplayer_rendering.rs");
include!("tests/overworld_rendering.rs");
include!("tests/pokemon_overworld_context.rs");
include!("tests/pokemon_overworld_batch.rs");
include!("tests/pokemon_party_sprites.rs");
include!("tests/character_source.rs");
include!("tests/tile_animation_provenance.rs");
include!("tests/intro_title_rendering.rs");
include!("tests/title_flow.rs");
include!("tests/runtime_surfaces.rs");
include!("tests/new_game_flow.rs");
include!("tests/integrated_menus.rs");
include!("tests/audio_and_battle_ui.rs");
include!("tests/menu_and_input.rs");
include!("tests/story_progression.rs");
include!("tests/quests.rs");
include!("tests/art_text_and_determinism.rs");
include!("tests/battle_render_regressions.rs");
include!("tests/battle_origin.rs");
include!("tests/encounter_anchor.rs");
include!("tests/battle_origin_fishing.rs");
include!("tests/fishing_anchor.rs");
include!("tests/walking_anchor.rs");
include!("tests/continuous_encounter_witness.rs");
include!("tests/kanto_interior_states.rs");
include!("tests/surf_anchor.rs");
include!("tests/walking_encounter_eligibility.rs");
include!("tests/trainer_anchor.rs");
include!("tests/scripted_trainer_anchor.rs");
include!("tests/fishing_presentation.rs");
include!("tests/battle_timeline_clock.rs");
include!("tests/battle_capture_palettes.rs");
include!("tests/battle_oam_layers.rs");
include!("tests/minnesota_signature_animations.rs");
include!("tests/northwoods_signature_controller.rs");
include!("tests/northwoods_starter_signatures.rs");
include!("tests/northwoods_fairy_controller.rs");
include!("tests/northwoods_starter_battle_backs.rs");
include!("tests/northwoods_sticky_progression.rs");
include!("tests/battle_sliding_intro.rs");
include!("tests/shop_rendering.rs");
#[path = "tests/shop_tm_sale.rs"]
mod shop_tm_sale;
#[path = "tests/minnesota_tmhm.rs"]
mod minnesota_tmhm;
include!("tests/all_pokemon.rs");
include!("tests/trainer_rematches.rs");
include!("tests/heal_machine_rendering.rs");
include!("tests/town_map_rendering.rs");

#[test]
fn script_earthquake_shakes_then_sleeps_for_both_low_six_bit_counters() {
    let mut earthquake = super::VisibleEarthquake::from_script(84, 20, 20);
    assert_eq!(earthquake.intensity, 2);
    assert_eq!(earthquake.frames_remaining, 40);
    assert_eq!(earthquake.shake_frames_remaining, 20);

    earthquake.advance(20);
    assert_eq!(earthquake.frames_remaining, 20);
    assert_eq!(earthquake.shake_frames_remaining, 0);

    earthquake.advance(20);
    assert_eq!(earthquake.frames_remaining, 0);

    let wrapped = super::VisibleEarthquake::from_script(0, 256, 256);
    assert_eq!(wrapped.frames_remaining, 512);
    assert_eq!(wrapped.shake_frames_remaining, 256);
}

#[test]
fn overworld_screen_shake_uses_the_vertical_source_counter_sequence() {
    let mut earthquake = super::VisibleEarthquake::from_script(84, 20, 20);

    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, 8.0),
        "remaining counter 19 applies -2 to SCY, which projects to +8 in Bevy's upward Y axis"
    );
    earthquake.advance(1);
    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, -8.0),
        "remaining counter 18 applies +2 to SCY"
    );
    earthquake.advance(18);
    assert_eq!(earthquake.shake_frames_remaining, 1);
    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, 0.0),
        "StepFunction_ScreenShake restores the baseline before deleting on its final update"
    );
    earthquake.advance(1);
    assert_eq!(earthquake.shake_frames_remaining, 0);
    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, 0.0),
        "the generated step_sleep phase must not continue shaking"
    );
}

#[test]
fn poison_step_uses_four_opaque_bg_palette_frames_then_one_restored_frame() {
    let poison = [230.0 / 255.0, 173.0 / 255.0, 1.0, 1.0];
    for frames_remaining in [5, 4, 3, 2] {
        assert_eq!(
            super::visible_poison_bg_palette_rgba(frames_remaining),
            Some(poison),
            "LoadPoisonBGPals keeps every CGB background color at the poison color"
        );
    }
    assert_eq!(
        super::visible_poison_bg_palette_rgba(1),
        None,
        "the trailing DelayFrame runs after _UpdateTimePals restores the map palette"
    );
    assert_eq!(super::visible_poison_bg_palette_rgba(0), None);
}

#[test]
fn egg_hatch_wobble_uses_exact_asm_pairs_and_crack_boundaries() {
    assert_eq!(visible_egg_wobble_x(0), -2);
    assert_eq!(visible_egg_wobble_x(2), -2);
    assert_eq!(visible_egg_wobble_x(3), 2);
    assert_eq!(visible_egg_wobble_x(5), 2);
    assert_eq!(visible_egg_wobble_x(6), 0);
    assert_eq!(visible_egg_wobble_x(21), 0);
    assert_eq!(visible_egg_wobble_x(22), -2);
    assert_eq!(visible_egg_wobble_x(343), 0);
    assert_eq!(
        (0..344)
            .filter(|frame| visible_egg_crack_at(*frame))
            .collect::<Vec<_>>(),
        vec![50, 124, 222]
    );
}

include!("tests/webmcp.rs");

// Optional visual-reference files are read only by the tests that use them.
// Their absence must not prevent unrelated gameplay regressions from compiling.
fn external_oracle_fixture_bytes(name: &str) -> Vec<u8> {
    let root = std::env::var_os("GEOTHITE_ORACLE_FIXTURES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tools/asm-oracle/fixtures")
        });
    let path = root.join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("Visual reference {} unavailable ({error}); set GEOTHITE_ORACLE_FIXTURES to the reference directory", path.display()))
}
fn external_oracle_fixture_text(name: &str) -> String {
    String::from_utf8(external_oracle_fixture_bytes(name)).expect("UTF-8 visual reference")
}

include!("tests/runtime_frontend_smoke.rs");
#[path = "tests/controller_snapshots.rs"]
mod controller_snapshots;
#[path = "tests/grand_marais_rewards.rs"]
mod grand_marais_rewards;
#[path = "tests/north_shore.rs"]
mod north_shore;
#[path = "tests/north_shore_expansion.rs"]
mod north_shore_expansion;
#[path = "tests/minneapolis_falls.rs"]
mod minneapolis_falls;
#[path = "tests/minneapolis_falls_animation_native.rs"]
mod minneapolis_falls_animation_native;
#[cfg(feature = "voxel-view")]
include!("tests/immersive_battle.rs");
#[cfg(feature = "voxel-view")]
include!("tests/queued_source_scene.rs");

#[cfg(feature = "location-tester")]
include!("tests/day_cycle.rs");

#[cfg(all(feature = "location-tester", feature = "voxel-view"))]
include!("tests/native_terrace_regions.rs");

#[cfg(all(feature = "location-tester", feature = "voxel-view"))]
include!("tests/summit_backdrop.rs");

include!("tests/shared_yes_no_layout.rs");

include!("tests/pc_yes_no_timing.rs");

#[path = "tests/gunflint_art.rs"]
mod gunflint_art;

#[path = "tests/authored_route_fixture.rs"]
mod authored_route_fixture;
#[path = "tests/gunflint.rs"]
mod gunflint;
#[path = "tests/compact_gunflint.rs"]
mod compact_gunflint;
#[path = "tests/inland_winter.rs"]
mod inland_winter;

#[path = "tests/league_approach.rs"]
mod league_approach;

#[path = "tests/border_pines.rs"]
mod border_pines;
#[path = "tests/grand_rapids.rs"]
mod grand_rapids;
#[path = "tests/iron_range.rs"]
mod iron_range;
#[path = "tests/iron_range_story.rs"]
mod iron_range_story;
#[path = "tests/iron_range_descent.rs"]
mod iron_range_descent;
#[path = "tests/r10_gallery.rs"]
mod r10_gallery;
#[path = "tests/compact_interior_stories.rs"]
mod compact_interior_stories;
#[path = "tests/interior_life.rs"]
mod interior_life;
#[path = "tests/interior_layouts.rs"]
mod interior_layouts;
#[path = "tests/city_refinement.rs"]
mod city_refinement;
#[path = "tests/detroit_lakes.rs"]
mod detroit_lakes;
#[path = "tests/international_falls.rs"]
mod international_falls;
#[path = "tests/winona_station.rs"]
mod winona_station;
#[path = "tests/minnesota_fishing_rods.rs"]
mod minnesota_fishing_rods;
#[path = "tests/minnesota_hm_progression.rs"]
mod minnesota_hm_progression;
#[path = "tests/shore_gate.rs"]
mod shore_gate;
#[path = "tests/moorhead.rs"]
mod moorhead;
#[path = "tests/cave_quality.rs"]
mod cave_quality;
#[path = "tests/voyageurs_ice.rs"]
mod voyageurs_ice;
#[path = "tests/r12_shelter.rs"]
mod r12_shelter;
#[path = "tests/r13_refuge.rs"]
mod r13_refuge;
#[path = "tests/r20_cleft.rs"]
mod r20_cleft;
#[path = "tests/r09_mine.rs"]
mod r09_mine;
#[path = "tests/r01_hollows.rs"]
mod r01_hollows;
#[path = "tests/r03_bank_cave.rs"]
mod r03_bank_cave;
#[path = "tests/r17_tor_cave.rs"]
mod r17_tor_cave;
#[path = "tests/r05_rockcut.rs"]
mod r05_rockcut;
#[path = "tests/mille_lacs_shore.rs"]
mod mille_lacs_shore;
#[path = "tests/mille_lacs_park.rs"]
mod mille_lacs_park;
#[path = "tests/leech_lake.rs"]
mod leech_lake;
#[path = "tests/northern_lakes.rs"]
mod northern_lakes;
#[path = "tests/pine_country.rs"]
mod pine_country;
#[path = "tests/minnesota_gyms.rs"]
mod minnesota_gyms;
#[path = "tests/boundary_basecamp_story.rs"]
mod boundary_basecamp_story;
#[path = "tests/boundary_basecamp_connections.rs"]
mod boundary_basecamp_connections;
#[path = "tests/prairie_forest_link.rs"]
mod prairie_forest_link;
#[path = "tests/route_quality_r19.rs"]
mod route_quality_r19;
#[path = "tests/route_quality_r08.rs"]
mod route_quality_r08;
#[path = "tests/route_quality_r18.rs"]
mod route_quality_r18;
#[path = "tests/red_river_prairie.rs"]
mod red_river_prairie;
#[path = "tests/voyageurs_border_loop.rs"]
mod voyageurs_border_loop;
#[path = "tests/winter_sections.rs"]
mod winter_sections;

#[path = "tests/central_lakes.rs"]
mod central_lakes;
#[path = "tests/central_lakes_journey.rs"]
mod central_lakes_journey;
#[path = "tests/driftless.rs"]
mod driftless;
#[path = "tests/headwaters.rs"]
mod headwaters;
#[path = "tests/headwaters_art.rs"]
mod headwaters_art;

#[path = "tests/prairie_farm.rs"]
mod prairie_farm;

#[path = "tests/prairie_farm_journey.rs"]
mod prairie_farm_journey;

#[path = "tests/river_valley.rs"]
mod river_valley;

#[path = "tests/river_valley_journey.rs"]
mod river_valley_journey;

#[path = "tests/twin_cities.rs"]
mod twin_cities;

#[path = "tests/twin_cities_journey.rs"]
mod twin_cities_journey;
#[path = "tests/brainerd_city.rs"]
mod brainerd_city;
#[path = "tests/duluth_connections.rs"]
mod duluth_connections;
#[path = "tests/duluth_story.rs"]
mod duluth_story;
#[path = "tests/minneapolis.rs"]
mod minneapolis;
#[path = "tests/northstar_dialogue.rs"]
mod northstar_dialogue;
#[path = "tests/northstar_story_events.rs"]
mod northstar_story_events;
#[path = "tests/northstar_ice_lair.rs"]
mod northstar_ice_lair;
#[path = "tests/northstar_ice_balance.rs"]
mod northstar_ice_balance;
#[path = "tests/northstar_ice_headquarters.rs"]
mod northstar_ice_headquarters;
#[path = "tests/northstar_drill_lair.rs"]
mod northstar_drill_lair;
#[path = "tests/northstar_rival_arc.rs"]
mod northstar_rival_arc;
#[path = "tests/northstar_iron_workings.rs"]
mod northstar_iron_workings;
#[path = "tests/northstar_story_bridges.rs"]
mod northstar_story_bridges;
#[path = "tests/winona.rs"]
mod winona;
#[path = "tests/rochester.rs"]
mod rochester;
#[path = "tests/rochester_earned.rs"]
mod rochester_earned;
#[path = "tests/prairie_story.rs"]
mod prairie_story;
#[path = "tests/courier_story.rs"]
mod courier_story;
#[path = "tests/northstar_earned_gyms.rs"]
mod northstar_earned_gyms;
#[path = "tests/battle_pack_navigation.rs"]
mod battle_pack_navigation;
#[path = "tests/ryans_house.rs"]
mod ryans_house;
#[path = "tests/duluth_city.rs"]
mod duluth_city;
#[path = "tests/bemidji_city.rs"]
mod bemidji_city;
#[path = "tests/prairie_grass_rustle.rs"]
mod prairie_grass_rustle;

#[path = "tests/river_bluffs.rs"]
mod river_bluffs;

#[path = "tests/river_bluffs_journey.rs"]
mod river_bluffs_journey;

#[path = "tests/cut_gated_routes.rs"]
mod cut_gated_routes;
#[path = "tests/root_beer_lady.rs"]
mod root_beer_lady;
#[path = "tests/boundary_waters.rs"]
mod boundary_waters;
#[path = "tests/boundary_safari_expansion.rs"]
mod boundary_safari_expansion;
#[path = "tests/safari_expedition.rs"]
mod safari_expedition;

#[path = "tests/bridge_overlay_rendering.rs"]
mod bridge_overlay_rendering;

#[cfg(feature = "voxel-view")]
#[path = "tests/city_exteriors.rs"]
mod city_exteriors;

#[path = "tests/minnesota_trainers.rs"]
mod minnesota_trainers;
#[path = "tests/original_ranger.rs"]
mod original_ranger;

#[path = "tests/ely_shore_pass.rs"]
mod ely_shore_pass;
#[path = "tests/ely_pine_knolls.rs"]
mod ely_pine_knolls;
#[path = "tests/split_rock_clinic.rs"]
mod split_rock_clinic;
#[path = "tests/staples_station.rs"]
mod staples_station;

#[path = "tests/northstar_ancient_swamp.rs"]
mod northstar_ancient_swamp;

#[path = "tests/northstar_progression_rewards.rs"]
mod northstar_progression_rewards;

#[path = "tests/pokedex_habitats.rs"]
mod pokedex_habitats;

#[path = "tests/minnesota_habitats.rs"]
mod minnesota_habitats;

#[path = "tests/canoe.rs"]
mod canoe;


#[path = "tests/central_lakes_grass.rs"]
mod central_lakes_grass;

#[path = "tests/verdigrim_scene.rs"]
mod verdigrim_scene;

#[path = "tests/league_entrance_alignment.rs"]
mod league_entrance_alignment;
