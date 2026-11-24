"""Writes the four-legged animals of art/emoji/animals2.fsa from one template, so they share
a build: a round body, four stubby legs, a big head turned to the viewer.

    python art/gen_animals2.py

The other animals in that sheet are drawn by hand in art/animals2_hand.fsa, which this
script appends unchanged.
"""

import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "emoji", "animals2.fsa")
HAND = os.path.join(HERE, "animals2_hand.fsa")
out = [
    "// More animals, whole and as faces. Original Remedy Emoji art.",
    "// The four-legged ones are written by art/gen_animals2.py; the rest are art/animals2_hand.fsa.",
    "category nature",
    "",
    "part a3_eyes",
    "  fill $natk",
    "  ellipse 38.5 25 2.5 3.3",
    "  ellipse 51.5 25 2.5 3.3",
    "  fill $white",
    "  circle 39.4 23.8 1",
    "  circle 52.4 23.8 1",
    "",
    "part a3_mouth",
    "  nofill",
    "  stroke $natk 1.7",
    "  curve 41 34 43.2 36.2 45 33.4 46.8 36.2 49 34",
    "",
]


def animal(name, code, keywords, light, dark, line, *, muzzle=None, nose="#3a2a22", ears="round", inner="#ffb6c0",
           tail="thin", before=(), body=(), head=(), legs=None, blush=False, mouth=True):  # fmt: skip
    legs = legs or dark
    L = [f"emoji {name} U+{code:X} | {keywords}"]
    add = L.append
    # Tail, behind everything.
    if tail == "thin":
        add("  nofill"); add(f"  stroke {line} 5"); add("  curve 8 40 2 34 4 24")
        add(f"  stroke {dark} 2.6"); add("  curve 8 40 2 34 4 24")
    elif tail == "curl":
        add("  nofill"); add(f"  stroke {line} 4.6"); add("  curve 7 38 2 36 3 30 8 30 7 34")
        add(f"  stroke {dark} 2.4"); add("  curve 7 38 2 36 3 30 8 30 7 34")
    elif tail == "tuft":
        add("  nofill"); add(f"  stroke {line} 4.6"); add("  curve 7 38 2 42 3 50")
        add(f"  stroke {dark} 2.4"); add("  curve 7 38 2 42 3 50")
        add(f"  fill {line}"); add("  nostroke"); add("  ellipse 3 52 2.6 4")
    elif tail == "puff":
        add(f"  fill rad {light} {dark} 5 36 8"); add(f"  stroke {line} 2"); add("  circle 6 38 5.5")
    elif tail == "bushy":
        add(f"  fill rad {light} {dark} 6 26 14"); add(f"  stroke {line} 2")
        add("  path M 10 42 Q -2 36 4 20 Q 10 14 14 22 Q 10 30 14 38 Z")
    L.extend("  " + b for b in before)
    # Legs.
    add(f"  fill {legs}"); add(f"  stroke {line} 2")
    for x, y, h in ((8, 46, 14), (17, 48, 12), (27, 48, 12), (36, 46, 14)):
        add(f"  rect {x} {y} 9.5 {h} 4.7")
    # Body.
    add(f"  fill rad {light} {dark} 20 34 30"); add(f"  stroke {line} 2.2"); add("  ellipse 26 41 21 13.5")
    L.extend("  " + b for b in body)
    # Ears, behind the head.
    add(f"  fill rad {light} {dark} 40 18 26"); add(f"  stroke {line} 2.2")
    if ears == "round":
        add("  circle 33.5 13.5 6"); add("  circle 56.5 13.5 6")
        add(f"  fill {inner}"); add("  nostroke"); add("  circle 33.5 14 3"); add("  circle 56.5 14 3")
    elif ears == "big":
        add("  circle 34.5 14 8.5"); add("  circle 55.5 14 8.5")
        add(f"  fill {inner}"); add("  nostroke"); add("  circle 34.5 14.5 5.4"); add("  circle 55.5 14.5 5.4")
    elif ears == "pointed":
        add("  poly 31 22 31.5 6 43 14"); add("  poly 59 22 58.5 6 47 14")
        add(f"  fill {inner}"); add("  nostroke"); add("  poly 34 18 34.4 11 39.6 15"); add("  poly 56 18 55.6 11 50.4 15")
    elif ears == "floppy":
        add("  ellipse 30.5 24 5 10 12"); add("  ellipse 59.5 24 5 10 -12")
    elif ears == "long":
        add("  ellipse 38 8 4.6 12 -8"); add("  ellipse 52 8 4.6 12 8")
        add(f"  fill {inner}"); add("  nostroke"); add("  ellipse 38 9 2 8 -8"); add("  ellipse 52 9 2 8 8")
    elif ears == "side":
        add("  ellipse 29 20 6 4 -25"); add("  ellipse 61 20 6 4 25")
    # Head.
    add(f"  fill rad {light} {dark} 40 18 26"); add(f"  stroke {line} 2.2"); add("  circle 45 26 15.5")
    if muzzle:
        add(f"  fill {muzzle}"); add("  nostroke"); add("  ellipse 45 33 8.5 6.5")
    L.extend("  " + b for b in head)
    add("  use a3_eyes")
    if blush:
        add("  fill #ff8a8a66"); add("  nostroke"); add("  ellipse 34.5 31 3 2"); add("  ellipse 55.5 31 3 2")
    add(f"  fill {nose}"); add("  nostroke"); add("  poly 43 30 47 30 45 32.6")
    if mouth:
        add("  use a3_mouth")
    out.extend(L)
    out.append("")


HORNS = ["fill #fff3d6", "stroke #a8905a 1.6", "path M 33 15 Q 26 8 30 2 Q 33 8 37 12 Z", "path M 57 15 Q 64 8 60 2 Q 57 8 53 12 Z"]
SNOUT = lambda fill, line: [f"fill {fill}", f"stroke {line} 1.6", "ellipse 45 33.5 8 5.5", f"fill {line}", "nostroke", "ellipse 42 33.5 1.5 2", "ellipse 48 33.5 1.5 2"]  # noqa: E731

animal("ox", 0x1F402, "bull cattle farm animal", "#d99a5a", "#9a5a22", "#4a2a0c", tail="tuft", ears="side", mouth=False, nose="#00000000",
       head=HORNS + SNOUT("#ffd8c0", "#4a2a0c") + ["nofill", "stroke #c8a02a 2", "ellipse 45 37.5 2.6 2"])  # fmt: skip
animal("water_buffalo", 0x1F403, "buffalo cattle farm animal", "#7d8795", "#3f4652", "#16181d", tail="tuft", ears="side", mouth=False, nose="#00000000",
       head=["nofill", "stroke #16181d 5.4", "curve 35 15 28 12 28.5 2", "curve 55 15 62 12 61.5 2", "stroke #d6dde6 3", "curve 35 15 28 12 28.5 2", "curve 55 15 62 12 61.5 2"] + SNOUT("#a8b2c0", "#16181d"))  # fmt: skip
animal("cow2", 0x1F404, "cattle dairy farm moo animal", "#ffffff", "#d6dde6", "#5d6a7c", tail="tuft", ears="side", mouth=False, nose="#00000000", legs="#e3e7ec",
       body=["fill #2a2f3b", "nostroke", "blob 12 34 20 30 26 36 20 44 12 42", "blob 30 42 36 38 40 46 34 50"],
       head=HORNS + ["fill #2a2f3b", "nostroke", "blob 31 18 38 12 42 20 37 28 31 26"] + SNOUT("#ffb6c0", "#5d6a7c"))  # fmt: skip
STRIPES = ["nofill", "stroke #2a1c14 2.4", "line 14 31 15 40", "line 21 28.4 22 38", "line 28 28.6 28 37"]
animal("tiger2", 0x1F405, "big cat stripes wild animal", "#ffb85a", "#e87a1c", "#7a3a0c", tail="thin", ears="round", inner="#ffe0b0", muzzle="#ffffff", nose="#e8607a",
       body=STRIPES, head=["nofill", "stroke #2a1c14 2.2", "line 45 11.5 45 17", "line 40 12.5 41 17", "line 50 12.5 49 17", "line 30.6 24 35 25.4", "line 59.4 24 55 25.4"])  # fmt: skip
SPOTS = ["fill #6a4208", "nostroke", "circle 12 38 2", "circle 18 32 2.2", "circle 20 44 2", "circle 26 36 2.2", "circle 30 46 1.8", "circle 11 46 1.6"]
animal("leopard", 0x1F406, "big cat spots jaguar wild animal", "#ffe09a", "#e8aa3a", "#7a5a0c", tail="thin", ears="round", inner="#fff0cc", muzzle="#fff6e0", nose="#3a2a22",
       body=SPOTS, head=["fill #6a4208", "nostroke", "circle 40 14.5 1.6", "circle 50 14.5 1.6", "circle 45 12.5 1.4", "circle 33 27 1.4", "circle 57 27 1.4"])  # fmt: skip
animal("rabbit2", 0x1F407, "bunny hare hop animal easter", "#ffffff", "#d6dde6", "#8793a3", tail="puff", ears="long", inner="#ffb6c0", nose="#f07a9c", blush=True, legs="#e3e7ec")
animal("cat2", 0x1F408, "kitty feline pet animal", "#ffd88a", "#f5a12e", "#b0661a", tail="thin", ears="pointed", inner="#ffb0b0", muzzle="#fff3d6", nose="#ff8fa3",
       head=["nofill", "stroke #b0661a 1.1", "line 36 33 29 31.5", "line 36 35 29 36.5", "line 54 33 61 31.5", "line 54 35 61 36.5"])  # fmt: skip
def WOOL(c, l):
    """A fleece as one cloud: the outline is drawn first as larger lumps, so no lines cross inside."""
    lumps = [(9, 39, 7), (15, 31, 8), (25, 28, 8.5), (35, 31, 7), (11, 47, 7), (21, 51, 7.5), (32, 50, 7), (24, 40, 13)]
    out = [f"fill {l}", "nostroke"] + [f"circle {x} {y} {r + 1.2}" for x, y, r in lumps]
    out += [f"fill rad #ffffff {c} 18 30 34"] + [f"circle {x} {y} {r}" for x, y, r in lumps]
    return out + ["nofill", f"stroke {c} 1.8", "arc 16 38 5 4 200 330", "arc 27 36 5 4 200 330", "arc 20 46 5 4 200 330", "arc 31 44 4 3.4 200 330"]


def CAP(c, l):
    """The tuft of wool on the head, drawn the same way."""
    lumps = [(38.5, 13.5, 5.5), (45, 11, 6.5), (51.5, 13.5, 5.5)]
    return [f"fill {l}", "nostroke"] + [f"circle {x} {y} {r + 1.2}" for x, y, r in lumps] + [f"fill rad #ffffff {c} 44 9 12"] + [f"circle {x} {y} {r}" for x, y, r in lumps]
animal("ram_sheep", 0x1F40F, "sheep male horns animal ram", "#ffffff", "#d6dde6", "#8793a3", tail="none", ears="none", legs="#4a3a2c", muzzle="#f2e6d0", nose="#3a2a22",
       body=WOOL("#d6dde6", "#8793a3"),
       head=["nofill", "stroke #6e4526 6", "curve 36 15 29 12 27.5 21 32 26", "curve 54 15 61 12 62.5 21 58 26", "stroke #d9b88a 3", "curve 36 15 29 12 27.5 21 32 26", "curve 54 15 61 12 62.5 21 58 26"])  # fmt: skip
animal("goat", 0x1F410, "farm animal kid billy", "#fbf2e2", "#d0bfa0", "#6a5a44", tail="puff", ears="side", muzzle="#ffffff", nose="#3a2a22", legs="#b8a686",
       head=["fill #d9c8a8", "stroke #6a5a44 1.6", "path M 37 12 Q 34 4 38 0 Q 40 6 42 11 Z", "path M 53 12 Q 56 4 52 0 Q 50 6 48 11 Z", "fill #ffffff", "stroke #6a5a44 1.4", "path M 41 39.5 L 45 48 L 49 39.5 Z"])  # fmt: skip
animal("sheep", 0x1F411, "ewe lamb wool farm animal", "#fbead6", "#e6c8a6", "#7a5a3c", tail="none", ears="side", legs="#9a8068", muzzle="#fff6ea", nose="#8a5444", blush=True,
       body=WOOL("#dfe5ec", "#8793a3"), head=CAP("#dfe5ec", "#8793a3"))  # fmt: skip
animal("dog2", 0x1F415, "puppy pet canine walk animal", "#f5cb82", "#cd8f3c", "#6e4208", tail="thin", ears="floppy", muzzle="#fff3d6", nose="#2a1c14",
       head=["fill #cd8f3c", "nostroke", "blob 50 15 58 17 58 27 52 26", "fill #ff7184", "stroke #d94a60 1", "path M 43.4 36 Q 45 41 46.6 36 Z"])  # fmt: skip
animal("pig2", 0x1F416, "hog swine farm animal oink", "#ffd0dc", "#f29ab0", "#b84a6a", tail="curl", ears="pointed", inner="#f07a9c", mouth=False, nose="#00000000", blush=True,
       head=SNOUT("#f07a9c", "#b84a6a"))  # fmt: skip
animal("racehorse", 0x1F40E, "horse gallop race equestrian animal", "#d99a5a", "#9a5a22", "#4a2208", tail="none", ears="pointed", inner="#f2c89a", muzzle="#f2d0a8", nose="#00000000", mouth=False,
       before=["fill #4a2a12", "stroke #2a1808 1.8", "path M 10 34 Q -2 34 2 54 Q 8 50 12 42 Z"],
       body=["fill #e2584a", "stroke #8a140c 1.6", "path M 16 28.4 Q 24 26 32 28.6 L 31 38 Q 24 40 17 38 Z"],
       head=["fill #4a2a12", "nostroke", "path M 38 11.5 Q 45 6 52 11.5 Q 49 16 45 20 Q 41 16 38 11.5 Z", "fill #4a2208", "ellipse 42.4 33 1.3 1.8", "ellipse 47.6 33 1.3 1.8"])  # fmt: skip
animal("elephant", 0x1F418, "trunk big grey animal", "#cfd6de", "#8f99a8", "#4a525c", tail="tuft", ears="none", mouth=False, nose="#00000000", legs="#a3adba",
       before=[],
       head=["fill rad #dfe5ec #a3adba 28 22 14", "stroke #4a525c 2.2", "ellipse 29 26 8 12 8", "ellipse 61 26 8 12 -8", "fill rad #cfd6de #8f99a8 40 18 26", "circle 45 26 15.5",
             "path M 41 30 Q 40 46 46 50 Q 52 50 51 44 Q 48 44 48 40 L 49 30 Z", "nofill", "stroke #4a525c 1.2", "line 42 37 48 37", "line 42.6 42 48 41.4",
             "fill #fff6e0", "stroke #b8a37a 1.2", "path M 38 33 Q 34 40 38 43 Q 38 38 40 34 Z", "path M 52 33 Q 56 40 52 43 Q 52 38 50 34 Z"])  # fmt: skip
HUMP = lambda xs, l, d, ln: [f"fill rad {l} {d} 20 26 20", f"stroke {ln} 2.2"] + [f"path M {x - 8} 36 Q {x - 8} 12 {x} 12 Q {x + 8} 12 {x + 8} 36 Z" for x in xs]  # noqa: E731
animal("dromedary_camel", 0x1F42A, "camel desert one hump animal", "#f5d896", "#cc9a48", "#7a5214", tail="tuft", ears="side", muzzle="#fbe8c0", nose="#5a3a0c",
       body=HUMP([20], "#f5d896", "#cc9a48", "#7a5214"), head=["nofill", "stroke #7a5214 1.4", "arc 38.5 22 3.4 2 200 340", "arc 51.5 22 3.4 2 200 340"])  # fmt: skip
animal("camel", 0x1F42B, "bactrian two humps desert animal", "#ecc878", "#b8862a", "#6e4a0c", tail="tuft", ears="side", muzzle="#f5e0b0", nose="#5a3a0c",
       body=HUMP([12, 27], "#ecc878", "#b8862a", "#6e4a0c"), head=["nofill", "stroke #6e4a0c 1.4", "arc 38.5 22 3.4 2 200 340", "arc 51.5 22 3.4 2 200 340"])  # fmt: skip
animal("poodle", 0x1F429, "dog fluffy pet fancy animal", "#ffffff", "#dfe5ec", "#8793a3", tail="none", ears="none", muzzle="#ffffff", nose="#2a1c14", legs="#f2f5f8",
       before=["nofill", "stroke #8793a3 4.4", "line 7 38 3 28", "stroke #ffffff 2.2", "line 7 38 3 28", "fill rad #ffffff #dfe5ec 3 24 6", "stroke #8793a3 2", "circle 3 24 5"],
       body=["fill rad #ffffff #dfe5ec 14 56 6", "stroke #8793a3 1.8", "circle 12.7 57 5", "circle 21.7 58 4.6", "circle 31.7 58 4.6", "circle 40.7 57 5"],
       head=["fill rad #ffffff #dfe5ec 44 10 12", "stroke #8793a3 2", "circle 45 10 8.5", "circle 31 24 7", "circle 59 24 7"])  # fmt: skip

out.append(open(HAND, encoding="utf-8").read().rstrip("\n"))
out.append("")
with open(OUT, "w", encoding="utf-8", newline="\n") as f:
    f.write("\n".join(out))
print(sum(line.startswith("emoji ") for line in "\n".join(out).split("\n")), "emoji ->", OUT)


# ---- animals3.fsa: the newer animals, same template; the rest from art/animals3_hand.fsa.
OUT3 = os.path.join(HERE, "emoji", "animals3.fsa")
out[:] = [
    "// The newer animals. Original Remedy Emoji art.",
    "// The four-legged ones are written by art/gen_animals2.py; the rest are art/animals3_hand.fsa.",
    "category nature",
    "",
]
ANTLERS = ["nofill", "stroke #6e4526 4.4", "curve 39 13 36 6 30 4", "line 36.5 7.5 37 1.5", "curve 51 13 54 6 60 4", "line 53.5 7.5 53 1.5",
           "stroke #c9a878 2", "curve 39 13 36 6 30 4", "line 36.5 7.5 37 1.5", "curve 51 13 54 6 60 4", "line 53.5 7.5 53 1.5"]  # fmt: skip
animal("deer", 0x1F98C, "stag buck antlers forest animal", "#e0a868", "#b0702c", "#5f3a10", tail="puff", ears="side", muzzle="#f5dcb8", nose="#3a2a22",
       body=["fill #fff3d6", "nostroke", "circle 14 36 1.8", "circle 21 32.5 1.8", "circle 22 41 1.6", "circle 29 36 1.8"], head=ANTLERS)  # fmt: skip
animal("rhinoceros", 0x1F98F, "rhino horn safari animal", "#c4ccd6", "#868f9c", "#434a54", tail="tuft", ears="side", mouth=False, nose="#00000000", legs="#9aa3b0",
       head=["fill rad #d6dde6 #9aa3b0 45 34 10", "stroke #434a54 1.6", "ellipse 45 34.5 9 6.5", "fill #fff6e0", "stroke #b8a37a 1.6", "path M 41.5 32 Q 44 18 46 16 Q 48 24 48.5 32 Z",
             "fill #434a54", "nostroke", "ellipse 41.5 36.5 1.4 1.8", "ellipse 48.5 36.5 1.4 1.8"])  # fmt: skip
animal("llama", 0x1F999, "alpaca wool andes animal", "#fbf2e2", "#dcc9a6", "#7a6646", tail="puff", ears="pointed", inner="#f5c8c0", muzzle="#ffffff", nose="#7a5a4a",
       body=WOOL("#ece0c8", "#7a6646"), head=CAP("#ece0c8", "#7a6646"))  # fmt: skip
animal("hippopotamus", 0x1F99B, "hippo river africa animal", "#c8b4d8", "#8f78a8", "#4a3a62", tail="none", ears="round", inner="#f0c0d0", mouth=False, nose="#00000000", legs="#a892c0",
       head=["fill rad #dccdea #b29ccc 45 34 12", "stroke #4a3a62 1.8", "ellipse 45 34 12 8", "fill #4a3a62", "nostroke", "ellipse 40.5 31.5 1.6 2", "ellipse 49.5 31.5 1.6 2",
             "fill #ffffff", "stroke #4a3a62 1", "rect 40 38.6 3 3.4 0.8", "rect 47 38.6 3 3.4 0.8"])  # fmt: skip
animal("raccoon", 0x1F99D, "trash panda bandit mask animal", "#c8ced6", "#868f9c", "#3a4048", tail="none", ears="pointed", inner="#3a4048", muzzle="#ffffff", nose="#2a1c14",
       before=["fill rad #c8ced6 #868f9c 6 28 14", "stroke #3a4048 2", "path M 10 42 Q -2 36 4 20 Q 10 14 14 22 Q 10 30 14 38 Z", "nofill", "stroke #3a4048 3", "line 2.5 30 12 28", "line 3.6 23 12 22"],
       head=["fill #3a4048", "nostroke", "ellipse 38.5 25 5.4 4.4 20", "ellipse 51.5 25 5.4 4.4 -20"])  # fmt: skip
animal("badger", 0x1F9A1, "honey badger stripe animal", "#8a94a0", "#4f5762", "#22262e", tail="puff", ears="round", inner="#ffffff", muzzle="#ffffff", nose="#16181d",
       head=["fill #ffffff", "nostroke", "path M 42 11 Q 45 10 48 11 L 49 30 L 41 30 Z", "fill #22262e", "ellipse 38.5 25 4.6 6", "ellipse 51.5 25 4.6 6"])  # fmt: skip
animal("mammoth", 0x1F9A3, "woolly extinct ice age tusks animal", "#b98a5c", "#7a4e26", "#3f240c", tail="tuft", ears="none", mouth=False, nose="#00000000", legs="#8a5c30",
       head=["fill rad #c99a6c #8a5c30 28 22 14", "stroke #3f240c 2.2", "ellipse 30 26 7 11 8", "ellipse 60 26 7 11 -8", "fill rad #b98a5c #7a4e26 40 18 26", "circle 45 26 15.5",
             "fill #7a4e26", "nostroke", "path M 33 15 Q 45 4 57 15 Q 51 12 45 16 Q 39 12 33 15 Z",
             "fill rad #b98a5c #7a4e26 45 40 10", "stroke #3f240c 2.2", "path M 41 30 Q 40 46 46 50 Q 52 50 51 44 Q 48 44 48 40 L 49 30 Z",
             "fill #fff6e0", "stroke #b8a37a 1.4", "path M 38 33 Q 28 44 36 50 Q 34 42 40 35 Z", "path M 52 33 Q 62 44 54 50 Q 56 42 50 35 Z"])  # fmt: skip
animal("otter", 0x1F9A6, "river sea playful animal", "#b98a5c", "#7a4e26", "#3f240c", tail="thin", ears="round", inner="#7a4e26", muzzle="#f5dcb8", nose="#2a1c14", blush=True,
       body=["fill #f5dcb8", "nostroke", "ellipse 28 46 10 6"],
       head=["nofill", "stroke #3f240c 1.1", "line 37 33 30 31.5", "line 37 35 30 36.5", "line 53 33 60 31.5", "line 53 35 60 36.5"])  # fmt: skip
animal("skunk", 0x1F9A8, "smell stink stripe animal", "#5a6270", "#22262e", "#0e1014", tail="none", ears="round", inner="#22262e", muzzle="#3a3f4c", nose="#f07a9c",
       before=["fill rad #5a6270 #22262e 6 26 14", "stroke #0e1014 2", "path M 12 44 Q -4 38 2 18 Q 8 8 16 14 Q 22 22 14 28 Q 12 34 16 40 Z", "fill #ffffff", "nostroke", "path M 10 40 Q 0 34 5 20 Q 9 13 13 17 Q 8 24 10 32 Z"],
       body=["fill #ffffff", "nostroke", "path M 8 34 Q 22 26 40 31 L 40 35 Q 22 31 9 38 Z"],
       head=["fill #ffffff", "nostroke", "path M 43 11 Q 45 10 47 11 L 47.6 22 L 42.4 22 Z"])  # fmt: skip
animal("beaver", 0x1F9AB, "dam builder teeth river animal", "#b9824c", "#7a4a1c", "#3f240c", tail="none", ears="round", inner="#7a4a1c", muzzle="#f0d2a8", nose="#2a1c14", mouth=False,
       before=["fill rad #6a4a2c #3f2a14 4 44 10", "stroke #2a1808 2", "ellipse 5 46 6 11 -15", "nofill", "stroke #2a1808 1", "line 1 42 9 42", "line 0.6 47 9.6 47", "line 1.6 52 9 52"],
       head=["fill #ffffff", "stroke #3f240c 1", "rect 42.4 34 2.6 4.4 0.6", "rect 45 34 2.6 4.4 0.6"])  # fmt: skip
animal("bison", 0x1F9AC, "buffalo plains herd animal", "#8a5c36", "#4f3018", "#24140a", tail="tuft", ears="side", mouth=False, nose="#00000000", legs="#4f3018",
       body=["fill rad #6a4424 #3a220e 16 30 16", "stroke #24140a 2", "path M 8 34 Q 12 20 26 22 Q 38 24 40 36 Q 30 30 22 34 Q 14 30 8 34 Z"],
       head=["fill #3a220e", "nostroke", "path M 32 16 Q 45 4 58 16 Q 52 14 49 20 L 41 20 Q 38 14 32 16 Z", "fill #fff3d6", "stroke #a8905a 1.6",
             "path M 33 17 Q 26 12 28 4 Q 32 10 37 13 Z", "path M 57 17 Q 64 12 62 4 Q 58 10 53 13 Z"] + SNOUT("#6a4424", "#24140a") + ["fill #3a220e", "nostroke", "path M 39 40 Q 45 50 51 40 Q 45 43 39 40 Z"])  # fmt: skip
animal("guide_dog", 0x1F9AE, "service dog assistance blind animal", "#f5cb82", "#cd8f3c", "#6e4208", tail="thin", ears="floppy", muzzle="#fff3d6", nose="#2a1c14",
       body=["fill #ffd23a", "stroke #a8801c 1.6", "path M 22 28.2 L 30 28.6 L 30 53 L 22 53.6 Z", "nofill", "stroke #5d6a7c 3", "line 26 28 20 14 30 12"],
       head=["fill #ff7184", "stroke #d94a60 1", "path M 43.4 36 Q 45 41 46.6 36 Z"])  # fmt: skip
animal("hedgehog", 0x1F994, "spiky spines garden animal", "#f5dcb8", "#d9b07c", "#7a5a34", tail="none", ears="round", inner="#f0b0a0", muzzle="#fff3e0", nose="#2a1c14", blush=True, legs="#d9b07c",
       body=["fill rad #9a7850 #5a3e20 20 30 26", "stroke #3a2610 2", "poly 2 44 6 34 3 30 10 28 9 21 16 22 18 15 24 19 29 14 32 21 38 20 37 28 42 32 38 40 40 50 30 54 12 54 4 50",
             "nofill", "stroke #3a2610 1.4", "line 12 34 15 40", "line 20 28 22 35", "line 28 26 28 33", "line 14 44 18 48", "line 24 40 27 46", "line 33 34 33 41"])  # fmt: skip

out.append(open(os.path.join(HERE, "animals3_hand.fsa"), encoding="utf-8").read().rstrip("\n"))
out.append("")
with open(OUT3, "w", encoding="utf-8", newline="\n") as f:
    f.write("\n".join(out))
print(sum(line.startswith("emoji ") for line in "\n".join(out).split("\n")), "emoji ->", OUT3)
