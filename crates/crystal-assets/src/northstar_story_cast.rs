//! Accepted cast art belongs to the story installer, rather than an optional pack.
use anyhow::Result;

use crate::CompiledGamePack;

pub(crate) fn install(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let source = include_str!(
        "../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/native-pixels-v4.json"
    );
    let art = crate::minnesota_native_npc::compile(source, None)?;
    let pack = crate::bind_minnesota_trainer_art(
        base,
        include_str!("../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/binding-v1.json"),
        &art,
    )?;
    let art = crate::minnesota_native_npc::compile(
        include_str!("../../../planning/minnesota/npcs/cast/01-reed/native-pixels-v3.json"),
        Some(include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/01-reed/native-pixels-v2.json"
        )),
    )?;
    crate::bind_minnesota_trainer_art(
        &pack,
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/01-reed.json"
        ),
        &art,
    )
}
