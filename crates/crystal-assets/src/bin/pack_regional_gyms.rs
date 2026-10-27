use anyhow::{Context, Result, ensure};
use crystal_assets::{
    RegionalGymSources, build_regional_gyms_modpack, read_verified_compiled_game_pack,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 4,
        "usage: pack_regional_gyms STORY_BASE REVIEWED_GYMS PLANNING_ROOT OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[3].canonicalize().unwrap_or_else(|_| p[3].clone()),
        "output must differ from base"
    );
    let sources = [
        "gyms/interiors.json",
        "towns/MnMankato.json",
        "towns/MnMankato.layout.json",
        "towns/MnStPaul.json",
        "towns/MnStPaul.layout.json",
        "transit/MnStPaulUnionDepot.json",
        "towns/MnStCloud.json",
        "towns/MnStCloud.layout.json",
        "transit/MnStCloudStation.json",
    ]
    .iter()
    .map(|name| {
        std::fs::read_to_string(p[2].join(name))
            .with_context(|| format!("authored regional source {name}"))
    })
    .collect::<Result<Vec<_>>>()?;
    let pack = build_regional_gyms_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &read_verified_compiled_game_pack(&p[1])?,
        &RegionalGymSources {
            gyms: &sources[0],
            mankato: [&sources[1], &sources[2]],
            st_paul: [&sources[3], &sources[4], &sources[5]],
            st_cloud: [&sources[6], &sources[7], &sources[8]],
        },
    )?;
    pack.write_preserving_storage(&p[3])?;
    let checked =
        read_verified_compiled_game_pack(&p[3]).context("written regional gym candidate")?;
    println!(
        "published all eight current gyms with their city facades: {} maps in {}",
        checked.data().maps.len(),
        p[3].display()
    );
    Ok(())
}
