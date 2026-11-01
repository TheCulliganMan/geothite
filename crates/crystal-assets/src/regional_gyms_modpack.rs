//! Compose the reviewed gym generation and the three remaining native cities.
use crate::CompiledGamePack;
use anyhow::Result;

/// Borrowed authored sources captured before regional construction begins.
pub struct RegionalGymSources<'a> {
    /// Complete canonical eight-gym collection.
    pub gyms: &'a str,
    /// Mankato town and layout, in that order.
    pub mankato: [&'a str; 2],
    /// St Paul town, layout and public depot, in that order.
    pub st_paul: [&'a str; 3],
    /// St Cloud town, layout and public station, in that order.
    pub st_cloud: [&'a str; 3],
}

/// Refreshes all obsolete gyms in the story lineage and adds the native cities.
///
/// Rochester, Minneapolis and the northern chapters remain installed. Mankato,
/// St Paul and St Cloud use their checked production builders, including the
/// R04/R05/R06 seams. Every gym then has a real city facade in the same pack.
/// This does not invent the still-missing regional links or claim an earned
/// continuous journey. No player state, badge or quest progress is granted.
///
/// # Errors
/// Returns an error from the exact-generation gym refresh, any native city
/// builder or final verification. The borrowed packs and source strings remain
/// unchanged, and no partial output is published by this function.
pub fn build_regional_gyms_modpack(
    base: &CompiledGamePack,
    reviewed: &CompiledGamePack,
    source: &RegionalGymSources<'_>,
) -> Result<CompiledGamePack> {
    let mut pack = crate::refresh_installed_minnesota_gyms(base, reviewed, source.gyms)?;
    pack = crate::build_mankato_modpack(&pack, source.mankato[0], source.mankato[1], source.gyms)?;
    pack = crate::build_st_paul_modpack(
        &pack,
        source.st_paul[0],
        source.st_paul[1],
        source.st_paul[2],
        source.gyms,
    )?;
    pack = crate::build_st_cloud_modpack(
        &pack,
        source.st_cloud[0],
        source.st_cloud[1],
        source.st_cloud[2],
        source.gyms,
    )?;
    Ok(pack)
}
