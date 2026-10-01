fn require_consumed_divider_trace(context: &str, divider: &ReplayDivider) -> Result<()> {
    if divider.remaining() != 0 {
        anyhow::bail!(
            "{context} divider trace has {} unconsumed samples after {} reads",
            divider.remaining(),
            divider.consumed()
        );
    }
    Ok(())
}

fn item_happiness_change_code(item: &Item) -> Option<&'static str> {
    if item.vitamin_stat.is_some() {
        return Some("HAPPINESS_USEDITEM");
    }
    if item.script_name != "X_ACCURACY" && item.battle_stat_boost_stat.is_some() {
        return Some("HAPPINESS_USEDXITEM");
    }
    match item.script_name.as_str() {
        "HEAL_POWDER" | "ENERGYPOWDER" => Some("HAPPINESS_BITTERPOWDER"),
        "ENERGY_ROOT" => Some("HAPPINESS_ENERGYROOT"),
        "REVIVAL_HERB" => Some("HAPPINESS_REVIVALHERB"),
        _ => None,
    }
}

/// ASM `data/trainers/leaders.asm`: battle/victory music and happiness classes.
pub fn is_gym_leader_class(trainer_class: &str) -> bool {
    matches!(
        trainer_class,
        "FALKNER"
            | "WHITNEY"
            | "BUGSY"
            | "MORTY"
            | "PRYCE"
            | "JASMINE"
            | "CHUCK"
            | "CLAIR"
            | "WILL"
            | "BRUNO"
            | "KAREN"
            | "KOGA"
            | "CHAMPION"
            | "RED"
            | "BROCK"
            | "MISTY"
            | "LT_SURGE"
            | "ERIKA"
            | "JANINE"
            | "SABRINA"
            | "BLAINE"
            | "BLUE"
    )
}

fn apply_normal_vblank_divider_trace(
    state: &mut GameState,
    vblanks: u32,
    trace: &RuntimeDividerTrace,
) -> Result<()> {
    if trace.samples.len() % 2 != 0 {
        anyhow::bail!(
            "VBlank_Normal divider trace has odd sample count {}",
            trace.samples.len()
        );
    }
    let normal_vblanks = u32::try_from(trace.samples.len() / 2)
        .context("VBlank_Normal divider trace length exceeds u32")?;
    if normal_vblanks > vblanks {
        anyhow::bail!(
            "VBlank_Normal divider trace has {normal_vblanks} samples for only {vblanks} elapsed VBlanks"
        );
    }
    let mut divider = ReplayDivider::new(trace.samples.iter().copied());
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    for _ in 0..normal_vblanks {
        rng.random(false)
            .map_err(|error| anyhow::anyhow!("replay VBlank_Normal Random: {error}"))?;
    }
    state.random_state = rng.state();
    state.vblank_counter = state.vblank_counter.wrapping_add(normal_vblanks as u8);
    drop(rng);
    require_consumed_divider_trace("VBlank_Normal", &divider)
}

fn can_encounter_on_any_non_ice_land(environment: &str) -> bool {
    matches!(environment, "CAVE" | "DUNGEON")
}

fn decoration_display_name(decoration: &str) -> Result<&'static str> {
    let name = match decoration {
        "DECO_FAMICOM" => "NES",
        "DECO_SNES" => "SUPER NES",
        "DECO_N64" => "NINTENDO 64",
        "DECO_VIRTUAL_BOY" => "VIRTUAL BOY",
        "DECO_PIKACHU_DOLL" => "PIKACHU DOLL",
        "DECO_SURF_PIKACHU_DOLL" => "SURF PIKACHU DOLL",
        "DECO_CLEFAIRY_DOLL" => "CLEFAIRY DOLL",
        "DECO_JIGGLYPUFF_DOLL" => "JIGGLYPUFF DOLL",
        "DECO_BULBASAUR_DOLL" => "BULBASAUR DOLL",
        "DECO_CHARMANDER_DOLL" => "CHARMANDER DOLL",
        "DECO_SQUIRTLE_DOLL" => "SQUIRTLE DOLL",
        "DECO_POLIWAG_DOLL" => "POLIWAG DOLL",
        "DECO_DIGLETT_DOLL" => "DIGLETT DOLL",
        "DECO_STARYU_DOLL" => "STARYU DOLL",
        "DECO_MAGIKARP_DOLL" => "MAGIKARP DOLL",
        "DECO_ODDISH_DOLL" => "ODDISH DOLL",
        "DECO_GENGAR_DOLL" => "GENGAR DOLL",
        "DECO_SHELLDER_DOLL" => "SHELLDER DOLL",
        "DECO_GRIMER_DOLL" => "GRIMER DOLL",
        "DECO_VOLTORB_DOLL" => "VOLTORB DOLL",
        "DECO_WEEDLE_DOLL" => "WEEDLE DOLL",
        "DECO_UNOWN_DOLL" => "UNOWN DOLL",
        "DECO_GEODUDE_DOLL" => "GEODUDE DOLL",
        "DECO_MACHOP_DOLL" => "MACHOP DOLL",
        "DECO_TENTACOOL_DOLL" => "TENTACOOL DOLL",
        "DECO_GOLD_TROPHY_DOLL" => "GOLD TROPHY",
        "DECO_SILVER_TROPHY_DOLL" => "SILVER TROPHY",
        other => anyhow::bail!("unknown equipped ornament or console decoration {other}"),
    };
    Ok(name)
}

fn decoration_description_scripts() -> BTreeMap<String, Value> {
    [
        (
            "DecorationDesc_TownMapPoster",
            serde_json::json!([
                {"command": "opentext", "args": []},
                {"command": "farwritetext", "args": ["_LookTownMapText"]},
                {"command": "waitbutton", "args": []},
                {"command": "special", "args": ["OverworldTownMap"]},
                {"command": "closetext", "args": []},
                {"command": "end", "args": []}
            ]),
        ),
        (
            "DecorationDesc_PikachuPoster",
            serde_json::json!([{"command": "farjumptext", "args": ["_LookPikachuPosterText"]}]),
        ),
        (
            "DecorationDesc_ClefairyPoster",
            serde_json::json!([{"command": "farjumptext", "args": ["_LookClefairyPosterText"]}]),
        ),
        (
            "DecorationDesc_JigglypuffPoster",
            serde_json::json!([{"command": "farjumptext", "args": ["_LookJigglypuffPosterText"]}]),
        ),
        (
            "DecorationDesc_NullPoster",
            serde_json::json!([{"command": "end", "args": []}]),
        ),
        (
            ".OrnamentConsoleScript@DecorationDesc_OrnamentOrConsole",
            serde_json::json!([{"command": "farjumptext", "args": ["_LookAdorableDecoText"]}]),
        ),
        (
            ".BigDollScript@DecorationDesc_GiantOrnament",
            serde_json::json!([{"command": "farjumptext", "args": ["_LookGiantDecoText"]}]),
        ),
    ]
    .into_iter()
    .map(|(label, body)| (label.to_string(), body))
    .collect()
}

fn consume_enemy_ai_divider_trace(
    state: &mut GameState,
    trace: &RuntimeDividerTrace,
) -> Result<()> {
    if trace.samples.len() % 2 != 0 {
        anyhow::bail!(
            "enemy battle AI divider trace has odd sample count {}",
            trace.samples.len()
        );
    }
    let mut divider = ReplayDivider::new(trace.samples.iter().copied());
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    for _ in 0..(trace.samples.len() / 2) {
        rng.battle_random()
            .map_err(|error| anyhow::anyhow!("replay enemy battle AI Random: {error}"))?;
    }
    state.random_state = rng.state();
    drop(rng);
    require_consumed_divider_trace("enemy battle AI", &divider)
}

fn generate_scripted_gift_dvs<S>(
    state: &mut GameState,
    divider: &mut S,
) -> std::result::Result<Dv, S::Error>
where
    S: DividerSource + ?Sized,
{
    // GeneratePartyMonStats calls Random twice. `ld b, a` between the calls
    // does not alter flags, so the second call consumes the first SBC carry.
    let mut rng = CrystalRandom::new(state.random_state, divider);
    let attack_defense = rng.random(false)?;
    let speed_special = rng.random(attack_defense.carry_out)?;
    state.random_state = rng.state();
    Ok(Dv::from_non_hp(
        attack_defense.value >> 4,
        attack_defense.value & 0x0f,
        speed_special.value >> 4,
        speed_special.value & 0x0f,
    ))
}

fn generate_scripted_box_gift_ot_id<S>(
    state: &mut GameState,
    divider: &mut S,
) -> std::result::Result<u16, S::Error>
where
    S: DividerSource + ?Sized,
{
    // GivePoke's custom-OT box branch calls Random twice consecutively after
    // the name terminator comparison has cleared carry.
    let mut rng = CrystalRandom::new(state.random_state, divider);
    let high = rng.random(false)?;
    let low = rng.random(high.carry_out)?;
    state.random_state = rng.state();
    Ok(u16::from_be_bytes([high.value, low.value]))
}

fn runtime_game_timer_outcome(state: &GameState, counted: bool) -> RuntimeGameTimerOutcome {
    RuntimeGameTimerOutcome {
        counted,
        counting: state.game_timer_counting,
        logic_paused: state.game_logic_paused,
        hours: state.time.game_time_hours,
        minutes: state.time.game_time_minutes,
        seconds: state.time.game_time_seconds,
        frames: state.time.game_time_frames,
    }
}

fn grass_rustle_duration_for_speed(speed_multiplier: u8) -> Result<u8> {
    let source_step_duration = match speed_multiplier {
        1 => 8_u8,
        2 => 4_u8,
        speed => anyhow::bail!(
            "player grass step has unsupported source speed multiplier {speed}"
        ),
    };
    source_step_duration
        .checked_sub(2)
        .context("player grass step is too short for same-frame tracking-object decrements")
}

fn crystal_days_since(previous_day: u8, current_day: u8) -> u8 {
    let (elapsed, borrowed) = current_day.overflowing_sub(previous_day);
    if borrowed {
        elapsed.wrapping_add(20 * 7)
    } else {
        elapsed
    }
}

fn apply_elapsed_daily_time_events<S>(
    state: &mut GameState,
    elapsed_days: u8,
    divider: &mut S,
    context: &str,
) -> Result<()>
where
    S: DividerSource + ?Sized,
    S::Error: std::fmt::Display,
{
    if elapsed_days == 0 {
        return Ok(());
    }
    // CheckDailyResetTimer owns one one-day countdown. Even after a long
    // process pause it expires and restarts once at the current wCurDay;
    // it does not replay every missed midnight (or resample Kenji for each).
    state
        .apply_daily_reset(divider)
        .map_err(|error| anyhow::anyhow!("{context}: {error}"))?;
    // CheckPokerusTick is separate and applies the complete wDaysSince value.
    // apply_daily_reset already covered the first elapsed day.
    state.apply_pokerus_tick(elapsed_days.saturating_sub(1));
    Ok(())
}

include!("game_data_scripts.rs");
include!("game_data_mutations.rs");
include!("game_data_overworld_and_saves.rs");
include!("game_data_items_and_battles.rs");
include!("game_data_encounters_and_loading.rs");

/// `CheckTileEvent` dispatches pits through `FallIntoMapScript`; every other
/// player-triggered warp uses `WarpToNewMapScript`.
fn player_event_warp_map_setup(permission: u8) -> &'static str {
    if matches!(permission, permissions::PIT | permissions::PIT_68) {
        "MAPSETUP_FALL"
    } else {
        "MAPSETUP_DOOR"
    }
}

fn prepare_bug_contest_results_warp(state: &mut GameState) -> Result<()> {
    for index in 1..=10 {
        let flag_a = format!("EVENT_BUG_CATCHING_CONTESTANT_{index}A");
        let flag_b = format!("EVENT_BUG_CATCHING_CONTESTANT_{index}B");
        let selected = state
            .flags
            .is_event_flag_set(&flag_a)
            .map_err(|error| anyhow::anyhow!("read Bug Contest contestant flag: {error}"))?;
        if !selected {
            state
                .flags
                .set_event_flag(&flag_b, false)
                .map_err(|error| anyhow::anyhow!("clear Bug Contest contestant flag: {error}"))?;
        }
    }
    state
        .flags
        .set_event_flag(
            "EVENT_ROUTE_36_NATIONAL_PARK_GATE_OFFICER_CONTEST_DAY",
            true,
        )
        .map_err(|error| anyhow::anyhow!("set Bug Contest result flag: {error}"))?;
    state
        .flags
        .set_event_flag(
            "EVENT_ROUTE_36_NATIONAL_PARK_GATE_OFFICER_NOT_CONTEST_DAY",
            false,
        )
        .map_err(|error| anyhow::anyhow!("clear Bug Contest result flag: {error}"))?;
    state
        .flags
        .set_event_flag("EVENT_WARPED_FROM_ROUTE_35_NATIONAL_PARK_GATE", true)
        .map_err(|error| anyhow::anyhow!("set Bug Contest warp flag: {error}"))?;
    Ok(())
}

fn resolve_background_event_script_target(
    scripts: &BTreeMap<String, serde_json::Value>,
    source_script: &str,
    target: &str,
) -> Result<String> {
    if !target.starts_with('.') {
        if scripts.contains_key(target) {
            return Ok(target.to_string());
        }
        anyhow::bail!(
            "conditional background event {} targets missing script {}",
            source_script,
            target
        );
    }

    let parent = script_label_parent(source_script);
    let local = if target.contains('@') {
        anyhow::ensure!(
            script_label_parent(target) == parent,
            "conditional background event {} targets local script {} outside parent {}",
            source_script,
            target,
            parent
        );
        target.to_string()
    } else {
        format!("{target}@{parent}")
    };
    if scripts.contains_key(&local) {
        Ok(local)
    } else {
        anyhow::bail!(
            "conditional background event {} targets unresolved local script {}",
            source_script,
            target
        )
    }
}

fn conditional_background_event_payload<'a>(
    scripts: &'a BTreeMap<String, serde_json::Value>,
    source_script: &str,
    event_type: &str,
) -> Result<[&'a str; 2]> {
    let commands = scripts
        .get(source_script)
        .and_then(serde_json::Value::as_array)
        .with_context(|| {
            format!("{event_type} background event {source_script} has no script data payload")
        })?;
    let mut matching = commands.iter().filter(|command| {
        command.get("command").and_then(serde_json::Value::as_str) == Some("conditional_event")
    });
    let command = matching.next().with_context(|| {
        format!("{event_type} background event {source_script} has no conditional_event payload")
    })?;
    anyhow::ensure!(
        matching.next().is_none(),
        "{event_type} background event {source_script} has multiple conditional_event payloads"
    );
    let args = command
        .get("args")
        .and_then(serde_json::Value::as_array)
        .with_context(|| {
            format!(
                "{event_type} background event {source_script} conditional_event args are not an array"
            )
        })?;
    anyhow::ensure!(
        args.len() == 2,
        "{event_type} background event {source_script} conditional_event has {} args instead of 2",
        args.len()
    );
    let parse_arg = |index: usize| -> Result<&str> {
        let value = args[index].as_str().with_context(|| {
            format!(
                "{event_type} background event {source_script} conditional_event arg {index} is not a string"
            )
        })?;
        anyhow::ensure!(
            !value.is_empty() && value.trim() == value && !value.chars().any(char::is_control),
            "{event_type} background event {source_script} conditional_event arg {index} is not an exact non-empty token"
        );
        Ok(value)
    };
    Ok([parse_arg(0)?, parse_arg(1)?])
}

fn callasm_indirect_symbol(operand: &str) -> Option<&str> {
    operand
        .strip_prefix('[')
        .and_then(|operand| operand.strip_suffix(']'))
        .filter(|symbol| !symbol.is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchingCallasmHostService {
    CheckPartyMove,
    CheckEngineFlag,
    GetPartyNickname,
    GetNickname,
    CopyName1,
    CopyName2,
    UpdateSprites,
    UpdatePlayerSprite,
    CheckWaterfallTile,
    StubbedTrainerRankingsWaterfall,
}

impl BranchingCallasmHostService {
    fn from_label(label: &str) -> Option<Self> {
        match label {
            "CheckPartyMove" => Some(Self::CheckPartyMove),
            "CheckEngineFlag" => Some(Self::CheckEngineFlag),
            "GetPartyNickname" => Some(Self::GetPartyNickname),
            "GetNickname" => Some(Self::GetNickname),
            "CopyName1" => Some(Self::CopyName1),
            "CopyName2" => Some(Self::CopyName2),
            "UpdateSprites" => Some(Self::UpdateSprites),
            "UpdatePlayerSprite" => Some(Self::UpdatePlayerSprite),
            "CheckWaterfallTile" => Some(Self::CheckWaterfallTile),
            "StubbedTrainerRankings_Waterfall" => Some(Self::StubbedTrainerRankingsWaterfall),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchingCallasmInstruction<'a> {
    Load {
        destination: &'a str,
        source: &'a str,
    },
    Call {
        service: BranchingCallasmHostService,
    },
    Jump {
        condition: Option<ScriptRuntimeCpuCondition>,
        target: &'a str,
    },
    Bit {
        bit: &'a str,
    },
    Set {
        bit: &'a str,
    },
    AddHlDe,
    XorA,
    Return {
        condition: Option<ScriptRuntimeCpuCondition>,
    },
}

fn classify_branching_callasm_instruction<'a>(
    command: &str,
    args: &'a [&'a str],
) -> Option<BranchingCallasmInstruction<'a>> {
    match command {
        "ld" | "ldh" if args.len() == 2 => {
            let destination = args[0];
            (matches!(destination, "a" | "d" | "e" | "de" | "hl")
                || callasm_indirect_symbol(destination).is_some())
            .then_some(BranchingCallasmInstruction::Load {
                destination,
                source: args[1],
            })
        }
        "call" | "farcall" if args.len() == 1 => BranchingCallasmHostService::from_label(args[0])
            .map(|service| BranchingCallasmInstruction::Call { service }),
        "jr" if args.len() == 1 => Some(BranchingCallasmInstruction::Jump {
            condition: None,
            target: args[0],
        }),
        "jr" if args.len() == 2 => Some(BranchingCallasmInstruction::Jump {
            condition: ScriptRuntimeCpuCondition::from_asm_token(args[0]),
            target: args[1],
        })
        .filter(|instruction| {
            matches!(
                instruction,
                BranchingCallasmInstruction::Jump {
                    condition: Some(_),
                    ..
                }
            )
        }),
        "bit" if args.len() == 2 && args[1] == "[hl]" => {
            Some(BranchingCallasmInstruction::Bit { bit: args[0] })
        }
        "set" if args.len() == 2 && args[1] == "[hl]" => {
            Some(BranchingCallasmInstruction::Set { bit: args[0] })
        }
        "add" if args == ["hl", "de"] => Some(BranchingCallasmInstruction::AddHlDe),
        "xor" if args == ["a"] => Some(BranchingCallasmInstruction::XorA),
        "ret" if args.is_empty() => Some(BranchingCallasmInstruction::Return { condition: None }),
        "ret" if args.len() == 1 => {
            ScriptRuntimeCpuCondition::from_asm_token(args[0]).map(|condition| {
                BranchingCallasmInstruction::Return {
                    condition: Some(condition),
                }
            })
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccumulatorCallasmAluOperation {
    Compare,
    And,
    Or,
    Xor,
    Subtract,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccumulatorCallasmInstruction<'a> {
    LoadA {
        source: &'a str,
    },
    StoreA {
        destination: &'a str,
    },
    Alu {
        operation: AccumulatorCallasmAluOperation,
        operand: &'a str,
    },
    IncrementA,
    DecrementA,
    SetCarry,
    ComplementCarry,
    PushAf,
    PopAf,
    Return {
        condition: Option<ScriptRuntimeCpuCondition>,
    },
}

fn classify_accumulator_callasm_instruction<'a>(
    command: &str,
    args: &'a [&'a str],
) -> Option<AccumulatorCallasmInstruction<'a>> {
    match command {
        "ld" | "ldh" if args.len() == 2 && args[0] == "a" => {
            Some(AccumulatorCallasmInstruction::LoadA { source: args[1] })
        }
        "ld" | "ldh"
            if args.len() == 2
                && args[1] == "a"
                && callasm_indirect_symbol(args[0])
                    .is_some_and(|destination| matches!(destination, "rWBK" | "wScriptVar")) =>
        {
            Some(AccumulatorCallasmInstruction::StoreA {
                destination: callasm_indirect_symbol(args[0])
                    .expect("classified indirect callasm destination"),
            })
        }
        "cp" | "and" | "or" | "xor" | "sub" | "add" if args.len() == 1 => {
            let operation = match command {
                "cp" => AccumulatorCallasmAluOperation::Compare,
                "and" => AccumulatorCallasmAluOperation::And,
                "or" => AccumulatorCallasmAluOperation::Or,
                "xor" => AccumulatorCallasmAluOperation::Xor,
                "sub" => AccumulatorCallasmAluOperation::Subtract,
                "add" => AccumulatorCallasmAluOperation::Add,
                _ => unreachable!("matched accumulator ALU instruction"),
            };
            Some(AccumulatorCallasmInstruction::Alu {
                operation,
                operand: args[0],
            })
        }
        "inc" if args == ["a"] => Some(AccumulatorCallasmInstruction::IncrementA),
        "dec" if args == ["a"] => Some(AccumulatorCallasmInstruction::DecrementA),
        "scf" if args.is_empty() => Some(AccumulatorCallasmInstruction::SetCarry),
        "ccf" if args.is_empty() => Some(AccumulatorCallasmInstruction::ComplementCarry),
        "push" if args == ["af"] => Some(AccumulatorCallasmInstruction::PushAf),
        "pop" if args == ["af"] => Some(AccumulatorCallasmInstruction::PopAf),
        "ret" if args.is_empty() => Some(AccumulatorCallasmInstruction::Return { condition: None }),
        "ret" if args.len() == 1 => {
            ScriptRuntimeCpuCondition::from_asm_token(args[0]).map(|condition| {
                AccumulatorCallasmInstruction::Return {
                    condition: Some(condition),
                }
            })
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptCallasmCertificateFailure {
    pub(crate) target_script: String,
    pub(crate) command_index: usize,
    pub(crate) command: String,
    pub(crate) reason: String,
}

impl ScriptCallasmCertificateFailure {
    fn at(
        target_script: &str,
        command_index: usize,
        command: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            target_script: target_script.to_string(),
            command_index,
            command: command.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BranchingCallasmCertificateState {
    registers: BTreeMap<String, String>,
    known_memory: BTreeSet<String>,
    zero_flag_defined: bool,
    carry_flag_defined: bool,
    selected_party_source: bool,
    party_move_selection_pending: bool,
}

impl Default for BranchingCallasmCertificateState {
    fn default() -> Self {
        Self {
            registers: BTreeMap::new(),
            known_memory: BTreeSet::new(),
            zero_flag_defined: false,
            carry_flag_defined: false,
            selected_party_source: false,
            party_move_selection_pending: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AccumulatorCallasmCertificateState {
    register_a: AccumulatorCallasmValue,
    zero_flag_defined: bool,
    carry_flag_defined: bool,
    af_stack: Vec<(AccumulatorCallasmValue, bool, bool)>,
    scratch_memory: BTreeMap<String, AccumulatorCallasmValue>,
    script_value_written: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccumulatorCallasmValue {
    Unresolved,
    ConcreteByte,
    OpaqueBankByte,
}

impl Default for AccumulatorCallasmCertificateState {
    fn default() -> Self {
        Self {
            register_a: AccumulatorCallasmValue::Unresolved,
            zero_flag_defined: false,
            carry_flag_defined: false,
            af_stack: Vec::new(),
            scratch_memory: BTreeMap::new(),
            script_value_written: false,
        }
    }
}

fn callasm_certificate_entry<'a>(
    definitions: &'a BTreeMap<String, Value>,
    label: &str,
    command_index: usize,
) -> std::result::Result<(&'a str, Vec<&'a str>), ScriptCallasmCertificateFailure> {
    let Some(entries) = definitions.get(label).and_then(Value::as_array) else {
        return Err(ScriptCallasmCertificateFailure::at(
            label,
            command_index,
            "<missing>",
            "routine body is missing or is not a command array",
        ));
    };
    let Some(entry) = entries.get(command_index) else {
        return Err(ScriptCallasmCertificateFailure::at(
            label,
            command_index,
            "<fallthrough>",
            "reachable control flow falls off the routine without returning",
        ));
    };
    let command = entry
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ScriptCallasmCertificateFailure::at(
                label,
                command_index,
                "<missing>",
                "reachable command has no exact command name",
            )
        })?;
    let args = entry
        .get("args")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ScriptCallasmCertificateFailure::at(
                label,
                command_index,
                command,
                "reachable command has no argument array",
            )
        })?
        .iter()
        .enumerate()
        .map(|(arg_index, arg)| {
            arg.as_str().ok_or_else(|| {
                ScriptCallasmCertificateFailure::at(
                    label,
                    command_index,
                    command,
                    format!("argument {arg_index} is not an exact string"),
                )
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((command, args))
}

fn branching_callasm_condition_is_defined(
    state: &BranchingCallasmCertificateState,
    condition: ScriptRuntimeCpuCondition,
) -> bool {
    match condition {
        ScriptRuntimeCpuCondition::Z | ScriptRuntimeCpuCondition::Nz => state.zero_flag_defined,
        ScriptRuntimeCpuCondition::C | ScriptRuntimeCpuCondition::Nc => state.carry_flag_defined,
    }
}

fn branching_callasm_source_is_resolvable(
    state: &BranchingCallasmCertificateState,
    source: &str,
) -> bool {
    if matches!(source, "a" | "d" | "e" | "de" | "hl") {
        return state.registers.contains_key(source);
    }
    if let Some(symbol) = callasm_indirect_symbol(source) {
        if symbol == "hl" {
            return state.registers.get("hl").is_some_and(|address| {
                address.starts_with("wPartySpecies+") && state.selected_party_source
            });
        }
        return symbol == "wCurPartyMon" || state.known_memory.contains(symbol);
    }
    true
}

fn refine_branching_callasm_party_selection(
    state: &mut BranchingCallasmCertificateState,
    condition: ScriptRuntimeCpuCondition,
    condition_is_met: bool,
) {
    if !state.party_move_selection_pending {
        return;
    }
    let carry = match condition {
        ScriptRuntimeCpuCondition::C => Some(condition_is_met),
        ScriptRuntimeCpuCondition::Nc => Some(!condition_is_met),
        ScriptRuntimeCpuCondition::Z | ScriptRuntimeCpuCondition::Nz => None,
    };
    if let Some(carry) = carry {
        state.selected_party_source = !carry;
        state.party_move_selection_pending = false;
    }
}

fn certify_branching_script_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    let mut pending = vec![(
        start_label.to_string(),
        0usize,
        BranchingCallasmCertificateState::default(),
        BTreeSet::<(String, usize)>::new(),
    )];
    let mut saw_return = false;
    while let Some((label, command_index, mut state, mut path)) = pending.pop() {
        if !path.insert((label.clone(), command_index)) {
            return Err(ScriptCallasmCertificateFailure::at(
                &label,
                command_index,
                "<cycle>",
                "reachable CPU control flow contains a cycle that cannot be proven to return",
            ));
        }
        let (command, args) = callasm_certificate_entry(definitions, &label, command_index)?;
        let Some(instruction) = classify_branching_callasm_instruction(command, &args) else {
            let reason = if command == "call" && args.len() == 1 {
                format!(
                    "host call '{}' has no synchronous Rust implementation",
                    args[0]
                )
            } else {
                "instruction or operand shape is unsupported by the branching interpreter"
                    .to_string()
            };
            return Err(ScriptCallasmCertificateFailure::at(
                &label,
                command_index,
                command,
                reason,
            ));
        };
        let mut enqueue_next = true;
        match instruction {
            BranchingCallasmInstruction::Load {
                destination,
                source,
            } => {
                if !branching_callasm_source_is_resolvable(&state, source) {
                    return Err(ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        format!("source operand '{source}' is not initialized or resolvable"),
                    ));
                }
                let value = state
                    .registers
                    .get(source)
                    .cloned()
                    .unwrap_or_else(|| source.to_string());
                if callasm_indirect_symbol(source) == Some("wCurPartyMon") {
                    state.selected_party_source = true;
                    state.party_move_selection_pending = false;
                }
                if matches!(destination, "a" | "d" | "e" | "de" | "hl") {
                    state.registers.insert(destination.to_string(), value);
                } else if callasm_indirect_symbol(destination) == Some("hl") {
                    if !state.registers.contains_key("hl") {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "destination [hl] is not initialized",
                        ));
                    }
                } else if let Some(destination) = callasm_indirect_symbol(destination) {
                    state.known_memory.insert(destination.to_string());
                }
            }
            BranchingCallasmInstruction::Call { service } => match service {
                BranchingCallasmHostService::CheckPartyMove => {
                    if !state.registers.contains_key("d") {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "CheckPartyMove requires initialized register d",
                        ));
                    }
                    state.zero_flag_defined = true;
                    state.carry_flag_defined = true;
                    state.selected_party_source = false;
                    state.party_move_selection_pending = true;
                    state
                        .registers
                        .insert("a".to_string(), "<party-move-result>".to_string());
                    state
                        .registers
                        .insert("e".to_string(), "<party-scan-index>".to_string());
                }
                BranchingCallasmHostService::CheckEngineFlag => {
                    if !state.registers.contains_key("de") {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "CheckEngineFlag requires initialized register de",
                        ));
                    }
                    state.zero_flag_defined = true;
                    state.carry_flag_defined = true;
                    state.party_move_selection_pending = false;
                    state
                        .registers
                        .insert("a".to_string(), "<engine-flag>".to_string());
                }
                BranchingCallasmHostService::GetPartyNickname => {
                    if !state.selected_party_source {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "nickname host call has no provable selected-party source",
                        ));
                    }
                }
                BranchingCallasmHostService::GetNickname => {
                    if !state.selected_party_source
                        || state.registers.get("a").is_none()
                        || state.registers.get("hl").map(String::as_str)
                            != Some("wPartyMonNicknames")
                    {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "GetNickname requires a selected-party index in a and hl = wPartyMonNicknames",
                        ));
                    }
                    state
                        .registers
                        .insert("de".to_string(), "wStringBuffer1".to_string());
                }
                BranchingCallasmHostService::CopyName1 => {
                    if state.registers.get("de").map(String::as_str) != Some("wStringBuffer1") {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "CopyName1 requires de = wStringBuffer1 from GetNickname",
                        ));
                    }
                }
                BranchingCallasmHostService::CopyName2 => {
                    if state.registers.get("de").map(String::as_str) != Some("wStringBuffer2")
                        || state.registers.get("hl").map(String::as_str) != Some("wStringBuffer3")
                    {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "CopyName2 requires de = wStringBuffer2 and hl = wStringBuffer3",
                        ));
                    }
                }
                BranchingCallasmHostService::UpdateSprites
                | BranchingCallasmHostService::UpdatePlayerSprite => {}
                BranchingCallasmHostService::CheckWaterfallTile => {
                    if !state.registers.contains_key("a") {
                        return Err(ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            "CheckWaterfallTile requires initialized register a",
                        ));
                    }
                    state.zero_flag_defined = true;
                }
                BranchingCallasmHostService::StubbedTrainerRankingsWaterfall => {}
            },
            BranchingCallasmInstruction::Jump { condition, target } => {
                if let Some(condition) = condition
                    && !branching_callasm_condition_is_defined(&state, condition)
                {
                    return Err(ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        format!("condition {condition:?} reads an undefined CPU flag"),
                    ));
                }
                let target =
                    resolve_script_target_label(definitions, &label, target).ok_or_else(|| {
                        ScriptCallasmCertificateFailure::at(
                            &label,
                            command_index,
                            command,
                            format!("jump target '{target}' is missing"),
                        )
                    })?;
                let mut target_state = state.clone();
                if let Some(condition) = condition {
                    refine_branching_callasm_party_selection(&mut target_state, condition, true);
                    refine_branching_callasm_party_selection(&mut state, condition, false);
                }
                pending.push((target, 0, target_state, path.clone()));
                enqueue_next = condition.is_some();
            }
            BranchingCallasmInstruction::Bit { bit } => {
                let memory = state.registers.get("hl").ok_or_else(|| {
                    ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        "bit [hl] requires initialized register hl",
                    )
                })?;
                if branching_callasm_bit_effect(memory, bit).is_none() {
                    return Err(ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        format!("bit {bit}, [hl] cannot resolve runtime memory '{memory}'"),
                    ));
                }
                state.zero_flag_defined = true;
            }
            BranchingCallasmInstruction::Set { bit } => {
                let memory = state.registers.get("hl").ok_or_else(|| {
                    ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        "set [hl] requires initialized register hl",
                    )
                })?;
                if branching_callasm_bit_effect(memory, bit).is_none() {
                    return Err(ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        format!("set {bit}, [hl] cannot resolve runtime memory '{memory}'"),
                    ));
                }
            }
            BranchingCallasmInstruction::AddHlDe => {
                if !state.registers.contains_key("hl")
                    || !(state.registers.contains_key("de")
                        || (state.registers.contains_key("d") && state.registers.contains_key("e")))
                {
                    return Err(ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        "add hl, de requires initialized hl and de operands",
                    ));
                }
                let base = state
                    .registers
                    .get("hl")
                    .expect("certified initialized hl")
                    .clone();
                state
                    .registers
                    .insert("hl".to_string(), format!("{base}+<de>"));
            }
            BranchingCallasmInstruction::XorA => {
                state.registers.insert("a".to_string(), "0".to_string());
                state.zero_flag_defined = true;
                state.carry_flag_defined = true;
                state.party_move_selection_pending = false;
            }
            BranchingCallasmInstruction::Return { condition } => {
                if let Some(condition) = condition
                    && !branching_callasm_condition_is_defined(&state, condition)
                {
                    return Err(ScriptCallasmCertificateFailure::at(
                        &label,
                        command_index,
                        command,
                        format!("condition {condition:?} reads an undefined CPU flag"),
                    ));
                }
                saw_return = true;
                if let Some(condition) = condition {
                    refine_branching_callasm_party_selection(&mut state, condition, false);
                }
                enqueue_next = condition.is_some();
            }
        }
        if enqueue_next {
            pending.push((label, command_index + 1, state, path));
        }
    }
    saw_return.then_some(()).ok_or_else(|| {
        ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<missing return>",
            "routine has no reachable return",
        )
    })
}

fn accumulator_callasm_operand_resolution(
    state: &AccumulatorCallasmCertificateState,
    operand: &str,
) -> Option<AccumulatorCallasmValue> {
    if operand == "a" {
        return Some(state.register_a);
    }
    if let Some(symbol) = callasm_indirect_symbol(operand) {
        if let Some(resolved) = state.scratch_memory.get(symbol) {
            return Some(*resolved);
        }
        return match symbol {
            // The runtime deliberately carries the concrete WRAM bank as
            // `None`: it may be preserved through AF/rWBK, but it cannot be
            // consumed as a game byte or exposed through wScriptVar.
            "rWBK" => Some(AccumulatorCallasmValue::OpaqueBankByte),
            "wBTChoiceOfLvlGroup" => Some(AccumulatorCallasmValue::ConcreteByte),
            "wScriptVar" if state.script_value_written => {
                Some(AccumulatorCallasmValue::ConcreteByte)
            }
            _ => Some(AccumulatorCallasmValue::Unresolved),
        };
    }
    if operand.starts_with("BANK(") && operand.ends_with(')') {
        return Some(AccumulatorCallasmValue::OpaqueBankByte);
    }
    parse_script_i32(operand)
        .ok()
        .map(|_| AccumulatorCallasmValue::ConcreteByte)
}

fn accumulator_callasm_condition_is_defined(
    state: &AccumulatorCallasmCertificateState,
    condition: ScriptRuntimeCpuCondition,
) -> bool {
    match condition {
        ScriptRuntimeCpuCondition::Z | ScriptRuntimeCpuCondition::Nz => state.zero_flag_defined,
        ScriptRuntimeCpuCondition::C | ScriptRuntimeCpuCondition::Nc => state.carry_flag_defined,
    }
}

fn certify_accumulator_script_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    let mut state = AccumulatorCallasmCertificateState::default();
    let mut command_index = 0usize;
    loop {
        let (command, args) = callasm_certificate_entry(definitions, start_label, command_index)?;
        let Some(instruction) = classify_accumulator_callasm_instruction(command, &args) else {
            let reason = if matches!(command, "ld" | "ldh")
                && args.len() == 2
                && args[1] == "a"
                && let Some(destination) = callasm_indirect_symbol(args[0])
            {
                format!(
                    "destination '[{destination}]' is not a modeled persistent accumulator side effect"
                )
            } else {
                "instruction or operand shape is unsupported by the accumulator interpreter"
                    .to_string()
            };
            return Err(ScriptCallasmCertificateFailure::at(
                start_label,
                command_index,
                command,
                reason,
            ));
        };
        match instruction {
            AccumulatorCallasmInstruction::LoadA { source } => {
                let Some(value) = accumulator_callasm_operand_resolution(&state, source) else {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        format!("source operand '{source}' cannot resolve to a byte"),
                    ));
                };
                if value == AccumulatorCallasmValue::Unresolved {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        format!("source operand '{source}' cannot resolve to a byte"),
                    ));
                }
                state.register_a = value;
            }
            AccumulatorCallasmInstruction::StoreA { destination } => {
                if destination == "rWBK" && state.register_a == AccumulatorCallasmValue::Unresolved
                {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "rWBK write uses an unresolved accumulator byte",
                    ));
                }
                if destination == "wScriptVar"
                    && state.register_a != AccumulatorCallasmValue::ConcreteByte
                {
                    let kind = match state.register_a {
                        AccumulatorCallasmValue::OpaqueBankByte => "an opaque bank byte",
                        AccumulatorCallasmValue::Unresolved => "an unresolved byte",
                        AccumulatorCallasmValue::ConcreteByte => {
                            unreachable!("checked non-concrete accumulator")
                        }
                    };
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        format!("wScriptVar write uses {kind}"),
                    ));
                }
                state
                    .scratch_memory
                    .insert(destination.to_string(), state.register_a);
                if destination == "wScriptVar" {
                    state.script_value_written = true;
                }
            }
            AccumulatorCallasmInstruction::Alu {
                operation: _,
                operand,
            } => {
                if state.register_a != AccumulatorCallasmValue::ConcreteByte
                    || accumulator_callasm_operand_resolution(&state, operand)
                        != Some(AccumulatorCallasmValue::ConcreteByte)
                {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        format!("ALU operand '{operand}' or accumulator byte is unresolved"),
                    ));
                }
                state.zero_flag_defined = true;
                // CP, AND, OR, XOR, SUB, and ADD all assign the carry flag.
                state.carry_flag_defined = true;
            }
            AccumulatorCallasmInstruction::IncrementA
            | AccumulatorCallasmInstruction::DecrementA => {
                if state.register_a != AccumulatorCallasmValue::ConcreteByte {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "increment/decrement uses an unresolved accumulator byte",
                    ));
                }
                state.zero_flag_defined = true;
            }
            AccumulatorCallasmInstruction::SetCarry => state.carry_flag_defined = true,
            AccumulatorCallasmInstruction::ComplementCarry => {
                if !state.carry_flag_defined {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "ccf reads an undefined carry flag",
                    ));
                }
            }
            AccumulatorCallasmInstruction::PushAf => {
                if state.register_a == AccumulatorCallasmValue::Unresolved {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "push af reads an unresolved accumulator byte",
                    ));
                }
                state.af_stack.push((
                    state.register_a,
                    state.zero_flag_defined,
                    state.carry_flag_defined,
                ));
            }
            AccumulatorCallasmInstruction::PopAf => {
                let Some((register_a, zero_flag_defined, carry_flag_defined)) =
                    state.af_stack.pop()
                else {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "pop af reads an empty stack",
                    ));
                };
                state.register_a = register_a;
                state.zero_flag_defined = zero_flag_defined;
                state.carry_flag_defined = carry_flag_defined;
            }
            AccumulatorCallasmInstruction::Return { condition } => {
                if let Some(condition) = condition
                    && !accumulator_callasm_condition_is_defined(&state, condition)
                {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        format!("condition {condition:?} reads an undefined CPU flag"),
                    ));
                }
                if !state.script_value_written {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "reachable return has not written a resolved wScriptVar byte",
                    ));
                }
                if !state.af_stack.is_empty() {
                    return Err(ScriptCallasmCertificateFailure::at(
                        start_label,
                        command_index,
                        command,
                        "reachable return leaves the modeled AF stack unbalanced",
                    ));
                }
                if condition.is_none() {
                    return Ok(());
                }
            }
        }
        command_index = command_index.checked_add(1).ok_or_else(|| {
            ScriptCallasmCertificateFailure::at(
                start_label,
                command_index,
                command,
                "routine command index overflowed",
            )
        })?;
    }
}

fn certify_exact_callasm_body(
    definitions: &BTreeMap<String, Value>,
    label: &str,
    expected: &[(&str, &[&str])],
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    let Some(entries) = definitions.get(label).and_then(Value::as_array) else {
        return Err(ScriptCallasmCertificateFailure::at(
            label,
            0,
            "<missing>",
            "exact typed routine body is missing or is not a command array",
        ));
    };
    if entries.len() != expected.len() {
        return Err(ScriptCallasmCertificateFailure::at(
            label,
            entries.len().min(expected.len()),
            "<body length>",
            format!(
                "exact typed routine requires {} commands, found {}",
                expected.len(),
                entries.len()
            ),
        ));
    }
    for (command_index, (expected_command, expected_args)) in expected.iter().enumerate() {
        let (command, args) = callasm_certificate_entry(definitions, label, command_index)?;
        if command != *expected_command || args.as_slice() != *expected_args {
            return Err(ScriptCallasmCertificateFailure::at(
                label,
                command_index,
                command,
                format!(
                    "exact typed routine requires {} {:?}, found {} {:?}",
                    expected_command, expected_args, command, args
                ),
            ));
        }
    }
    Ok(())
}

pub(crate) fn certify_rock_mon_encounter_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    if start_label != "RockMonEncounter" {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<target>",
            "typed Rock encounter target must resolve exactly to RockMonEncounter",
        ));
    }
    const MAIN: &[(&str, &[&str])] = &[
        ("xor", &["a"]),
        ("ld", &["[wTempWildMonSpecies]", "a"]),
        ("ld", &["[wCurPartyLevel]", "a"]),
        ("ld", &["hl", "RockMonMaps"]),
        ("call", &["GetTreeMonSet"]),
        ("jr", &["nc", ".no_battle"]),
        ("call", &["GetTreeMons"]),
        ("jr", &["nc", ".no_battle"]),
        ("ld", &["a", "10"]),
        ("call", &["RandomRange"]),
        ("cp", &["4"]),
        ("jr", &["nc", ".no_battle"]),
        ("call", &["SelectTreeMon"]),
        ("jr", &["nc", ".no_battle"]),
        ("ret", &[]),
    ];
    const NO_BATTLE: &[(&str, &[&str])] = &[("xor", &["a"]), ("ret", &[]), ("db", &["5"])];
    const ROCK_SMASH_SCRIPT: &[(&str, &[&str])] = &[
        ("callasm", &["GetPartyNickname"]),
        ("writetext", &["UseRockSmashText"]),
        ("closetext", &[]),
        ("special", &["WaitSFX"]),
        ("playsound", &["SFX_STRENGTH"]),
        ("earthquake", &["84"]),
        ("applymovementlasttalked", &["MovementData_RockSmash"]),
        ("disappear", &["LAST_TALKED"]),
        ("callasm", &["RockMonEncounter"]),
        ("readmem", &["wTempWildMonSpecies"]),
        ("iffalse", &[".done"]),
        ("randomwildmon", &[]),
        ("startbattle", &[]),
        ("reloadmapafterbattle", &[]),
        ("end", &[]),
    ];
    const FROM_MENU: &[(&str, &[&str])] = &[
        ("refreshmap", &[]),
        ("special", &["UpdateTimePals"]),
        ("sjump", &["RockSmashScript"]),
    ];
    const DONE: &[(&str, &[&str])] = &[("end", &[])];
    certify_exact_callasm_body(definitions, start_label, MAIN)?;
    certify_exact_callasm_body(definitions, ".no_battle@RockMonEncounter", NO_BATTLE)?;
    certify_exact_callasm_body(definitions, "RockSmashScript", ROCK_SMASH_SCRIPT)?;
    let resolved_done = resolve_script_target_label(definitions, "RockSmashScript", ".done")
        .ok_or_else(|| {
            ScriptCallasmCertificateFailure::at(
                "RockSmashScript",
                10,
                "iffalse",
                "exact Rock Smash no-encounter branch target does not resolve",
            )
        })?;
    if resolved_done != ".done@RockSmashScript" {
        return Err(ScriptCallasmCertificateFailure::at(
            "RockSmashScript",
            10,
            "iffalse",
            format!(
                "exact Rock Smash no-encounter branch must resolve to .done@RockSmashScript, found {resolved_done}"
            ),
        ));
    }
    certify_exact_callasm_body(definitions, &resolved_done, DONE)?;
    certify_exact_callasm_body(definitions, "RockSmashFromMenuScript", FROM_MENU)
}

pub(crate) fn certify_tree_mon_encounter_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    if start_label != "TreeMonEncounter" {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<target>",
            "typed Headbutt encounter target must resolve exactly to TreeMonEncounter",
        ));
    }
    const MAIN: &[(&str, &[&str])] = &[
        ("farcall", &["StubbedTrainerRankings_TreeEncounters"]),
        ("xor", &["a"]),
        ("ld", &["[wTempWildMonSpecies]", "a"]),
        ("ld", &["[wCurPartyLevel]", "a"]),
        ("ld", &["hl", "TreeMonMaps"]),
        ("call", &["GetTreeMonSet"]),
        ("jr", &["nc", ".no_battle"]),
        ("call", &["GetTreeMons"]),
        ("jr", &["nc", ".no_battle"]),
        ("call", &["GetTreeMon"]),
        ("jr", &["nc", ".no_battle"]),
        ("ld", &["a", "BATTLETYPE_TREE"]),
        ("ld", &["[wBattleType]", "a"]),
        ("ld", &["a", "1"]),
        ("ld", &["[wScriptVar]", "a"]),
        ("ret", &[]),
    ];
    const NO_BATTLE: &[(&str, &[&str])] = &[
        ("xor", &["a"]),
        ("ld", &["[wScriptVar]", "a"]),
        ("ret", &[]),
    ];
    const HEADBUTT_SCRIPT: &[(&str, &[&str])] = &[
        ("callasm", &["GetPartyNickname"]),
        ("writetext", &["UseHeadbuttText"]),
        ("refreshmap", &[]),
        ("callasm", &["ShakeHeadbuttTree"]),
        ("callasm", &["TreeMonEncounter"]),
        ("iffalse", &[".no_battle"]),
        ("closetext", &[]),
        ("randomwildmon", &[]),
        ("startbattle", &[]),
        ("reloadmapafterbattle", &[]),
        ("end", &[]),
    ];
    const NO_BATTLE_SCRIPT: &[(&str, &[&str])] = &[
        ("writetext", &["HeadbuttNothingText"]),
        ("waitbutton", &[]),
        ("closetext", &[]),
        ("end", &[]),
    ];
    certify_exact_callasm_body(definitions, start_label, MAIN)?;
    certify_exact_callasm_body(definitions, ".no_battle@TreeMonEncounter", NO_BATTLE)?;
    certify_exact_callasm_body(definitions, "HeadbuttScript", HEADBUTT_SCRIPT)?;
    let resolved_no_battle =
        resolve_script_target_label(definitions, "HeadbuttScript", ".no_battle").ok_or_else(
            || {
                ScriptCallasmCertificateFailure::at(
                    "HeadbuttScript",
                    5,
                    "iffalse",
                    "exact Headbutt no-encounter branch target does not resolve",
                )
            },
        )?;
    if resolved_no_battle != ".no_battle@HeadbuttScript" {
        return Err(ScriptCallasmCertificateFailure::at(
            "HeadbuttScript",
            5,
            "iffalse",
            format!(
                "exact Headbutt no-encounter branch must resolve to .no_battle@HeadbuttScript, found {resolved_no_battle}"
            ),
        ));
    }
    certify_exact_callasm_body(definitions, &resolved_no_battle, NO_BATTLE_SCRIPT)
}

pub(crate) fn certify_sweet_scent_encounter_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    if start_label != "SweetScentEncounter" {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<target>",
            "typed Sweet Scent encounter target must resolve exactly to SweetScentEncounter",
        ));
    }
    const MAIN: &[(&str, &[&str])] = &[
        ("farcall", &["CanEncounterWildMon"]),
        ("jr", &["nc", ".no_battle"]),
        ("ld", &["hl", "wStatusFlags2"]),
        ("bit", &["STATUSFLAGS2_BUG_CONTEST_TIMER_F", "[hl]"]),
        ("jr", &["nz", ".in_bug_contest"]),
        ("farcall", &["GetMapEncounterRate"]),
        ("ld", &["a", "b"]),
        ("and", &["a"]),
        ("jr", &["z", ".no_battle"]),
        ("farcall", &["ChooseWildEncounter"]),
        ("jr", &["nz", ".no_battle"]),
        ("jr", &[".start_battle"]),
    ];
    const CONTEST: &[(&str, &[&str])] = &[
        ("farcall", &["ChooseWildEncounter_BugContest"]),
        ("jp", &[".start_battle@SweetScentEncounter"]),
    ];
    const START: &[(&str, &[&str])] = &[
        ("ld", &["a", "$1"]),
        ("ld", &["[wScriptVar]", "a"]),
        ("ret", &[]),
    ];
    const NO_BATTLE: &[(&str, &[&str])] = &[
        ("xor", &["a"]),
        ("ld", &["[wScriptVar]", "a"]),
        ("ld", &["[wBattleType]", "a"]),
        ("ret", &[]),
    ];
    const SCRIPT: &[(&str, &[&str])] = &[
        ("refreshmap", &[]),
        ("special", &["UpdateTimePals"]),
        ("callasm", &["GetPartyNickname"]),
        ("writetext", &["UseSweetScentText"]),
        ("waitbutton", &[]),
        ("callasm", &["SweetScentEncounter"]),
        ("iffalse", &["SweetScentNothing"]),
        ("checkflag", &["ENGINE_BUG_CONTEST_TIMER"]),
        ("iftrue", &[".BugCatchingContest"]),
        ("randomwildmon", &[]),
        ("startbattle", &[]),
        ("reloadmapafterbattle", &[]),
        ("end", &[]),
    ];
    certify_exact_callasm_body(definitions, start_label, MAIN)?;
    certify_exact_callasm_body(definitions, ".in_bug_contest@SweetScentEncounter", CONTEST)?;
    certify_exact_callasm_body(definitions, ".start_battle@SweetScentEncounter", START)?;
    certify_exact_callasm_body(definitions, ".no_battle@SweetScentEncounter", NO_BATTLE)?;
    certify_exact_callasm_body(definitions, ".SweetScent@SweetScentFromMenu", SCRIPT)
}

pub(crate) fn certify_synchronous_script_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    if start_label == "SweetScentEncounter" {
        return certify_sweet_scent_encounter_callasm_target(definitions, start_label);
    }
    if start_label == "Fishing_CheckFacingUp" {
        return certify_fishing_check_facing_up_callasm_target(definitions, start_label);
    }
    if start_label == ".CheckContinueWaterfall@Script_UsedWaterfall" {
        const BODY: &[(&str, &[&str])] = &[
            ("xor", &["a"]),
            ("ld", &["[wScriptVar]", "a"]),
            ("ld", &["a", "[wPlayerTileCollision]"]),
            ("call", &["CheckWaterfallTile"]),
            ("ret", &["z"]),
            ("farcall", &["StubbedTrainerRankings_Waterfall"]),
            ("ld", &["a", "$1"]),
            ("ld", &["[wScriptVar]", "a"]),
            ("ret", &[]),
        ];
        return certify_exact_callasm_body(definitions, start_label, BODY);
    }
    if exact_overworld_visual_callasm_effect(definitions, start_label)?.is_some() {
        return Ok(());
    }
    if exact_phone_callasm_effect(definitions, start_label)?.is_some() {
        return Ok(());
    }
    let branching = certify_branching_script_callasm_target(definitions, start_label);
    if branching.is_ok() {
        return Ok(());
    }
    let accumulator = certify_accumulator_script_callasm_target(definitions, start_label);
    if accumulator.is_ok() {
        return Ok(());
    }
    let branching = branching.expect_err("checked failing branching certificate");
    let accumulator = accumulator.expect_err("checked failing accumulator certificate");
    if branching.command_index >= accumulator.command_index {
        Err(branching)
    } else {
        Err(accumulator)
    }
}

fn certify_fishing_check_facing_up_callasm_target(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<(), ScriptCallasmCertificateFailure> {
    const MAIN: &[(&str, &[&str])] = &[
        ("ld", &["a", "[wPlayerDirection]"]),
        ("and", &["$c"]),
        ("cp", &["OW_UP"]),
        ("ld", &["a", "$1"]),
        ("jr", &["z", ".up"]),
        ("xor", &["a"]),
        ("ld", &["[wScriptVar]", "a"]),
        ("ret", &[]),
    ];
    const UP: &[(&str, &[&str])] = &[("ld", &["[wScriptVar]", "a"]), ("ret", &[])];
    if start_label != "Fishing_CheckFacingUp" {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<target>",
            "typed fishing-facing target must resolve exactly to Fishing_CheckFacingUp",
        ));
    }
    certify_exact_callasm_body(definitions, start_label, MAIN)?;
    let up = resolve_script_target_label(definitions, start_label, ".up").ok_or_else(|| {
        ScriptCallasmCertificateFailure::at(
            start_label,
            4,
            "jr",
            "exact fishing-facing up branch does not resolve",
        )
    })?;
    if up != ".up@Fishing_CheckFacingUp" {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            4,
            "jr",
            format!("exact fishing-facing branch resolved to {up}"),
        ));
    }
    certify_exact_callasm_body(definitions, &up, UP)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactOverworldVisualCallasmEffect {
    HideSprites,
    FlyAnimation,
    ShakeHeadbuttTree,
    CutBlockRefresh,
    WhirlpoolBlockRefresh,
    BlindingFlash,
    LoadFishingGfx,
    SkipUpdateMapSprites,
    ReturnFromFly,
    StubbedSurfRanking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactPhoneCallasmEffect {
    RingTwice,
    HangUp,
    InitReceiveDelay,
    LoadCaller(&'static str),
}

fn exact_phone_callasm_effect(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<Option<ExactPhoneCallasmEffect>, ScriptCallasmCertificateFailure> {
    const RING_LABELS: &[&str] = &[
        "RingTwice_StartCall",
        ".Ring@RingTwice_StartCall",
        "StubbedTrainerRankings_PhoneCalls",
        "Phone_StartRinging",
        "Phone_Wait20Frames",
        ".CallerTextboxWithName@RingTwice_StartCall",
        "Phone_CallerTextbox",
        "Phone_TextboxWithName",
        "GetCallerClassAndName",
        "GetCallerTrainerClass",
        "GetCallerName",
        ".NotTrainer@GetCallerName",
        "Phone_GetTrainerName",
        "Phone_GetTrainerClassName",
    ];
    const HANG_UP_LABELS: &[&str] = &[
        "HangUp",
        "HangUp_Beep",
        "HangUp_Wait20Frames",
        "Phone_CallEnd",
        "Phone_Wait20Frames",
        "HangUp_BoopOn",
        "HangUp_BoopOff",
    ];
    const INIT_DELAY_LABELS: &[&str] = &[
        "InitCallReceiveDelay",
        "NextCallReceiveDelay",
        ".okay@NextCallReceiveDelay",
        "RestartReceiveCallDelay",
        "CopyDayHourMinToHL",
    ];
    const LOAD_CALLER_LABELS: &[&str] = &[
        ".LoadBillScript@Script_SpecialBillCall",
        ".LoadElmScript@Script_SpecialElmCall",
        "LoadCallerScript",
        ".actualcaller@LoadCallerScript",
        ".proceed@LoadCallerScript",
    ];
    let (effect, labels, expected_fingerprint) = match start_label {
        "RingTwice_StartCall" => (
            ExactPhoneCallasmEffect::RingTwice,
            RING_LABELS,
            0xb446_101b_0f6a_4e5e,
        ),
        "HangUp" => (
            ExactPhoneCallasmEffect::HangUp,
            HANG_UP_LABELS,
            0x3fc5_e1c3_3df1_e7bd,
        ),
        "InitCallReceiveDelay" => (
            ExactPhoneCallasmEffect::InitReceiveDelay,
            INIT_DELAY_LABELS,
            0xa868_f27f_fd09_2149,
        ),
        ".LoadBillScript@Script_SpecialBillCall" => (
            ExactPhoneCallasmEffect::LoadCaller("PHONE_BILL"),
            LOAD_CALLER_LABELS,
            0x5ed8_c8d4_0a65_ac4e,
        ),
        ".LoadElmScript@Script_SpecialElmCall" => (
            ExactPhoneCallasmEffect::LoadCaller("PHONE_ELM"),
            LOAD_CALLER_LABELS,
            0x5ed8_c8d4_0a65_ac4e,
        ),
        _ => return Ok(None),
    };
    let actual_fingerprint = callasm_source_body_fingerprint(definitions, labels)?;
    if actual_fingerprint != expected_fingerprint {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<source fingerprint>",
            format!(
                "exact phone routine fingerprint changed: expected {expected_fingerprint:#018x}, found {actual_fingerprint:#018x}"
            ),
        ));
    }
    Ok(Some(effect))
}

fn commit_pending_block_field_move(
    state: &mut GameState,
    overworld: &mut OverworldSession,
    expected_move_id: &str,
) -> Result<()> {
    let pending = state
        .script_runtime
        .pending_block_field_move
        .as_ref()
        .with_context(|| {
            format!("{expected_move_id} source callasm has no prepared block field move")
        })?;
    anyhow::ensure!(
        pending.move_id == expected_move_id,
        "{expected_move_id} source callasm found pending {} field move",
        pending.move_id
    );
    anyhow::ensure!(
        pending.map_name == overworld.map.name,
        "{expected_move_id} source callasm prepared on {}, current map is {}",
        pending.map_name,
        overworld.map.name
    );
    let metatile_x = i16::try_from(pending.metatile_x)
        .with_context(|| format!("prepared {expected_move_id} metatile x overflows i16"))?;
    let metatile_y = i16::try_from(pending.metatile_y)
        .with_context(|| format!("prepared {expected_move_id} metatile y overflows i16"))?;
    let index = overworld
        .map
        .metatile_index(metatile_x, metatile_y)
        .with_context(|| {
            format!(
                "prepared {expected_move_id} metatile ({}, {}) is outside current map {}",
                pending.metatile_x, pending.metatile_y, overworld.map.name
            )
        })?;
    let current_block = overworld.map.metatile_ids[index];
    anyhow::ensure!(
        current_block == pending.previous_block_id,
        "prepared {expected_move_id} block changed before source callasm: expected {:#04x}, found {current_block:#04x}",
        pending.previous_block_id
    );
    overworld.map.metatile_ids[index] = pending.replacement_block_id;
    state
        .map_block_overrides
        .entry(pending.map_name.clone())
        .or_default()
        .insert(
            (pending.metatile_x, pending.metatile_y),
            pending.replacement_block_id,
        );
    state.script_runtime.pending_block_field_move = None;
    Ok(())
}

fn exact_overworld_visual_callasm_effect(
    definitions: &BTreeMap<String, Value>,
    start_label: &str,
) -> std::result::Result<Option<ExactOverworldVisualCallasmEffect>, ScriptCallasmCertificateFailure>
{
    const HIDE_LABELS: &[&str] = &["HideSprites", ".loop@HideSprites"];
    const FLY_LABELS: &[&str] = &[
        "FlyFromAnim",
        ".loop@FlyFromAnim",
        ".exit@FlyFromAnim",
        "FlyToAnim",
        ".loop@FlyToAnim",
        ".exit@FlyToAnim",
        ".RestorePlayerSprite_DespawnLeaves@FlyToAnim",
        ".OAMloop@FlyToAnim",
        "FlyFunction_InitGFX",
        "FlyFunction_FrameTimer",
        ".exit@FlyFunction_FrameTimer",
        ".SpawnLeaf@FlyFunction_FrameTimer",
    ];
    const SHAKE_HEADBUTT_TREE_LABELS: &[&str] = &[
        "ShakeHeadbuttTree",
        ".loop@ShakeHeadbuttTree",
        ".done@ShakeHeadbuttTree",
        "HideHeadbuttTree",
        "TreeRelativeLocationTable",
        "OWCutAnimation",
        ".loop@OWCutAnimation",
        ".finish@OWCutAnimation",
        ".LoadCutGFX@OWCutAnimation",
        "OWCutJumptable",
        ".dw@OWCutJumptable",
        "Cut_SpawnAnimateTree",
        "Cut_SpawnAnimateLeaves",
        "Cut_StartWaiting",
        "Cut_WaitAnimSFX",
        ".finished@Cut_WaitAnimSFX",
        "Cut_SpawnLeaf",
        "Cut_GetLeafSpawnCoords",
        ".left_side@Cut_GetLeafSpawnCoords",
        ".top_side@Cut_GetLeafSpawnCoords",
        ".Coords@Cut_GetLeafSpawnCoords",
        "Cut_Headbutt_GetPixelFacing",
        ".Coords@Cut_Headbutt_GetPixelFacing",
        "FlyFromAnim",
        ".loop@FlyFromAnim",
        ".exit@FlyFromAnim",
        "FlyFunction_InitGFX",
        "FlyFunction_FrameTimer",
        ".exit@FlyFunction_FrameTimer",
        ".SpawnLeaf@FlyFunction_FrameTimer",
    ];
    const CUT_BLOCK_LABELS: &[&str] = &["CutDownTreeOrGrass"];
    const WHIRLPOOL_BLOCK_LABELS: &[&str] = &["DisappearWhirlpool"];
    const BLINDING_FLASH_LABELS: &[&str] = &["BlindingFlash"];
    const FISHING_GFX_LABELS: &[&str] = &[
        "LoadFishingGFX",
        ".got_gender@LoadFishingGFX",
        ".LoadGFX@LoadFishingGFX",
    ];
    const SKIP_LABELS: &[&str] = &["SkipUpdateMapSprites"];
    const RETURN_LABELS: &[&str] = &[".ReturnFromFly@FlyFunction"];
    const STUBBED_SURF_LABELS: &[&str] = &[".stubbed_fn@UsedSurfScript"];
    let (effect, labels, expected_fingerprint) = match start_label {
        "HideSprites" => (
            ExactOverworldVisualCallasmEffect::HideSprites,
            HIDE_LABELS,
            0xbe58_6c52_a86d_4425,
        ),
        "FlyFromAnim" | "FlyToAnim" => (
            ExactOverworldVisualCallasmEffect::FlyAnimation,
            FLY_LABELS,
            0x7cf4_8359_a980_a0c9,
        ),
        "ShakeHeadbuttTree" => (
            ExactOverworldVisualCallasmEffect::ShakeHeadbuttTree,
            SHAKE_HEADBUTT_TREE_LABELS,
            0x7cc6_b834_baea_40a4,
        ),
        "CutDownTreeOrGrass" => (
            ExactOverworldVisualCallasmEffect::CutBlockRefresh,
            CUT_BLOCK_LABELS,
            0x9a81_bb16_ce9d_613e,
        ),
        "DisappearWhirlpool" => (
            ExactOverworldVisualCallasmEffect::WhirlpoolBlockRefresh,
            WHIRLPOOL_BLOCK_LABELS,
            0xb623_c6f0_cfbb_04f4,
        ),
        "BlindingFlash" => (
            ExactOverworldVisualCallasmEffect::BlindingFlash,
            BLINDING_FLASH_LABELS,
            0x1136_b908_028b_1c95,
        ),
        "LoadFishingGFX" => (
            ExactOverworldVisualCallasmEffect::LoadFishingGfx,
            FISHING_GFX_LABELS,
            0x3443_c1fb_3d26_8d04,
        ),
        "SkipUpdateMapSprites" => (
            ExactOverworldVisualCallasmEffect::SkipUpdateMapSprites,
            SKIP_LABELS,
            0xd9fb_aad4_e73a_8c1c,
        ),
        ".ReturnFromFly@FlyFunction" => (
            ExactOverworldVisualCallasmEffect::ReturnFromFly,
            RETURN_LABELS,
            0x508d_b25c_e664_4646,
        ),
        ".stubbed_fn@UsedSurfScript" => (
            ExactOverworldVisualCallasmEffect::StubbedSurfRanking,
            STUBBED_SURF_LABELS,
            0xdfd2_b2c7_cc67_e0b9,
        ),
        _ => return Ok(None),
    };
    let actual_fingerprint = callasm_source_body_fingerprint(definitions, labels)?;
    if actual_fingerprint != expected_fingerprint {
        return Err(ScriptCallasmCertificateFailure::at(
            start_label,
            0,
            "<source fingerprint>",
            format!(
                "exact overworld visual routine fingerprint changed: expected {expected_fingerprint:#018x}, found {actual_fingerprint:#018x}"
            ),
        ));
    }
    Ok(Some(effect))
}

fn callasm_source_body_fingerprint(
    definitions: &BTreeMap<String, Value>,
    labels: &[&str],
) -> std::result::Result<u64, ScriptCallasmCertificateFailure> {
    fn update(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(1_099_511_628_211);
        }
        *hash ^= 0;
        *hash = hash.wrapping_mul(1_099_511_628_211);
    }

    let mut hash = 14_695_981_039_346_656_037_u64;
    for label in labels {
        update(&mut hash, label.as_bytes());
        let entries = definitions
            .get(*label)
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ScriptCallasmCertificateFailure::at(
                    label,
                    0,
                    "<missing>",
                    "exact overworld visual routine body is missing",
                )
            })?;
        for (command_index, entry) in entries.iter().enumerate() {
            let command = entry
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ScriptCallasmCertificateFailure::at(
                        label,
                        command_index,
                        "<missing>",
                        "exact overworld visual command is missing",
                    )
                })?;
            update(&mut hash, command.as_bytes());
            let args = entry.get("args").and_then(Value::as_array).ok_or_else(|| {
                ScriptCallasmCertificateFailure::at(
                    label,
                    command_index,
                    command,
                    "exact overworld visual args are missing",
                )
            })?;
            for argument in args {
                let argument = argument.as_str().ok_or_else(|| {
                    ScriptCallasmCertificateFailure::at(
                        label,
                        command_index,
                        command,
                        "exact overworld visual arg is not a string",
                    )
                })?;
                update(&mut hash, argument.as_bytes());
            }
            hash ^= 0xff;
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
        hash ^= 0xfe;
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    Ok(hash)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CallasmCpuFlags {
    zero: Option<bool>,
    carry: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptCallasmExecution {
    script_value: Option<String>,
    phone_presentation:
        Option<crystal_core::systems::script_runtime::ScriptPhoneCallasmPresentation>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct BranchingCallasmState {
    registers: BTreeMap<String, String>,
    flags: CallasmCpuFlags,
    selected_party_index: Option<usize>,
    script_value: Option<String>,
}

fn read_branching_callasm_value(
    state: &GameState,
    cpu: &BranchingCallasmState,
    operand: &str,
) -> Option<String> {
    if let Some(value) = cpu.registers.get(operand) {
        return Some(value.clone());
    }

    if let Some(symbol) = callasm_indirect_symbol(operand) {
        let address = if symbol == "hl" {
            cpu.registers.get("hl")?.as_str()
        } else {
            symbol
        };
        if let Some(offset) = address.strip_prefix("wPartySpecies+") {
            let party_index: usize = parse_script_i32(offset).ok()?.try_into().ok()?;
            return state
                .storage
                .party
                .pokemon
                .get(party_index)
                .and_then(Option::as_ref)
                .map(|pokemon| pokemon.species.id.clone());
        }
        return match address {
            "wScriptVar" => state.script_runtime.script_value.clone(),
            _ => state.script_runtime.memory.get(address).cloned(),
        };
    }

    Some(
        parse_script_i32(operand)
            .map(|value| value.to_string())
            .unwrap_or_else(|_| operand.to_string()),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BranchingCallasmBitEffect<'a> {
    memory: &'a str,
    bit: &'a str,
    engine_flag: &'static str,
    inverted: bool,
}

fn branching_callasm_bit_effect<'a>(
    memory: &'a str,
    bit: &'a str,
) -> Option<BranchingCallasmBitEffect<'a>> {
    let (engine_flag, inverted) = match (memory, bit) {
        ("wBikeFlags", "BIKEFLAGS_STRENGTH_ACTIVE_F" | "0") => ("ENGINE_STRENGTH_ACTIVE", false),
        ("wBikeFlags", "BIKEFLAGS_ALWAYS_ON_BIKE_F" | "1") => ("ENGINE_ALWAYS_ON_BIKE", false),
        ("wBikeFlags", "BIKEFLAGS_DOWNHILL_F" | "2") => ("ENGINE_DOWNHILL", false),
        (
            "wEnabledPlayerEvents",
            "PLAYEREVENTS_WILD_ENCOUNTERS" | "PLAYEREVENTS_WILD_ENCOUNTERS_F" | "4",
        ) => ("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true),
        _ => return None,
    };
    Some(BranchingCallasmBitEffect {
        memory,
        bit,
        engine_flag,
        inverted,
    })
}

fn branching_callasm_engine_flag_is_set(state: &GameState, flag: &str) -> bool {
    if let Some(index) = crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS
        .iter().position(|candidate| *candidate == flag)
    {
        return if index < 8 { state.badges.johto[index] } else { state.badges.kanto[index - 8] };
    }
    state.flags.engine_flags.get(flag).copied().unwrap_or(false)
}

fn branching_callasm_party_nickname(state: &GameState, party_index: usize) -> Option<String> {
    let pokemon = state
        .storage
        .party
        .pokemon
        .get(party_index)
        .and_then(Option::as_ref)?;
    Some(if pokemon.nickname.is_empty() {
        pokemon.species.id.clone()
    } else {
        pokemon.nickname.clone()
    })
}

fn branching_callasm_named_buffer_key(address: &str) -> Option<&'static str> {
    match address {
        "wStringBuffer1" => Some("STRING_BUFFER_1"),
        "wStringBuffer2" => Some("STRING_BUFFER_2"),
        "wStringBuffer3" => Some("STRING_BUFFER_3"),
        _ => None,
    }
}

fn branching_callasm_named_buffer(state: &GameState, address: &str) -> Option<String> {
    let key = branching_callasm_named_buffer_key(address)?;
    state.script_runtime.named_buffers.get(key).cloned()
}

#[cfg(test)]
mod branching_callasm_host_service_tests {
    use super::*;

    fn base_game_data() -> GameDataSet {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repository_root = manifest_dir
            .ancestors()
            .nth(3)
            .expect("workspace is nested under rust/crates/crystal-assets");
        AssetRoot::new(repository_root)
            .load_base_game_data()
            .expect("load base game data")
    }

    #[test]
    fn describedecoration_resolves_equipped_poster_and_ornament_scripts() {
        let data = GameDataSet::default();
        let mut state = GameState::default();
        state.script_runtime.memory.insert(
            "wDecoPoster".to_string(),
            "DECO_CLEFAIRY_POSTER".to_string(),
        );
        state.script_runtime.memory.insert(
            "wDecoLeftOrnament".to_string(),
            "DECO_PIKACHU_DOLL".to_string(),
        );
        let command = |selector: &str| ScriptRuntimeCommand {
            command: "describedecoration".to_string(),
            args: vec![selector.to_string()],
            source_script: "PlayersHouseDoll1Script".to_string(),
            command_index: 0,
        };

        assert_eq!(
            data.resolve_script_runtime_decoration(&state, &command("DECODESC_POSTER"))
                .expect("resolve poster"),
            Some(ScriptRuntimeDecorationResolution {
                target_script: "DecorationDesc_ClefairyPoster".to_string(),
                string_buffer_3: None,
            })
        );
        assert_eq!(
            data.resolve_script_runtime_decoration(&state, &command("DECODESC_LEFT_DOLL"))
                .expect("resolve ornament"),
            Some(ScriptRuntimeDecorationResolution {
                target_script: ".OrnamentConsoleScript@DecorationDesc_OrnamentOrConsole"
                    .to_string(),
                string_buffer_3: Some("PIKACHU DOLL".to_string()),
            })
        );
    }

    #[test]
    fn canonical_decoration_scripts_preserve_source_control_flow() {
        let scripts = decoration_description_scripts();
        assert_eq!(
            scripts["DecorationDesc_TownMapPoster"],
            serde_json::json!([
                {"command": "opentext", "args": []},
                {"command": "farwritetext", "args": ["_LookTownMapText"]},
                {"command": "waitbutton", "args": []},
                {"command": "special", "args": ["OverworldTownMap"]},
                {"command": "closetext", "args": []},
                {"command": "end", "args": []}
            ])
        );
        assert_eq!(
            scripts[".BigDollScript@DecorationDesc_GiantOrnament"],
            serde_json::json!([
                {"command": "farjumptext", "args": ["_LookGiantDecoText"]}
            ])
        );
    }

    #[test]
    fn base_pack_materializes_executable_decoration_description_scripts() {
        let data = base_game_data();
        let town_map = data
            .compiled_script_body("DecorationDesc_TownMapPoster")
            .expect("town-map decoration script is executable")
            .as_array()
            .expect("town-map decoration body");
        assert_eq!(town_map.len(), 6);
        assert_eq!(
            data.script_text_command("PlayersHouse2F", "DecorationDesc_TownMapPoster", 1,)
                .expect("town-map text command")
                .text_label,
            Some("_LookTownMapText".to_string())
        );
        assert_eq!(
            data.script_runtime_command("PlayersHouse2F", "DecorationDesc_TownMapPoster", 3,)
                .expect("town-map special command")
                .args,
            ["OverworldTownMap"]
        );
        assert!(
            data.compiled_script_body(".OrnamentConsoleScript@DecorationDesc_OrnamentOrConsole")
                .is_some()
        );
    }

    #[test]
    fn check_engine_flag_returns_zero_accumulator_for_both_carry_results() {
        let data = GameDataSet::default();
        for (badge_set, expected_carry) in [(false, true), (true, false)] {
            let mut state = GameState::default();
            state.badges.johto[0] = badge_set;
            let mut cpu = BranchingCallasmState::default();
            cpu.registers
                .insert("de".to_string(), "ENGINE_ZEPHYRBADGE".to_string());

            assert!(
                data.apply_branching_callasm_host_service(
                    &mut state,
                    &mut cpu,
                    BranchingCallasmHostService::CheckEngineFlag,
                )
                .expect("execute CheckEngineFlag host service")
            );
            assert_eq!(cpu.registers.get("a").map(String::as_str), Some("0"));
            assert_eq!(cpu.flags.zero, Some(true));
            assert_eq!(cpu.flags.carry, Some(expected_carry));
        }
    }

    #[test]
    fn nickname_host_calls_copy_each_exact_asm_buffer_stage() {
        let data = base_game_data();
        let species = data
            .pokemon
            .get("CHIKORITA")
            .expect("compiled Chikorita species");
        let mut pokemon = create_pokemon_from_known_dvs(
            species,
            5,
            Dv::default(),
            &data.learnsets,
            &data.moves,
            &data.growth_rates,
        )
        .expect("materialize party Pokemon");
        pokemon.nickname = "LEAF".to_string();
        let mut state = GameState::default();
        state.storage.party.pokemon[0] = Some(pokemon);
        let mut cpu = BranchingCallasmState::default();
        cpu.registers.insert("a".to_string(), "0".to_string());
        cpu.registers
            .insert("hl".to_string(), "wPartyMonNicknames".to_string());

        assert!(
            data.apply_branching_callasm_host_service(
                &mut state,
                &mut cpu,
                BranchingCallasmHostService::GetNickname,
            )
            .expect("execute GetNickname host service")
        );
        assert_eq!(
            state.script_runtime.named_buffers.get("STRING_BUFFER_1"),
            Some(&"LEAF".to_string())
        );
        assert!(
            !state
                .script_runtime
                .named_buffers
                .contains_key("STRING_BUFFER_2")
        );
        assert!(
            !state
                .script_runtime
                .named_buffers
                .contains_key("STRING_BUFFER_3")
        );
        assert_eq!(
            cpu.registers.get("de").map(String::as_str),
            Some("wStringBuffer1")
        );

        assert!(
            data.apply_branching_callasm_host_service(
                &mut state,
                &mut cpu,
                BranchingCallasmHostService::CopyName1,
            )
            .expect("execute CopyName1 host service")
        );
        assert_eq!(
            state.script_runtime.named_buffers.get("STRING_BUFFER_2"),
            Some(&"LEAF".to_string())
        );
        assert!(
            !state
                .script_runtime
                .named_buffers
                .contains_key("STRING_BUFFER_3")
        );

        cpu.registers
            .insert("de".to_string(), "wStringBuffer2".to_string());
        cpu.registers
            .insert("hl".to_string(), "wStringBuffer3".to_string());
        assert!(
            data.apply_branching_callasm_host_service(
                &mut state,
                &mut cpu,
                BranchingCallasmHostService::CopyName2,
            )
            .expect("execute CopyName2 host service")
        );
        assert_eq!(
            state.script_runtime.named_buffers.get("STRING_BUFFER_3"),
            Some(&"LEAF".to_string())
        );
    }

    #[test]
    fn check_party_move_clears_stale_cursor_and_returns_exact_failure_flags() {
        let data = GameDataSet::default();
        let mut state = GameState::default();
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), "4".to_string());
        let mut cpu = BranchingCallasmState::default();
        cpu.registers
            .insert("d".to_string(), "STRENGTH".to_string());

        assert!(
            data.apply_branching_callasm_host_service(
                &mut state,
                &mut cpu,
                BranchingCallasmHostService::CheckPartyMove,
            )
            .expect("execute CheckPartyMove host service")
        );
        assert_eq!(
            state
                .script_runtime
                .memory
                .get("wCurPartyMon")
                .map(String::as_str),
            Some("0")
        );
        assert_eq!(cpu.registers.get("a").map(String::as_str), Some("255"));
        assert_eq!(cpu.registers.get("e").map(String::as_str), Some("0"));
        assert_eq!(cpu.flags.zero, Some(true));
        assert_eq!(cpu.flags.carry, Some(true));
        assert_eq!(cpu.selected_party_index, None);
    }

    #[test]
    fn try_strength_cpu_failure_returns_one_synchronously() {
        let data = base_game_data();
        let definitions = &data
            .global_scripts
            .as_ref()
            .expect("base pack global scripts")
            .definitions;
        let mut state = GameState::default();
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), "4".to_string());

        let execution = data
            .execute_branching_script_callasm(
                &mut state,
                "GlobalScripts",
                definitions,
                "TryStrengthOW",
            )
            .expect("execute TryStrengthOW CPU routine")
            .expect("TryStrengthOW is synchronously executable");

        assert_eq!(execution.script_value.as_deref(), Some("1"));
        assert_eq!(state.script_runtime.script_value.as_deref(), Some("1"));
        assert_eq!(
            state
                .script_runtime
                .memory
                .get("wCurPartyMon")
                .map(String::as_str),
            Some("0")
        );
    }

    #[test]
    fn gift_name_data_uses_scoped_local_over_bare_collision() {
        let mut data = base_game_data();
        let scripts = &mut data
            .maps
            .get_mut("Route29")
            .expect("compiled Route 29 map")
            .scripts;
        scripts.insert(
            ".GiftName".to_string(),
            serde_json::json!([{"command": "db", "args": ["\"BARE@\""]}]),
        );
        scripts.insert(
            ".GiftName@ParentScript".to_string(),
            serde_json::json!([{"command": "db", "args": ["\"SCOPED@\""]}]),
        );

        assert_eq!(
            data.resolve_gift_name_label("Route29", "ParentScript", ".GiftName")
                .expect("resolve scoped gift name"),
            "SCOPED"
        );
    }
}

#[cfg(test)]
mod overworld_tracking_object_tests {
    use super::*;

    #[test]
    fn grass_rustle_duration_matches_same_frame_tracking_object_decrements() {
        assert_eq!(grass_rustle_duration_for_speed(1).unwrap(), 6);
        assert_eq!(grass_rustle_duration_for_speed(2).unwrap(), 2);
        assert!(grass_rustle_duration_for_speed(0).is_err());
        assert!(grass_rustle_duration_for_speed(3).is_err());
    }
}

impl CallasmCpuFlags {
    fn condition_is_met(self, condition: ScriptRuntimeCpuCondition) -> Option<bool> {
        match condition {
            ScriptRuntimeCpuCondition::Z | ScriptRuntimeCpuCondition::Nz => {
                self.zero.map(|zero| condition.is_met(zero, false))
            }
            ScriptRuntimeCpuCondition::C | ScriptRuntimeCpuCondition::Nc => {
                self.carry.map(|carry| condition.is_met(false, carry))
            }
        }
    }
}

fn read_callasm_alu_operand(
    state: &GameState,
    scratch_memory: &BTreeMap<String, Option<u8>>,
    register_a: Option<u8>,
    operand: &str,
) -> Result<Option<u8>> {
    if operand == "a" {
        return Ok(register_a);
    }
    read_callasm_byte_operand(state, scratch_memory, operand)
}

fn read_callasm_byte_operand(
    state: &GameState,
    scratch_memory: &BTreeMap<String, Option<u8>>,
    operand: &str,
) -> Result<Option<u8>> {
    if let Some(symbol) = callasm_indirect_symbol(operand) {
        if let Some(value) = scratch_memory.get(symbol) {
            return Ok(*value);
        }
        let token = match symbol {
            // This saved field is the authoritative Rust representation of
            // Crystal's WRAM byte selected by the Battle Tower level menu.
            "wBTChoiceOfLvlGroup" => return Ok(Some(state.battle_tower.level_group)),
            "wScriptVar" => state.script_runtime.script_value.as_deref(),
            _ => state.script_runtime.memory.get(symbol).map(String::as_str),
        };
        return token
            .map(|token| {
                parse_script_i32(token)
                    .map(|value| value.rem_euclid(256) as u8)
                    .with_context(|| format!("callasm cannot read byte {token:?} from [{symbol}]"))
            })
            .transpose();
    }
    if operand.starts_with("BANK(") && operand.ends_with(')') {
        // Bank values are only used to select WRAM while the routine runs;
        // the concrete bank number is irrelevant to the mapped byte transfer.
        return Ok(None);
    }
    match parse_script_i32(operand) {
        Ok(value) => Ok(Some(value.rem_euclid(256) as u8)),
        Err(_) => Ok(None),
    }
}

fn strip_compiled_mail_text(value: &str) -> String {
    let trimmed = value.trim();
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(trimmed);
    unquoted.trim_end_matches('@').to_string()
}

fn warp_sound_effect_for_collision(permission: u8) -> &'static str {
    match permission {
        permissions::DOOR => "SFX_ENTER_DOOR",
        permissions::WARP_PANEL => "SFX_WARP_TO",
        _ => "SFX_EXIT_BUILDING",
    }
}

fn set_compiled_mail_check_result(state: &mut GameState, result: u8) {
    let result = result.to_string();
    state.script_runtime.script_value = Some(result.clone());
    state
        .script_runtime
        .variables
        .insert("wScriptVar".to_string(), result);
}

fn set_npc_trade_result(state: &mut GameState, result: u8) {
    let result = result.to_string();
    state.script_runtime.script_value = Some(result.clone());
    state
        .script_runtime
        .variables
        .insert("_npc_trade_result".to_string(), result);
}

fn compiled_mail_message(
    entries: &[serde_json::Value],
    first_entry_is_mail_item: bool,
) -> Result<String> {
    if entries.is_empty() {
        anyhow::bail!("compiled mail definition has no entries");
    }
    Ok(entries
        .iter()
        .skip(usize::from(first_entry_is_mail_item))
        .filter_map(|entry| {
            entry
                .get("args")
                .and_then(serde_json::Value::as_array)
                .and_then(|args| args.first())
                .and_then(serde_json::Value::as_str)
        })
        .map(strip_compiled_mail_text)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn required_map_attribute_label<'a>(
    map_name: &str,
    field_name: &str,
    value: &'a Option<String>,
) -> Result<&'a str> {
    let Some(label) = value.as_deref() else {
        anyhow::bail!("missing {field_name} for map {map_name}");
    };
    validate_map_reference_token(label, &format!("map attributes {field_name}"))?;
    Ok(label)
}

fn connection_destination_tile(
    source_tile: crystal_core::world::map::TilePosition,
    direction: &str,
    offset: i32,
    target_attributes: &MapAttributes,
) -> Result<crystal_core::world::map::TilePosition> {
    let (target_x, target_y, max_x, max_y) =
        connection_destination_tile_components(source_tile, direction, offset, target_attributes)?;
    let min_tile = 0;
    if target_x < min_tile || target_x > max_x || target_y < min_tile || target_y > max_y {
        anyhow::bail!(
            "connection destination tile ({target_x}, {target_y}) is outside target map tile bounds {min_tile}..={max_x}, {min_tile}..={max_y}"
        );
    }
    let target_x = i16::try_from(target_x)
        .with_context(|| format!("connection destination x {target_x} overflows runtime tile"))?;
    let target_y = i16::try_from(target_y)
        .with_context(|| format!("connection destination y {target_y} overflows runtime tile"))?;
    Ok(crystal_core::world::map::TilePosition::new(
        target_x, target_y,
    ))
}

fn canonical_global_phone_script_body(label: &str, payload: &Value) -> Result<Option<Value>> {
    let entries = payload
        .as_array()
        .with_context(|| format!("global phone script {label} body must be a command array"))?;
    let mut body = Vec::new();
    for (command_index, entry) in entries.iter().enumerate() {
        let command = entry
            .get("command")
            .and_then(Value::as_str)
            .with_context(|| {
                format!("global phone script {label} command {command_index} has no command name")
            })?;
        let args = entry
            .get("args")
            .and_then(Value::as_array)
            .with_context(|| {
                format!(
                    "global phone script {label} command {command_index} {command} has no argument array"
                )
            })?;
        body.push(entry.clone());
        if global_phone_body_terminates(command, args.len()) {
            break;
        }
    }
    if body.iter().any(|entry| {
        entry
            .get("command")
            .and_then(Value::as_str)
            .is_some_and(global_phone_cpu_or_data_command)
    }) {
        return Ok(None);
    }
    Ok((!body.is_empty()).then_some(Value::Array(body)))
}

fn global_phone_body_terminates(command: &str, arg_count: usize) -> bool {
    matches!(
        command,
        "end"
            | "endcallback"
            | "farjumptext"
            | "farsjump"
            | "jumptext"
            | "jumptextfaceplayer"
            | "jumpstd"
            | "sjump"
            | "return"
            | "done"
            | "text_end"
            | "prompt"
    ) || command == "ret" && arg_count == 0
}

fn global_phone_cpu_or_data_command(command: &str) -> bool {
    matches!(
        command,
        "adc"
            | "add"
            | "and"
            | "bit"
            | "call"
            | "ccf"
            | "cp"
            | "cpl"
            | "daa"
            | "db"
            | "dba"
            | "dbw"
            | "dec"
            | "di"
            | "dn"
            | "ds"
            | "dw"
            | "ei"
            | "farcall"
            | "halt"
            | "hlcoord"
            | "inc"
            | "jp"
            | "jr"
            | "ld"
            | "ldh"
            | "nop"
            | "or"
            | "pop"
            | "push"
            | "res"
            | "ret"
            | "reti"
            | "rl"
            | "rla"
            | "rlc"
            | "rlca"
            | "rr"
            | "rra"
            | "rrc"
            | "rrca"
            | "rst"
            | "sbc"
            | "scf"
            | "set"
            | "sla"
            | "sra"
            | "srl"
            | "sub"
            | "swap"
            | "xor"
    )
}

const RECEIVE_CALL_DELAYS_MINUTES: [u8; 4] = [20, 10, 5, 3];

fn restart_receive_call_delay(state: &mut GameState, reset_cycles: bool) {
    let day = state.time.current_day % 140;
    let hour = state.time.registers.hours;
    let minute = state.time.registers.minutes;
    let timer = &mut state.script_runtime.phone_call_timer;
    if reset_cycles {
        timer.time_cycles_since_last_call = 0;
    }
    let cycle = usize::from(timer.time_cycles_since_last_call.min(3));
    timer.initialized = true;
    timer.minutes_remaining = RECEIVE_CALL_DELAYS_MINUTES[cycle];
    timer.last_day = day;
    timer.last_hour = hour;
    timer.last_minute = minute;
}

fn check_receive_call_timer(state: &mut GameState) -> bool {
    if !state.script_runtime.phone_call_timer.initialized {
        restart_receive_call_delay(state, true);
        return false;
    }

    let now_day = u32::from(state.time.current_day % 140);
    let now_hour = u32::from(state.time.registers.hours);
    let now_minute = u32::from(state.time.registers.minutes);
    let timer = &mut state.script_runtime.phone_call_timer;
    let last = u32::from(timer.last_day) * 24 * 60
        + u32::from(timer.last_hour) * 60
        + u32::from(timer.last_minute);
    let now = now_day * 24 * 60 + now_hour * 60 + now_minute;
    let elapsed = if now >= last {
        now - last
    } else {
        now + 140 * 24 * 60 - last
    };
    timer.last_day = now_day as u8;
    timer.last_hour = now_hour as u8;
    timer.last_minute = now_minute as u8;

    if elapsed < u32::from(timer.minutes_remaining) {
        timer.minutes_remaining -= elapsed as u8;
        return false;
    }

    timer.minutes_remaining = 0;
    timer.time_cycles_since_last_call = timer.time_cycles_since_last_call.saturating_add(1).min(3);
    restart_receive_call_delay(state, false);
    true
}

fn connection_destination_tile_in_bounds(
    source_tile: crystal_core::world::map::TilePosition,
    direction: &str,
    offset: i32,
    target_attributes: &MapAttributes,
) -> Result<bool> {
    let (target_x, target_y, max_x, max_y) =
        connection_destination_tile_components(source_tile, direction, offset, target_attributes)?;
    Ok(target_x >= 0 && target_x <= max_x && target_y >= 0 && target_y <= max_y)
}

fn connection_destination_tile_components(
    source_tile: crystal_core::world::map::TilePosition,
    direction: &str,
    offset: i32,
    target_attributes: &MapAttributes,
) -> Result<(i32, i32, i32, i32)> {
    let metatile_width = i32::from(METATILE_WIDTH);
    let offset_tiles = offset
        .checked_mul(metatile_width)
        .with_context(|| format!("connection offset {offset} overflows runtime tile space"))?;
    let width = i32::from(target_attributes.width)
        .checked_mul(metatile_width)
        .with_context(|| {
            format!(
                "connection target map {} width overflows runtime tile space",
                target_attributes
                    .map_constant
                    .as_deref()
                    .unwrap_or("<unknown>")
            )
        })?;
    let height = i32::from(target_attributes.height)
        .checked_mul(metatile_width)
        .with_context(|| {
            format!(
                "connection target map {} height overflows runtime tile space",
                target_attributes
                    .map_constant
                    .as_deref()
                    .unwrap_or("<unknown>")
            )
        })?;
    let (target_x, target_y) = match direction {
        "north" => (
            i32::from(source_tile.x)
                .checked_sub(offset_tiles)
                .context("north connection destination x overflows runtime tile space")?,
            height - 1,
        ),
        "south" => (
            i32::from(source_tile.x)
                .checked_sub(offset_tiles)
                .context("south connection destination x overflows runtime tile space")?,
            0,
        ),
        "west" => (
            width - 1,
            i32::from(source_tile.y)
                .checked_sub(offset_tiles)
                .context("west connection destination y overflows runtime tile space")?,
        ),
        "east" => (
            0,
            i32::from(source_tile.y)
                .checked_sub(offset_tiles)
                .context("east connection destination y overflows runtime tile space")?,
        ),
        other => anyhow::bail!("unsupported connection direction '{other}'"),
    };
    let max_x = width - 1;
    let max_y = height - 1;
    Ok((target_x, target_y, max_x, max_y))
}

fn trainer_ai_type_matchup(
    table: &TypeEffectivenessTable,
    move_type: &str,
    defender_types: &[String],
    identified: bool,
) -> crystal_core::battle::ai::TrainerAiTypeMatchup {
    let mut effectiveness_num = 1_u32;
    let mut effectiveness_den = 1_u32;
    for entry in table.matchups.iter().chain(
        (!identified)
            .then_some(table.foresight_matchups.iter())
            .into_iter()
            .flatten(),
    ) {
        if entry.attacker == move_type
            && defender_types
                .iter()
                .any(|defender| defender != "NONE" && defender == &entry.defender)
        {
            effectiveness_num = effectiveness_num
                .saturating_mul(u32::from(entry.multiplier.numerator));
            effectiveness_den =
                effectiveness_den.saturating_mul(u32::from(entry.multiplier.denominator.max(1)));
        }
    }
    if effectiveness_num == 0 {
        crystal_core::battle::ai::TrainerAiTypeMatchup::Immune
    } else if effectiveness_num > effectiveness_den {
        crystal_core::battle::ai::TrainerAiTypeMatchup::SuperEffective
    } else if effectiveness_num < effectiveness_den {
        crystal_core::battle::ai::TrainerAiTypeMatchup::NotVeryEffective
    } else {
        crystal_core::battle::ai::TrainerAiTypeMatchup::Neutral
    }
}

fn trainer_switch_type_matchup(
    table: &TypeEffectivenessTable,
    move_type: &str,
    defender: &Pokemon,
) -> Result<i8> {
    let defender_types = if defender.species.type1 == defender.species.type2 {
        vec![defender.species.type1.clone()]
    } else {
        vec![
            defender.species.type1.clone(),
            defender.species.type2.clone(),
        ]
    };
    let multiplier = crystal_core::battle::damage::calculate_type_effectiveness_multiplier(
        table,
        move_type,
        &defender_types,
    )
    .map_err(|error| anyhow::anyhow!("trainer switch type matchup: {error:?}"))?;
    if multiplier.numerator == 0 {
        return Ok(-2);
    }
    Ok(match multiplier.numerator.cmp(&multiplier.denominator) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    })
}

fn read_json_file<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn write_compiled_game_pack(path: impl AsRef<Path>, pack: &CompiledGamePack) -> Result<()> {
    let path = path.as_ref();
    validate_compiled_game_pack_path(path)?;
    if pack.format_version != COMPILED_GAME_PACK_FORMAT_VERSION {
        anyhow::bail!(
            "compiled game pack {} has unsupported format version {}",
            path.display(),
            pack.format_version
        );
    }
    validate_compiled_game_pack_identity(pack)
        .with_context(|| format!("validate compiled game pack identity {}", path.display()))?;
    let mut serialized_pack = pack.clone();
    compress_pack_audio(&mut serialized_pack)?;
    serialized_pack.identity = derive_compiled_game_pack_identity_from_manifest(
        serialized_pack.format_version,
        &serialized_pack.data,
        &serialized_pack.audio_manifest,
        &serialized_pack.runtime_files,
        &serialized_pack.report,
    )?;
    write_serialized_compiled_game_pack(path, &serialized_pack)
}

fn write_serialized_compiled_game_pack(path: &Path, serialized_pack: &CompiledGamePack) -> Result<()> {
    let mut encoded = Vec::new();
    ciborium::into_writer(serialized_pack, &mut encoded)
        .context("encode compiled game pack")?;
    if encoded.len() > u32::MAX as usize {
        anyhow::bail!(
            "compiled game pack exceeds binary payload length field"
        );
    }
    let mut bytes = Vec::with_capacity(COMPILED_GAME_PACK_HEADER_LEN + encoded.len());
    bytes.extend_from_slice(COMPILED_GAME_PACK_MAGIC);
    bytes.extend_from_slice(&COMPILED_GAME_PACK_FORMAT_VERSION.to_be_bytes());
    bytes.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&fnv1a32_bytes(&encoded).to_be_bytes());
    bytes.extend_from_slice(&encoded);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create compiled game pack directory {}", parent.display()))?;
    }
    std::fs::write(path, bytes)
        .with_context(|| format!("write compiled game pack {}", path.display()))
}

#[cfg(any(test, feature = "test-fixtures"))]
pub fn write_compiled_game_pack_for_tests(
    path: impl AsRef<Path>,
    pack: &CompiledGamePack,
) -> Result<()> {
    write_compiled_game_pack(path, pack)
}

#[cfg(any(test, feature = "test-fixtures"))]
pub fn write_compiled_game_pack_with_midi_audio_for_tests(
    path: impl AsRef<Path>,
    pack: &CompiledGamePack,
) -> Result<()> {
    write_compiled_game_pack_with_midi_audio(path, pack)
}
