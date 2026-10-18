//! Repeat battles use ordinary compiled scripts and the authoritative trainer engine.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::{
    battle::start::{
        TrainerBattleRequest, TrainerBattleStartStatus, trainer_battle_start_from_trainer,
    },
    models::{LearnedMove, Pokemon, Trainer},
    state::{GameState, ScriptReturnFrame},
    world::session::{OverworldInteraction, OverworldInteractionTarget},
};
use serde_json::{Value, json};

use crate::{CompiledGamePack, GameDataSet};

pub const TRAINER_REMATCHES_MANIFEST_ID: &str = "trainer-rematches";
const SCRIPT_PREFIX: &str = "GeothiteRematch_";
const TALK_PREFIX: &str = "GeothiteRematch_Talk_";

fn script_label(map: &str, trainer: &str) -> String {
    format!("{SCRIPT_PREFIX}{map}_{trainer}")
}

fn talk_script_label(map: &str, trainer: &str, original: &str) -> String {
    format!("{TALK_PREFIX}{map}_{trainer}_{original}")
}

fn win_key(map: &str, trainer: &str) -> String {
    format!("GEOTHITE_TRAINER_WON_{map}_{trainer}")
}

fn command(name: &str, args: &[&str]) -> Value {
    json!({"command": name, "args": args})
}

/// Compose with a verified native, browser, or TUI pack without copying source content.
pub fn build_trainer_rematches_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data.trainer_rematches,
        "base already enables trainer rematches"
    );
    let mut pack = base.clone();
    let map_names = pack.data.map_name_by_constant_from_attributes()?;
    for (map, module) in &mut pack.data.maps {
        let requests: BTreeMap<_, _> = module
            .trainer_scripts
            .values()
            .chain(
                module
                    .scripted_trainer_battles
                    .iter()
                    .map(|battle| &battle.request),
            )
            .map(|request| (request.trainer_id.clone(), request.clone()))
            .collect();
        if requests.is_empty() {
            continue;
        }
        let intro = format!("{SCRIPT_PREFIX}{map}_IntroText");
        let victory = format!("{SCRIPT_PREFIX}{map}_WinText");
        let loss = format!("{SCRIPT_PREFIX}{map}_LossText");
        let mut scripts = BTreeMap::new();
        for (label, lines) in [
            (&intro, ["I've trained hard!", "Let's go again!"]),
            (&victory, ["You win again!", "Come back anytime!"]),
            (&loss, ["Training paid off!", "Try again anytime!"]),
        ] {
            scripts.insert(
                label.clone(),
                json!([
                    command("text", &[&format!("\"{}\"", lines[0])]),
                    command("line", &[&format!("\"{}\"", lines[1])]),
                    command("done", &[]),
                ]),
            );
        }
        for request in requests.values() {
            let label = script_label(map, &request.trainer_id);
            ensure!(
                !module.scripts.contains_key(&label),
                "rematch script collision: {label}"
            );
            scripts.insert(
                label,
                json!([
                    command("faceplayer", &[]),
                    command("opentext", &[]),
                    command("writetext", &[&intro]),
                    command("waitbutton", &[]),
                    command("closetext", &[]),
                    command("winlosstext", &[&victory, &loss]),
                    command(
                        "loadtrainer",
                        &[&request.trainer_class, &request.trainer_id]
                    ),
                    command("startbattle", &[]),
                    command("reloadmapafterbattle", &[]),
                    command("end", &[]),
                ]),
            );
        }
        // Call the existing conversation as a subroutine. Its normal end (or
        // jumptext return) resumes here only after every authored text page
        // and prompt has finished. No original command indexes are changed.
        let roots: BTreeSet<_> = module
            .objects
            .iter()
            .map(|object| object.script.as_str())
            .chain(
                module
                    .trainer_scripts
                    .values()
                    .map(|request| request.source_script.as_str()),
            )
            .chain(
                module
                    .scripted_trainer_battles
                    .iter()
                    .map(|battle| battle.source_script.as_str()),
            )
            .collect();
        for root in roots {
            let closure = crate::runtime_module_script_subset(&module.scripts, [root], false);
            for request in module.trainer_scripts.values().chain(
                module
                    .scripted_trainer_battles
                    .iter()
                    .map(|battle| &battle.request),
            ) {
                if !closure.contains_key(&request.source_script) {
                    continue;
                }
                let label = talk_script_label(map, &request.trainer_id, root);
                ensure!(
                    !module.scripts.contains_key(&label),
                    "rematch conversation collision: {label}"
                );
                scripts.insert(
                    label,
                    json!([
                        command("scall", &[root]),
                        command("sjump", &[&script_label(map, &request.trainer_id)]),
                    ]),
                );
            }
        }
        // Compile only the added scripts. Original scripts and their command
        // indexes remain intact, including all story rewards and callbacks.
        module
            .scripted_trainer_battles
            .extend(crate::parse_scripted_trainer_battles(map, &scripts)?);
        module
            .script_object_commands
            .extend(crate::parse_script_object_commands(map, &scripts)?);
        module
            .script_text_commands
            .extend(crate::parse_script_text_commands(map, &scripts)?);
        module
            .script_text_bodies
            .extend(crate::parse_script_text_bodies(map, &scripts)?);
        module
            .script_map_commands
            .extend(crate::parse_script_map_commands(map, &scripts, &map_names)?);
        // Calls into the original conversation need its labels in the
        // resolver, while only newly added commands belong in the overlay.
        let mut control_sources = module.scripts.clone();
        control_sources.extend(scripts.clone());
        module.script_control_commands.extend(
            crate::parse_script_control_commands(map, &control_sources)?
                .into_iter()
                .filter(|entry| scripts.contains_key(&entry.source_script)),
        );
        module
            .script_runtime_commands
            .extend(crate::parse_script_runtime_commands(map, &scripts)?);
        module.scripts.extend(scripts);
    }
    pack.data.trainer_rematches = true;
    pack.report
        .manifests
        .push(TRAINER_REMATCHES_MANIFEST_ID.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

impl GameDataSet {
    pub fn is_trainer_rematch_script(&self, source: &str) -> bool {
        self.trainer_rematches && source.starts_with(SCRIPT_PREFIX)
    }

    pub(crate) fn record_trainer_rematch_win(
        &self,
        state: &mut GameState,
        map: &str,
        trainer: &str,
    ) {
        if self.trainer_rematches {
            // Script memory is persisted in ordinary saves, like the PvE counters.
            state
                .script_runtime
                .memory
                .insert(win_key(map, trainer), "1".into());
        }
    }

    /// Only explicit object conversations enter this route. Sight, coordinate
    /// events, phone calls, and post-battle callbacks retain their authored flow.
    pub(crate) fn trainer_rematch_interaction(
        &self,
        state: &GameState,
        interaction: &OverworldInteraction,
    ) -> Result<Option<String>> {
        if !self.trainer_rematches
            || !matches!(
                &interaction.target,
                OverworldInteractionTarget::Object { .. }
            )
        {
            return Ok(None);
        }
        let module = self.map_module(&interaction.map_name)?;
        let closure = crate::runtime_module_script_subset(
            &module.scripts,
            [interaction.script.as_str()],
            false,
        );
        // Finish deferred story rewards (notably Whitney's/Clair's badges
        // and bag-full TM retries) before appending a leader's rematch.
        for flag in module.script_flag_commands.iter().filter(|flag| {
            closure.contains_key(&flag.source_script) && flag.command.starts_with("set")
        }) {
            // League defeat flags reset for each challenge. Let the authored
            // battle open its doors again before offering optional rematches.
            if flag.flag_id.starts_with("EVENT_BEAT_")
                && module.script_flag_commands.iter().any(|check| {
                    check.command == "checkevent"
                        && check.flag_id == flag.flag_id
                        && closure.contains_key(&check.source_script)
                })
                && !state
                    .flags
                    .is_event_flag_set(&flag.flag_id)
                    .unwrap_or(false)
            {
                return Ok(None);
            }
            if crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS
                .contains(&flag.flag_id.as_str())
                && !state
                    .flags
                    .is_engine_flag_set(&flag.flag_id)
                    .unwrap_or(false)
            {
                return Ok(None);
            }
            if (flag.flag_id.starts_with("EVENT_GOT_TM")
                || flag.flag_id.starts_with("EVENT_RECEIVED_TM"))
                && !state
                    .flags
                    .is_event_flag_set(&flag.flag_id)
                    .unwrap_or(false)
            {
                return Ok(None);
            }
        }
        let mut seen = BTreeSet::new();
        let mut eligible = Vec::new();
        for request in module.trainer_scripts.values().chain(
            module
                .scripted_trainer_battles
                .iter()
                .map(|battle| &battle.request),
        ) {
            if !closure.contains_key(&request.source_script) || !seen.insert(&request.trainer_id) {
                continue;
            }
            let recorded = state
                .script_runtime
                .memory
                .contains_key(&win_key(&interaction.map_name, &request.trainer_id));
            let flag = if !request.event_flag.is_empty() {
                request.event_flag.clone()
            } else {
                // Leaders' scripted requests have no event flag; their original
                // scripts own EVENT_BEAT_<class>. Respect existing progression.
                format!("EVENT_BEAT_{}", request.trainer_class)
            };
            let defeated = state.flags.is_event_flag_set(&flag).unwrap_or(false)
                || (request.event_flag.is_empty() && recorded);
            if defeated {
                let trainer = self
                    .trainers
                    .get(&request.trainer_id)
                    .context("rematch trainer missing")?;
                eligible.push((
                    trainer.party.iter().map(|p| p.level).max().unwrap_or(0),
                    &request.trainer_id,
                ));
            }
        }
        // Phone trainers and rivals can have several authored teams. Use the
        // strongest team actually beaten, never a future story team's roster.
        eligible.sort_unstable();
        Ok(eligible
            .last()
            .map(|(_, id)| talk_script_label(&interaction.map_name, id, &interaction.script)))
    }

    /// If an authored conversation itself offers a battle (for example a
    /// ready phone trainer), finish its dialogue at startbattle and resume
    /// the wrapper's appended line. The runtime unwinds through ordinary
    /// recorded call-stack mutations, so only one scaled battle starts and
    /// no original battle rewards/continuations are replayed.
    pub fn trainer_rematch_dialogue_return(&self, state: &GameState) -> Option<ScriptReturnFrame> {
        if !self.trainer_rematches {
            return None;
        }
        state
            .script_runtime
            .call_stack
            .iter()
            .rev()
            .find(|frame| {
                frame.source_script.starts_with(TALK_PREFIX) && frame.next_command_index == 1
            })
            .cloned()
    }

    pub(crate) fn rematch_battle_start(
        &self,
        state: &GameState,
        request: TrainerBattleRequest,
    ) -> Result<TrainerBattleStartStatus> {
        let trainer = self
            .trainers
            .get(&request.trainer_id)
            .context("rematch trainer missing")?;
        let authored_ace = trainer
            .party
            .iter()
            .map(|p| p.level)
            .max()
            .context("empty trainer team")?;
        let mut levels: Vec<_> = state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .filter(|p| !p.is_egg)
            .map(|p| u16::from(p.level))
            .collect();
        levels.sort_unstable_by(|a, b| b.cmp(a));
        levels.truncate(3);
        let anchor = if levels.is_empty() {
            authored_ace
        } else {
            ((levels.iter().sum::<u16>() + levels.len() as u16 / 2) / levels.len() as u16) as u8
        };
        let ace = anchor.max(authored_ace).clamp(1, 100);
        let scaled = self.rematch_trainer_at_ace(trainer, ace)?;
        Ok(TrainerBattleStartStatus::Started(
            trainer_battle_start_from_trainer(
                &scaled,
                &self.pokemon,
                &self.learnsets,
                &self.moves,
                &self.growth_rates,
                request,
            )?,
        ))
    }

    pub(crate) fn rematch_trainer_at_ace(&self, trainer: &Trainer, ace: u8) -> Result<Trainer> {
        let authored_ace = trainer
            .party
            .iter()
            .map(|p| p.level)
            .max()
            .context("empty trainer team")?;
        ensure!((authored_ace..=100).contains(&ace), "invalid rematch level");
        let original = crate::materialize_trainer_party(
            trainer,
            &self.pokemon,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
        )?;
        let mut scaled = trainer.clone();
        for (slot, original) in scaled.party.iter_mut().zip(original) {
            let authored_level = slot.level;
            slot.level += ace - authored_ace;
            slot.moves = self.rematch_moves(&original, authored_level, slot.level)?;
        }
        Ok(scaled)
    }

    fn rematch_moves(
        &self,
        original: &Pokemon,
        authored: u8,
        level: u8,
    ) -> Result<Vec<LearnedMove>> {
        let mut moves: Vec<_> = original.moves.iter().map(|m| m.name.clone()).collect();
        for entry in self
            .learnsets
            .get(&original.species.id)
            .context("rematch learnset missing")?
            .iter()
            .filter(|entry| entry.0 > authored && entry.0 <= level)
        {
            if !moves.contains(&entry.1) {
                if moves.len() == 4 {
                    moves.remove(0);
                }
                moves.push(entry.1.clone());
            }
        }
        moves
            .into_iter()
            .map(|name| {
                let pp = self.moves.get(&name).context("rematch move missing")?.pp;
                Ok(LearnedMove {
                    name,
                    current_pp: pp,
                    pp_ups: 0,
                })
            })
            .collect()
    }
}
