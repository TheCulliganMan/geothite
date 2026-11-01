//! Repair Duluth's flat cliff crossings with native outdoor stair flights.

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Repairs the current Duluth exterior while retaining every indexed activity.
///
/// Joins the two falsely separated western upper streets at one height, restores
/// the garden's east wall, and gives eleven actual rock crossings cave36 stairs.
/// Requires frozen before/after layouts and a complete native anchor contract.
/// Writes no files and leaves the borrowed parent unchanged.
///
/// # Errors
/// Rejects source or terrain drift, unavailable native art, unsafe arrivals,
/// inaccessible doors/actors, invalid hops, or changed regional connections.
pub fn revise_duluth_cliff_stairs(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    for (source, expected) in [
        (
            before_json,
            "4de2582b08a13a7e5bb8d2abba6559fc60f80fb19852094b6b49a1b88259128a",
        ),
        (
            after_json,
            "9fa5ae81c728efc12121b52f4ebf6aa47740ba83b8544274a98c0b02b49d9b8b",
        ),
        (
            contract_json,
            "63ae2733c69197248bf918a533e9523f0a2cbac453bf46be6a42a5b28bd95b9a",
        ),
    ] {
        ensure!(
            format!("{:x}", Sha256::digest(source.as_bytes())) == expected,
            "Duluth cliff repair source drift"
        );
    }
    crate::validate_compact_superior_geography(base)?;
    let before: Layout = serde_json::from_str(before_json)?;
    let city = base.data.maps.get("MnDuluth").context("current Duluth")?;
    ensure!(
        before.compile()? == city.blocks,
        "Duluth terrain preimage differs"
    );
    let art = crate::north_shore_art::append_native_cave_stairs(
        base,
        "mn_duluth_expanded",
        0x8c,
        &[0x36],
        "minnesota-duluth-cliff-stairs-v1",
    )?;
    let revised = crate::recompose_authored_city(&art, after_json, contract_json)?;
    crate::validate_compact_superior_geography(&revised)?;
    Ok(revised)
}
