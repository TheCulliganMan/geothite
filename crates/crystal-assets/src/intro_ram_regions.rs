//! Source-authored intro RAM region effects shared by the clock and presentation.
use anyhow::{Context, Result};
use std::collections::BTreeMap;

pub fn runtime_intro_apply_region_effect(
    regions: &mut BTreeMap<String, Vec<u8>>,
    operation: &super::RuntimePresentationOperation,
    memory: &mut BTreeMap<String, u16>,
    resolve: &mut dyn FnMut(&str) -> Result<Vec<u8>>,
) -> Result<()> {
    let text = |key: &str| operation.fields.get(key).and_then(serde_json::Value::as_str)
        .with_context(|| format!("CrystalIntro {} has no source region {key}", operation.op));
    let number = |key: &str| operation.fields.get(key).and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .with_context(|| format!("CrystalIntro {} has no source region {key}", operation.op));
    if let Some(condition) = operation.fields.get("condition") {
        if let (Some(source), Some(predicate)) = (condition.get("source").and_then(serde_json::Value::as_str),
            condition.get("predicate").and_then(serde_json::Value::as_str)) {
            let value = memory.get(source).copied().with_context(|| format!("CrystalIntro source condition {source} is uninitialized"))?;
            if (predicate == "zero" && value != 0) || (predicate == "nonzero" && value == 0) { return Ok(()); }
        }
    }
    if let Some(effects) = operation.fields.get("source_memory_effects") {
        let effects = effects.as_object().context("CrystalIntro source memory effects are not byte writes")?;
        for (target, value) in effects {
            let value = value.as_u64().filter(|value| *value <= 255).context("CrystalIntro source memory effect exceeds one byte")?;
            memory.insert(target.clone(), value as u16);
        }
    }
    if matches!(operation.op.as_str(), "write_memory_byte" | "palette_transfer_request") {
        if let Some(value) = operation.fields.get("value").and_then(serde_json::Value::as_u64) {
            anyhow::ensure!(value <= 255, "CrystalIntro source region scalar write exceeds one byte");
            memory.insert(text("target")?.to_string(), value as u16);
        }
    }
    if let Some(initializer) = operation.fields.get("source_sprite_init_timing") {
        let count = match initializer.get("routine").and_then(serde_json::Value::as_str) {
            Some("InitSpriteAnimStruct") => 1,
            Some("CrystalIntro_InitUnownAnim") => 4,
            _ => anyhow::bail!("CrystalIntro source sprite initializer has an unsupported routine"),
        };
        let slots = regions.get_mut("wSpriteAnimationStructs")
            .context("CrystalIntro source sprite initializer preceded its RAM clear")?;
        anyhow::ensure!(slots.len() == 160, "CrystalIntro source sprite RAM differs from its 10x16 layout");
        for _ in 0..count {
            if let Some(slot) = (0..10).find(|slot| slots[slot * 16] == 0) {
                let previous = u8::try_from(*memory.get("wSpriteAnimCount")
                    .context("CrystalIntro source sprite counter is missing")?)
                    .context("CrystalIntro source sprite counter exceeds one byte")?;
                let index = previous.wrapping_add(1).max(1);
                memory.insert("wSpriteAnimCount".to_string(), u16::from(index));
                slots[slot * 16] = index;
            }
        }
    }
    match operation.op.as_str() {
        "save_memory_byte" if operation.fields.contains_key("result") => {
            let value = *memory.get(text("source")?).context("CrystalIntro source stack save byte is uninitialized")?;
            memory.insert(text("result")?.to_string(), value);
        }
        "restore_memory_byte" if operation.fields.contains_key("result") => {
            let value = *memory.get(text("result")?).context("CrystalIntro source stack restore result is uninitialized")?;
            memory.insert(text("target")?.to_string(), value);
        }
        "decompress_lz3_resource" => {
            let data = resolve(text("resource")?)?;
            let decoded = super::runtime_intro_decode_lz3(&data, 64 * 1024)?;
            regions.insert(text("target")?.to_string(), decoded);
        }
        "fill_memory" => {
            let target = text("target")?;
            let count = number("byte_count")?;
            let offset = operation.fields.get("target_offset").and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()).unwrap_or(0);
            let value = u8::try_from(number("value")?).context("CrystalIntro region fill exceeds one byte")?;
            let end = offset.checked_add(count).context("CrystalIntro region fill range overflows")?;
            anyhow::ensure!(end <= 64 * 1024, "CrystalIntro region fill exceeds bounded RAM");
            let region = regions.entry(target.to_string()).or_default();
            region.resize(region.len().max(end), 0);
            region[offset..end].fill(value);
            if target == "wSpriteAnimData" {
                anyhow::ensure!(offset == 0 && count == 193 && value == 0,
                    "CrystalIntro sprite RAM clear differs from its source layout");
                regions.insert("wSpriteAnimDict".to_string(), vec![0; 20]);
                regions.insert("wSpriteAnimationStructs".to_string(), vec![0; 160]);
                for key in ["wSpriteAnimCount", "wGlobalAnimXOffset", "wGlobalAnimYOffset"] {
                    memory.insert(key.to_string(), 0);
                }
            }
        }
        "write_memory_bytes" => {
            let bytes = operation.fields.get("values").and_then(serde_json::Value::as_array)
                .context("CrystalIntro source region byte write has no values")?.iter()
                .map(|byte| byte.as_u64().and_then(|value| u8::try_from(value).ok())
                    .context("CrystalIntro source region write is not an exact byte"))
                .collect::<Result<Vec<_>>>()?;
            let target = regions.get_mut(text("target")?)
                .context("CrystalIntro source region byte write preceded initialization")?;
            anyhow::ensure!(bytes.len() <= target.len(), "CrystalIntro source region byte write exceeds its RAM");
            target[..bytes.len()].copy_from_slice(&bytes);
        }
        "copy_memory" | "copy_strided_memory" => {
            let source = text("source")?;
            let data = if let Some(region) = regions.get(source) { region.clone() } else { resolve(source)? };
            let target = text("target")?.to_string();
            if operation.op == "copy_memory" {
                let count = number("byte_count")?;
                anyhow::ensure!(count <= data.len(), "CrystalIntro source region copy reads past its source");
                regions.insert(target, data[..count].to_vec());
            } else {
                let rows = number("rows")?;
                let row_bytes = number("row_bytes")?;
                let source_stride = number("source_stride")?;
                let target_stride = number("target_stride")?;
                let size = rows.checked_mul(target_stride).context("CrystalIntro strided target overflows")?;
                anyhow::ensure!(size <= 64 * 1024, "CrystalIntro strided copy exceeds bounded RAM");
                let mut output = vec![0; size];
                for row in 0..rows {
                    let begin = row.checked_mul(source_stride).context("CrystalIntro strided source overflows")?;
                    let end = begin.checked_add(row_bytes).context("CrystalIntro strided source range overflows")?;
                    anyhow::ensure!(end <= data.len() && row_bytes <= target_stride,
                        "CrystalIntro strided copy exceeds its source or destination row");
                    output[row * target_stride..row * target_stride + row_bytes].copy_from_slice(&data[begin..end]);
                }
                regions.insert(target, output);
            }
        }
        "tilemap_xor" | "conditional_tilemap_xor" => {
            if operation.op == "conditional_tilemap_xor" {
                let clock = memory.get(text("clock")?).copied().context("CrystalIntro tilemap XOR clock is missing")?;
                let mask = u16::try_from(number("clock_mask")?).context("CrystalIntro tilemap XOR clock mask exceeds u16")?;
                let swap = operation.fields.get("swap_phase").and_then(|phase| phase.get("equals"))
                    .and_then(serde_json::Value::as_u64).context("CrystalIntro tilemap XOR swap phase is missing")?;
                let current = u64::from(clock & mask);
                for (phase, write) in [("prepare_phase", "write"), ("swap_phase", "completion_write")] {
                    if operation.fields.get(phase).and_then(|phase| phase.get("equals"))
                        .and_then(serde_json::Value::as_u64) == Some(current) {
                        let write = &operation.fields[phase][write];
                        let target = write["target"].as_str().context("CrystalIntro XOR phase has no write target")?;
                        let value = write["value"].as_u64().filter(|value| *value <= 255)
                            .context("CrystalIntro XOR phase has no exact byte write")?;
                        memory.insert(target.to_string(), value as u16);
                    }
                }
                if current != swap { return Ok(()); }
            }
            let xor = u8::try_from(number("xor_mask")?).context("CrystalIntro tilemap XOR mask exceeds one byte")?;
            let target = regions.get_mut(text("target")?)
                .context("CrystalIntro tilemap XOR preceded source tilemap initialization")?;
            let condition = operation.fields.get("condition");
            let nonzero = condition.and_then(|value| value.get("nonzero")).and_then(serde_json::Value::as_bool)
                .or_else(|| operation.fields.get("nonzero_only").and_then(serde_json::Value::as_bool)).unwrap_or(false);
            let less_than = condition.and_then(|value| value.get("unsigned_less_than")).and_then(serde_json::Value::as_u64)
                .or_else(|| operation.fields.get("unsigned_less_than").and_then(serde_json::Value::as_u64));
            for byte in target {
                if (!nonzero || *byte != 0) && less_than.is_none_or(|limit| u64::from(*byte) < limit) { *byte ^= xor; }
            }
            if let Some(write) = operation.fields.get("completion_write") {
                let target = write["target"].as_str().context("CrystalIntro XOR has no completion target")?;
                let value = write["value"].as_u64().filter(|value| *value <= 255)
                    .context("CrystalIntro XOR has no exact completion byte")?;
                memory.insert(target.to_string(), value as u16);
            }
        }
        "deinitialize_all_sprites" => {
            let slots = regions.get_mut("wSpriteAnimationStructs")
                .context("CrystalIntro source sprite teardown preceded its RAM clear")?;
            anyhow::ensure!(slots.len() == 160, "CrystalIntro source sprite teardown has a different RAM layout");
            for slot in 0..10 { slots[slot * 16] = 0; }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn operation(mut value: serde_json::Value) -> super::super::RuntimePresentationOperation {
        value["source_span"] = json!({"file":"synthetic-region-test","start_line":1,"end_line":1});
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn source_sprite_region_clear_allocates_real_holes_and_preserves_counter_on_teardown() {
        let mut memory = BTreeMap::from([("wSpriteAnimCount".to_string(), 255)]);
        let mut regions = BTreeMap::new();
        let mut resolve = |_: &str| anyhow::bail!("this source RAM operation reads no resource");
        runtime_intro_apply_region_effect(&mut regions,
            &operation(json!({"op":"fill_memory","target":"wSpriteAnimData","byte_count":193,"value":0})),
            &mut memory, &mut resolve).unwrap();
        assert_eq!(regions["wSpriteAnimDict"], vec![0; 20]);
        assert_eq!(regions["wSpriteAnimationStructs"], vec![0; 160]);
        assert_eq!(memory["wSpriteAnimCount"], 0);
        let slots = regions.get_mut("wSpriteAnimationStructs").unwrap();
        slots[0] = 7;
        slots[32] = 9;
        slots[1] = 99;
        memory.insert("wSpriteAnimCount".to_string(), 254);
        runtime_intro_apply_region_effect(&mut regions,
            &operation(json!({"op":"sprite_init_group","source_sprite_init_timing":{"routine":"CrystalIntro_InitUnownAnim"}})),
            &mut memory, &mut resolve).unwrap();
        let indices: Vec<_> = regions["wSpriteAnimationStructs"].chunks_exact(16).map(|slot| slot[0]).collect();
        assert_eq!(indices, vec![7,255,9,1,2,3,0,0,0,0]);
        assert_eq!(memory["wSpriteAnimCount"], 3);
        runtime_intro_apply_region_effect(&mut regions,
            &operation(json!({"op":"deinitialize_all_sprites"})), &mut memory, &mut resolve).unwrap();
        assert!(regions["wSpriteAnimationStructs"].chunks_exact(16).all(|slot| slot[0] == 0));
        assert_eq!(regions["wSpriteAnimationStructs"][1], 99);
        assert_eq!(memory["wSpriteAnimCount"], 3);
    }

    #[test]
    fn source_strided_tilemap_copy_and_conditional_xor_preserve_guarded_bytes() {
        let mut memory = BTreeMap::from([("wIntroSceneFrameCounter".to_string(), 0)]);
        let mut regions = BTreeMap::from([("scratch".to_string(), vec![0,1,127,88,128,255,9,77])]);
        let mut resolve = |_: &str| anyhow::bail!("copy reads source scratch RAM");
        runtime_intro_apply_region_effect(&mut regions,
            &operation(json!({"op":"copy_strided_memory","source":"scratch","target":"wTilemap","rows":2,"row_bytes":3,"source_stride":4,"target_stride":3})),
            &mut memory, &mut resolve).unwrap();
        assert_eq!(regions["wTilemap"], vec![0,1,127,128,255,9]);
        let xor = operation(json!({"op":"conditional_tilemap_xor","target":"wTilemap","xor_mask":8,"clock":"wIntroSceneFrameCounter","clock_mask":3,"nonzero_only":true,"unsigned_less_than":128,"prepare_phase":{"equals":3,"write":{"target":"hBGMapMode","value":0}},"swap_phase":{"equals":0,"completion_write":{"target":"hBGMapMode","value":1}}}));
        runtime_intro_apply_region_effect(&mut regions, &xor, &mut memory, &mut resolve).unwrap();
        assert_eq!(regions["wTilemap"], vec![0,9,119,128,255,1]);
        assert_eq!(memory["hBGMapMode"], 1);
        memory.insert("wIntroSceneFrameCounter".to_string(), 3);
        runtime_intro_apply_region_effect(&mut regions, &xor, &mut memory, &mut resolve).unwrap();
        assert_eq!(regions["wTilemap"], vec![0,9,119,128,255,1]);
        assert_eq!(memory["hBGMapMode"], 0);
    }
}
