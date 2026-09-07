// Alphabetical order from pokecrystal/data/pokemon/dex_order_alpha.asm.
const POKEDEX_ALPHA_ORDER: &[&str] = &[
    "ABRA",
    "AERODACTYL",
    "AIPOM",
    "ALAKAZAM",
    "AMPHAROS",
    "ARBOK",
    "ARCANINE",
    "ARIADOS",
    "ARTICUNO",
    "AZUMARILL",
    "BAYLEEF",
    "BEEDRILL",
    "BELLOSSOM",
    "BELLSPROUT",
    "BLASTOISE",
    "BLISSEY",
    "BULBASAUR",
    "BUTTERFREE",
    "CATERPIE",
    "CELEBI",
    "CHANSEY",
    "CHARIZARD",
    "CHARMANDER",
    "CHARMELEON",
    "CHIKORITA",
    "CHINCHOU",
    "CLEFABLE",
    "CLEFAIRY",
    "CLEFFA",
    "CLOYSTER",
    "CORSOLA",
    "CROBAT",
    "CROCONAW",
    "CUBONE",
    "CYNDAQUIL",
    "DELIBIRD",
    "DEWGONG",
    "DIGLETT",
    "DITTO",
    "DODRIO",
    "DODUO",
    "DONPHAN",
    "DRAGONAIR",
    "DRAGONITE",
    "DRATINI",
    "DROWZEE",
    "DUGTRIO",
    "DUNSPARCE",
    "EEVEE",
    "EKANS",
    "ELECTABUZZ",
    "ELECTRODE",
    "ELEKID",
    "ENTEI",
    "ESPEON",
    "EXEGGCUTE",
    "EXEGGUTOR",
    "FARFETCH_D",
    "FEAROW",
    "FERALIGATR",
    "FLAAFFY",
    "FLAREON",
    "FORRETRESS",
    "FURRET",
    "GASTLY",
    "GENGAR",
    "GEODUDE",
    "GIRAFARIG",
    "GLIGAR",
    "GLOOM",
    "GOLBAT",
    "GOLDEEN",
    "GOLDUCK",
    "GOLEM",
    "GRANBULL",
    "GRAVELER",
    "GRIMER",
    "GROWLITHE",
    "GYARADOS",
    "HAUNTER",
    "HERACROSS",
    "HITMONCHAN",
    "HITMONLEE",
    "HITMONTOP",
    "HO_OH",
    "HOOTHOOT",
    "HOPPIP",
    "HORSEA",
    "HOUNDOOM",
    "HOUNDOUR",
    "HYPNO",
    "IGGLYBUFF",
    "IVYSAUR",
    "JIGGLYPUFF",
    "JOLTEON",
    "JUMPLUFF",
    "JYNX",
    "KABUTO",
    "KABUTOPS",
    "KADABRA",
    "KAKUNA",
    "KANGASKHAN",
    "KINGDRA",
    "KINGLER",
    "KOFFING",
    "KRABBY",
    "LANTURN",
    "LAPRAS",
    "LARVITAR",
    "LEDIAN",
    "LEDYBA",
    "LICKITUNG",
    "LUGIA",
    "MACHAMP",
    "MACHOKE",
    "MACHOP",
    "MAGBY",
    "MAGCARGO",
    "MAGIKARP",
    "MAGMAR",
    "MAGNEMITE",
    "MAGNETON",
    "MANKEY",
    "MANTINE",
    "MAREEP",
    "MARILL",
    "MAROWAK",
    "MEGANIUM",
    "MEOWTH",
    "METAPOD",
    "MEW",
    "MEWTWO",
    "MILTANK",
    "MISDREAVUS",
    "MOLTRES",
    "MR__MIME",
    "MUK",
    "MURKROW",
    "NATU",
    "NIDOKING",
    "NIDOQUEEN",
    "NIDORAN_F",
    "NIDORAN_M",
    "NIDORINA",
    "NIDORINO",
    "NINETALES",
    "NOCTOWL",
    "OCTILLERY",
    "ODDISH",
    "OMANYTE",
    "OMASTAR",
    "ONIX",
    "PARAS",
    "PARASECT",
    "PERSIAN",
    "PHANPY",
    "PICHU",
    "PIDGEOT",
    "PIDGEOTTO",
    "PIDGEY",
    "PIKACHU",
    "PILOSWINE",
    "PINECO",
    "PINSIR",
    "POLITOED",
    "POLIWAG",
    "POLIWHIRL",
    "POLIWRATH",
    "PONYTA",
    "PORYGON",
    "PORYGON2",
    "PRIMEAPE",
    "PSYDUCK",
    "PUPITAR",
    "QUAGSIRE",
    "QUILAVA",
    "QWILFISH",
    "RAICHU",
    "RAIKOU",
    "RAPIDASH",
    "RATICATE",
    "RATTATA",
    "REMORAID",
    "RHYDON",
    "RHYHORN",
    "SANDSHREW",
    "SANDSLASH",
    "SCIZOR",
    "SCYTHER",
    "SEADRA",
    "SEAKING",
    "SEEL",
    "SENTRET",
    "SHELLDER",
    "SHUCKLE",
    "SKARMORY",
    "SKIPLOOM",
    "SLOWBRO",
    "SLOWKING",
    "SLOWPOKE",
    "SLUGMA",
    "SMEARGLE",
    "SMOOCHUM",
    "SNEASEL",
    "SNORLAX",
    "SNUBBULL",
    "SPEAROW",
    "SPINARAK",
    "SQUIRTLE",
    "STANTLER",
    "STARMIE",
    "STARYU",
    "STEELIX",
    "SUDOWOODO",
    "SUICUNE",
    "SUNFLORA",
    "SUNKERN",
    "SWINUB",
    "TANGELA",
    "TAUROS",
    "TEDDIURSA",
    "TENTACOOL",
    "TENTACRUEL",
    "TOGEPI",
    "TOGETIC",
    "TOTODILE",
    "TYPHLOSION",
    "TYRANITAR",
    "TYROGUE",
    "UMBREON",
    "UNOWN",
    "URSARING",
    "VAPOREON",
    "VENOMOTH",
    "VENONAT",
    "VENUSAUR",
    "VICTREEBEL",
    "VILEPLUME",
    "VOLTORB",
    "VULPIX",
    "WARTORTLE",
    "WEEDLE",
    "WEEPINBELL",
    "WEEZING",
    "WIGGLYTUFF",
    "WOBBUFFET",
    "WOOPER",
    "XATU",
    "YANMA",
    "ZAPDOS",
    "ZUBAT",
];

// Species order from pokecrystal/data/pokemon/dex_order_new.asm.
const POKEDEX_NEW_ORDER: &[&str] = &[
    "CHIKORITA",
    "BAYLEEF",
    "MEGANIUM",
    "CYNDAQUIL",
    "QUILAVA",
    "TYPHLOSION",
    "TOTODILE",
    "CROCONAW",
    "FERALIGATR",
    "PIDGEY",
    "PIDGEOTTO",
    "PIDGEOT",
    "SPEAROW",
    "FEAROW",
    "HOOTHOOT",
    "NOCTOWL",
    "RATTATA",
    "RATICATE",
    "SENTRET",
    "FURRET",
    "PICHU",
    "PIKACHU",
    "RAICHU",
    "CATERPIE",
    "METAPOD",
    "BUTTERFREE",
    "WEEDLE",
    "KAKUNA",
    "BEEDRILL",
    "LEDYBA",
    "LEDIAN",
    "SPINARAK",
    "ARIADOS",
    "GEODUDE",
    "GRAVELER",
    "GOLEM",
    "ZUBAT",
    "GOLBAT",
    "CROBAT",
    "CLEFFA",
    "CLEFAIRY",
    "CLEFABLE",
    "IGGLYBUFF",
    "JIGGLYPUFF",
    "WIGGLYTUFF",
    "TOGEPI",
    "TOGETIC",
    "SANDSHREW",
    "SANDSLASH",
    "EKANS",
    "ARBOK",
    "DUNSPARCE",
    "MAREEP",
    "FLAAFFY",
    "AMPHAROS",
    "WOOPER",
    "QUAGSIRE",
    "GASTLY",
    "HAUNTER",
    "GENGAR",
    "UNOWN",
    "ONIX",
    "STEELIX",
    "BELLSPROUT",
    "WEEPINBELL",
    "VICTREEBEL",
    "HOPPIP",
    "SKIPLOOM",
    "JUMPLUFF",
    "PARAS",
    "PARASECT",
    "POLIWAG",
    "POLIWHIRL",
    "POLIWRATH",
    "POLITOED",
    "MAGIKARP",
    "GYARADOS",
    "GOLDEEN",
    "SEAKING",
    "SLOWPOKE",
    "SLOWBRO",
    "SLOWKING",
    "ODDISH",
    "GLOOM",
    "VILEPLUME",
    "BELLOSSOM",
    "DROWZEE",
    "HYPNO",
    "ABRA",
    "KADABRA",
    "ALAKAZAM",
    "DITTO",
    "PINECO",
    "FORRETRESS",
    "NIDORAN_F",
    "NIDORINA",
    "NIDOQUEEN",
    "NIDORAN_M",
    "NIDORINO",
    "NIDOKING",
    "YANMA",
    "SUNKERN",
    "SUNFLORA",
    "EXEGGCUTE",
    "EXEGGUTOR",
    "SUDOWOODO",
    "WOBBUFFET",
    "VENONAT",
    "VENOMOTH",
    "SCYTHER",
    "SCIZOR",
    "PINSIR",
    "HERACROSS",
    "KOFFING",
    "WEEZING",
    "GRIMER",
    "MUK",
    "MAGNEMITE",
    "MAGNETON",
    "VOLTORB",
    "ELECTRODE",
    "AIPOM",
    "SNUBBULL",
    "GRANBULL",
    "VULPIX",
    "NINETALES",
    "GROWLITHE",
    "ARCANINE",
    "STANTLER",
    "MARILL",
    "AZUMARILL",
    "DIGLETT",
    "DUGTRIO",
    "MANKEY",
    "PRIMEAPE",
    "MEOWTH",
    "PERSIAN",
    "PSYDUCK",
    "GOLDUCK",
    "MACHOP",
    "MACHOKE",
    "MACHAMP",
    "TYROGUE",
    "HITMONLEE",
    "HITMONCHAN",
    "HITMONTOP",
    "GIRAFARIG",
    "TAUROS",
    "MILTANK",
    "MAGBY",
    "MAGMAR",
    "SMOOCHUM",
    "JYNX",
    "ELEKID",
    "ELECTABUZZ",
    "MR__MIME",
    "SMEARGLE",
    "FARFETCH_D",
    "NATU",
    "XATU",
    "QWILFISH",
    "TENTACOOL",
    "TENTACRUEL",
    "KRABBY",
    "KINGLER",
    "SHUCKLE",
    "STARYU",
    "STARMIE",
    "SHELLDER",
    "CLOYSTER",
    "CORSOLA",
    "REMORAID",
    "OCTILLERY",
    "CHINCHOU",
    "LANTURN",
    "SEEL",
    "DEWGONG",
    "LICKITUNG",
    "TANGELA",
    "EEVEE",
    "VAPOREON",
    "JOLTEON",
    "FLAREON",
    "ESPEON",
    "UMBREON",
    "HORSEA",
    "SEADRA",
    "KINGDRA",
    "GLIGAR",
    "DELIBIRD",
    "SWINUB",
    "PILOSWINE",
    "TEDDIURSA",
    "URSARING",
    "PHANPY",
    "DONPHAN",
    "MANTINE",
    "SKARMORY",
    "DODUO",
    "DODRIO",
    "PONYTA",
    "RAPIDASH",
    "CUBONE",
    "MAROWAK",
    "KANGASKHAN",
    "RHYHORN",
    "RHYDON",
    "MURKROW",
    "HOUNDOUR",
    "HOUNDOOM",
    "SLUGMA",
    "MAGCARGO",
    "SNEASEL",
    "MISDREAVUS",
    "PORYGON",
    "PORYGON2",
    "CHANSEY",
    "BLISSEY",
    "LAPRAS",
    "OMANYTE",
    "OMASTAR",
    "KABUTO",
    "KABUTOPS",
    "AERODACTYL",
    "SNORLAX",
    "BULBASAUR",
    "IVYSAUR",
    "VENUSAUR",
    "CHARMANDER",
    "CHARMELEON",
    "CHARIZARD",
    "SQUIRTLE",
    "WARTORTLE",
    "BLASTOISE",
    "ARTICUNO",
    "ZAPDOS",
    "MOLTRES",
    "RAIKOU",
    "ENTEI",
    "SUICUNE",
    "DRATINI",
    "DRAGONAIR",
    "DRAGONITE",
    "LARVITAR",
    "PUPITAR",
    "TYRANITAR",
    "LUGIA",
    "HO_OH",
    "MEWTWO",
    "MEW",
    "CELEBI",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum VisiblePokedexMode {
    #[default]
    New,
    Old,
    Alphabetical,
}

#[derive(Clone, Debug, Default, Hash)]
struct VisiblePokedexControls {
    mode: VisiblePokedexMode,
    option_cursor: Option<usize>,
    search_cursor: Option<usize>,
    search_types: [usize; 2],
    search_results: Option<Vec<usize>>,
    search_backup: Option<(usize, usize)>,
    search_not_found: bool,
    entry_action: usize,
    area_region: Option<bool>,
    area_frames: u8,
    area_show_player: bool,
    printer_open: bool,
    unown_cursor: Option<usize>,
    return_to_start: bool,
    search_animation: Option<(u16, Vec<usize>)>,
    not_found_frames: u16,
}

fn visible_pokedex_order(snapshot: &RuntimeShellSnapshot, mode: VisiblePokedexMode) -> Vec<usize> {
    let mut order: Vec<usize> = (0..snapshot.pokemon.len()).collect();
    match mode {
        VisiblePokedexMode::New => order.sort_by_key(|&i| {
            (
                POKEDEX_NEW_ORDER
                    .iter()
                    .position(|&id| id == snapshot.pokemon[i].species_id)
                    .unwrap_or(POKEDEX_NEW_ORDER.len()),
                snapshot.pokemon[i].int_id,
            )
        }),
        VisiblePokedexMode::Old => order.sort_by_key(|&i| snapshot.pokemon[i].int_id),
        VisiblePokedexMode::Alphabetical => {
            order.retain(|&i| {
                snapshot
                    .progression
                    .pokedex_seen_species
                    .contains(&snapshot.pokemon[i].species_id)
            });
            order.sort_by_key(|&i| {
                (
                    POKEDEX_ALPHA_ORDER
                        .iter()
                        .position(|&id| id == snapshot.pokemon[i].species_id)
                        .unwrap_or(POKEDEX_ALPHA_ORDER.len()),
                    snapshot.pokemon[i].species_id.clone(),
                )
            });
            return order;
        }
    }
    let end = order
        .iter()
        .rposition(|&i| {
            snapshot
                .progression
                .pokedex_seen_species
                .contains(&snapshot.pokemon[i].species_id)
        })
        .map_or(0, |i| i + 1);
    order.truncate(end);
    order
}

fn pokedex_option_entries(runtime_shell: &BevyRuntimeShell) -> Option<Vec<String>> {
    let cursor = runtime_shell.pokedex_controls.option_cursor?;
    let mut options = vec!["NEW #DEX MODE", "OLD #DEX MODE", "A to Z MODE"];
    if pokedex_unown_unlocked(runtime_shell) {
        options.push("UNOWN MODE");
    }
    Some(
        options
            .into_iter()
            .enumerate()
            .map(|(i, label)| format!("{}{label}", if i == cursor { ">" } else { " " }))
            .collect(),
    )
}

fn pokedex_unown_unlocked(shell: &BevyRuntimeShell) -> bool {
    shell
        .shell
        .session()
        .state()
        .flags
        .engine_flags
        .get("ENGINE_UNOWN_DEX")
        == Some(&true)
}

fn press_visible_pokedex_select(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if pokedex_input_delay_active(runtime_shell) {
        return Ok(());
    }
    if runtime_shell.pokedex_controls.unown_cursor.is_some()
        || runtime_shell.pokedex_detail_open
        || runtime_shell.pokedex_controls.search_cursor.is_some()
        || runtime_shell.pokedex_controls.search_results.is_some()
    {
        return Ok(());
    }
    runtime_shell.pokedex_controls.option_cursor =
        if runtime_shell.pokedex_controls.option_cursor.is_some() {
            None
        } else {
            Some(match runtime_shell.pokedex_controls.mode {
                VisiblePokedexMode::New => 0,
                VisiblePokedexMode::Old => 1,
                VisiblePokedexMode::Alphabetical => 2,
            })
        };
    record_visible_runtime_action(runtime_shell, "pokedex:options")?;
    Ok(())
}

const POKEDEX_SEARCH_TYPES: &[&str] = &[
    "----", "NORMAL", "FIRE", "WATER", "GRASS", "ELECTRIC", "ICE", "FIGHTING", "POISON", "GROUND",
    "FLYING", "PSYCHIC", "BUG", "ROCK", "GHOST", "DRAGON", "DARK", "STEEL",
];

fn visible_pokedex_listing(
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
) -> Vec<usize> {
    shell
        .pokedex_controls
        .search_results
        .clone()
        .unwrap_or_else(|| visible_pokedex_order(snapshot, shell.pokedex_controls.mode))
}

fn visible_pokedex_listing_height(shell: &BevyRuntimeShell) -> usize {
    if shell.pokedex_controls.search_results.is_some() {
        4
    } else {
        7
    }
}

fn press_visible_pokedex_start(shell: &mut BevyRuntimeShell) -> Result<()> {
    if pokedex_input_delay_active(shell) {
        return Ok(());
    }
    if shell.pokedex_controls.unown_cursor.is_some()
        || shell.pokedex_detail_open
        || shell.pokedex_controls.option_cursor.is_some()
        || shell.pokedex_controls.search_results.is_some()
    {
        return Ok(());
    }
    if shell.pokedex_controls.search_cursor.take().is_none() {
        shell.pokedex_controls.search_cursor = Some(0);
        shell.pokedex_controls.search_types = [1, 0];
    }
    shell.pokedex_controls.search_not_found = false;
    record_visible_runtime_action(shell, "pokedex:search")
}

fn change_visible_pokedex_search_type(shell: &mut BevyRuntimeShell, delta: isize) {
    let Some(cursor) = shell.pokedex_controls.search_cursor else {
        return;
    };
    if cursor >= 2 {
        return;
    }
    let minimum = if cursor == 0 { 1 } else { 0 };
    let count = POKEDEX_SEARCH_TYPES.len() - minimum;
    let value = &mut shell.pokedex_controls.search_types[cursor];
    *value = ((*value as isize - minimum as isize + delta).rem_euclid(count as isize)) as usize
        + minimum;
    shell.pokedex_controls.search_not_found = false;
}

fn pokedex_search_entries(shell: &BevyRuntimeShell) -> Option<Vec<String>> {
    let cursor = shell.pokedex_controls.search_cursor?;
    let [first, second] = shell.pokedex_controls.search_types;
    let mut entries = [
        format!("TYPE1 {}", POKEDEX_SEARCH_TYPES[first]),
        format!("TYPE2 {}", POKEDEX_SEARCH_TYPES[second]),
        "BEGIN SEARCH!!".into(),
        "CANCEL".into(),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, s)| format!("{}{s}", if i == cursor { ">" } else { " " }))
    .collect::<Vec<_>>();
    if shell.pokedex_controls.search_not_found {
        entries.push("The specified type".into());
        entries.push("was not found.".into());
    }
    Some(entries)
}

fn press_visible_pokedex_search_a(shell: &mut BevyRuntimeShell) -> Result<()> {
    match shell.pokedex_controls.search_cursor {
        Some(0 | 1) => change_visible_pokedex_search_type(shell, 1),
        Some(2) => {
            let snapshot = shell.shell.snapshot()?;
            let results = visible_pokedex_order(&snapshot, shell.pokedex_controls.mode)
                .into_iter()
                .filter(|&i| {
                    let mon = &snapshot.pokemon[i];
                    snapshot
                        .progression
                        .pokedex_caught_species
                        .contains(&mon.species_id)
                        && shell.pokedex_controls.search_types.iter().all(|&kind| {
                            let species_type = if kind == 11 {
                                "PSYCHIC_TYPE"
                            } else {
                                POKEDEX_SEARCH_TYPES[kind]
                            };
                            kind == 0 || mon.type1 == species_type || mon.type2 == species_type
                        })
                })
                .collect::<Vec<_>>();
            shell.pokedex_controls.search_animation = Some((0, results));
        }
        Some(3) => {
            shell.pokedex_controls.search_cursor = None;
        }
        _ => {}
    }
    record_visible_runtime_action(shell, "pokedex:search:confirm")
}

fn visible_pokedex_nests(
    shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    kanto: bool,
) -> Result<Vec<crate::core::models::PokegearLandmark>> {
    let species = &selected_pokedex_catalog_species(snapshot, shell.pokedex_cursor)?.species_id;
    let data = shell.shell.runtime().data();
    let mut maps = std::collections::BTreeSet::new();
    for (map, encounters) in &data.wild_encounters {
        if [&encounters.grass, &encounters.water]
            .into_iter()
            .flatten()
            .any(|table| {
                table
                    .morning
                    .iter()
                    .chain(&table.day)
                    .chain(&table.night)
                    .any(|entry| &entry.species == species)
            })
        {
            maps.insert(map.clone());
        }
    }
    if !kanto {
        // FindNest checks RoamMon1 and RoamMon2 only, as in Crystal.
        for mon in shell.shell.session().state().roaming_pokemon.iter().take(2) {
            if mon.species.as_ref() == Some(species) {
                if let Some(metadata) = data.runtime_map_metadata.values().find(|map| {
                    map.group_id == u16::from(mon.map_group)
                        && map.map_id == u16::from(mon.map_number)
                }) {
                    maps.insert(metadata.name.clone());
                }
            }
        }
    }
    let mut constants = std::collections::BTreeSet::new();
    for map in maps {
        let module = data
            .maps
            .get(&map)
            .with_context(|| format!("Pokedex nest map {map} is missing"))?;
        if let Some(location) = &module.attributes.location {
            constants.insert(location.clone());
        }
    }
    Ok(snapshot
        .presentation
        .pokegear_landmarks
        .landmarks
        .iter()
        .filter(|landmark| {
            landmark.region == if kanto { "KANTO" } else { "JOHTO" }
                && constants.contains(&landmark.constant)
        })
        .cloned()
        .collect())
}

const POKEDEX_UNOWN_WORDS: &[&str] = &[
    "ANGRY", "BEAR", "CHASE", "DIRECT", "ENGAGE", "FIND", "GIVE", "HELP", "INCREASE", "JOIN",
    "KEEP", "LAUGH", "MAKE", "NUZZLE", "OBSERVE", "PERFORM", "QUICKEN", "REASSURE", "SEARCH",
    "TELL", "UNDO", "VANISH", "WANT", "XXXXX", "YIELD", "ZOOM",
];

const POKEDEX_OLD_CURSOR: &[(i16, i16, u8, bool, bool)] = &[
    (63, 8, 0x30, false, false),
    (63, 0, 0x31, false, false),
    (71, 0, 0x32, false, false),
    (79, 0, 0x32, false, false),
    (87, 0, 0x32, false, false),
    (95, 0, 0x33, false, false),
    (118, 0, 0x33, true, false),
    (126, 0, 0x32, true, false),
    (134, 0, 0x32, true, false),
    (142, 0, 0x32, true, false),
    (150, 0, 0x31, true, false),
    (150, 8, 0x30, true, false),
    (63, 16, 0x30, false, true),
    (63, 24, 0x31, false, true),
    (71, 24, 0x32, false, true),
    (79, 24, 0x32, false, true),
    (87, 24, 0x32, false, true),
    (95, 24, 0x33, false, true),
    (118, 24, 0x33, true, true),
    (126, 24, 0x32, true, true),
    (134, 24, 0x32, true, true),
    (142, 24, 0x32, true, true),
    (150, 24, 0x31, true, true),
    (150, 16, 0x30, true, true),
];

const POKEDEX_OLD_TOP_CURSOR: &[(i16, i16, u8, bool, bool)] = &[
    (63, 8, 0x30, false, false),
    (63, 0, 0x34, false, false),
    (71, 0, 0x35, false, false),
    (79, 0, 0x35, false, false),
    (87, 0, 0x35, false, false),
    (95, 0, 0x36, false, false),
    (118, 0, 0x36, true, false),
    (126, 0, 0x35, true, false),
    (134, 0, 0x35, true, false),
    (142, 0, 0x35, true, false),
    (150, 0, 0x34, true, false),
    (150, 8, 0x30, true, false),
    (63, 16, 0x30, false, true),
    (63, 24, 0x31, false, true),
    (71, 24, 0x32, false, true),
    (79, 24, 0x32, false, true),
    (87, 24, 0x32, false, true),
    (95, 24, 0x33, false, true),
    (118, 24, 0x33, true, true),
    (126, 24, 0x32, true, true),
    (134, 24, 0x32, true, true),
    (142, 24, 0x32, true, true),
    (150, 24, 0x31, true, true),
    (150, 16, 0x30, true, true),
];

const POKEDEX_NEW_CURSOR: &[(i16, i16, u8, bool, bool)] = &[
    (63, 11, 0x30, false, false),
    (63, 3, 0x31, false, false),
    (71, 3, 0x32, false, false),
    (79, 3, 0x32, false, false),
    (87, 3, 0x33, false, false),
    (120, 3, 0x33, true, false),
    (128, 3, 0x32, true, false),
    (136, 3, 0x32, true, false),
    (144, 3, 0x31, true, false),
    (144, 11, 0x30, true, false),
    (63, 19, 0x30, false, true),
    (63, 27, 0x31, false, true),
    (71, 27, 0x32, false, true),
    (79, 27, 0x32, false, true),
    (87, 27, 0x33, false, true),
    (120, 27, 0x33, true, true),
    (128, 27, 0x32, true, true),
    (136, 27, 0x32, true, true),
    (144, 27, 0x31, true, true),
    (144, 19, 0x30, true, true),
];

const POKEDEX_RESULTS_CURSOR: &[(i16, i16, u8, bool, bool)] = &[
    (63, 11, 0x30, false, false),
    (63, 3, 0x31, false, false),
    (71, 3, 0x32, false, false),
    (79, 3, 0x32, false, false),
    (87, 3, 0x32, false, false),
    (95, 3, 0x33, false, false),
    (118, 3, 0x33, true, false),
    (126, 3, 0x32, true, false),
    (134, 3, 0x32, true, false),
    (142, 3, 0x32, true, false),
    (150, 3, 0x31, true, false),
    (150, 11, 0x30, true, false),
    (63, 19, 0x30, false, true),
    (63, 27, 0x31, false, true),
    (71, 27, 0x32, false, true),
    (79, 27, 0x32, false, true),
    (87, 27, 0x32, false, true),
    (95, 27, 0x33, false, true),
    (118, 27, 0x33, true, true),
    (126, 27, 0x32, true, true),
    (134, 27, 0x32, true, true),
    (142, 27, 0x32, true, true),
    (150, 27, 0x31, true, true),
    (150, 19, 0x30, true, true),
];

fn pokedex_input_delay_active(shell: &BevyRuntimeShell) -> bool {
    shell.pokedex_controls.search_animation.is_some()
        || shell.pokedex_controls.not_found_frames != 0
}

fn advance_visible_pokedex_search(shell: &mut BevyRuntimeShell, frames: u32) {
    if !shell.pokedex_menu_open {
        return;
    }
    if let Some((elapsed, results)) = shell.pokedex_controls.search_animation.take() {
        let elapsed = u32::from(elapsed) + frames;
        if elapsed >= 207 {
            if let Some(&index) = results.first() {
                shell.pokedex_controls.search_backup =
                    Some((shell.pokedex_cursor, shell.pokedex_scroll));
                shell.pokedex_cursor = index;
                shell.pokedex_scroll = 0;
                shell.pokedex_controls.search_results = Some(results);
                shell.pokedex_controls.search_cursor = None;
            } else {
                shell.pokedex_controls.search_cursor = Some(0);
                shell.pokedex_controls.not_found_frames =
                    128u32.saturating_sub(elapsed - 207) as u16;
                shell.pokedex_controls.search_not_found =
                    shell.pokedex_controls.not_found_frames != 0;
            }
        } else {
            shell.pokedex_controls.search_animation = Some((elapsed as u16, results));
        }
        mark_runtime_presentation_dirty(shell);
    } else if shell.pokedex_controls.not_found_frames != 0 {
        shell.pokedex_controls.not_found_frames = shell
            .pokedex_controls
            .not_found_frames
            .saturating_sub(frames.min(u32::from(u16::MAX)) as u16);
        if shell.pokedex_controls.not_found_frames == 0 {
            shell.pokedex_controls.search_not_found = false;
            mark_runtime_presentation_dirty(shell);
        }
    }
}

const POKEDEX_SEARCH_LABELS: &[&str] = &[
    "  ----  ", " NORMAL ", "  FIRE  ", " WATER  ", " GRASS  ", "ELECTRIC", "  ICE   ", "FIGHTING",
    " POISON ", " GROUND ", " FLYING ", "PSYCHIC ", "  BUG   ", "  ROCK  ", " GHOST  ", " DRAGON ",
    "  DARK  ", " STEEL  ",
];
