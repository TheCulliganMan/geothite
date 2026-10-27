//! The complete Pokemon inputs consumed by the damage formula.
//!
//! Keeping the formula behind this view makes preview cache equality include
//! every input, without retaining unrelated HP, PP, party or learnset data.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::models::{Pokemon, Stat};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct DamagePokemon<'a> {
    pub id: Cow<'a, str>,
    pub type1: Cow<'a, str>,
    pub type2: Cow<'a, str>,
    pub level: u8,
    pub attack: u16,
    pub defense: u16,
    pub special_attack: u16,
    pub special_defense: u16,
    pub stages: Cow<'a, BTreeMap<Stat, i8>>,
    pub status: Cow<'a, Option<String>>,
    pub item: Cow<'a, Option<String>>,
}

impl<'a> From<&'a Pokemon> for DamagePokemon<'a> {
    fn from(pokemon: &'a Pokemon) -> Self {
        Self {
            id: Cow::Borrowed(&pokemon.species.id),
            type1: Cow::Borrowed(&pokemon.species.type1),
            type2: Cow::Borrowed(&pokemon.species.type2),
            level: pokemon.level,
            attack: pokemon.attack,
            defense: pokemon.defense,
            special_attack: pokemon.special_attack,
            special_defense: pokemon.special_defense,
            stages: Cow::Borrowed(&pokemon.stat_boosts),
            status: Cow::Borrowed(&pokemon.status),
            item: Cow::Borrowed(&pokemon.item),
        }
    }
}

impl DamagePokemon<'_> {
    pub(crate) fn into_owned(self) -> DamagePokemon<'static> {
        DamagePokemon {
            id: Cow::Owned(self.id.into_owned()),
            type1: Cow::Owned(self.type1.into_owned()),
            type2: Cow::Owned(self.type2.into_owned()),
            level: self.level,
            attack: self.attack,
            defense: self.defense,
            special_attack: self.special_attack,
            special_defense: self.special_defense,
            stages: Cow::Owned(self.stages.into_owned()),
            status: Cow::Owned(self.status.into_owned()),
            item: Cow::Owned(self.item.into_owned()),
        }
    }
}
