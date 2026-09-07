use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use crystal_core::battle::damage::MoveCategory;
use serde::Deserialize;

use crate::{
    CompiledGamePack, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};

pub const MODERN_MOVE_SPLIT_MANIFEST_ID: &str = "modern-move-split";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    source: SourceIdentity,
    moves: BTreeMap<String, MoveCategory>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceIdentity {
    repository: String,
    commit: String,
}

fn catalog() -> Result<BTreeMap<String, MoveCategory>> {
    let source: Source = serde_json::from_str(include_str!(
        "../../../../modpacks/modern-move-split/data.json"
    ))?;
    ensure!(
        source.source.repository == "https://github.com/pret/pokeplatinum"
            && source.source.commit == "bca37652996330898fdd2408281ea419b8c995c7",
        "unexpected move split source provenance"
    );
    ensure!(
        source.moves.len() == 251,
        "move split must classify all 251 Crystal moves"
    );
    Ok(source.moves)
}

/// Apply per-move categories while retaining the base pack's species stats,
/// move properties, type matchups, and all other Crystal mechanics.
pub fn build_modern_move_split_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == MODERN_MOVE_SPLIT_MANIFEST_ID)
            && base.data.type_categories.moves.is_empty(),
        "base pack already has a move split"
    );
    let categories = catalog()?;
    ensure!(
        categories.keys().eq(base.data.moves.keys()),
        "move split catalog must exactly cover the base pack's moves"
    );
    let mut pack = base.clone();
    pack.data.type_categories.moves = categories;
    pack.report
        .manifests
        .push(MODERN_MOVE_SPLIT_MANIFEST_ID.to_string());
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_move_split_catalog_is_exhaustive_and_classifies_cross_type_moves() {
        let moves = catalog().unwrap();
        for id in [
            "FIRE_PUNCH",
            "ICE_PUNCH",
            "THUNDERPUNCH",
            "BITE",
            "WATERFALL",
        ] {
            assert_eq!(moves[id], MoveCategory::Physical, "{id}");
        }
        for id in ["SHADOW_BALL", "GUST", "HYPER_BEAM", "HIDDEN_POWER"] {
            assert_eq!(moves[id], MoveCategory::Special, "{id}");
        }
        assert_eq!(moves["GROWL"], MoveCategory::Status);
    }
}
