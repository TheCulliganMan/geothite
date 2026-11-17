# Wreckquiem front candidate

User feedback: this revision is better, but the forehead highlight dropping down
into the eye is unwanted.

Applied to the downsized 56x56 front only: native pixels (21,14) and (22,14)
changed from white to black to stop the descending highlight at the eye boundary.
Guarded edits are recorded in `manifest-candidate.json`. Exactly those two pixels
changed; the eye pixels, pupil, pose and all remaining pixels are unchanged.
PNG and 2bpp tile data were independently checked against the indexed output.
This is specific to this asset, not a general skill rule.

Preview: `target/wreckquiem-definition-20261103/manual-fix/mn-094-wreckquiem-definition-20261103-v1.front.review.png`.

Original source artwork and original selected manifest remain untouched. This front candidate has not
replaced the selected sprite; the matching back has not been regenerated.
