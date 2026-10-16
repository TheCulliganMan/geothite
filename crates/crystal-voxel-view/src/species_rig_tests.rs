use super::*;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::OnceLock};

#[test]
fn unique_skin_motion_preserves_full_vertex_animation_envelope_bits() {
    for species in [Species::Slowpoke, Species::Chikorita] {
        let rig = fixture(species);
        let complete = rig.compute_envelope_vertices(&(0..rig.neutral.positions.len()).collect::<Vec<_>>());
        let unique = rig.compute_envelope();
        assert_eq!(complete.0.to_array().map(f32::to_bits), unique.0.to_array().map(f32::to_bits));
        assert_eq!(complete.1.to_array().map(f32::to_bits), unique.1.to_array().map(f32::to_bits));
    }
}

fn candidate(species: Species) -> Vec<u8> {
    let key = format!("{}_GLB", species.name().to_uppercase());
    let canonical = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(match species {
        Species::Procedural(_) => unreachable!("GLB fixture only"),
        Species::Clefairy | Species::Jigglypuff => format!("models/actor_props/icon_{}.glb", species.name()),
        Species::Chikorita | Species::Cyndaquil | Species::Totodile => {
            format!("models/actor_props/battle_{}.glb", species.name())
        }
        Species::Bayleef | Species::Meganium | Species::Gengar | Species::Spearow | Species::Wooper | Species::Gyarados | Species::Sudowoodo | Species::Lapras | Species::Snorlax | Species::Pikachu | Species::Bulbasaur | Species::Charmander | Species::Squirtle | Species::Geodude | Species::Pidgey | Species::Oddish | Species::Poliwag | Species::Rattata | Species::Raticate | Species::Psyduck | Species::Golduck | Species::Zubat | Species::Golbat | Species::Crobat | Species::Butterfree | Species::Beedrill | Species::Hoothoot | Species::Sentret | Species::Staryu | Species::Voltorb | Species::Onix | Species::Diglett | Species::Slowpoke | Species::Entei | Species::Raikou | Species::Suicune | Species::Furret | Species::Noctowl | Species::Ledyba | Species::Ledian | Species::Spinarak | Species::Ariados | Species::Quagsire | Species::Hoppip | Species::Bellossom | Species::Qwilfish | Species::Wobbuffet | Species::Dunsparce | Species::Shuckle | Species::Gligar | Species::Yanma | Species::Murkrow | Species::Misdreavus | Species::Teddiursa | Species::Ursaring | Species::Snubbull | Species::Granbull | Species::Houndour | Species::Houndoom | Species::Miltank | Species::Tauros | Species::Delibird | Species::Stantler | Species::Girafarig | Species::Phanpy | Species::Marill | Species::Natu | Species::Remoraid | Species::Octillery | Species::Swinub | Species::Piloswine | Species::Sneasel | Species::Skarmory | Species::Smeargle | Species::Lugia | Species::Articuno | Species::Zapdos | Species::Moltres | Species::Corsola | Species::Mantine | Species::Porygon | Species::Porygon2 | Species::Chinchou | Species::Lanturn | Species::Slugma | Species::Magcargo | Species::Togepi | Species::Togetic | Species::Hitmonlee | Species::Hitmonchan | Species::Hitmontop | Species::Tyrogue | Species::Smoochum | Species::Elekid | Species::Magby | Species::Pichu | Species::Cleffa | Species::Igglybuff | Species::Mareep | Species::Flaaffy | Species::Ampharos | Species::Xatu | Species::Azumarill | Species::Donphan | Species::Sunkern | Species::Sunflora | Species::Aipom | Species::Skiploom | Species::Jumpluff | Species::Unown(_) => {
            format!("models/battle_species/{}.glb", species.name())
        }
    });
    let path = std::env::var_os(key)
        .map(PathBuf::from)
        .unwrap_or(canonical);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn fixture(species: Species) -> &'static SpeciesRig {
    static CHIKORITA: OnceLock<SpeciesRig> = OnceLock::new();
    static BAYLEEF: OnceLock<SpeciesRig> = OnceLock::new();
    static MEGANIUM: OnceLock<SpeciesRig> = OnceLock::new();
    static CYNDAQUIL: OnceLock<SpeciesRig> = OnceLock::new();
    static TOTODILE: OnceLock<SpeciesRig> = OnceLock::new();
    static GENGAR: OnceLock<SpeciesRig> = OnceLock::new();
    static SPEAROW: OnceLock<SpeciesRig> = OnceLock::new();
    static WOOPER: OnceLock<SpeciesRig> = OnceLock::new();
    static GYARADOS: OnceLock<SpeciesRig> = OnceLock::new();
    static SUDOWOODO: OnceLock<SpeciesRig> = OnceLock::new();
    static LAPRAS: OnceLock<SpeciesRig> = OnceLock::new();
    static SNORLAX: OnceLock<SpeciesRig> = OnceLock::new();
    static PIKACHU: OnceLock<SpeciesRig> = OnceLock::new();
    static BULBASAUR: OnceLock<SpeciesRig> = OnceLock::new();
    static CHARMANDER: OnceLock<SpeciesRig> = OnceLock::new();
    static PSYDUCK: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLDUCK: OnceLock<SpeciesRig> = OnceLock::new();
    static ZUBAT: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLBAT: OnceLock<SpeciesRig> = OnceLock::new();
    static CROBAT: OnceLock<SpeciesRig> = OnceLock::new();
    static BUTTERFREE: OnceLock<SpeciesRig> = OnceLock::new();
    static BEEDRILL: OnceLock<SpeciesRig> = OnceLock::new();
    static HOOTHOOT: OnceLock<SpeciesRig> = OnceLock::new();
    static SENTRET: OnceLock<SpeciesRig> = OnceLock::new();
    static STARYU: OnceLock<SpeciesRig> = OnceLock::new();
    static VOLTORB: OnceLock<SpeciesRig> = OnceLock::new();
    static ONIX: OnceLock<SpeciesRig> = OnceLock::new();
    static DIGLETT: OnceLock<SpeciesRig> = OnceLock::new();
    static SLOWPOKE: OnceLock<SpeciesRig> = OnceLock::new();
    static LEDYBA: OnceLock<SpeciesRig> = OnceLock::new();
    static LEDIAN: OnceLock<SpeciesRig> = OnceLock::new();
    static ARIADOS: OnceLock<SpeciesRig> = OnceLock::new();
    static JUMPLUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static BELLOSSOM: OnceLock<SpeciesRig> = OnceLock::new();
    static QWILFISH: OnceLock<SpeciesRig> = OnceLock::new();
    static WOBBUFFET: OnceLock<SpeciesRig> = OnceLock::new();
    static DUNSPARCE: OnceLock<SpeciesRig> = OnceLock::new();
    static SHUCKLE: OnceLock<SpeciesRig> = OnceLock::new();
    static GLIGAR: OnceLock<SpeciesRig> = OnceLock::new();
    static TEDDIURSA: OnceLock<SpeciesRig> = OnceLock::new();
    static SNUBBULL: OnceLock<SpeciesRig> = OnceLock::new();
    static HOUNDOUR: OnceLock<SpeciesRig> = OnceLock::new();
    static MILTANK: OnceLock<SpeciesRig> = OnceLock::new();
    static TAUROS: OnceLock<SpeciesRig> = OnceLock::new();
    static DELIBIRD: OnceLock<SpeciesRig> = OnceLock::new();
    static STANTLER: OnceLock<SpeciesRig> = OnceLock::new();
    static GIRAFARIG: OnceLock<SpeciesRig> = OnceLock::new();
    static PHANPY: OnceLock<SpeciesRig> = OnceLock::new();
    static MARILL: OnceLock<SpeciesRig> = OnceLock::new();
    static NATU: OnceLock<SpeciesRig> = OnceLock::new();
    static TOGEPI: OnceLock<SpeciesRig> = OnceLock::new();
    static TOGETIC: OnceLock<SpeciesRig> = OnceLock::new();
    static REMORAID: OnceLock<SpeciesRig> = OnceLock::new();
    static OCTILLERY: OnceLock<SpeciesRig> = OnceLock::new();
    static SWINUB: OnceLock<SpeciesRig> = OnceLock::new();
    static PILOSWINE: OnceLock<SpeciesRig> = OnceLock::new();
    static SNEASEL: OnceLock<SpeciesRig> = OnceLock::new();
    static SKARMORY: OnceLock<SpeciesRig> = OnceLock::new();
    static SMEARGLE: OnceLock<SpeciesRig> = OnceLock::new();
    static MOLTRES: OnceLock<SpeciesRig> = OnceLock::new();
    static ZAPDOS: OnceLock<SpeciesRig> = OnceLock::new();
    static ARTICUNO: OnceLock<SpeciesRig> = OnceLock::new();
    static LUGIA: OnceLock<SpeciesRig> = OnceLock::new();
    static CORSOLA: OnceLock<SpeciesRig> = OnceLock::new();
    static MANTINE: OnceLock<SpeciesRig> = OnceLock::new();
    static PORYGON: OnceLock<SpeciesRig> = OnceLock::new();
    static PORYGON2: OnceLock<SpeciesRig> = OnceLock::new();
    static CHINCHOU: OnceLock<SpeciesRig> = OnceLock::new();
    static LANTURN: OnceLock<SpeciesRig> = OnceLock::new();
    static SLUGMA: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGCARGO: OnceLock<SpeciesRig> = OnceLock::new();
    static HITMONLEE: OnceLock<SpeciesRig> = OnceLock::new();
    static HITMONCHAN: OnceLock<SpeciesRig> = OnceLock::new();
    static HITMONTOP: OnceLock<SpeciesRig> = OnceLock::new();
    static TYROGUE: OnceLock<SpeciesRig> = OnceLock::new();
    static SMOOCHUM: OnceLock<SpeciesRig> = OnceLock::new();
    static ELEKID: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGBY: OnceLock<SpeciesRig> = OnceLock::new();
    static PICHU: OnceLock<SpeciesRig> = OnceLock::new();
    static CLEFFA: OnceLock<SpeciesRig> = OnceLock::new();
    static IGGLYBUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static MAREEP: OnceLock<SpeciesRig> = OnceLock::new();
    static FLAAFFY: OnceLock<SpeciesRig> = OnceLock::new();
    static AMPHAROS: OnceLock<SpeciesRig> = OnceLock::new();
    static XATU: OnceLock<SpeciesRig> = OnceLock::new();
    static AZUMARILL: OnceLock<SpeciesRig> = OnceLock::new();
    static DONPHAN: OnceLock<SpeciesRig> = OnceLock::new();
    static HOUNDOOM: OnceLock<SpeciesRig> = OnceLock::new();
    static GRANBULL: OnceLock<SpeciesRig> = OnceLock::new();
    static URSARING: OnceLock<SpeciesRig> = OnceLock::new();
    static MISDREAVUS: OnceLock<SpeciesRig> = OnceLock::new();
    static MURKROW: OnceLock<SpeciesRig> = OnceLock::new();
    static YANMA: OnceLock<SpeciesRig> = OnceLock::new();
    static SUNKERN: OnceLock<SpeciesRig> = OnceLock::new();
    static SUNFLORA: OnceLock<SpeciesRig> = OnceLock::new();
    static AIPOM: OnceLock<SpeciesRig> = OnceLock::new();
    static SKIPLOOM: OnceLock<SpeciesRig> = OnceLock::new();
    static HOPPIP: OnceLock<SpeciesRig> = OnceLock::new();
    static QUAGSIRE: OnceLock<SpeciesRig> = OnceLock::new();
    static SPINARAK: OnceLock<SpeciesRig> = OnceLock::new();
    static NOCTOWL: OnceLock<SpeciesRig> = OnceLock::new();
    static FURRET: OnceLock<SpeciesRig> = OnceLock::new();
    static SUICUNE: OnceLock<SpeciesRig> = OnceLock::new();
    static RAIKOU: OnceLock<SpeciesRig> = OnceLock::new();
    static ENTEI: OnceLock<SpeciesRig> = OnceLock::new();
    static UNOWN: [OnceLock<SpeciesRig>; 26] = [const { OnceLock::new() }; 26];


    static RATICATE: OnceLock<SpeciesRig> = OnceLock::new();
    static RATTATA: OnceLock<SpeciesRig> = OnceLock::new();
    static JIGGLYPUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static CLEFAIRY: OnceLock<SpeciesRig> = OnceLock::new();
    static POLIWAG: OnceLock<SpeciesRig> = OnceLock::new();
    static ODDISH: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEY: OnceLock<SpeciesRig> = OnceLock::new();
    static GEODUDE: OnceLock<SpeciesRig> = OnceLock::new();
    static SQUIRTLE: OnceLock<SpeciesRig> = OnceLock::new();
    let cache = match species {
        Species::Procedural(_) => unreachable!("GLB fixture only"),
        Species::Chikorita => &CHIKORITA,
        Species::Bayleef => &BAYLEEF,
        Species::Meganium => &MEGANIUM,
        Species::Cyndaquil => &CYNDAQUIL,
        Species::Totodile => &TOTODILE,
        Species::Gengar => &GENGAR,
        Species::Spearow => &SPEAROW,
        Species::Wooper => &WOOPER,
        Species::Gyarados => &GYARADOS,
        Species::Sudowoodo => &SUDOWOODO,
        Species::Lapras => &LAPRAS,
        Species::Snorlax => &SNORLAX,
        Species::Pikachu => &PIKACHU,
        Species::Bulbasaur => &BULBASAUR,
        Species::Charmander => &CHARMANDER,
        Species::Squirtle => &SQUIRTLE,
        Species::Geodude => &GEODUDE,
        Species::Pidgey => &PIDGEY,
        Species::Oddish => &ODDISH,
        Species::Poliwag => &POLIWAG,
        Species::Clefairy => &CLEFAIRY,
        Species::Jigglypuff => &JIGGLYPUFF,
        Species::Rattata => &RATTATA,
        Species::Raticate => &RATICATE,
        Species::Psyduck => &PSYDUCK,
        Species::Golduck => &GOLDUCK,
        Species::Zubat => &ZUBAT,
        Species::Golbat => &GOLBAT,
        Species::Crobat => &CROBAT,
        Species::Butterfree => &BUTTERFREE,
        Species::Beedrill => &BEEDRILL,
        Species::Hoothoot => &HOOTHOOT,
        Species::Sentret => &SENTRET,
        Species::Staryu => &STARYU,
        Species::Voltorb => &VOLTORB,
        Species::Onix => &ONIX,
        Species::Diglett => &DIGLETT,
        Species::Slowpoke => &SLOWPOKE,
        Species::Entei => &ENTEI,
        Species::Raikou => &RAIKOU,
        Species::Ledyba => &LEDYBA,
        Species::Ledian => &LEDIAN,
        Species::Ariados => &ARIADOS,
        Species::Quagsire => &QUAGSIRE,
        Species::Hoppip => &HOPPIP,
        Species::Skiploom => &SKIPLOOM,
        Species::Bellossom => &BELLOSSOM,
        Species::Qwilfish => &QWILFISH,
        Species::Wobbuffet => &WOBBUFFET,
        Species::Dunsparce => &DUNSPARCE,
        Species::Shuckle => &SHUCKLE,
        Species::Gligar => &GLIGAR,
        Species::Yanma => &YANMA,
        Species::Murkrow => &MURKROW,
        Species::Misdreavus => &MISDREAVUS,
        Species::Teddiursa => &TEDDIURSA,
        Species::Ursaring => &URSARING,
        Species::Snubbull => &SNUBBULL,
        Species::Granbull => &GRANBULL,
        Species::Houndour => &HOUNDOUR,
        Species::Houndoom => &HOUNDOOM,
        Species::Miltank => &MILTANK,
        Species::Tauros => &TAUROS,
        Species::Delibird => &DELIBIRD,
        Species::Stantler => &STANTLER,
        Species::Girafarig => &GIRAFARIG,
        Species::Phanpy => &PHANPY,
        Species::Marill => &MARILL,
        Species::Natu => &NATU,
        Species::Togepi => &TOGEPI,
        Species::Togetic => &TOGETIC,
        Species::Remoraid => &REMORAID,
        Species::Octillery => &OCTILLERY,
        Species::Swinub => &SWINUB,
        Species::Piloswine => &PILOSWINE,
        Species::Sneasel => &SNEASEL,
        Species::Skarmory => &SKARMORY,
        Species::Smeargle => &SMEARGLE,
        Species::Moltres => &MOLTRES,
        Species::Zapdos => &ZAPDOS,
        Species::Articuno => &ARTICUNO,
        Species::Lugia => &LUGIA,
        Species::Corsola => &CORSOLA,
        Species::Mantine => &MANTINE,
        Species::Porygon => &PORYGON,
        Species::Porygon2 => &PORYGON2,
        Species::Chinchou => &CHINCHOU,
        Species::Lanturn => &LANTURN,
        Species::Slugma => &SLUGMA,
        Species::Magcargo => &MAGCARGO,
        Species::Hitmonlee => &HITMONLEE,
        Species::Hitmonchan => &HITMONCHAN,
        Species::Hitmontop => &HITMONTOP,
        Species::Tyrogue => &TYROGUE,
        Species::Smoochum => &SMOOCHUM,
        Species::Elekid => &ELEKID,
        Species::Magby => &MAGBY,
        Species::Pichu => &PICHU,
        Species::Cleffa => &CLEFFA,
        Species::Igglybuff => &IGGLYBUFF,
        Species::Mareep => &MAREEP,
        Species::Flaaffy => &FLAAFFY,
        Species::Ampharos => &AMPHAROS,
        Species::Xatu => &XATU,
        Species::Azumarill => &AZUMARILL,
        Species::Donphan => &DONPHAN,
        Species::Sunkern => &SUNKERN,
        Species::Sunflora => &SUNFLORA,
        Species::Aipom => &AIPOM,
        Species::Jumpluff => &JUMPLUFF,
        Species::Spinarak => &SPINARAK,
        Species::Noctowl => &NOCTOWL,
        Species::Furret => &FURRET,
        Species::Suicune => &SUICUNE,
        Species::Unown(form) => &UNOWN[usize::from(form)],


    };
    cache.get_or_init(|| {
        SpeciesRig::parse(species, &candidate(species))
            .unwrap_or_else(|error| panic!("{} authored skin parses: {error}", species.name()))
    })
}

fn storage(rig: &SpeciesRig) -> (Vec<Transform>, Vec<Mat4>) {
    (
        vec![Transform::IDENTITY; rig.joints.len()],
        vec![Mat4::IDENTITY; rig.joints.len()],
    )
}

fn assert_vec_close(a: Vec3, b: Vec3, epsilon: f32) {
    assert!(
        (a - b).abs().max_element() <= epsilon,
        "{a:?} differs from {b:?}"
    );
}

fn assert_bits(a: impl Iterator<Item = f32>, b: impl Iterator<Item = f32>) {
    assert_eq!(
        a.map(f32::to_bits).collect::<Vec<_>>(),
        b.map(f32::to_bits).collect::<Vec<_>>()
    );
}

#[test]
fn species_schema_retains_anatomy_skin_order_and_physical_size() {
    for (species, vertices, triangles, height) in [
        (Species::Chikorita, 7463, 3220, 1.0_f32),
        (Species::Bayleef, 9626, 3836, 1.15_f32),
        (Species::Meganium, 11353, 4520, 1.48_f32),
        (Species::Cyndaquil, 16314, 8138, 1.25_f32),
        (Species::Totodile, 7531, 7078, 1.02_f32),
        (Species::Gengar, 6952, 5560, 1.05_f32),
        (Species::Spearow, 2139, 1929, 0.75_f32),
        (Species::Wooper, 3991, 5532, 0.70_f32),
        (Species::Gyarados, 9187, 11666, 1.40_f32),
        (Species::Sudowoodo, 9089, 11376, 1.04_f32),
        (Species::Lapras, 19471, 23332, 1.50_f32),
        (Species::Snorlax, 13680, 21820, 1.40_f32),
        (Species::Pikachu, 11568, 13876, 1.00_f32),
        (Species::Bulbasaur, 19587, 31896, 0.95_f32),
        (Species::Charmander, 13250, 21086, 1.00_f32),
        (Species::Squirtle, 17298, 26948, 0.96_f32),
        (Species::Geodude, 12179, 15352, 0.76_f32),
        (Species::Pidgey, 8404, 13064, 0.82_f32),
        (Species::Oddish, 8865, 14620, 1.00_f32),
        (Species::Poliwag, 15265, 25532, 0.90_f32),
        (Species::Clefairy, 15257, 24216, 0.82_f32),
        (Species::Jigglypuff, 13706, 22956, 0.82_f32),
        (Species::Rattata, 11494, 20432, 0.82_f32),
        (Species::Raticate, 5642, 7764, 0.96_f32),
        (Species::Psyduck, 3754, 3828, 1.03_f32),
        (Species::Golduck, 4974, 4460, 1.03_f32),
        (Species::Zubat, 2368, 3352, 1.0_f32),
        (Species::Golbat, 3545, 4784, 1.0_f32),
        (Species::Crobat, 4153, 4116, 1.0_f32),
        (Species::Butterfree, 5428, 9420, 1.04_f32),
        (Species::Beedrill, 4304, 7304, 1.04_f32),
        (Species::Hoothoot, 16597, 25884, 1.0_f32),
        (Species::Sentret, 9293, 16152, 1.0_f32),
        (Species::Staryu, 1982, 2852, 1.0_f32),
        (Species::Voltorb, 4660, 8248, 0.82_f32),
        (Species::Onix, 12083, 9848, 1.72_f32),
        (Species::Diglett, 10646, 14064, 0.82_f32),
        (Species::Slowpoke, 19574, 32984, 1.0_f32),
        (Species::Entei, 15542, 26368, 1.42_f32),
        (Species::Raikou, 18482, 28632, 1.30_f32),
        (Species::Suicune, 10500, 17216, 1.60_f32),
        (Species::Furret, 11866, 10372, 1.20_f32),
        (Species::Noctowl, 14815, 22660, 1.30_f32),
        (Species::Ledyba, 18127, 28940, 0.95_f32),
        (Species::Ledian, 19044, 30356, 1.05_f32),
        (Species::Ariados, 6498, 6768, 0.90_f32),
        (Species::Quagsire, 4380, 6836, 1.13_f32),
        (Species::Hoppip, 5963, 8380, 0.85_f32),
        (Species::Bellossom, 18524, 26980, 0.95_f32),
        (Species::Qwilfish, 9014, 13644, 0.91_f32),
        (Species::Wobbuffet, 11791, 19260, 1.14_f32),
        (Species::Dunsparce, 11833, 12248, 0.85_f32),
        (Species::Shuckle, 14875, 19836, 0.91_f32),
        (Species::Gligar, 17422, 26768, 1.06_f32),
        (Species::Yanma, 19976, 26452, 0.95_f32),
        (Species::Murkrow, 13359, 16616, 0.78_f32),
        (Species::Misdreavus, 14175, 23794, 1.02_f32),
        (Species::Teddiursa, 19943, 23880, 1.14_f32),
        (Species::Ursaring, 19016, 27116, 1.14_f32),
        (Species::Snubbull, 16555, 19516, 1.06_f32),
        (Species::Granbull, 14140, 19048, 1.06_f32),
        (Species::Houndour, 11024, 14248, 1.20_f32),
        (Species::Houndoom, 15865, 22736, 1.20_f32),
        (Species::Miltank, 17281, 17976, 1.12_f32),
        (Species::Tauros, 14025, 16504, 1.28_f32),
        (Species::Delibird, 9916, 17808, 1.10_f32),
        (Species::Stantler, 15993, 20548, 1.28_f32),
        (Species::Girafarig, 14025, 22432, 1.28_f32),
        (Species::Phanpy, 18472, 31284, 1.02_f32),
        (Species::Marill, 3437, 5648, 1.02_f32),
        (Species::Natu, 18853, 28820, 1.04_f32),
        (Species::Togepi, 17825, 28896, 0.94_f32),
        (Species::Togetic, 16755, 26728, 0.94_f32),
        (Species::Remoraid, 6822, 7724, 1.00_f32),
        (Species::Octillery, 19798, 21740, 1.00_f32),
        (Species::Swinub, 7044, 8496, 0.98_f32),
        (Species::Piloswine, 5062, 9432, 0.98_f32),
        (Species::Sneasel, 8779, 9936, 1.11_f32),
        (Species::Skarmory, 3635, 4288, 1.21_f32),
        (Species::Smeargle, 16875, 21004, 1.14_f32),
        (Species::Moltres, 19671, 26024, 1.28_f32),
        (Species::Zapdos, 18495, 28584, 1.28_f32),
        (Species::Articuno, 19074, 26704, 1.28_f32),
        (Species::Lugia, 19823, 34796, 1.45_f32),
        (Species::Corsola, 7359, 14180, 0.93_f32),
        (Species::Mantine, 19485, 27416, 1.02_f32),
        (Species::Porygon, 4789, 2008, 1.05_f32),
        (Species::Porygon2, 14189, 23912, 1.05_f32),
        (Species::Chinchou, 12754, 11116, 0.95_f32),
        (Species::Lanturn, 17971, 15572, 0.95_f32),
        (Species::Slugma, 2290, 4108, 0.98_f32),
        (Species::Magcargo, 5554, 10276, 0.98_f32),
        (Species::Hitmonlee, 19747, 35276, 1.24_f32),
        (Species::Hitmonchan, 18362, 30096, 1.30_f32),
        (Species::Hitmontop, 17394, 29828, 1.25_f32),
        (Species::Tyrogue, 19924, 33424, 0.86_f32),
        (Species::Smoochum, 19907, 34392, 0.78_f32),
        (Species::Elekid, 19466, 33128, 0.91_f32),
        (Species::Magby, 19764, 35032, 0.77_f32),
        (Species::Pichu, 9462, 9104, 0.62_f32),
        (Species::Cleffa, 14716, 24376, 0.6_f32),
        (Species::Igglybuff, 16964, 28684, 0.58_f32),
        (Species::Mareep, 6382, 11840, 0.85_f32),
        (Species::Flaaffy, 10302, 16636, 1.0_f32),
        (Species::Ampharos, 2402, 4700, 1.35_f32),
        (Species::Xatu, 15289, 24060, 1.04_f32),
        (Species::Azumarill, 4873, 7448, 1.02_f32),
        (Species::Donphan, 17395, 29596, 1.02_f32),
        (Species::Sunkern, 11694, 16524, 0.95_f32),
        (Species::Sunflora, 9135, 13696, 0.95_f32),
        (Species::Aipom, 17804, 30016, 1.02_f32),
        (Species::Skiploom, 5182, 10028, 0.85_f32),
        (Species::Jumpluff, 6247, 10560, 0.85_f32),
        (Species::Spinarak, 7932, 7684, 0.90_f32),
        (Species::Unown(0), 4226, 7400, 1.0_f32),
        (Species::Unown(1), 4386, 7856, 1.0_f32),
        (Species::Unown(2), 4459, 7956, 1.0_f32),
        (Species::Unown(3), 4315, 7700, 1.0_f32),
        (Species::Unown(4), 4394, 7656, 1.0_f32),
        (Species::Unown(5), 4162, 7340, 1.0_f32),
        (Species::Unown(6), 4501, 7984, 1.0_f32),
        (Species::Unown(7), 4610, 8168, 1.0_f32),
        (Species::Unown(8), 4002, 7088, 1.0_f32),
        (Species::Unown(9), 4219, 7472, 1.0_f32),
        (Species::Unown(10), 4226, 7400, 1.0_f32),
        (Species::Unown(11), 4084, 7216, 1.0_f32),
        (Species::Unown(12), 4562, 8140, 1.0_f32),
        (Species::Unown(13), 4246, 7344, 1.0_f32),
        (Species::Unown(14), 4402, 7956, 1.0_f32),
        (Species::Unown(15), 4002, 7088, 1.0_f32),
        (Species::Unown(16), 4073, 7216, 1.0_f32),
        (Species::Unown(17), 4002, 7088, 1.0_f32),
        (Species::Unown(18), 4506, 7856, 1.0_f32),
        (Species::Unown(19), 4002, 7088, 1.0_f32),
        (Species::Unown(20), 4434, 7884, 1.0_f32),
        (Species::Unown(21), 3970, 7092, 1.0_f32),
        (Species::Unown(22), 4242, 7500, 1.0_f32),
        (Species::Unown(23), 4226, 7400, 1.0_f32),
        (Species::Unown(24), 4226, 7400, 1.0_f32),
        (Species::Unown(25), 4150, 7344, 1.0_f32),


    ] {
        let rig = fixture(species);
        assert_eq!(rig.species, species);
        assert_eq!(rig.neutral.positions.len(), vertices);
        assert_eq!(rig.neutral.indices.len() / 3, triangles);
        assert_eq!(rig.anatomy.len(), species.part_count());
        assert_eq!(rig.joint_indices.len(), vertices);
        assert_eq!(rig.joint_weights.len(), vertices);
        assert_eq!(rig.source_normals.len(), vertices);
        assert_eq!(rig.neutral_bounds.0.y, 0.0);
        assert!((rig.neutral_bounds.1.y - height).abs() < 0.000001);
        assert_eq!(
            rig.joints.iter().map(|j| j.parent).collect::<Vec<_>>(),
            species.parents()
        );
        for (joint, suffix) in rig.joints.iter().zip(species.joint_names()) {
            assert_eq!(joint.name, format!("{}/{suffix}", species.name()));
        }
        let mut cursor = (0, 0);
        for part in &rig.anatomy {
            assert!(!part.name.is_empty());
            assert_eq!((part.vertices.start, part.indices.start), cursor);
            cursor = (part.vertices.end, part.indices.end);
            let material = &rig.materials[part.material];
            assert!(!material.name.is_empty());
            assert_eq!(material.alpha_mode, gltf::material::AlphaMode::Opaque);
            assert!(rig.neutral.colors[part.vertices.clone()]
                .iter()
                .all(|c| *c == material.color));
        }
        assert_eq!(cursor, (vertices, triangles * 3));
        for (&source, &normal) in rig.source_normals.iter().zip(&rig.neutral.normals) {
            assert_bits(
                Vec3::from_array(source).normalize().to_array().into_iter(),
                normal.into_iter(),
            );
        }
    }
}

#[test]
fn spearow_lookup_keeps_canonical_geometry_and_authored_clip_data() {
    let rig = fixture(Species::Spearow);
    for name in ["SPEAROW", "spearow", "SpEaRoW"] {
        let registered = for_species(name).expect("Spearow uses the generic skin runtime");
        assert!(std::ptr::eq(registered, super::rig(Species::Spearow)));
        assert_eq!(registered.neutral, rig.neutral);
        assert_eq!(
            crate::battle_species_models::mesh(name).unwrap(),
            rig.neutral
        );
    }
    assert_eq!(rig.neutral_bounds.0, Vec3::new(-0.368, 0.0, -0.425));
    assert_eq!(rig.neutral_bounds.1, Vec3::new(0.368, 0.75, 0.425));

    let bytes = candidate(Species::Spearow);
    let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
    let binary = gltf.blob.as_deref().unwrap();
    for ((animation, kind), (duration, keys)) in
        gltf.animations()
            .zip(Clip::ALL)
            .zip([(2.8_f32, 73), (1.0, 65), (1.0, 65)])
    {
        let clip = rig.clip(kind);
        assert_eq!(clip.duration, duration);
        assert_eq!(clip.times.len(), keys);
        assert_eq!(clip.extras, animation.extras().as_ref().unwrap().get());
        for channel in animation.channels() {
            let reader = channel.reader(|_| Some(binary));
            assert_bits(reader.read_inputs().unwrap(), clip.times.iter().copied());
            let Some(gltf::animation::util::ReadOutputs::Rotations(rotations)) =
                reader.read_outputs()
            else {
                panic!("Spearow clip must retain rotation channels");
            };
            assert_bits(
                rotations.into_f32().flatten(),
                clip.rotations[channel.target().node().index() - 1]
                    .iter()
                    .flat_map(|q| q.to_array()),
            );
        }
    }
}

#[test]
fn spearow_each_joint_deforms_its_anatomy_while_feet_stay_planted() {
    let rig = fixture(Species::Spearow);
    let feet: Vec<_> = rig
        .anatomy
        .iter()
        .filter(|part| part.name.starts_with("Feet /"))
        .collect();
    assert_eq!(feet.len(), 4);
    let feet: Vec<_> = feet.iter().flat_map(|part| part.vertices.clone()).collect();
    for &vertex in &feet {
        assert_eq!(rig.joint_indices[vertex], [0; 4]);
        assert_eq!(rig.joint_weights[vertex], [1.0, 0.0, 0.0, 0.0]);
    }
    let bind: Vec<_> = rig.joints.iter().map(|joint| joint.bind).collect();
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        let mut peak_angles = vec![0.0_f32; rig.joints.len()];
        let mut peak_rotations = vec![Quat::IDENTITY; rig.joints.len()];
        let frames = (rig.clip(clip).duration * 60.0).ceil() as usize;
        for frame in 0..=frames {
            rig.sample_into(
                clip,
                rig.clip(clip).duration * frame as f32 / frames as f32,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            assert_eq!(pose[0], Transform::IDENTITY);
            for joint in 1..pose.len() {
                let angle = pose[joint].rotation.angle_between(bind[joint].rotation);
                if angle > peak_angles[joint] {
                    peak_angles[joint] = angle;
                    peak_rotations[joint] = pose[joint].rotation;
                }
            }
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &feet {
                assert_vec_close(
                    rig.skin_point(vertex, &matrices),
                    Vec3::from_array(rig.neutral.positions[vertex]),
                    0.000001,
                );
            }
        }
        for joint in 1..rig.joints.len() {
            assert!(
                peak_angles[joint] > 0.01,
                "{} lacks local articulation of {}",
                rig.clip(clip).name,
                rig.joints[joint].name
            );
            // Isolate this joint's own rotation. An ancestor or whole-body
            // motion cannot satisfy the anatomical displacement requirement.
            pose.copy_from_slice(&bind);
            pose[joint].rotation = peak_rotations[joint];
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            let displacement = (0..rig.neutral.positions.len())
                .filter(|&vertex| {
                    rig.joint_indices[vertex]
                        .iter()
                        .zip(rig.joint_weights[vertex])
                        .any(|(&id, weight)| id as usize == joint && weight > 0.0)
                })
                .map(|vertex| {
                    rig.skin_point(vertex, &matrices)
                        .distance(Vec3::from_array(rig.neutral.positions[vertex]))
                })
                .fold(0.0_f32, f32::max);
            assert!(
                displacement > 0.0005,
                "{} does not move the geometry owned by {}: {displacement}",
                rig.clip(clip).name,
                rig.joints[joint].name
            );
        }
    }
}

#[test]
fn spearow_pack_scale_and_contact_ignore_animation_envelope() {
    use crate::battle_layout::{
        model_scale, reference_span, BattleBody, BattleSceneLayout, WORLD_UNITS_PER_METER,
    };
    use bevy::prelude::Vec2;

    let rig = fixture(Species::Spearow);
    let (min, max) = rig.neutral_bounds;
    assert_eq!(reference_span("SPEAROW", min, max), 0.75);
    assert!(model_scale("SPEAROW", None, min, max).is_none());
    // The current pack supplies 0.3048 m. Also check another supplied value
    // so a hidden per-species constant cannot satisfy the physical-size test.
    for meters in [0.3048_f32, 0.6096] {
        let scale = model_scale("SPEAROW", Some(meters), min, max).unwrap();
        assert!(((max.y - min.y) * scale / WORLD_UNITS_PER_METER - meters).abs() < 0.000001);
        let neutral = BattleBody::modeled("SPEAROW", min, max, scale);
        let mut animated = neutral;
        animated.visual_bounds = Some(rig.animated_bounds);
        for viewport in [Vec2::new(1180.0, 812.0), Vec2::new(600.0, 1000.0)] {
            let before = BattleSceneLayout::for_bodies([Some(neutral); 2], viewport);
            let after = BattleSceneLayout::for_bodies([Some(animated); 2], viewport);
            for side in 0..2 {
                let pose = after.body_poses[side].unwrap();
                assert_eq!(pose.scale, Vec3::splat(scale));
                assert_eq!(pose.scale, before.body_poses[side].unwrap().scale);
                assert_eq!(
                    pose.translation.y,
                    before.body_poses[side].unwrap().translation.y
                );
                assert_eq!(after.hit_anchors[side].y, before.hit_anchors[side].y);
                let contact_y = rig
                    .neutral
                    .positions
                    .iter()
                    .map(|point| pose.transform_point(Vec3::from_array(*point)).y)
                    .fold(f32::INFINITY, f32::min);
                assert!((contact_y - after.origins[side].y).abs() < 0.000001);
            }
        }
    }
}

#[test]
fn grass_starter_attack_clips_brace_with_source_cue_onset() {
    for (species, last_joint) in [
        (Species::Chikorita, 2),
        (Species::Bayleef, 4),
        (Species::Meganium, 5),
    ] {
        let rig = fixture(species);
        let (mut pose, _) = storage(rig);
        let mut rotations = Vec::new();
        for step in 0..=100 {
            rig.sample_into(
                Clip::Attack,
                rig.clip(Clip::Attack).duration * step as f32 / 100.0,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            rotations.push(
                pose.iter()
                    .map(|joint| joint.rotation.x)
                    .collect::<Vec<_>>(),
            );
        }
        for joint in 1..=last_joint {
            let peak = rotations
                .iter()
                .map(|frame| frame[joint])
                .fold(0.0_f32, f32::max);
            for frame in &rotations[1..=25] {
                assert!(
                    frame[joint] > 0.0,
                    "{species:?} joint {joint} reverses at onset"
                );
            }
            assert!(rotations[10][joint] > peak * 0.40);
            assert!(rotations[20][joint] > peak * 0.90);
            assert!(rotations[80][joint] < peak * 0.25);
        }
    }
}

#[test]
fn weighted_bind_and_clip_endpoints_preserve_neutral_vertices() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            assert_eq!(
                rig.clip(clip).name,
                format!("{}.{}", species.name(), clip.suffix())
            );
            assert_eq!(rig.clip(clip).loop_suggested, clip == Clip::Idle);
            assert_eq!(rig.clip(clip).cue_relative, clip != Clip::Idle);
            assert!(!rig.clip(clip).extras.is_empty());
            for time in [
                -1.0,
                0.0,
                rig.clip(clip).duration,
                rig.clip(clip).duration + 1.0,
            ] {
                rig.sample_into(clip, time, Playback::Clamp, &mut pose)
                    .unwrap();
                assert_eq!(pose, rig.joints.iter().map(|j| j.bind).collect::<Vec<_>>());
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for m in &matrices {
                    assert!(m
                        .to_cols_array()
                        .iter()
                        .zip(Mat4::IDENTITY.to_cols_array())
                        .all(|(a, b)| (a - b).abs() < 0.000002));
                }
                for (i, p) in rig.neutral.positions.iter().enumerate() {
                    assert_vec_close(rig.skin_point(i, &matrices), Vec3::from_array(*p), 0.000002);
                }
            }
        }
    }
}

#[test]
fn authored_clips_are_finite_at_30_and_60_hz_and_do_articulate() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        let allocation = pose.as_ptr();
        for clip in Clip::ALL {
            let mut displaced = false;
            for hz in [30.0, 60.0] {
                let frames = (rig.clip(clip).duration * hz).ceil() as usize;
                for frame in 0..=frames {
                    let t = rig.clip(clip).duration * frame as f32 / frames as f32;
                    rig.sample_into(clip, t, Playback::Clamp, &mut pose)
                        .unwrap();
                    assert_eq!(pose.as_ptr(), allocation);
                    assert_eq!(pose[0], Transform::IDENTITY);
                    assert!(pose.iter().all(|p| p.translation.is_finite()
                        && p.rotation.is_finite()
                        && p.scale == Vec3::ONE));
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    for (i, p) in rig.neutral.positions.iter().enumerate() {
                        let posed = rig.skin_point(i, &matrices);
                        assert!(posed.is_finite());
                        displaced |= posed.distance(Vec3::from_array(*p)) > 0.001;
                    }
                }
            }
            assert!(
                displaced,
                "{} was flattened to static geometry",
                rig.clip(clip).name
            );
        }
        for t in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(rig
                .sample_into(Clip::Idle, t, Playback::Loop, &mut pose)
                .is_err());
        }
        assert!(rig
            .sample_into(Clip::Idle, 0.0, Playback::Loop, &mut [])
            .is_err());
    }
}

#[test]
fn idle_loop_is_closed_and_cue_clips_clamp_to_neutral() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        let duration = rig.clip(Clip::Idle).duration;
        for time in [-duration, 0.0, duration, duration * 2.0] {
            rig.sample_into(Clip::Idle, time, Playback::Loop, &mut pose)
                .unwrap();
            assert_eq!(pose, rig.joints.iter().map(|j| j.bind).collect::<Vec<_>>());
        }
        for time in [duration - 0.00001, duration + 0.00001] {
            rig.sample_into(Clip::Idle, time, Playback::Loop, &mut pose)
                .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for (i, p) in rig.neutral.positions.iter().enumerate() {
                assert_vec_close(rig.skin_point(i, &matrices), Vec3::from_array(*p), 0.00001);
            }
        }
    }
}

#[test]
fn animation_envelope_covers_dense_off_grid_samples_without_resizing() {
    for species in Species::ALL {
        let rig = fixture(species);
        let (mut pose, mut matrices) = storage(rig);
        let envelope_extent = rig.animated_bounds.1 - rig.animated_bounds.0;
        let mut sampled_bounds = rig.neutral_bounds;
        for clip in Clip::ALL {
            // Prime grid intentionally misses both the authored keys and the
            // 30 Hz envelope samples. Include every vertex and clip endpoints.
            for frame in 0..=251 {
                let time = rig.clip(clip).duration * frame as f32 / 251.0;
                rig.sample_into(clip, time, Playback::Clamp, &mut pose)
                    .unwrap();
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for i in 0..rig.neutral.positions.len() {
                    let point = rig.skin_point(i, &matrices);
                    sampled_bounds.0 = sampled_bounds.0.min(point);
                    sampled_bounds.1 = sampled_bounds.1.max(point);
                    assert!(
                        point.cmpge(rig.animated_bounds.0).all()
                            && point.cmple(rig.animated_bounds.1).all(),
                        "{} t={time} vertex={i} outside {:?}: {point:?}",
                        rig.clip(clip).name,
                        rig.animated_bounds
                    );
                }
            }
        }
        // Bound padding around actual animation, not motion relative to rest.
        // Gengar's larger pose legitimately grows depth; neutral bounds still
        // exclusively control physical scale and footing in separate tests.
        // Independent decoded sampling (including 2,100 blended poses) found
        // Gengar's conservative/animated extent ratios to be 1.052/1.067/1.092.
        let sampled_extent = sampled_bounds.1 - sampled_bounds.0;
        assert!(
            (envelope_extent / sampled_extent).max_element() < 1.10,
            "{} camera padding exceeds its budget: {sampled_extent:?} -> {envelope_extent:?}",
            rig.species.name()
        );
        for (from_clip, to_clip) in [
            (Clip::Idle, Clip::Attack),
            (Clip::Attack, Clip::Hit),
            (Clip::Hit, Clip::Idle),
        ] {
            for phase in [0.2, 0.43, 0.75] {
                let mut from = pose.clone();
                let mut target = pose.clone();
                rig.sample_into(
                    from_clip,
                    phase * rig.clip(from_clip).duration,
                    Playback::Clamp,
                    &mut from,
                )
                .unwrap();
                rig.sample_into(
                    to_clip,
                    (1.0 - phase) * rig.clip(to_clip).duration,
                    Playback::Clamp,
                    &mut target,
                )
                .unwrap();
                for blend in [0.25, 0.5, 0.75] {
                    for ((p, a), b) in pose.iter_mut().zip(&from).zip(&target) {
                        *p = *b;
                        p.rotation = a.rotation.slerp(b.rotation, blend).normalize();
                    }
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    for i in 0..rig.neutral.positions.len() {
                        let point = rig.skin_point(i, &matrices);
                        assert!(
                            point.cmpge(rig.animated_bounds.0).all()
                                && point.cmple(rig.animated_bounds.1).all(),
                            "{} blended vertex {i} outside {:?}: {point:?}",
                            rig.species.name(),
                            rig.animated_bounds
                        );
                    }
                }
            }
        }
    }
}

fn rewrite(bytes: &[u8], edit: impl FnOnce(&mut Value, &mut Vec<u8>)) -> Vec<u8> {
    let glb = gltf::binary::Glb::from_slice(bytes).unwrap();
    let mut doc = serde_json::from_slice(&glb.json).unwrap();
    let mut binary = glb.bin.unwrap().to_vec();
    edit(&mut doc, &mut binary);
    let mut json = serde_json::to_vec(&doc).unwrap();
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    while binary.len() % 4 != 0 {
        binary.push(0);
    }
    let mut out = Vec::new();
    out.extend(b"glTF");
    out.extend(2_u32.to_le_bytes());
    out.extend(((28 + json.len() + binary.len()) as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(json);
    out.extend((binary.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(binary);
    out
}

fn offset(doc: &Value, accessor: usize) -> usize {
    let a = &doc["accessors"][accessor];
    doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize]["byteOffset"]
        .as_u64()
        .unwrap_or(0) as usize
        + a["byteOffset"].as_u64().unwrap_or(0) as usize
}

#[test]
fn parser_rejects_malformed_or_unsupported_json_before_reading_binary() {
    let species = Species::Totodile;
    let bytes = candidate(species);
    for length in [0, 4, 12, 20, 27, bytes.len() - 1] {
        assert!(SpeciesRig::parse(species, &bytes[..length]).is_err());
    }
    type Edit = fn(&mut Value, &mut Vec<u8>);
    let edits: &[Edit] = &[
        |d, _| d["buffers"][0]["uri"] = json!("other.bin"),
        |d, _| d["buffers"][0]["byteLength"] = json!(999_999_999),
        |d, _| d["bufferViews"][0]["byteOffset"] = json!(999_999_996),
        |d, _| d["bufferViews"][0]["byteStride"] = json!(64),
        |d, _| d["accessors"][0]["count"] = json!(999_999),
        |d, _| d["accessors"][0]["byteOffset"] = json!(999_999_996),
        |d, _| d["accessors"][0]["normalized"] = json!(true),
        |d, _| d["nodes"][2]["children"] = json!([1]),
        |d, _| d["nodes"][0]["children"] = json!([1, 1, 6]),
        |d, _| d["nodes"][3]["name"] = json!("totodile/wrong_joint"),
        |d, _| d["nodes"][8]["translation"] = json!([1, 0, 0]),
        |d, _| d["nodes"][1]["scale"] = json!([2, 1, 1]),
        |d, _| d["skins"][0]["joints"] = json!([0, 2, 1, 3, 4, 5, 6, 7]),
        |d, _| d["meshes"][0]["primitives"][0]["mode"] = json!(1),
        |d, _| d["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"] = json!(1),
        |d, _| d["materials"][0]["pbrMetallicRoughness"]["metallicFactor"] = json!(1),
        |d, _| d["materials"][0]["extensions"] = json!({"KHR_materials_unlit": {}}),
        |d, _| d["animations"][0]["samplers"][0]["interpolation"] = json!("STEP"),
        |d, _| d["animations"][0]["channels"][0]["target"]["path"] = json!("translation"),
        |d, _| d["animations"][0]["channels"][0]["target"]["node"] = json!(0),
        |d, _| d["animations"][0]["extras"]["loopSuggested"] = json!(false),
        |d, _| d["scenes"][0]["nodes"] = json!([0]),
    ];
    for (i, edit) in edits.iter().enumerate() {
        assert!(
            SpeciesRig::parse(species, &rewrite(&bytes, edit)).is_err(),
            "malformed JSON case {i} accepted"
        );
    }
}

#[test]
fn parser_rejects_nonfinite_keys_bad_weights_invalid_binds_and_changed_geometry() {
    let species = Species::Totodile;
    let bytes = candidate(species);
    type Edit = fn(&mut Value, &mut Vec<u8>);
    let edits: &[Edit] = &[
        |d, b| {
            let o = offset(d, 0);
            b[o..o + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 0);
            b[o..o + 4].copy_from_slice(&2_f32.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 1);
            b[o..o + 4].copy_from_slice(&f32::INFINITY.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 1);
            b[o..o + 4].copy_from_slice(&0.123_f32.to_le_bytes());
        },
        |d, b| {
            let o = offset(d, 3);
            b[o] = 99;
        },
        |d, b| {
            let o = offset(d, 4);
            b[o..o + 4].fill(0);
        },
        |d, b| {
            let o = offset(d, 5);
            b[o..o + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        },
        |d, b| {
            let a = d["animations"][0]["samplers"][0]["input"].as_u64().unwrap() as usize;
            let o = offset(d, a) + 4;
            b[o..o + 4].copy_from_slice(&0_f32.to_le_bytes());
        },
        |d, b| {
            let a = d["animations"][0]["samplers"][0]["output"]
                .as_u64()
                .unwrap() as usize;
            let o = offset(d, a) + 16;
            b[o..o + 16].fill(0);
        },
        |d, b| {
            let a = d["animations"][0]["samplers"][0]["output"]
                .as_u64()
                .unwrap() as usize;
            let o = offset(d, a) + 16;
            b[o..o + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        },
    ];
    for (i, edit) in edits.iter().enumerate() {
        assert!(
            SpeciesRig::parse(species, &rewrite(&bytes, edit)).is_err(),
            "malformed binary case {i} accepted"
        );
    }
}

#[test]
#[ignore = "one-time migration audit: set CYNDAQUIL_MESH_SOURCE, TOTODILE_MESH_SOURCE and GENGAR_MESH_SOURCE to retired source JSON"]
fn exact_neutral_equivalence_to_surviving_source_json() {
    // These three assets migrated unchanged from JSON. Spearow's canonical
    // sculpture was authored directly as GLB and has no equivalent JSON asset.
    for species in [Species::Cyndaquil, Species::Totodile, Species::Gengar] {
        let rig = fixture(species);
        let path = std::env::var(format!("{}_MESH_SOURCE", species.name().to_uppercase()))
            .expect("source path required");
        let source: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let primitives = source["primitives"].as_array().unwrap();
        assert_eq!(primitives.len(), rig.anatomy.len());
        for (part, original) in rig.anatomy.iter().zip(primitives) {
            let floats = |name: &str| {
                original[name]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect::<Vec<_>>()
            };
            assert_bits(
                rig.neutral.positions[part.vertices.clone()]
                    .iter()
                    .flatten()
                    .copied(),
                floats("positions").into_iter(),
            );
            assert_bits(
                rig.source_normals[part.vertices.clone()]
                    .iter()
                    .flatten()
                    .copied(),
                floats("normals").into_iter(),
            );
            assert_bits(
                rig.materials[part.material].color.into_iter(),
                floats("base_color").into_iter(),
            );
            let expected: Vec<_> = original["indices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u32)
                .collect();
            let actual: Vec<_> = rig.neutral.indices[part.indices.clone()]
                .iter()
                .map(|i| i - part.vertices.start as u32)
                .collect();
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn chikorita_registry_shares_the_authored_skin_and_keeps_four_ground_contacts() {
    let rig = fixture(Species::Chikorita);
    for name in ["CHIKORITA", "chikorita", "ChIkOrItA"] {
        assert!(std::ptr::eq(
            for_species(name).unwrap(),
            super::rig(Species::Chikorita)
        ));
    }
    assert_eq!(
        crate::new_bark_actors::actor_props::mesh(
            crate::new_bark_actors::actor_props::PropKind::BattleChikorita,
        ),
        rig.neutral
    );
    let sole_vertices: Vec<_> = rig
        .neutral
        .positions
        .iter()
        .enumerate()
        .filter(|(_, point)| point[1] == 0.0)
        .collect();
    let quadrants: std::collections::HashSet<_> = sole_vertices
        .iter()
        .map(|(_, point)| (point[0] > 0.0, point[2] > -0.015))
        .collect();
    assert_eq!(
        quadrants.len(),
        4,
        "each authored leg must retain a grounded sole"
    );
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for sample in 0..=61 {
            rig.sample_into(
                clip,
                rig.clip(clip).duration * sample as f32 / 61.0,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for (index, point) in &sole_vertices {
                assert_vec_close(
                    rig.skin_point(*index, &matrices),
                    Vec3::from_array(**point),
                    0.000001,
                );
            }
        }
    }
    assert_eq!(rig.neutral_bounds.0.y, 0.0);
    assert_eq!(rig.neutral_bounds.1.y, 1.0);
}

#[test]
fn bayleef_registry_shares_the_authored_skin_and_keeps_four_ground_contacts() {
    let rig = fixture(Species::Bayleef);
    for name in ["BAYLEEF", "bayleef", "BaYlEeF"] {
        assert!(std::ptr::eq(
            for_species(name).unwrap(),
            super::rig(Species::Bayleef)
        ));
    }
    assert_eq!(
        crate::battle_species_models::mesh("BAYLEEF").unwrap(),
        rig.neutral
    );
    let sole_vertices: Vec<_> = rig
        .neutral
        .positions
        .iter()
        .enumerate()
        .filter(|(_, point)| point[1] == 0.0)
        .collect();
    let quadrants: std::collections::HashSet<_> = sole_vertices
        .iter()
        .map(|(_, point)| (point[0] > 0.0, point[2] > -0.020))
        .collect();
    assert_eq!(
        quadrants.len(),
        4,
        "each authored leg must retain a grounded sole"
    );
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for sample in 0..=61 {
            rig.sample_into(
                clip,
                rig.clip(clip).duration * sample as f32 / 61.0,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for (index, point) in &sole_vertices {
                assert_vec_close(
                    rig.skin_point(*index, &matrices),
                    Vec3::from_array(**point),
                    0.000001,
                );
            }
        }
    }
    assert_eq!(rig.neutral_bounds.0.y, 0.0);
    assert_eq!(rig.neutral_bounds.1.y, 1.15);
}

#[test]
fn meganium_registry_shares_the_authored_skin_and_keeps_four_ground_contacts() {
    let rig = fixture(Species::Meganium);
    for name in ["MEGANIUM", "meganium", "MeGaNiUm"] {
        assert!(std::ptr::eq(
            for_species(name).unwrap(),
            super::rig(Species::Meganium)
        ));
    }
    assert_eq!(
        crate::battle_species_models::mesh("MEGANIUM").unwrap(),
        rig.neutral
    );
    let sole_vertices: Vec<_> = rig
        .neutral
        .positions
        .iter()
        .enumerate()
        .filter(|(_, point)| point[1] == 0.0)
        .collect();
    let quadrants: std::collections::HashSet<_> = sole_vertices
        .iter()
        .map(|(_, point)| (point[0] > 0.0, point[2] > -0.020))
        .collect();
    assert_eq!(
        quadrants.len(),
        4,
        "each authored leg must retain a grounded sole"
    );
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for sample in 0..=61 {
            rig.sample_into(
                clip,
                rig.clip(clip).duration * sample as f32 / 61.0,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for (index, point) in &sole_vertices {
                assert_vec_close(
                    rig.skin_point(*index, &matrices),
                    Vec3::from_array(**point),
                    0.000001,
                );
            }
        }
    }
    assert_eq!(rig.neutral_bounds.0.y, 0.0);
    assert_eq!(rig.neutral_bounds.1.y, 1.48);
}

#[test]
fn wooper_registry_shares_skin_and_preserves_planted_feet() {
    let rig = fixture(Species::Wooper);
    for name in ["WOOPER", "wooper", "WoOpEr"] {
        let registered = super::for_species(name).unwrap();
        assert!(std::ptr::eq(registered, super::rig(Species::Wooper)));
        assert_bits(
            crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),
            rig.neutral.positions.iter().flatten().copied(),
        );
    }
    let feet: Vec<_> = rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /")).collect();
    assert_eq!(feet.len(), 2);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for foot in &feet {
                for vertex in foot.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn gyarados_registry_shares_skin_and_keeps_coil_contacts() {
    let rig = fixture(Species::Gyarados);
    for name in ["GYARADOS", "gyarados", "GyArAdOs"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Gyarados)));
        assert_bits(
            crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),
            rig.neutral.positions.iter().flatten().copied(),
        );
    }
    let body = &rig.anatomy[0];
    let contacts: Vec<_> = body.vertices.clone().filter(|&i| rig.neutral.positions[i][1] < 0.035).collect();
    assert!(contacts.len() > 5);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &contacts {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }
        }
    }
}

#[test]
fn sudowoodo_registry_shares_skin_and_keeps_root_pads_planted() {
    let rig = fixture(Species::Sudowoodo);
    for name in ["SUDOWOODO", "sudowoodo", "SuDoWoOdO"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Sudowoodo)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let feet: Vec<_> = rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /")).collect();
    assert_eq!(feet.len(), 2);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for foot in &feet {
                for vertex in foot.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn lapras_registry_shares_skin_and_keeps_shell_seated() {
    let rig = fixture(Species::Lapras);
    for name in ["LAPRAS", "lapras", "LaPrAs"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Lapras)));
        let mesh = crate::battle_species_models::mesh(name).unwrap();
        assert_eq!(mesh.positions, rig.neutral.positions);
    }
    let (mut local, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for step in 0..=32 {
            rig.sample_into(clip, rig.clip(clip).duration * step as f32 / 32.0, Playback::Clamp, &mut local).unwrap();
            rig.skin_matrices_into(&local, &mut matrices).unwrap();
            for (index, part) in rig.anatomy.iter().enumerate() {
                if part.name.starts_with("Shell") {
                    for vertex in part.vertices.clone() {
                        let ids = rig.joint_indices[vertex];
                        assert_eq!(ids, [0; 4], "shell part {index} must remain seated");
                        assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                    }
                }
            }
        }
    }
}

#[test]
fn snorlax_registry_shares_skin_and_keeps_soles_planted() {
    let rig = fixture(Species::Snorlax);
    for name in ["SNORLAX", "snorlax", "SnOrLaX"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Snorlax)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn pikachu_registry_shares_skin_and_keeps_mouse_pads_planted() {
    let rig = fixture(Species::Pikachu);
    for name in ["PIKACHU", "pikachu", "PiKaChU"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Pikachu)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /") || part.name.starts_with("Toe crease /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn bulbasaur_registry_shares_skin_and_keeps_four_pads_and_claws_planted() {
    let rig = fixture(Species::Bulbasaur);
    for name in ["BULBASAUR", "bulbasaur", "BuLbAsAuR"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Bulbasaur)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| ["Forefoot /", "Hindfoot /", "Foreclaw /", "Hindclaw /"].iter().any(|prefix| part.name.starts_with(prefix))) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn charmander_registry_shares_skin_and_keeps_pads_and_claws_planted() {
    let rig = fixture(Species::Charmander);
    for name in ["CHARMANDER", "charmander", "ChArMaNdEr"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Charmander)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn squirtle_registry_shares_skin_and_keeps_pads_and_toes_planted() {
    let rig = fixture(Species::Squirtle);
    for name in ["SQUIRTLE", "squirtle", "SqUiRtLe"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Squirtle)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /") || part.name.starts_with("Toe /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn geodude_registry_shares_skin_and_keeps_lowest_boulder_contact_fixed() {
    let rig = fixture(Species::Geodude);
    for name in ["GEODUDE", "geodude", "GeOdUdE"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Geodude)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for (vertex, point) in rig.neutral.positions.iter().enumerate().filter(|(_, p)| p[1] <= 0.035) {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(*point), 0.000001);
            }
        }
    }
}

#[test]
fn pidgey_registry_shares_skin_and_keeps_pads_and_toes_planted() {
    let rig = fixture(Species::Pidgey);
    for name in ["PIDGEY", "pidgey", "PiDgEy"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Pidgey)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /") || part.name.starts_with("Toe /") || part.name.starts_with("Rear toe /") || part.name.starts_with("Claw /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn oddish_registry_shares_skin_and_keeps_root_feet_planted() {
    let rig = fixture(Species::Oddish);
    for name in ["ODDISH", "oddish", "OdDiSh"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Oddish)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn poliwag_registry_shares_skin_and_keeps_paddle_feet_planted() {
    let rig = fixture(Species::Poliwag);
    for name in ["POLIWAG", "poliwag", "PoLiWaG"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Poliwag)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn clefairy_registry_shares_icon_skin_and_keeps_feet_planted() {
    let rig = fixture(Species::Clefairy);
    for name in ["CLEFAIRY", "clefairy", "ClEfAiRy"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Clefairy)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let icon = crate::new_bark_actors::actor_props::mesh(crate::new_bark_actors::actor_props::PropKind::IconClefairy);
    assert_bits(icon.positions.iter().flatten().copied(), rig.neutral.positions.iter().flatten().copied());
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /") || part.name.starts_with("Toe /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn jigglypuff_registry_shares_icon_skin_and_keeps_feet_planted() {
    let rig = fixture(Species::Jigglypuff);
    for name in ["JIGGLYPUFF", "jigglypuff", "JiGgLyPuFf"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Jigglypuff)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let icon = crate::new_bark_actors::actor_props::mesh(crate::new_bark_actors::actor_props::PropKind::IconJigglypuff);
    assert_bits(icon.positions.iter().flatten().copied(), rig.neutral.positions.iter().flatten().copied());
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Foot /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn rattata_registry_shares_skin_and_keeps_all_four_paws_planted() {
    let rig = fixture(Species::Rattata);
    for name in ["RATTATA", "rattata", "RaTtAtA"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Rattata)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Paw /") || part.name.starts_with("Toes /") || part.name.starts_with("Leg /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn raticate_registry_preserves_sculpture_and_keeps_splayed_feet_planted() {
    let rig = fixture(Species::Raticate);
    for name in ["RATICATE", "raticate", "RaTiCaTe"] {
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(Species::Raticate)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|part| part.name.starts_with("Long splayed rat foot") || part.name.starts_with("Long rat toe") || part.name.starts_with("Rat toe claw")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn duck_registry_preserves_distinct_anatomy_and_planted_webs() {
    for species in [Species::Psyduck, Species::Golduck] {
        let rig = fixture(species);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(), super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(species.name()).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            for frame in 0..=60 {
                rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for part in rig.anatomy.iter().filter(|p| p.name.starts_with("Leg /") || p.name.starts_with("Foot /") || p.name.starts_with("Foot membrane /") || p.name.starts_with("Foot claw /")) {
                    for vertex in part.vertices.clone() {
                        assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                    }
                }
            }
        }
    }
}

#[test]
fn bat_registry_preserves_hollow_faces_and_articulates_all_wing_pairs() {
    for species in [Species::Zubat, Species::Golbat, Species::Crobat] {
        let rig = fixture(species);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(), super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(species.name()).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p| p.name.starts_with("Wing /")).count(), if species == Species::Crobat {4} else {2});
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            rig.sample_into(clip, rig.clip(clip).duration * 0.24, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|p| p.name.starts_with("Wing /")) {
                assert!(part.vertices.clone().any(|vertex| rig.skin_point(vertex, &matrices).distance(Vec3::from_array(rig.neutral.positions[vertex])) > 0.001));
            }
        }
    }
}

#[test]
fn wing_insect_registry_preserves_compound_faces_and_four_articulated_wings() {
    for species in [Species::Butterfree, Species::Beedrill] {
        let rig = fixture(species);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(), super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(species.name()).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p| p.name.starts_with("Wing /")).count(), 8);
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            rig.sample_into(clip, rig.clip(clip).duration * 0.24, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|p| p.name.starts_with("Wing /")) {
                assert!(part.vertices.clone().any(|vertex| rig.skin_point(vertex, &matrices).distance(Vec3::from_array(rig.neutral.positions[vertex])) > 0.001));
            }
        }
    }
}

#[test]
fn hoothoot_registry_keeps_the_single_displayed_foot_and_fitted_owl_face() {
    let rig = fixture(Species::Hoothoot);
    assert!(std::ptr::eq(super::for_species("HOOTHOOT").unwrap(), super::rig(Species::Hoothoot)));
    assert_bits(crate::battle_species_models::mesh("HOOTHOOT").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in rig.anatomy.iter().filter(|p| p.name.starts_with("Leg /") || p.name.starts_with("Foot /") || p.name.starts_with("Toe /") || p.name.starts_with("Claw /") || p.name.starts_with("Rear toe /")) {
                for vertex in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
                }
            }
        }
    }
}

#[test]
fn sentret_registry_preserves_the_grounded_tail_support_and_belly_annulus() {
    let rig = fixture(Species::Sentret);
    assert!(std::ptr::eq(super::for_species("SENTRET").unwrap(), super::rig(Species::Sentret)));
    assert_bits(crate::battle_species_models::mesh("SENTRET").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let contacts: Vec<_> = rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1] < 0.002).then_some(i)).collect();
    assert!(!contacts.is_empty());
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &contacts {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }
        }
    }
}

#[test]
fn staryu_registry_preserves_two_lower_tips_and_radial_arm_chains() {
    let rig = fixture(Species::Staryu);
    assert!(std::ptr::eq(super::for_species("STARYU").unwrap(), super::rig(Species::Staryu)));
    assert_bits(crate::battle_species_models::mesh("STARYU").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let contacts: Vec<_> = rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1] < 0.001).then_some(i)).collect();
    assert_eq!(contacts.len(), 2);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &contacts {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }
        }
    }
}

#[test]
fn voltorb_registry_preserves_the_grounded_lower_shell_band_and_fitted_face() {
    let rig = fixture(Species::Voltorb);
    assert!(std::ptr::eq(super::for_species("VOLTORB").unwrap(), super::rig(Species::Voltorb)));
    assert_bits(crate::battle_species_models::mesh("VOLTORB").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let contacts: Vec<_> = rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1] < 0.001).then_some(i)).collect();
    assert!(contacts.len() >= 64);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &contacts {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }
        }
    }
}

#[test]
fn onix_registry_preserves_three_tail_supports_and_the_connected_chain() {
    let rig = fixture(Species::Onix);
    assert!(std::ptr::eq(super::for_species("ONIX").unwrap(), super::rig(Species::Onix)));
    assert_bits(crate::battle_species_models::mesh("ONIX").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let contacts: Vec<_> = rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1] < 0.003).then_some(i)).collect();
    assert!(contacts.len() >= 64);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for &vertex in &contacts {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }
        }
    }
}

#[test]
fn diglett_registry_keeps_soil_fixed_while_the_capsule_and_face_articulate() {
    let rig = fixture(Species::Diglett);
    assert!(std::ptr::eq(super::for_species("DIGLETT").unwrap(), super::rig(Species::Diglett)));
    assert_bits(crate::battle_species_models::mesh("DIGLETT").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let soil: Vec<_> = rig.anatomy.iter().filter(|part| part.name.starts_with("Soil /")).collect();
    assert_eq!(soil.len(), 8);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &soil { for vertex in part.vertices.clone() {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }}
        }
    }
}

#[test]
fn slowpoke_registry_articulates_head_and_tail_with_four_planted_feet() {
    let rig = fixture(Species::Slowpoke);
    assert!(std::ptr::eq(super::for_species("SLOWPOKE").unwrap(), super::rig(Species::Slowpoke)));
    assert_bits(crate::battle_species_models::mesh("SLOWPOKE").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let supports: Vec<_> = rig.anatomy.iter().filter(|p| p.name.starts_with("Foot /") || p.name.starts_with("Toe /")).collect();
    assert_eq!(supports.len(), 16);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &supports { for vertex in part.vertices.clone() {
                assert_vec_close(rig.skin_point(vertex, &matrices), Vec3::from_array(rig.neutral.positions[vertex]), 0.000001);
            }}
            if frame == 15 {
                for prefix in ["Head /", "Tail tip /"] {
                    let part = rig.anatomy.iter().find(|p| p.name.starts_with(prefix)).unwrap();
                    assert!(part.vertices.clone().any(|v| rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) > 0.001));
                }
            }
        }
    }
}

#[test]
fn unown_registry_preserves_all_forms_and_coherent_eye_owners() {
    assert!(super::for_species("UNOWN").is_none(), "generic species cannot choose a DV form");
    for unknown in ["UNOWN_!", "UNOWN_AA", "UNOWN_UNRESOLVED"] {
        assert!(super::for_species(unknown).is_none());
    }
    let mut digests = std::collections::HashSet::new();
    for form in 0..26 {
        let species = Species::Unown(form);
        let rig = fixture(species);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(), super::rig(species)));
        assert!(digests.insert(species.neutral_digest()));
        assert_bits(crate::battle_species_models::mesh(species.name()).unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.joints.len(), 3);
        for part in rig.anatomy.iter().take(4) { for vertex in part.vertices.clone() {
            assert_eq!(rig.joint_indices[vertex], [1,0,0,0]);
        }}
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            rig.sample_into(clip, rig.clip(clip).duration * 0.28, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for vertex in 0..rig.neutral.positions.len() {
                assert!((rig.skin_point(vertex, &matrices).y - rig.neutral.positions[vertex][1]).abs() < 0.000001);
            }
        }
    }
}

#[test]
fn unown_o_rounded_ring_keeps_a_tight_continuous_animation_envelope() {
    let rig = fixture(Species::Unown(14));
    let (mut pose, mut matrices) = storage(rig);
    let mut sampled = rig.neutral_bounds;
    for clip in Clip::ALL {
        for step in 0..=513 {
            let time = rig.clip(clip).duration * ((step as f32 + 0.317) / 514.0);
            rig.sample_into(clip, time, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for i in 0..rig.neutral.positions.len() {
                let point = rig.skin_point(i, &matrices);
                sampled.0 = sampled.0.min(point);sampled.1 = sampled.1.max(point);
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all());
            }
        }
    }
    assert!(((rig.animated_bounds.1-rig.animated_bounds.0)/(sampled.1-sampled.0)).max_element() < 1.10);
    for (from_clip,to_clip) in [(Clip::Idle,Clip::Attack),(Clip::Attack,Clip::Hit),(Clip::Hit,Clip::Idle)] {
        for phase in [0.2,0.43,0.75] {
            let mut from=pose.clone();let mut target=pose.clone();
            rig.sample_into(from_clip,phase*rig.clip(from_clip).duration,Playback::Clamp,&mut from).unwrap();
            rig.sample_into(to_clip,(1.-phase)*rig.clip(to_clip).duration,Playback::Clamp,&mut target).unwrap();
            for blend in [0.25,0.5,0.75] {
                for ((p,a),b) in pose.iter_mut().zip(&from).zip(&target) {*p=*b;p.rotation=a.rotation.slerp(b.rotation,blend).normalize();}
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for i in 0..rig.neutral.positions.len() {
                    let point=rig.skin_point(i,&matrices);
                    assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all());
                }
            }
        }
    }
}

#[test]
fn entei_registry_keeps_paws_and_cuffs_planted_while_the_mane_and_smoke_move() {
    let rig=fixture(Species::Entei);
    assert!(std::ptr::eq(super::for_species("ENTEI").unwrap(),super::rig(Species::Entei)));
    assert_bits(crate::battle_species_models::mesh("ENTEI").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=rig.anatomy.iter().filter(|p| p.name.starts_with("Foot /") || p.name.starts_with("Toe /") || p.name.starts_with("Cuff /")).collect();
    assert_eq!(supports.len(),20);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if frame==17 {
                for prefix in ["Head /","Mane /","Smoke /","Tail /"] {
                    let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
                    assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.001));
                }
            }
        }
    }
}

#[test]
fn raikou_registry_keeps_paws_planted_while_the_storm_mane_and_tail_move() {
    let rig=fixture(Species::Raikou);
    assert!(std::ptr::eq(super::for_species("RAIKOU").unwrap(),super::rig(Species::Raikou)));
    assert_bits(crate::battle_species_models::mesh("RAIKOU").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=rig.anatomy.iter().filter(|p| p.name.starts_with("Foot /") || p.name.starts_with("Toe /")).collect();
    assert_eq!(supports.len(),16);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if frame==17 {
                for prefix in ["Head /","Storm mane /","Storm curl /","Tail /"] {
                    let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
                    assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.001));
                }
            }
        }
    }
}

#[test]
fn suicune_registry_keeps_paws_planted_while_the_crest_mane_and_ribbons_move() {
    let rig=fixture(Species::Suicune);
    assert!(std::ptr::eq(super::for_species("SUICUNE").unwrap(),super::rig(Species::Suicune)));
    assert_bits(crate::battle_species_models::mesh("SUICUNE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=rig.anatomy.iter().filter(|p| p.name.starts_with("Foot /") || p.name.starts_with("Toe /")).collect();
    assert_eq!(supports.len(),16);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if frame==17 {
                for prefix in ["Head /","Crest /","Mane /","Ribbon /"] {
                    let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
                    assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.001));
                }
            }
        }
    }
}

#[test]
fn furret_registry_keeps_paws_planted_while_body_ears_arms_and_tail_move() {
    let rig = fixture(Species::Furret);
    assert!(std::ptr::eq(super::for_species("FURRET").unwrap(), super::rig(Species::Furret)));
    assert_bits(crate::battle_species_models::mesh("FURRET").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let supports: Vec<_> = rig.anatomy.iter().filter(|p| p.name.starts_with("Foot /")).collect();
    assert_eq!(supports.len(), 2);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &supports {
                for v in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(v, &matrices), Vec3::from_array(rig.neutral.positions[v]), 0.000001);
                }
            }
            if frame == 17 {
                for prefix in ["Body /", "Eye /", "Ear /", "Arm /", "Tail /"] {
                    assert!(rig.anatomy.iter().filter(|p| p.name.starts_with(prefix)).any(|p|
                        p.vertices.clone().any(|v| rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) > 0.001)), "{prefix}");
                }
            }
        }
    }
}

#[test]
fn noctowl_registry_keeps_digits_planted_while_face_crown_wings_and_tail_move() {
    let rig = fixture(Species::Noctowl);
    assert!(std::ptr::eq(super::for_species("NOCTOWL").unwrap(), super::rig(Species::Noctowl)));
    assert_bits(crate::battle_species_models::mesh("NOCTOWL").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let supports: Vec<_> = rig.anatomy.iter().filter(|p| ["Foot /", "Toe /", "Claw /", "Rear toe /"].iter().any(|prefix| p.name.starts_with(prefix))).collect();
    assert_eq!(supports.len(), 16);
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL {
        for frame in 0..=60 {
            rig.sample_into(clip, rig.clip(clip).duration * frame as f32 / 60.0, Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &supports {
                for v in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(v, &matrices), Vec3::from_array(rig.neutral.positions[v]), 0.000001);
                }
            }
            if frame == 17 {
                for prefix in ["Eye /", "Crown /", "Wing /", "Tail /"] {
                    assert!(rig.anatomy.iter().filter(|p| p.name.starts_with(prefix)).any(|p|
                        p.vertices.clone().any(|v| rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) > 0.001)), "{prefix}");
                }
            }
        }
    }
}

#[test]
fn ladybird_registry_keeps_feet_fixed_and_articulates_fists_feelers_and_cases() {
    for (species, name) in [(Species::Ledyba,"LEDYBA"),(Species::Ledian,"LEDIAN")] {
        let rig = fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let supports: Vec<_> = rig.anatomy.iter().filter(|p| p.name.starts_with("Foot /")).collect();
        assert_eq!(supports.len(),2);
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {
            for frame in 0..=60 {
                rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.0,Playback::Clamp,&mut pose).unwrap();
                rig.skin_matrices_into(&pose,&mut matrices).unwrap();
                for part in &supports {for v in part.vertices.clone() {
                    assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);
                }}
                if frame==17 {
                    for prefix in ["Eye /","Hand /","Antenna /","Shell /","Wing /"] {
                        if prefix=="Wing /" && species==Species::Ledyba {continue;}
                        assert!(rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|
                            p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.001)),"{name} {prefix}");
                    }
                }
            }
        }
    }
}

#[test]
fn spider_registry_keeps_tips_fixed_and_articulates_six_limbs_and_face() {
    for (species,name) in [(Species::Spinarak,"SPINARAK"),(Species::Ariados,"ARIADOS")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let supports: Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Walking leg /") && p.name.ends_with("band 4"))
            .flat_map(|p|p.vertices.clone()).filter(|&v|rig.joint_indices[v]==[0;4]).collect();
        assert!(supports.len()>=if species==Species::Spinarak {84} else {56});
        let (mut pose,mut matrices)=storage(rig);
        let mut moved=std::collections::HashSet::new();
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &supports {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            if frame==17 {for prefix in ["Abdomen /","Head /","Eye socket /","Mandible /","Horn /","Walking leg /","Raised leg /"] {
                if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|
                    rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
            }}
        }}
        for prefix in ["Abdomen /","Head /","Eye socket /","Mandible /","Horn /","Walking leg /"] {assert!(moved.contains(prefix),"{name} {prefix}");}
        if species==Species::Ariados {assert!(moved.contains("Raised leg /"));}
    }
}

#[test]
fn quagsire_registry_keeps_pads_fixed_and_articulates_face_palms_and_dorsal_paddle() {
    let rig=fixture(Species::Quagsire);
    assert!(std::ptr::eq(super::for_species("QUAGSIRE").unwrap(),super::rig(Species::Quagsire)));
    assert_bits(crate::battle_species_models::mesh("QUAGSIRE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports: Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") || p.name.starts_with("Toe crease /")).collect();
    assert_eq!(supports.len(),6);
    let (mut pose,mut matrices)=storage(rig);
    let mut moved=std::collections::HashSet::new();
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==17 {for prefix in ["Body /","Eye /","Arm /","Finger crease /","Tail /","Dorsal fin /"] {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|
                rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    for prefix in ["Body /","Eye /","Arm /","Finger crease /","Tail /","Dorsal fin /"] {assert!(moved.contains(prefix),"{prefix}");}
}

#[test]
fn hoppip_registry_keeps_feet_fixed_and_articulates_leaf_veins_ears_and_tail() {
    let rig=fixture(Species::Hoppip);
    assert!(std::ptr::eq(super::for_species("HOPPIP").unwrap(),super::rig(Species::Hoppip)));
    assert_bits(crate::battle_species_models::mesh("HOPPIP").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports: Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();
    assert_eq!(supports.len(),2);
    let (mut pose,mut matrices)=storage(rig);
    let mut moved=std::collections::HashSet::new();
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"off-grid Hoppip point outside culling envelope");
            assert!(point.y>=-0.000001);
        }
        for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==17 {for prefix in ["Body /","Eye /","Arm /","Ear /","Leaf /","Leaf vein /","Tail /"] {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|
                rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    for prefix in ["Body /","Eye /","Arm /","Ear /","Leaf /","Leaf vein /","Tail /"] {assert!(moved.contains(prefix),"{prefix}");}
}

#[test]
fn skiploom_registry_keeps_feet_fixed_and_articulates_leaf_ears_and_radial_flower() {
    let rig=fixture(Species::Skiploom);
    assert!(std::ptr::eq(super::for_species("SKIPLOOM").unwrap(),super::rig(Species::Skiploom)));
    assert_bits(crate::battle_species_models::mesh("SKIPLOOM").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();
    assert_eq!(supports.len(),2);
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"off-grid Skiploom point outside culling envelope");
            assert!(point.y>=-0.000001);
        }
        for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==17 {for prefix in ["Body /","Eye /","Arm /","Ear leaf /","Ear fold /","Flower /","Flower petal /"] {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|
                rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    for prefix in ["Body /","Eye /","Arm /","Ear leaf /","Ear fold /","Flower /","Flower petal /"] {assert!(moved.contains(prefix),"{prefix}");}
}

#[test]
fn jumpluff_registry_keeps_feet_fixed_and_all_bristles_with_their_three_seed_heads() {
    let rig=fixture(Species::Jumpluff);
    assert!(std::ptr::eq(super::for_species("JUMPLUFF").unwrap(),super::rig(Species::Jumpluff)));
    assert_bits(crate::battle_species_models::mesh("JUMPLUFF").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();
    assert_eq!(supports.len(),2);
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.contains("fitted bristle group")).count(),3);
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Seed head /")) {
        let owner=if part.name.contains("lower -1") {6} else if part.name.contains("lower 1") {7} else {10};
        for v in part.vertices.clone() {
            assert_eq!(rig.joint_indices[v],[owner,0,0,0]);
            assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);
        }
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"off-grid Jumpluff point outside culling envelope");
            assert!(point.y>=-0.000001);
        }
        for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==17 {for prefix in ["Body /","Eye /","Arm /","Leaf /","Seed stem /","Seed head /"] {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|
                rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    for prefix in ["Body /","Eye /","Arm /","Leaf /","Seed stem /","Seed head /"] {assert!(moved.contains(prefix),"{prefix}");}
}

#[test]
fn aipom_keeps_supports_fixed_and_articulates_prehensile_tail_hand() {
    let rig=fixture(Species::Aipom);
    assert!(std::ptr::eq(super::for_species("AIPOM").unwrap(),super::rig(Species::Aipom)));
    assert_bits(crate::battle_species_models::mesh("AIPOM").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") || p.name.starts_with("Toe fold /")).collect();
    assert_eq!(supports.len(),6);
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Tail fingertip /")) {
        let owner=14+part.name.rsplit(' ').next().unwrap().parse::<u16>().unwrap();
        for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v],[owner,0,0,0]); }
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Aipom animation leaves culling bounds");
            assert!(point.y>=-0.000001,"Aipom animation crosses floor");
        }
        for part in &supports {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==17 {for prefix in ["Head /","Ear /","Arm /","Hand /","Tail /","Tail hand /","Tail fingertip /","Tuft /"] {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),8);
}

#[test]
fn sunflower_family_keeps_seed_base_and_feet_fixed_with_coherent_botanical_detail() {
    for species in [Species::Sunkern,Species::Sunflora] {
        let rig=fixture(species);let name=species.name().to_ascii_uppercase();
        assert!(std::ptr::eq(super::for_species(&name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(&name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let supports:Vec<_>=rig.anatomy.iter().flat_map(|part|part.vertices.clone().filter(|&v|part.name.starts_with("Foot /") || (species==Species::Sunkern && rig.neutral.positions[v][1]<0.001))).collect();
        assert!(!supports.is_empty());
        if species==Species::Sunflora {
            for part in &rig.anatomy {
                let owner=if part.name.starts_with("Petal /") {Some(9+part.name.rsplit(' ').next().unwrap().parse::<u16>().unwrap())}
                    else if part.name.starts_with("Bract /") {Some(17+part.name.rsplit(' ').next().unwrap().parse::<u16>().unwrap())}
                    else if ["Face /","Eye /","Mouth /","Tongue /"].iter().any(|p|part.name.starts_with(p)) {Some(4)} else {None};
                if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
            }
        }
        let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
        let prefixes:&[&str]=if species==Species::Sunkern {&["Body /","Sprout /","Leaf /","Leaf vein /"]} else {&["Body /","Face /","Leaf arm /","Leaf vein /","Petal /","Bract /"]};
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"sunflower family leaves culling bounds");
                assert!(point.y>=-0.000001,"sunflower family crosses floor");
            }
            for &v in &supports {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            if frame==17 {for &prefix in prefixes {
                if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
            }}
        }}
        assert_eq!(moved.len(),prefixes.len());
    }
}


#[test]
fn bellossom_keeps_leaf_tips_planted_and_crown_parts_coherent() {
    let rig=fixture(Species::Bellossom);
    assert!(std::ptr::eq(super::for_species("BELLOSSOM").unwrap(),super::rig(Species::Bellossom)));
    assert_bits(crate::battle_species_models::mesh("BELLOSSOM").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let supports:Vec<_>=(0..rig.neutral.positions.len()).filter(|&v|rig.neutral.positions[v][1]<0.001).collect();
    assert!(!supports.is_empty());
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Flower ") {
            let words:Vec<_>=part.name.split_whitespace().collect();
            let sign=words[words.len()-if part.name.starts_with("Flower petal /") {2} else {1}];
            Some(if sign=="-1" {8} else {9})
        } else if ["Head /","Eye /","Eye glint /","Cheek /","Mouth /","Tongue /"].iter().any(|prefix|part.name.starts_with(prefix)) {Some(3)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Body /","Head /","Arm /","Skirt /","Flower petal /","Flower center /","Flower well /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Bellossom leaves culling bounds");
            assert!(point.y>=-0.000001,"Bellossom crosses floor");
        }
        for &v in &supports {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        if frame==17 {for prefix in prefixes {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn qwilfish_keeps_spines_face_and_spotted_tail_coherent_without_floor_drift() {
    let rig=fixture(Species::Qwilfish);
    assert!(std::ptr::eq(super::for_species("QWILFISH").unwrap(),super::rig(Species::Qwilfish)));
    assert_bits(crate::battle_species_models::mesh("QWILFISH").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Tail fin /") || part.name.starts_with("Tail spot /") {Some(6)}
            else if !["Tail /","Fin /","Fin crease /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Body /","Spine /","Fin /","Fin crease /","Tail /","Tail fin /","Tail spot /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001,"Qwilfish changes floor height");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Qwilfish leaves culling bounds");
        }
        if frame==17 {for prefix in prefixes {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn wobbuffet_keeps_basal_lobes_planted_face_fitted_and_tail_eyes_coherent() {
    let rig=fixture(Species::Wobbuffet);
    assert!(std::ptr::eq(super::for_species("WOBBUFFET").unwrap(),super::rig(Species::Wobbuffet)));
    assert_bits(crate::battle_species_models::mesh("WOBBUFFET").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Foot /") {Some(0)}
            else if ["Eye /","Mouth /","Tongue /"].iter().any(|p|part.name.starts_with(p)) {Some(1)}
            else if ["Tail paddle /","Tail eye /","Tail pupil /"].iter().any(|p|part.name.starts_with(p)) {Some(10)}
            else if part.name.starts_with("Hand /") {Some(if part.name.contains("salute") {4} else {7})} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Body /","Arm /","Hand /","Tail /","Tail paddle /","Tail eye /","Tail pupil /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Wobbuffet crosses the supporting floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Wobbuffet leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")) {
            for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }
        if frame==17 {for prefix in prefixes {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn dunsparce_keeps_belly_grounded_wings_fitted_and_drill_spiral_coherent() {
    let rig=fixture(Species::Dunsparce);
    assert!(std::ptr::eq(super::for_species("DUNSPARCE").unwrap(),super::rig(Species::Dunsparce)));
    assert_bits(crate::battle_species_models::mesh("DUNSPARCE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if ["Drill /","Drill groove /"].iter().any(|p|part.name.starts_with(p)) {Some(6)}
            else if ["Body /","Belly /","Eye /","Jaw /","Back band /"].iter().any(|p|part.name.starts_with(p)) {Some(1)}
            else if ["Wing /","Wing crease /"].iter().any(|p|part.name.starts_with(p)) {Some(if part.name.ends_with("-1") {2} else {3})} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Body /","Belly /","Eye /","Jaw /","Wing /","Wing crease /","Back band /","Tail /","Drill /","Drill groove /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Dunsparce crosses the supporting floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Dunsparce leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Body /") || p.name.starts_with("Belly /")) {
            for v in part.vertices.clone() {assert!((rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]).abs()<0.000001,"Dunsparce lifts its belly");}
        }
        if frame==17 {for prefix in prefixes {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn shuckle_keeps_shell_apertures_and_feet_fixed_while_soft_body_moves() {
    let rig=fixture(Species::Shuckle);
    assert!(std::ptr::eq(super::for_species("SHUCKLE").unwrap(),super::rig(Species::Shuckle)));
    assert_bits(crate::battle_species_models::mesh("SHUCKLE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let hard=["Shell /","Interior /","Rim /","Spot /","Foot /"];
    for part in &rig.anatomy {
        let owner=if hard.iter().any(|p|part.name.starts_with(p)) {Some(0)}
            else if ["Head /","Eye /","Mouth /"].iter().any(|p|part.name.starts_with(p)) {Some(7)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Limb /","Neck /","Head /","Eye /","Mouth /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Shuckle crosses the supporting floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Shuckle leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|part|hard.iter().any(|p|part.name.starts_with(p))) {
            for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }
        if frame==17 {for prefix in prefixes {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn gligar_keeps_talons_planted_and_face_pincers_sails_and_venom_tail_coherent() {
    let rig=fixture(Species::Gligar);
    assert!(std::ptr::eq(super::for_species("GLIGAR").unwrap(),super::rig(Species::Gligar)));
    assert_bits(crate::battle_species_models::mesh("GLIGAR").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if ["Foot /","Foot claw /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
            else if ["Head /","Eye /","Pupil /","Eye glint /","Mouth /","Fang /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
            else if ["Tongue /","Tongue crease /"].iter().any(|p|part.name.starts_with(p)) {Some(5)}
            else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {3} else {4})}
            else if part.name.starts_with("Pincer /") {Some(if part.name.ends_with("-1") {7} else {9})}
            else if part.name.starts_with("Tail bulb /") {Some(13)} else if part.name.starts_with("Stinger /") {Some(14)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Body /","Head /","Ear /","Tongue /","Arm /","Pincer /","Wing /","Wing spar /","Leg /","Tail /","Tail bulb /","Stinger /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Gligar crosses the supporting floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Gligar leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") || p.name.starts_with("Foot claw /")) {
            for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }
        if frame==17 {for prefix in prefixes {
            if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}
        }}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn yanma_keeps_coated_wings_and_forked_toes_coherent_through_segmented_tail_motion() {
    let rig=fixture(Species::Yanma);
    assert!(std::ptr::eq(super::for_species("YANMA").unwrap(),super::rig(Species::Yanma)));
    assert_bits(crate::battle_species_models::mesh("YANMA").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Foot /") {Some(0)} else if part.name.starts_with("Thorax /") {Some(1)}
            else if ["Head /","Eye /","Eye glint /","Crown /","Mandible /","Forehead /","Mouth /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
            else if ["Tail cap /","Tail fin /","Tail tip /"].iter().any(|p|part.name.starts_with(p)) {Some(15)}
            else if ["Wing /","Wing band /","Wing vein /"].iter().any(|p|part.name.starts_with(p)) {
                let words:Vec<_>=part.name.split_whitespace().collect();let end=words.len()-usize::from(part.name.starts_with("Wing vein /"));
                let side=words[end-2];let rear:usize=words[end-1].parse().unwrap();Some(3+if side=="-1" {0} else {2}+rear)
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Thorax /","Head /","Wing /","Wing band /","Wing vein /","Leg /","Abdomen /","Abdomen collar /","Tail cap /","Tail fin /","Tail tip /"];
    for clip in Clip::ALL {for frame in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/61.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"Yanma crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Yanma leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==17 {for prefix in prefixes {if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}}}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn murkrow_keeps_planted_talons_and_coherent_folded_wings_through_broom_motion() {
    let rig=fixture(Species::Murkrow);
    assert!(std::ptr::eq(super::for_species("MURKROW").unwrap(),super::rig(Species::Murkrow)));
    assert_bits(crate::battle_species_models::mesh("MURKROW").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if ["Foot /","Toe /","Talon /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
            else if ["Head /","Hat /","Eye /","Pupil /","Nostril /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
            else if part.name.starts_with("Bill /") {Some(if part.name.contains("lower") || part.name.contains("seam") {3} else {2})}
            else if ["Tail fan /","Tail fold /"].iter().any(|p|part.name.starts_with(p)) {Some(7)}
            else if part.name.starts_with("Binding /") {Some(6)}
            else if part.name.starts_with("Crest /") {Some(8+part.name.split_whitespace().last().unwrap().parse::<usize>().unwrap())}
            else if part.name.starts_with("Wing /") || part.name.starts_with("Wing feather /") {
                let words:Vec<_>=part.name.split_whitespace().collect();let index=words.len()-if part.name.starts_with("Wing feather /") {2} else {1};Some(if words[index]=="-1" {4} else {5})
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
    let prefixes=["Head /","Crest /","Bill /","Wing /","Wing feather /","Tail /","Binding /","Tail fan /","Tail fold /"];
    for clip in Clip::ALL {for frame in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"Murkrow crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Murkrow leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|["Foot /","Toe /","Talon /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        if frame==9 {for prefix in prefixes {if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}}}
    }}
    assert_eq!(moved.len(),prefixes.len());
}

#[test]
fn misdreavus_keeps_colored_curl_seams_closed_and_necklace_coherent() {
    let rig=fixture(Species::Misdreavus);
    assert!(std::ptr::eq(super::for_species("MISDREAVUS").unwrap(),super::rig(Species::Misdreavus)));
    assert_bits(crate::battle_species_models::mesh("MISDREAVUS").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let mut seams=Vec::new();
    for part in &rig.anatomy {
        if let Some(name)=part.name.strip_prefix("Hair tip / tapered pink curl ") {
            let base=rig.anatomy.iter().find(|p|p.name==format!("Hair / swept volumetric {name}")).unwrap();
            let vertices:std::collections::HashMap<_,_>=base.vertices.clone().map(|v|(rig.neutral.positions[v].map(f32::to_bits),v)).collect();
            let mut shared=std::collections::HashSet::new();
            for v in part.vertices.clone() {let key=rig.neutral.positions[v].map(f32::to_bits);if let Some(&a)=vertices.get(&key) {
                assert_eq!(rig.joint_indices[a],rig.joint_indices[v]);assert_eq!(rig.joint_weights[a],rig.joint_weights[v]);
                shared.insert(key);seams.push((a,v));
            }}
            assert_eq!(shared.len(),24);
        } else if !part.name.starts_with("Body /") && !part.name.starts_with("Hair /") {
            for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[2,0,0,0]);}
        }
    }
    assert!(!seams.is_empty());
    let body=rig.anatomy.iter().find(|p|p.name.starts_with("Body /")).unwrap();
    let support:Vec<_>=body.vertices.clone().filter(|&v|rig.neutral.positions[v][1]<0.015).collect();assert!(support.len()>=5);
    let (mut pose,mut matrices)=storage(rig);let mut moved=false;
    for clip in Clip::ALL {for frame in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"Misdreavus crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Misdreavus leaves culling bounds");
        }
        for &v in &support {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        for &(a,b) in &seams {assert_vec_close(rig.skin_point(a,&matrices),rig.skin_point(b,&matrices),0.000001);}
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Hair tip /")) {if part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005) {moved=true;}}
    }}
    assert!(moved,"curled hair tips never articulate");
}

#[test]
fn bear_family_keeps_paws_lunar_markings_and_honey_contact_coherent() {
    for (name,species) in [("TEDDIURSA",Species::Teddiursa),("URSARING",Species::Ursaring)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let owner=if ["Foot /","Foot claw /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
                else if part.name.starts_with("Ring /") {Some(1)}
                else if ["Head /","Moon /","Eye /","Pupil /","Muzzle /","Nose /","Mouth /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
                else if part.name.starts_with("Tail /") {Some(7)}
                else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {8} else {9})}
                else if part.name.starts_with("Hand /") || part.name.starts_with("Hand claw /") {
                    let words:Vec<_>=part.name.split_whitespace().collect();let index=words.len()-if part.name.starts_with("Hand claw /") {2} else {1};Some(if words[index]=="-1" {4} else {6})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);let mut moved=false;
        for clip in Clip::ALL {for frame in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Foot claw /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if species==Species::Teddiursa {for (a,b) in matrices[2].to_cols_array().iter().zip(matrices[4].to_cols_array()) {assert!((a-b).abs()<0.000001,"honey paw separates from head");}}
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Hand / broad rounded bear palm 1")) {if part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005) {moved=true;}}
        }}
        assert!(moved,"{name} right palm never articulates");
    }
}

#[test]
fn bulldog_family_keeps_paws_folded_ears_spots_and_tusks_coherent() {
    for (name,species) in [("SNUBBULL",Species::Snubbull),("GRANBULL",Species::Granbull)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let owner=if ["Foot /","Toe groove /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
                else if part.name.starts_with("Spot /") {Some(1)}
                else if ["Head /","Jowl /","Eye /","Pupil /","Muzzle /","Nose /","Mouth /","Fang /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
                else if ["Jaw /","Tusk /"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
                else if part.name.starts_with("Tail /") {Some(8)}
                else if part.name.starts_with("Ear /") || part.name.starts_with("Ear marking /") {Some(if part.name.ends_with("-1") {9} else {10})}
                else if ["Hand /","Hand fold /","Wrist /"].iter().any(|p|part.name.starts_with(p)) {
                    let words:Vec<_>=part.name.split_whitespace().collect();let index=words.len()-if part.name.starts_with("Hand fold /") {2} else {1};Some(if words[index]=="-1" {5} else {7})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
        let prefixes=["Head /","Ear /","Arm /","Hand /","Jaw /","Tail /"];
        for clip in Clip::ALL {for frame in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Toe groove /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if frame==9 {for prefix in prefixes {if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}}}
        }}
        assert_eq!(moved.len(),prefixes.len(),"{name} has an inactive feature");
    }
}

#[test]
fn hellhound_family_keeps_armored_paws_skulls_horns_and_tails_coherent() {
    for (name,species) in [("HOUNDOUR",Species::Houndour),("HOUNDOOM",Species::Houndoom)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let owner=if ["Paw /","Paw crease /","Claw /","Cuff /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
                else if ["Body /","Armor / broad"].iter().any(|p|part.name.starts_with(p)) {Some(1)}
                else if ["Collar /","Charm /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
                else if ["Head /","Muzzle /","Nose /","Eye /","Horn /","Fang /","Armor / fitted","Armor / dark","Mouth / recessed","Mouth / fitted closed"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
                else if ["Jaw /","Mouth / pink"].iter().any(|p|part.name.starts_with(p)) {Some(4)}
                else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {11} else {12})}
                else if part.name.starts_with("Tail / pointed") {Some(11)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
        let prefixes=["Head /","Leg /","Tail /","Armor /"];
        for clip in Clip::ALL {for frame in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|["Paw /","Paw crease /","Claw /","Cuff /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if frame==9 {for prefix in prefixes {if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}}}
        }}
        assert_eq!(moved.len(),prefixes.len(),"{name} has an inactive feature");
    }
}

#[test]
fn bovine_family_keeps_hooves_udder_studs_and_three_tail_chains_coherent() {
    for (name,species) in [("MILTANK",Species::Miltank),("TAUROS",Species::Tauros)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let owner=if species==Species::Miltank {
                if part.name.starts_with("Foot /") {Some(0)}
                else if ["Body /","Udder /","Teat /","Spot /"].iter().any(|p|part.name.starts_with(p)) {Some(1)}
                else if ["Head /","Face /","Eye /","Muzzle /","Nose /","Mouth /","Horn /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
                else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {7} else {8})}
                else if part.name.starts_with("Hand /") {Some(if part.name.ends_with("-1") {4} else {6})}
                else if part.name.starts_with("Tail / rounded") {Some(11)} else {None}
            } else {
                if ["Hoof /","Hoof groove /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
                else if part.name.starts_with("Body /") {Some(1)}
                else if ["Mane /","Mane crease /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
                else if ["Head /","Eye /","Nose /","Nostril /","Rivet /","Horn /"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
                else if part.name.starts_with("Tail / pointed") {Some(10+3*part.name.split_whitespace().last().unwrap().parse::<usize>().unwrap())} else {None}
            };
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);let mut moved=std::collections::HashSet::new();
        let prefixes=["Head /","Horn /","Leg /","Tail /"];
        for clip in Clip::ALL {for frame in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(frame as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Hoof /","Hoof groove /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            if frame==9 {for prefix in prefixes {if rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)).any(|p|p.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))>0.0005)) {moved.insert(prefix);}}}
        }}
        assert_eq!(moved.len(),prefixes.len(),"{name} has an inactive feature");
    }
}

#[test]
fn delibird_skin_keeps_webs_planted_and_held_sack_coherent() {
    let rig=fixture(Species::Delibird);
    assert!(std::ptr::eq(super::for_species("DELIBIRD").unwrap(),super::rig(Species::Delibird)));
    assert_bits(crate::battle_species_models::mesh("DELIBIRD").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Foot /") {Some(0)}
            else if ["Head /","Eye /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
            else if part.name.starts_with("Beak /") {Some(if part.name.contains("lower") || part.name.contains("seam") {3} else {2})}
            else if part.name.starts_with("Crest /") {Some(if part.name.ends_with("-1") {6} else {7})}
            else if part.name.starts_with("Tail sack /") || part.name=="Wing / broad tapered flipper 1" {Some(5)}
            else if part.name=="Wing / broad tapered flipper -1" {Some(4)} else {None};
        if let Some(owner)=owner {
            for i in part.vertices.clone() {assert_eq!(rig.joint_indices[i],[owner,0,0,0],"{}",part.name);assert_eq!(rig.joint_weights[i],[1.0,0.0,0.0,0.0]);}
        }
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*step as f32/30.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all());
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")) {for v in part.vertices.clone() {
            assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);
        }}
    }}
}

#[test]
fn stantler_skin_keeps_hooves_planted_and_antler_loops_coherent() {
    let rig=fixture(Species::Stantler);
    assert!(std::ptr::eq(super::for_species("STANTLER").unwrap(),super::rig(Species::Stantler)));
    assert_bits(crate::battle_species_models::mesh("STANTLER").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if ["Hoof /","Hoof groove /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
            else if ["Head /","Eye /","Brow /","Brow spot /","Muzzle /","Nostril /","Antler"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
            else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {4} else {5})}
            else if part.name.starts_with("Tail / large") {Some(11)}
            else if ["Body /","Spot /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {
            for i in part.vertices.clone() {assert_eq!(rig.joint_indices[i],[owner,0,0,0],"{}",part.name);assert_eq!(rig.joint_weights[i],[1.0,0.0,0.0,0.0]);}
        }
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*step as f32/30.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all());
        }
        for part in rig.anatomy.iter().filter(|p|["Hoof /","Hoof groove /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {
            assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);
        }}
    }}
}

#[test]
fn girafarig_skin_keeps_hooves_planted_and_both_heads_coherent() {
    let rig=fixture(Species::Girafarig);
    assert!(std::ptr::eq(super::for_species("GIRAFARIG").unwrap(),super::rig(Species::Girafarig)));
    assert_bits(crate::battle_species_models::mesh("GIRAFARIG").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let owner=if ["Hoof /","Hoof groove /"].iter().any(|p|part.name.starts_with(p)) {Some(0)}
            else if ["Head /","Eye /","Mouth /","Muzzle /","Horn /","Horn tip /"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
            else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {4} else {5})}
            else if ["Tail head /","Tail eye /","Tail mouth /","Tail tooth /"].iter().any(|p|part.name.starts_with(p)) {Some(12)}
            else if ["Body /","Spot / fitted gold","Mane / pink back"].iter().any(|p|part.name.starts_with(p)) || part.name=="Spot / fitted brown shoulder marking group 0" {Some(1)} else {None};
        if let Some(owner)=owner {
            for i in part.vertices.clone() {assert_eq!(rig.joint_indices[i],[owner,0,0,0],"{}",part.name);assert_eq!(rig.joint_weights[i],[1.0,0.0,0.0,0.0]);}
        }
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*step as f32/30.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001);
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all());
        }
        for part in rig.anatomy.iter().filter(|p|["Hoof /","Hoof groove /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {
            assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);
        }}
    }}
}

#[test]
fn elephant_family_keeps_pads_toes_ears_and_armor_coherent() {
    for (name,species) in [("PHANPY",Species::Phanpy),("DONPHAN",Species::Donphan)] {
        let rig=fixture(species);let baby=species==Species::Phanpy;
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Body /") {Some(1)}
                else if ["Foot /","Toenail /","Ankle armor /"].iter().any(|p|n.starts_with(p)) {Some(if baby && n.ends_with("-1 front") {9} else {0})}
                else if ["Ear /","Ear tip /","Ear lining /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Ear tip /") {n.split_whitespace().rev().nth(1).unwrap()} else {n.split_whitespace().last().unwrap()};Some(if side=="-1" {5} else {6})
                }
                else if ["Nose /","Nostril /"].iter().any(|p|n.starts_with(p)) {Some(4)}
                else if ["Head /","Eye /","Eye glint /","Eye white /","Pupil /","Forehead /","Mouth /","Tongue /","Tusk /"].iter().any(|p|n.starts_with(p)) {Some(2)}
                else if n.starts_with("Tread /") {let p=n.split_whitespace().last().unwrap().parse::<usize>().unwrap();Some(if p<3 {1} else if p<5 {2} else if p<7 {3} else {4})} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Toenail /","Ankle armor /"].iter().any(|s|p.name.starts_with(s)) && !(baby && p.name.ends_with("-1 front"))) {for v in part.vertices.clone() {
                assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);
            }}
        }}
    }
}

#[test]
fn water_rabbit_family_keeps_paddles_planted_and_bellies_welded() {
    for (name,species) in [("MARILL",Species::Marill),("AZUMARILL",Species::Azumarill)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") {Some(0)}
                else if ["Eye /","Mouth /","Nose /"].iter().any(|p|part.name.starts_with(p)) {Some(2)}
                else if part.name.starts_with("Tail / blue") {Some(11)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let a=&rig.anatomy[0];let b=&rig.anatomy[1];let mut seam=Vec::new();
        for v in a.vertices.clone() {if let Some(w)=b.vertices.clone().find(|&w|rig.neutral.positions[v]==rig.neutral.positions[w]) {seam.push((v,w));}}
        assert_eq!(seam.len(),40);
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for &(v,w) in &seam {assert_eq!(rig.skin_point(v,&matrices),rig.skin_point(w,&matrices),"{name} tears its belly seam");}
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn psychic_bird_family_keeps_talons_planted_and_wing_markings_coherent() {
    for (name,species) in [("NATU",Species::Natu),("XATU",Species::Xatu)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Toe /","Claw /"].iter().any(|p|n.starts_with(p)) {Some(0)}
                else if ["Head /","Eye /","Pupil /","Eye glint /","Beak /","Bill seam /"].iter().any(|p|n.starts_with(p)) {Some(2)}
                else if ["Wing /","Wing gold /","Wing red /","Wing badge","Wing hem"].iter().any(|p|n.starts_with(p)) {
                    let core=n.strip_suffix(" rear").unwrap_or(n);let side=if n.starts_with("Wing red /") || n.starts_with("Wing hem") {core.split_whitespace().rev().nth(1).unwrap()} else {core.split_whitespace().last().unwrap()};Some(if side=="-1" {3} else {4})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|["Toe /","Claw /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn electric_sheep_family_keeps_supports_curls_and_beacons_coherent() {
    for (name,species,beacon) in [("MAREEP",Species::Mareep,8),("FLAAFFY",Species::Flaaffy,10),("AMPHAROS",Species::Ampharos,8)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Foot /","Toe /"].iter().any(|p|n.starts_with(p)) {Some(0)}
                else if ["Crown /","Crown groove /","Eye /","Nostril /","Muzzle /","Forehead /","Horn /"].iter().any(|p|n.starts_with(p)) {Some(3)}
                else if ["Tail / round","Tail / ivory","Tail / ruby","Tail / quiet"].iter().any(|p|n.starts_with(p)) {Some(beacon)}
                else if species==Species::Mareep && n.starts_with("Fleece /") {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Toe /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn baby_family_keeps_planted_feet_and_fitted_faces_coherent() {
    for (name,species) in [("PICHU",Species::Pichu),("CLEFFA",Species::Cleffa),("IGGLYBUFF",Species::Igglybuff)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Foot /") {Some(0)}
                else if ["Head /","Eye /","Eye glint /","Eye border /","Iris /","Pupil /","Cheek /","Nose /","Mouth /","Tongue /","Forehead /"].iter().any(|p|n.starts_with(p)) {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Toe /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn magby_keeps_planted_claws_open_bill_and_crown_coherent() {
    for (name,species) in [("MAGBY",Species::Magby)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Foot /","Toe claw /"].iter().any(|p|n.starts_with(p)) {Some(0)}
                else if ["Head /","Eye socket /","Eye /","Pupil /","Glint /","Bill /","Mouth back /"].iter().any(|p|n.starts_with(p)) {Some(3)}
                else if n.starts_with("Crown lobe /") {Some(4)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /","Toe claw /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn elekid_keeps_planted_feet_open_plugs_and_striped_arms_coherent() {
    for (name,species) in [("ELEKID",Species::Elekid)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Foot /") {Some(0)}
                else if ["Eye socket /","Eye /","Pupil /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(2)}
                else if n.starts_with("Plug /") {Some(if n.ends_with("-1") {3} else {4})}
                else if ["Arm root /","Arm /","Arm stripe /","Hand /","Hand claw /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Arm stripe /") || n.starts_with("Hand claw /") {n.split_whitespace().nth_back(1).unwrap()} else {n.split_whitespace().next_back().unwrap()};
                    Some(if side=="-1" {5} else {6})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn smoochum_keeps_planted_feet_fitted_gaze_and_bob_coherent() {
    for (name,species) in [("SMOOCHUM",Species::Smoochum)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Foot /") {Some(0)}
                else if n.starts_with("Hair side /") {Some(if n.ends_with("-1") {4} else {5})}
                else if n.starts_with("Hair tuft /") {Some(6)}
                else if ["Head /","Eye socket /","Eye /","Pupil /","Iris /","Glint /","Lips /","Mouth /","Hair","Bangs /"].iter().any(|p|n.starts_with(p)) {Some(3)}
                else if ["Arm /","Hand /","Finger /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Finger /") {n.split_whitespace().nth_back(1).unwrap()} else {n.split_whitespace().next_back().unwrap()};
                    Some(if side=="-1" {7} else {8})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Foot /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn tyrogue_keeps_planted_shoes_fitted_gaze_and_wrapped_wrists_coherent() {
    for (name,species) in [("TYROGUE",Species::Tyrogue)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Shoe /","Shoe seam /"].iter().any(|p|n.starts_with(p)) {Some(0)}
                else if ["Ear /","Ear contour /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {4} else {5})}
                else if ["Head /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(3)}
                else if ["Wrist band /","Palm /","Finger /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Finger /") {n.split_whitespace().nth_back(1).unwrap()} else {n.split_whitespace().next_back().unwrap()};
                    Some(if side=="-1" {7} else {9})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Shoe /","Shoe seam /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn hitmontop_keeps_grounded_spin_point_and_raised_balance_limbs_coherent() {
    for (name,species) in [("HITMONTOP",Species::Hitmontop)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Spin point /") {Some(0)}
                else if ["Head /","Eye /","Pupil /","Helmet crease /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(1)}
                else if ["Arm /","Wrist wrap /","Hand /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {4} else {5})}
                else if ["Leg /","Ankle wrap /","Foot /","Foot spike /","Foot sole /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Foot spike /") {n.split_whitespace().nth_back(1).unwrap()} else {n.split_whitespace().next_back().unwrap()};
                    Some(if side=="-1" {6} else {7})
                } else if ["Tail ball /","Tail spike /"].iter().any(|p|n.starts_with(p)) {Some(10)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Spin point /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn hitmontop_idle_rotates_once_forward_and_wraps_without_a_ground_or_pose_jump() {
    use bevy::prelude::Vec2;
    let rig=fixture(Species::Hitmontop);
    let origin=rig.joints[1].bind.translation;
    let helmet=rig.anatomy.iter().find(|p|p.name.starts_with("Head /")).unwrap();
    let vertex=helmet.vertices.clone().find(|&i| {
        let p=Vec3::from_array(rig.neutral.positions[i])-origin;
        Vec2::new(p.x,p.z).length()>0.08
    }).unwrap();
    let (mut pose,mut matrices)=storage(rig);
    let duration=rig.clip(Clip::Idle).duration;
    let mut previous:Option<Vec2>=None;
    let mut total=0.0_f32;
    for step in 0..=160 {
        rig.sample_into(Clip::Idle,duration*step as f32/160.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        let point=rig.skin_point(vertex,&matrices)-origin;
        let direction=Vec2::new(point.x,point.z);
        if let Some(previous)=previous {
            let angle=previous.perp_dot(direction).atan2(previous.dot(direction));
            assert!(angle<0.0 && angle> -0.08,"idle must rotate forward continuously: {step} {angle}");
            total+=angle;
        }
        previous=Some(direction);
    }
    assert!((total+std::f32::consts::TAU).abs()<0.00002);
    for time in [duration-0.00001,duration+0.00001] {
        rig.sample_into(Clip::Idle,time,Playback::Loop,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for (i,p) in rig.neutral.positions.iter().enumerate() {
            assert_vec_close(rig.skin_point(i,&matrices),Vec3::from_array(*p),0.00001);
        }
    }
}

#[test]
fn hitmonchan_keeps_planted_boots_fitted_gaze_and_gloves_coherent() {
    for (name,species) in [("HITMONCHAN",Species::Hitmonchan)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Boot /","Boot cuff /","Boot seam /"].iter().any(|p|n.starts_with(p)) {Some(0)}
                else if ["Head /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(3)}
                else if n.starts_with("Shoulder guard /") {Some(if n.ends_with("-1") {4} else {5})}
                else if ["Glove cuff /","Glove /","Glove thumb /","Glove seam /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {7} else {9})} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if rig.neutral.positions[v][1]==0. {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            }
            for part in rig.anatomy.iter().filter(|p|["Boot /","Boot cuff /","Boot seam /"].iter().any(|s|p.name.starts_with(s))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn hitmonlee_keeps_support_foot_grounded_and_executes_a_lifted_kick() {
    for (name,species) in [("HITMONLEE",Species::Hitmonlee)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Foot /","Toe claw /","Foot mark /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Toe claw /") {n.split_whitespace().nth_back(1).unwrap()} else {n.split_whitespace().next_back().unwrap()};
                    Some(if side=="-1" {3} else {0})
                } else if ["Eye /","Pupil /","Pupil core /"].iter().any(|p|n.starts_with(p)) {Some(5)}
                else if ["Hand /","Finger /"].iter().any(|p|n.starts_with(p)) {
                    let side=if n.starts_with("Finger /") {n.split_whitespace().nth_back(1).unwrap()} else {n.split_whitespace().next_back().unwrap()};
                    Some(if side=="-1" {7} else {9})
                } else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") && p.name.ends_with(" 1")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
        rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        let kick=rig.anatomy.iter().find(|p|p.name.starts_with("Foot /") && p.name.ends_with(" -1")).unwrap();
        assert!(kick.vertices.clone().map(|v|rig.skin_point(v,&matrices).y).fold(f32::INFINITY,f32::min)>0.10);
    }
}

#[test]
fn egg_fairies_preserve_fitted_details_and_floor_contacts_through_battle_cues() {
    for (name,species,head) in [("TOGEPI",Species::Togepi,2),("TOGETIC",Species::Togetic,3)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Foot /","Foot point /","Foot crease /","Toe crease /"].iter().any(|p|n.starts_with(p)) {Some(0)}
                else if ["Head /","Eye /","Eye glint /","Eye crease /","Brow /","Mouth /","Tongue /"].iter().any(|p|n.starts_with(p)) {Some(head)}
                else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn jet_fish_keep_ground_contacts_and_coherent_facial_details_through_battle_cues() {
    for (name,species) in [("REMORAID",Species::Remoraid),("OCTILLERY",Species::Octillery)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Eye /","Mouth /","Stripe /","Crown /"].iter().any(|p|n.starts_with(p)) {Some(1)}
                else if species==Species::Remoraid && n.starts_with("Tail /") {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);let neutral=Vec3::from_array(rig.neutral.positions[v]);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if neutral.y<0.025 {
                    if species==Species::Octillery {assert_vec_close(point,neutral,0.000001);}
                    else {assert!((point.y-neutral.y).abs()<0.000001);}
                }
            }
        }}
    }
}

#[test]
fn fur_pigs_keep_hooves_grounded_and_snouts_coherent_through_battle_cues() {
    for (name,species) in [("SWINUB",Species::Swinub),("PILOSWINE",Species::Piloswine)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Hoof /") {Some(0)}
                else if ["Snout /","Nostril /","Tusk /","Eye /","Fur crease /"].iter().any(|p|n.starts_with(p)) {Some(2)}
                else if n.starts_with("Stripe /") {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Hoof /")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        }}
    }
}

#[test]
fn sneasel_keeps_toe_talons_planted_and_advances_its_hooked_claws() {
    let rig=fixture(Species::Sneasel);
    assert!(std::ptr::eq(super::for_species("SNEASEL").unwrap(),super::rig(Species::Sneasel)));
    assert_bits(crate::battle_species_models::mesh("SNEASEL").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Foot /") || n.starts_with("Talon / paired") {Some(0)}
            else if ["Head /","Eye /","Mouth /"].iter().any(|p|n.starts_with(p)) || n=="Gem / fitted gold oval forehead" {Some(3)}
            else if n=="Gem / fitted gold oval chest" {Some(1)}
            else if n.starts_with("Ear /") {Some(4)} else if n.starts_with("Plume /") {Some(5)}
            else if n.starts_with("Tail /") {Some(6)}
            else if n.starts_with("Hand /") || n.starts_with("Talon / hooked") {
                let side=if n.starts_with("Hand /") {n.split_whitespace().next_back().unwrap()} else {n.split_whitespace().nth_back(1).unwrap()};Some(if side=="-1" {8} else {10})
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"Sneasel crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Sneasel leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") || p.name.starts_with("Talon / paired")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name=="Hand / broad claw-bearing palm -1" || p.name.starts_with("Talon / hooked") && p.name.split_whitespace().nth_back(1)==Some("-1")) {
        let forward=part.vertices.clone().map(|v|rig.skin_point(v,&matrices).z-rig.neutral.positions[v][2]).sum::<f32>()/part.vertices.len() as f32;
        assert!(forward>0.03,"{} does not advance into the strike",part.name);
    }
}

#[test]
fn skarmory_keeps_talons_planted_strokes_metal_wings_and_opens_its_bill() {
    let rig=fixture(Species::Skarmory);
    assert!(std::ptr::eq(super::for_species("SKARMORY").unwrap(),super::rig(Species::Skarmory)));
    assert_bits(crate::battle_species_models::mesh("SKARMORY").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Claw /") {Some(0)}
            else if ["Head /","Eye /","Crest /","Nape /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if n.starts_with("Bill /") {Some(9)} else if n.starts_with("Tail /") {Some(6)}
            else if n.starts_with("Neck /") {Some(if n.ends_with(" 3") {3} else {2})}
            else if n.starts_with("Wing /") {
                let side=if n.starts_with("Wing / swept") {n.split_whitespace().next_back().unwrap()} else {n.split_whitespace().nth_back(1).unwrap()};Some(if side=="-1" {4} else {5})
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"Skarmory crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Skarmory leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Claw /")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing / swept")) {
        let tip=part.vertices.clone().max_by(|&a,&b|rig.neutral.positions[a][0].abs().total_cmp(&rig.neutral.positions[b][0].abs())).unwrap();
        assert!(rig.skin_point(tip,&matrices).y<rig.neutral.positions[tip][1]-0.10,"{} does not stroke",part.name);
    }
    let lower=rig.anatomy.iter().find(|p|p.name.starts_with("Bill / tapered")).unwrap();
    let tip=lower.vertices.clone().max_by(|&a,&b|rig.neutral.positions[a][2].total_cmp(&rig.neutral.positions[b][2])).unwrap();
    let closed=matrices[3].transform_point3(Vec3::from_array(rig.neutral.positions[tip]));
    assert!(rig.skin_point(tip,&matrices).y<closed.y-0.008);
}

#[test]
fn smeargle_keeps_paws_planted_and_paints_with_a_coherent_held_tail() {
    let rig=fixture(Species::Smeargle);
    assert!(std::ptr::eq(super::for_species("SMEARGLE").unwrap(),super::rig(Species::Smeargle)));
    assert_bits(crate::battle_species_models::mesh("SMEARGLE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Toe groove /","Cuff /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Head /","Eye /","Muzzle /","Nostril /","Mouth /","Tongue /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if n.starts_with("Beret /") {Some(6)} else if n.starts_with("Brush /") {Some(10)}
            else if n.starts_with("Ear /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Hand /") || n.starts_with("Finger /") {
                let side=if n.starts_with("Hand /") {n.split_whitespace().next_back().unwrap()} else {n.split_whitespace().nth_back(1).unwrap()};Some(if side=="-1" {8} else {10})
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let hand=rig.anatomy.iter().find(|p|p.name=="Hand / cream open palm 1").unwrap();
    let center=hand.vertices.clone().map(|v|Vec3::from_array(rig.neutral.positions[v])).sum::<Vec3>()/hand.vertices.len() as f32;
    let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail /")).unwrap();
    let grip=tail.vertices.clone().min_by(|&a,&b|Vec3::from_array(rig.neutral.positions[a]).distance_squared(center).total_cmp(&Vec3::from_array(rig.neutral.positions[b]).distance_squared(center))).unwrap();
    assert!(rig.joint_indices[grip].iter().zip(rig.joint_weights[grip]).any(|(&j,w)|j==10 && w==1.));
    let distance=Vec3::from_array(rig.neutral.positions[grip]).distance(center);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"Smeargle crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Smeargle leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|["Foot /","Toe groove /","Cuff /"].iter().any(|prefix|p.name.starts_with(prefix))) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        assert!((rig.skin_point(grip,&matrices).distance(matrices[10].transform_point3(center))-distance).abs()<0.000001);
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    let paint=rig.anatomy.iter().find(|p|p.name.starts_with("Brush / tapered")).unwrap();
    let advance=paint.vertices.clone().map(|v|rig.skin_point(v,&matrices).z-rig.neutral.positions[v][2]).sum::<f32>()/paint.vertices.len() as f32;
    assert!(advance>0.035,"Smeargle's brush does not paint forward");
}

#[test]
fn lava_mollusks_keep_contact_height_and_articulate_stalks_and_shell() {
    for (name,species) in [("SLUGMA",Species::Slugma),("MAGCARGO",Species::Magcargo)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if ["Shell /","Shell ridge /","Shell seam /","Chimney /"].iter().any(|p|n.starts_with(p)) {Some(5)}
                else if n.starts_with("Eye stalk /") || n.starts_with("Eye /") {Some(if n.ends_with("-1") {3} else {4})}
                else if n.starts_with("Lava /") || n.starts_with("Mouth /") {Some(2)}
                else if n.starts_with("Mantle /") {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);let neutral=Vec3::from_array(rig.neutral.positions[v]);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if neutral.y<0.005 {assert!((point.y-neutral.y).abs()<0.000001,"{name} contact layer changes height");}
            }
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Eye / broad")) {
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[2].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.003),"{name} eye stalk does not peer independently");
        }
        if species==Species::Magcargo {
            let shell=rig.anatomy.iter().find(|p|p.name=="Chimney / hollow stone vent").unwrap();
            assert!(shell.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005));
        }
    }
}

#[test]
fn porygon2_keeps_paddles_fixed_and_advances_coherent_bill() {
    let rig=fixture(Species::Porygon2);
    assert!(std::ptr::eq(super::for_species("PORYGON2").unwrap(),super::rig(Species::Porygon2)));
    assert_bits(crate::battle_species_models::mesh("PORYGON2").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Foot /") || n.starts_with("Foot highlight /") {Some(0)}
            else if ["Bill /","Eye socket /","Eye white /","Pupil /","Eye glint /","Head highlight /"].iter().any(|p|n.starts_with(p)) {Some(3)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Porygon2 crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Porygon2 leaves culling bounds");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") || p.name.starts_with("Foot highlight /")) {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    let bill=rig.anatomy.iter().find(|p|p.name.starts_with("Bill /")).unwrap();
    let advance=bill.vertices.clone().map(|v|rig.skin_point(v,&matrices).z-rig.neutral.positions[v][2]).sum::<f32>()/bill.vertices.len() as f32;
    assert!(advance>0.015,"Porygon2 bill does not advance in attack");
    let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail /")).unwrap();
    assert!(tail.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.01));
}

#[test]
fn porygon_keeps_panels_rigid_and_paddle_contact_height() {
    let rig=fixture(Species::Porygon);
    assert!(std::ptr::eq(super::for_species("PORYGON").unwrap(),super::rig(Species::Porygon)));
    assert_bits(crate::battle_species_models::mesh("PORYGON").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Foot /") {if n.ends_with("-1") {5} else {6}}
            else if ["Head /","Bill /","Eye border /","Eye white /","Pupil /","Eye glint /"].iter().any(|p|n.starts_with(p)) {3}
            else if n.starts_with("Neck /") {2} else if n.starts_with("Tail /") {4} else {1};
        for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Porygon crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Porygon leaves culling bounds");
            if part.name.starts_with("Foot /") {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001,"paddle contact height changes");}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    let bill=rig.anatomy.iter().find(|p|p.name.starts_with("Bill /")).unwrap();
    let advance=bill.vertices.clone().map(|v|rig.skin_point(v,&matrices).z-rig.neutral.positions[v][2]).sum::<f32>()/bill.vertices.len() as f32;
    assert!(advance>0.010,"Porygon bill does not advance in attack");
    let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail /")).unwrap();
    assert!(tail.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005));
}

#[test]
fn anglers_keep_pelvic_contact_and_articulate_fins_lures_and_tail() {
    for (name,species) in [("CHINCHOU",Species::Chinchou),("LANTURN",Species::Lanturn)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Fin / broad") || n.starts_with("Fin fold /") {Some(if n.ends_with("-1") {2} else {3})}
                else if n.starts_with("Fin / soft") {Some(if n.ends_with("-1") {8} else {9})}
                else if n.starts_with("Lure /") {Some(if species==Species::Chinchou {if n.ends_with("-1") {5} else {7}} else if n.contains("large") {5} else {7})}
                else if n.starts_with("Tail fin /") {Some(11)}
                else if ["Body /","Mask /","Eye /","Pupil /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if part.name.starts_with("Fin / soft") {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001,"{name} pelvic contact height changes");}
            }}
        }}
        rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Fin / broad")) {
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]< -0.02),"{name} fin does not stroke");
        }
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Lure / hanging") || p.name.starts_with("Lure / golden")) {
            let owner=rig.joint_indices[part.vertices.start][0] as usize;
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[owner-1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005),"{name} lure does not move independently");
        }
        if species==Species::Lanturn {
            let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail fin /")).unwrap();
            assert!(tail.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[10].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005));
        }
    }
}

#[test]
fn mantine_keeps_shared_wing_seams_and_lifts_attached_remora() {
    let rig=fixture(Species::Mantine);
    assert!(std::ptr::eq(super::for_species("MANTINE").unwrap(),super::rig(Species::Mantine)));
    assert_bits(crate::battle_species_models::mesh("MANTINE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Tail leaf /") || n.starts_with("Tail leaf back /") {Some(9)}
            else if n.starts_with("Cephalic fin /") {Some(if n.ends_with("-1") {6} else {7})}
            else if n.starts_with("Remora tail /") {Some(12)}
            else if n.starts_with("Remora") && !n.starts_with("Remora / slender") {Some(10)}
            else if ["Body /","Back /","Eye /","Mouth /","Gill /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let mut seams=Vec::new();
    for side in ["-1","1"] {
        let upper=rig.anatomy.iter().find(|p|p.name==format!("Wing / curved blue back {side}")).unwrap();
        let lower=rig.anatomy.iter().find(|p|p.name==format!("Wing / pale curved underside {side}")).unwrap();
        let keys=lower.vertices.clone().map(|v|(rig.neutral.positions[v].map(f32::to_bits),v)).collect::<std::collections::HashMap<_,_>>();
        let pairs=upper.vertices.clone().filter_map(|v|keys.get(&rig.neutral.positions[v].map(f32::to_bits)).map(|&w|(v,w))).collect::<Vec<_>>();
        assert!(pairs.len()>45);seams.extend(pairs);
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Mantine crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Mantine leaves culling bounds");
            if part.name.starts_with("Body /") {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001,"Mantine belly contact height changes");}
        }}
        for &(v,w) in &seams {assert_vec_close(rig.skin_point(v,&matrices),rig.skin_point(w,&matrices),0.0000001);}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for side in ["-1","1"] {
        let wing=rig.anatomy.iter().find(|p|p.name==format!("Wing / curved blue back {side}")).unwrap();
        assert!(wing.vertices.clone().any(|v|rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]>0.20),"Mantine wing does not stroke upward");
    }
    let fish=rig.anatomy.iter().find(|p|p.name.starts_with("Remora / attached")).unwrap();
    let lift=fish.vertices.clone().map(|v|rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]).sum::<f32>()/fish.vertices.len() as f32;
    assert!(lift>0.06,"Remora does not follow the right wing");
}

#[test]
fn corsola_keeps_four_pads_fixed_and_articulates_complete_coral_forks() {
    let rig=fixture(Species::Corsola);
    assert!(std::ptr::eq(super::for_species("CORSOLA").unwrap(),super::rig(Species::Corsola)));
    assert_bits(crate::battle_species_models::mesh("CORSOLA").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Foot /") {0} else if n.starts_with("Coral /") {2+n.split_whitespace().next_back().unwrap().parse::<u16>().unwrap()}
            else if n.starts_with("Side arm /") {if n.ends_with("-1") {7} else {8}} else {1};
        for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}
    }
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).count(),4);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Corsola crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Corsola leaves culling bounds");
            if part.name.starts_with("Foot /") {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Coral / smooth primary tip") || p.name.starts_with("Side arm / rounded tip")) {
        let distance=if part.name.starts_with("Side arm /") {0.025} else {0.005};
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>distance),"{} does not move independently",part.name);
    }
}

#[test]
fn moltres_keeps_talons_planted_and_articulates_coherent_flame_banks() {
    let rig=fixture(Species::Moltres);
    assert!(std::ptr::eq(super::for_species("MOLTRES").unwrap(),super::rig(Species::Moltres)));
    assert_bits(crate::battle_species_models::mesh("MOLTRES").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Toe /","Talon /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if n.starts_with("Wing /") {Some(if n.ends_with("-1") {4} else {5})}
            else if ["Wing fire /","Wing glow /","Wing heart /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {6} else {7})}
            else if n.starts_with("Crown") {Some(8)} else if n.starts_with("Tail") {Some(9)}
            else if n.starts_with("Neck /") {Some(2)}
            else if ["Head /","Eye","Pupil /","Bill"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if n.starts_with("Body /") {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Moltres crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Moltres leaves culling bounds");
            if ["Foot /","Toe /","Talon /"].iter().any(|p|part.name.starts_with(p)) {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]< -0.10),"{} does not stroke",part.name);
    }
    for (prefix,parent) in [("Wing fire / flowing red-orange flame bank -1",4),("Wing fire / flowing red-orange flame bank 1",5),("Crown /",3),("Tail /",1)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005),"{prefix} does not articulate independently");
    }
}

#[test]
fn zapdos_keeps_talons_planted_and_articulates_coherent_thunder_fans() {
    let rig=fixture(Species::Zapdos);
    assert!(std::ptr::eq(super::for_species("ZAPDOS").unwrap(),super::rig(Species::Zapdos)));
    assert_bits(crate::battle_species_models::mesh("ZAPDOS").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Toe /","Talon /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Wing /","Underfeather /","Covert /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Underfeather /") {1} else {0}).unwrap();
                Some(if side=="-1" {4} else {5})
            } else if n.starts_with("Tail /") {Some(6)}
            else if ["Neck /","Throat /"].iter().any(|p|n.starts_with(p)) {Some(2)}
            else if ["Head /","Eye","Pupil /","Bill","Crown /","Cheek /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if ["Body /","Breast feather /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Zapdos crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Zapdos leaves culling bounds");
            if ["Foot /","Toe /","Talon /"].iter().any(|p|part.name.starts_with(p)) {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]< -0.10),"{} does not stroke",part.name);
    }
    let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail /")).unwrap();
    assert!(tail.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005));
}

#[test]
fn articuno_keeps_talons_planted_and_articulates_wings_and_streamer() {
    let rig=fixture(Species::Articuno);
    assert!(std::ptr::eq(super::for_species("ARTICUNO").unwrap(),super::rig(Species::Articuno)));
    assert_bits(crate::battle_species_models::mesh("ARTICUNO").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Toe /","Talon /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if n.starts_with("Wing /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Flight feathers /") {Some(if n.split_whitespace().rev().nth(1)==Some("-1") {4} else {5})}
            else if ["Neck /","Throat /"].iter().any(|p|n.starts_with(p)) {Some(2)}
            else if ["Head /","Eye","Pupil /","Glint /","Bill","Crown /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if ["Body /","Breast /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail /")).unwrap();
    for joint in 6..=8 {assert!(tail.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(&rig.joint_weights[v]).any(|(&j,&w)|j==joint && w>0.05)),"streamer joint {joint} has no influence");}
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Articuno crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Articuno leaves culling bounds");
            if ["Foot /","Toe /","Talon /"].iter().any(|p|part.name.starts_with(p)) {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            if ["Tail /","Tail fold /"].iter().any(|p|part.name.starts_with(p)) {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001,"streamer floor clearance changes");}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).y-rig.neutral.positions[v][1]< -0.10),"{} does not stroke",part.name);
    }
    assert!(tail.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[6].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.005),"streamer curve does not articulate independently");
}

#[test]
fn lugia_keeps_pads_planted_and_articulates_palms_mandible_and_tail() {
    let rig=fixture(Species::Lugia);
    assert!(std::ptr::eq(super::for_species("LUGIA").unwrap(),super::rig(Species::Lugia)));
    assert_bits(crate::battle_species_models::mesh("LUGIA").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Toes /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Wing palm /","Wing digits /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {5} else {7})}
            else if n.starts_with("Tail fin /") {Some(10)}
            else if n.starts_with("Neck /") {Some(2)}
            else if ["Jaw / open","Tongue /"].iter().any(|p|n.starts_with(p)) {Some(8)}
            else if ["Head /","Head crest /","Eye","Mask /","Pupil /","Jaw /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if ["Body /","Belly /","Back plates /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Lugia crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Lugia leaves culling bounds");
            if ["Foot /","Toes /"].iter().any(|p|part.name.starts_with(p)) {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            if ["Tail /","Tail fin /"].iter().any(|p|part.name.starts_with(p)) {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing palm /")) {
        assert!(part.vertices.clone().any(|v|rig.neutral.positions[v][1]-rig.skin_point(v,&matrices).y>0.08),"{} does not stroke",part.name);
    }
    for (prefix,parent) in [("Wing palm / rounded five-finger white fan -1",4),("Wing palm / rounded five-finger white fan 1",6),("Jaw / open",3),("Tail fin /",9)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.003),"{prefix} does not articulate independently");
    }
}
