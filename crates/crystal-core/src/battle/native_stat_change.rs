//! Parameters encoded by Crystal's ordinary stat-change move effects.
use crate::models::Move;
use crate::models::pokemon::Stat;

// Native packs may omit the optional convenience annotations. The effect
// program still specifies the stat and number of stages. Explicit authored
// annotations retain precedence, including zero/partial parameter pairs.
pub(super) fn parameters(move_data: &Move) -> Option<(Stat, i8)> {
    match (move_data.stat, move_data.amount) {
        (Some(stat), Some(amount)) => Some((stat, amount)),
        (None, None) => match move_data.effect.as_str() {
            "ATTACK_UP" => Some((Stat::Attack, 1)),
            "ATTACK_UP_2" => Some((Stat::Attack, 2)),
            "DEFENSE_UP" => Some((Stat::Defense, 1)),
            "DEFENSE_UP_2" => Some((Stat::Defense, 2)),
            "SPEED_UP" => Some((Stat::Speed, 1)),
            "SPEED_UP_2" => Some((Stat::Speed, 2)),
            "SPECIAL_ATTACK_UP" => Some((Stat::SpecialAttack, 1)),
            "SPECIAL_ATTACK_UP_2" => Some((Stat::SpecialAttack, 2)),
            "SPECIAL_DEFENSE_UP" => Some((Stat::SpecialDefense, 1)),
            "SPECIAL_DEFENSE_UP_2" => Some((Stat::SpecialDefense, 2)),
            "EVASION_UP" => Some((Stat::Evasion, 1)),
            "EVASION_UP_2" => Some((Stat::Evasion, 2)),
            "ATTACK_DOWN" => Some((Stat::Attack, -1)),
            "ATTACK_DOWN_2" => Some((Stat::Attack, -2)),
            "DEFENSE_DOWN" => Some((Stat::Defense, -1)),
            "DEFENSE_DOWN_2" => Some((Stat::Defense, -2)),
            "SPEED_DOWN" => Some((Stat::Speed, -1)),
            "SPEED_DOWN_2" => Some((Stat::Speed, -2)),
            "SPECIAL_ATTACK_DOWN" => Some((Stat::SpecialAttack, -1)),
            "SPECIAL_ATTACK_DOWN_2" => Some((Stat::SpecialAttack, -2)),
            "SPECIAL_DEFENSE_DOWN" => Some((Stat::SpecialDefense, -1)),
            "SPECIAL_DEFENSE_DOWN_2" => Some((Stat::SpecialDefense, -2)),
            "ACCURACY_DOWN" => Some((Stat::Accuracy, -1)),
            "ACCURACY_DOWN_2" => Some((Stat::Accuracy, -2)),
            "EVASION_DOWN" => Some((Stat::Evasion, -1)),
            "EVASION_DOWN_2" => Some((Stat::Evasion, -2)),
            _ => None,
        },
        _ => None,
    }
}
