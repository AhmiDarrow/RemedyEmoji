# Remedy Emoji

Our own emoji and stickers: 677 emoji in 13 categories and 46 stickers, all original art.
No emoji font and no third-party image set. First drawn for FreeSpeak.

Each glyph is a few lines of text in a small vector format (`.fsa`) and is drawn by the
rasterizer in this crate. The crate has no dependencies.

- Nothing in a glyph is executed, and it cannot name a file or a URL.
- No image decoder runs on a sticker that came from someone else: the reader has hard
  limits on source size, shapes and points.
- A glyph draws sharply at any size, from a 20 pixel chat line to a 160 pixel sticker.
- Six skin tones, chosen when drawing.

## Use

```toml
[dependencies]
remedy-emoji = { git = "https://github.com/AhmiDarrow/RemedyEmoji" }
```

```rust
let lib = remedy_emoji::builtin();
if let Some(glyph) = lib.by_name("grinning") {
    // 64 x 64 pixels, default skin tone; `image.rgba` is width * height * 4 bytes.
    let image = remedy_emoji::render(glyph, 64, 0);
}
// Split a chat line into text and glyphs (Unicode emoji and `:name:` shortcodes).
let parts = lib.segments("well done :gg: 👍");
```

`Library` also has the picker's needs: `categories`, `search`, `complete`, and
`parse_sticker` for a sticker received from someone else.

## The art

- `art/parts.fsa`: shared colours and reusable pieces (the face, eyes, a heart, a star).
- `art/emoji/*.fsa`: one sheet per picker category.
- `art/stickers/*.fsa`: the built-in sticker pack.

The format is described in [docs/emoji.md](docs/emoji.md). To look at the art, draw a
contact sheet:

```
cargo run --example sheet -- sheet.png --px 96 --cols 8
```

## Check

```
cargo test
```

## License

MIT or Apache-2.0, at your option, for the code and the art.
