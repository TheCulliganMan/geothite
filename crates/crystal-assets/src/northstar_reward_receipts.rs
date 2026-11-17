//! Commit successful regional reward claims before presenting their receipts.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

/// Rewrites only the exact grant/failure/claim sequence. Callers must derive a
/// fresh pack identity afterward. Other regions and arbitrary gift scripts are
/// untouched; an unsuccessful grant still takes its original failure branch.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<usize> {
    let names: Vec<_> = pack
        .data
        .maps
        .keys()
        .filter(|name| name.starts_with("Mn"))
        .cloned()
        .collect();
    let mut count = 0;
    for name in names {
        let mut room = pack.data.maps[&name].clone();
        prepare_bicycle_claim(&mut room);
        let fossils = fossil_receipts(&mut room)?;
        count += fossils;
        let programs: Vec<_> = room.scripts.keys().cloned().collect();
        let mut changed = fossils > 0;
        for label in programs {
            let Some(mut body) = room.scripts[&label].as_array().cloned() else {
                continue;
            };
            let grants: Vec<_> = body
                .windows(3)
                .enumerate()
                .filter_map(|(at, rows)| {
                    (rows[0]["command"] == "verbosegiveitem"
                    && rows[1]["command"] == "iffalse"
                    && rows[2]["command"] == "setevent"
                    // This records which fossil was chosen; the actual claim
                    // lives in a shared branch and needs its own staged repair.
                    && rows[2]["args"][0] != "EVENT_MN_FOSSIL_CHOSE_LENS")
                        .then_some(at)
                })
                .collect();
            if grants.is_empty() {
                continue;
            }
            // Work backwards so several grants in one program retain their
            // source positions while receipt programs are inserted.
            for at in grants.into_iter().rev() {
                let item_id = body[at]["args"][0].as_str().context("reward item")?;
                let item = pack
                    .data
                    .items
                    .get(item_id)
                    .context("literal reward item")?;
                let amount = body[at]["args"]
                    .get(1)
                    .and_then(Value::as_str)
                    .unwrap_or("1");
                let quantity: u8 = amount.parse().context("literal reward quantity")?;
                ensure!(quantity > 0, "positive reward quantity");
                let receipt = format!("{label}ClaimReceipt{at}Text");
                ensure!(
                    !room.scripts.contains_key(&receipt),
                    "occupied reward receipt: {name}/{receipt}"
                );
                let message = if quantity == 1 {
                    format!("Received {}!", item.name)
                } else {
                    format!("Received {quantity} {}!", item.name)
                };
                authoring::text(&mut room, &receipt, &[&message]);
                body[at]["command"] = "giveitem".into();
                let mut receipt_at = at + 3;
                while body.get(receipt_at).is_some_and(|row| {
                    matches!(row["command"].as_str(), Some("setevent" | "setflag"))
                }) {
                    receipt_at += 1;
                }
                body.splice(
                    receipt_at..receipt_at,
                    [
                        c("playsound", &["SFX_ITEM"]),
                        c("waitsfx", &[]),
                        c("writetext", &[&receipt]),
                        c("waitbutton", &[]),
                    ],
                );
                count += 1;
            }
            room.scripts.insert(label, Value::Array(body));
            changed = true;
        }
        if changed {
            crate::northstar_story_revision::publish(pack, room)?;
        }
    }
    if count > 0 {
        let marker = "northstar-committed-reward-receipts-v1";
        if !pack.report.manifests.iter().any(|entry| entry == marker) {
            pack.report.manifests.push(marker.into());
        }
    }
    Ok(count)
}

fn prepare_bicycle_claim(room: &mut MapModule) {
    if room.id != "MnMinneapolisToolWorkshop" {
        return;
    }
    let Some(body) = room
        .scripts
        .get_mut("MnRewardBike")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let expected = [
        c("verbosegiveitem", &["BICYCLE", "1"]),
        c("iffalse", &["MnRewardBikeFull"]),
        c("sjump", &["MnRewardBikeOwned"]),
    ];
    if let Some(at) = body.windows(3).position(|rows| rows == expected) {
        // Keep the existing-owned branch for a bike obtained elsewhere. Only a
        // successful new grant gets the receipt added by the ordinary rewrite.
        body.insert(at + 2, c("setevent", &["EVENT_MN_BICYCLE_RECEIVED"]));
    }
}

// Fossil choice and exchange have shared success branches. The choice bit alone
// does not own the claim: wait until CHOSEN/EXCHANGED is committed to show it.
fn fossil_receipts(room: &mut MapModule) -> Result<usize> {
    let (grants, results): (&[&str], &[(&str, &str)]) = match room.id.as_str() {
        "MnR01C00R00" => (
            &[
                "MnFossilBluff",
                "MnFossilSpireConfirm",
                "MnFossilExchange",
                "MnFossilExchangeSpire",
            ],
            &[
                ("MnFossilChoiceDone", "EVENT_MN_FOSSIL_CHOSEN"),
                ("MnFossilExchangeResult", "EVENT_MN_FOSSIL_EXCHANGED"),
            ],
        ),
        "MnRochesterFossilLab" => (
            &["MnLabSecondOffer", "MnLabSecondLens"],
            &[("MnLabSecondResult", "EVENT_MN_FOSSIL_EXCHANGED")],
        ),
        _ => return Ok(0),
    };
    let mut count = 0;
    for label in grants {
        let Some(body) = room.scripts.get_mut(*label).and_then(Value::as_array_mut) else {
            continue;
        };
        for row in body
            .iter_mut()
            .filter(|row| row["command"] == "verbosegiveitem")
        {
            ensure!(
                matches!(
                    row["args"][0].as_str(),
                    Some("LENS_FOSSIL" | "SPIRE_FOSSIL")
                ),
                "unexpected fossil grant in {label}"
            );
            row["command"] = "giveitem".into();
            count += 1;
        }
    }
    if count == 0 {
        return Ok(0);
    }
    const RECEIPT: &str = "MnFossilCommittedReceiptText";
    for (label, flag) in results {
        let body = room
            .scripts
            .get_mut(*label)
            .and_then(Value::as_array_mut)
            .with_context(|| format!("fossil result {label}"))?;
        let at = body
            .iter()
            .position(|row| *row == c("setevent", &[flag]))
            .context("fossil claim before receipt")?;
        if !body.iter().any(|row| *row == c("writetext", &[RECEIPT])) {
            body.splice(
                at + 1..at + 1,
                [
                    c("playsound", &["SFX_ITEM"]),
                    c("waitsfx", &[]),
                    c("writetext", &[RECEIPT]),
                    c("waitbutton", &[]),
                ],
            );
        }
    }
    authoring::text(room, RECEIPT, &["Received a fossil!", "It is in the PACK."]);
    Ok(count)
}
