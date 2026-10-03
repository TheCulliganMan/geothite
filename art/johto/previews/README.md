# Rendered asset previews

These previews render the editable source meshes. They are authoring views;
map lighting, placement, collisions and character scale are verified separately
in the game. More native source files and previews are being converted in this PR.

## Kanto capped posts

![Capped post sculpture and independently spaced rows](kanto-capped-posts.png)

- [Open the editable Blender source](../source/kanto-capped-posts.blend)
- [View the generator](../../../tools/build-kanto-capped-posts.py)
- [Read the coverage notes](../../../docs/3d-remaining-families.md)

The source scene includes a larger display copy on the left. The ordinary course
shows the independent posts and gaps. The `.blend` is a normal, losslessly
compressed Blender file and opens directly in Blender without a reconstruction
step. This first native file preserves the previous source's exact bytes; the
remaining source-storage migration is still in progress.
