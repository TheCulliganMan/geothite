//! Minnesota's eight native gym interiors, independent from unfinished cities.
use anyhow::{Context, Result};

use crate::gym_design::GymDesign;
use crate::{CompiledGamePack, gym_art, gym_publication, gym_scripts};

/// Publishes eight authored gym interiors and a separate native review hall.
///
/// Terrain uses explicit rows and native metatile pixels. Declared wind slats
/// receive native directional walk collision; other source collision is retained.
/// Puzzles, battles, badges and retryable TM rewards use production scripts.
/// The hall supplies reciprocal doors until each real city is ready for binding;
/// it never substitutes for city progression. Existing HOME, recovery, maps,
/// runtime files, audio and unrelated catalogs are preserved. No badge or event
/// is granted to a player by installation.
///
/// # Errors
/// Returns an error for invalid design, occupied map/art/trainer identities,
/// missing or incompatible source art, illegal moves, unsafe geometry, blocked
/// actors/returns, malformed native scripts or failed pack verification. The
/// borrowed input pack and design string remain unchanged on failure.
pub fn build_minnesota_gym_review_pack(
    base: &CompiledGamePack,
    design_json: &str,
) -> Result<CompiledGamePack> {
    build_review_pack(base, design_json, None)
}

/// Publishes one gym and a small review vestibule for individual authoring.
///
/// # Errors
/// Returns the same validation errors as [`build_minnesota_gym_review_pack`],
/// or an error when `gym_slot` does not identify one of the eight authored gyms.
pub fn build_minnesota_gym_review_room(
    base: &CompiledGamePack,
    design_json: &str,
    gym_slot: u8,
) -> Result<CompiledGamePack> {
    build_review_pack(base, design_json, Some(gym_slot))
}

fn build_review_pack(
    base: &CompiledGamePack,
    design_json: &str,
    gym_slot: Option<u8>,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut design: GymDesign =
        serde_json::from_str(design_json).context("Minnesota gym design")?;
    design.validate()?;
    if let Some(slot) = gym_slot {
        design.gyms.retain(|gym| gym.slot == slot);
        anyhow::ensure!(design.gyms.len() == 1, "gym slot must be 1..=8");
    }
    let mut pack = base.clone();
    pack.data.materialize_global_scripts()?;
    let mut modules = Vec::new();
    for (index, gym) in design.gyms.iter().enumerate() {
        let symbols = gym_art::install(&mut pack, gym)?;
        let mut room = gym_publication::room(&pack, gym, &symbols, index)?;
        gym_scripts::install(&mut pack, &mut room, gym, &symbols)?;
        gym_publication::validate(&pack, &room, gym, &symbols)?;
        modules.push((room, gym.arrival_core));
    }
    let hall_arrival = if design.gyms.len() == 1 {
        [4, 6]
    } else {
        [17, 12]
    };
    modules.push((gym_publication::hall(&pack, &design.gyms)?, hall_arrival));
    gym_publication::publish(pack, modules)
}
