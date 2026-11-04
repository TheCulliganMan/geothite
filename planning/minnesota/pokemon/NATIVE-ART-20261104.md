# Native art and media — 2026-11-04

Published [Pokédex](https://geothite.ryanculligan.com/pokedex/) release `20261104-rerender-181-186-45`:225 active species across99 families,121 archived designs,225 walking thumbnails and225 original cries. This is a static Pokédex publication. The game container, start time, configuration and selected campaign pack are unchanged.

The selected thematic-refinement manifests for MN_F070, MN_F080, MN_F088, MN_F091 and MN_F132 replace eight requested front/back pairs. Pothollow retains its prairie-pothole living-filter theme and Water/Poison typing; the new art has layered filter folds, a separate sludge chamber, dithered rounded flippers and two stalk eyes. Guarded native edits retain the pupils. Mudnip and Bankmaw gain stronger warm-gold/teal and jade/teal palettes. Loomlet, Pipulse/Cadenset and all other battle sprites retain their selected sources.868 other active normal/shiny battle PNGs are byte-identical to release42. Casserogre is unchanged.

The overworld registry contains225 current active species:43 newly required sets and six revisions, with176 prior sets retained. Each bank has six16×16 poses,384 native2bpp bytes, transparent index0 and species-specific normal/shiny palettes. Retired registry entries and superseded selections remain recorded. Original ImageGen sheets/prompts are preserved; native outputs, context captures and packs remain ignored.

The opt-in field converter preserves enclosed source facial marks without hand-painting field pixels. The opt-in battle converter carries fractional source shading into native ordered dither. Existing conversion defaults remain unchanged.

Verification completed:

- Nine sprite-conversion tests, eight field-conversion tests and two source-provenance tests pass. Clippy for crystal-assets/all targets and workspace formatting checks pass with existing repository warnings.
- Independent palette/index/PNG/2bpp/provenance verification passes for all22 views in the five touched families, including unchanged stages. The broader historical-art audit still encounters a pre-existing missing immutable JSON version for MN_101.
- Nine real renderer batches cover all49 new/revised field sets in all eight directional standing/stepping contexts and every shiny bank. Source/native normal/shiny boards and terrain captures were visually reviewed.
- The verified519-species local pack refresh changes only34 runtime graphics/palette files for the eight selected species. Game data, all compiled audio, PCM metadata, frame counts and loop ranges are identical to the voiced225 base.
- Public browser checks verify225 active cards, archive filtering,450 decoded normal/shiny walking sheets, eight changed normal/shiny front/back pairs, actual walking frame changes and mobile bounds.
- All225 public WAVs decode to non-silent audio; the actual Pothollow Cry button plays to completion. Cry assets reuse the preserved release42 media directory.

Retained local evidence: `target/overworld-225-20261104/`. Selected native field provenance is recorded in `overworld/20261027-v1/accepted.json`; battle source provenance is recorded in `sprite-generation.json` and the selected family manifests.

## Releases44–45: native conversion and bounded crop

Release44 refines all six Cindore/Pelletor/Mesablast views, preserving designs and output palettes, and restores Cascavolt’s clipped front forelimb with a wider source crop. Release45 re-renders current public #181–186: Knotlet, Lasserp, Knotitan, Taconib, Pelletank and Taconarch. Existing sources, types and size budgets are preserved. The opt-in source-tone converter represents intermediate source colors using ordered mixtures of the four existing inks, protecting facial white/black marks and paper boundaries. Pelletank uses calibrated source light to avoid excessive white checker highlights.

Ten conversion tests, two provenance tests, crystal-assets clippy and formatting pass. Independent native verification passes for24 views across all four touched families. Release45 changes exactly24 battle PNGs from44; HTML changes only immutable release URLs, and catalog/media bytes are retained. Mobile browser checks decode all24 normal/shiny front/back images and walking sheets without overflow. Originals remain selected-history inputs. These static publications do not update the game pack.
