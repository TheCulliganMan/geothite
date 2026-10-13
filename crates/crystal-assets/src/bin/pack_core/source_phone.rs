//! Phone tables retain authored slot numbers, scripts and caller presentation.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn slots(source: &str, prefix: &str, end: &str) -> Result<BTreeMap<usize, String>> {
    let mut result = BTreeMap::new();
    let mut index = 0;
    for line in source.lines().map(code_part) {
        if line.starts_with(end) {
            break;
        }
        if line == "const_def" {
            index = 0;
        } else if let Some(raw) = line.strip_prefix("const_def ") {
            index = raw.parse()?;
        } else if line == "const_skip" {
            index += 1;
        } else if let Some(raw) = line.strip_prefix("const_skip ") {
            index += raw.parse::<usize>()?;
        } else if let Some(id) = line.strip_prefix("const ") {
            if id.starts_with(prefix) {
                ensure!(
                    result.insert(index, id.to_string()).is_none(),
                    "duplicate phone slot"
                );
            }
            index += 1;
        }
    }
    ensure!(!result.is_empty(), "missing authored phone slots");
    Ok(result)
}

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let read = |path: &str| std::fs::read_to_string(vendor.join(path)).with_context(|| format!("phone source {path}"));
    let data = root.join("apps/web/assets/data");
    let constants: Value = serde_json::from_slice(&std::fs::read(
        data.join("story_event_script_constants.json"),
    )?)?;
    let numeric = |id: &str| -> Result<u8> {
        id.parse()
            .ok()
            .or_else(|| {
                constants["global"][id]
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
            })
            .with_context(|| format!("missing phone constant {id}"))
    };
    let phone_constants = read("constants/phone_constants.asm")?;
    let contact_slots = slots(&phone_constants, "PHONE_", "DEF NUM_PHONE_CONTACTS ")?;
    let special_source = phone_constants
        .split("DEF NUM_PHONE_CONTACTS ")
        .nth(1)
        .context("special phone constants")?;
    let special_slots = slots(special_source, "SPECIALCALL_", "DEF NUM_SPECIALCALLS ")?;
    let rows = read("data/phone/phone_contacts.asm")?
        .lines()
        .map(code_part)
        .filter_map(|line| line.strip_prefix("phone ").map(source_arguments))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        rows.len() == usize::from(numeric("NUM_PHONE_CONTACTS")?) + 1,
        "phone source slot count mismatch"
    );
    let name_source = read("data/phone/non_trainer_names.asm")?;
    let pointers: Vec<_> = name_source
        .lines()
        .map(code_part)
        .filter_map(|line| line.strip_prefix("dw "))
        .collect();
    let mut names = BTreeMap::new();
    for line in name_source.lines().map(code_part) {
        if let Some((label, raw)) = line.split_once(':') {
            if !label.starts_with('.') { continue; }
            let text = raw
                .trim()
                .strip_prefix("db ")
                .context("phone name string")?
                .trim()
                .trim_matches('"')
                .trim_end_matches('@');
            names.insert(
                label.to_string(),
                text.split("<LF>")
                    .map(str::trim)
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
            );
        }
    }
    let trainer_constants = read("constants/trainer_constants.asm")?;
    let classes = trainer_constants
        .lines()
        .map(code_part)
        .filter_map(|line| line.strip_prefix("trainerclass "))
        .filter(|id| *id != "TRAINER_NONE")
        .collect::<Vec<_>>();
    let class_names = read("data/trainers/class_names.asm")?
        .lines()
        .map(code_part)
        .filter_map(|line| {
            line.strip_prefix("li ")
                .map(|name| name.trim_matches('"').trim_end_matches('@').to_string())
        })
        .collect::<Vec<_>>();
    ensure!(
        classes.len() == class_names.len(),
        "phone trainer class name count mismatch"
    );
    let class_names: BTreeMap<_, _> = classes.into_iter().zip(class_names).collect();
    let trainers: Value = serde_json::from_slice(&std::fs::read(data.join("trainers.json"))?)?;
    let mut contacts = BTreeMap::new();
    let mut non_trainers = BTreeMap::new();
    for (index, id) in contact_slots {
        let args = rows.get(index).context("phone slot outside source table")?;
        ensure!(args.len() == 7, "phone table requires seven operands");
        let lines = if args[0] == "TRAINER_NONE" {
            let index = usize::from(numeric(&args[1])?);
            let target = pointers
                .get(index)
                .context("phone name index outside source")?;
            if args[1] != "PHONE_00" {
                ensure!(
                    non_trainers.insert(args[1].clone(), id.clone()).is_none(),
                    "duplicate named nontrainer phone contact"
                );
            }
            names
                .get(*target)
                .context("missing authored nontrainer name")?
                .clone()
        } else {
            let trainer = &trainers["trainers"][&args[1]];
            ensure!(
                trainer["trainer_class"] == args[0],
                "phone trainer class mismatch"
            );
            vec![
                format!(
                    "{}:",
                    trainer["name"].as_str().context("phone trainer name")?
                ),
                class_names
                    .get(args[0].as_str())
                    .context("phone trainer class name")?
                    .clone(),
            ]
        };
        let primary = lines
            .first()
            .context("phone presentation missing")?
            .trim_end_matches(':')
            .trim()
            .to_string();
        let value = json!({"contactId":id,"trainerClass":args[0],"trainerLabel":args[1],"lines":lines,"primaryLabel":primary,"mapConstant":if args[2] == "N_A" {None} else {Some(&args[2])},"calleeTimeMask":numeric(&args[3])?,"calleeScript":args[4],"callerTimeMask":numeric(&args[5])?,"callerScript":args[6]});
        let _: crystal_core::systems::phone::PhoneContactRecord =
            serde_json::from_value(value.clone())?;
        contacts.insert(id, value);
    }
    let mut permanent = BTreeMap::new();
    let mut ended = false;
    for line in read("data/phone/permanent_numbers.asm")?
        .lines()
        .map(code_part)
    {
        let Some(id) = line.strip_prefix("db ") else {
            continue;
        };
        ensure!(!ended, "permanent phone rows after sentinel");
        if id == "-1" {
            ended = true;
            continue;
        }
        let contact = non_trainers
            .get(id)
            .context("unknown permanent phone contact")?;
        let index = permanent.len();
        ensure!(
            permanent
                .insert(contact.clone(), json!({"listIndex":index}))
                .is_none(),
            "duplicate permanent phone number"
        );
    }
    ensure!(
        ended && !permanent.is_empty(),
        "missing permanent phone table"
    );
    let special_rows = read("data/phone/special_calls.asm")?
        .lines()
        .map(code_part)
        .filter_map(|line| line.strip_prefix("specialcall ").map(source_arguments))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        special_rows.len() == usize::from(numeric("NUM_SPECIALCALLS")?),
        "special phone source count mismatch"
    );
    let mut special = BTreeMap::new();
    for (index, args) in special_rows.into_iter().enumerate() {
        ensure!(
            args.len() == 3
                && matches!(
                    args[0].as_str(),
                    "SpecialCallOnlyWhenOutside" | "SpecialCallWhereverYouAre"
                ),
            "invalid special phone row"
        );
        let value = index + 1;
        let id = special_slots
            .get(&value)
            .context("missing special phone ID")?;
        let contact = non_trainers
            .get(&args[1])
            .context("unknown special phone contact")?;
        special.insert(
            id.clone(),
            json!({"value":value,"condition":args[0],"contactId":contact,"callerScript":args[2]}),
        );
    }
    let _: crystal_core::systems::phone::PhoneContactCatalog =
        serde_json::from_value(json!(contacts))?;
    for (name, value) in [
        ("phone_contacts", json!(contacts)),
        ("permanent_phone_numbers", json!(permanent)),
        ("special_phone_calls", json!(special)),
    ] {
        let directory = data.join("content-packs/core-modular").join(name);
        std::fs::create_dir_all(&directory)?;
        let bytes = serde_json::to_vec_pretty(&value)?;
        std::fs::write(data.join(format!("{name}.json")), &bytes)?;
        std::fs::write(directory.join("rules.json"), bytes)?;
    }
    Ok(())
}
