#[path = "../src/bin/pack_core/source_battle_rules.rs"]
mod source_battle_rules;
#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;

#[test]
fn wobble_rejects_incomplete_or_unordered_source() {
    let rows = "db 1, 63\ndb 255, 255";
    let table = source_battle_rules::wobble(rows).unwrap();
    assert_eq!(table[0]["chance"], 63);
    assert!(source_battle_rules::wobble("db 1, 63").is_err());
    assert!(source_battle_rules::wobble("db 2, 75\ndb 1, 63\ndb 255, 255").is_err());
}

#[test]
fn weather_distinguishes_type_and_effect_modifiers() {
    let source = "WeatherTypeModifiers:\ndb WEATHER_RAIN, WATER, MORE_EFFECTIVE\ndb -1\nWeatherMoveModifiers:\ndb WEATHER_RAIN, EFFECT_SOLARBEAM, NOT_VERY_EFFECTIVE\ndb -1";
    let constants =
        "DEF EFFECTIVE EQU 10\nDEF MORE_EFFECTIVE EQU 15\nDEF NOT_VERY_EFFECTIVE EQU 05";
    let table = source_battle_rules::weather(source, constants).unwrap();
    assert_eq!(
        table["type_modifiers"]["WEATHER_RAIN"]["WATER"]["numerator"],
        15
    );
    assert_eq!(
        table["move_effect_modifiers"]["WEATHER_RAIN"]["SOLARBEAM"]["numerator"],
        5
    );
    assert!(source_battle_rules::weather(&source.replace("db -1", ""), constants).is_err());
}

#[test]
fn type_tables_preserve_source_split_and_foresight_boundary() {
    let types = "DEF PHYSICAL EQU const_value\nconst NORMAL\nDEF UNUSED_TYPES EQU const_value\nconst CURSE_TYPE\nDEF SPECIAL EQU const_value\nconst FIRE\nDEF TYPES_END EQU const_value";
    let rows = "db FIRE, FIRE, NOT_VERY_EFFECTIVE\ndb -2\ndb NORMAL, GHOST, NO_EFFECT\ndb -1";
    let constants = "DEF EFFECTIVE EQU 10\nDEF NOT_VERY_EFFECTIVE EQU 05\nDEF NO_EFFECT EQU 00";
    let (categories, table) = source_battle_rules::type_tables(types, rows, constants).unwrap();
    assert_eq!(categories["physical"], serde_json::json!(["NORMAL"]));
    assert_eq!(categories["special"], serde_json::json!(["FIRE"]));
    assert_eq!(table["matchups"][0]["multiplier"]["numerator"], 5);
    assert_eq!(table["matchups"][0]["multiplier"]["denominator"], 10);
    assert_eq!(table["foresight_matchups"][0]["defender"], "GHOST");
    assert!(
        source_battle_rules::type_tables(types, &rows.replace("db -2", ""), constants).is_err()
    );
}

#[test]
fn priorities_expand_authored_base_and_preserve_move_override() {
    let effects = "const EFFECT_NORMAL_HIT\nconst EFFECT_PROTECT\nconst EFFECT_SP_ATK_UP\nconst EFFECT_SP_DEF_UP_2\n";
    let rows = "db EFFECT_PROTECT, 3\ndb -1\n";
    let constants = "DEF BASE_PRIORITY EQU 1\n";
    let engine = "GetMovePriority:\ncp VITAL_THROW\nld a, 0\nret z\ncall GetMoveEffect\n";
    let table = source_battle_rules::priorities(effects, rows, constants, engine).unwrap();
    assert_eq!(table["effect_priorities"]["NORMAL_HIT"], 1);
    assert_eq!(table["effect_priorities"]["SPECIAL_ATTACK_UP"], 1);
    assert_eq!(table["effect_priorities"]["SPECIAL_DEFENSE_UP_2"], 1);
    assert_eq!(table["effect_priorities"]["PROTECT"], 3);
    assert_eq!(table["move_priorities"][0]["move"], "VITAL_THROW");
    assert_eq!(table["move_priorities"][0]["priority"], 0);
    assert!(
        source_battle_rules::priorities(effects, "db EFFECT_UNKNOWN, 2\ndb -1", constants, engine)
            .is_err()
    );
    assert!(
        source_battle_rules::priorities(effects, "db EFFECT_PROTECT, 3", constants, engine)
            .is_err()
    );
}

#[test]
fn ratios_keep_authored_rounding_and_stage_order() {
    let stat = "db 25,100\ndb 28,100\ndb 33,100\ndb 40,100\ndb 50,100\ndb 66,100\ndb 1,1\ndb 15,10\ndb 2,1\ndb 25,10\ndb 3,1\ndb 35,10\ndb 4,1\n";
    let accuracy = "db 33,100\ndb 36,100\ndb 43,100\ndb 50,100\ndb 60,100\ndb 75,100\ndb 1,1\ndb 133,100\ndb 166,100\ndb 2,1\ndb 233,100\ndb 133,50\ndb 3,1\n";
    let tables = source_battle_rules::multipliers(stat, accuracy).unwrap();
    assert_eq!(tables.stat[5].as_fraction().multiply_floor(100), 66);
    assert_eq!(tables.accuracy[11].as_fraction().multiply_floor(100), 266);
    assert_eq!(tables.stat[6].numerator, 1);
    assert_eq!(tables.accuracy.len(), 13);
    assert!(source_battle_rules::multipliers(&stat.replace("db 28,100\n", ""), accuracy).is_err());
    assert!(source_battle_rules::multipliers(&stat.replace("db 1,1", "db 1,0"), accuracy).is_err());
}
