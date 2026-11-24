"""Checks the built font against the art: run after `build.py`.

    cargo run --release --example export -- target/art.json
    python font/build.py && python font/check.py
"""

import hashlib
import json
import os
import sys

import build
from fontTools.ttLib import TTFont

FONT = os.path.join(build.OUT, "RemedyEmoji.ttf")


def digest():
    return hashlib.sha256(open(FONT, "rb").read()).hexdigest()


def main():
    art = json.load(open(build.ART, encoding="utf-8"))["glyphs"]
    listing = json.load(open(os.path.join(build.OUT, "RemedyEmoji.json"), encoding="utf-8"))["glyphs"]
    font = TTFont(FONT)
    cmap = font.getBestCmap()
    colr = font["COLR"].table
    painted = {r.BaseGlyph for r in colr.BaseGlyphList.BaseGlyphPaintRecord}

    assert colr.Version == 1, "a COLRv1 font"
    assert font["OS/2"].fsType == 0, "installable and embeddable without restriction"
    assert [g["name"] for g in listing] == [g["name"] for g in art], "the listing follows the art"
    texts = [g["text"] for g in listing]
    assert len(set(texts)) == len(texts), "two glyphs share their text"
    for g in listing:
        if len(g["text"]) == 1:
            glyph = cmap.get(ord(g["text"]))
            assert glyph in painted, f"{g['name']} is not a painted glyph at {g['codepoints']}"
    toned = sum(g["tones"] for g in listing)
    # Each glyph, five more for each that takes tones, and the five tone swatches.
    assert len(painted) == len(listing) + 5 * toned + 5, f"{len(painted)} painted glyphs"
    for code in (0xFE0F, 0x200D, *build.TONE_MODIFIERS):
        assert code in cmap, f"U+{code:X} is not in the font"
    lookups = font["GSUB"].table.LookupList.Lookup
    ligatures = sum(len(v) for lk in lookups for st in lk.SubTable for v in st.ligatures.values())
    unicode = sum(g["unicode"] for g in listing)
    assert ligatures == unicode + 5 * toned, f"{ligatures} sequences"
    private = build.read_codepoints()
    assert len(set(private.values())) == len(private), "a private use code point is given twice"
    assert all(build.PUA_START <= c <= 0xFFFFD for c in private.values()), "outside private use plane 15"

    # The font in the repository is what the art builds, byte for byte.
    before = digest()
    build.main()
    assert digest() == before, "fonts/RemedyEmoji.ttf is not what the art builds: commit the rebuilt font"
    print(f"ok: {len(listing)} glyphs, {toned} with tones, {ligatures} sequences, {os.path.getsize(FONT)} bytes")


if __name__ == "__main__":
    sys.exit(main())
