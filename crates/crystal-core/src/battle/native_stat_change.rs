//! Parameters encoded by Crystal's direct and damaging stat-change effects.
use crate::models::Move;
use crate::models::pokemon::Stat;

// Native packs may omit the optional convenience annotations. The effect
// program still specifies the stat and number of stages. Explicit authored
// annotations retain precedence, including zero/partial parameter pairs.
pub(super) fn parameters(move_data: &Move) -> Option<(Stat, i8)> {
    match (move_data.stat, move_data.amount) {
        (Some(stat), Some(amount)) => Some((stat, amount)),
        (None, None) => match move_data.effect.as_str() {
            "ATTACK_UP" | "ATTACK_UP_HIT" => Some((Stat::Attack, 1)),
            "ATTACK_UP_2" => Some((Stat::Attack, 2)),
            "DEFENSE_UP" | "DEFENSE_UP_HIT" => Some((Stat::Defense, 1)),
            "DEFENSE_UP_2" => Some((Stat::Defense, 2)),
            "SPEED_UP" => Some((Stat::Speed, 1)),
            "SPEED_UP_2" => Some((Stat::Speed, 2)),
            "SPECIAL_ATTACK_UP" => Some((Stat::SpecialAttack, 1)),
            "SPECIAL_ATTACK_UP_2" => Some((Stat::SpecialAttack, 2)),
            "SPECIAL_DEFENSE_UP" => Some((Stat::SpecialDefense, 1)),
            "SPECIAL_DEFENSE_UP_2" => Some((Stat::SpecialDefense, 2)),
            "EVASION_UP" => Some((Stat::Evasion, 1)),
            "EVASION_UP_2" => Some((Stat::Evasion, 2)),
            "ATTACK_DOWN" | "ATTACK_DOWN_HIT" => Some((Stat::Attack, -1)),
            "ATTACK_DOWN_2" => Some((Stat::Attack, -2)),
            "DEFENSE_DOWN" | "DEFENSE_DOWN_HIT" => Some((Stat::Defense, -1)),
            "DEFENSE_DOWN_2" => Some((Stat::Defense, -2)),
            "SPEED_DOWN" | "SPEED_DOWN_HIT" => Some((Stat::Speed, -1)),
            "SPEED_DOWN_2" => Some((Stat::Speed, -2)),
            "SPECIAL_ATTACK_DOWN" | "SPECIAL_ATTACK_DOWN_HIT" => Some((Stat::SpecialAttack, -1)),
            "SPECIAL_ATTACK_DOWN_2" => Some((Stat::SpecialAttack, -2)),
            "SPECIAL_DEFENSE_DOWN" | "SPECIAL_DEFENSE_DOWN_HIT" => Some((Stat::SpecialDefense, -1)),
            "SPECIAL_DEFENSE_DOWN_2" => Some((Stat::SpecialDefense, -2)),
            "ACCURACY_DOWN" | "ACCURACY_DOWN_HIT" => Some((Stat::Accuracy, -1)),
            "ACCURACY_DOWN_2" => Some((Stat::Accuracy, -2)),
            "EVASION_DOWN" | "EVASION_DOWN_HIT" => Some((Stat::Evasion, -1)),
            "EVASION_DOWN_2" => Some((Stat::Evasion, -2)),
            _ => None,
        },
        _ => None,
    }
}
