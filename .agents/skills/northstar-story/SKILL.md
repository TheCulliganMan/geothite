---
name: northstar-story
description: Write and revise the Northstar (Minnesota) main story, villains, rival, and NPC dialogue in Geothite. Use for plot changes, character arcs, Team Lode / Team ICE scenes, Reed, Kettlurk, gym leader lines and city dialogue in planning/minnesota/story.json.
---

# Northstar story

The canonical story and every revised line live in
`planning/minnesota/story.json`. `narrative`, `allied_antagonists`, `bosses`
and `rival_arc` describe intent; `dialogue_overhaul.texts[MAP][LABEL]` holds the
shipped pages. Change writing there rather than in Rust whenever possible.

## Fixed character decisions

- **REED stays a villain for the whole game.** He is the rival who chooses
  Team ICE and then Team Lode for pay, status and a stronger team. He never
  redeems, apologizes, switches sides, helps the player, thanks the player,
  frees anyone by choice, or "needs to think". Losses make him bitter and
  push him to the next job; they do not reform him. He may be hurt that Lode
  cheats him, but he blames the player and keeps working for the villains.
  Do not soften him in post-game or epilogue lines either.
- **DIRECTOR HALE** (male, `SPRITE_ROCKET`) leads Team Lode and the DEEP BORE
  project. He is the `MN_LODE_CHIEF` / `MN_SURVEY_CHIEF` battle at the Boundary
  Waters shelter. Seed his name before the shelter (Ivy, courier, foreman,
  Bemidji, Duluth harbor) so the final fight is someone the player knows.
  Calm, competent and certain; he knows the drainage destroys the lakes and
  chooses it anyway. No cartoon greed speeches and no redemption.
- **VOSS** is the Team ICE captain in Minneapolis (`MN_ICE_ENFORCER`). Terse,
  coercive, fake police authority, no ice puns, no comic incompetence.
- The courier and foreman stay loyal to Lode and self-protective.
- **Kettlurk** is a captive, never a villain. The player frees it at the
  intake but never catches, battles or registers it in the main story.

## Writing rules

- Show the harm: falling rivers, stranded Pokemon, docks over mud, empty nets,
  rings on pier posts. Escalate it as the player moves north.
- Ordinary people talk about their lives, town and Pokemon. Keep survey
  "records", "copies" and "logs" to Tamarack, Bemidji and the depot evidence.
- People are not omnipotent trail guides. Nobody blocks a road just to give
  directions; NPCs only know what they could plausibly know.
- Keep the progression order, flags, maps, battles and parties unless the
  user asks for gameplay changes.
- Keep Minnesota creature names to the active roster (see `AGENTS.md`).

## Native text limits

Each page is one or two lines; each line is ASCII, 1-18 characters, with no
`"`, `<`, `>` or control characters (`authored_dialogue::compile`). Check with:

```sh
cargo test --locked -p crystal-assets canonical_dialogue_fits_native_pages
```

Before rewording a line, grep `crates/` for its text: some Bevy tests match
exact substrings (for example Bemidji's "ink blot" evidence).

Preserve the file's formatting when editing with a script: two-space indent,
`ensure_ascii` matching the existing file, and the trailing newline.
