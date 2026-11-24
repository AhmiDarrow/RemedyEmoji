//! Draw a contact sheet of FreeSpeak art as a PNG, for reviewing glyphs.
//!
//! cargo run --example sheet -- out.png [--px 64] [--cols 10]
//!     [--light] [--kind emoji|sticker|all] [--only category] [--tone 0..5] [files.fsa ...]
//!
//! With no files, the built-in art is drawn. With files, they are loaded in
//! order on top of an empty library (list `parts.fsa` first).

use std::process::ExitCode;

use remedy_emoji::{render, Kind, Library};

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let mut tagged = kind.to_vec();
    tagged.extend_from_slice(body);
    out.extend_from_slice(&tagged);
    out.extend_from_slice(&crc32(&tagged).to_be_bytes());
}

/// An uncompressed RGB PNG.
fn png(width: usize, height: usize, rgb: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width * 3 + 1) * height);
    for row in rgb.chunks(width * 3) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65_535).peekable();
    while let Some(block) = blocks.next() {
        z.push(u8::from(blocks.peek().is_none()));
        z.extend_from_slice(&(block.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + *byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut head = Vec::new();
    head.extend_from_slice(&(width as u32).to_be_bytes());
    head.extend_from_slice(&(height as u32).to_be_bytes());
    head.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &head);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

struct Sheet {
    width: usize,
    rgb: Vec<u8>,
}

impl Sheet {
    fn blit(&mut self, img: &remedy_emoji::Image, x0: usize, y0: usize, rows: usize) {
        for y in 0..rows.min(img.height as usize) {
            for x in 0..img.width as usize {
                let s = &img.rgba[(y * img.width as usize + x) * 4..][..4];
                let at = ((y0 + y) * self.width + x0 + x) * 3;
                if at + 3 > self.rgb.len() || x0 + x >= self.width {
                    continue;
                }
                let a = s[3] as u32;
                for (to, from) in self.rgb[at..at + 3].iter_mut().zip(s) {
                    *to = ((*from as u32 * a + *to as u32 * (255 - a)) / 255) as u8;
                }
            }
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(out_path) = args.next() else {
        eprintln!("usage: sheet out.png [--px N] [--cols N] [--light] [--kind emoji|sticker|all] [--only category] [--tone N] [files.fsa ...]");
        return ExitCode::FAILURE;
    };
    let (mut px, mut cols, mut light, mut kind, mut only, mut tone) = (64usize, 10usize, false, "emoji".to_string(), None, 0u8);
    let mut files = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--px" => px = args.next().and_then(|v| v.parse().ok()).unwrap_or(64),
            "--cols" => cols = args.next().and_then(|v| v.parse().ok()).unwrap_or(10),
            "--tone" => tone = args.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            "--light" => light = true,
            "--kind" => kind = args.next().unwrap_or_default(),
            "--only" => only = args.next(),
            _ => files.push(a),
        }
    }
    let px = px.clamp(16, 512);
    let lib = if files.is_empty() {
        let mut lib = Library::new();
        for (name, src) in remedy_emoji::builtin_sheets() {
            if let Err(e) = lib.add_sheet(src, name) {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
        lib
    } else {
        let mut lib = Library::new();
        for f in &files {
            let src = match std::fs::read_to_string(f) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{f}: {e}");
                    return ExitCode::FAILURE;
                }
            };
            if let Err(e) = lib.add_sheet(&src, f) {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
        lib
    };
    let shown: Vec<&remedy_emoji::Glyph> = lib
        .glyphs()
        .iter()
        .filter(|g| match kind.as_str() {
            "all" => true,
            "sticker" => g.kind == Kind::Sticker,
            "part" => g.kind == Kind::Part,
            _ => g.kind == Kind::Emoji,
        })
        .filter(|g| only.as_deref().is_none_or(|c| g.category == c))
        .collect();
    if shown.is_empty() {
        eprintln!("nothing to draw");
        return ExitCode::FAILURE;
    }
    let pad = 10;
    let label_h = (px / 4).clamp(14, 26);
    let cell_w = (px + 30).max(96) + pad;
    let cell_h = px + label_h + pad;
    // A column of small sizes beside each glyph shows how it reads in a chat line.
    let rows = shown.len().div_ceil(cols);
    let width = cols * cell_w + pad;
    let height = rows * cell_h + pad;
    let bg: [u8; 3] = if light { [244, 242, 236] } else { [20, 20, 19] };
    let ink = if light { "#3a3833" } else { "#b9b5ab" };
    let mut sheet = Sheet { width, rgb: bg.iter().copied().cycle().take(width * height * 3).collect() };
    for (i, g) in shown.iter().enumerate() {
        let x0 = pad + (i % cols) * cell_w;
        let y0 = pad + (i / cols) * cell_h;
        let big = render(g, px as u32, tone);
        sheet.blit(&big, x0, y0, px);
        let small = render(g, 22, tone);
        sheet.blit(&small, x0 + px + 4, y0, 22);
        let short: String = g.name.to_uppercase().chars().take(24).collect();
        let lh = (58.0 / (short.len() as f32 * 0.92)).min(5.2);
        let label_src = format!("sticker label\nfill {ink}\ntext 32 9 {lh:.2} {short} 0.8\n");
        if let Ok(label) = lib.parse_sticker(&label_src) {
            let img = remedy_emoji::render_plain(&label, cell_w as u32, 0);
            sheet.blit(&img, x0 - pad / 2, y0 + px + 2, label_h);
        }
    }
    if let Err(e) = std::fs::write(&out_path, png(width, height, &sheet.rgb)) {
        eprintln!("{out_path}: {e}");
        return ExitCode::FAILURE;
    }
    println!("{} glyphs -> {out_path} ({width}x{height})", shown.len());
    ExitCode::SUCCESS
}
