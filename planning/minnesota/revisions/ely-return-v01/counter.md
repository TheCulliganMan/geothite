# Preparing for the fourth match at Ely

I traced Finder Lesson Intro from both League entrances to the Hall register. The registrar checks the eight badges individually, then checks the intake survey. If the survey is unfinished, the clerk gives directions based on the player's latest handoff. A qualified challenger learns there are five consecutive fights, can decline, and gets directions to the clinic and Mart. Both counters are reachable from either lobby exit. Ely's nurse heals the party, the clinic PC opens storage, and a free Full Restore is available under the pines. [Registration evidence](../ely-challenge-v01/ely.md) · [Service and street check](../league-journey-v01/check-preparation.py)

Ely stocks the same seven items as Crystal's Indigo Plateau, including Revives, Full Heals and three strengths of HP medicine. Crystal puts its clerk and nurse together and has a trainer explain the loss condition nearby. Polished Crystal keeps that service arrangement and warning. At Ely the registrar covers the risk before entry and the services remain in town. I found no missing medicine tier in those shops. [Crystal's shop inventory](https://github.com/pret/pokecrystal/blob/master/data/items/marts.asm) · [Crystal's lobby](https://github.com/pret/pokecrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm) · [Polished Crystal's lobby](https://github.com/Rangi42/polishedcrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm)

| Room | What the player can do |
| --- | --- |
| Fern | Registration sends the challenger into her room. Speaking to Fern starts the first battle. Returning to the lobby ends the run. |
| Flint | Fern's victory opens the way. Flint names Veil after the second win. The challenger can return to Fern's room with both wins. |
| Veil | After the third win she tells the challenger that Halden gives people a minute before their match. |
| Halden | His original greeting promised to wait, then started the battle as soon as the text was acknowledged. The new invitation asks if the challenger is ready. No leaves the three wins intact and frees the player to use their own medicine or choose a different lead. When approached again, Halden goes straight to the question. Yes starts the fourth match. His championship rematch keeps its original team. |
| Casey and the register | Four wins admit the challenger to Casey. Victory opens the Hall. The register requires all five wins in the active run and records the party. |

Bruno's original Crystal room and its Polished Crystal counterpart make speaking to him the start signal. Neither version offers a pause in his greeting. Ely's choice follows the specific promise Veil and Halden make. A prior refusal only changes the length of the greeting when the challenger next reaches Halden with three wins. It grants no healing or additional access. [Crystal's Bruno](https://github.com/pret/pokecrystal/blob/master/maps/BrunosRoom.asm) · [Polished Crystal's Bruno](https://github.com/Rangi42/polishedcrystal/blob/master/maps/BrunosRoom.asm) · [Halden's branch and room-preservation check](check-halden.py)

From the repository root, the first command checks the actual approach to Ely's services. The second compares both Halden scripts, native control targets, text pages, and the rest of his room against the preceding edition.

```sh
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/league-journey-v01/check-preparation.py "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/ely-return-v01/check-halden.py "$parent" "$child"
```

The native regression seeds a three-win run with a wounded CINDORE. It checks immediate acceptance, declining and healing 35 HP with one Super Potion, saving and continuing, and reaching the first battle command after both openings. Both selected editions passed. The Intro LCD showed KNOTITAN at level 50 against CINDORE at level 45 with 65/100 HP on immediate acceptance and 100/100 HP after using the medicine. Both images showed the four commands.

```sh
MN_BADGE_TRAILS_PACK="$child" MN_ELY_RETURN_FRAMES="$frames" cargo test --locked -p crystal-bevy --lib halden_readiness_ -- --ignored --test-threads=1
```
