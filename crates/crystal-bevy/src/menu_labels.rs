//! Visible labels shared by the graphical and text frontends. The English
//! office scripts retain Japanese menu strings; keep their source choices
//! intact while presenting their authored meanings with the English font.
use crystal_runtime::{RuntimeMenuSnapshot, RuntimeMenuSource};

pub fn visible_script_menu_option_label<'a>(
    menu: &RuntimeMenuSnapshot,
    option: &'a str,
) -> &'a str {
    if !matches!(&menu.source, RuntimeMenuSource::ScriptDefinition { map_name }
        if map_name == "PokecomCenterAdminOfficeMobile")
    {
        return option;
    }
    match (menu.menu_id.as_str(), option) {
        (
            ".Computer1MenuHeader@PokecomCenterAdminOfficeMobileComputer1",
            "<POKEMON>　つうしん　クラブ",
        ) => "POKECOM CLUB",
        (".Computer1MenuHeader@PokecomCenterAdminOfficeMobileComputer1", "モバイルセンター") => {
            "MOBILE CENTER"
        }
        (
            ".Computer2MenuHeader@PokecomCenterAdminOfficeMobileComputer2",
            "でんわ<WO>つかうとき",
        ) => "USING PHONE",
        (
            ".Computer2MenuHeader@PokecomCenterAdminOfficeMobileComputer2",
            "でんわ<GA>つながらないとき",
        ) => "NO CONNECTION",
        (
            ".Computer1MenuHeader@PokecomCenterAdminOfficeMobileComputer1"
            | ".Computer2MenuHeader@PokecomCenterAdminOfficeMobileComputer2",
            "やめる",
        ) => "QUIT",
        _ => option,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn menu(computer: u8) -> RuntimeMenuSnapshot {
        RuntimeMenuSnapshot {
            menu_id: format!(
                ".Computer{computer}MenuHeader@PokecomCenterAdminOfficeMobileComputer{computer}"
            ),
            source: RuntimeMenuSource::ScriptDefinition {
                map_name: "PokecomCenterAdminOfficeMobile".into(),
            },
            definition: None,
            window_open: true,
            coords: Some([0, 0, 15, 8]),
            menu_2d_requested: false,
            layout: crystal_runtime::RuntimeMenuLayoutSnapshot {
                declared_coords: Some([0, 0, 15, 8]),
                data_commands: vec![],
                vertical_menus: vec![],
            },
        }
    }
    #[test]
    fn office_menu_labels_fit_authored_column_and_keep_exact_source_scope() {
        for (computer, choices) in [
            (
                1,
                [
                    ("<POKEMON>　つうしん　クラブ", "POKECOM CLUB"),
                    ("モバイルセンター", "MOBILE CENTER"),
                    ("やめる", "QUIT"),
                ],
            ),
            (
                2,
                [
                    ("でんわ<WO>つかうとき", "USING PHONE"),
                    ("でんわ<GA>つながらないとき", "NO CONNECTION"),
                    ("やめる", "QUIT"),
                ],
            ),
        ] {
            let original = menu(computer);
            for (source, expected) in choices {
                let label = visible_script_menu_option_label(&original, source);
                assert_eq!(label, expected);
                assert!(
                    label.is_ascii() && label.len() <= 13,
                    "the English bitmap label must fit inside the authored border"
                );
                let mut foreign = original.clone();
                foreign.source = RuntimeMenuSource::SpecialRoutine;
                assert_eq!(visible_script_menu_option_label(&foreign, source), source);
                foreign.source = RuntimeMenuSource::ScriptDefinition {
                    map_name: "GoldenrodPokecenter1F".into(),
                };
                assert_eq!(visible_script_menu_option_label(&foreign, source), source);
                foreign = original.clone();
                foreign.menu_id.push_str("-other");
                assert_eq!(visible_script_menu_option_label(&foreign, source), source);
            }
            assert_eq!(
                visible_script_menu_option_label(&original, "custom source option"),
                "custom source option"
            );
        }
    }
}
