//! Native gym battles, retryable rewards, puzzle menus and saved gate changes.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{BackgroundEvent, CoordEvent, MapScriptSectionCommand};
use crystal_core::models::{LearnedMove, TrainerPartyPokemon};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::gym_design::{Gym, Member};
use crate::{CompiledGamePack, MapModule};

pub(crate) const BADGES: [&str; 8] = [
    "ENGINE_ZEPHYRBADGE",
    "ENGINE_HIVEBADGE",
    "ENGINE_PLAINBADGE",
    "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE",
    "ENGINE_MINERALBADGE",
    "ENGINE_GLACIERBADGE",
    "ENGINE_RISINGBADGE",
];

fn command(name: &str, args: &[&str]) -> Value {
    authoring::command(name, args)
}

fn event(pack: &mut CompiledGamePack, name: &str) -> Result<()> {
    let globals = &mut pack.data.story_event_script_constants.global;
    if let Some(id) = globals.get(name) {
        ensure!(
            !globals
                .iter()
                .any(|(other, value)| other != name && other.starts_with("EVENT_") && value == id),
            "event collision {name}"
        );
    } else {
        let next = globals
            .iter()
            .filter(|(k, _)| k.starts_with("EVENT_"))
            .map(|(_, n)| *n)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .context("gym event overflow")?;
        globals.insert(name.into(), next);
    }
    Ok(())
}

fn text(module: &mut MapModule, label: &str, lines: &[String]) {
    authoring::text(
        module,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
}

fn speech(module: &mut MapModule, label: &str, lines: &[&str]) {
    let body = format!("{label}Text");
    authoring::text(module, &body, lines);
    module
        .scripts
        .insert(label.into(), json!([command("jumptext", &[&body])]));
}

fn npc(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    script: &str,
    sprite: &str,
    at: [u16; 2],
) -> Result<()> {
    ensure!(
        pack.runtime_files().contains_key(&format!(
            "gfx/sprites/{}.png",
            sprite.trim_start_matches("SPRITE_").to_ascii_lowercase()
        )),
        "missing sprite {sprite}"
    );
    authoring::npc(
        pack,
        module,
        script,
        sprite,
        i16::try_from(at[0])?,
        i16::try_from(at[1])?,
    )?;
    let actor = module.objects.last_mut().context("gym actor")?;
    actor.object_type = "OBJECTTYPE_SCRIPT".into();
    actor.object_identifier = Some(script.into());
    actor.radius = 0;
    actor.sprite_has_facings = pack
        .data
        .maps
        .values()
        .flat_map(|map| &map.objects)
        .find(|source| source.sprite == sprite)
        .is_some_and(|source| source.sprite_has_facings);
    Ok(())
}

fn register_trainer(
    pack: &mut CompiledGamePack,
    id: &str,
    name: &str,
    class: &str,
    party: &[Member],
) -> Result<()> {
    ensure!(
        !pack.data.trainers.trainers.contains_key(id),
        "trainer occupied {id}"
    );
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == class)
        .with_context(|| format!("source trainer class {class}"))?
        .clone();
    let dvs = trainer.party.first().context("native trainer DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == class)
        .filter_map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
        })
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("trainer id overflow")?;
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .contains_key(id),
        "trainer constant occupied {id}"
    );
    trainer.trainer_id = id.into();
    trainer.name = name.into();
    trainer.items.clear();
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    ensure!((1..=6).contains(&party.len()), "gym party extent");
    trainer.party = party
        .iter()
        .map(|member| {
            ensure!((1..=100).contains(&member.level), "gym level");
            let pokemon = pack
                .data
                .create_pokemon(&member.species, member.level, dvs)?;
            let moves = if member.moves.is_empty() {
                pokemon
                    .moves
                    .into_iter()
                    .filter(|m| {
                        !matches!(
                            m.name.as_str(),
                            "SELFDESTRUCT" | "EXPLOSION" | "SONICBOOM" | "DOUBLE_TEAM"
                        )
                    })
                    .collect()
            } else {
                ensure!(member.moves.len() <= 4, "four move slots");
                member
                    .moves
                    .iter()
                    .map(|name| {
                        let native = pack.data.moves.get(name).context("gym move")?;
                        let species = pack
                            .data
                            .pokemon
                            .get(&member.species)
                            .context("gym species")?;
                        let legal =
                            species.tmhm_learnset.contains(name)
                                || pack.data.learnsets.get(&member.species).is_some_and(
                                    |learnset| {
                                        learnset.iter().any(|entry| {
                                            entry.0 <= member.level && &entry.1 == name
                                        })
                                    },
                                );
                        ensure!(
                            legal,
                            "{} cannot learn {name} at {}",
                            member.species,
                            member.level
                        );
                        Ok(LearnedMove {
                            name: name.clone(),
                            current_pp: native.pp,
                            pp_ups: 0,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
            };
            ensure!(!moves.is_empty(), "gym member has no moves");
            Ok(TrainerPartyPokemon {
                species: member.species.clone(),
                level: member.level,
                item: None,
                moves,
                dvs,
            })
        })
        .collect::<Result<_>>()?;
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(id.into(), next);
    Ok(())
}

pub(crate) fn requirement(commands: &mut Vec<Value>, token: &str, denied: &str) -> Result<()> {
    if let Some(slot) = token.strip_prefix("BADGE_") {
        let index = slot
            .parse::<usize>()?
            .checked_sub(1)
            .context("badge slot")?;
        commands.push(command(
            "checkflag",
            &[BADGES.get(index).context("badge slot range")?],
        ));
    } else {
        commands.push(command("checkevent", &[token]));
    }
    commands.push(command("iffalse", &[denied]));
    Ok(())
}

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    module: &mut MapModule,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
) -> Result<()> {
    ensure!(
        BADGES[usize::from(gym.slot - 1)] == gym.badge_flag,
        "inherited badge permission changed"
    );
    ensure!(
        pack.data.items.contains_key(&gym.reward),
        "missing TM reward"
    );
    for token in gym.requires.iter().filter(|t| t.starts_with("EVENT_")) {
        event(pack, token)?;
    }
    if !gym.cut_gates.is_empty() {
        ensure!(
            pack.data.items.contains_key("HM_CUT"),
            "Cut garden requires its teaching item"
        );
        event(pack, &crate::gym_cut::guide_gift_event(gym))?;
    }
    for token in [gym.defeated(), gym.reward_event(), gym.court_event()]
        .iter()
        .chain(gym.gates.iter().map(|g| &g.event))
        .chain(gym.stations.iter().map(|s| &s.event))
        .chain(gym.pads.iter().map(|p| &p.event))
        .chain(gym.cut_gates.iter().map(|gate| &gate.event))
        .chain(gym.switch_puzzle.iter().flat_map(|p| p.events.iter()))
        .chain(
            gym.routers
                .iter()
                .flat_map(|r| r.modes.iter().map(|m| &m.event)),
        )
    {
        event(pack, token)?;
    }
    let refresh = format!("{}RefreshGates", gym.id);
    let mut refresh_body = Vec::new();
    crate::gym_cut::install_scripts(module, gym, symbols, &refresh, &mut refresh_body);
    if !gym.court_requires.is_empty() {
        let denied = format!("{}CourtUnqualified", gym.id);
        for token in &gym.court_requires {
            requirement(&mut refresh_body, token, &denied)?;
        }
        refresh_body.push(command("setevent", &[&gym.court_event()]));
        module.scripts.insert(
            denied,
            json!([
                command("clearevent", &[&gym.court_event()]),
                command("end", &[])
            ]),
        );
    }
    for (index, gate) in gym.gates.iter().enumerate() {
        let label = format!("{}Gate{index}", gym.id);
        let done = format!("{label}Done");
        let mut body = Vec::new();
        for token in &gate.requires {
            requirement(&mut body, token, &done)?;
        }
        body.extend([
            command(
                "changeblock",
                &[
                    &(gate.at[0] * 2).to_string(),
                    &(gate.at[1] * 2).to_string(),
                    &symbols[&gate.open_symbol].to_string(),
                ],
            ),
            command("setevent", &[&gate.event]),
            command("end", &[]),
        ]);
        module.scripts.insert(label.clone(), Value::Array(body));
        let closed_symbol = gym.rows[usize::from(gate.at[1])]
            .chars()
            .nth(usize::from(gate.at[0]))
            .context("closed gate symbol")?;
        module.scripts.insert(
            done,
            json!([
                command(
                    "changeblock",
                    &[
                        &(gate.at[0] * 2).to_string(),
                        &(gate.at[1] * 2).to_string(),
                        &symbols[&closed_symbol].to_string()
                    ]
                ),
                command("clearevent", &[&gate.event]),
                command("end", &[])
            ]),
        );
        refresh_body.push(command("scall", &[&label]));
    }
    refresh_body.push(command("end", &[]));
    module
        .scripts
        .insert(refresh.clone(), Value::Array(refresh_body));
    module
        .map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_TILES".into(), refresh.clone()],
            command_index: module.map_script_section_commands.len(),
        });
    let denied = format!("{}Requirements", gym.id);
    speech(
        module,
        &denied,
        &[
            "Finish the earlier",
            "challenge first.",
            "Ask the gym guide.",
        ],
    );
    let guide = format!("{}Guide", gym.id);
    let guide_text = format!("{guide}Text");
    text(module, &guide_text, &gym.guide);
    module.scripts.insert(
        guide.clone(),
        json!([command("jumptextfaceplayer", &[&guide_text])]),
    );
    crate::gym_cut::install_guide(module, gym, &guide, &guide_text);
    npc(pack, module, &guide, "SPRITE_GYM_GUIDE", gym.guide_at)?;
    for pad in &gym.pads {
        let label = format!("{}{}Pad", gym.id, pad.id);
        let done = format!("{label}Done");
        let message = format!("{label}Text");
        let mut body = vec![
            command("checkevent", &[&pad.event]),
            command("iftrue", &[&done]),
        ];
        for token in &pad.requires {
            requirement(&mut body, token, &denied)?;
        }
        body.extend([
            command("setevent", &[&pad.event]),
            command("scall", &[&refresh]),
            command("opentext", &[]),
            command("writetext", &[&message]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[]),
        ]);
        module.scripts.insert(label.clone(), Value::Array(body));
        module.scripts.insert(done, json!([command("end", &[])]));
        text(module, &message, &pad.lines);
        for [x, y] in &pad.tiles_core {
            module.events.coord_events.push(CoordEvent {
                x: *x,
                y: *y,
                scene_id: String::new(),
                script_name: label.clone(),
            });
        }
    }
    for trainer in &gym.trainers {
        let flag = format!("EVENT_{}_DEFEATED", trainer.id);
        event(pack, &flag)?;
        register_trainer(
            pack,
            &trainer.id,
            &trainer.name,
            &trainer.trainer_class,
            &trainer.party,
        )?;
        let after = format!("{}After", trainer.id);
        let seen = format!("{}Seen", trainer.id);
        let won = format!("{}WonText", trainer.id);
        let lost = format!("{}LostText", trainer.id);
        authoring::text(module, &seen, &["Let's practice", "before the leader!"]);
        speech(module, &after, &["Well played!", "Try the next stop."]);
        authoring::text(module, &won, &["Good battle!"]);
        authoring::text(module, &lost, &["Let's try again."]);
        if !trainer.before_lines.is_empty() {
            text(module, &seen, &trainer.before_lines);
        }
        if !trainer.after_lines.is_empty() {
            text(module, &format!("{after}Text"), &trainer.after_lines);
        }
        let repeat = gym
            .routers
            .iter()
            .find(|r| r.operator.as_deref() == Some(&trainer.id))
            .map_or_else(|| after.clone(), |r| format!("{}{}Routing", gym.id, r.id));
        let mut battle_script = vec![
            command("faceplayer", &[]),
            command("checkevent", &[&flag]),
            command("iftrue", &[&repeat]),
        ];
        for token in &trainer.requires {
            requirement(&mut battle_script, token, &denied)?;
        }
        battle_script.extend([
            command("opentext", &[]),
            command("writetext", &[&seen]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("winlosstext", &[&won, &lost]),
            command("loadtrainer", &[&trainer.trainer_class, &trainer.id]),
            command("startbattle", &[]),
            command("reloadmapafterbattle", &[]),
            command("setevent", &[&flag]),
            command("scall", &[&refresh]),
            command("sjump", &[&after]),
        ]);
        module
            .scripts
            .insert(trainer.id.clone(), Value::Array(battle_script));
        npc(pack, module, &trainer.id, &trainer.sprite, trainer.at)?;
    }
    for station in &gym.stations {
        let label = format!("{}{}Station", gym.id, station.id);
        let already = format!("{label}Already");
        let correct = format!("{label}Correct");
        let wrong = format!("{label}Wrong");
        let prompt = format!("{label}Prompt");
        let header = format!("{label}Header");
        let data = format!("{label}MenuData");
        let mut body = vec![
            command("checkevent", &[&station.event]),
            command("iftrue", &[&already]),
        ];
        for token in &station.requires {
            requirement(&mut body, token, &denied)?;
        }
        body.extend([
            command("opentext", &[]),
            command("writetext", &[&prompt]),
            command("loadmenu", &[&header]),
            command("verticalmenu", &[]),
            command("closewindow", &[]),
            command("ifequal", &[&station.answer.to_string(), &correct]),
            command("sjump", &[&wrong]),
        ]);
        module.scripts.insert(label.clone(), Value::Array(body));
        text(module, &prompt, &station.prompt);
        module.scripts.insert(
            header,
            json!([
                command("db", &["MENU_BACKUP_TILES"]),
                command("menu_coords", &["0", "0", "17", "9"]),
                command("dw", &[&data]),
                command("db", &["1"])
            ]),
        );
        let mut choices = vec![
            command("db", &["STATICMENU_CURSOR"]),
            command("db", &[&station.options.len().to_string()]),
        ];
        choices.extend(
            station
                .options
                .iter()
                .map(|option| command("db", &[&format!("\"{option}@\"")])),
        );
        module.scripts.insert(data, Value::Array(choices));
        speech(module, &already, &["This stop is done.", "Keep going!"]);
        if !station.already_lines.is_empty() {
            text(module, &format!("{already}Text"), &station.already_lines);
        }
        let success_text = format!("{correct}Text");
        authoring::text(module, &success_text, &["Recorded!", "The way is marked."]);
        if !station.success_lines.is_empty() {
            text(module, &success_text, &station.success_lines);
        }
        module.scripts.insert(
            correct,
            json!([
                command("setevent", &[&station.event]),
                command("scall", &[&refresh]),
                command("writetext", &[&success_text]),
                command("waitbutton", &[]),
                command("closetext", &[]),
                command("end", &[])
            ]),
        );
        let wrong_text = format!("{wrong}Text");
        authoring::text(
            module,
            &wrong_text,
            &["Read the guide.", "You can try again."],
        );
        if !station.decline_lines.is_empty() {
            text(module, &wrong_text, &station.decline_lines);
        }
        module.scripts.insert(
            wrong,
            json!([
                command("writetext", &[&wrong_text]),
                command("waitbutton", &[]),
                command("closetext", &[]),
                command("end", &[])
            ]),
        );
        module.events.bg_events.push(BackgroundEvent {
            x: station.at[0],
            y: station.at[1],
            event_type: "BGEVENT_READ".into(),
            script: label,
        });
    }
    crate::gym_routing::install(module, gym, &refresh)?;
    crate::gym_switches::install(module, gym, &refresh)?;
    leader(pack, module, gym, &refresh, &denied)?;
    finish(pack, module)
}

fn leader(
    pack: &mut CompiledGamePack,
    module: &mut MapModule,
    gym: &Gym,
    refresh: &str,
    denied: &str,
) -> Result<()> {
    let battle = gym.battle_id();
    register_trainer(pack, &battle, &gym.leader, &gym.trainer_class, &gym.party)?;
    let script = format!("{}Leader", gym.id);
    let reward = format!("{script}Reward");
    let after = format!("{script}After");
    let full = format!("{script}Full");
    let seen = format!("{script}SeenText");
    let won = format!("{script}WonText");
    let lost = format!("{script}LostText");
    let gift = format!("{script}GiftText");
    authoring::text(
        module,
        &seen,
        &[&format!("I am {}.", gym.leader), "Show me your team!"],
    );
    authoring::text(
        module,
        &gift,
        &[
            "You earned a BADGE",
            "and this TM!",
            "Ask the city guide",
            "where to go next.",
        ],
    );
    speech(module, &after, &["Your BADGE is safe.", "Keep exploring!"]);
    authoring::text(module, &won, &["Your team is ready."]);
    authoring::text(module, &lost, &["Rest your partners", "and try again."]);
    if !gym.before_lines.is_empty() {
        text(module, &seen, &gym.before_lines);
    }
    if !gym.victory_lines.is_empty() {
        text(module, &won, &gym.victory_lines);
    }
    if !gym.after_lines.is_empty() {
        text(module, &format!("{after}Text"), &gym.after_lines);
    }
    speech(
        module,
        &full,
        &["Your TM POCKET", "is full. Return", "when you have room."],
    );
    let mut body = vec![
        command("faceplayer", &[]),
        command("checkevent", &[&gym.defeated()]),
        command("iftrue", &[&reward]),
    ];
    for token in &gym.requires {
        requirement(&mut body, token, denied)?;
    }
    let court_denied = format!("{}CourtRequirements", gym.id);
    if gym.court_denied_lines.is_empty() {
        requirement(&mut body, &gym.court_event(), denied)?;
    } else {
        speech(module, &court_denied, &["Complete practice."]);
        text(
            module,
            &format!("{court_denied}Text"),
            &gym.court_denied_lines,
        );
        requirement(&mut body, &gym.court_event(), &court_denied)?;
    }
    body.extend([
        command("opentext", &[]),
        command("writetext", &[&seen]),
        command("waitbutton", &[]),
        command("closetext", &[]),
        command("winlosstext", &[&won, &lost]),
        command("loadtrainer", &[&gym.trainer_class, &battle]),
        command("startbattle", &[]),
        command("reloadmapafterbattle", &[]),
        command("setflag", &[&gym.badge_flag]),
        command("setevent", &[&gym.defeated()]),
        command("scall", &[refresh]),
        command("sjump", &[&reward]),
    ]);
    module.scripts.insert(script.clone(), Value::Array(body));
    module.scripts.insert(
        reward,
        json!([
            command("checkevent", &[&gym.reward_event()]),
            command("iftrue", &[&after]),
            command("giveitem", &[&gym.reward, "1"]),
            command("iffalse", &[&full]),
            command("setevent", &[&gym.reward_event()]),
            command("opentext", &[]),
            command("writetext", &[&gift]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
    npc(pack, module, &script, &gym.leader_sprite, gym.leader_at)
}

pub(crate) fn finish(pack: &CompiledGamePack, module: &mut MapModule) -> Result<()> {
    authoring::finish(pack, module)?;
    let name = &module.id;
    let scripts = &module.scripts;
    module.script_runtime_commands = crate::parse_script_runtime_commands(name, scripts)?;
    module.script_object_commands = crate::parse_script_object_commands(name, scripts)?;
    module.script_variable_commands = crate::parse_script_variable_commands(name, scripts)?;
    let map_names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|map| (map.constant.clone(), map.name.clone()))
        .collect();
    module.script_map_commands = crate::parse_script_map_commands(name, scripts, &map_names)?;
    module.script_block_changes = crate::parse_script_block_changes(name, scripts)?;
    module.script_menu_definitions = crate::parse_script_menu_definitions(name, scripts)?;
    module.script_vertical_menus =
        crate::parse_script_vertical_menus(name, scripts, &module.script_menu_definitions)?;
    module.trainer_scripts = crate::parse_trainer_scripts(name, scripts)?;
    module.scripted_trainer_battles = crate::parse_scripted_trainer_battles(name, scripts)?;
    Ok(())
}
