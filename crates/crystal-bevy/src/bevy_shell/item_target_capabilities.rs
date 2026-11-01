// Scope declares a PP payload. Its optional amount controls the core effect:
// None means full restoration, rather than a missing target capability.
fn item_targets_party_move_fields(
    pp_restore_scope: Option<&str>,
    _pp_restore_points: Option<u8>,
    pp_up_stages: Option<u8>,
) -> bool {
    pp_up_stages.is_some() || pp_restore_scope == Some("MOVE")
}

fn item_targets_party_pokemon_fields(item: &crate::RuntimeItemCatalogSnapshot) -> bool {
    item.party_hp_restore
        || item.revive_hp_percent.is_some()
        || !item.status_heals.is_empty()
        || item.confusion_heal == Some(true)
        || item.pp_restore_scope.as_deref() == Some("POKEMON")
        || item.vitamin_stat.is_some()
        || item.rare_candy_level_gain.is_some()
        || item.party_special_effect
}
