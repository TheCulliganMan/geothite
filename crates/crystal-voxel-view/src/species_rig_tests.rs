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
        Species::Bayleef | Species::Meganium | Species::Gengar | Species::Spearow | Species::Wooper | Species::Gyarados | Species::Sudowoodo | Species::Lapras | Species::Snorlax | Species::Pikachu | Species::Bulbasaur | Species::Charmander | Species::Squirtle | Species::Geodude | Species::Pidgey | Species::Oddish | Species::Poliwag | Species::Poliwhirl | Species::Poliwrath | Species::Politoed | Species::Sandshrew | Species::Sandslash | Species::Horsea | Species::Seadra | Species::Kingdra | Species::Goldeen | Species::Seaking | Species::Magikarp | Species::Larvitar | Species::Pupitar | Species::Tyranitar | Species::Quilava | Species::Typhlosion | Species::Charmeleon | Species::Charizard | Species::Graveler | Species::Golem | Species::Omanyte | Species::Omastar | Species::Aerodactyl | Species::Dratini | Species::Dragonair | Species::Dragonite | Species::Clefable | Species::Wigglytuff | Species::Rattata | Species::Raticate | Species::Psyduck | Species::Golduck | Species::Zubat | Species::Golbat | Species::Crobat | Species::Butterfree | Species::Beedrill | Species::Hoothoot | Species::Sentret | Species::Staryu | Species::Voltorb | Species::Bellsprout | Species::Weepinbell | Species::Victreebel | Species::Gloom | Species::Vileplume | Species::Raichu | Species::Ekans | Species::Arbok | Species::Steelix | Species::Pineco | Species::Forretress | Species::Kabuto | Species::Kabutops | Species::Electrode | Species::Onix | Species::Diglett | Species::Dugtrio | Species::Tentacool | Species::Tentacruel | Species::Jynx | Species::Mankey | Species::Primeape | Species::Meowth | Species::Persian | Species::Doduo | Species::Dodrio | Species::Pidgeotto | Species::FarfetchD | Species::Ivysaur | Species::Venusaur | Species::Exeggcute | Species::Exeggutor | Species::Fearow | Species::Pidgeot | Species::Slowpoke | Species::Entei | Species::Raikou | Species::Suicune | Species::Furret | Species::Noctowl | Species::Ledyba | Species::Ledian | Species::Spinarak | Species::Ariados | Species::Quagsire | Species::Hoppip | Species::Bellossom | Species::Qwilfish | Species::Wobbuffet | Species::Dunsparce | Species::Shuckle | Species::Gligar | Species::Yanma | Species::Murkrow | Species::Misdreavus | Species::Teddiursa | Species::Ursaring | Species::Snubbull | Species::Granbull | Species::Houndour | Species::Houndoom | Species::Miltank | Species::Tauros | Species::Delibird | Species::Stantler | Species::Girafarig | Species::Phanpy | Species::Marill | Species::Natu | Species::Remoraid | Species::Octillery | Species::Swinub | Species::Piloswine | Species::Sneasel | Species::Skarmory | Species::Smeargle | Species::Mew | Species::Mewtwo | Species::Heracross | Species::Pinsir | Species::Scizor | Species::Scyther | Species::Starmie | Species::Gastly | Species::Haunter | Species::Eevee | Species::Vulpix | Species::Ninetales | Species::Venonat | Species::Venomoth | Species::Paras | Species::Parasect | Species::Metapod | Species::Kakuna | Species::Caterpie | Species::Weedle | Species::Drowzee | Species::Hypno | Species::Abra | Species::Alakazam | Species::Kadabra | Species::Koffing | Species::Weezing | Species::Wartortle | Species::Blastoise | Species::Croconaw | Species::Feraligatr | Species::Cubone | Species::Marowak | Species::Magnemite | Species::Magneton | Species::Tangela | Species::Kangaskhan | Species::Ponyta | Species::Rapidash | Species::Machop | Species::Machoke | Species::Machamp | Species::Rhyhorn | Species::Rhydon | Species::Seel | Species::Dewgong | Species::Grimer | Species::Muk | Species::Ditto | Species::Shellder | Species::Cloyster | Species::Krabby | Species::Kingler | Species::Umbreon | Species::Jolteon | Species::Flareon | Species::Espeon | Species::Vaporeon | Species::Celebi | Species::HoOh | Species::Lugia | Species::Articuno | Species::Zapdos | Species::Moltres | Species::Corsola | Species::Mantine | Species::Porygon | Species::Porygon2 | Species::Chinchou | Species::Lanturn | Species::Slugma | Species::Magcargo | Species::Togepi | Species::Togetic | Species::Hitmonlee | Species::Hitmonchan | Species::Hitmontop | Species::Tyrogue | Species::Smoochum | Species::Elekid | Species::Magby | Species::Magmar | Species::Electabuzz | Species::Slowbro | Species::Slowking | Species::Lickitung | Species::MrMime | Species::Chansey | Species::Blissey | Species::Growlithe | Species::Arcanine | Species::NidoranF | Species::NidoranM | Species::Nidorina | Species::Nidorino | Species::Nidoqueen | Species::Nidoking | Species::Pichu | Species::Cleffa | Species::Igglybuff | Species::Mareep | Species::Flaaffy | Species::Ampharos | Species::Xatu | Species::Azumarill | Species::Donphan | Species::Sunkern | Species::Sunflora | Species::Aipom | Species::Skiploom | Species::Jumpluff | Species::Unown(_) => {
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
    static BELLSPROUT: OnceLock<SpeciesRig> = OnceLock::new();
    static WEEPINBELL: OnceLock<SpeciesRig> = OnceLock::new();
    static VICTREEBEL: OnceLock<SpeciesRig> = OnceLock::new();
    static GLOOM: OnceLock<SpeciesRig> = OnceLock::new();
    static VILEPLUME: OnceLock<SpeciesRig> = OnceLock::new();
    static RAICHU: OnceLock<SpeciesRig> = OnceLock::new();
    static EKANS: OnceLock<SpeciesRig> = OnceLock::new();
    static ARBOK: OnceLock<SpeciesRig> = OnceLock::new();
    static STEELIX: OnceLock<SpeciesRig> = OnceLock::new();

    static PINECO: OnceLock<SpeciesRig> = OnceLock::new();
    static FORRETRESS: OnceLock<SpeciesRig> = OnceLock::new();
    static KABUTO: OnceLock<SpeciesRig> = OnceLock::new();
    static KABUTOPS: OnceLock<SpeciesRig> = OnceLock::new();
    static ELECTRODE: OnceLock<SpeciesRig> = OnceLock::new();
    static ONIX: OnceLock<SpeciesRig> = OnceLock::new();
    static DIGLETT: OnceLock<SpeciesRig> = OnceLock::new();
    static DUGTRIO: OnceLock<SpeciesRig> = OnceLock::new();
    static TENTACRUEL: OnceLock<SpeciesRig> = OnceLock::new();
    static TENTACOOL: OnceLock<SpeciesRig> = OnceLock::new();
    static JYNX: OnceLock<SpeciesRig> = OnceLock::new();
    static PRIMEAPE: OnceLock<SpeciesRig> = OnceLock::new();
    static MANKEY: OnceLock<SpeciesRig> = OnceLock::new();
    static MEOWTH: OnceLock<SpeciesRig> = OnceLock::new();
    static PERSIAN: OnceLock<SpeciesRig> = OnceLock::new();
    static DODUO: OnceLock<SpeciesRig> = OnceLock::new();
    static DODRIO: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEOTTO: OnceLock<SpeciesRig> = OnceLock::new();
    static EXEGGCUTE: OnceLock<SpeciesRig> = OnceLock::new();
    static EXEGGUTOR: OnceLock<SpeciesRig> = OnceLock::new();
    static IVYSAUR: OnceLock<SpeciesRig> = OnceLock::new();
    static VENUSAUR: OnceLock<SpeciesRig> = OnceLock::new();
    static FARFETCH_D: OnceLock<SpeciesRig> = OnceLock::new();
    static FEAROW: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEOT: OnceLock<SpeciesRig> = OnceLock::new();
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
    static MEW: OnceLock<SpeciesRig> = OnceLock::new();
    static MEWTWO: OnceLock<SpeciesRig> = OnceLock::new();
    static HERACROSS: OnceLock<SpeciesRig> = OnceLock::new();
    static PINSIR: OnceLock<SpeciesRig> = OnceLock::new();
    static SCIZOR: OnceLock<SpeciesRig> = OnceLock::new();
    static SCYTHER: OnceLock<SpeciesRig> = OnceLock::new();
    static STARMIE: OnceLock<SpeciesRig> = OnceLock::new();
    static GASTLY: OnceLock<SpeciesRig> = OnceLock::new();
    static HAUNTER: OnceLock<SpeciesRig> = OnceLock::new();
    static EEVEE: OnceLock<SpeciesRig> = OnceLock::new();
    static VULPIX: OnceLock<SpeciesRig> = OnceLock::new();
    static NINETALES: OnceLock<SpeciesRig> = OnceLock::new();
    static VENONAT: OnceLock<SpeciesRig> = OnceLock::new();
    static VENOMOTH: OnceLock<SpeciesRig> = OnceLock::new();
    static PARAS: OnceLock<SpeciesRig> = OnceLock::new();
    static PARASECT: OnceLock<SpeciesRig> = OnceLock::new();
    static METAPOD: OnceLock<SpeciesRig> = OnceLock::new();
    static KAKUNA: OnceLock<SpeciesRig> = OnceLock::new();
    static CATERPIE: OnceLock<SpeciesRig> = OnceLock::new();
    static WEEDLE: OnceLock<SpeciesRig> = OnceLock::new();
    static RHYDON: OnceLock<SpeciesRig> = OnceLock::new();
    static DROWZEE: OnceLock<SpeciesRig> = OnceLock::new();
    static HYPNO: OnceLock<SpeciesRig> = OnceLock::new();
    static ABRA: OnceLock<SpeciesRig> = OnceLock::new();
    static ALAKAZAM: OnceLock<SpeciesRig> = OnceLock::new();
    static KADABRA: OnceLock<SpeciesRig> = OnceLock::new();
    static KOFFING: OnceLock<SpeciesRig> = OnceLock::new();
    static WEEZING: OnceLock<SpeciesRig> = OnceLock::new();
    static WARTORTLE: OnceLock<SpeciesRig> = OnceLock::new();
    static BLASTOISE: OnceLock<SpeciesRig> = OnceLock::new();
    static CROCONAW: OnceLock<SpeciesRig> = OnceLock::new();
    static FERALIGATR: OnceLock<SpeciesRig> = OnceLock::new();
    static CUBONE: OnceLock<SpeciesRig> = OnceLock::new();
    static MAROWAK: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGNEMITE: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGNETON: OnceLock<SpeciesRig> = OnceLock::new();
    static TANGELA: OnceLock<SpeciesRig> = OnceLock::new();
    static KANGASKHAN: OnceLock<SpeciesRig> = OnceLock::new();
    static PONYTA: OnceLock<SpeciesRig> = OnceLock::new();
    static RAPIDASH: OnceLock<SpeciesRig> = OnceLock::new();
    static MACHOP: OnceLock<SpeciesRig> = OnceLock::new();
    static MACHOKE: OnceLock<SpeciesRig> = OnceLock::new();
    static MACHAMP: OnceLock<SpeciesRig> = OnceLock::new();
    static RHYHORN: OnceLock<SpeciesRig> = OnceLock::new();
    static DEWGONG: OnceLock<SpeciesRig> = OnceLock::new();
    static SEEL: OnceLock<SpeciesRig> = OnceLock::new();
    static DITTO: OnceLock<SpeciesRig> = OnceLock::new();
    static MUK: OnceLock<SpeciesRig> = OnceLock::new();
    static GRIMER: OnceLock<SpeciesRig> = OnceLock::new();
    static CLOYSTER: OnceLock<SpeciesRig> = OnceLock::new();
    static SHELLDER: OnceLock<SpeciesRig> = OnceLock::new();
    static KINGLER: OnceLock<SpeciesRig> = OnceLock::new();
    static KRABBY: OnceLock<SpeciesRig> = OnceLock::new();
    static UMBREON: OnceLock<SpeciesRig> = OnceLock::new();
    static JOLTEON: OnceLock<SpeciesRig> = OnceLock::new();
    static FLAREON: OnceLock<SpeciesRig> = OnceLock::new();
    static ESPEON: OnceLock<SpeciesRig> = OnceLock::new();
    static VAPOREON: OnceLock<SpeciesRig> = OnceLock::new();
    static CELEBI: OnceLock<SpeciesRig> = OnceLock::new();
    static HO_OH: OnceLock<SpeciesRig> = OnceLock::new();
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
    static MAGMAR: OnceLock<SpeciesRig> = OnceLock::new();
    static ELECTABUZZ: OnceLock<SpeciesRig> = OnceLock::new();
    static SLOWBRO: OnceLock<SpeciesRig> = OnceLock::new();
    static LICKITUNG: OnceLock<SpeciesRig> = OnceLock::new();
    static MR_MIME: OnceLock<SpeciesRig> = OnceLock::new();
    static CHANSEY: OnceLock<SpeciesRig> = OnceLock::new();
    static BLISSEY: OnceLock<SpeciesRig> = OnceLock::new();
    static GROWLITHE: OnceLock<SpeciesRig> = OnceLock::new();
    static ARCANINE: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORAN_F: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORAN_M: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORINA: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORINO: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDOKING: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDOQUEEN: OnceLock<SpeciesRig> = OnceLock::new();
    static SLOWKING: OnceLock<SpeciesRig> = OnceLock::new();
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
    static HORSEA: OnceLock<SpeciesRig> = OnceLock::new();
    static SEADRA: OnceLock<SpeciesRig> = OnceLock::new();
    static KINGDRA: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLDEEN: OnceLock<SpeciesRig> = OnceLock::new();
    static SEAKING: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGIKARP: OnceLock<SpeciesRig> = OnceLock::new();
    static LARVITAR: OnceLock<SpeciesRig> = OnceLock::new();
    static PUPITAR: OnceLock<SpeciesRig> = OnceLock::new();
    static TYRANITAR: OnceLock<SpeciesRig> = OnceLock::new();
    static QUILAVA: OnceLock<SpeciesRig> = OnceLock::new();
    static TYPHLOSION: OnceLock<SpeciesRig> = OnceLock::new();
    static CHARMELEON: OnceLock<SpeciesRig> = OnceLock::new();
    static CHARIZARD: OnceLock<SpeciesRig> = OnceLock::new();
    static GRAVELER: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLEM: OnceLock<SpeciesRig> = OnceLock::new();
    static OMANYTE: OnceLock<SpeciesRig> = OnceLock::new();
    static OMASTAR: OnceLock<SpeciesRig> = OnceLock::new();
    static AERODACTYL: OnceLock<SpeciesRig> = OnceLock::new();
    static DRATINI: OnceLock<SpeciesRig> = OnceLock::new();
    static DRAGONAIR: OnceLock<SpeciesRig> = OnceLock::new();
    static DRAGONITE: OnceLock<SpeciesRig> = OnceLock::new();
    static CLEFABLE: OnceLock<SpeciesRig> = OnceLock::new();
    static WIGGLYTUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static SANDSHREW: OnceLock<SpeciesRig> = OnceLock::new();
    static SANDSLASH: OnceLock<SpeciesRig> = OnceLock::new();
    static POLITOED: OnceLock<SpeciesRig> = OnceLock::new();
    static POLIWRATH: OnceLock<SpeciesRig> = OnceLock::new();
    static POLIWHIRL: OnceLock<SpeciesRig> = OnceLock::new();
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
        Species::Horsea => &HORSEA,
        Species::Seadra => &SEADRA,
        Species::Kingdra => &KINGDRA,
        Species::Goldeen => &GOLDEEN,
        Species::Seaking => &SEAKING,
        Species::Magikarp => &MAGIKARP,
        Species::Larvitar => &LARVITAR,
        Species::Pupitar => &PUPITAR,
        Species::Tyranitar => &TYRANITAR,
        Species::Quilava => &QUILAVA,
        Species::Typhlosion => &TYPHLOSION,
        Species::Charmeleon => &CHARMELEON,
        Species::Charizard => &CHARIZARD,
        Species::Graveler => &GRAVELER,
        Species::Golem => &GOLEM,
        Species::Omanyte => &OMANYTE,
        Species::Omastar => &OMASTAR,
        Species::Aerodactyl => &AERODACTYL,
        Species::Dratini => &DRATINI,
        Species::Dragonair => &DRAGONAIR,
        Species::Dragonite => &DRAGONITE,
        Species::Clefable => &CLEFABLE,
        Species::Wigglytuff => &WIGGLYTUFF,
        Species::Sandshrew => &SANDSHREW,
        Species::Sandslash => &SANDSLASH,
        Species::Politoed => &POLITOED,
        Species::Poliwrath => &POLIWRATH,
        Species::Poliwhirl => &POLIWHIRL,
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
        Species::Bellsprout => &BELLSPROUT,
        Species::Weepinbell => &WEEPINBELL,
        Species::Victreebel => &VICTREEBEL,
        Species::Gloom => &GLOOM,
        Species::Vileplume => &VILEPLUME,
        Species::Raichu => &RAICHU,
        Species::Ekans => &EKANS,
        Species::Arbok => &ARBOK,
        Species::Steelix => &STEELIX,

        Species::Pineco => &PINECO,
        Species::Forretress => &FORRETRESS,
        Species::Kabuto => &KABUTO,
        Species::Kabutops => &KABUTOPS,
        Species::Electrode => &ELECTRODE,
        Species::Onix => &ONIX,
        Species::Diglett => &DIGLETT,
        Species::Dugtrio => &DUGTRIO,
        Species::Tentacruel => &TENTACRUEL,
        Species::Tentacool => &TENTACOOL,
        Species::Jynx => &JYNX,
        Species::Primeape => &PRIMEAPE,
        Species::Mankey => &MANKEY,
        Species::Meowth => &MEOWTH,
        Species::Persian => &PERSIAN,
        Species::Doduo => &DODUO,
        Species::Dodrio => &DODRIO,
        Species::Pidgeotto => &PIDGEOTTO,
        Species::Exeggcute => &EXEGGCUTE,
        Species::Exeggutor => &EXEGGUTOR,
        Species::Ivysaur => &IVYSAUR,
        Species::Venusaur => &VENUSAUR,
        Species::FarfetchD => &FARFETCH_D,
        Species::Fearow => &FEAROW,
        Species::Pidgeot => &PIDGEOT,
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
        Species::Mew => &MEW,
        Species::Mewtwo => &MEWTWO,
        Species::Heracross => &HERACROSS,
        Species::Pinsir => &PINSIR,
        Species::Scizor => &SCIZOR,
        Species::Scyther => &SCYTHER,
        Species::Starmie => &STARMIE,
        Species::Gastly => &GASTLY,
        Species::Haunter => &HAUNTER,
        Species::Eevee => &EEVEE,
        Species::Vulpix => &VULPIX,
        Species::Ninetales => &NINETALES,
        Species::Venonat => &VENONAT,
        Species::Venomoth => &VENOMOTH,
        Species::Paras => &PARAS,
        Species::Parasect => &PARASECT,
        Species::Metapod => &METAPOD,
        Species::Kakuna => &KAKUNA,
        Species::Caterpie => &CATERPIE,
        Species::Weedle => &WEEDLE,
        Species::Rhydon => &RHYDON,
        Species::Drowzee => &DROWZEE,
        Species::Hypno => &HYPNO,
        Species::Abra => &ABRA,
        Species::Alakazam => &ALAKAZAM,
        Species::Kadabra => &KADABRA,
        Species::Koffing => &KOFFING,
        Species::Weezing => &WEEZING,
        Species::Wartortle => &WARTORTLE,
        Species::Blastoise => &BLASTOISE,
        Species::Croconaw => &CROCONAW,
        Species::Feraligatr => &FERALIGATR,
        Species::Cubone => &CUBONE,
        Species::Marowak => &MAROWAK,
        Species::Magnemite => &MAGNEMITE,
        Species::Magneton => &MAGNETON,
        Species::Tangela => &TANGELA,
        Species::Kangaskhan => &KANGASKHAN,
        Species::Ponyta => &PONYTA,
        Species::Rapidash => &RAPIDASH,
        Species::Machop => &MACHOP,
        Species::Machoke => &MACHOKE,
        Species::Machamp => &MACHAMP,
        Species::Rhyhorn => &RHYHORN,
        Species::Dewgong => &DEWGONG,
        Species::Seel => &SEEL,
        Species::Ditto => &DITTO,
        Species::Muk => &MUK,
        Species::Grimer => &GRIMER,
        Species::Cloyster => &CLOYSTER,
        Species::Shellder => &SHELLDER,
        Species::Kingler => &KINGLER,
        Species::Krabby => &KRABBY,
        Species::Umbreon => &UMBREON,
        Species::Jolteon => &JOLTEON,
        Species::Flareon => &FLAREON,
        Species::Espeon => &ESPEON,
        Species::Vaporeon => &VAPOREON,
        Species::Celebi => &CELEBI,
        Species::HoOh => &HO_OH,
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
        Species::Magmar => &MAGMAR,
        Species::Electabuzz => &ELECTABUZZ,
        Species::Slowking => &SLOWKING,
        Species::Slowbro => &SLOWBRO,
        Species::Lickitung => &LICKITUNG,
        Species::MrMime => &MR_MIME,
        Species::Chansey => &CHANSEY,
        Species::Blissey => &BLISSEY,
        Species::Growlithe => &GROWLITHE,
        Species::Arcanine => &ARCANINE,
        Species::NidoranF => &NIDORAN_F,
        Species::NidoranM => &NIDORAN_M,
        Species::Nidorina => &NIDORINA,
        Species::Nidorino => &NIDORINO,
        Species::Nidoking => &NIDOKING,
        Species::Nidoqueen => &NIDOQUEEN,
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
        (Species::Chikorita, 5108, 3004, 1.0_f32),
        (Species::Bayleef, 6800, 4360, 1.15_f32),
        (Species::Meganium, 13870, 7148, 1.48_f32),
        (Species::Cyndaquil, 16559, 8610, 1.25_f32),
        (Species::Totodile, 5422, 10552, 1.02_f32),
        (Species::Gengar, 6952, 5560, 1.05_f32),
        (Species::Spearow, 3703, 2848, 0.75_f32),
        (Species::Wooper, 3991, 5532, 0.70_f32),
        (Species::Gyarados, 9187, 11666, 1.40_f32),
        (Species::Sudowoodo, 9089, 11376, 1.04_f32),
        (Species::Lapras, 19313, 21968, 1.50_f32),
        (Species::Snorlax, 9905, 17584, 1.40_f32),
        (Species::Pikachu, 11568, 13876, 1.00_f32),
        (Species::Bulbasaur, 19587, 31896, 0.95_f32),
        (Species::Charmander, 13250, 21086, 1.00_f32),
        (Species::Squirtle, 17298, 26948, 0.96_f32),
        (Species::Geodude, 11598, 13932, 0.76_f32),
        (Species::Pidgey, 8757, 12600, 0.82_f32),
        (Species::Oddish, 8845, 14620, 1.00_f32),
        (Species::Poliwag, 15265, 25532, 0.90_f32),
        (Species::Horsea, 15790, 24912, 1.07_f32),
        (Species::Seadra, 13214, 20324, 1.07_f32),
        (Species::Kingdra, 19154, 31156, 1.07_f32),
        (Species::Goldeen, 16748, 20668, 0.8_f32),
        (Species::Seaking, 19923, 24272, 1.0_f32),
        (Species::Magikarp, 15922, 19240, 0.9_f32),
        (Species::Larvitar, 7394, 10816, 1.04_f32),
        (Species::Pupitar, 4503, 5224, 1.2_f32),
        (Species::Tyranitar, 12272, 18056, 1.4_f32),
        (Species::Quilava, 10428, 13428, 1.2_f32),
        (Species::Typhlosion, 11439, 15262, 1.45_f32),
        (Species::Charmeleon, 10656, 17122, 1.2_f32),
        (Species::Charizard, 17482, 26850, 1.4_f32),
        (Species::Graveler, 8670, 7928, 1.1_f32),
        (Species::Golem, 19722, 30140, 1.25_f32),
        (Species::Omanyte, 12587, 18848, 0.8_f32),
        (Species::Omastar, 14178, 21292, 1.08_f32),
        (Species::Aerodactyl, 16822, 25888, 1.3_f32),
        (Species::Dratini, 10976, 18524, 1.18_f32),
        (Species::Dragonair, 14274, 24676, 1.18_f32),
        (Species::Dragonite, 18499, 28652, 1.4_f32),
        (Species::Clefable, 15603, 27448, 1.0_f32),
        (Species::Wigglytuff, 18648, 31548, 1.0_f32),
        (Species::Sandshrew, 14089, 22600, 1.02_f32),
        (Species::Sandslash, 15013, 26080, 1.18_f32),
        (Species::Politoed, 15317, 27556, 1.2_f32),
        (Species::Poliwrath, 11964, 21420, 1.2_f32),
        (Species::Poliwhirl, 11464, 20648, 1.1_f32),
        (Species::Clefairy, 15257, 24216, 0.82_f32),
        (Species::Jigglypuff, 13706, 22956, 0.82_f32),
        (Species::Rattata, 11494, 20432, 0.82_f32),
        (Species::Raticate, 5642, 7764, 0.96_f32),
        (Species::Psyduck, 3754, 3828, 1.03_f32),
        (Species::Golduck, 4974, 4460, 1.03_f32),
        (Species::Zubat, 2368, 3352, 1.0_f32),
        (Species::Golbat, 3762, 4784, 1.0_f32),
        (Species::Crobat, 4653, 4116, 1.0_f32),
        (Species::Butterfree, 5428, 9420, 1.04_f32),
        (Species::Beedrill, 4304, 7304, 1.04_f32),
        (Species::Hoothoot, 7029, 8684, 1.0_f32),
        (Species::Sentret, 14430, 13752, 1.0_f32),
        (Species::Staryu, 1982, 2852, 1.0_f32),
        (Species::Voltorb, 4660, 8248, 0.82_f32),
        (Species::Bellsprout, 3001, 4728, 1.05_f32),
        (Species::Weepinbell, 3855, 5700, 1.05_f32),
        (Species::Victreebel, 4116, 6232, 1.05_f32),
        (Species::Gloom, 9063, 13848, 0.85_f32),
        (Species::Vileplume, 8644, 12508, 1.12_f32),
        (Species::Raichu, 11400, 18848, 0.96_f32),
        (Species::Ekans, 8481, 14536, 1.18_f32),
        (Species::Arbok, 11554, 18900, 1.18_f32),
        (Species::Steelix, 10405, 9244, 1.45_f32),

        (Species::Pineco, 5418, 3900, 0.93_f32),
        (Species::Forretress, 15039, 14524, 0.93_f32),
        (Species::Kabuto, 5970, 9356, 0.60_f32),
        (Species::Kabutops, 8050, 12326, 1.20_f32),
        (Species::Electrode, 17210, 23480, 0.94_f32),
        (Species::Onix, 12083, 9848, 1.72_f32),
        (Species::Diglett, 10646, 14064, 0.82_f32),
        (Species::Dugtrio, 12018, 13584, 0.98_f32),
        (Species::Tentacruel, 7641, 12260, 1.13_f32),
        (Species::Tentacool, 4519, 7540, 1.13_f32),
        (Species::Jynx, 18473, 30004, 1.28_f32),
        (Species::Primeape, 13633, 23004, 1.02_f32),
        (Species::Mankey, 11992, 19964, 1.02_f32),
        (Species::Meowth, 12615, 20788, 1.01_f32),
        (Species::Persian, 11977, 20220, 1.01_f32),
        (Species::Doduo, 11294, 18988, 1.22_f32),
        (Species::Dodrio, 19801, 33196, 1.22_f32),
        (Species::Pidgeotto, 9353, 14116, 1.0_f32),
        (Species::Exeggcute, 19868, 35472, 1.22_f32),
        (Species::Exeggutor, 19278, 30396, 1.22_f32),
        (Species::Ivysaur, 18609, 29796, 1.20_f32),
        (Species::Venusaur, 19789, 31500, 1.20_f32),
        (Species::FarfetchD, 17863, 26972, 1.10_f32),
        (Species::Fearow, 13219, 16748, 1.23_f32),
        (Species::Pidgeot, 18266, 24648, 1.35_f32),
        (Species::Slowpoke, 19574, 32984, 1.0_f32),
        (Species::Entei, 15542, 26368, 1.42_f32),
        (Species::Raikou, 18482, 28632, 1.30_f32),
        (Species::Suicune, 10500, 17216, 1.60_f32),
        (Species::Furret, 17037, 10040, 1.20_f32),
        (Species::Noctowl, 7706, 10076, 1.30_f32),
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
        (Species::Granbull, 10841, 14536, 1.06_f32),
        (Species::Houndour, 11024, 14248, 1.20_f32),
        (Species::Houndoom, 15865, 22736, 1.20_f32),
        (Species::Miltank, 17281, 17976, 1.12_f32),
        (Species::Tauros, 14025, 16504, 1.28_f32),
        (Species::Delibird, 9916, 17808, 1.10_f32),
        (Species::Stantler, 13113, 12428, 1.28_f32),
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
        (Species::Sneasel, 8992, 8736, 1.11_f32),
        (Species::Skarmory, 3635, 4288, 1.21_f32),
        (Species::Smeargle, 16875, 21004, 1.14_f32),
        (Species::Moltres, 19671, 26024, 1.28_f32),
        (Species::Zapdos, 18495, 28584, 1.28_f32),
        (Species::Articuno, 19074, 26704, 1.28_f32),
        (Species::Mew, 18486, 30224, 0.78_f32),
        (Species::Mewtwo, 15722, 27880, 1.45_f32),
        (Species::Heracross, 19588, 32148, 1.16_f32),
        (Species::Pinsir, 14082, 23756, 1.16_f32),
        (Species::Scizor, 18558, 27348, 1.28_f32),
        (Species::Scyther, 13780, 21460, 1.18_f32),
        (Species::Starmie, 2966, 3344, 1.03_f32),
        (Species::Gastly, 18911, 26448, 1.05_f32),
        (Species::Haunter, 9833, 13444, 1.10_f32),
        (Species::Eevee, 10813, 19076, 0.80_f32),
        (Species::Vulpix, 13624, 22736, 0.85_f32),
        (Species::Ninetales, 14231, 23784, 1.30_f32),
        (Species::Venonat, 15723, 22268, 1.10_f32),
        (Species::Venomoth, 19412, 30580, 1.16_f32),
        (Species::Paras, 19755, 31860, 0.78_f32),
        (Species::Parasect, 13877, 22168, 1.14_f32),
        (Species::Metapod, 10664, 15788, 0.80_f32),
        (Species::Kakuna, 11069, 16336, 0.80_f32),
        (Species::Caterpie, 13539, 25360, 0.95_f32),
        (Species::Weedle, 10537, 19772, 0.95_f32),
        (Species::Rhydon, 17699, 27280, 1.4_f32),
        (Species::Drowzee, 19170, 32056, 1.12_f32),
        (Species::Hypno, 19105, 32320, 1.12_f32),
        (Species::Abra, 7254, 12040, 1.12_f32),
        (Species::Alakazam, 7513, 12264, 1.12_f32),
        (Species::Kadabra, 4776, 5848, 1.12_f32),
        (Species::Koffing, 10563, 18264, 1.0_f32),
        (Species::Weezing, 16974, 28756, 1.2_f32),
        (Species::Wartortle, 19000, 30760, 1.1_f32),
        (Species::Blastoise, 19603, 32188, 1.18_f32),
        (Species::Croconaw, 14789, 23092, 1.30_f32),
        (Species::Feraligatr, 17447, 25520, 1.30_f32),
        (Species::Cubone, 19943, 34972, 1.0_f32),
        (Species::Marowak, 19716, 32924, 1.18_f32),
        (Species::Magnemite, 5966, 9140, 0.96_f32),
        (Species::Magneton, 17898, 27420, 0.96_f32),
        (Species::Tangela, 17318, 31236, 1.0_f32),
        (Species::Kangaskhan, 18743, 31108, 1.4_f32),
        (Species::Ponyta, 15002, 22280, 1.1_f32),
        (Species::Rapidash, 17555, 26744, 1.3_f32),
        (Species::Machop, 11179, 16304, 0.85_f32),
        (Species::Machoke, 14694, 21812, 1.15_f32),
        (Species::Machamp, 16806, 24648, 1.45_f32),
        (Species::Rhyhorn, 11214, 15636, 1.11_f32),
        (Species::Dewgong, 15206, 23740, 1.03_f32),
        (Species::Seel, 13304, 21680, 1.03_f32),
        (Species::Ditto, 9310, 13980, 0.96_f32),
        (Species::Muk, 16163, 21396, 0.96_f32),
        (Species::Grimer, 18739, 25172, 0.96_f32),
        (Species::Cloyster, 17021, 23892, 1.05_f32),
        (Species::Shellder, 15585, 22292, 1.05_f32),
        (Species::Kingler, 18026, 25908, 0.97_f32),
        (Species::Krabby, 17883, 25576, 0.97_f32),
        (Species::Umbreon, 12563, 21084, 1.05_f32),
        (Species::Jolteon, 10892, 17840, 1.05_f32),
        (Species::Flareon, 11495, 20424, 0.97_f32),
        (Species::Espeon, 10809, 19164, 1.00_f32),
        (Species::Vaporeon, 13565, 23164, 1.00_f32),
        (Species::Celebi, 14678, 23056, 0.98_f32),
        (Species::HoOh, 19747, 25200, 1.45_f32),
        (Species::Lugia, 19823, 34796, 1.45_f32),
        (Species::Corsola, 7359, 14180, 0.93_f32),
        (Species::Mantine, 19485, 27416, 1.02_f32),
        (Species::Porygon, 4789, 2008, 1.05_f32),
        (Species::Porygon2, 14189, 23912, 1.05_f32),
        (Species::Chinchou, 12754, 11116, 0.95_f32),
        (Species::Lanturn, 17971, 15572, 0.95_f32),
        (Species::Slugma, 2290, 4108, 0.98_f32),
        (Species::Magcargo, 5554, 10276, 0.98_f32),
        (Species::Hitmonlee, 15445, 25744, 1.24_f32),
        (Species::Hitmonchan, 10981, 16084, 1.30_f32),
        (Species::Hitmontop, 10934, 17604, 1.25_f32),
        (Species::Tyrogue, 13839, 21596, 0.86_f32),
        (Species::Smoochum, 17939, 30344, 0.78_f32),
        (Species::Elekid, 11517, 19024, 0.91_f32),
        (Species::Magby, 12864, 22336, 0.77_f32),
        (Species::Magmar, 19024, 29200, 1.30_f32),
        (Species::Electabuzz, 19747, 33880, 1.28_f32),
        (Species::Slowking, 18831, 31512, 1.25_f32),
        (Species::Slowbro, 19322, 32532, 1.2_f32),
        (Species::Lickitung, 17028, 29256, 1.18_f32),
        (Species::MrMime, 19916, 33656, 1.20_f32),
        (Species::Chansey, 10390, 18704, 1.10_f32),
        (Species::Blissey, 19379, 34472, 1.20_f32),
        (Species::Growlithe, 16302, 26140, 0.90_f32),
        (Species::Arcanine, 19539, 31404, 1.30_f32),
        (Species::NidoranF, 19256, 31768, 0.78_f32),
        (Species::NidoranM, 16782, 28160, 0.80_f32),
        (Species::Nidorina, 19478, 32064, 1.12_f32),
        (Species::Nidorino, 19860, 32172, 1.12_f32),
        (Species::Nidoking, 19990, 32920, 1.4_f32),
        (Species::Nidoqueen, 18397, 30024, 1.4_f32),
        (Species::Pichu, 9462, 9104, 0.62_f32),
        (Species::Cleffa, 14716, 24376, 0.6_f32),
        (Species::Igglybuff, 16964, 28684, 0.58_f32),
        (Species::Mareep, 6382, 11840, 0.85_f32),
        (Species::Flaaffy, 10302, 16636, 1.0_f32),
        (Species::Ampharos, 3806, 5804, 1.35_f32),
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
    assert_eq!(rig.neutral_bounds.0, Vec3::new(-0.2993245125, 0.0, -0.4881474078));
    assert_eq!(rig.neutral_bounds.1, Vec3::new(0.2993245125, 0.75, 0.3335007131));

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
        assert_idle_loop_closes(fixture(species));
    }
}

fn assert_idle_loop_closes(rig: &SpeciesRig) {
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
            let actual = rig.skin_point(i, &matrices);
            let neutral = Vec3::from_array(*p);
            assert!(
                (actual - neutral).abs().max_element() <= 0.00001,
                "{} idle t={time} vertex {i}: {actual:?} differs from {neutral:?}",
                rig.species.name()
            );
        }
    }
}

#[test]
fn dragon_pair_stored_idle_closes_at_production_time_offsets() {
    for species in [Species::Charizard, Species::Dragonite] {
        assert_idle_loop_closes(fixture(species));
    }
}

#[test]
fn pidgey_family_stored_idle_closes_at_production_time_offsets() {
    for species in [Species::Pidgey, Species::Pidgeotto, Species::Pidgeot] {
        assert_idle_loop_closes(fixture(species));
    }
}

#[test]
fn spearow_fearow_stored_idle_closes_at_production_time_offsets() {
    for species in [Species::Spearow, Species::Fearow] {
        assert_idle_loop_closes(fixture(species));
    }
}

#[test]
fn sentry_ferret_stored_idle_closes_at_production_time_offsets() {
    for species in [Species::Sentret, Species::Furret] {
        assert_idle_loop_closes(fixture(species));
    }
}

#[test]
fn owl_stored_idle_closes_at_production_time_offsets() {
    for species in [Species::Hoothoot, Species::Noctowl] {
        assert_idle_loop_closes(fixture(species));
    }
}

#[test]
fn fossil_pair_stored_idle_closes_at_production_time_offsets() {
    for species in [Species::Aerodactyl, Species::Kabutops] {
        assert_idle_loop_closes(fixture(species));
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
    assert!(super::authored_for_species("UNOWN").is_none(), "a DV form requires an explicit authored identity");
    assert!(std::ptr::eq(super::for_species("UNOWN").unwrap(), super::rig(Species::Unown(8))),
        "the world roster retains its explicit generic Unown-to-I binding");
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
                else if ["Jaw /","Tusk /","Tongue /"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
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
            else if ["Head /","Eye /","Brow /","Brow spot /","Muzzle /","Muzzle pad /","Nostril /","Antler"].iter().any(|p|part.name.starts_with(p)) {Some(3)}
            else if part.name.starts_with("Ear /") {Some(if part.name.ends_with("-1") {4} else {5})}
            else if part.name.starts_with("Tail / large") {Some(11)}
            else if ["Body /","Spot /","Rib /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
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
                else if ["Head /","Helmet flange /","Eye /","Pupil /","Helmet crease /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(1)}
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
            else if n.starts_with("Ear /") {Some(if n.ends_with("-1") {4} else {5})} else if n.starts_with("Plume /") {Some(5)}
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

#[test]
fn ho_oh_keeps_hooks_planted_and_articulates_tricolor_wings_roar_and_tail() {
    let rig=fixture(Species::HoOh);
    assert!(std::ptr::eq(super::for_species("HO_OH").unwrap(),super::rig(Species::HoOh)));
    assert_bits(crate::battle_species_models::mesh("HO_OH").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Toes /","Talons /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Wing fan /","Wing ivory /","Wing green /","Wing coverts /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {4} else {5})}
            else if ["Tail fan /","Tail tint /"].iter().any(|p|n.starts_with(p)) {Some(6)}
            else if ["Neck /","Collar /","Collar edge /"].iter().any(|p|n.starts_with(p)) {Some(2)}
            else if n.starts_with("Jaw / open") {Some(7)}
            else if ["Head /","Eye","Iris /","Pupil /","Jaw /","Mouth /","Crown feathers /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if ["Body /","Belly /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Ho-Oh crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Ho-Oh leaves culling bounds");
            if ["Foot /","Toes /","Talons /"].iter().any(|p|part.name.starts_with(p)) {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            if ["Tail fan /","Tail tint /"].iter().any(|p|part.name.starts_with(p)) {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing fan /")) {
        assert!(part.vertices.clone().any(|v|rig.neutral.positions[v][1]-rig.skin_point(v,&matrices).y>0.08),"{} does not stroke",part.name);
    }
    for (prefix,parent) in [("Wing fan / coherent red phoenix flight bank -1",1),("Wing fan / coherent red phoenix flight bank 1",1),("Jaw / open",3),("Tail fan /",1)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.003),"{prefix} does not articulate independently");
    }
}

#[test]
fn celebi_keeps_feet_seated_and_owns_gaze_fingers_wings_and_blue_antenna_tips() {
    let rig=fixture(Species::Celebi);
    assert!(std::ptr::eq(super::for_species("CELEBI").unwrap(),super::rig(Species::Celebi)));
    assert_bits(crate::battle_species_models::mesh("CELEBI").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Foot /") {Some(0)}
            else if ["Head /","Head groove /","Eye","Iris /","Pupil /","Glint /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(2)}
            else if ["Arm /","Hand /","Finger /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Finger /") {1} else {0}).unwrap();Some(if side=="-1" {3} else {4})
            } else if n.starts_with("Wing /") {Some(if n.ends_with("-1") {5} else {6})}
            else if n.starts_with("Antenna") {Some(if n.ends_with("-1") {7} else {8})}
            else if ["Body /","Abdomen /","Chest /"].iter().any(|p|n.starts_with(p)) {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Celebi crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Celebi leaves culling bounds");
            if part.name.starts_with("Foot /") {assert_vec_close(point,Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",1),("Wing /",1),("Antenna /",2),("Hand /",1)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn mew_owns_fitted_gaze_ear_insets_hands_paws_and_three_tail_carriers() {
    let rig=fixture(Species::Mew);
    assert!(std::ptr::eq(super::for_species("MEW").unwrap(),super::rig(Species::Mew)));
    assert_bits(crate::battle_species_models::mesh("MEW").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Neck /") {Some(2)}
            else if ["Head /","Eye","Iris /","Pupil /","Glint /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if ["Ear /","Ear inset /"].iter().any(|p|n.starts_with(p)) {Some(if n.ends_with("-1") {11} else {12})}
            else if ["Arm /","Hand /","Finger /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Finger /") {1} else {0}).unwrap();Some(if side=="-1" {4} else {5})
            } else if ["Haunch /","Leg /","Foot /","Toe crease /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Toe crease /") {1} else {0}).unwrap();Some(if side=="-1" {6} else {7})
            } else if n.starts_with("Tail tip /") {Some(10)} else if n.starts_with("Body /") {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Mew crosses its authoring floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Mew leaves culling bounds");
            if ["Tail /","Tail tip /"].iter().any(|p|part.name.starts_with(p)) {assert!((point.y-rig.neutral.positions[v][1]).abs()<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",2),("Ear /",3),("Hand /",1),("Foot /",1),("Tail tip /",9)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn mewtwo_plants_toe_pads_and_articulates_genetic_anatomy() {
    let rig=fixture(Species::Mewtwo);
    assert!(std::ptr::eq(super::for_species("MEWTWO").unwrap(),super::rig(Species::Mewtwo)));
    assert_bits(crate::battle_species_models::mesh("MEWTWO").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Mewtwo crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Mewtwo leaves culling bounds");
            if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {
                assert_eq!(rig.joint_indices[v],[0,0,0,0]);
                assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);
            }
            if ["Head /","Horn /","Eye","Iris /","Pupil /","Glint /","Mouth /","Nostril /"].iter().any(|p|part.name.starts_with(p)) {assert_eq!(rig.joint_indices[v],[3,0,0,0]);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",2),("Hand /",5),("Forearm /",4),("Tail tip /",15)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn heracross_plants_claws_and_articulates_horn_antennae_and_gauntlets() {
    let rig=fixture(Species::Heracross);
    assert!(std::ptr::eq(super::for_species("HERACROSS").unwrap(),super::rig(Species::Heracross)));
    assert_bits(crate::battle_species_models::mesh("HERACROSS").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Foot claw /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Head /","Horn /","Eye","Pupil /","Glint /","Mouth /"].iter().any(|p|n.starts_with(p)) {Some(3)}
            else if ["Antenna /","Antenna bead /"].iter().any(|p|n.starts_with(p)) {Some(if n.split_whitespace().last()==Some("-1") {8} else {9})}
            else if ["Forearm /","Hand /","Hand claw /","Gauntlet tooth /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Hand claw /") || n.starts_with("Gauntlet tooth /") {1} else {0}).unwrap();Some(if side=="-1" {5} else {7})
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Heracross crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Heracross leaves culling bounds");
            if ["Foot /","Foot claw /"].iter().any(|p|part.name.starts_with(p)) {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.18,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",2),("Antenna bead /",3),("Forearm /",4)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn pinsir_plants_tarsi_and_owns_spiked_pincers_and_barred_face() {
    let rig=fixture(Species::Pinsir);
    assert!(std::ptr::eq(super::for_species("PINSIR").unwrap(),super::rig(Species::Pinsir)));
    assert_bits(crate::battle_species_models::mesh("PINSIR").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Foot /","Foot claw /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Eye","Pupil /","Mouth /","Tooth /"].iter().any(|p|n.starts_with(p)) {Some(2)}
            else if ["Pincer /","Pincer spike /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Pincer spike /") {1} else {0}).unwrap();Some(if side=="-1" {3} else {4})
            } else if ["Hand /","Hand claw /"].iter().any(|p|n.starts_with(p)) {
                let side=n.split_whitespace().rev().nth(if n.starts_with("Hand claw /") {1} else {0}).unwrap();Some(if side=="-1" {7} else {10})
            } else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Pinsir crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Pinsir leaves culling bounds");
            if ["Foot /","Foot claw /"].iter().any(|p|part.name.starts_with(p)) {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Pincer /",2),("Forearm /",5),("Hand /",6)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn scizor_plants_greaves_and_owns_open_jaws_false_eyes_and_wings() {
    let rig=fixture(Species::Scizor);
    assert!(std::ptr::eq(super::for_species("SCIZOR").unwrap(),super::rig(Species::Scizor)));
    assert_bits(crate::battle_species_models::mesh("SCIZOR").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Greave /","Toe cap /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Head /","Crest /","Eye","Pupil /"].iter().any(|p|n.starts_with(p)) {Some(4)}
            else if ["Pincer lower /","Pincer lower lining /"].iter().any(|p|n.starts_with(p)) {Some(if n.split_whitespace().last()==Some("-1") {8} else {12})}
            else if n.starts_with("Pincer ") {Some(if n.split_whitespace().last()==Some("-1") {7} else {11})}
            else if ["Wing /","Wing vein /"].iter().any(|p|n.starts_with(p)) {Some(if n.split_whitespace().rev().nth(1)==Some("-1") {13} else {14})}
            else if ["Abdomen /","Abdominal plate /"].iter().any(|p|n.starts_with(p)) {Some(2)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Scizor crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Scizor leaves culling bounds");
            if ["Greave /","Toe cap /"].iter().any(|p|part.name.starts_with(p)) {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",3),("Pincer upper /",6),("Pincer lower /",7),("Wing /",1)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn scyther_plants_claws_and_owns_fitted_scythe_spines_and_wing_membranes() {
    let rig=fixture(Species::Scyther);
    assert!(std::ptr::eq(super::for_species("SCYTHER").unwrap(),super::rig(Species::Scyther)));
    assert_bits(crate::battle_species_models::mesh("SCYTHER").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if ["Greave /","Foot claw /"].iter().any(|p|n.starts_with(p)) {Some(0)}
            else if ["Head /","Head marking /","Crest /","Eye","Pupil /"].iter().any(|p|n.starts_with(p)) {Some(4)}
            else if ["Scythe /","Scythe rim /"].iter().any(|p|n.starts_with(p)) {Some(if n.split_whitespace().last()==Some("-1") {6} else {8})}
            else if ["Wing /","Wing case /","Wing vein /"].iter().any(|p|n.starts_with(p)) {Some(if n.split_whitespace().rev().nth(1)==Some("-1") {9} else {10})}
            else if ["Abdomen /","Abdomen seam /"].iter().any(|p|n.starts_with(p)) {Some(2)}
            else if ["Haunch /","Haunch thorn /"].iter().any(|p|n.starts_with(p)) {Some(if n.split_whitespace().last()==Some("-1") {11} else {12})} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Scyther crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Scyther leaves culling bounds");
            if ["Greave /","Foot claw /"].iter().any(|p|part.name.starts_with(p)) {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",3),("Scythe /",5),("Wing /",1),("Haunch /",2)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
}

#[test]
fn starmie_articulates_both_stars_and_preserves_coherent_core_and_bounds() {
    let rig=fixture(Species::Starmie);
    assert!(std::ptr::eq(super::for_species("STARMIE").unwrap(),super::rig(Species::Starmie)));
    assert_bits(crate::battle_species_models::mesh("STARMIE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        if part.name.starts_with("Mount") || part.name.starts_with("Core") {
            for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[1,0,0,0],"{}",part.name);}
        }
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Starmie crosses the authoring floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Starmie leaves culling bounds");
        }
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for prefix in ["Front star /","Back star /"] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently of its core");
    }
}

#[test]
fn gastly_preserves_its_fitted_face_and_articulates_vapor_within_bounds() {
    let rig=fixture(Species::Gastly);
    assert!(std::ptr::eq(super::for_species("GASTLY").unwrap(),super::rig(Species::Gastly)));
    assert_bits(crate::battle_species_models::mesh("GASTLY").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        if !part.name.starts_with("Gas ") {
            for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[1,0,0,0],"{}",part.name);}
        }
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Gastly crosses the authoring floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Gastly leaves culling bounds");
        }
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Gas ")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{} does not drift independently of the face",part.name);
    }
}

#[test]
fn haunter_keeps_its_face_coherent_and_articulates_six_detached_hand_claws() {
    let rig=fixture(Species::Haunter);
    assert!(std::ptr::eq(super::for_species("HAUNTER").unwrap(),super::rig(Species::Haunter)));
    assert_bits(crate::battle_species_models::mesh("HAUNTER").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Body /") {None}
            else if n.starts_with("Hand /") {Some(if n.ends_with("-1") {8} else {12})}
            else if n.starts_with("Finger /") {
                let words:Vec<_>=n.split_whitespace().collect();
                Some((if words[words.len()-2]=="-1" {9} else {13})+words.last().unwrap().parse::<u16>().unwrap())
            } else {Some(1)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Haunter crosses the authoring floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Haunter leaves culling bounds");
        }
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Finger /")) {
        let parent=if part.name.split_whitespace().rev().nth(1)==Some("-1") {8} else {12};
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{} does not curl independently of its palm",part.name);
    }
    let body=&rig.anatomy[0];
    for hinge in 2..8 {
        assert!(body.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==hinge && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"shroud hinge {hinge} does not articulate");
    }
}

#[test]
fn eevee_plants_four_paws_and_articulates_head_ears_limbs_and_brush() {
    let rig=fixture(Species::Eevee);
    assert!(std::ptr::eq(super::for_species("EEVEE").unwrap(),super::rig(Species::Eevee)));
    assert_bits(crate::battle_species_models::mesh("EEVEE").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") {Some(1)}
            else if n.starts_with("Ruff /") || n.starts_with("Ruff tuft /") {Some(2)}
            else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Tail /") || n.starts_with("Tail tip /") || n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Eevee crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Eevee leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Tail tip /",7,0.002)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn vaporeon_plants_paws_and_articulates_fans_frill_and_split_fin_tail() {
    let rig=fixture(Species::Vaporeon);
    assert!(std::ptr::eq(super::for_species("VAPOREON").unwrap(),super::rig(Species::Vaporeon)));
    assert_bits(crate::battle_species_models::mesh("VAPOREON").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if ["Body /","Shoulder /","Haunch /","Back ridge /"].iter().any(|p|n.starts_with(p)) {Some(1)}
            else if n.starts_with("Frill /") {Some(15)}
            else if n.starts_with("Head fin") {Some(if rig.neutral.positions[part.vertices.start][0]<0. {4} else {5})}
            else if n.starts_with("Fluke /") {Some(if n.ends_with("-1") {9} else {10})}
            else if ["Tail /","Tail ridge /","Leg /","Neck /"].iter().any(|p|n.starts_with(p)) {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Vaporeon crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Vaporeon leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent) in [("Head /",2),("Head fin /",3),("Frill /",2),("Tail /",1)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Fluke /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[8].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{} does not articulate independently of the tail",part.name);
    }
}
#[test]
fn jolteon_plants_paws_and_articulates_sharp_ears_and_needle_collar() {
    let rig=fixture(Species::Jolteon);
    assert!(std::ptr::eq(super::for_species("JOLTEON").unwrap(),super::rig(Species::Jolteon)));
    assert_bits(crate::battle_species_models::mesh("JOLTEON").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") || n.starts_with("Back quill /") || n.starts_with("Rump quill /") {Some(1)}
            else if n.starts_with("Ruff /") || n.starts_with("Ruff tuft /") {Some(10)}
            else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Jolteon crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Jolteon leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Ruff /",2,0.002)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn flareon_plants_paws_and_articulates_ears_cream_collar_and_plume() {
    let rig=fixture(Species::Flareon);
    assert!(std::ptr::eq(super::for_species("FLAREON").unwrap(),super::rig(Species::Flareon)));
    assert_bits(crate::battle_species_models::mesh("FLAREON").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") {Some(1)}
            else if n.starts_with("Ruff /") || n.starts_with("Ruff tuft /") {Some(13)}
            else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Tail /") || n.starts_with("Tail tip /") || n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Flareon crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Flareon leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Tail /",1,0.002),("Ruff /",2,0.002)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn espeon_plants_paws_and_articulates_ears_and_both_forked_tail_tips() {
    let rig=fixture(Species::Espeon);
    assert!(std::ptr::eq(super::for_species("ESPEON").unwrap(),super::rig(Species::Espeon)));
    assert_bits(crate::battle_species_models::mesh("ESPEON").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") || n.starts_with("Shoulder /") {Some(1)}
            else if n.starts_with("Tail hub /") {Some(8)}
            else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Tail /") || n.starts_with("Tail fork /") || n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Espeon crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Espeon leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Tail /",1,0.002),("Tail fork /",8,0.003)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Tail fork /")) {
        let expected=if part.name.ends_with("-1") {9} else {10};
        assert!(part.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==expected && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[8].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.003),"{} does not articulate independently of the hub",part.name);
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn umbreon_plants_paws_and_articulates_gold_banded_ears_and_tail() {
    let rig=fixture(Species::Umbreon);
    assert!(std::ptr::eq(super::for_species("UMBREON").unwrap(),super::rig(Species::Umbreon)));
    assert_bits(crate::battle_species_models::mesh("UMBREON").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") || n.starts_with("Shoulder /") || n.starts_with("Leg ring /") {Some(1)}
            else if n.starts_with("Ear /") || n.starts_with("Ear band /") || n.starts_with("Ear tip /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Tail /") || n.starts_with("Tail band /") || n.starts_with("Tail tip /") || n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Umbreon crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Umbreon leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Tail tip /",7,0.002)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn lapras_swept_flippers_keep_blue_cream_seams_closed_and_roots_seated() {
    let rig=fixture(Species::Lapras);
    let mut seams=Vec::new();
    for side in [-1,1] { for rear in [false,true] {
        let blue=format!("{} flipper / {side}",if rear {"Rear"} else {"Fore"});
        let cream=format!("Flipper underside / {} {side}",if rear {"rear"} else {"fore"});
        let top=rig.anatomy.iter().find(|p|p.name==blue).unwrap();
        let bottom=rig.anatomy.iter().find(|p|p.name==cream).unwrap();
        let positions: std::collections::BTreeMap<_,_>=top.vertices.clone().map(|i|(rig.neutral.positions[i].map(f32::to_bits),i)).collect();
        let shared: std::collections::BTreeMap<_,_>=bottom.vertices.clone().filter_map(|i|positions.get(&rig.neutral.positions[i].map(f32::to_bits)).map(|&j|(rig.neutral.positions[i].map(f32::to_bits),(j,i)))).collect();
        assert_eq!(shared.len(),if rear {20} else {26});
        for &(a,b) in shared.values() {
            assert_eq!(rig.joint_indices[a],rig.joint_indices[b]);
            assert_eq!(rig.joint_weights[a],rig.joint_weights[b]);
            seams.push((a,b));
        }
        let owner=if rear {if side<0 {6} else {7}} else {if side<0 {4} else {5}};
        for p in [top,bottom] {
            assert!(p.vertices.clone().any(|i|rig.joint_indices[i].iter().zip(rig.joint_weights[i]).any(|(&j,w)|j==owner && w>0.99)),"{} must articulate with its flipper",p.name);
            assert!(p.vertices.clone().any(|i|rig.joint_indices[i]==[0;4]),"{} needs seated root vertices",p.name);
        }
    }}
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL { for step in 0..=32 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/33.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for &(a,b) in &seams {assert_vec_close(rig.skin_point(a,&matrices),rig.skin_point(b,&matrices),0.000001);}
        for p in rig.anatomy.iter().filter(|p|p.name.contains("flipper /") || p.name.starts_with("Flipper underside /")) {
            for i in p.vertices.clone() {
                let point=rig.skin_point(i,&matrices);
                assert!(point.y>=-0.000001,"{} crosses the floor",p.name);
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{} leaves its culling envelope",p.name);
            }
        }
    }}
}

#[test]
fn vulpix_plants_paws_and_articulates_all_six_curled_tails() {
    let rig=fixture(Species::Vulpix);
    assert!(std::ptr::eq(super::for_species("VULPIX").unwrap(),super::rig(Species::Vulpix)));
    assert_bits(crate::battle_species_models::mesh("VULPIX").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") {Some(1)}
            else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Tail /") || n.starts_with("Bib /") || n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Vulpix crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Vulpix leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Tail /",6,0.002)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Tail /")) {
        let owner=7+part.name.split_whitespace().last().unwrap().parse::<u16>().unwrap();
        assert!(part.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==owner && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[6].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{} does not articulate independently",part.name);
    }
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Tail /")).count(),6);
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn ninetales_plants_paws_and_keeps_all_nine_golden_tip_seams_closed() {
    let rig=fixture(Species::Ninetales);
    assert!(std::ptr::eq(super::for_species("NINETALES").unwrap(),super::rig(Species::Ninetales)));
    assert_bits(crate::battle_species_models::mesh("NINETALES").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")).count(),4);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Paw /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Body /") || n.starts_with("Haunch /") || n.starts_with("Shoulder /") {Some(1)}
            else if n.starts_with("Mane /") || n.starts_with("Mane tuft /") {Some(2)}
            else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Tail /") || n.starts_with("Tail tip /") || n.starts_with("Leg /") || n.starts_with("Neck /") {None}
            else {Some(3)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let tips=rig.anatomy.iter().find(|p|p.name.starts_with("Tail tip /")).unwrap();
    let mut seams=Vec::new();
    for base in rig.anatomy.iter().filter(|p|p.name.starts_with("Tail /")) {
        let points: std::collections::BTreeMap<_,_>=base.vertices.clone().map(|i|(rig.neutral.positions[i].map(f32::to_bits),i)).collect();
        let common: std::collections::BTreeMap<_,_>=tips.vertices.clone().filter_map(|i|points.get(&rig.neutral.positions[i].map(f32::to_bits)).map(|&j|(rig.neutral.positions[i].map(f32::to_bits),(j,i)))).collect();
        assert_eq!(common.len(),20);
        for &(a,b) in common.values() {assert_eq!(rig.joint_indices[a],rig.joint_indices[b]);assert_eq!(rig.joint_weights[a],rig.joint_weights[b]);seams.push((a,b));}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for &(a,b) in &seams {assert_vec_close(rig.skin_point(a,&matrices),rig.skin_point(b,&matrices),0.000001);}
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Ninetales crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Ninetales leaves culling bounds");
            if part.name.starts_with("Paw /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Head /",2,0.002),("Ear /",3,0.002),("Tail /",6,0.002)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Tail /")) {
        let owner=7+part.name.split_whitespace().last().unwrap().parse::<u16>().unwrap();
        assert!(part.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==owner && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[6].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{} does not articulate independently",part.name);
    }
    for owner in 7..=15 {
        assert!(tips.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==owner && w>0.99) && rig.skin_point(v,&matrices).distance(matrices[6].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"packed tip {owner} lost its independent carrier");
    }
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Tail /")).count(),9);
    for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")) {
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{} does not flex independently of the torso",part.name);
    }
}

#[test]
fn venonat_plants_broad_feet_and_articulates_feelers_and_claw_hands() {
    let rig=fixture(Species::Venonat);
    assert!(std::ptr::eq(super::for_species("VENONAT").unwrap(),super::rig(Species::Venonat)));
    assert_bits(crate::battle_species_models::mesh("VENONAT").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).count(),2);
    for part in &rig.anatomy {
        let n=part.name.as_str();
        let owner=if n.starts_with("Foot /") || n.starts_with("Toe crease /") {Some(0)}
            else if n.starts_with("Antenna /") || n.starts_with("Antenna tip /") {Some(if n.ends_with("-1") {2} else {3})}
            else if n.starts_with("Hand /") {Some(if n.ends_with("-1") {6} else {7})}
            else if n.starts_with("Finger /") {Some(if n.split_whitespace().rev().nth(1).unwrap()=="-1" {6} else {7})}
            else if n.starts_with("Body /") || n.starts_with("Leg /") || n.starts_with("Arm /") {None}
            else {Some(1)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Venonat crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Venonat leaves culling bounds");
            if part.name.starts_with("Foot /") || part.name.starts_with("Toe crease /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Antenna /",1,0.002),("Arm /",1,0.001),("Hand /",4,0.001)] {
        let part=rig.anatomy.iter().find(|p|p.name.starts_with(prefix)).unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{prefix} does not articulate independently");
    }
}

#[test]
fn venomoth_articulates_four_coated_wings_three_antennae_and_tucked_legs() {
    let rig=fixture(Species::Venomoth);
    assert!(std::ptr::eq(super::for_species("VENOMOTH").unwrap(),super::rig(Species::Venomoth)));
    assert_bits(crate::battle_species_models::mesh("VENOMOTH").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Wing /")).count(),4);
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Wing vein /")).count(),20);
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")).count(),6);
    for part in &rig.anatomy {
        let n=part.name.as_str();let words:Vec<_>=n.split_whitespace().collect();
        let owner=if n.starts_with("Thorax /") {Some(1)}
            else if n.starts_with("Abdomen /") || n.starts_with("Abdomen band /") {Some(3)}
            else if n.starts_with("Antenna /") {Some(if n.contains("central") {8} else if n.ends_with("-1") {9} else {10})}
            else if n.starts_with("Wing /") || n.starts_with("Wing vein /") {
                let end=words.len()-if n.starts_with("Wing vein /") {2} else {0};
                Some(4+if words[end-2]=="-1" {0} else {2}+words[end-1].parse::<u16>().unwrap())
            } else if n.starts_with("Foot /") {Some(11+if words[words.len()-2]=="-1" {0} else {3}+words.last().unwrap().parse::<u16>().unwrap())}
            else if n.starts_with("Leg /") {None} else {Some(2)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let point=rig.skin_point(v,&matrices);
            assert!(point.y>=-0.000001,"Venomoth crosses the floor");
            assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"Venomoth leaves culling bounds");
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
    rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for (prefix,parent,threshold) in [("Wing /",1,0.002),("Antenna /",2,0.001),("Leg /",1,0.001),("Abdomen /",1,0.001)] {
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)) {
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{} does not articulate independently",part.name);
        }
    }
}

#[test]
fn mushroom_crabs_plant_four_feet_and_articulate_claws_and_coherent_fungal_caps() {
    for (species,name,caps) in [(Species::Paras,"PARAS",2),(Species::Parasect,"PARASECT",1)] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Mushroom cap /")).count(),caps);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")).count(),4);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Foreclaw /")).count(),2);
        for part in &rig.anatomy {
            let n=part.name.as_str();let words:Vec<_>=n.split_whitespace().collect();
            let owner=if n.starts_with("Foot /") {Some(0)}
                else if n.starts_with("Body /") || n.starts_with("Carapace seam /") {Some(1)}
                else if n.starts_with("Foreclaw /") {Some(if n.ends_with("-1") {3} else {4})}
                else if n.starts_with("Mushroom") {
                    let label=words[words.len()-if n.starts_with("Mushroom spot /") || n.starts_with("Mushroom gill /") {2} else {1}];
                    Some(if species==Species::Parasect || label=="-1" {9} else {10})
                } else if n.starts_with("Leg /") {None} else {Some(2)};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if part.name.starts_with("Foot /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for (prefix,parent,threshold) in [("Foreclaw /",1,0.002),("Mushroom cap /",1,0.001),("Leg /",1,0.001)] {
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with(prefix)) {
                assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>threshold),"{name} {} does not articulate independently",part.name);
            }
        }
    }
}

#[test]
fn cocoons_keep_lower_shell_contact_and_articulate_coherent_faces_and_folds() {
    for (species,name) in [(Species::Metapod,"METAPOD"),(Species::Kakuna,"KAKUNA")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Shell /")).count(),1);
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Eye /") || n.starts_with("Pupil /") || n.starts_with("Glint /") || n.starts_with("Brow /") {Some(2)}
                else if n.starts_with("Arm fold /") || n.starts_with("Arm crease /") {Some(if n.ends_with("-1") {3} else {4})}
                else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let floor: Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)|(p[1]<0.000001).then_some(i)).collect();
        assert!(floor.len()>=24);
        for &v in &floor {assert_eq!(rig.joint_indices[v],[0,0,0,0]);}
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }}
            for &v in &floor {assert!(rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Eye /") || p.name.starts_with("Arm fold /")) {
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} does not articulate independently",part.name);
        }
        if species==Species::Metapod {
            assert!(rig.anatomy[0].vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==3 && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001));
        }
    }
}

#[test]
fn larvae_keep_contact_and_articulate_coherent_heads_tips_and_closed_body_seams() {
    for (species,name) in [(Species::Caterpie,"CATERPIE"),(Species::Weedle,"WEEDLE")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Proleg / planted") {Some(0)}
                else if n.starts_with("Horn branch /") || n.starts_with("Horn tip /") {Some(if n.ends_with("-1") {10} else {11})}
                else if n.starts_with("Horn /") {Some(10)}
                else if n.starts_with("Tail sting /") {Some(11)}
                else if n.starts_with("Tail tip /") {Some(1)}
                else if n.starts_with("Head /") || n.starts_with("Eye") || n.starts_with("Glint /") || n.starts_with("Muzzle") || n.starts_with("Nose") || n.starts_with("Horn stem /") {Some(9)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let mut seams=Vec::new();
        if species==Species::Caterpie {
            let a=rig.anatomy.iter().find(|p|p.name.starts_with("Body /")).unwrap();let b=rig.anatomy.iter().find(|p|p.name.starts_with("Belly /")).unwrap();
            let left:std::collections::BTreeMap<_,_>=a.vertices.clone().map(|i|(rig.neutral.positions[i].map(f32::to_bits),i)).collect();
            let common:std::collections::BTreeMap<_,_>=b.vertices.clone().filter_map(|i|left.get(&rig.neutral.positions[i].map(f32::to_bits)).map(|&j|(rig.neutral.positions[i].map(f32::to_bits),(j,i)))).collect();
            assert_eq!(common.len(),114);
            for &(a,b) in common.values() {assert_eq!(rig.joint_indices[a],rig.joint_indices[b]);assert_eq!(rig.joint_weights[a],rig.joint_weights[b]);seams.push((a,b));}
        }
        let floor:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)|(p[1]<0.000001).then_some(i)).collect();assert!(!floor.is_empty());
        for &v in &floor {assert_eq!(rig.joint_indices[v],[0,0,0,0]);}
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let point=rig.skin_point(v,&matrices);assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if part.name.starts_with("Proleg / planted") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
            for &v in &floor {assert!(rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            for &(a,b) in &seams {assert_vec_close(rig.skin_point(a,&matrices),rig.skin_point(b,&matrices),0.000001);}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {
            let n=part.name.as_str();let parent=if n.starts_with("Eye /") {Some(8)} else if n.starts_with("Horn branch /") || n.starts_with("Horn /") {Some(9)} else if n.starts_with("Tail sting /") {Some(1)} else {None};
            if let Some(parent)=parent {assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {n} does not articulate independently");}
        }
    }
}

#[test]
fn crabs_plant_four_toes_and_articulate_separate_arms_wrists_and_jaws() {
    for (species,name) in [(Species::Krabby,"KRABBY"),(Species::Kingler,"KINGLER")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Walking leg /")).count(),4);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).count(),4);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Crown /")).count(),if species==Species::Kingler {5} else {2});
        for part in &rig.anatomy {
            let n=part.name.as_str();
            let owner=if n.starts_with("Foot /") {Some(0)}
                else if n.starts_with("Wrist /") || n.starts_with("Claw palm /") {Some(if n.ends_with("-1") {3} else {6})}
                else if n.starts_with("Claw jaw /") {Some(if n.ends_with("-1") {4} else {7})}
                else if n.starts_with("Walking leg /") || n.starts_with("Leg knee /") || n.starts_with("Pincer arm /") {None}
                else {Some(1)};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if part.name.starts_with("Foot /") {assert!(point.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Claw jaw /")) {
            let wrist=if part.name.ends_with("-1") {3} else {6};
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[wrist].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.004),"{name} {} does not close independently",part.name);
        }
    }
}

#[test]
fn bivalves_keep_shell_contact_and_articulate_valves_around_coherent_faces() {
    for (species,name) in [(Species::Shellder,"SHELLDER"),(Species::Cloyster,"CLOYSTER")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Shell /")).count(),if species==Species::Cloyster {2} else {1});
        for part in &rig.anatomy {
            let n=part.name.as_str();let words:Vec<_>=n.split_whitespace().collect();
            let owner=if species==Species::Shellder {
                if n.starts_with("Tongue /") || n.starts_with("Tongue crease /") {None}
                else if n.starts_with("Jaw /") || n.starts_with("Jaw lining /") {Some(3)}
                else if ["Shell /","Lining /","Hinge /","Horn /","Rear horn /","Shell groove /"].iter().any(|p|n.starts_with(p)) {Some(2)}
                else {Some(1)}
            } else {
                if n.starts_with("Shell /") {Some(if n.ends_with("-1") {2} else {3})}
                else if n.starts_with("Outer spike /") {Some(if words[words.len()-2]=="-1" {2} else {3})}
                else if ["Inner hood /","Mantle lining /","Inner horn /"].iter().any(|p|n.starts_with(p)) {Some(4)}
                else {Some(1)}
            };
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let mut floor=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);floor=floor.min(point.y);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            assert!(floor.abs()<0.000001,"{name} loses shell contact: {floor}");
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Shell /") || p.name.starts_with("Tongue /") || p.name.starts_with("Inner hood /")) {
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} does not articulate independently",part.name);
        }
    }
}

#[test]
fn slimes_keep_broad_puddles_fixed_and_articulate_soft_appendages_with_coherent_faces() {
    for (species,name) in [(Species::Grimer,"GRIMER"),(Species::Muk,"MUK"),(Species::Ditto,"DITTO")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let floor:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)|(p[1]<0.0000001).then_some(i)).collect();
        assert!(floor.len()>=48);
        for &v in &floor {assert_eq!(rig.joint_indices[v],[0,0,0,0]);assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}
        for part in &rig.anatomy {
            let n=part.name.as_str();
            if ["Eye /","Eye border /","Eye white /","Pupil /","Mouth /","Lip /","Tongue /","Smile interior /"].iter().any(|p|n.starts_with(p)) {
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[2,0,0,0],"{name} {n}");}
            }
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            for &v in &floor {assert!(rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();
        rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        if species==Species::Ditto {
            let body=rig.anatomy.iter().find(|p|p.name.starts_with("Body /")).unwrap();
            for lobe in [3,4] {assert!(body.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==lobe && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[2].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"Ditto lobe {lobe} does not articulate");}
        } else {
            assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Finger /")).count(),6);
            for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Finger /")) {
                let words:Vec<_>=part.name.split_whitespace().collect();let side=words[words.len()-2].parse::<i32>().unwrap();let digit=words.last().unwrap().parse::<u16>().unwrap();let own=7+if side<0 {0} else {3}+digit;let hand=if side<0 {4} else {6};
                assert!(part.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==own && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[hand].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} does not flex independently",part.name);
            }
        }
    }
}

#[test]
fn seals_keep_low_supports_and_articulate_curved_tails_and_paired_fans() {
    for (species,name) in [(Species::Seel,"SEEL"),(Species::Dewgong,"DEWGONG")] {
        let rig=fixture(species);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Flipper /")).count(),2);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Tail fluke /")).count(),2);
        for part in &rig.anatomy {
            let n=part.name.as_str();let words:Vec<_>=n.split_whitespace().collect();
            let owner=if n.starts_with("Body /") {None}
                else if n.starts_with("Flipper /") {Some(if n.ends_with("-1") {4} else {5})}
                else if n.starts_with("Flipper fold /") {Some(if words[words.len()-2]=="-1" {4} else {5})}
                else if n.starts_with("Tail fluke /") {Some(if n.ends_with("-1") {7} else {8})}
                else if species==Species::Seel && (n.starts_with("Tongue /") || n.starts_with("Tongue crease /")) {Some(9)}
                else {Some(2)};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let floor:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(v,p)|(p[1]<0.0000001).then_some(v)).collect();assert!(!floor.is_empty());
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);low=low.min(point.y);
                assert!(point.y>=-0.000001,"{name} crosses the floor");
                assert!(point.cmpge(rig.animated_bounds.0).all() && point.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
            }
            assert!(low.abs()<0.000001,"{name} loses ground contact");
            for &v in &floor {assert!(rig.skin_point(v,&matrices).y.abs()<0.000001);}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Flipper /") || p.name.starts_with("Tail fluke /") || p.name.starts_with("Tongue /")) {
            let parent=if part.name.starts_with("Tail fluke /") {6} else if part.name.starts_with("Tongue /") {2} else {1};
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} lacks independent motion",part.name);
        }
        let body=rig.anatomy.iter().find(|p|p.name.starts_with("Body /")).unwrap();
        assert!(body.vertices.clone().any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j==6 && w>0.5) && rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002),"{name} tail stem does not articulate");
    }
}

#[test]
fn rhinos_keep_planted_pads_and_articulate_jaws_ears_tails_and_adult_drill() {
    for (species,name) in [(Species::Rhyhorn,"RHYHORN"),(Species::Rhydon,"RHYDON")] {
        let rig=fixture(species);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).count(),if species==Species::Rhyhorn {4} else {2});
        for part in &rig.anatomy {
            let n=part.name.as_str();let owner=if n.starts_with("Foot /") || n.starts_with("Foot claw /") || species==Species::Rhyhorn && n.starts_with("Claw /") {Some(0)}
                else if n.starts_with("Leg /") || n.starts_with("Leg armor /") || n.starts_with("Thigh /") {None}
                else if species==Species::Rhyhorn {
                    if n.starts_with("Body /") || n.starts_with("Back armor /") {Some(1)}
                    else if n.starts_with("Tail /") {Some(3)}
                    else if n.starts_with("Jaw /") {Some(8)}
                    else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {9} else {10})} else {Some(2)}
                } else {
                    if ["Body /","Belly plate /","Collar /","Back armor /"].iter().any(|p|n.starts_with(p)) {Some(1)}
                    else if n.starts_with("Jaw /") || n.starts_with("Tongue /") {Some(3)}
                    else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {12} else {13})}
                    else if n.starts_with("Horn /") || n.starts_with("Horn spiral /") {Some(14)}
                    else if n.starts_with("Shoulder /") {Some(if n.ends_with("-1") {4} else {6})}
                    else if n.starts_with("Hand /") || n.starts_with("Hand claw /") {Some(if n.ends_with("-1") {5} else {7})}
                    else if n.starts_with("Arm /") || n.starts_with("Tail /") || n.starts_with("Tail armor /") {None} else {Some(2)}
                };
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.0,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} crosses the floor");assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all(),"{name} leaves culling bounds");
                if part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /") || species==Species::Rhyhorn && part.name.starts_with("Claw /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Jaw /") || p.name.starts_with("Ear /") || p.name.starts_with("Hand /") || species==Species::Rhydon && p.name.starts_with("Horn /")) {
            let parent=if part.name.starts_with("Hand /") {if part.name.ends_with("-1") {4} else {6}} else {2};
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} lacks independent motion",part.name);
        }
        let tail=rig.anatomy.iter().find(|p|p.name.starts_with("Tail /")).unwrap();assert!(tail.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.002));
    }
}

#[test]
fn fighting_family_keeps_two_grounded_feet_and_independent_arm_chains() {
    for (species,name,arms) in [(Species::Machop,"MACHOP",2),(Species::Machoke,"MACHOKE",2),(Species::Machamp,"MACHAMP",4)] {
        let rig=fixture(species);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Arm /")).count(),arms);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Hand /")).count(),arms);
        for part in &rig.anatomy {
            if part.name.starts_with("Foot /") || part.name.starts_with("Toe /") {
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[0,0,0,0]);}
            }
            if ["Eye /","Mouth /","Brow /","Crest /"].iter().any(|p|part.name.starts_with(p)) {
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[3,0,0,0]);}
            }
            if part.name.starts_with("Hand /") || part.name.starts_with("Finger /") || part.name.starts_with("Thumb /") {
                let words:Vec<_>=part.name.split_ascii_whitespace().collect();let side=if part.name.starts_with("Finger /") {words[words.len()-2]} else {words[words.len()-1]};
                let row=if part.name.contains(" / lower ") {2} else {0};let owner=8+3*(row+usize::from(side=="1"));
                for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner as u16,0,0,0]);}
            }
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} crosses floor");assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") || part.name.starts_with("Toe /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Hand /")) {
            let wrist=rig.joint_indices[part.vertices.start][0] as usize;let parent=rig.joints[wrist].parent.unwrap();
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} lacks wrist articulation",part.name);
        }
    }
}

#[test]
fn fire_horses_preserve_hoof_support_coherent_banks_and_independent_plumes() {
    for (species,name) in [(Species::Ponyta,"PONYTA"),(Species::Rapidash,"RAPIDASH")] {
        let rig=fixture(species);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Leg /")).count(),4);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Flame /")).count(),7);
        assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Flame glow /")).count(),7);
        for part in &rig.anatomy {
            let n=part.name.as_str();let owner=if n.starts_with("Hoof /") {Some(if species==Species::Rapidash && n.ends_with("-1 0") {8} else {0})}
                else if n.starts_with("Leg /") || n.starts_with("Neck /") {None}
                else if n.starts_with("Body /") || n.starts_with("Shoulder /") {Some(1)}
                else if n.starts_with("Ear /") || n.starts_with("Ear inset /") {Some(if n.ends_with("-1") {4} else {5})}
                else if n.starts_with("Tail stem /") {Some(6)}
                else if n.starts_with("Flame /") || n.starts_with("Flame glow /") {
                    let location=n.split_once(" / ").unwrap().1;Some(if location.starts_with("mane") {12} else if location.starts_with("tail") {7} else if location.starts_with("ankle ") {
                        let words:Vec<_>=location.split_ascii_whitespace().collect();8+if words[1]=="-1" {0} else {2}+words[2].parse::<u16>().unwrap()
                    } else {13})
                } else if n.starts_with("Jaw /") || n.starts_with("Tongue /") {Some(14)} else {Some(3)};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{name} {n}");}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=30 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} crosses floor");assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Hoof /") {
                    if species==Species::Rapidash && part.name.ends_with("-1 0") {assert!(p.y>0.25);} else {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
                }
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Ear /") || p.name.starts_with("Tail stem /") || p.name.starts_with("Jaw /") || p.name.starts_with("Flame / mane") || p.name.starts_with("Flame / tail") || p.name.starts_with("Flame / crown") || p.name.starts_with("Flame / forelock")) {
            let owner=rig.joint_indices[part.vertices.start][0] as usize;let parent=rig.joints[owner].parent.unwrap();
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.001),"{name} {} lacks independent motion",part.name);
        }
    }
}

#[test]
fn kangaskhan_skin_keeps_planted_feet_and_independent_mother_and_baby_carriers() {
    let rig=fixture(Species::Kangaskhan);assert!(std::ptr::eq(super::for_species("KANGASKHAN").unwrap(),super::rig(Species::Kangaskhan)));
    assert_eq!(rig.joints.len(),20);assert_eq!(rig.anatomy.len(),46);
    for part in &rig.anatomy {
        let n=part.name.as_str();let owner=if n.starts_with("Foot /") || n.starts_with("Foot claw /") {Some(0)}
            else if ["Body /","Belly /","Pouch /","Back plate /"].iter().any(|p|n.starts_with(p)) {Some(1)}
            else if n.starts_with("Jaw /") || n.starts_with("Tongue /") {Some(3)}
            else if n.starts_with("Ear /") {Some(if n.ends_with("-1") {4} else {5})}
            else if n.starts_with("Shoulder /") || n.starts_with("Shoulder plate /") {Some(if n.ends_with("-1") {6} else {8})}
            else if n.starts_with("Fist /") || n.starts_with("Hand claw /") {Some(if n.ends_with("-1") {7} else {9})}
            else if n.starts_with("Arm /") || n.starts_with("Leg /") || n.starts_with("Tail /") || n.starts_with("Baby body /") {None}
            else if n.starts_with("Haunch armor outline /") || n.starts_with("Haunch plate /") {Some(if n.ends_with("-1") {10} else {11})}
            else if n.starts_with("Baby arm /") {Some(if n.ends_with("-1") {16} else {17})}
            else if n.starts_with("Baby ear /") {Some(if n.ends_with("-1") {18} else {19})}
            else if n.starts_with("Baby ") {Some(15)} else {Some(2)};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
            if part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for part in rig.anatomy.iter().filter(|p|["Ear /","Jaw /","Fist /","Baby head /","Baby arm /","Baby ear /"].iter().any(|prefix|p.name.starts_with(prefix))) {
        let owner=rig.joint_indices[part.vertices.start][0] as usize;let parent=rig.joints[owner].parent.unwrap();
        assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{} lacks independent motion",part.name);
    }
    let baby=rig.anatomy.iter().find(|p|p.name.starts_with("Baby body /")).unwrap();assert!(baby.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005));
}

#[test]
fn tangela_skin_keeps_shoes_planted_and_thirty_seven_independent_vine_carriers() {
    let rig=fixture(Species::Tangela);assert!(std::ptr::eq(super::for_species("TANGELA").unwrap(),super::rig(Species::Tangela)));
    assert_eq!(rig.joints.len(),39);assert_eq!(rig.anatomy.len(),10);
    assert_eq!(rig.anatomy.iter().filter(|p|p.name.starts_with("Shoe /")).count(),2);
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Shoe /") {Some(0)} else if part.name.starts_with("Body /") || part.name.starts_with("Face /") || part.name.starts_with("Eye /") {Some(1)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=30 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/31.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
            if part.name.starts_with("Shoe /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for joint in 2..39 {
        assert_eq!(rig.joints[joint].parent,Some(1));
        assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[1].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"vine {joint} lacks independent articulation");
    }
}

#[test]
fn tapir_skin_keeps_feet_planted_and_articulates_ears_hands_and_pendulum() {
    for (species,joints,parts) in [(Species::Drowzee,13,43),(Species::Hypno,14,48)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") {Some(0)} else if part.name.starts_with("Pendulum /") {Some(13)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Ear /") || p.name.starts_with("Hand /") || p.name.starts_with("Pendulum /")) {
            let owner=rig.joint_indices[part.vertices.start][0] as usize;let parent=rig.joints[owner].parent.unwrap();
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{} lacks independent motion",part.name);
        }
    }
}

#[test]
fn psychic_fox_skin_keeps_claws_planted_and_articulates_tail_moustache_and_spoons() {
    for (species,joints,parts) in [(Species::Abra,15,43),(Species::Alakazam,17,53)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") {Some(0)} else if part.name.starts_with("Spoon /") {Some(if part.name.ends_with("-1") {15} else {16})} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Ear /") || p.name.starts_with("Hand /") || p.name.starts_with("Moustache /") || p.name.starts_with("Spoon /")) {
            let owner=rig.joint_indices[part.vertices.start][0] as usize;let parent=rig.joints[owner].parent.unwrap();
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{} lacks independent motion",part.name);
        }
    }
}

#[test]
fn kadabra_skin_closes_neck_and_keeps_claws_planted_with_independent_psychic_props() {
    for (species,joints,parts) in [(Species::Kadabra,18,38)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if (part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /")) {Some(0)} else if part.name.starts_with("Held silver spoon") || part.name.starts_with("Hollow silver spoon") {Some(17)} else if part.name.starts_with("Long swept moustache") {Some(if part.name.ends_with(".001") {14} else {13})} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if (part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /")) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Long tapered fox ear") || p.name.starts_with("Hand /") || p.name.starts_with("Long swept moustache") || p.name.starts_with("Held silver spoon") || p.name.starts_with("Hollow silver spoon")) {
            let owner=rig.joint_indices[part.vertices.start][0] as usize;let parent=rig.joints[owner].parent.unwrap();
            assert!(part.vertices.clone().any(|v|rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{} lacks independent motion",part.name);
        }
    }
}

#[test]
fn gas_pair_skin_keeps_coherent_faces_and_independent_linked_orbs_and_vapor() {
    for (species,joints,parts,base) in [(Species::Koffing,12,23,2),(Species::Weezing,14,35,4)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let n=part.name.as_str();let owner=if n.starts_with("Gas cloud /") {Some(base+5*n.split_whitespace().last().unwrap().parse::<u16>().unwrap())}
                else if n.starts_with("Gas lobe /") || n.starts_with("Main stalk /") || n.starts_with("Rear stalk /") {None}
                else if n.starts_with("Rear node /") {Some(3)} else if n.starts_with("Small body /") || n.contains("small Weezing") {Some(2)} else {Some(1)};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{n}");}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=24 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/25.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());}
            assert!(low.abs()<0.000001,"main orb yaw changed the authoring floor");
            super::motion::battle_float(rig,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            assert!((0..rig.neutral.positions.len()).all(|v|rig.skin_point(v,&matrices).y>height*0.15),"intentional battle hover was lost");
        }}
        for joint in 1..joints {
            let mut moves=false;
            for t in [0.13,0.31,0.57,0.83] {
                rig.sample_into(Clip::Idle,rig.clip(Clip::Idle).duration*t,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();let parent=rig.joints[joint].parent.unwrap();
                moves |= (0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005);
            }
            assert!(moves,"joint {joint} lacks independent motion");
        }
    }
}

#[test]
fn magnet_skin_keeps_mechanics_rigid_with_independent_orbs_magnets_and_screws() {
    for (species,units,joints,parts) in [(Species::Magnemite,1,7,9),(Species::Magneton,3,19,27)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let unit=part.name.rsplit(" / unit ").next().unwrap().parse::<u16>().unwrap();let base=1+6*unit;
            let owners:Vec<u16>=if part.name.starts_with("Sphere /") || part.name.starts_with("Eye dark /") || part.name.starts_with("Eye white /") {vec![base]}
                else if part.name.starts_with("Crown /") {vec![base+3]}
                else if part.name.starts_with("Face bolts /") {vec![base+4,base+5]}
                else if part.name.starts_with("Slots /") {vec![base+3,base+4,base+5]} else {vec![base+1,base+2]};
            for v in part.vertices.clone() {assert!(owners.contains(&rig.joint_indices[v][0]),"{} has wrong owner",part.name);assert_eq!(rig.joint_indices[v][1..],[0,0,0]);assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);}
        }
        for unit in 0..units {let base=1+6*unit;assert_eq!(rig.joints[base].parent,Some(0));for joint in base+1..=base+5 {assert_eq!(rig.joints[joint].parent,Some(base));}}
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=24 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/25.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();let mut low=f32::INFINITY;
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);low=low.min(p.y);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());}
            assert!(low.abs()<0.000001,"vertical magnet hinges changed the authoring floor");
            super::motion::battle_float(rig,&mut pose);rig.skin_matrices_into(&pose,&mut matrices).unwrap();let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
            assert!((0..rig.neutral.positions.len()).all(|v|rig.skin_point(v,&matrices).y>height*0.15));
        }}
        for joint in 1..joints {
            let mut moves=false;
            for t in [0.13,0.31,0.57,0.83] {
                rig.sample_into(Clip::Idle,rig.clip(Clip::Idle).duration*t,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();let parent=rig.joints[joint].parent.unwrap();
                moves |= (0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v][0] as usize==joint && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005);
            }
            assert!(moves,"joint {joint} lacks independent motion");
        }
    }
}


#[test]
fn bone_keeper_skin_keeps_supports_fixed_and_club_with_its_gripping_wrist() {
    for (species,parts) in [(Species::Cubone,45),(Species::Marowak,52)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),14);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") || part.name.starts_with("Toe claw /") {Some(0)} else if part.name.starts_with("Bone shaft /") || part.name.starts_with("Bone lobe /") || part.name.starts_with("Thumb /") {Some(6)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") || part.name.starts_with("Toe claw /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
                if part.name.starts_with("Bone shaft /") || part.name.starts_with("Bone lobe /") {assert!(p.distance(matrices[6].transform_point3(Vec3::from_array(rig.neutral.positions[v])))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in [3,6,9,13] {
            let parent=rig.joints[joint].parent.unwrap();
            assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");
        }
    }
}

#[test]
fn crocodile_skin_keeps_supports_fixed_and_articulates_jaw_wrists_and_tail() {
    for (species,parts) in [(Species::Croconaw,29),(Species::Feraligatr,43)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),14);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") {Some(0)} else if part.name.starts_with("Jaw / fitted pale lower") || part.name.starts_with("Tooth / lower") {Some(3)} else if part.name.starts_with("Tooth / upper") || part.name.starts_with("Eye /") {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in [2,3,6,9,13] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");}
    }
}

#[test]
fn turtle_skin_preserves_rigid_shells_supports_and_independent_features() {
    for (species,parts) in [(Species::Wartortle,37),(Species::Blastoise,42)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),15);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /") || part.name.starts_with("Toe /") {Some(0)} else if ["Shell /","Shell rim /","Shell seam /","Belly /","Belly seam /","Cannon mount /"].iter().any(|prefix|part.name.starts_with(prefix)) {Some(1)} else if part.name.starts_with("Eye /") || part.name.starts_with("Muzzle /") {Some(2)} else if part.name.starts_with("Cannon /") || part.name.starts_with("Cannon floor /") || part.name.starts_with("Cannon lining /") {Some(if part.name.ends_with("-1") {11} else {12})} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /") || part.name.starts_with("Toe /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in [2,5,8,11,12,14] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");}
    }
}

#[test]
fn magmar_skin_keeps_open_bill_supported_claws_and_independent_flames() {
    let rig=fixture(Species::Magmar);
    assert_eq!((rig.joints.len(),rig.anatomy.len()),(17,39));
    assert_eq!(rig.neutral.positions.len(),19024);assert_eq!(rig.neutral.indices.len(),29200*3);
    assert!(std::ptr::eq(super::for_species("MAGMAR").unwrap(),super::rig(Species::Magmar)));
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Foot /") || part.name.starts_with("Toe claw /") {Some(0)} else if ["Head /","Forehead lobe /","Eye socket /","Eye /","Pupil /","Bill /","Mouth back /","Tongue /"].iter().any(|p|part.name.starts_with(p)) {Some(3)} else if part.name.starts_with("Hand /") || part.name.starts_with("Hand claw /") {Some(if part.name.ends_with("-1") {6} else {9})} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=20 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
            if part.name.starts_with("Foot /") || part.name.starts_with("Toe claw /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for joint in [2,3,5,6,8,9,12,13,14,15,16] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");}
}

#[test]
fn electabuzz_skin_keeps_open_jaws_fists_and_independent_sensory_features() {
    let rig=fixture(Species::Electabuzz);
    assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(19,41,19747,33880*3));
    assert!(std::ptr::eq(super::for_species("ELECTABUZZ").unwrap(),super::rig(Species::Electabuzz)));
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Foot /") || part.name.starts_with("Toe claw /") {Some(0)} else if ["Head /","Eye socket /","Eye /","Pupil /","Forehead /","Jaw /","Mouth back /","Tongue /","Fang /","Nostril /"].iter().any(|p|part.name.starts_with(p)) {Some(3)} else if part.name.starts_with("Hand /") || part.name.starts_with("Knuckle /") {Some(if part.name.ends_with("-1") {6} else {9})} else if part.name.starts_with("Tail tip /") {Some(18)} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0]);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=20 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
            if part.name.starts_with("Foot /") || part.name.starts_with("Toe claw /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for joint in [2,3,5,6,8,9,12,13,14,15,16,17,18] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");}
}

#[test]
fn slow_pair_skin_keeps_grounded_claws_and_independent_held_assemblies() {
    for (species,parts,vertices,triangles) in [(Species::Slowbro,38,19322,32532),(Species::Slowking,33,18831,31512)] {
        let rig=fixture(species);
        assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(18,parts,vertices,triangles*3));
        assert!(std::ptr::eq(super::for_species(species.name()).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /") {Some(0)} else if part.name.starts_with("Shell jaw /") || part.name.starts_with("Shell lower tooth /") {Some(17)} else if ["Shell ","Crown ","Jewel"].iter().any(|p|part.name.starts_with(p)) {Some(14)} else if ["Head /","Muzzle /","Jaw /","Mouth back /","Mouth seam /","Tongue /","Eye /","Pupil /","Nostril /","Fang /","Tooth /"].iter().any(|p|part.name.starts_with(p)) {Some(3)} else if part.name.starts_with("Hand /") || part.name.starts_with("Hand claw /") {Some(if part.name.ends_with("-1") {6} else {9})} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") || part.name.starts_with("Foot claw /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in [3,5,6,8,9,12,13,14,15,16,17] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{species:?} joint {joint} lacks independent motion");}
    }
}

#[test]
fn lickitung_skin_keeps_recessed_face_supported_toes_and_independent_tongue_tail() {
    let rig=fixture(Species::Lickitung);
    assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(18,26,17028,29256*3));
    assert!(std::ptr::eq(super::for_species("LICKITUNG").unwrap(),super::rig(Species::Lickitung)));
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Foot /") || part.name.starts_with("Toe /") {Some(0)} else if ["Head /","Mouth lining /","Eye /","Eye glint /","Nostril /"].iter().any(|p|part.name.starts_with(p)) {Some(3)} else if part.name.starts_with("Hand tip /") {Some(if part.name.ends_with("-1") {6} else {9})} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=20 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
            if part.name.starts_with("Foot /") || part.name.starts_with("Toe /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for joint in [3,5,6,8,9,12,13,14,15,16,17] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");}
}


#[test]
fn mr_mime_skin_keeps_recessed_smile_fixed_slippers_and_ten_independent_digits() {
    let rig=fixture(Species::MrMime);
    assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(26,34,19916,33656*3));
    assert!(std::ptr::eq(super::for_species("MR__MIME").unwrap(),super::rig(Species::MrMime)));
    for part in &rig.anatomy {
        let owner=if part.name.starts_with("Shoe /") || part.name.starts_with("Shoe curl /") {Some(0)} else if ["Head /","Upper lip /","Lower lip /","Mouth back /","Tongue /","Mouth lining /","Eye /","Pupil /","Eye glint /","Cheek /"].iter().any(|p|part.name.starts_with(p)) {Some(3)} else if part.name.starts_with("Palm /") {Some(if part.name.ends_with("-1") {6} else {9})} else {None};
        if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
    }
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=20 {
        rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for part in &rig.anatomy {for v in part.vertices.clone() {
            let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
            if part.name.starts_with("Shoe /") || part.name.starts_with("Shoe curl /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }}
    }}
    rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
    for joint in [3,5,6,8,9,12,13,14,15,16,17,18,19,20,21,22,23,24,25] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"joint {joint} lacks independent motion");}
}

#[test]
fn egg_nurse_skin_keeps_open_pouches_eggs_faces_and_complete_supports_coherent() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Chansey,"CHANSEY",17,16,10390,18704),(Species::Blissey,"BLISSEY",25,19,19379,34472)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if part.name.starts_with("Foot /") {Some(0)} else if ["Pouch /","Egg /","Lower body /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else if ["Eye /","Eye glint /","Smile /","Chin /","Mouth lining /","Tongue /"].iter().any(|p|part.name.starts_with(p)) {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if part.name.starts_with("Foot /") {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 9..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn fire_canine_skin_preserves_four_paws_fitted_marks_and_independent_fur_tail() {
    for (species,name,parts,vertices,triangles) in [(Species::Growlithe,"GROWLITHE",38,16302,26140),(Species::Arcanine,"ARCANINE",46,19539,31404)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(18,parts,vertices,triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Paw /","Paw crease /","Ankle fur /","Ankle tuft /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Flank stripe /","Back stripe /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else if ["Head /","Muzzle /","Nose /","Lip /","Eye socket /","Eye /","Pupil /","Eye glint /","Forehead stripe /"].iter().any(|p|part.name.starts_with(p)) {Some(3)} else if part.name.starts_with("Jaw /") {Some(6)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Paw /","Paw crease /","Ankle fur /","Ankle tuft /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in [3,4,5,6,11,12,13,14,15,16,17] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn nidoran_skin_preserves_four_paws_fitted_spots_and_independent_ears_tail() {
    for (species,name,parts,vertices,triangles) in [(Species::NidoranF,"NIDORAN_F",40,19256,31768),(Species::NidoranM,"NIDORAN_M",36,16782,28160)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(11,parts,vertices,triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Foot /","Claw /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Flank spot /","Back spine /","Haunch /","Haunch spot /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else if ["Head /","Muzzle /","Horn /","Mouth /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Nostril /","Incisor /","Cheek barb /"].iter().any(|p|part.name.starts_with(p)) {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {
                let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Foot /","Claw /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
            }}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in [2,3,4,5,6,7,8,9,10] {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn nido_middle_skin_keeps_grounded_paws_and_independent_anatomical_carriers() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Nidorina,"NIDORINA",15,35,19478,32064),(Species::Nidorino,"NIDORINO",13,39,19860,32172)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Foot /","Foot claw /","Claw /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Breast /","Flank spot /","Back spine /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else if ["Head /","Muzzle /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Nostril /","Fang /","Head spot /"].iter().any(|p|part.name.starts_with(p)) {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());if ["Foot /","Foot claw /","Claw /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn nido_royal_skin_keeps_grounded_paws_and_independent_anatomical_carriers() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Nidoqueen,"NIDOQUEEN",17,39,18397,30024),(Species::Nidoking,"NIDOKING",19,44,19990,32920)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Foot /","Foot claw /","Claw /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Chest plate /","Belly plate /","Back spine /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else if ["Head /","Muzzle /","Eye socket /","Eye /","Iris /","Pupil /","Glint /","Nostril /"].iter().any(|p|part.name.starts_with(p)) {Some(2)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());if ["Foot /","Foot claw /","Claw /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn poli_frog_skin_keeps_grounded_paws_and_independent_anatomical_carriers() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Poliwhirl,"POLIWHIRL",12,25,11464,20648),(Species::Poliwrath,"POLIWRATH",12,27,11964,21420),(Species::Politoed,"POLITOED",15,32,15317,27556)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Belly /","Spiral /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.31,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn sand_mammal_skin_keeps_grounded_paws_and_independent_anatomical_carriers() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Sandshrew,"SANDSHREW",15,33,14089,22600),(Species::Sandslash,"SANDSLASH",17,30,15013,26080)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Belly /","Shell /","Plate /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.44,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn fairy_pair_skin_keeps_grounded_paws_and_independent_anatomical_carriers() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Clefable,"CLEFABLE",11,32,15603,27448),(Species::Wigglytuff,"WIGGLYTUFF",7,24,18648,31548)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {Some(0)} else if ["Body /","Belly /","Eye /","Eye border /","Eye white /","Eye glint /","Iris /","Iris sheen /","Mouth /","Tongue /","Cheek /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());if ["Foot /","Toe /"].iter().any(|p|part.name.starts_with(p)) {assert!(p.distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.34,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn seahorse_skin_preserves_waterline_snout_and_independent_controls() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Horsea,"HORSEA",10,24,15790,24912),(Species::Seadra,"SEADRA",10,30,13214,20324),(Species::Kingdra,"KINGDRA",12,35,19154,31156)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Head /","Snout /","Eye socket /","Eye white /","Iris /","Pupil /","Glint /","Cheek /"].iter().any(|p|part.name.starts_with(p)) {Some(2)} else if ["Body /","Belly /","Belly fold /","Body seam /","Armor /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());if part.name.starts_with("Tail /") {assert!((p.y-rig.neutral.positions[v][1]).abs()<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.34,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn goldfish_skin_preserves_waterline_mouth_and_independent_controls() {
    for (species,name,joints,parts,vertices,triangles) in [(Species::Goldeen,"GOLDEEN",6,28,16748,20668),(Species::Seaking,"SEAKING",8,29,19923,24272),(Species::Magikarp,"MAGIKARP",9,25,15922,19240)] {
        let rig=fixture(species);assert_eq!((rig.joints.len(),rig.anatomy.len(),rig.neutral.positions.len(),rig.neutral.indices.len()),(joints,parts,vertices,triangles*3));assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        for part in &rig.anatomy {
            let owner=if ["Body /","Lip /","Mouth /","Eye socket /","Eye white /","Iris /","Pupil /","Lid /","Glint /","Horn /","Mask /","Flank /","Flank spot /","Gill /","Bib /","Scale seam /"].iter().any(|p|part.name.starts_with(p)) {Some(1)} else {None};
            if let Some(owner)=owner {for v in part.vertices.clone() {assert_eq!(rig.joint_indices[v],[owner,0,0,0],"{}",part.name);}}
        }
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=20 {
            rig.sample_into(clip,rig.clip(clip).duration*(step as f32+0.37)/21.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &rig.anatomy {for v in part.vertices.clone() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001,"{name} {} crosses floor",part.name);assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());{assert!((p.y-rig.neutral.positions[v][1]).abs()<0.000001);}}}
        }}
        rig.sample_into(Clip::Attack,0.34,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for joint in 1..joints {let parent=rig.joints[joint].parent.unwrap();assert!((0..rig.neutral.positions.len()).any(|v|rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)|j as usize==joint && w>0.8) && rig.skin_point(v,&matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v])))>0.0005),"{name} joint {joint} lacks independent motion");}
    }
}

#[test]
fn dragon_family_skin_preserves_contact_roots_faces_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Dratini, "DRATINI", 7, 13, 10976, 18524),
        (Species::Dragonair, "DRAGONAIR", 7, 17, 14274, 24676),
        (Species::Dragonite, "DRAGONITE", 15, 34, 18499, 28652),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles * 3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let mut fixed: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!fixed.is_empty(), "{name} has no ground contact");
        for part in &rig.anatomy {
            if ["Foot /", "Foot claw /"].iter().any(|p| part.name.starts_with(p)) {
                fixed.extend(part.vertices.clone());
            }
            if ["Head /", "Muzzle /", "Eye socket /", "Eye white /", "Iris /", "Pupil /", "Glint /", "Forehead /", "Horn /", "Mouth /", "Nostril /"].iter().any(|p| part.name.starts_with(p)) {
                for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [2, 0, 0, 0], "{name} {}", part.name); }
            }
        }
        for &v in &fixed {
            assert_eq!(rig.joint_indices[v], [0; 4], "{name} contact {v}");
            assert_eq!(rig.joint_weights[v], [1., 0., 0., 0.]);
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL {
            for step in 0..=20 {
                rig.sample_into(clip, rig.clip(clip).duration * (step as f32 + 0.37) / 21., Playback::Clamp, &mut pose).unwrap();
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                for v in 0..vertices {
                    let p = rig.skin_point(v, &matrices);
                    assert!(p.y >= -0.000001, "{name} vertex {v} crosses floor");
                    assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all(), "{name} vertex {v} leaves bounds");
                }
                for &v in &fixed { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001, "{name} contact {v} moved"); }
            }
        }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j, w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks independent motion");
        }
    }
}

#[test]
fn aerodactyl_skin_preserves_grounded_talons_jaw_and_independent_controls() {
    let rig = fixture(Species::Aerodactyl);
    assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (9, 32, 16822, 25888 * 3));
    assert!(std::ptr::eq(super::for_species("AERODACTYL").unwrap(), super::rig(Species::Aerodactyl)));
    let mut fixed: Vec<_> = (0..rig.neutral.positions.len()).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
    assert!(!fixed.is_empty());
    for part in &rig.anatomy {
        let owner = if ["Foot /", "Foot claw /"].iter().any(|p| part.name.starts_with(p)) { fixed.extend(part.vertices.clone()); Some(0) }
            else if ["Skull /", "Skull horn /", "Jaw hinge /", "Brow /", "Eye socket /", "Eye white /", "Iris /", "Mouth ceiling /", "Mouth back /"].iter().any(|p| part.name.starts_with(p)) { Some(2) }
            else if ["Wing /", "Wing back /", "Wing arm /"].iter().any(|p| part.name.starts_with(p)) { Some(if part.name.ends_with("-1") { 4 } else { 5 }) }
            else if ["Jaw /", "Mouth floor /", "Tongue /"].iter().any(|p| part.name.starts_with(p)) { Some(3) } else { None };
        if let Some(owner) = owner { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [owner, 0, 0, 0], "{}", part.name); assert_eq!(rig.joint_weights[v], [1., 0., 0., 0.]); } }
    }
    let (mut pose, mut matrices) = storage(rig);
    for clip in Clip::ALL { for step in 0..=20 {
        rig.sample_into(clip, rig.clip(clip).duration * (step as f32 + 0.37) / 21., Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for v in 0..rig.neutral.positions.len() { let p = rig.skin_point(v, &matrices); assert!(p.y >= -0.000001); assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all()); }
        for &v in &fixed { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
    } }
    rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
    for joint in 1..9 {
        let parent = rig.joints[joint].parent.unwrap();
        assert!((0..rig.neutral.positions.len()).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "joint {joint} lacks independent motion");
    }
}

#[test]
fn nautilus_skin_preserves_shell_eyes_grounded_tentacles_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Omanyte, "OMANYTE", 15, 20, 12587, 18848),
        (Species::Omastar, "OMASTAR", 19, 26, 14178, 21292),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles * 3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let floor: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        let tentacles: Vec<_> = rig.anatomy.iter().filter(|p| p.name.starts_with("Tentacle /")).collect();
        assert_eq!(tentacles.len(), 10);
        for (i, part) in tentacles.iter().enumerate() { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [(5+i) as u16, 0, 0, 0]); } }
        for part in &rig.anatomy {
            if ["Shell /", "Shell seam /", "Shell spiral /", "Spine /"].iter().any(|p| part.name.starts_with(p)) {
                for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [2, 0, 0, 0]); }
            }
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL { for step in 0..=20 {
            rig.sample_into(clip, rig.clip(clip).duration * (step as f32 + 0.37) / 21., Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &rig.anatomy { for v in part.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y >= -0.000001, "{name} {} crosses floor", part.name);
                assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if !part.name.starts_with("Beak /") { assert!((p.y-rig.neutral.positions[v][1]).abs() < 0.000001); }
            } }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).y.abs() < 0.000001); }
        } }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks motion");
        }
    }
}

#[test]
fn stone_pair_skin_preserves_fixed_supports_faces_shells_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Graveler, "GRAVELER", 10, 22, 8670, 7928),
        (Species::Golem, "GOLEM", 12, 29, 19722, 30140),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let floor: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        for part in &rig.anatomy {
            let owner = if ["Foot /", "Foot claw /", "Heel claw /"].iter().any(|p| part.name.starts_with(p)) { Some(0) }
                else if name == "GRAVELER" && ["Body /", "Forehead /", "Socket /", "Eye /", "Mouth inset /", "Rubble /"].iter().any(|p| part.name.starts_with(p)) { Some(1) }
                else if name == "GOLEM" && ["Shell /", "Shell plate /"].iter().any(|p| part.name.starts_with(p)) { Some(1) }
                else if name == "GOLEM" && ["Head /", "Muzzle /", "Socket /", "Eye /", "Iris /"].iter().any(|p| part.name.starts_with(p)) { Some(2) }
                else if name == "GOLEM" && ["Jaw /", "Tongue /", "Lower tooth /"].iter().any(|p| part.name.starts_with(p)) { Some(3) } else { None };
            if let Some(owner) = owner { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [owner,0,0,0], "{name} {}", part.name); assert_eq!(rig.joint_weights[v], [1.,0.,0.,0.]); } }
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL { for step in 0..=20 {
            rig.sample_into(clip, rig.clip(clip).duration*(step as f32+0.37)/21., Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &rig.anatomy { for v in part.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y >= -0.000001, "{name} {} crosses floor", part.name);
                assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Foot /", "Foot claw /", "Heel claw /"].iter().any(|prefix| part.name.starts_with(prefix)) { assert!(p.distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
            } }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
        } }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks motion");
        }
    }
}

#[test]
fn fire_dragon_skin_preserves_fixed_supports_faces_jaws_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Charmeleon, "CHARMELEON", 13, 29, 10656, 17122),
        (Species::Charizard, "CHARIZARD", 15, 45, 17482, 26850),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let floor: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        for part in &rig.anatomy {
            let owner = if ["Foot /", "Foot claw /"].iter().any(|p| part.name.starts_with(p)) { Some(0) }
                else if ["Body /", "Belly /"].iter().any(|p| part.name.starts_with(p)) { Some(1) }
                else if ["Head /", "Horn /", "Eye socket /", "Eye white /", "Iris /", "Pupil /", "Glint /", "Nostril /", "Fang /", "Throat /"].iter().any(|p| part.name.starts_with(p)) { Some(2) }
                else if ["Jaw /", "Tongue /", "Oral floor /"].iter().any(|p| part.name.starts_with(p)) { Some(3) }
                else if part.name.starts_with("Wing back /") { Some(if part.name.ends_with(" -1") { 13 } else { 14 }) }
                else if part.name.starts_with("Flame /") { Some(12) } else { None };
            if let Some(owner) = owner { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [owner,0,0,0], "{name} {}", part.name); assert_eq!(rig.joint_weights[v], [1.,0.,0.,0.]); } }
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL { for step in 0..=20 {
            rig.sample_into(clip, rig.clip(clip).duration*(step as f32+0.37)/21., Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &rig.anatomy { for v in part.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y >= -0.000001, "{name} {} crosses floor", part.name);
                assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Foot /", "Foot claw /", "Heel claw /"].iter().any(|prefix| part.name.starts_with(prefix)) { assert!(p.distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
            } }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
        } }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks motion");
        }
    }
}

#[test]
fn fire_badger_skin_preserves_ground_roots_coat_seams_jaws_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Quilava, "QUILAVA", 13, 39, 10428, 13428),
        (Species::Typhlosion, "TYPHLOSION", 17, 44, 11439, 15262),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let floor: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        for part in &rig.anatomy {
            let owner = if ["Foot /", "Toe /"].iter().any(|p| part.name.starts_with(p)) { Some(0) }
                else if part.name.starts_with("Body /") && species == Species::Quilava { Some(1) }
                else if ["Head /", "Ear /", "Eye socket /", "Eye white /", "Iris /", "Pupil /", "Glint /", "Nose /", "Lip /", "Fang /"].iter().any(|p| part.name.starts_with(p)) { Some(2) }
                else if ["Jaw /", "Tongue /"].iter().any(|p| part.name.starts_with(p)) { Some(3) }
                else { None };
            if let Some(owner) = owner { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [owner,0,0,0], "{name} {}", part.name); assert_eq!(rig.joint_weights[v], [1.,0.,0.,0.]); } }
            if species == Species::Typhlosion && part.name.starts_with("Body /") {
                assert!(part.vertices.clone().all(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).all(|(&j,w)| w == 0. || j <= 1)));
                for owner in [0,1] { assert!(part.vertices.clone().any(|v| rig.joint_indices[v] == [owner,0,0,0] && rig.joint_weights[v] == [1.,0.,0.,0.])); }
            }
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL { for step in 0..=20 {
            rig.sample_into(clip, rig.clip(clip).duration*(step as f32+0.37)/21., Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &rig.anatomy { for v in part.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y >= -0.000001, "{name} {} crosses floor", part.name);
                assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Foot /", "Toe /"].iter().any(|prefix| part.name.starts_with(prefix)) { assert!(p.distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
            } }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
        } }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks motion");
        }
    }
}

#[test]
fn tyranitar_skin_preserves_armor_jaws_grounded_claws_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Tyranitar, "TYRANITAR", 14, 41, 12272, 18056),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let floor: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        for part in &rig.anatomy {
            let owner = if ["Foot /", "Foot claw /"].iter().any(|p| part.name.starts_with(p)) { Some(0) }
                else if ["Body /", "Belly /", "Belly armor /", "Mantle /", "Torso vent /", "Back armor /", "Back vent /", "Back spike /"].iter().any(|p| part.name.starts_with(p)) { Some(1) }
                else if ["Head /", "Brow keel /", "Mouth back /", "Crown spike /", "Cheek plate /", "Eye socket /", "Eye white /", "Iris /", "Pupil /", "Glint /", "Nostril /", "Fang /"].iter().any(|p| part.name.starts_with(p)) { Some(2) }
                else if ["Jaw /", "Tongue /", "Lower tooth /"].iter().any(|p| part.name.starts_with(p)) { Some(3) }
                else { None };
            if let Some(owner) = owner { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [owner,0,0,0], "{name} {}", part.name); assert_eq!(rig.joint_weights[v], [1.,0.,0.,0.]); } }
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL { for step in 0..=20 {
            rig.sample_into(clip, rig.clip(clip).duration*(step as f32+0.37)/21., Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &rig.anatomy { for v in part.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y >= -0.000001, "{name} {} crosses floor", part.name);
                assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Foot /", "Foot claw /"].iter().any(|prefix| part.name.starts_with(prefix)) { assert!(p.distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
            } }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
        } }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks motion");
        }
    }
}

#[test]
fn rock_pupa_skin_preserves_fitted_gaze_grounded_feet_keel_and_independent_controls() {
    for (species, name, joints, parts, vertices, triangles) in [
        (Species::Larvitar, "LARVITAR", 11, 21, 7394, 10816),
        (Species::Pupitar, "PUPITAR", 4, 9, 4503, 5224),
    ] {
        let rig = fixture(species);
        assert_eq!((rig.joints.len(), rig.anatomy.len(), rig.neutral.positions.len(), rig.neutral.indices.len()), (joints, parts, vertices, triangles*3));
        assert!(std::ptr::eq(super::for_species(name).unwrap(), super::rig(species)));
        let floor: Vec<_> = (0..vertices).filter(|&v| rig.neutral.positions[v][1] < 0.0000001).collect();
        assert!(!floor.is_empty());
        for part in &rig.anatomy {
            let owner = if ["Foot /", "Toe /"].iter().any(|p| part.name.starts_with(p)) { Some(0) }
                else if ["Belly /", "Body vent /", "Back vent /", "Shell seam /", "Back seam /"].iter().any(|p| part.name.starts_with(p)) { Some(1) }
                else if ["Head /", "Eye socket /", "Eye white /", "Iris /", "Pupil /", "Glint /", "Lip /", "Cheek tear /", "Nostril /", "Horn /", "Crown spike /"].iter().any(|p| part.name.starts_with(p)) { Some(2) } else { None };
            if let Some(owner) = owner { for v in part.vertices.clone() { assert_eq!(rig.joint_indices[v], [owner,0,0,0], "{name} {}", part.name); assert_eq!(rig.joint_weights[v], [1.,0.,0.,0.]); } }
        }
        let (mut pose, mut matrices) = storage(rig);
        for clip in Clip::ALL { for step in 0..=20 {
            rig.sample_into(clip, rig.clip(clip).duration*(step as f32+0.37)/21., Playback::Clamp, &mut pose).unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for part in &rig.anatomy { for v in part.vertices.clone() {
                let p = rig.skin_point(v, &matrices);
                assert!(p.y >= -0.000001, "{name} {} crosses floor", part.name);
                assert!(p.cmpge(rig.animated_bounds.0).all() && p.cmple(rig.animated_bounds.1).all());
                if ["Foot /", "Foot claw /"].iter().any(|prefix| part.name.starts_with(prefix)) { assert!(p.distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
            } }
            for &v in &floor { assert!(rig.skin_point(v, &matrices).distance(Vec3::from_array(rig.neutral.positions[v])) < 0.000001); }
        } }
        rig.sample_into(Clip::Attack, 0.34, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        for joint in 1..joints {
            let parent = rig.joints[joint].parent.unwrap();
            assert!((0..vertices).any(|v| rig.joint_indices[v].iter().zip(rig.joint_weights[v]).any(|(&j,w)| j as usize == joint && w > 0.8) && rig.skin_point(v, &matrices).distance(matrices[parent].transform_point3(Vec3::from_array(rig.neutral.positions[v]))) > 0.0005), "{name} joint {joint} lacks motion");
        }
    }
}

#[test]
fn electrode_skin_preserves_the_grounded_lower_shell_band_and_fitted_face() {
    let rig = fixture(Species::Electrode);
    assert!(std::ptr::eq(super::for_species("ELECTRODE").unwrap(), super::rig(Species::Electrode)));
    assert_bits(crate::battle_species_models::mesh("ELECTRODE").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
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
fn fossil_scythe_skin_registry_keeps_canonical_geometry_and_fixed_supports() {
    for (species,name,joints,parts) in [(Species::Kabuto,"KABUTO",6,20),(Species::Kabutops,"KABUTOPS",13,27)] {
        let rig=fixture(species); assert_eq!(rig.joints.len(),joints); assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn armored_bug_skin_keeps_canonical_geometry_and_fixed_ground_supports() {
    for (species,name,parts) in [(Species::Pineco,"PINECO",13),(Species::Forretress,"FORRETRESS",18)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),7);assert_eq!(rig.anatomy.len(),parts);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn pitcher_skin_keeps_canonical_geometry_and_fixed_root_or_pitcher_supports() {
    for (species,name,joints) in [(Species::Bellsprout,"BELLSPROUT",10),(Species::Weepinbell,"WEEPINBELL",9),(Species::Victreebel,"VICTREEBEL",10)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn rafflesia_skin_keeps_canonical_geometry_and_fixed_root_feet() {
    for (species,name,joints) in [(Species::Gloom,"GLOOM",11),(Species::Vileplume,"VILEPLUME",12)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn raichu_skin_keeps_canonical_geometry_and_fixed_root_paws() {
    for (species,name,joints) in [(Species::Raichu,"RAICHU",13)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn ekans_skin_keeps_canonical_geometry_and_grounded_coil() {
    for (species,name,joints) in [(Species::Ekans,"EKANS",5)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn arbok_skin_keeps_canonical_geometry_and_grounded_coil() {
    for (species,name,joints) in [(Species::Arbok,"ARBOK",8)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn steelix_skin_keeps_canonical_geometry_and_grounded_steel_supports() {
    for (species,name,joints) in [(Species::Steelix,"STEELIX",10)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let contacts:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!contacts.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for frame in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*frame as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &contacts {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.00001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn dugtrio_skin_keeps_soil_fixed_while_three_capsules_articulate() {
    let rig = fixture(Species::Dugtrio);
    assert!(std::ptr::eq(super::for_species("DUGTRIO").unwrap(), super::rig(Species::Dugtrio)));
    assert_bits(crate::battle_species_models::mesh("DUGTRIO").unwrap().positions.into_iter().flatten(), rig.neutral.positions.iter().flatten().copied());
    let soil: Vec<_> = rig.anatomy.iter().filter(|part| part.name.starts_with("Soil /")).collect();
    assert_eq!(soil.len(), 12);
    assert_eq!(rig.joints.len(), 4);
    assert_eq!(rig.neutral.positions.len(), 12018);
    assert_eq!(rig.neutral.indices.len()/3, 13584);
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
fn tentacle_skin_keeps_canonical_bells_faces_and_floor_tips() {
    for (species,name,joints) in [(Species::Tentacool,"TENTACOOL",6),(Species::Tentacruel,"TENTACRUEL",24)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let tips:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(!tips.is_empty());let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for &v in &tips {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.000001,"{name} crosses floor");}
        }}
    }
}

#[test]
fn jynx_skin_keeps_canonical_gown_and_fixed_hem_through_all_clips() {
    let rig=fixture(Species::Jynx);assert_eq!(rig.joints.len(),22);assert!(std::ptr::eq(super::for_species("JYNX").unwrap(),super::rig(Species::Jynx)));
    assert_bits(crate::battle_species_models::mesh("JYNX").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let hem:Vec<_>=rig.neutral.positions.iter().enumerate().filter_map(|(i,p)| (p[1]<0.000001).then_some(i)).collect();assert!(hem.len()>40);let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=60 {
        rig.sample_into(clip,rig.clip(clip).duration*step as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for &v in &hem {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}
        for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.000001);}
    }}
}

#[test]
fn pig_monkey_skin_keeps_canonical_fur_faces_fists_and_fixed_paws() {
    for (species,name,joints) in [(Species::Mankey,"MANKEY",12),(Species::Primeape,"PRIMEAPE",10)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let feet:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();assert_eq!(feet.len(),2);let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for p in &feet {for v in p.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.000001);}
        }}
    }
}

#[test]
fn feline_skin_keeps_canonical_gaze_whiskers_charms_and_fixed_paws() {
    for (species,name,joints) in [(Species::Meowth,"MEOWTH",15),(Species::Persian,"PERSIAN",13)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let feet:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();assert_eq!(feet.len(),if name=="MEOWTH" {2} else {4});let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for p in &feet {for v in p.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.000001);}
        }}
    }
}

#[test]
fn dodo_skin_keeps_canonical_independent_heads_and_fixed_clawed_feet() {
    for (species,name,joints) in [(Species::Doduo,"DODUO",12),(Species::Dodrio,"DODRIO",19)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let feet:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();assert_eq!(feet.len(),4);let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=60 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/60.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for p in &feet {for v in p.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            for v in 0..rig.neutral.positions.len() {assert!(rig.skin_point(v,&matrices).y>=-0.000001);}
        }}
    }
}

#[test]
fn pidgeotto_skin_keeps_coherent_anatomy_and_grounded_claws() {
    let rig=fixture(Species::Pidgeotto);assert_eq!(rig.joints.len(),14);
    assert!(std::ptr::eq(super::for_species("PIDGEOTTO").unwrap(),super::rig(Species::Pidgeotto)));
    assert_bits(crate::battle_species_models::mesh("PIDGEOTTO").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let toes:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Toe")).collect();assert_eq!(toes.len(),6);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=64 {
        rig.sample_into(clip,rig.clip(clip).duration*step as f32/64.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for p in &toes {for v in p.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all()&&p.cmple(rig.animated_bounds.1).all());}
    }}
}

#[test]
fn raptor_bird_skin_keeps_coherent_anatomy_and_species_specific_claws() {
    for (species,name,joints) in [(Species::Pidgeot,"PIDGEOT",20),(Species::Fearow,"FEAROW",19)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let toes:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with(if species==Species::Fearow {"Airborne digits /"} else {"Grounded digits /"})).collect();assert_eq!(toes.len(),2);
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=64 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/64.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for p in &toes {for v in p.vertices.clone() {
                let rest=Vec3::from_array(rig.neutral.positions[v]);
                let expected=if species==Species::Fearow {
                    let leg=if rest.x<0. {15} else {16};
                    assert_eq!(rig.joint_indices[v],[leg as u16,0,0,0]);
                    assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);
                    matrices[leg].transform_point3(rest)
                } else {rest};
                assert_vec_close(rig.skin_point(v,&matrices),expected,0.000001);
            }}
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all()&&p.cmple(rig.animated_bounds.1).all());}
        }}
    }
}

#[test]
fn leek_duck_skin_keeps_canonical_anatomy_and_fixed_complete_webs() {
    let rig=fixture(Species::FarfetchD);assert_eq!(rig.joints.len(),21);
    assert!(std::ptr::eq(super::for_species("FARFETCH_D").unwrap(),super::rig(Species::FarfetchD)));
    assert_bits(crate::battle_species_models::mesh("FARFETCH_D").unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
    let feet:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")).collect();assert_eq!(feet.len(),4);
    let (mut pose,mut matrices)=storage(rig);
    for clip in Clip::ALL {for step in 0..=64 {
        rig.sample_into(clip,rig.clip(clip).duration*step as f32/64.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
        for p in &feet {for v in p.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
        for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all()&&p.cmple(rig.animated_bounds.1).all());}
    }}
}

#[test]
fn psychic_palm_skin_keeps_coherent_anatomy_grounding_and_complete_feet() {
    for (species,name,joints,parts,source_parts) in [(Species::Exeggcute,"EXEGGCUTE",7,17,46),(Species::Exeggutor,"EXEGGUTOR",30,24,70)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);
        let bytes=candidate(species);let glb=gltf::binary::Glb::from_slice(&bytes).unwrap();let doc:Value=serde_json::from_slice(&glb.json).unwrap();
        let ranges=doc["extras"]["source_part_ranges"].as_array().unwrap();assert_eq!(ranges.len(),parts);
        assert_eq!(ranges.iter().map(|r|r["source_parts"].as_array().map_or(1,Vec::len)).sum::<usize>(),source_parts);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let supports:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /")||p.name.starts_with("Sole /")||p.name.starts_with("Claw /")).collect();
        assert_eq!(supports.len(),if species==Species::Exeggutor {6} else {0});
        assert_eq!(supports.iter().map(|p|p.vertices.len()).sum::<usize>(),if species==Species::Exeggutor {3718} else {0});
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=64 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/64.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for p in &supports {for v in p.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all()&&p.cmple(rig.animated_bounds.1).all());}
        }}
    }
}

#[test]
fn bulb_saur_skin_keeps_complete_supports_anatomy_and_animation_envelopes() {
    for (species,name,joints,parts,logical) in [(Species::Ivysaur,"IVYSAUR",21,26,58),(Species::Venusaur,"VENUSAUR",32,29,100)] {
        let rig=fixture(species);assert_eq!(rig.joints.len(),joints);assert_eq!(rig.anatomy.len(),parts);
        let bytes=candidate(species);let glb=gltf::binary::Glb::from_slice(&bytes).unwrap();let doc:Value=serde_json::from_slice(&glb.json).unwrap();
        let ranges=doc["extras"]["source_part_ranges"].as_array().unwrap();assert_eq!(ranges.len(),parts);
        assert_eq!(ranges.iter().map(|r|r["source_parts"].as_array().map_or(1,Vec::len)).sum::<usize>(),logical);
        assert!(std::ptr::eq(super::for_species(name).unwrap(),super::rig(species)));
        assert_bits(crate::battle_species_models::mesh(name).unwrap().positions.into_iter().flatten(),rig.neutral.positions.iter().flatten().copied());
        let feet:Vec<_>=rig.anatomy.iter().filter(|p|p.name.starts_with("Forefoot /")||p.name.starts_with("Hindfoot /")).collect();
        assert_eq!(feet.len(),8);assert_eq!(feet.iter().map(|p|p.vertices.len()).sum::<usize>(),2688);
        let (mut pose,mut matrices)=storage(rig);
        for clip in Clip::ALL {for step in 0..=64 {
            rig.sample_into(clip,rig.clip(clip).duration*step as f32/64.,Playback::Clamp,&mut pose).unwrap();rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for part in &feet {for v in part.vertices.clone() {assert_vec_close(rig.skin_point(v,&matrices),Vec3::from_array(rig.neutral.positions[v]),0.000001);}}
            for v in 0..rig.neutral.positions.len() {let p=rig.skin_point(v,&matrices);assert!(p.y>=-0.000001);assert!(p.cmpge(rig.animated_bounds.0).all()&&p.cmple(rig.animated_bounds.1).all());}
        }}
    }
}
