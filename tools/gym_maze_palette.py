"""Named materials for the original Viridian maze kit, in linear render colors."""

def linear(rgb):
    def channel(value):
        value /= 255
        return value/12.92 if value <= .04045 else ((value+.055)/1.055)**2.4
    return tuple(channel(v) for v in rgb)

PALETTE = {
    'wall': linear((74,107,0)),
    'wall_light': linear((123,165,8)),
    'wall_shadow': linear((57,57,57)),
    'coping': linear((123,165,8)),
    'trim': linear((148,198,74)),
}
