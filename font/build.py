"""Builds Remedy Emoji, the color font of this repository's emoji and stickers.

    cargo run --release --example export -- target/art.json
    python font/build.py          # writes fonts/RemedyEmoji.ttf, .woff2 and .json

The art is the `.fsa` sheets in `art/`; the exporter writes each glyph's shapes as flat
polygons and this script turns them into a COLRv1 font: one outline per shape (its pieces
merged and thinned), painted with the shape's colour or gradient.

- An emoji with a Unicode code point is at that code point. U+FE0F after it is absorbed.
- A glyph of our own (no code point) gets one in the private use area (plane 15), kept in
  `font/codepoints.txt` so it never changes once given.
- A glyph that takes skin tones has five more forms, reached by the Unicode modifiers
  U+1F3FB to U+1F3FF after it.
- `fonts/RemedyEmoji.json` lists every glyph: name, kind, category, keywords, text, tones.

Needs fontTools, skia-pathops and brotli (for the .woff2). The output is deterministic.
"""

import json
import os
import sys

os.environ["SOURCE_DATE_EPOCH"] = "1795478400"  # 2026-11-24, so the same art gives the same bytes

from fontTools.colorLib.builder import buildCOLR, buildCPAL
from fontTools.feaLib.builder import addOpenTypeFeaturesFromString
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont
from fontTools.ttLib.tables.otTables import PaintFormat
from pathops import FillType, Path

FAMILY = "Remedy Emoji"
VERSION = "1.000"
UPM = 2048
SCALE = UPM / 64  # the art's canvas is 64 units square
TOP = 1792  # font y of the canvas's top edge: the square sits 256 below the baseline
ASCENT, DESCENT, LINE_GAP = 1908, -364, 184
# shortcut: outlines are short straight pieces, about 550,000 points; fit curves to them
# to roughly halve the file if its size starts to matter.
TOLERANCE = 1.0  # font units a thinned outline may leave the drawn one (0.06 px at 128 px)
PUA_START = 0xF0000
TONE_MODIFIERS = [0x1F3FB, 0x1F3FC, 0x1F3FD, 0x1F3FE, 0x1F3FF]

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
ART = os.path.join(ROOT, "target", "art.json")
CODEPOINTS = os.path.join(HERE, "codepoints.txt")
OUT = os.path.join(ROOT, "fonts")


def thin(points):
    """Ramer-Douglas-Peucker on a closed outline: drops points within TOLERANCE of the line."""
    if len(points) < 4:
        return points
    # Open the ring at the two points farthest apart, so each half has real end points.
    a = 0
    b = max(range(len(points)), key=lambda i: (points[i][0] - points[0][0]) ** 2 + (points[i][1] - points[0][1]) ** 2)
    return _thin_open(points[a : b + 1])[:-1] + _thin_open(points[b:] + points[:1])[:-1]


def _thin_open(points):
    keep = [False] * len(points)
    keep[0] = keep[-1] = True
    stack = [(0, len(points) - 1)]
    while stack:
        first, last = stack.pop()
        (x0, y0), (x1, y1) = points[first], points[last]
        dx, dy = x1 - x0, y1 - y0
        length = (dx * dx + dy * dy) ** 0.5
        worst, at = 0.0, None
        for i in range(first + 1, last):
            px, py = points[i]
            d = abs(dx * (py - y0) - dy * (px - x0)) / length if length else ((px - x0) ** 2 + (py - y0) ** 2) ** 0.5
            if d > worst:
                worst, at = d, i
        if at is not None and worst > TOLERANCE:
            keep[at] = True
            stack += [(first, at), (at, last)]
    return [p for p, k in zip(points, keep) if k]


def outline(contours):
    """One shape's pieces as a font outline: merged as the rasterizer fills them (non-zero
    winding), thinned, on the font's grid. A tuple of closed contours of (x, y)."""
    path = Path(fillType=FillType.WINDING)
    pen = path.getPen()
    for flat in contours:
        pts = list(zip(flat[0::2], flat[1::2]))
        if len(pts) < 3:
            continue
        # On the font's grid before merging: the merge is exact there and never gives up.
        pen.moveTo((round(pts[0][0] * SCALE), round(TOP - pts[0][1] * SCALE)))
        for x, y in pts[1:]:
            pen.lineTo((round(x * SCALE), round(TOP - y * SCALE)))
        pen.closePath()
    path.simplify(fix_winding=True)
    rec = RecordingPen()
    path.draw(rec)
    out, cur = [], []
    for op, args in rec.value:
        if op in ("moveTo", "lineTo"):
            cur.append(args[0])
        elif op in ("closePath", "endPath"):
            ring = []
            for x, y in thin(cur):
                p = (round(x), round(y))
                if not ring or ring[-1] != p:
                    ring.append(p)
            if len(ring) > 1 and ring[0] == ring[-1]:
                ring.pop()
            if len(ring) >= 3:
                out.append(tuple(ring))
            cur = []
        else:
            raise SystemExit(f"unexpected {op} in a merged outline")
    return tuple(out)


def read_codepoints():
    table = {}
    if os.path.exists(CODEPOINTS):
        for line in open(CODEPOINTS, encoding="utf-8"):
            line = line.split("#")[0].strip()
            if line:
                name, code = line.split()
                table[name] = int(code[2:], 16)
    return table


def write_codepoints(table):
    with open(CODEPOINTS, "w", encoding="utf-8", newline="\n") as f:
        f.write("# Private use code points of the glyphs that have no Unicode one. A line is never\n")
        f.write("# changed or reused once it is here: text people already saved keeps its meaning.\n")
        for name, code in sorted(table.items(), key=lambda kv: kv[1]):
            f.write(f"{name} U+{code:X}\n")


def main():
    if not os.path.exists(ART):
        raise SystemExit("run first: cargo run --release --example export -- target/art.json")
    art = json.load(open(ART, encoding="utf-8"))
    tones = art["tones"]

    # Private use code points for our own glyphs, in the order the sheets list them.
    # A sequence is reachable only when each of its parts is a glyph here too (a font that
    # lacks a part is not asked for the sequence); one that is not becomes our own glyph.
    singles = {g["chars"][0] for g in art["glyphs"] if len(g["chars"]) == 1} | {0x200D, 0xFE0F}
    for g in art["glyphs"]:
        if len(g["chars"]) > 1 and not all(c in singles for c in g["chars"]):
            missing = " ".join(f"U+{c:X}" for c in g["chars"] if c not in singles)
            print(f"note: {g['name']} needs {missing}, which is not drawn: it gets a private use code point")
            g["chars"] = []
    private = read_codepoints()
    for g in art["glyphs"]:
        if not g["chars"] and g["name"] not in private:
            private[g["name"]] = max([PUA_START - 1, *private.values()]) + 1
    write_codepoints(private)

    palette, palette_index = [], {}

    def color_index(rgb):
        if rgb not in palette_index:
            palette_index[rgb] = len(palette)
            palette.append(rgb)
        return palette_index[rgb]

    def stop(c, tone, offset):
        r, g, b, a, skin = c
        rgb = tuple(tones[tone][skin - 1]) if skin else (r, g, b)
        return {"StopOffset": offset, "PaletteIndex": color_index(rgb), "Alpha": a / 255}

    def fx(x):
        return round(x * SCALE)

    def fy(y):
        return round(TOP - y * SCALE)

    def paint(p, tone):
        if "solid" in p:
            s = stop(p["solid"], tone, 0)
            return {"Format": PaintFormat.PaintSolid, "PaletteIndex": s["PaletteIndex"], "Alpha": s["Alpha"]}
        if "linear" in p:
            c0, c1, x1, y1, x2, y2 = p["linear"]
            line = {"Extend": "pad", "ColorStop": [stop(c0, tone, 0.0), stop(c1, tone, 1.0)]}
            # The third point turns the gradient's lines: at right angles to p0-p1 is a plain one.
            return {
                "Format": PaintFormat.PaintLinearGradient,
                "ColorLine": line,
                "x0": fx(x1), "y0": fy(y1), "x1": fx(x2), "y1": fy(y2),
                "x2": fx(x1) - (fy(y2) - fy(y1)), "y2": fy(y1) + (fx(x2) - fx(x1)),
            }  # fmt: skip
        c0, c1, cx, cy, r = p["radial"]
        line = {"Extend": "pad", "ColorStop": [stop(c0, tone, 0.0), stop(c1, tone, 1.0)]}
        return {
            "Format": PaintFormat.PaintRadialGradient,
            "ColorLine": line,
            "x0": fx(cx), "y0": fy(cy), "r0": 0, "x1": fx(cx), "y1": fy(cy), "r1": round(r * SCALE),
        }  # fmt: skip

    outlines, outline_name = [], {}

    def outline_glyph(contours):
        key = outline(contours)
        if not key:
            return None
        if key not in outline_name:
            outline_name[key] = f"o{len(outlines):05d}"
            outlines.append(key)
        return outline_name[key]

    order = [".notdef", "vs16", "zwj"]
    cmap = {0xFE0F: "vs16", 0x200D: "zwj"}
    color_glyphs, clip_boxes, rules, listing = {}, {}, [], []
    by_code = {}

    def add_color_glyph(name, shapes, tone):
        layers, box = [], None
        for shape, glyph in shapes:
            layers.append({"Format": PaintFormat.PaintGlyph, "Paint": paint(shape["paint"], tone), "Glyph": glyph})
            for ring in outlines[int(glyph[1:])]:
                xs, ys = [p[0] for p in ring], [p[1] for p in ring]
                b = (min(xs), min(ys), max(xs), max(ys))
                box = b if box is None else (min(box[0], b[0]), min(box[1], b[1]), max(box[2], b[2]), max(box[3], b[3]))
        order.append(name)
        color_glyphs[name] = (PaintFormat.PaintColrLayers, layers)
        if box:
            clip_boxes[name] = box

    # A tone modifier on its own is a swatch of that tone.
    swatch = outline_glyph([[14, 14, 50, 14, 50, 50, 14, 50]])
    for i, code in enumerate(TONE_MODIFIERS):
        name = f"tone{i + 1}"
        add_color_glyph(name, [({"paint": {"solid": [0, 0, 0, 255, 1]}}, swatch)], i + 1)
        cmap[code] = name

    for n, g in enumerate(art["glyphs"]):
        name = f"g{n:04d}"
        shapes = [(s, o) for s in g["shapes"] if (o := outline_glyph(s["contours"]))]
        add_color_glyph(name, shapes, 0)
        chars = g["chars"] or [private[g["name"]]]
        if len(chars) == 1:
            cmap[chars[0]] = name
        else:
            rules.append((chars, name))
        by_code[tuple(chars)] = name
        if g["chars"]:
            rules.append(([name, 0xFE0F], name))
        if g["tones"]:
            for i, code in enumerate(TONE_MODIFIERS):
                add_color_glyph(f"{name}.t{i + 1}", shapes, i + 1)
                rules.append(([name, code], f"{name}.t{i + 1}"))
        listing.append(
            {
                "name": g["name"],
                "kind": g["kind"],
                "category": g["category"],
                "keywords": g["keywords"],
                "text": "".join(chr(c) for c in chars),
                "codepoints": [f"U+{c:04X}" for c in chars],
                "unicode": bool(g["chars"]),
                "tones": g["tones"],
            }
        )

    glyph_order = order + [f"o{i:05d}" for i in range(len(outlines))]
    fb = FontBuilder(UPM, isTTF=True)
    fb.setupGlyphOrder(glyph_order)
    fb.setupCharacterMap(cmap)
    glyphs = {}
    empty = TTGlyphPen(None).glyph()
    for name in order:
        glyphs[name] = empty
    for i, rings in enumerate(outlines):
        pen = TTGlyphPen(None)
        for ring in rings:
            pen.moveTo(ring[0])
            for p in ring[1:]:
                pen.lineTo(p)
            pen.closePath()
        glyphs[f"o{i:05d}"] = pen.glyph()
    fb.setupGlyf(glyphs)
    # Joiners and the layer outlines take no room; everything a reader sees is one em wide.
    narrow = {"vs16", "zwj"}
    # The left side bearing is the outline's left edge: a renderer moves the outline to it.
    fb.setupHorizontalMetrics(
        {
            n: (0 if n in narrow or n.startswith("o") else UPM, getattr(glyphs[n], "xMin", 0))
            for n in glyph_order
        }
    )
    fb.setupHorizontalHeader(ascent=ASCENT, descent=DESCENT, lineGap=LINE_GAP)
    fb.setupNameTable(
        {
            "copyright": "Copyright 2026 Ahmi Darrow",
            "familyName": FAMILY,
            "styleName": "Regular",
            "uniqueFontIdentifier": f"{FAMILY} {VERSION}",
            "fullName": FAMILY,
            "version": f"Version {VERSION}",
            "psName": "RemedyEmoji-Regular",
            "licenseDescription": "MIT OR Apache-2.0",
            "licenseInfoURL": "https://github.com/AhmiDarrow/RemedyEmoji",
        }
    )
    fb.setupOS2(
        sTypoAscender=TOP,
        sTypoDescender=TOP - UPM,
        sTypoLineGap=LINE_GAP,
        usWinAscent=ASCENT,
        usWinDescent=-DESCENT,
        achVendID="AHMI",
    )
    fb.setupPost(keepGlyphNames=False)
    font = fb.font
    font["COLR"] = buildCOLR(color_glyphs, version=1, glyphMap=font.getReverseGlyphMap(), clipBoxes=clip_boxes)
    font["CPAL"] = buildCPAL([[(r / 255, g / 255, b / 255, 1.0) for r, g, b in palette]])

    # Sequences: a code point is its glyph, so a rule is written in glyph names.
    def glyph_of(item):
        return item if isinstance(item, str) else cmap.get(item) or by_code[(item,)]

    lines = ["languagesystem DFLT dflt;", "feature ccmp {"]
    for seq, result in sorted(rules, key=lambda r: -len(r[0])):
        lines.append(f"  sub {' '.join(glyph_of(i) for i in seq)} by {result};")
    lines.append("} ccmp;")
    addOpenTypeFeaturesFromString(font, "\n".join(lines))

    os.makedirs(OUT, exist_ok=True)
    ttf = os.path.join(OUT, "RemedyEmoji.ttf")
    font.save(ttf)
    web = TTFont(ttf)
    web.flavor = "woff2"
    web.save(os.path.join(OUT, "RemedyEmoji.woff2"))
    with open(os.path.join(OUT, "RemedyEmoji.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump({"family": FAMILY, "version": VERSION, "glyphs": listing}, f, ensure_ascii=False, indent=1)
        f.write("\n")
    points = sum(len(r) for o in outlines for r in o)
    print(
        f"{FAMILY} {VERSION}: {len(listing)} glyphs ({sum(g['unicode'] for g in listing)} Unicode, "
        f"{len(private)} private use), {len(outlines)} outlines, {points} points, {len(palette)} colours; "
        f"{os.path.getsize(ttf)} bytes"
    )


if __name__ == "__main__":
    sys.exit(main())
