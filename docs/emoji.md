# Emoji and stickers

Remedy Emoji is our own emoji and sticker set, first drawn for FreeSpeak. There is no emoji font and no third-party image set in the tree: every glyph is original art written as a small text file and drawn by the `remedy-emoji` crate.

The art lives in `art/`:

- `parts.fsa` holds shared colours and reusable pieces (the face, eyes, a heart, a star).
- `emoji/*.fsa` holds one sheet per picker category.
- `stickers/*.fsa` holds the built-in sticker pack.

## Why a text format

- A sticker is a short list of shapes. Nothing in it is executed, and it cannot name a file or a URL.
- No image decoder runs on a sticker that came from another member. The reader has hard bounds on source size, shapes, and points.
- A glyph draws sharply at any size, from a 20 pixel chat line to a 160 pixel sticker.
- Anyone can add or change art with a text editor.

## Looking at the art

```
cargo run --example sheet -- sheet.png --px 96 --cols 8
```

Options: `--light` for a light background, `--kind emoji|sticker|part|all`, `--only <category>`, `--tone 0..5`, and a list of `.fsa` files to draw instead of the built-in art (list `parts.fsa` first). Each cell shows the glyph large and at chat size.

## The format

A sheet is plain text, one command per line. `//` starts a comment. The canvas is 64 by 64 units with y pointing down, so (0,0) is the top left and (32,32) is the centre.

### Headers

| Line | Meaning |
| --- | --- |
| `category smileys` | Category for the glyphs that follow. |
| `color name #rrggbb` | A named colour, used as `$name`. |
| `part name` | A reusable piece. Never shown in a picker. |
| `emoji name U+1F600 \| keywords` | An emoji. The code point is optional. |
| `sticker name \| keywords` | A sticker. |

Names use `a-z`, `0-9` and `_`, up to 32 characters. Parts have their own namespace, so a part and an emoji may share a name.

An emoji with a code point is sent as that Unicode character, so other apps and bots read it normally. Several code points are joined with `-` (`U+2764-FE0F`). An emoji without one is our own and is sent as `:name:`.

### Style

Style applies to the shapes after it and resets at each header to a black fill and no stroke.

| Line | Meaning |
| --- | --- |
| `fill #ffcc33` | Solid fill. `#rgb`, `#rrggbb`, `#rrggbbaa`, or `$name`. |
| `fill lin A B x1 y1 x2 y2` | Linear gradient from colour A at the first point to B at the second. |
| `fill rad A B cx cy r` | Radial gradient, A at the centre, B at radius r. |
| `nofill` | No fill. |
| `stroke #4a2c10 3` | Outline colour and width. Ends and corners are round. |
| `nostroke` | No outline. |

The colours `$skin`, `$skind` and `$skinl` are the skin base, shade and line. An emoji that uses them follows the sender's skin tone.

### Shapes

| Line | Meaning |
| --- | --- |
| `circle cx cy r` | |
| `ellipse cx cy rx ry [degrees]` | |
| `rect x y w h [radius]` | |
| `poly x y x y ...` | Closed polygon. |
| `blob x y x y ...` | Closed smooth curve through the points. |
| `path M x y L x y Q cx cy x y C c1x c1y c2x c2y x y Z` | Lines, quadratic and cubic curves. Absolute coordinates, each command a separate word. |
| `line x y x y ...` | Open line. Needs a stroke. |
| `curve x y x y ...` | Open smooth curve through the points. Needs a stroke. |
| `arc cx cy rx ry from to` | Open arc in degrees. 0 is right, 90 is down, so `25 155` is a smile and `205 335` is a frown. Needs a stroke. |
| `text cx baseline height WORDS [weight]` | Lettering in the built-in stroke alphabet, centred on `cx`, drawn in the fill. `_` is a space. |

### Reuse and placement

| Line | Meaning |
| --- | --- |
| `use name [dx dy [scale [degrees]]] [flip]` | Draw another glyph or part, scaled and turned around the canvas centre, then moved. |
| `xf dx dy [scale [degrees]] [flip]` | Do the same to every shape that follows. `xf` alone resets it. |
| `mirror <shape or use>` | Draw it, then draw it again mirrored left to right. |

### Example

```
emoji wink U+1F609 | flirt joke
  use face
  fill $ink
  ellipse 22 26 3.3 4.5
  stroke $ink 3
  arc 42 28.5 5 4 200 340
  use smile
```

## Limits

| Limit | Value |
| --- | --- |
| Sticker source from another member | 8 KiB |
| Shapes in one glyph | 600 |
| Points in one glyph, after curves and outlines are flattened | 120,000 |
| Largest drawn edge | 512 pixels |
| Line length | 4096 bytes |

A sheet loads whole or not at all. A sticker from another member must hold exactly one sticker and can only `use` built-in art.
