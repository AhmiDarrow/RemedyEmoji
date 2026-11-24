//! Remedy Emoji: our own emoji and stickers, and the small vector format (`.fsa`) they
//! are written in. First drawn for FreeSpeak.
//!
//! Every glyph is original art written as plain text and drawn by the
//! rasterizer in this crate. Nothing is executed, nothing is fetched, and no
//! image decoder ever touches a sticker that arrived over the network: a
//! sticker is a short list of shapes with hard bounds on its size.
//!
//! See `docs/emoji.md` for the format.

mod font;
mod parse;
mod raster;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

pub use raster::{render, render_into, render_plain, Image};

/// Glyphs are drawn on a 64 by 64 unit canvas.
pub const CANVAS: f32 = 64.0;
/// Largest sticker source accepted from another member.
pub const MAX_STICKER_SOURCE_BYTES: usize = 8 * 1024;
/// Shapes in one glyph, after `use` is expanded.
pub const MAX_SHAPES: usize = 600;
/// Points in one glyph, after curves and strokes are flattened.
pub const MAX_POINTS: usize = 120_000;
/// Shapes and points in a sticker from another member. Smaller than the
/// built-in budget, because every device in the room draws it.
pub const MAX_STICKER_SHAPES: usize = 300;
pub const MAX_STICKER_POINTS: usize = 30_000;
/// Largest edge, in pixels, the rasterizer will draw.
pub const MAX_RENDER_PX: u32 = 512;
/// Skin tones: 0 is the default yellow, 1 to 5 follow the Unicode modifiers.
pub const TONES: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    /// 0 is a fixed colour. 1, 2 and 3 are the skin base, shade and line,
    /// replaced at draw time by the chosen tone.
    pub skin: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255, skin: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Solid(Color),
    Linear { from: Color, to: Color, x1: f32, y1: f32, x2: f32, y2: f32 },
    Radial { inner: Color, outer: Color, cx: f32, cy: f32, r: f32 },
}

/// Closed polygons filled as one union.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub paint: Paint,
    pub contours: Vec<Vec<[f32; 2]>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A reusable piece. Never shown in a picker.
    Part,
    Emoji,
    Sticker,
}

#[derive(Clone, Debug)]
pub struct Glyph {
    pub name: String,
    pub kind: Kind,
    /// The Unicode sequence this glyph stands for, without U+FE0F. Empty for
    /// FreeSpeak's own glyphs, which travel as `:name:`.
    pub chars: String,
    pub category: String,
    pub keywords: Vec<String>,
    /// True when the art uses the skin colours, so a tone changes it.
    pub tones: bool,
    pub shapes: Vec<Shape>,
}

impl Glyph {
    pub fn shortcode(&self) -> String {
        format!(":{}:", self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtError {
    pub origin: String,
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ArtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.origin, self.line, self.message)
    }
}

impl std::error::Error for ArtError {}

/// One piece of a chat line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seg<'a> {
    Text(&'a str),
    Glyph { index: usize, tone: u8, src: &'a str },
}

#[derive(Clone, Debug, Default)]
pub struct Library {
    glyphs: Vec<Glyph>,
    by_name: HashMap<String, usize>,
    by_chars: HashMap<String, usize>,
    first_chars: HashSet<char>,
    palette: HashMap<String, Color>,
}

const SHEETS: &[(&str, &str)] = &[
    ("parts.fsa", include_str!("../art/parts.fsa")),
    ("emoji/smileys.fsa", include_str!("../art/emoji/smileys.fsa")),
    ("emoji/smileys2.fsa", include_str!("../art/emoji/smileys2.fsa")),
    ("emoji/people.fsa", include_str!("../art/emoji/people.fsa")),
    ("emoji/nature.fsa", include_str!("../art/emoji/nature.fsa")),
    ("emoji/food.fsa", include_str!("../art/emoji/food.fsa")),
    ("emoji/gamer.fsa", include_str!("../art/emoji/gamer.fsa")),
    ("emoji/gamer_chat.fsa", include_str!("../art/emoji/gamer_chat.fsa")),
    ("emoji/streamer.fsa", include_str!("../art/emoji/streamer.fsa")),
    ("emoji/tech.fsa", include_str!("../art/emoji/tech.fsa")),
    ("emoji/science.fsa", include_str!("../art/emoji/science.fsa")),
    ("emoji/business.fsa", include_str!("../art/emoji/business.fsa")),
    ("emoji/music.fsa", include_str!("../art/emoji/music.fsa")),
    ("emoji/symbols.fsa", include_str!("../art/emoji/symbols.fsa")),
    ("stickers/originals.fsa", include_str!("../art/stickers/originals.fsa")),
];

/// The art that ships with this crate.
pub fn builtin() -> &'static Library {
    static LIB: OnceLock<Library> = OnceLock::new();
    LIB.get_or_init(|| {
        let mut lib = Library::new();
        for (name, src) in SHEETS {
            // The sheets are checked by `builtin_art_parses`; a bad sheet in a
            // release build drops out rather than taking the app down.
            let _ = lib.add_sheet(src, name);
        }
        lib
    })
}

/// Sheet names and sources, for tests and the preview tool.
pub fn builtin_sheets() -> &'static [(&'static str, &'static str)] {
    SHEETS
}

fn is_tone_modifier(c: char) -> bool {
    ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)
}

pub fn tone_modifier(tone: u8) -> Option<char> {
    match tone {
        1..=5 => char::from_u32(0x1F3FA + tone as u32),
        _ => None,
    }
}

impl Library {
    pub fn new() -> Self {
        let mut lib = Self::default();
        for (name, skin) in [("skin", 1u8), ("skind", 2), ("skinl", 3)] {
            let c = raster::tone_color(0, skin);
            lib.palette.insert(name.to_string(), Color { skin, ..c });
        }
        lib
    }

    /// Parse a sheet and add its glyphs. A sheet either loads whole or not at all.
    pub fn add_sheet(&mut self, src: &str, origin: &str) -> Result<usize, ArtError> {
        let mut next = self.clone();
        let added = parse::parse_sheet(&mut next, src, origin)?;
        *self = next;
        Ok(added)
    }

    /// Parse one sticker that came from another member. The source is small,
    /// holds exactly one sticker and no parts of its own, and may only `use`
    /// built-in art.
    pub fn parse_sticker(&self, src: &str) -> Result<Glyph, ArtError> {
        let err = |message: &str| ArtError { origin: "sticker".into(), line: 0, message: message.into() };
        if src.len() > MAX_STICKER_SOURCE_BYTES {
            return Err(err("sticker source is too large"));
        }
        // The sticker is read into an empty scratch library that can look at
        // this one but not change it, so nothing is copied per message.
        let mut scratch = Library::default();
        parse::parse_member_sticker(&mut scratch, self, src)?;
        let mut fresh = scratch.glyphs;
        if fresh.len() != 1 || fresh[0].kind != Kind::Sticker {
            return Err(err("expected exactly one sticker"));
        }
        Ok(fresh.remove(0))
    }

    pub(crate) fn push(&mut self, glyph: Glyph) -> Result<(), String> {
        if self.by_name.contains_key(&glyph.name) {
            return Err(format!("'{}' is already defined", glyph.name));
        }
        let index = self.glyphs.len();
        if !glyph.chars.is_empty() {
            if self.by_chars.contains_key(&glyph.chars) {
                return Err(format!("the characters of '{}' are already taken", glyph.name));
            }
            if let Some(c) = glyph.chars.chars().next() {
                self.first_chars.insert(c);
            }
            self.by_chars.insert(glyph.chars.clone(), index);
        }
        self.by_name.insert(glyph.name.clone(), index);
        self.glyphs.push(glyph);
        Ok(())
    }

    pub(crate) fn palette(&self) -> &HashMap<String, Color> {
        &self.palette
    }

    pub(crate) fn palette_mut(&mut self) -> &mut HashMap<String, Color> {
        &mut self.palette
    }

    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    pub fn get(&self, index: usize) -> Option<&Glyph> {
        self.glyphs.get(index)
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    pub fn by_name(&self, name: &str) -> Option<&Glyph> {
        self.index_of(name).map(|i| &self.glyphs[i])
    }

    /// Resolve what a reaction or a picker stored: `:name:` or a Unicode sequence.
    pub fn resolve(&self, token: &str) -> Option<(usize, u8)> {
        match self.segments(token).as_slice() {
            [Seg::Glyph { index, tone, .. }] => Some((*index, *tone)),
            _ => None,
        }
    }

    /// Categories in sheet order, for one kind.
    pub fn categories(&self, kind: Kind) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for g in self.glyphs.iter().filter(|g| g.kind == kind) {
            if !out.contains(&g.category.as_str()) {
                out.push(&g.category);
            }
        }
        out
    }

    /// Glyph indexes of one kind whose name or keywords contain every word of the query.
    pub fn search(&self, kind: Kind, query: &str) -> Vec<usize> {
        let words: Vec<String> = query.split_whitespace().map(|w| w.trim_matches(':').to_lowercase()).collect();
        self.glyphs
            .iter()
            .enumerate()
            .filter(|(_, g)| g.kind == kind)
            .filter(|(_, g)| {
                words.iter().all(|w| {
                    g.name.contains(w.as_str())
                        || g.category.contains(w.as_str())
                        || g.keywords.iter().any(|k| k.contains(w.as_str()))
                })
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Up to `limit` emoji whose name starts with `prefix`, then those that
    /// only contain it. For `:fi` completion in the composer.
    pub fn complete(&self, prefix: &str, limit: usize) -> Vec<usize> {
        let p = prefix.to_lowercase();
        let pool = || self.glyphs.iter().enumerate().filter(|(_, g)| g.kind == Kind::Emoji);
        let mut out: Vec<usize> = pool().filter(|(_, g)| g.name.starts_with(&p)).map(|(i, _)| i).collect();
        for (i, g) in pool() {
            if out.len() >= limit {
                break;
            }
            if !out.contains(&i) && (g.name.contains(&p) || g.keywords.iter().any(|k| k.starts_with(&p))) {
                out.push(i);
            }
        }
        out.truncate(limit);
        out
    }

    fn match_shortcode(&self, rest: &str) -> Option<(usize, usize)> {
        let body = rest.strip_prefix(':')?;
        let end = body.as_bytes().iter().take(34).position(|b| *b == b':')?;
        let name = &body[..end];
        let index = self.index_of(name)?;
        if self.glyphs[index].kind == Kind::Part {
            return None;
        }
        Some((index, end + 2))
    }

    fn match_chars(&self, rest: &str) -> Option<(usize, u8, usize)> {
        let first = rest.chars().next()?;
        if !self.first_chars.contains(&first) {
            return None;
        }
        let mut key = String::new();
        let mut best: Option<(usize, usize)> = None;
        for (count, (at, c)) in rest.char_indices().enumerate() {
            if count >= 8 {
                break;
            }
            let after = at + c.len_utf8();
            if c == '\u{FE0F}' {
                if let Some(b) = best.as_mut() {
                    if b.1 == at {
                        b.1 = after;
                    }
                }
                continue;
            }
            key.push(c);
            if let Some(i) = self.by_chars.get(&key) {
                best = Some((*i, after));
            }
        }
        let (index, mut len) = best?;
        let mut tone = 0;
        if let Some(c) = rest[len..].chars().next() {
            if is_tone_modifier(c) {
                tone = (c as u32 - 0x1F3FA) as u8;
                len += c.len_utf8();
            }
        }
        Some((index, tone, len))
    }

    /// Split a chat line into text and glyphs. Unknown `:names:` and
    /// characters with no art stay as text.
    pub fn segments<'a>(&self, text: &'a str) -> Vec<Seg<'a>> {
        let mut out = Vec::new();
        let mut text_from = 0;
        let mut i = 0;
        while i < text.len() {
            let rest = &text[i..];
            let hit = if rest.starts_with(':') {
                self.match_shortcode(rest).map(|(index, len)| (index, 0u8, len))
            } else {
                self.match_chars(rest)
            };
            match hit {
                Some((index, tone, len)) => {
                    if text_from < i {
                        out.push(Seg::Text(&text[text_from..i]));
                    }
                    out.push(Seg::Glyph { index, tone, src: &text[i..i + len] });
                    i += len;
                    text_from = i;
                }
                None => i += rest.chars().next().map(char::len_utf8).unwrap_or(1),
            }
        }
        if text_from < text.len() {
            out.push(Seg::Text(&text[text_from..]));
        }
        out
    }

    /// What goes on the wire: `:name:` becomes the Unicode character where
    /// the glyph has one, so any client or bot reads it as a normal emoji.
    /// FreeSpeak's own glyphs stay `:name:`.
    pub fn to_wire(&self, text: &str, tone: u8) -> String {
        let mut out = String::with_capacity(text.len());
        for seg in self.segments(text) {
            match seg {
                Seg::Text(t) => out.push_str(t),
                Seg::Glyph { index, src, .. } => {
                    let g = &self.glyphs[index];
                    if src.starts_with(':') && !g.chars.is_empty() {
                        out.push_str(&g.chars);
                        if g.tones {
                            if let Some(m) = tone_modifier(tone) {
                                out.push(m);
                            }
                        }
                    } else {
                        out.push_str(src);
                    }
                }
            }
        }
        out
    }

    /// The reverse, for a text field whose font has no emoji: known Unicode
    /// emoji become `:name:`.
    pub fn to_shortcodes(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for seg in self.segments(text) {
            match seg {
                Seg::Text(t) => out.push_str(t),
                Seg::Glyph { index, .. } => out.push_str(&self.glyphs[index].shortcode()),
            }
        }
        out
    }
}

/// How a chat line made only of glyphs should be shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Solo {
    /// Text, or too many glyphs: draw inline.
    No,
    /// One to six emoji and nothing else: draw them large.
    Emoji,
    /// Exactly one sticker and nothing else.
    Sticker(usize),
}

pub fn solo(lib: &Library, segs: &[Seg<'_>]) -> Solo {
    let mut glyphs = Vec::new();
    for seg in segs {
        match seg {
            Seg::Text(t) if t.trim().is_empty() => {}
            Seg::Text(_) => return Solo::No,
            Seg::Glyph { index, .. } => glyphs.push(*index),
        }
    }
    let kind = |i: &usize| lib.get(*i).map(|g| g.kind);
    match glyphs.as_slice() {
        [one] if kind(one) == Some(Kind::Sticker) => Solo::Sticker(*one),
        [] => Solo::No,
        many if many.len() <= 6 && many.iter().all(|i| kind(i) == Some(Kind::Emoji)) => Solo::Emoji,
        _ => Solo::No,
    }
}

#[cfg(test)]
mod tests;
