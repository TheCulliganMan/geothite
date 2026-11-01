//! Compile original species overlays from a supplied verified local pack.
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result, ensure};
use crystal_assets::MinnesotaPokemonCatalog;

fn main() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    let usage = "usage: pack_minnesota_pokemon BASE_PACK POKEMON_ROOT NATIVE_ROOT PROFILES_JSON_OR_DIR OUTPUT_PACK [--family MN_F001] [--families-dir OUTPUT_DIRECTORY [--jobs N]]";
    let base_path = PathBuf::from(arguments.next().context(usage)?).canonicalize()?;
    let pokemon_root = PathBuf::from(arguments.next().context(usage)?);
    let native_root = PathBuf::from(arguments.next().context(usage)?);
    let profiles = PathBuf::from(arguments.next().context(usage)?);
    let output = PathBuf::from(arguments.next().context(usage)?);
    let mut family = None;
    let mut families_directory = None;
    let mut jobs = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--family" => {
                ensure!(family.is_none(), "duplicate --family");
                family = Some(arguments.next().context("missing family ID")?);
            }
            "--families-dir" => {
                ensure!(families_directory.is_none(), "duplicate --families-dir");
                families_directory = Some(PathBuf::from(
                    arguments
                        .next()
                        .context("missing family output directory")?,
                ));
            }
            "--jobs" => {
                ensure!(jobs.is_none(), "duplicate --jobs");
                jobs = Some(
                    arguments
                        .next()
                        .context("missing worker count")?
                        .parse::<usize>()
                        .context("parse family worker count")?,
                );
            }
            _ => anyhow::bail!("unknown argument {argument}; {usage}"),
        }
    }
    ensure!(
        family.is_none() || families_directory.is_none(),
        "choose --family or --families-dir"
    );
    ensure!(
        jobs.is_none() || families_directory.is_some(),
        "--jobs requires --families-dir"
    );
    let jobs = jobs.unwrap_or(1);
    ensure!((1..=8).contains(&jobs), "family worker count must be 1..8");
    ensure_ignored_output(&output, &base_path)?;
    let base = crystal_assets::read_verified_compiled_game_pack(&base_path)?;
    let catalog = MinnesotaPokemonCatalog::load(&pokemon_root, &native_root, &profiles)?;
    if let Some(directory) = &families_directory {
        for id in catalog.family_ids() {
            let path = directory.join(format!("{}.crystalpack", id.to_ascii_lowercase()));
            ensure_ignored_output(&path, &base_path)?;
            ensure!(
                path.parent()
                    .context("missing family output parent")?
                    .canonicalize()?
                    .join(path.file_name().context("missing family output filename")?)
                    != output
                        .parent()
                        .unwrap_or(Path::new("."))
                        .canonicalize()?
                        .join(output.file_name().context("missing output filename")?),
                "aggregate and family output paths overlap"
            );
        }
    }
    let aggregate = catalog.build(&base, family.as_deref())?;
    write_and_check(&aggregate, &output)?;
    if let Some(directory) = families_directory {
        write_families(&catalog, &base, &directory, jobs)?;
    }
    Ok(())
}

fn write_families(
    catalog: &MinnesotaPokemonCatalog,
    base: &crystal_assets::CompiledGamePack,
    directory: &Path,
    jobs: usize,
) -> Result<()> {
    let families = catalog.family_ids();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..jobs.min(families.len()) {
            workers.push(scope.spawn(|| -> Result<()> {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(id) = families.get(index) else {
                        return Ok(());
                    };
                    let path = directory.join(format!("{}.crystalpack", id.to_ascii_lowercase()));
                    let pack = catalog.build(base, Some(id))?;
                    write_and_check(&pack, &path)?;
                }
            }));
        }
        // Join every worker before propagating any error. Each family has a
        // unique preflight-validated output and shares only immutable inputs.
        let results: Vec<_> = workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| anyhow::anyhow!("family publisher worker panicked"))?
            })
            .collect();
        for result in results {
            result?;
        }
        Ok(())
    })
}

fn ensure_ignored_output(path: &Path, base: &Path) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let resolved = parent
        .canonicalize()?
        .join(path.file_name().context("missing output filename")?);
    ensure!(
        resolved != base && (!resolved.exists() || resolved.canonicalize()? != base),
        "output must differ from immutable source pack"
    );
    let tracked = std::process::Command::new("git")
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(&resolved)
        .output()
        .context("check output is not tracked by Git")?;
    ensure!(
        !tracked.status.success(),
        "output is tracked by Git: {}",
        resolved.display()
    );
    let ignored = std::process::Command::new("git")
        .args(["check-ignore", "--quiet", "--no-index"])
        .arg(&resolved)
        .status()
        .context("check pack output is ignored by Git")?;
    ensure!(
        ignored.success(),
        "pack output must be ignored by Git: {}",
        resolved.display()
    );
    Ok(())
}

fn write_and_check(pack: &crystal_assets::CompiledGamePack, output: &Path) -> Result<()> {
    pack.write_preserving_storage(output)?;
    let restored = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(
        restored.identity()? == pack.identity()?,
        "written species pack identity mismatch"
    );
    println!(
        "exported {}: {} species",
        output.display(),
        restored.data().pokemon.len()
    );
    Ok(())
}
