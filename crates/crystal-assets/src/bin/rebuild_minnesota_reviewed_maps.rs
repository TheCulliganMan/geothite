//! Reproduce the reviewed map continuation from its coastal parent in one run.
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledGamePack;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const RECIPE_SHA256: &str = "e00da9d5f66546314ad9cb203dd72fb0ce2dbff9a6ea6494a4cf1a4006a46ed7";
const RECIPE_V2_SHA256: &str = "da0b34a501451333e655e5f5c3b17beed0c183e121fce094641036acb0d86316";
const RECIPE_V3_SHA256: &str = "6a371d56959c7b76c578d10e8c0d809c7f4c94c3c4be75776655979b15cbea6e";
const RECIPE_V4_SHA256: &str = "34ad209b29faac1a5806f42ec9a0b6f105ac20e978a92350785f5ab37d146a00";
const RECIPE_V5_SHA256: &str = "a2aba4c2ea8fa36ad57299d8bff1a872ac22572332c95ca93a4ba6ad7887082e";
const RECIPE_V6_SHA256: &str = "c438671a06dc8625e9603bdc238be7cc4963c2d06e7b9a4bee968dc976822c41";
const RECIPE_V7_SHA256: &str = "cedc824090bfd3851a2315cca0e38f74a765593ba689f77393f14878f46b0664";
const RECIPE_V8_SHA256: &str = "c2d2115413ebebb74e6b45d146fe4575267ec97b0342a6e7af001b8b68fbb0d4";
const RECIPE_V9_SHA256: &str = "e1d8b228d9e4850b2ae009f0f7140fcc5ea669fb179c3c9b2b46c9eb49911b1e";

const RECIPE_V10_SHA256: &str = "5c8c1c0da2750306133b917b8f6601f522b68b59419a9c12bfe366f20453bcef";

const RECIPE_V11_SHA256: &str = "5563e695bcada8127e41e4815cdb923e1029b7738b5f4cddad8aeb0789ffbd4b";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    schema_version: u8,
    id: String,
    status: String,
    parent_pack: String,
    parent_pack_sha256: String,
    parent_source_index: Source,
    stages: Vec<Stage>,
    expected_pack_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    path: PathBuf,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    BemidjiRoofPlateau,
    InlandWinterBelt,
    IronRockClearance,
    TamarackSnowline,
    IronDescentSnowline,
    DescentRockClearance,
    InlandGrassCut,
    HeadwatersSnowline,
    HeadwatersRockClearance,
    ItascaSnowline,
    ElyPineCamp,
    InnerBasinBedrock,
    HaulageRockClearance,
    SafariExpedition,
    RochesterCareGreen,
    GrandRapidsDepotQuay,
    MankatoTerracePond,
    BrainerdLakesideLoop,
    R16SeedBank,
    R02WorkingFarmyard,
    R05WoodedTerraces,
    R06PineFork,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Stage {
    kind: Kind,
    sources: Vec<Source>,
    expected_content_hash: String,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_source(root: &Path, source: &Source) -> Result<String> {
    ensure!(
        source
            .path
            .components()
            .all(|c| matches!(c, Component::Normal(_))),
        "source must be relative to the repository"
    );
    let path = root.join(&source.path);
    let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    ensure!(
        hash(&bytes) == source.sha256,
        "source drift: {}",
        path.display()
    );
    String::from_utf8(bytes).with_context(|| format!("UTF-8 source {}", path.display()))
}

fn apply(base: &CompiledGamePack, stage: &Stage, sources: &[String]) -> Result<CompiledGamePack> {
    // Only existing source-bound Rust builders run here. Their complete parent,
    // collision and residual guards remain authoritative for each stage.
    match (&stage.kind, sources) {
        (Kind::BemidjiRoofPlateau, [before, after, contract]) => {
            crystal_assets::rebuild_bemidji_roof_plateau(base, before, after, contract)
        }
        (Kind::InlandWinterBelt, [source]) => {
            crystal_assets::build_inland_winter_belt(base, source)
        }
        (Kind::IronRockClearance, [source]) => {
            crystal_assets::revise_iron_rock_clearance(base, source)
        }
        (Kind::TamarackSnowline, [source]) => crystal_assets::build_tamarack_snowline(base, source),
        (Kind::IronDescentSnowline, [source]) => {
            crystal_assets::build_iron_descent_snowline(base, source)
        }
        (Kind::DescentRockClearance, [source]) => {
            crystal_assets::revise_descent_rock_clearance(base, source)
        }
        (Kind::InlandGrassCut, [source]) => crystal_assets::repair_inland_grass_cut(base, source),
        (Kind::HeadwatersSnowline, [source]) => {
            crystal_assets::build_headwaters_snowline(base, source)
        }
        (Kind::HeadwatersRockClearance, [source]) => {
            crystal_assets::revise_headwaters_rock_clearance(base, source)
        }
        (Kind::ItascaSnowline, [source]) => crystal_assets::build_itasca_snowline(base, source),
        (Kind::ElyPineCamp, [before, after, activities, contract]) => {
            crystal_assets::rebuild_ely_pine_camp(base, before, after, activities, contract)
        }
        (Kind::InnerBasinBedrock, [source]) => {
            crystal_assets::rebuild_inner_basin_bedrock(base, source)
        }
        (Kind::HaulageRockClearance, [source]) => {
            crystal_assets::revise_haulage_rock_clearance(base, source)
        }
        (Kind::SafariExpedition, [source]) => base.with_safari_zone(serde_json::from_str(source)?),
        (Kind::RochesterCareGreen, [source]) => {
            crystal_assets::revise_rochester_care_green(base, source)
        }
        (Kind::GrandRapidsDepotQuay, [source]) => {
            crystal_assets::revise_grand_rapids_depot_quay(base, source)
        }
        (Kind::MankatoTerracePond, [source]) => {
            crystal_assets::revise_mankato_terrace_pond(base, source)
        }
        (Kind::BrainerdLakesideLoop, [source]) => {
            crystal_assets::revise_brainerd_lakeside_loop(base, source)
        }
        (Kind::R16SeedBank, [source]) => crystal_assets::revise_r16_seed_bank(base, source),
        (Kind::R02WorkingFarmyard, [source]) => {
            crystal_assets::revise_r02_working_farmyard(base, source)
        }
        (Kind::R05WoodedTerraces, [source]) => {
            crystal_assets::revise_r05_wooded_terraces(base, source)
        }
        (Kind::R06PineFork, [source]) => crystal_assets::revise_r06_pine_fork(base, source),
        _ => anyhow::bail!("wrong source count for reviewed builder"),
    }
}

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(args.len() == 4, "REPOSITORY_ROOT PARENT_PACK RECIPE OUTPUT");
    let root = PathBuf::from(&args[0]).canonicalize()?;
    let input = PathBuf::from(&args[1]);
    let raw_recipe = std::fs::read(&args[2])?;
    let recipe_hash = hash(&raw_recipe);
    let (recipe_id, stage_count) = if recipe_hash == RECIPE_SHA256 {
        ("minnesota-reviewed-maps-v1", 12)
    } else if recipe_hash == RECIPE_V2_SHA256 {
        ("minnesota-reviewed-maps-v2", 13)
    } else if recipe_hash == RECIPE_V3_SHA256 {
        ("minnesota-reviewed-maps-v3", 14)
    } else if recipe_hash == RECIPE_V4_SHA256 {
        ("minnesota-reviewed-maps-v4", 15)
    } else if recipe_hash == RECIPE_V5_SHA256 {
        ("minnesota-reviewed-maps-v5", 16)
    } else if recipe_hash == RECIPE_V6_SHA256 {
        ("minnesota-reviewed-maps-v6", 17)
    } else if recipe_hash == RECIPE_V7_SHA256 {
        ("minnesota-reviewed-maps-v7", 18)
    } else if recipe_hash == RECIPE_V8_SHA256 {
        ("minnesota-reviewed-maps-v8", 19)
    } else if recipe_hash == RECIPE_V9_SHA256 {
        ("minnesota-reviewed-maps-v9", 20)
    } else if recipe_hash == RECIPE_V10_SHA256 {
        ("minnesota-reviewed-maps-v10", 21)
    } else if recipe_hash == RECIPE_V11_SHA256 {
        ("minnesota-reviewed-maps-v11", 22)
    } else {
        anyhow::bail!("reviewed recipe differs");
    };
    let recipe: Recipe = serde_json::from_slice(&raw_recipe)?;
    ensure!(
        recipe.schema_version == 1
            && recipe.id == recipe_id
            && !recipe.status.is_empty()
            && !recipe.parent_pack.is_empty()
            && recipe.stages.len() == stage_count,
        "complete reviewed recipe required"
    );
    let output = std::path::absolute(&args[3])?;
    ensure!(!output.exists(), "preserve every prior pack");
    ensure!(
        std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "generated game content must remain ignored"
    );
    ensure!(
        hash(&std::fs::read(&input)?) == recipe.parent_pack_sha256,
        "reviewed coastal parent differs"
    );
    // Resolve and hash the complete source closure before any build or write.
    read_source(&root, &recipe.parent_source_index)?;
    let sources = recipe
        .stages
        .iter()
        .map(|stage| {
            stage
                .sources
                .iter()
                .map(|s| read_source(&root, s))
                .collect()
        })
        .collect::<Result<Vec<Vec<String>>>>()?;
    let mut pack = crystal_assets::read_verified_compiled_game_pack(&input)?;
    for (index, (stage, source)) in recipe.stages.iter().zip(&sources).enumerate() {
        pack = apply(&pack, stage, source)
            .with_context(|| format!("reviewed map stage {}", index + 1))?;
        let identity = pack.identity()?;
        ensure!(
            identity.content_hash == stage.expected_content_hash,
            "reviewed map stage {} differs",
            index + 1
        );
        println!(
            "stage {}/{}: {}",
            index + 1,
            stage_count,
            identity.content_hash
        );
    }
    ensure!(
        hash(&std::fs::read(&input)?) == recipe.parent_pack_sha256,
        "parent changed during rebuild"
    );
    ensure!(!output.exists(), "output appeared during rebuild");
    pack.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == pack
            && hash(&std::fs::read(&output)?) == recipe.expected_pack_sha256,
        "strict final readback or reviewed pack bytes differ"
    );
    println!("{}", output.display());
    Ok(())
}
