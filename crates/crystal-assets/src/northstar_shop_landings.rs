//! Put arriving shoppers on the open floor immediately inside each street door.
use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::refresh_event_section;
use crate::{CompiledGamePack, authored_town_publication};

pub(super) const DEPARTMENT: &str = "MnMinneapolisDepartmentStore1F";

const SHOPS: &[&str] = &[
    "MnDuluthHillSupplies",
    "MnDuluthLakeOutfitter",
    "MnDuluthToolExchange",
    "MnDuluthMaritimeCoop1F",
    "MnDuluthDockSupplies",
    "MnDuluthPointOutfitter",
    "MnMinneapolisSupplyCoop",
    "MnMinneapolisGardenShop",
    DEPARTMENT,
    "MnMinneapolisSouthMarket",
    "MnStPaulFairFoodHall",
    "MnStPaulFairGardenStand",
    "MnStPaulFairBerrySnack",
    "MnStPaulFairFestivalMail",
    "MnStPaulWestSeventhBakery",
    "MnStPaulLowertownExchange",
    "MnStPaulLowertownCrafts",
    "MnStPaulLowertownCafe",
    "MnStPaulRiversideSupplies",
];

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for &id in SHOPS {
        let mut room = pack
            .data
            .maps
            .get(id)
            .context("regional storefront")?
            .clone();
        let [left, right, arrival, ..] = room.events.warps.as_slice() else {
            anyhow::bail!("missing entrance carpets in {id}");
        };
        ensure!(
            left.y == right.y
                && left.x + 1 == right.x
                && left.target_map_constant == right.target_map_constant
                && Some(&arrival.target_map_constant) == room.attributes.map_constant.as_ref()
                && arrival.index == 3
                && arrival.target_warp_id == 3,
            "unexpected street entrance in {id}"
        );
        let x = if id == DEPARTMENT { left.x } else { arrival.x };
        let y = left.y.checked_sub(1).context("shop entry row")?;
        ensure!(
            (x == left.x || x == right.x)
                && !room.objects.iter().any(|o| (o.x, o.y) == (x, y))
                && !room.events.warps.iter().any(|w| (w.x, w.y) == (x, y)),
            "occupied entrance in {id}"
        );
        let block = room.blocks[usize::from(y / 2 * room.attributes.width + x / 2)];
        ensure!(
            pack.data.tilesets[&room.attributes.tileset_name].collision[&format!("{block:02x}")]
                [usize::from((y % 2) * 2 + x % 2)]
                == "FLOOR",
            "store entrance must be ordinary floor in {id}"
        );
        room.events.warps[2].x = x;
        room.events.warps[2].y = y;
        refresh_event_section(&mut room);
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
