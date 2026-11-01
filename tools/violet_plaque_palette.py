"""Named stone-marker palette shared by original art authoring and validation."""

RGB = {
    'stone_plaque_gray': (156, 156, 156),
    'stone_plaque_light': (247, 231, 214),
    'stone_plaque_shadow': (107, 107, 107),
    'stone_plaque_ink': (57, 57, 57),
}


def linear(value):
    value /= 255.
    return value / 12.92 if value <= .04045 else ((value + .055) / 1.055) ** 2.4


PALETTE = {name: tuple(round(linear(v), 5) for v in rgb) for name, rgb in RGB.items()}
