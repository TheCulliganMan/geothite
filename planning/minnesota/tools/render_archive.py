"""Retain exact render files before replacing convenient latest aliases."""
from hashlib import sha256
from io import BytesIO
from pathlib import Path
import argparse

from PIL import Image


def _retain(payload, path):
    digest = sha256(payload).hexdigest()
    directory = path.parent / 'history'
    directory.mkdir(parents=True, exist_ok=True)
    archived = directory / f'{path.stem}-{digest}{path.suffix}'
    try:
        with archived.open('xb') as stream:
            stream.write(payload)
    except FileExistsError:
        if archived.read_bytes() != payload:
            raise ValueError(f'Render archive collision: {archived}')
    return archived


def retain_render_file(path):
    """Keep an existing file byte-for-byte, including its original metadata."""
    path = Path(path)
    return _retain(path.read_bytes(), path) if path.is_file() else None


def save_render(image, path, format=None, **params):
    """Archive a lossless original or conversion before updating its latest file.

    This function never converts, resizes or deletes an image. Call it for the
    original image before any conversion, and separately for each derived view.
    Encoded bytes identify versions, so different palettes, alpha, metadata and
    compression settings cannot silently overwrite a previous artifact.
    """
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    # Preserve the previous alias even if encoding the new image fails.
    retain_render_file(path)
    image_format = format or Image.registered_extensions().get(path.suffix.lower())
    if not image_format:
        raise ValueError(f'Render output needs an explicit supported format: {path}')
    output = BytesIO()
    image.save(output, format=image_format, **params)
    payload = output.getvalue()
    archived = _retain(payload, path)
    path.write_bytes(payload)
    return archived


def archive_existing(directory):
    """Retain renders, skipping immutable histories and the creature archive."""
    directory = Path(directory)
    excluded = {'history', '.render-archive', 'creature-archive'}
    if excluded & set(directory.resolve().parts):
        return 0
    extensions = set(Image.registered_extensions()) | {'.svg', '.pdf'}
    paths = sorted(path for path in directory.rglob('*')
                   if path.is_file() and not excluded & set(path.relative_to(directory).parts)
                   and path.suffix.lower() in extensions)
    for path in paths:
        retain_render_file(path)
    return len(paths)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    args = parser.parse_args()
    print(f'Archived {archive_existing(args.directory)} existing render files byte-for-byte.')
