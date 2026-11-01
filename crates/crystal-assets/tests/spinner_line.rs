//! Opt-in regression for the lure family's earned final-stage power.
use std::path::Path;

use crystal_core::models::Dv;
use crystal_core::systems::evolution::{self, EvolutionContext, LinkMode};
use crystal_core::world::encounters::TimeOfDay;

#[test]
#[ignore = "requires CRYSTAL_SPINNER_PACK with the completed authored move/cry overlays"]
fn spinner_line_evolves_at_24_and_40_and_earns_its_physical_stab() {
    let path = std::env::var_os("CRYSTAL_SPINNER_PACK").expect("completed spinner pack");
    let pack = crystal_assets::read_verified_compiled_game_pack(Path::new(&path))
        .expect("verified spinner pack");
    let data = pack.data();
    for (id, expected_total) in [("JIGLET", 275), ("TROLLURE", 410), ("CRAPPELLER", 550)] {
        let species = &data.pokemon[id];
        let stats = &species.base_stats;
        assert_eq!(
            stats.hp
                + stats.attack
                + stats.defense
                + stats.speed
                + stats.special_attack
                + stats.special_defense,
            expected_total,
            "intended power band for {id}"
        );
        assert_eq!((&species.type1[..], &species.type2[..]), ("STEEL", "WATER"));
    }
    let context = EvolutionContext {
        species: &data.pokemon,
        moves: &data.moves,
        learnsets: &data.learnsets,
        time_of_day: TimeOfDay::Day,
        current_item: None,
        force_evolution: false,
        link_mode: LinkMode::None,
    };
    for (id, target, threshold) in [("JIGLET", "TROLLURE", 24), ("TROLLURE", "CRAPPELLER", 40)] {
        for level in [threshold - 1, threshold] {
            let mut pokemon = data
                .create_pokemon(id, level, Dv::default())
                .expect("legal evolution candidate");
            let result =
                evolution::check_and_evolve(&mut pokemon, &data.evolutions, &context, false)
                    .expect("authoritative level evolution");
            assert_eq!(
                result.target_species.as_deref(),
                (level == threshold).then_some(target)
            );
            data.validate_saved_pokemon_reference("spinner evolution", &pokemon)
                .expect("evolved stats, moves and references");
        }
    }
    for id in ["JIGLET", "TROLLURE"] {
        assert!(
            data.learnsets[id].iter().all(|entry| {
                !matches!(entry.1.as_str(), "WATERFALL" | "IRON_HEAD") || entry.0 >= 40
            }),
            "{id} must not regain its old early power spike"
        );
    }
    let adult = data
        .create_pokemon("CRAPPELLER", 40, Dv::default())
        .expect("legal level-40 final form");
    for name in ["WATERFALL", "IRON_HEAD"] {
        assert!(
            adult.moves.iter().any(|learned| learned.name == name),
            "the evolved adult can immediately use {name}"
        );
    }
    assert!(data.evolutions.0["CRAPPELLER"].is_empty());
}

#[test]
#[ignore = "requires CRYSTAL_SPINNER_PACK before installing the reviewed overworld banks"]
fn overworld_banks_preserve_content_and_reject_conflicting_reinstallation() {
    use crystal_assets::minnesota_overworld_sprites::OverworldManifest;

    let path = std::env::var_os("CRYSTAL_SPINNER_PACK").expect("completed spinner pack");
    let base = crystal_assets::read_verified_compiled_game_pack(Path::new(&path))
        .expect("verified spinner pack");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/pokemon/overworld/20261027-v1/sources/mn_f015");
    for filename in [
        "manifest-jiglet-v2-native-pupils.json",
        "manifest-trollure-v4-coarse-hooks.json",
        "manifest-crappeller-v5-hooks.json",
    ] {
        let manifest = OverworldManifest::from_json(
            &std::fs::read(root.join(filename)).expect("reviewed manifest"),
        )
        .expect("valid manifest");
        let mut compiled = manifest.compile(&root).expect("native source replay");
        let installed = compiled.build_pack(&base).expect("install reviewed banks");
        assert_eq!(
            installed.data(),
            base.data(),
            "species, maps and audio unchanged"
        );
        assert_eq!(
            installed.audio_manifest().expect("audio manifest"),
            base.audio_manifest().expect("base audio manifest")
        );
        assert_eq!(installed.compiled_audio(), base.compiled_audio());
        for (name, bytes) in base.runtime_files() {
            assert_eq!(installed.runtime_files().get(name), Some(bytes));
        }
        for (name, bytes) in &compiled.runtime_files {
            assert_eq!(installed.runtime_files().get(name), Some(bytes));
        }
        let repeated = compiled
            .build_pack(&installed)
            .expect("identical reinstallation");
        assert_eq!(
            installed.identity().expect("identity"),
            repeated.identity().expect("identity")
        );
        compiled
            .runtime_files
            .values_mut()
            .next()
            .expect("compiled bank")[0] ^= 1;
        assert!(
            compiled.build_pack(&installed).is_err(),
            "conflicting art must not replace an existing bank"
        );
    }
}
