//! Prepared Minnesota foods can be fed from the Pack or eaten when held.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::models::item::Item;

use crate::CompiledGamePack;

const FAIR: &str = "MnStateFairgrounds";
const BAKERY: &str = "MART_MN_SPX_WESTSEVENTHBAKERY_HOST";
const KITCHEN: &str = "MART_MN_DULUTH_X_COMMUNITYKITCHEN_F1_HOST";
const PIE: &str = "MN_BLUEBERRY_SLICE";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let vendors = &pack
        .data
        .maps
        .get(FAIR)
        .context("Fairgrounds")?
        .script_shop_commands;
    // The installed Fair vendors own the specialty menu. Check every sold portion
    // without creating a second catalog of the historic fair foods.
    let mut specialties = BTreeSet::new();
    for vendor in vendors {
        let inventory = pack.data.marts.inventory_ids(&vendor.mart_id)?;
        specialties.extend(
            inventory
                .iter()
                .filter(|id| id.starts_with("FAIR_"))
                .map(String::as_str),
        );
    }
    ensure!(
        specialties.len() == 16,
        "expected all sixteen Fair specialties"
    );
    for id in specialties.into_iter().chain([PIE]) {
        let item = pack
            .data
            .items
            .get_mut(id)
            .with_context(|| format!("prepared food {id}"))?;
        ensure!(
            item.held_effect == "HELD_NONE"
                && item.parameter > 0
                && item.field_usable
                && item.battle_usable
                && item.consumable
                && item.pocket == "ITEM"
                && item.field_menu == "ITEMMENU_PARTY"
                && item.battle_menu == "ITEMMENU_PARTY",
            "expected a portion of restorative food for {id}"
        );
        item.held_effect = "HELD_BERRY".into();
        if id == "FAIR_BEER" {
            ensure!(
                item.effect == "RESTORE_HP_CONFUSE"
                    && item.confusion_inflict_turns == Some(4)
                    && item.parameter == 100,
                "expected the Fair beer's confusion effect"
            );
            item.description = "100 HP (use/hold).\nCauses confusion.".into();
        } else {
            ensure!(
                item.effect == "RestoreHPEffect",
                "expected HP food for {id}"
            );
            let (flavor, _) = item
                .description
                .split_once('\n')
                .context("food description")?;
            item.description = food_description(flavor, item.parameter);
        }
    }

    let portion = pack
        .data
        .items
        .get(PIE)
        .context("local healing food")?
        .clone();
    for (id, name, price, hp, flavor) in [
        (
            "MN_BUTTER_ROLL",
            "BUTTER ROLL",
            200,
            20,
            "A warm butter roll",
        ),
        (
            "MN_CARDAMOM_BUN",
            "CARDAMOM BUN",
            350,
            50,
            "A spiced sweet bun",
        ),
        ("MN_WILD_RICE_SOUP", "RICE SOUP", 500, 60, "Wild rice soup."),
    ] {
        let item = Item {
            name: name.into(),
            script_name: id.into(),
            description: food_description(flavor, hp),
            price,
            parameter: hp,
            ..portion.clone()
        };
        ensure!(
            pack.data.items.insert(id.into(), item).is_none(),
            "occupied food ID {id}"
        );
    }
    for (mart, originals, specials) in [
        (
            BAKERY,
            ["BERRY_JUICE", "BERRY", "PRZCUREBERRY"],
            &["MN_BUTTER_ROLL", "MN_CARDAMOM_BUN"][..],
        ),
        (
            KITCHEN,
            ["FRESH_WATER", "SODA_POP", "LEMONADE"],
            &["MN_WILD_RICE_SOUP"][..],
        ),
    ] {
        let stock = pack
            .data
            .marts
            .0
            .get_mut(mart)
            .context("local food counter")?;
        ensure!(
            stock.iter().map(String::as_str).eq(originals),
            "unexpected stock at {mart}"
        );
        stock.splice(0..0, specials.iter().map(|id| (*id).to_owned()));
    }
    Ok(())
}

fn food_description(flavor: &str, hp: i16) -> String {
    format!("{flavor}\n{hp} HP (use/hold).")
}
