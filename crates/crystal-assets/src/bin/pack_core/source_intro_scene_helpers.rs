//! Source-certified presentation effects used by the Crystal intro scenes.
use super::super::super::source_syntax::{code_part, source_arguments};
use super::{Source, resource};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn table<'a>(source: &'a Source, label: &str) -> Result<&'a [(u32, String)]> {
    let start = source
        .lines
        .iter()
        .position(|(_, line)| line.trim_end_matches(':') == label)
        .context("intro palette label")?
        + 1;
    let end = source
        .lines
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, (_, line))| line.ends_with(':'))
        .map_or(source.lines.len(), |(index, _)| index);
    Ok(&source.lines[start..end])
}

fn expand_rgb(lines: &[String], variables: &mut BTreeMap<String, i64>) -> Result<Vec<[u8; 3]>> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        if let Some(args) = line.strip_prefix("RGB ") {
            let args = source_arguments(args)?;
            ensure!(args.len() == 3, "intro RGB component count");
            let mut color = [0; 3];
            for (slot, arg) in color.iter_mut().zip(args) {
                let value = super::super::super::source_expressions::evaluate(&arg, variables)?;
                ensure!((0..32).contains(&value), "intro RGB channel out of bounds");
                *slot = value as u8;
            }
            result.push(color);
        } else if let Some(raw) = line.strip_prefix("DEF ") {
            let (name, expression) = raw
                .split_once(" = ")
                .or_else(|| raw.split_once(" -= "))
                .context("intro palette variable definition")?;
            let value = super::super::super::source_expressions::evaluate(expression, variables)?;
            let value = if raw.contains(" -= ") {
                variables
                    .get(name)
                    .context("intro undefined palette variable")?
                    - value
            } else {
                value
            };
            variables.insert(name.into(), value);
        } else if line.starts_with("for ") || line.starts_with("rept ") {
            let mut depth = 1;
            let mut end = index + 1;
            while end < lines.len() {
                if lines[end].starts_with("for ") || lines[end].starts_with("rept ") {
                    depth += 1;
                }
                if lines[end] == "endr" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                end += 1;
            }
            ensure!(depth == 0, "unterminated intro RGB loop");
            let (variable, count) = if let Some(raw) = line.strip_prefix("for ") {
                let args = source_arguments(raw)?;
                ensure!(args.len() == 2, "intro RGB for loop arity");
                (
                    Some(args[0].clone()),
                    super::super::super::source_expressions::evaluate(&args[1], variables)?,
                )
            } else {
                (
                    None,
                    super::super::super::source_expressions::evaluate(
                        line.strip_prefix("rept ").unwrap(),
                        variables,
                    )?,
                )
            };
            ensure!((1..=32).contains(&count), "intro RGB loop count");
            for iteration in 0..count {
                if let Some(variable) = &variable {
                    variables.insert(variable.clone(), iteration);
                }
                result.extend(expand_rgb(&lines[index + 1..end], variables)?);
            }
            index = end;
        } else {
            anyhow::bail!("unlowered intro palette directive {line}");
        }
        index += 1;
    }
    Ok(result)
}

fn colors(source: &Source, label: &str) -> Result<Vec<[u8; 3]>> {
    expand_rgb(
        &table(source, label)?
            .iter()
            .map(|(_, line)| line.clone())
            .collect::<Vec<_>>(),
        &mut BTreeMap::new(),
    )
}

fn palette_file(vendor: &Path, path: &str) -> Result<Vec<[u8; 3]>> {
    let lines = std::fs::read_to_string(vendor.join(path))?
        .lines()
        .map(code_part)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    expand_rgb(&lines, &mut BTreeMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_palette_loops_preserve_authored_fast_and_slow_gradients() {
        let lines = [
            "DEF hue = 31",
            "rept 8",
            "RGB hue, hue, hue",
            "DEF hue -= 1",
            "RGB hue, hue, hue",
            "DEF hue -= 2",
            "endr",
        ]
        .map(str::to_owned);
        let colors = expand_rgb(&lines, &mut BTreeMap::new()).unwrap();
        assert_eq!(
            colors.iter().map(|c| c[0]).collect::<Vec<_>>(),
            vec![
                31, 30, 28, 27, 25, 24, 22, 21, 19, 18, 16, 15, 13, 12, 10, 9
            ]
        );
        assert!(colors.iter().all(|c| c[0] == c[1] && c[1] == c[2]));
        let lines = ["for hue, 32", "RGB 0, hue / 2, hue", "endr"].map(str::to_owned);
        let colors = expand_rgb(&lines, &mut BTreeMap::new()).unwrap();
        assert_eq!(colors[0], [0, 0, 0]);
        assert_eq!(colors[31], [0, 15, 31]);
    }

    #[test]
    fn pinned_intro_helpers_lower_all_seven_effects_and_source_palettes() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = Source::read(vendor, "engine/movie/intro.asm").unwrap();
        for (label, op) in [
            ("Intro_PerspectiveScrollBG", "perspective_scroll"),
            ("Intro_RustleGrass", "indexed_2bpp_request"),
            ("CrystalIntro_UnownFade", "palette_fade_lookup"),
            ("Intro_FadeUnownWordPals", "fade_unown_word_palettes"),
            (
                "Intro_Scene24_ApplyPaletteFade",
                "broadcast_indexed_palette",
            ),
            ("Intro_Scene16_AnimateSuicune", "conditional_tilemap_xor"),
            ("Intro_Scene20_AppearUnown", "copy_indexed_palette"),
        ] {
            let (actual, data) = fields(vendor, &source, label).unwrap();
            assert_eq!(actual, op);
            assert_eq!(
                data["implementation_source_span"]["file"],
                "engine/movie/intro.asm"
            );
        }
        let (_, fade) = fields(vendor, &source, "CrystalIntro_UnownFade").unwrap();
        for table in fade["tables"].as_array().unwrap() {
            assert_eq!(table["colors"].as_array().unwrap().len(), 32);
        }
        let (_, word) = fields(vendor, &source, "Intro_FadeUnownWordPals").unwrap();
        assert_eq!(
            word["fast_hues"],
            json!([
                31, 30, 28, 27, 25, 24, 22, 21, 19, 18, 16, 15, 13, 12, 10, 9
            ])
        );
        assert_eq!(word["slow_hues"], json!((16..32).rev().collect::<Vec<_>>()));
    }
}

pub(super) fn fields(
    vendor: &Path,
    source: &Source,
    helper: &str,
) -> Result<(&'static str, Value)> {
    let result = match helper {
        "Intro_PerspectiveScrollBG" => {
            source.operation(
                helper,
                &["ld a, [wIntroSceneFrameCounter]", "and $1", "jr z, .skip"],
                "source_scroll_cadence",
                json!({}),
            )?;
            source.operation(
                helper,
                &[
                    "ld hl, wLYOverrides",
                    "ld a, [hl]",
                    "inc a",
                    "ld bc, $5f",
                    "call ByteFill",
                ],
                "source_scroll_back",
                json!({}),
            )?;
            source.operation(
                helper,
                &[
                    "ld hl, wLYOverrides + $5f",
                    "ld a, [hl]",
                    "inc a",
                    "inc a",
                    "ld bc, $31",
                    "call ByteFill",
                ],
                "source_scroll_front",
                json!({}),
            )?;
            (
                "perspective_scroll",
                json!({"target":"wLYOverrides","byte_count":144,"frame":{"source":"wIntroSceneFrameCounter","parity_mask":1},"bands":[{"offset":0,"byte_count":95,"delta":1,"cadence":"odd_frames","value_source":"first_byte"},{"offset":95,"byte_count":49,"delta":2,"cadence":"every_frame","value_source":"first_byte"}],"horizontal_scroll":{"source_offset":0,"target":"hSCX"}}),
            )
        }
        "Intro_RustleGrass" => {
            source.operation(
                helper,
                &[
                    "ld a, [wIntroSceneFrameCounter]",
                    "cp 36",
                    "ret nc",
                    "and $c",
                    "srl a",
                ],
                "source_rustle_selector",
                json!({}),
            )?;
            source.operation(
                helper,
                &[
                    "ld a, LOW(vTiles2 tile $09)",
                    "ld [wRequested2bppDest], a",
                    "ld a, HIGH(vTiles2 tile $09)",
                    "ld [wRequested2bppDest + 1], a",
                    "ld a, 4",
                    "ld [wRequested2bppSize], a",
                ],
                "source_rustle_destination",
                json!({}),
            )?;
            let pointers = table(source, ".RustlingGrassPointers")?
                .iter()
                .filter_map(|(_, line)| line.strip_prefix("dw "))
                .collect::<Vec<_>>();
            ensure!(pointers.len() == 4, "intro grass pointer count");
            let entries = pointers
                .iter()
                .map(|symbol| Ok(json!({"path":resource(source,symbol)?})))
                .collect::<Result<Vec<_>>>()?;
            (
                "indexed_2bpp_request",
                json!({"condition":{"source":"wIntroSceneFrameCounter","predicate":"unsigned_less_than","operand":36},"selector":{"source":"wIntroSceneFrameCounter","mask":12,"shift_right":1,"byte_offsets":[0,2,4,6]},"table":{"entries":entries},"tile_count":4,"bytes_per_tile":16,"target_byte_offset":144,"target_vram_bank":0}),
            )
        }
        "CrystalIntro_UnownFade" => {
            source.operation(
                helper,
                &[
                    "add a",
                    "add a",
                    "add a",
                    "ld e, a",
                    "ld d, 0",
                    "ld hl, wBGPals2",
                    "add hl, de",
                    "inc hl",
                    "inc hl",
                ],
                "source_unown_palette_slot",
                json!({}),
            )?;
            source.operation(
                helper,
                &[
                    "ld a, [wIntroSceneTimer]",
                    "and %111111",
                    "cp %011111",
                    "jr z, .okay",
                    "jr c, .okay",
                    "ld c, a",
                    "ld a, %111111",
                    "sub c",
                ],
                "source_unown_timer",
                json!({}),
            )?;
            let names = [".BWFade", ".BlackLBlueFade", ".BlackBlueFade"];
            let tables = names
                .iter()
                .map(|label| {
                    let rgb = colors(source, label)?;
                    ensure!(rgb.len() == 32, "source Unown fade color count");
                    let words = rgb
                        .iter()
                        .map(|c| u16::from(c[0]) | (u16::from(c[1]) << 5) | (u16::from(c[2]) << 10))
                        .collect::<Vec<_>>();
                    Ok(json!({"label":label,"colors":words}))
                })
                .collect::<Result<Vec<_>>>()?;
            let writes=names.iter().enumerate().map(|(i,label)|json!({"table":label,"target_offset":2+i*2,"encoding":"rgb555_little_endian"})).collect::<Vec<_>>();
            (
                "palette_fade_lookup",
                json!({"palette_selector":"accumulator","selector_stride":8,"first_color_offset":2,"timer":{"source":"wIntroSceneTimer","mask":63,"fold_above":31,"fold_from":63},"tables":tables,"writes":writes}),
            )
        }
        "Intro_FadeUnownWordPals" => {
            source.operation(
                helper,
                &[
                    "add a",
                    "add a",
                    "add a",
                    "ld e, a",
                    "ld d, 0",
                    "ld hl, wBGPals2",
                    "add hl, de",
                    "rept 4",
                    "inc hl",
                    "endr",
                    "ld a, [wIntroSceneTimer]",
                    "add a",
                    "ld c, a",
                    "ld b, 0",
                ],
                "source_word_palette_slot",
                json!({}),
            )?;
            let fast = colors(source, ".FastFadePalettes")?;
            let slow = colors(source, ".SlowFadePalettes")?;
            ensure!(
                fast.len() == 16 && slow.len() == 16,
                "source word fade table counts"
            );
            (
                "fade_unown_word_palettes",
                json!({"palette_index":{"source":"accumulator","multiply":8,"valid_values":(0..8).collect::<Vec<_>>()},"fade_index":{"source":"wIntroSceneTimer","multiply":2,"valid_values":(0..16).collect::<Vec<_>>()},"target_color_offsets":[4,6],"color_encoding":"rgb555_grayscale","fast_hues":fast.iter().map(|c|c[0]).collect::<Vec<_>>(),"slow_hues":slow.iter().map(|c|c[0]).collect::<Vec<_>>()}),
            )
        }
        "Intro_Scene24_ApplyPaletteFade" => {
            source.operation(
                helper,
                &["ld de, wBGPals2", "ld b, 8"],
                "source_broadcast_destination",
                json!({}),
            )?;
            source.operation(
                helper,
                &["ld c, 1 palettes"],
                "source_broadcast_width",
                json!({}),
            )?;
            let path = resource(source, ".FadePals")?;
            let colors = palette_file(vendor, &path)?;
            ensure!(colors.len() == 32, "source fade broadcast palette count");
            let palettes = colors
                .chunks_exact(4)
                .map(|p| p.to_vec())
                .collect::<Vec<_>>();
            (
                "broadcast_indexed_palette",
                json!({"source":path,"source_offset":{"source":"accumulator","valid_values":(0..8).map(|i|i*8).collect::<Vec<_>>()},"bytes_per_palette":8,"destination":"wBGPals2","destination_palette_count":8,"behavior":"repeat_selected_palette","palettes":palettes}),
            )
        }
        "Intro_Scene16_AnimateSuicune" => {
            source.operation("Intro_ColoredSuicuneFrameSwap", &["hlcoord 0, 0", "ld bc, SCREEN_AREA"], "source_suicune_tile_region", json!({}))?;
            source.operation("Intro_ColoredSuicuneFrameSwap", &["inc hl", "dec bc", "ld a, c", "or b", "jr nz, .loop"], "source_suicune_tile_loop", json!({}))?;
            source.operation("Intro_ColoredSuicuneFrameSwap", &["ld a, [hl]", "and a", "jr z, .skip", "cp $80", "jr nc, .skip", "xor $8", "ld [hl], a"], "source_suicune_tile_toggle", json!({}))?;
            source.operation("Intro_ColoredSuicuneFrameSwap", &["ld a, $1", "ldh [hBGMapMode], a", "ret"], "source_suicune_swap_complete", json!({}))?;
            source.operation(
                helper,
                &[
                    "ld a, [wIntroSceneFrameCounter]",
                    "and $3",
                    "jr z, Intro_ColoredSuicuneFrameSwap",
                    "cp $3",
                    "jr z, .PrepareForSuicuneSwap",
                    "ret",
                ],
                "source_suicune_swap",
                json!({}),
            )?;
            (
                "conditional_tilemap_xor",
                json!({"clock":"wIntroSceneFrameCounter","clock_mask":3,"prepare_phase":{"equals":3,"write":{"target":"hBGMapMode","value":0}},"swap_phase":{"equals":0,"completion_write":{"target":"hBGMapMode","value":1}},"target":"wTilemap","xor_mask":8,"nonzero_only":true,"unsigned_less_than":128}),
            )
        }
        "Intro_Scene20_AppearUnown" => {
            source.operation(
                helper,
                &[
                    "and a",
                    "jr nz, .load_pal_2",
                    "ld hl, .pal1",
                    "jr .got_pointer",
                ],
                "source_appear_palette_variant",
                json!({}),
            )?;
            source.operation(
                helper,
                &[
                    "ld a, [wIntroSceneTimer]",
                    "and $7",
                    "add a",
                    "add a",
                    "add a",
                    "ld c, a",
                ],
                "source_appear_palette_index",
                json!({}),
            )?;
            source.operation(
                helper,
                &[
                    "ld bc, 1 palettes",
                    "call CopyBytes",
                    "pop bc",
                    "ld de, wBGPals1",
                ],
                "source_appear_copy_width",
                json!({}),
            )?;
            let paths = [resource(source, ".pal1")?, resource(source, ".pal2")?];
            let palettes = paths
                .iter()
                .map(|path| Ok(json!({"path":path,"colors":palette_file(vendor,path)?})))
                .collect::<Result<Vec<_>>>()?;
            (
                "copy_indexed_palette",
                json!({"palette_argument":{"source":"accumulator","variants":[{"value":0,"resource":paths[0]},{"predicate":"nonzero","resource":paths[1]}]},"selector":{"source":"wIntroSceneTimer","mask":7,"byte_scale":8},"palettes":palettes,"destinations":["wBGPals2","wBGPals1"],"bytes_per_palette":8}),
            )
        }
        _ => anyhow::bail!("unlowered intro scene helper {helper}"),
    };
    let mut fields = result.1.as_object().unwrap().clone();
    fields.insert("implementation_source_span".into(), source.span(helper)?);
    Ok((result.0, Value::Object(fields)))
}
