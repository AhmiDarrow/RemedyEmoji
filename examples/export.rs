//! Write every emoji and sticker as JSON, for `font/build.py` (which builds the font) and
//! for anything else that wants the shapes without this crate.
//!
//! cargo run --release --example export -- target/art.json
//!
//! Colours are `[r, g, b, a, skin]`: `skin` 0 is a fixed colour; 1, 2 and 3 are the skin
//! base, shade and line, whose colour for each tone is in the top-level `tones` table.
//! Points are in canvas units (64 by 64, y down).

use std::fmt::Write as _;

use remedy_emoji::{builtin, tone_color, Color, Kind, Paint, CANVAS, TONES};

fn text(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn color(c: Color) -> String {
    format!("[{},{},{},{},{}]", c.r, c.g, c.b, c.a, c.skin)
}

fn paint(p: &Paint) -> String {
    match *p {
        Paint::Solid(c) => format!("{{\"solid\":{}}}", color(c)),
        Paint::Linear { from, to, x1, y1, x2, y2 } => {
            format!("{{\"linear\":[{},{},{x1},{y1},{x2},{y2}]}}", color(from), color(to))
        }
        Paint::Radial { inner, outer, cx, cy, r } => {
            format!("{{\"radial\":[{},{},{cx},{cy},{r}]}}", color(inner), color(outer))
        }
    }
}

fn main() -> std::io::Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: export <out.json>");
        std::process::exit(2);
    };
    let mut out = String::new();
    let _ = write!(out, "{{\"canvas\":{CANVAS},\"tones\":[");
    for tone in 0..TONES {
        let row: Vec<String> = (1..=3)
            .map(|slot| {
                let c = tone_color(tone, slot);
                format!("[{},{},{}]", c.r, c.g, c.b)
            })
            .collect();
        let _ = write!(out, "{}[{}]", if tone == 0 { "" } else { "," }, row.join(","));
    }
    out.push_str("],\"glyphs\":[");
    let mut first = true;
    for g in builtin().glyphs().iter().filter(|g| g.kind != Kind::Part) {
        if !std::mem::take(&mut first) {
            out.push(',');
        }
        let kind = if g.kind == Kind::Sticker { "sticker" } else { "emoji" };
        let chars: Vec<String> = g.chars.chars().map(|c| (c as u32).to_string()).collect();
        let keywords: Vec<String> = g.keywords.iter().map(|k| text(k)).collect();
        let _ = write!(
            out,
            "\n{{\"name\":{},\"kind\":\"{kind}\",\"chars\":[{}],\"category\":{},\"keywords\":[{}],\"tones\":{},\"shapes\":[",
            text(&g.name),
            chars.join(","),
            text(&g.category),
            keywords.join(","),
            g.tones,
        );
        for (i, s) in g.shapes.iter().enumerate() {
            let _ = write!(out, "{}{{\"paint\":{},\"contours\":[", if i == 0 { "" } else { "," }, paint(&s.paint));
            for (j, c) in s.contours.iter().enumerate() {
                let pts: Vec<String> = c.iter().map(|p| format!("{:.3},{:.3}", p[0], p[1])).collect();
                let _ = write!(out, "{}[{}]", if j == 0 { "" } else { "," }, pts.join(","));
            }
            out.push_str("]}");
        }
        out.push_str("]}");
    }
    out.push_str("\n]}\n");
    std::fs::write(path, out)
}
