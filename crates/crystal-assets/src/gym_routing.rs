//! Mutually exclusive supply routing through native background-event menus.
use anyhow::Result;
use crystal_core::map::BackgroundEvent;
use serde_json::{Value, json};

use crate::MapModule;
use crate::grand_marais_modpack::authoring::{command, text};
use crate::gym_design::Gym;
use crate::gym_scripts::requirement;

fn message(module: &mut MapModule, label: &str, lines: &[String]) {
    let text_label = format!("{label}Text");
    text(
        module,
        &text_label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    module.scripts.insert(
        label.into(),
        json!([
            command("writetext", &[&text_label]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
}

pub(crate) fn install(module: &mut MapModule, gym: &Gym, refresh: &str) -> Result<()> {
    for router in &gym.routers {
        let root = format!("{}{}Routing", gym.id, router.id);
        if router.toggle {
            install_toggle(module, router, &root, refresh)?;
            continue;
        }
        let prompt = format!("{root}Prompt");
        let header = format!("{root}Header");
        let data = format!("{root}MenuData");
        text(
            module,
            &prompt,
            &router.prompt.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        module.scripts.insert(
            header.clone(),
            json!([
                command("db", &["MENU_BACKUP_TILES"]),
                command("menu_coords", &["0", "0", "17", "9"]),
                command("dw", &[&data]),
                command("db", &["1"])
            ]),
        );
        let mut options = vec![
            command("db", &["STATICMENU_CURSOR"]),
            command("db", &[&router.modes.len().to_string()]),
        ];
        options.extend(
            router
                .modes
                .iter()
                .map(|mode| command("db", &[&format!("\"{}@\"", mode.label)])),
        );
        module.scripts.insert(data, Value::Array(options));
        let mut menu = vec![
            command("opentext", &[]),
            command("writetext", &[&prompt]),
            command("loadmenu", &[&header]),
            command("verticalmenu", &[]),
            command("closewindow", &[]),
        ];
        for (index, mode) in router.modes.iter().enumerate() {
            let label = format!("{root}{}", mode.id);
            let denied = format!("{label}Denied");
            let success = format!("{label}Success");
            menu.push(command("ifequal", &[&(index + 1).to_string(), &label]));
            let mut body = Vec::new();
            for token in &mode.requires {
                requirement(&mut body, token, &denied)?;
            }
            // Only this desk's other route flags may be cleared. Trainer wins,
            // generator supplies and badge/reward flags are never reset here.
            body.extend(
                router
                    .modes
                    .iter()
                    .filter(|other| other.event != mode.event)
                    .map(|other| command("clearevent", &[&other.event])),
            );
            body.extend([
                command("setevent", &[&mode.event]),
                command("scall", &[refresh]),
                command("sjump", &[&success]),
            ]);
            module.scripts.insert(label, Value::Array(body));
            message(module, &denied, &mode.denied_lines);
            message(module, &success, &mode.lines);
        }
        // Native B/cancel is zero, so it takes neither routing branch.
        menu.extend([command("closetext", &[]), command("end", &[])]);
        module.scripts.insert(root.clone(), Value::Array(menu));
        if router.operator.is_none() {
            module.events.bg_events.push(BackgroundEvent {
                x: router.at[0],
                y: router.at[1],
                event_type: "BGEVENT_READ".into(),
                script: root,
            });
        }
    }
    Ok(())
}

fn install_toggle(
    module: &mut MapModule,
    router: &crate::gym_design::Router,
    root: &str,
    refresh: &str,
) -> Result<()> {
    let [first, second] = router.modes.as_slice() else {
        anyhow::bail!("a physical switch requires two positions");
    };
    module.scripts.insert(
        root.into(),
        json!([
            command("opentext", &[]),
            command("checkevent", &[&first.event]),
            command("iftrue", &[&format!("{root}{}", second.id)]),
            command("sjump", &[&format!("{root}{}", first.id)])
        ]),
    );
    for (selected, other) in [(first, second), (second, first)] {
        let branch = format!("{root}{}", selected.id);
        let denied = format!("{branch}Denied");
        let success = format!("{branch}Success");
        let mut body = Vec::new();
        for token in &selected.requires {
            requirement(&mut body, token, &denied)?;
        }
        body.extend([
            command("clearevent", &[&other.event]),
            command("setevent", &[&selected.event]),
            command("scall", &[refresh]),
            command("sjump", &[&success]),
        ]);
        module.scripts.insert(branch, Value::Array(body));
        message(module, &denied, &selected.denied_lines);
        message(module, &success, &selected.lines);
    }
    if router.operator.is_none() {
        module.events.bg_events.push(BackgroundEvent {
            x: router.at[0],
            y: router.at[1],
            event_type: "BGEVENT_READ".into(),
            script: root.into(),
        });
    }
    Ok(())
}
