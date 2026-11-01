#!/usr/bin/env python3
"""Check idle prompt retention and dealt cards in actual native 2D captures.

Requires Pillow and fresh Rust render_at_location captures at an integer LCD
scale: an idle Ask Play screen and a Choose a Card screen. These pixel checks
complement visual review and production input/wallet tests.
"""
import argparse
import json
from PIL import Image


def region(image, bounds):
    sx, sy = image.width // 160, image.height // 144
    assert sx == sy and sx > 0 and image.size == (160 * sx, 144 * sy), image.size
    x0, y0, x1, y1 = bounds
    pixels = list(image.crop((x0 * sx, y0 * sy, x1 * sx, y1 * sy)).convert("RGB").getdata())
    return dict(white=sum(min(p) >= 245 for p in pixels) / len(pixels),
                ink=sum(max(p) <= 40 for p in pixels) / len(pixels))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--deal", required=True)
    args = parser.parse_args()
    prompt, deal = Image.open(args.prompt), Image.open(args.deal)
    checks = {
        "prompt_first_line": region(prompt, (8, 104, 144, 112)),
        "prompt_second_line": region(prompt, (8, 120, 72, 128)),
        "yes_no": region(prompt, (104, 64, 144, 88)),
        "dealt_instruction": region(deal, (8, 104, 144, 112)),
        "dealt_card": region(deal, (16, 0, 56, 48)),
    }
    for key in ["prompt_first_line", "prompt_second_line", "yes_no", "dealt_instruction"]:
        assert checks[key]["white"] > 0.55 and checks[key]["ink"] > 0.02, (key, checks[key])
    assert checks["dealt_card"]["ink"] > 0.2, checks["dealt_card"]
    print(json.dumps(dict(status="PASS", checks=checks), indent=2))


if __name__ == "__main__":
    main()
