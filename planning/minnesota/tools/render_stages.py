"""Preserve map review intermediates around ordinary Pillow transformations."""
from pathlib import Path

from render_archive import retain_render_file, save_render


def retain_render_bytes(payload, path):
    """Keep copied encoded renders exact, preserving both alias generations."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    retain_render_file(path)
    path.write_bytes(payload)
    return retain_render_file(path)


def render_stage(image, operation, directory, label, *args, **kwargs):
    """Archive both sides of a crop, mode conversion or resize, then return it.

    The image operations and their arguments are unchanged. Exact encoded-byte
    history and replacement-alias preservation belong to shared save_render.
    Crops include their source bounds in the stage name for later reuse. This
    helper is for map render stages, not source sprite/catalog extraction.
    """
    if operation not in {'crop', 'convert', 'resize'}:
        raise ValueError(f'Unsupported map image stage: {operation}')
    filename = getattr(image, 'filename', None)
    if filename:
        # Keep original encoded file metadata too, before a lazy Image.open is
        # converted. In-memory images are retained losslessly below.
        retain_render_file(filename)
    if operation == 'crop' and args and args[0] is not None:
        label += '-bounds-' + '-'.join(str(value) for value in args[0])
    directory = Path(directory) / 'stages'
    save_render(image, directory / f'{label}-original.png')
    result = getattr(image, operation)(*args, **kwargs)
    save_render(result, directory / f'{label}-result.png')
    return result
