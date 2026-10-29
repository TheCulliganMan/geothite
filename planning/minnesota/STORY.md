# The Northstar Survey

A trainer from Winona takes Dr Tamarack's water readings to Ivy in Rochester.
Ellis, who was bringing their survey meters, has gone missing. Looking for him
leads the player to Team ICE, stolen equipment and Team Lode's plan to drain
the Boundary Waters for copper and nickel. Lode has trapped Kettlurk to power
the pumps. The player helps the crews stop the operation while earning eight
badges on the journey to the Ely League.

This is the story profile. [Progression](PROGRESSION.md) describes the route and
quest gates; [gym profiles](GYMS.md) describe the eight courses. Build commands,
pack identities and recorded checks belong in the
[current story composition record](revisions/current-story-v01/README.md).
That record covers a particular publication. It does not certify later source
edits or an uninterrupted, ordinary-party campaign.

## Sources and writing conventions

Read the sources in the order used by the Rust composition:

- [story.json](story.json) supplies the base quest and badge contracts. Its
  `dialogue_overhaul.texts` section supplies the final revised dialogue for the
  labels it contains; those pages override earlier versions of the same label.
- [story-events-v2.json](story-events-v2.json) supplies the acted opening,
  optional egg and character requests. [story-arc-v3.json](story-arc-v3.json)
  adds Reed's recurring scenes and the witnessed intake release.
- [story-pokemon.json](story-pokemon.json) binds regional appearances to
  [the roster](pokemon/roster.json), excluding
  [retired families](pokemon/roster-curation.json). Legacy species in base
  records are import inputs, not approved Minnesota teams.
- [world.json](world.json) supplies regional connections and revision pointers.
  The story builder in
  [northstar_current_story.rs](../../crates/crystal-assets/src/northstar_current_story.rs)
  applies the chapter overlays, regional binding, League dialogue and final
  dialogue revision. A Markdown profile cannot override those contracts.

Use ordinary, direct speech. Give each person a reason to speak, a concrete
request and a response to what the player did. Keep directions short enough to
remember. Minnesota identity comes from the river towns, prairie, lake country,
working communities and people helping one another. Avoid forced accents,
catchphrases, invented local customs and biographies unsupported by the source.

Use **St. Paul**, **St. Cloud**, **Dr Tamarack** and **Pokémon** in prose. Preserve
native uppercase text and exact map, trainer, species and event identifiers when
quoting data. Use the supplied character names without respelling them. Halden
is the fourth League trainer; unrelated trainers must not become Reed.

Write a planned scene as a plan, an implemented scene as source behavior, and a
test result as a recorded result tied to its pack. Do not turn a screenshot,
structural audit, boosted party or chapter fixture into a claim of a completed
campaign. Keep historical commands and saves attached to their original version.

## People and their roles

| Character | What the source establishes | Role in the journey |
| --- | --- | --- |
| Mom | Packs lunch, asks to meet the new partner and welcomes the player home | Gives departure and return a personal reason; home offers rest |
| Dr Tamarack | Studies river Pokémon, works with Ivy and tends the egg Reed brings in | Entrusts the original readings to the player; offers the optional Gillpup egg |
| Ivy | Keeps the original readings safe and looks for the missing crew | Sends the player to Jo, later joins the intake crew and helps confirm their safety |
| Ellis | Survey dispatcher whose equipment was diverted; imprisoned after asking questions | Needs a physical rescue, returns home and directs the player toward Mina |
| Jo | Ellis's partner in Minneapolis | Explains where Ellis was taken and responds to his return |
| Mina | Handles Union Depot records and stops the next Lode shipment | Challenges the intimidation and asks the player to follow the equipment north |
| Nell | Waits above the mine while Finn is still below | Asks the player to stop the test; reacts when Finn gets out |
| Finn | Works with the endangered crew and recovers Ellis's meter | Leaves after shutdown, then offers the meter for repair and log recovery |
| Sera | Refuses a forced transfer at St. Cloud | The player can intervene and see her leave custody |
| Reed | Wants a strong team, paid work and authority over other people | Repeatedly chooses ICE and Lode despite seeing whom they harm |

These are fictional game characters. Do not add a hometown, family connection,
occupation or personal history merely to fill out a profile. Keep art direction
in the [cast records](npcs/cast/README.md); appearance does not establish a new
story role.

## Reed's choices

Reed's six battles occur in Winona, Rochester, the ICE relay, St. Paul, the mine
and the Boundary intake. His appearances between battles explain his movement
through the region:

1. In Winona he dismisses the delivery errand and asks whether the egg he found
   could be sold. Tamarack takes care of it.
2. In Rochester he names the ICE job in Minneapolis. He wants the recognition
   and backing he thinks the player received too easily.
3. At the Minneapolis interview he sees Ellis imprisoned, hears why and accepts
   the job anyway. In the relay he guards the holding controls.
4. After losing, he takes Lode's shipment work in St. Paul. At St. Cloud he uses
   the counterfeit badge to threaten Sera until the player intervenes.
5. At the mine he hears Finn's warning and still protects the running drill,
   hoping for promotion.
6. At the intake he knows what is in the cage and guards the local drive.
   Losing the battle removes his obstruction; it does not stop the machine.

Reed remains resentful after the operation fails. His optional station return
is an aftermath scene, not a redemption or a League appointment.

## Team Lode and Team ICE

Lode's leaders want the ore beneath the lakes. They divert survey equipment,
falsify readings, continue dangerous tests and pay ICE to keep people away.
The courier demands the original records and their copies. The foreman sees
the meter spike and orders the next test. The chief admits ordering the false
surveys and paying ICE. Their decisions establish responsibility without a
comic villain speech or an invented excuse.

Team ICE uses counterfeit police authority to seize equipment, imprison people
and force transfers. Reed sees this before joining. The teams' wrongdoing is
specific to these fictional organizations; ordinary workers and Minnesota towns
are not collectively responsible. The people who help expose Lode include the
workers whose safety it disregards.

## Chapter sequence

| Chapter | What changes | Player action that matters |
| --- | --- | --- |
| Winona and Rochester | A first delivery becomes concern for a missing person | Choose Pouchid, Kindlit or Birchit; deliver the sealed originals to Ivy; meet Reed |
| Minneapolis | Jo's missing partner is found in ICE custody | Navigate the headquarters, win the guarded battles, disable holding power and separately open Ellis's cell |
| St. Paul and St. Cloud | Diverted equipment and forced transfers connect ICE to Lode | Confront the shipment guards, acknowledge Mina's request and intervene for Sera |
| Brainerd, Itasca and Bemidji | Independent records expose copied surveys | Obtain the reference and complete the officer's comparison; the Mankato notebook is supporting evidence |
| Iron Range and Grand Rapids | The supposed surveys are pump tests conducted over the crew's objections | Work the mine controls, defeat the guards, stop the drive, speak to Finn and hand the recovered meter to the depot technician |
| Duluth and the North Shore | The pump design reveals Kettlurk's intended use and the threat to the lakes | Earn Harbor, show the depot log to the harbor crew and follow their directions through Grand Marais |
| Boundary Waters | A warning becomes a coordinated rescue | Defeat the chief, acknowledge the live link, reach the intake, defeat Reed, stop the local drive and witness Kettlurk and the crew move clear |
| Ely and return visits | The crews can assess the damage; the gym journey ends in the League | Complete the League and Hall of Fame; revisit allies who react to the outcome |

The optional Gillpup egg, Mankato notebook, Fair, fishing, cabin visits and
Safari expedition provide reasons to explore. They cannot replace required
rescues or make paid admission a condition of finishing the story.

## Ending and persistence

The chief battle and radio link grant access to the intake. They do not restore
the survey. The intake has its own Reed battle and drive control. Kettlurk
enters the water, Ivy and Finn move clear, and the final acknowledged crew
sequence completes restoration. A catch, Pokédex entry or Safari permit is
never required. The optional Devil's Kettle encounter remains a separate branch.

Stopping the pumps prevents the planned drainage. It does not instantly repair
every habitat or erase the harm. The return dialogue lets the crews assess the
damage and the player go home.

Use persisted `EVENT_MN_*` events and production battle results. Refusal, a
loss, a full party or a full bag must leave the relevant action retryable.
Once-only rewards commit after successful delivery; completed scenes do not
repeat their rewards. Continue must restore opened doors, controls and moved
actors. Keep keyboard and MCP on `VisibleShellController`.

Kettlurk, its returning current and the drainage machinery are game fiction.
Itasca provides independent comparison records; it is not a pipe connecting
Minnesota's separate drainage basins. Do not present the story as an explanation
of real Devil's Kettle hydrology or attribute its crimes to a real company.

## Evidence and remaining work

The [current composition record](revisions/current-story-v01/README.md) reports
an earned New Game opening, local rival and grunt cases, rescue and release
checks, stdio replays and audio preservation for its named pack. The
[interior composition](interior-life/current-story-v01/README.md) records a
separate child and its own checks. These records retain their stated fixtures,
identities and limits; this prose cleanup does not rerun or extend them.

A full uninterrupted Winona-to-Hall-of-Fame run with an ordinary party and
complete difficulty acceptance remain unclaimed. Changed dialogue also needs
checks against the resulting pack; old page counts and screenshots cannot
certify newly authored pages. Earlier chapter packs and boosted playthroughs
remain historical evidence only.
