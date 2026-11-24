"""Writes art/emoji/shapes3.fsa: the emoji that are a pattern rather than a drawing
(letter tiles, word buttons, coloured shapes, arrows, media buttons, the 24 clock faces).

    python art/gen_shapes3.py

The sheet it writes is the art; this script is only how it was made, kept so a change to
a pattern (the tile, a colour) is made once.
"""

import math
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "emoji", "shapes3.fsa")
out = [
    "// Letter tiles, word buttons, coloured shapes, arrows and clock faces.",
    "// Original Remedy Emoji art. Written by art/gen_shapes3.py: change it there.",
    "category symbols",
    "",
    "part sh_red_tile",
    "  fill lin #ff7a6a #d8281c 0 5 0 59",
    "  stroke #8a140c 2.4",
    "  rect 5 5 54 54 12",
    "",
    "part sh_orange_tile",
    "  fill lin #ffb04a #e8701c 0 5 0 59",
    "  stroke #a8420a 2.4",
    "  rect 5 5 54 54 12",
    "",
    "part sh_purple_tile",
    "  fill lin #b98cf5 #7a48d6 0 5 0 59",
    "  stroke #4a2596 2.4",
    "  rect 5 5 54 54 12",
    "",
    "part sh_dbl",
    "  fill $white",
    "  poly 9 32 22 19 22 27.5 42 27.5 42 19 55 32 42 45 42 36.5 22 36.5 22 45",
    "",
    "part sh_hook",
    "  nofill",
    "  stroke $white 6",
    "  path M 22 25 L 39 25 Q 50 25 50 36 Q 50 47 39 47 L 26 47",
    "  fill $white",
    "  stroke $white 2",
    "  poly 23 14 10 25 23 36",
    "",
    "part sh_curve",
    "  nofill",
    "  stroke $white 6",
    "  path M 11 47 L 29 47 Q 43 47 43 31 L 43 25",
    "  fill $white",
    "  stroke $white 2",
    "  poly 32 26 43 12 54 26",
    "",
    "part sh_cycle",
    "  nofill",
    "  stroke $white 5",
    "  arc 32 32 15 15 205 325",
    "  arc 32 32 15 15 25 145",
    "  fill $white",
    "  stroke $white 2",
    "  poly 40 17 52 21 44 31",
    "  poly 24 47 12 43 20 33",
    "",
    "part sh_clock",
    "  fill rad #ffffff #dfe5ec 26 24 34",
    "  stroke #5d6a7c 3",
    "  circle 32 32 26",
    "  fill #5d6a7c",
    "  nostroke",
    "  circle 32 10.5 1.7",
    "  circle 53.5 32 1.7",
    "  circle 32 53.5 1.7",
    "  circle 10.5 32 1.7",
    "",
]


def emoji(name, code, keywords, *lines):
    out.append(f"emoji {name} U+{code:X} | {keywords}")
    out.extend("  " + line for line in lines)
    out.append("")


def tile(name, code, keywords, word, tile_part="sym_blue_tile", height=None, extra=()):
    # Lettering is 5.5 units a letter at height 6; fit the word in 42 units.
    n = len(word)
    h = height or min(26, 42 * 6 / (5.5 * n - 1.5))
    weight = max(2.6, min(5, h / 5))
    emoji(name, code, keywords, f"use {tile_part}", *extra, "fill $white", f"text 32 {32 + h / 2:.1f} {h:.1f} {word} {weight:.1f}")


# Regional indicator letters: flags are made of two of them.
for i in range(26):
    letter = chr(65 + i)
    tile(f"regional_{letter.lower()}", 0x1F1E6 + i, f"letter {letter.lower()} flag indicator alphabet", letter)

# Word and letter buttons.
tile("a_button", 0x1F170, "blood type a letter", "A", "sh_red_tile")
tile("b_button", 0x1F171, "blood type b letter", "B", "sh_red_tile")
tile("o_button", 0x1F17E, "blood type o letter", "O", "sh_red_tile")
tile("parking", 0x1F17F, "p car park sign", "P")
tile("ab_button", 0x1F18E, "blood type ab", "AB", "sh_red_tile")
tile("cl_button", 0x1F191, "clear delete", "CL", "sh_red_tile")
tile("cool_button", 0x1F192, "cool nice word", "COOL")
tile("free_button", 0x1F193, "free no cost word", "FREE")
tile("id_button", 0x1F194, "identity id word", "ID", "sh_purple_tile")
tile("new_button", 0x1F195, "new fresh word", "NEW")
tile("ng_button", 0x1F196, "no good ng word", "NG")
tile("ok_button", 0x1F197, "okay ok yes word", "OK")
tile("sos", 0x1F198, "help emergency rescue", "SOS", "sh_red_tile")
tile("up_button", 0x1F199, "up level word", "UP!")
tile("vs_button", 0x1F19A, "versus against match", "VS", "sh_orange_tile")
tile("keycap_ten", 0x1F51F, "ten 10 number", "10")
tile("input_capitals", 0x1F520, "abcd capital letters uppercase input", "ABCD")
tile("input_lowercase", 0x1F521, "abcd small letters lowercase input", "ABCD", height=9)
tile("input_numbers", 0x1F522, "1234 digits numbers input", "1234")
tile("input_symbols", 0x1F523, "symbols punctuation input", "#&%!")
tile("input_letters", 0x1F524, "abc alphabet letters input", "ABC")
tile("bangbang", 0x203C, "double exclamation surprise", "!!", "sh_red_tile")
tile("interrobang", 0x2049, "exclamation question what", "!?", "sh_red_tile")

emoji("tm", 0x2122, "trademark brand mark", "fill $sym_grey", "text 32 39 20 TM 4.2")
emoji("metro_m", 0x24C2, "circled m metro subway", "fill lin #74c2ff #2f86ea 0 5 0 59", "stroke $sym_blued 2.4", "circle 32 32 27", "fill $white", "text 32 42 20 M 4.4")
emoji("information", 0x2139, "info help details i", "use sym_blue_tile", "fill $white", "circle 32 17 4.6", "path M 25 26 L 36 26 L 36 46 L 41 46 L 41 51 L 24 51 L 24 46 L 28 46 L 28 31 L 25 31 Z")
emoji("grey_question", 0x2754, "question white what unsure", "fill #c4ccd6", "text 32 55 46 ? 9.5", "fill #ffffff", "text 32 55 46 ? 5")
emoji("grey_exclamation", 0x2755, "exclamation white alert", "fill #c4ccd6", "text 32 55 46 ! 9.5", "fill #ffffff", "text 32 55 46 ! 5")
emoji("heavy_equals", 0x1F7F0, "equals same math", "fill $sym_grey", "rect 10 19 44 9 3", "rect 10 36 44 9 3")
emoji("heavy_division", 0x2797, "divide math division", "fill $sym_grey", "rect 9 27.5 46 9 3", "circle 32 14 5.5", "circle 32 50 5.5")
emoji("underage", 0x1F51E, "no under eighteen 18 adults only", "use s2_ban_bg", "fill $s2k", "text 32 40 17 18 4", "use s2_ban")

# Words with an arrow above them.
for name, code, word, kw, arrow in [
    ("back_arrow", 0x1F519, "BACK", "back return previous", "left"),
    ("end_arrow", 0x1F51A, "END", "end finish last", "left"),
    ("on_arrow", 0x1F51B, "ON!", "on active mark", "both"),
    ("soon_arrow", 0x1F51C, "SOON", "soon coming next", "right"),
    ("top_arrow", 0x1F51D, "TOP", "top up best", "up"),
]:
    lines = ["fill $sym_grey"]
    if arrow == "up":
        lines += ["poly 32 6 44 20 36.5 20 36.5 32 27.5 32 27.5 20 20 20"]
    else:
        if arrow in ("left", "both"):
            lines += ["poly 8 19 20 9 20 15.5 32 15.5 32 22.5 20 22.5 20 29"]
        if arrow in ("right", "both"):
            lines += ["poly 56 19 44 9 44 15.5 32 15.5 32 22.5 44 22.5 44 29"]
        if arrow == "left":
            lines += ["rect 30 15.5 22 7 2"]
        if arrow == "right":
            lines += ["rect 12 15.5 22 7 2"]
    n = len(word)
    h = min(18, 52 * 6 / (5.5 * n - 1.5))
    lines += [f"text 32 {40 + h:.1f} {h:.1f} {word} {max(2.6, h / 5):.1f}"]
    emoji(name, code, kw, *lines)

# Arrows.
for name, code, kw, body in [
    ("left_right_arrow", 0x2194, "horizontal both ways resize", "use sh_dbl"),
    ("up_down_arrow", 0x2195, "vertical both ways resize", "use sh_dbl 0 0 1 90"),
    ("arrow_upper_left", 0x2196, "north west diagonal", "use sym_arrow 0 0 1 -45"),
    ("arrow_upper_right", 0x2197, "north east diagonal", "use sym_arrow 0 0 1 45"),
    ("arrow_lower_right", 0x2198, "south east diagonal", "use sym_arrow 0 0 1 135"),
    ("arrow_lower_left", 0x2199, "south west diagonal", "use sym_arrow 0 0 1 225"),
    ("arrow_hook_left", 0x21A9, "return back undo reply", "use sh_hook"),
    ("arrow_hook_right", 0x21AA, "forward redo turn", "use sh_hook 0 0 1 0 flip"),
    ("arrow_curve_up", 0x2934, "curve rise turn up", "use sh_curve"),
    ("arrow_curve_down", 0x2935, "curve fall turn down", "use sh_curve 0 0 1 180 flip"),
    ("arrows_clockwise", 0x1F503, "reload refresh sync vertical", "use sh_cycle"),
    ("arrows_counterclockwise", 0x1F504, "refresh sync anticlockwise repeat", "use sh_cycle 0 0 1 0 flip"),
]:
    emoji(name, code, kw, "use sym_blue_tile", body)

# Media buttons.
for name, code, kw, shapes in [
    ("eject", 0x23CF, "eject button disc remove", ["poly 32 13 51 35 13 35", "rect 13 41 38 8 2"]),
    ("arrow_double_up", 0x23EB, "fast up double scroll top", ["poly 32 11 50 30 14 30", "poly 32 31 50 50 14 50"]),
    ("arrow_double_down", 0x23EC, "fast down double scroll bottom", ["poly 32 53 50 34 14 34", "poly 32 33 50 14 14 14"]),
    ("next_track", 0x23ED, "skip next forward song", ["poly 13 17 31 32 13 47", "poly 29 17 47 32 29 47", "rect 46 17 6 30 1.5"]),
    ("previous_track", 0x23EE, "skip previous back song", ["poly 51 17 33 32 51 47", "poly 35 17 17 32 35 47", "rect 12 17 6 30 1.5"]),
    ("play_pause", 0x23EF, "play pause toggle button", ["poly 11 17 31 32 11 47", "rect 36 17 7 30 1.5", "rect 46 17 7 30 1.5"]),
    ("reverse", 0x25C0, "back left play backwards button", ["poly 44 14 44 50 16 32"]),
]:
    emoji(name, code, kw, "use sym_blue_tile", "fill $white", "stroke $white 2", *shapes)

emoji("radio_button", 0x1F518, "option select choice circle", "fill rad #ffffff #dfe5ec 26 24 34", "stroke #5d6a7c 3", "circle 32 32 25", "fill rad #74c2ff #2f86ea 28 28 16", "stroke $sym_blued 2", "circle 32 32 13")

# Coloured circles, squares, diamonds and triangles.
COLOURS = {
    "red": ("#ff7a6a", "#d8281c", "#8a140c"),
    "orange": ("#ffb04a", "#e8701c", "#a8420a"),
    "yellow": ("#ffe27a", "#f2b01c", "#a87a0c"),
    "green": ("#7fd47a", "#2f9a3c", "#145a24"),
    "blue": ("#74c2ff", "#2f86ea", "#1c5fc0"),
    "purple": ("#b98cf5", "#7a48d6", "#4a2596"),
    "brown": ("#c98a4a", "#8a4a1c", "#4a2208"),
    "black": ("#5a6270", "#22262e", "#0e1014"),
    "white": ("#ffffff", "#dfe5ec", "#8793a3"),
}


def paint(colour):
    light, dark, line = COLOURS[colour]
    return [f"fill rad {light} {dark} 24 22 40", f"stroke {line} 2.4"]


for colour, code in [("orange", 0x1F7E0), ("yellow", 0x1F7E1), ("green", 0x1F7E2), ("purple", 0x1F7E3), ("brown", 0x1F7E4), ("red", 0x1F534), ("blue", 0x1F535)]:
    emoji(f"{colour}_circle", code, f"{colour} circle dot round", *paint(colour), "circle 32 32 26")
emoji("white_circle", 0x26AA, "white circle dot round", *paint("white"), "circle 32 32 19")
emoji("black_circle", 0x26AB, "black circle dot round", *paint("black"), "circle 32 32 19")
emoji("o_ring", 0x2B55, "circle ring correct round red", "nofill", "stroke #8a140c 11", "circle 32 32 21.5", "stroke #e8382c 7", "circle 32 32 21.5")

for colour, code in [("red", 0x1F7E5), ("blue", 0x1F7E6), ("orange", 0x1F7E7), ("yellow", 0x1F7E8), ("green", 0x1F7E9), ("purple", 0x1F7EA), ("brown", 0x1F7EB), ("black", 0x2B1B), ("white", 0x2B1C)]:
    emoji(f"{colour}_square", code, f"{colour} square block large", *paint(colour), "rect 6 6 52 52 6")
for colour, code, size, word in [
    ("white", 0x25FB, 42, "medium"), ("black", 0x25FC, 42, "medium"),
    ("white", 0x25FD, 32, "medium_small"), ("black", 0x25FE, 32, "medium_small"),
    ("black", 0x25AA, 20, "small"), ("white", 0x25AB, 20, "small"),
]:  # fmt: skip
    o = 32 - size / 2
    emoji(f"{colour}_{word}_square", code, f"{colour} square {word.replace('_', ' ')}", *paint(colour), f"rect {o:g} {o:g} {size} {size} {size / 9:.1f}")
emoji("black_square_button", 0x1F532, "button square dark", *paint("black"), "rect 6 6 52 52 8", "nofill", "stroke #c4ccd6 3", "rect 16 16 32 32 4")
emoji("white_square_button", 0x1F533, "button square light", *paint("white"), "rect 6 6 52 52 8", "nofill", "stroke #5d6a7c 3", "rect 16 16 32 32 4")

for colour, code, r, word in [("orange", 0x1F536, 27, "large"), ("blue", 0x1F537, 27, "large"), ("orange", 0x1F538, 14, "small"), ("blue", 0x1F539, 14, "small")]:
    emoji(f"{word}_{colour}_diamond", code, f"{colour} diamond {word} rhombus", *paint(colour), f"poly 32 {32 - r} {32 + r} 32 32 {32 + r} {32 - r} 32")
for code, up, r, word in [(0x1F53A, True, 25, "red_triangle_up"), (0x1F53B, False, 25, "red_triangle_down"), (0x1F53C, True, 14, "small_red_triangle_up"), (0x1F53D, False, 14, "small_red_triangle_down")]:
    s = -1 if up else 1
    emoji(word, code, "red triangle " + ("up" if up else "down"), *paint("red"), f"poly 32 {32 + s * r} {32 + r} {32 - s * r * 0.8:g} {32 - r} {32 - s * r * 0.8:g}")

# The clock faces: on the hour, then on the half hour.
for half in (False, True):
    for h in range(1, 13):
        code = (0x1F55C if half else 0x1F550) + h - 1
        hour = math.radians((h + (0.5 if half else 0)) * 30)
        minute = math.radians(180 if half else 0)
        hx, hy = 32 + 13 * math.sin(hour), 32 - 13 * math.cos(hour)
        mx, my = 32 + 20 * math.sin(minute), 32 - 20 * math.cos(minute)
        name = f"clock{h}{'30' if half else ''}"
        words = f"{h} {'thirty half past' if half else 'o clock hour'} time"
        emoji(name, code, words, "use sh_clock", "nofill", "stroke #2a2f3b 4", f"line 32 32 {hx:.1f} {hy:.1f}", "stroke #2a2f3b 3", f"line 32 32 {mx:.1f} {my:.1f}", "fill #e2584a", "nostroke", "circle 32 32 2.6")

with open(OUT, "w", encoding="utf-8", newline="\n") as f:
    f.write("\n".join(out))
print(sum(1 for line in out if line.startswith("emoji ")), "emoji ->", OUT)
