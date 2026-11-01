"""Validate frozen preview ancestry after selecting a reviewed continuation.

Source previews still render their explicit plan and exact parent pack. This
guard checks the selected pack and each declared parent link; it does not turn
an earlier stage preview into a render of the complete current selection.
"""
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


def require_selected_ancestor(world, ancestor_revision, ancestor_sha256, root=ROOT):
    """Return the checked selection-to-ancestor chain, or raise ValueError.

    Every visited pack must match its recorded SHA256. Every descendant must
    bind a frozen build recipe and name the actual parent revision, path and hash.
    Unknown/disconnected
    selections, changed packs and cycles are refused before source rendering.
    """
    revision = world.get('active_local_review_revision')
    chain = []
    while revision not in chain:
        entry = world.get(revision)
        if not isinstance(entry, dict):
            raise ValueError(f'unknown selected lineage revision: {revision}')
        pack = Path(entry.get('pack', ''))
        if pack.is_absolute() or '..' in pack.parts or not pack.parts:
            raise ValueError(f'relative lineage pack required: {revision}')
        expected = entry.get('pack_sha256')
        actual = hashlib.sha256((root / pack).read_bytes()).hexdigest()
        if actual != expected:
            raise ValueError(f'selected lineage pack changed: {revision}')
        chain.append(revision)
        if revision == ancestor_revision:
            if actual != ancestor_sha256:
                raise ValueError(f'preview ancestor hash differs: {revision}')
            return chain
        parent_revision = entry.get('parent_revision')
        parent = world.get(parent_revision)
        if not isinstance(parent, dict):
            raise ValueError(f'selection does not descend from {ancestor_revision}')
        if (entry.get('parent_pack') != parent.get('pack')
                or entry.get('parent_pack_sha256') != parent.get('pack_sha256')):
            raise ValueError(f'selected lineage parent binding differs: {revision}')
        recipe_path = Path(entry.get('build_recipe', ''))
        if recipe_path.is_absolute() or '..' in recipe_path.parts or not recipe_path.parts:
            raise ValueError(f'relative reviewed build recipe required: {revision}')
        recipe_raw = (root / recipe_path).read_bytes()
        if hashlib.sha256(recipe_raw).hexdigest() != entry.get('build_recipe_sha256'):
            raise ValueError(f'selected lineage build recipe changed: {revision}')
        recipe = json.loads(recipe_raw)
        if (recipe.get('expected_pack_sha256') != actual
                or recipe.get('parent_pack') != parent['pack']
                or recipe.get('parent_pack_sha256') != parent['pack_sha256']):
            raise ValueError(f'selected lineage build recipe binding differs: {revision}')
        revision = parent_revision
    raise ValueError('selected lineage contains a cycle')
