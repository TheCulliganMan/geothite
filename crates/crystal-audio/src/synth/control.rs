use super::*;
type Sources = BTreeMap<String, AudioSource>;
fn resolve(sources: &Sources, current: &str, target: &str) -> Option<(String, usize)> {
    if sources.contains_key(target) {
        return Some((target.into(), 0));
    }
    let scoped = if target.starts_with('.') {
        format!("{}{}", current.split('.').next()?, target)
    } else {
        target.into()
    };
    for (name, source) in sources {
        for (index, c) in source.commands.iter().enumerate() {
            if c.command != "label" {
                continue;
            }
            let label = c.args.first()?;
            if (name == current && label == target)
                || label == &scoped
                || (label.starts_with('.') && format!("{name}{label}") == scoped)
            {
                return Some((name.clone(), index + 1));
            }
        }
    }
    None
}
fn reference(c: &AudioCommand) -> Result<Option<&str>> {
    let index = match c.command.as_str() {
        "sound_call" | "sound_jump" => 0,
        "sound_loop" => 1,
        _ => return Ok(None),
    };
    Ok(Some(c.args.get(index).with_context(|| {
        format!("{} missing target", c.command)
    })?))
}
pub fn validate_references(data: &MusicData) -> Result<()> {
    let sources = data.sources();
    for (name, s) in &sources {
        for c in &s.commands {
            if let Some(target) = reference(c)? {
                ensure!(
                    resolve(&sources, name, target).is_some(),
                    "unresolved {} target {target} in {name}",
                    c.command
                );
            }
        }
    }
    Ok(())
}
/// Link every call, jump, and counted/infinite loop against the canonical catalog.
/// Auxiliary channel bodies remain sources, never additional playing channels.
pub fn link(program: &Program, catalog: &BTreeMap<String, AudioSource>) -> Result<Program> {
    let mut linked = program.clone();
    loop {
        let sources = linked.music_data.sources();
        let mut additions = Sources::new();
        for (name, s) in &sources {
            for c in &s.commands {
                if let Some(target) = reference(c)? {
                    if resolve(&sources, name, target).is_some() {
                        continue;
                    }
                    let (owner, _) = resolve(catalog, name, target).with_context(|| {
                        format!("unresolved {} target {target} in {name}", c.command)
                    })?;
                    ensure!(
                        !sources.contains_key(&owner),
                        "source {owner} is incomplete for {target}"
                    );
                    additions.insert(owner.clone(), catalog[&owner].clone());
                }
            }
        }
        if additions.is_empty() {
            break;
        }
        linked.music_data.shared_sources.extend(additions);
    }
    validate_references(&linked.music_data)?;
    Ok(linked)
}
#[derive(Clone, Hash, PartialEq, Eq)]
struct Frame {
    src: String,
    pc: usize,
    loops: BTreeMap<(String, usize), i32>,
}
/// Expand control flow until the complete call/finite-loop state repeats.
/// Loop labels are arbitrary ASM symbols; their spelling has no runtime meaning.
pub(super) fn expand(data: &MusicData, channel: &str) -> Result<Vec<AudioCommand>> {
    let sources = data.sources();
    ensure!(sources.contains_key(channel), "missing channel {channel}");
    let mut stack = vec![Frame {
        src: channel.into(),
        pc: 0,
        loops: BTreeMap::new(),
    }];
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    for _ in 0..2_000_002 {
        if stack.is_empty() {
            return Ok(out);
        }
        ensure!(
            stack.len() < 64,
            "audio call stack exceeded limit in {channel}"
        );
        if let Some(index) = seen.insert(stack.clone(), out.len()) {
            out.insert(
                index,
                AudioCommand {
                    command: "__loop_point__".into(),
                    args: vec![],
                },
            );
            return Ok(out);
        }
        let frame = stack.last_mut().unwrap();
        let source = &sources[&frame.src];
        let Some(c) = source.commands.get(frame.pc) else {
            stack.pop();
            continue;
        };
        frame.pc += 1;
        match c.command.as_str() {
            "label" => {}
            "sound_ret" => {
                stack.pop();
            }
            "sound_call" | "sound_jump" | "sound_loop" => {
                let target = reference(c)?.unwrap();
                let (src, pc) = resolve(&sources, &frame.src, target)
                    .with_context(|| format!("missing audio target {target}"))?;
                if c.command == "sound_call" {
                    stack.push(Frame {
                        src,
                        pc,
                        loops: BTreeMap::new(),
                    });
                    continue;
                }
                if c.command == "sound_loop" {
                    let count = arg(c, 0)?;
                    ensure!(count >= 0, "negative sound_loop count");
                    if count != 0 {
                        let key = (frame.src.clone(), frame.pc - 1);
                        let left = frame.loops.entry(key.clone()).or_insert(count - 1);
                        if *left == 0 {
                            frame.loops.remove(&key);
                            continue;
                        }
                        *left -= 1;
                    }
                }
                frame.src = src;
                frame.pc = pc;
            }
            _ => out.push(c.clone()),
        }
    }
    bail!("audio instruction limit exceeded in {channel}")
}
#[cfg(test)]
mod tests {
    use super::*;
    fn program(commands: &[(&str, &[&str])]) -> Program {
        Program {
            profile: "pokecrystal-midi-v1".into(),
            cry_pitch: None,
            cry_length: None,
            music_data: MusicData {
                channel_count: 1,
                channels: BTreeMap::from([(
                    "Test_Ch1".into(),
                    AudioSource {
                        number: Some(1),
                        commands: commands
                            .iter()
                            .map(|(c, a)| AudioCommand {
                                command: c.to_string(),
                                args: a.iter().map(|a| a.to_string()).collect(),
                            })
                            .collect(),
                    },
                )]),
                subroutines: BTreeMap::new(),
                shared_sources: BTreeMap::new(),
            },
        }
    }
    #[test]
    fn missing_targets_are_errors() {
        let p = program(&[("sound_loop", &["0", "Missing.body"])]);
        assert!(validate_references(&p.music_data).is_err());
        assert!(expand(&p.music_data, "Test_Ch1").is_err());
    }
    #[test]
    fn arbitrary_loop_label_and_intro() {
        let p = program(&[
            ("note", &["D_", "1"]),
            ("label", &[".body"]),
            ("note", &["C_", "1"]),
            ("sound_loop", &["0", ".body"]),
        ]);
        let c = expand(&p.music_data, "Test_Ch1").unwrap();
        assert_eq!(
            c.iter().map(|c| c.command.as_str()).collect::<Vec<_>>(),
            ["note", "__loop_point__", "note"]
        );
    }
    #[test]
    fn finite_mainloop_is_not_an_infinite_loop() {
        let p = program(&[
            ("label", &[".mainloop"]),
            ("note", &["C_", "1"]),
            ("sound_loop", &["3", ".mainloop"]),
            ("sound_ret", &[]),
        ]);
        let c = expand(&p.music_data, "Test_Ch1").unwrap();
        assert_eq!(c.len(), 3);
        assert!(c.iter().all(|c| c.command == "note"));
    }
    #[test]
    fn jumps_loop_and_subroutine_calls_return() {
        let p = program(&[
            ("sound_call", &[".sub"]),
            ("label", &[".body"]),
            ("note", &["C_", "1"]),
            ("sound_jump", &[".body"]),
            ("label", &[".sub"]),
            ("note", &["D_", "1"]),
            ("sound_ret", &[]),
        ]);
        let c = expand(&p.music_data, "Test_Ch1").unwrap();
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].args[0], "D_");
        assert_eq!(c[1].command, "__loop_point__");
    }
}

#[cfg(test)]
mod linking_tests {
    use super::*;
    fn source(commands: &[(&str, &[&str])]) -> AudioSource {
        AudioSource {
            number: Some(1),
            commands: commands
                .iter()
                .map(|(command, args)| AudioCommand {
                    command: command.to_string(),
                    args: args.iter().map(|arg| arg.to_string()).collect(),
                })
                .collect(),
        }
    }
    #[test]
    fn external_loop_bodies_link_transitive_calls_without_adding_channels() {
        let program = Program {
            profile: "pokecrystal-midi-v1".into(),
            cry_pitch: None,
            cry_length: None,
            music_data: MusicData {
                channel_count: 1,
                channels: BTreeMap::from([(
                    "Night_Ch1".into(),
                    source(&[("sound_loop", &["0", "Day_Ch1.body"])]),
                )]),
                subroutines: BTreeMap::new(),
                shared_sources: BTreeMap::new(),
            },
        };
        let catalog = BTreeMap::from([
            (
                "Day_Ch1".into(),
                source(&[
                    ("label", &[".body"]),
                    ("sound_call", &["Day_Ch1.sub1"]),
                    ("label", &[".mainloop"]),
                    ("note", &["C_", "4"]),
                    ("sound_loop", &["0", ".mainloop"]),
                ]),
            ),
            (
                "Day_Ch1.sub1".into(),
                source(&[("note", &["D_", "1"]), ("sound_ret", &[])]),
            ),
        ]);
        let linked = link(&program, &catalog).unwrap();
        assert_eq!(linked.music_data.channels.len(), 1);
        assert_eq!(linked.music_data.shared_sources.len(), 2);
        assert_eq!(link(&linked, &catalog).unwrap(), linked);
        let expanded = expand(&linked.music_data, "Night_Ch1").unwrap();
        assert_eq!(expanded.len(), 3);
        assert_eq!(expanded[0].args[0], "D_");
        assert_eq!(expanded[1].command, "__loop_point__");
        assert_eq!(expanded[2].args[0], "C_");
    }
    #[test]
    fn local_labels_never_resolve_to_another_channel() {
        let data = MusicData {
            channel_count: 2,
            channels: BTreeMap::from([
                ("Song_Ch1".into(), source(&[("sound_jump", &[".missing"])])),
                (
                    "Song_Ch2".into(),
                    source(&[("label", &[".missing"]), ("sound_ret", &[])]),
                ),
            ]),
            subroutines: BTreeMap::new(),
            shared_sources: BTreeMap::new(),
        };
        assert!(validate_references(&data).is_err());
    }
}
