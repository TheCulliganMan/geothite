/// Battle styles, not difficulty levels. Every class receives the same search
/// budget and tactical safeguards; style only values close strategic choices.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TrainerBattleProfile {
    #[default]
    Balanced,
    Offensive,
    Setup,
    Disruption,
    Endurance,
    Switching,
}

impl TrainerBattleProfile {
    pub const ALL: [Self; 6] = [
        Self::Balanced,
        Self::Offensive,
        Self::Setup,
        Self::Disruption,
        Self::Endurance,
        Self::Switching,
    ];

    pub fn for_class(class: &str) -> Self {
        match class {
            "YOUNGSTER" | "LASS" | "SAILOR" | "BLACKBELT_T" | "KARATE_KING" | "CHUCK" | "BRUNO"
            | "LT_SURGE" | "BLAINE" | "BROCK" | "PRYCE" | "FALKNER" | "CAL" => Self::Offensive,
            "PSYCHIC_T" | "SAGE" | "MEDIUM" | "SCIENTIST" | "SUPER_NERD" | "WILL" | "SABRINA"
            | "CLAIR" => Self::Setup,
            "BUG_CATCHER" | "BUGSY" | "KOGA" | "JANINE" | "ROCKET" | "POKEMANIAC" | "JUGGLER"
            | "JUGGLER_T" | "KAREN" => Self::Disruption,
            "FISHER" | "HIKER" | "POKEFANM" | "POKEFANF" | "POKEFAN_M" | "POKEFAN_F" | "BEAUTY"
            | "SWIMMERM" | "SWIMMERF" | "SWIMMER_M" | "SWIMMER_F" | "ERIKA" | "JASMINE"
            | "WHITNEY" | "MISTY" | "GENTLEMAN" => Self::Endurance,
            "BIRD_KEEPER" | "PICNICKER" | "CAMPER" | "BIKER" | "FIREBREATHER" | "BOARDER"
            | "SKIER" | "GRUNTM" | "GRUNTF" | "EXECUTIVEM" | "EXECUTIVEF" | "GRUNT_M"
            | "GRUNT_F" | "EXECUTIVE_M" | "EXECUTIVE_F" | "BLUE" => Self::Switching,
            // Rivals, Cooltrainers, the Champion, RED, Battle Tower opponents,
            // and new/custom classes retain the full-strength balanced policy.
            _ => Self::Balanced,
        }
    }
}
