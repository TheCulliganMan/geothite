use super::*;

fn parent() -> CompiledGamePack {
    read_verified_compiled_game_pack(std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_DAY_CARE_BASE_PACK").expect("completed Minnesota base pack"),
    ))
    .expect("verified parent")
}

#[test]
#[ignore = "requires CRYSTAL_DAY_CARE_BASE_PACK"]
fn minnesota_daycares_preserve_story_and_add_two_connected_native_houses() {
    let base = parent();
    let next = build_minnesota_daycares(&base).expect("two usable houses");
    assert_eq!(next.data.maps.len(), base.data.maps.len() + 2);
    assert_eq!(next.runtime_files, base.runtime_files);
    assert_eq!(next.compiled_audio, base.compiled_audio);
    assert_eq!(next.audio_manifest, base.audio_manifest);
    assert_eq!(next.data.pokemon, base.data.pokemon);
    assert_eq!(next.data.wild_encounters, base.data.wild_encounters);
    for (name, original) in &base.data.maps {
        if !matches!(name.as_str(), "MnMankato" | "MnBemidji") {
            assert_eq!(
                serde_json::to_value(&next.data.maps[name]).unwrap(),
                serde_json::to_value(original).unwrap(),
                "untouched {name}"
            );
            continue;
        }
        let installed = &next.data.maps[name];
        assert_eq!(installed.objects, original.objects);
        assert_eq!(installed.attributes, original.attributes);
        assert_eq!(installed.events.coord_events, original.events.coord_events);
        assert!(installed.events.warps.starts_with(&original.events.warps));
        assert!(
            installed
                .events
                .bg_events
                .starts_with(&original.events.bg_events)
        );
        for (label, script) in &original.scripts {
            assert_eq!(&installed.scripts[label], script, "preserve {label}");
        }
        assert_eq!(
            installed.events.warps.len(),
            original.events.warps.len() + 2
        );
    }
    for (outside, room, door, landing) in [
        ("MnMankato", "MnMankatoDayCare", [21, 27], [21, 28]),
        ("MnBemidji", "MnBemidjiDayCare", [13, 33], [13, 34]),
    ] {
        let exterior = &next.data.maps[outside];
        assert_eq!(
            crate::authored_town_geometry::kind(&next, exterior, door).unwrap(),
            "DOOR"
        );
        assert_eq!(
            crate::authored_town_geometry::kind(&next, exterior, landing).unwrap(),
            "FLOOR"
        );
        let inside = &next.data.maps[room];
        assert_eq!(inside.objects.len(), 3);
        assert_eq!(inside.events.warps.len(), 3);
        for (suffix, routine) in [
            ("Man", "DayCareMan"),
            ("Lady", "DayCareLady"),
            ("Eggs", "DayCareManOutside"),
        ] {
            assert!(
                inside
                    .script_runtime_commands
                    .iter()
                    .any(|c| c.source_script == format!("{room}{suffix}")
                        && c.command == "special"
                        && c.args == [routine])
            );
        }
    }
    assert!(build_minnesota_daycares(&next).is_err());
}
