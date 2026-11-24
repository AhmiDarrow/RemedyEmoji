//! Scanline rasterizer: exact coverage along x, several sample rows per
//! pixel along y, non-zero winding so overlapping pieces form a union.

use crate::{Color, Glyph, Paint, CANVAS, MAX_RENDER_PX};

/// Sample rows per pixel row.
const SUB: usize = 6;

/// Straight (not premultiplied) RGBA, row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Base, shade and line colour for each skin tone.
const TONE_TABLE: [[(u8, u8, u8); 3]; 6] = [
    [(255, 204, 51), (240, 165, 30), (166, 98, 12)],
    [(247, 215, 181), (229, 183, 143), (176, 122, 88)],
    [(235, 193, 153), (212, 159, 114), (160, 108, 68)],
    [(201, 150, 104), (172, 119, 72), (122, 80, 44)],
    [(156, 101, 64), (127, 77, 44), (86, 50, 26)],
    [(107, 69, 47), (82, 50, 32), (52, 30, 16)],
];

pub(crate) fn tone_color(tone: u8, slot: u8) -> Color {
    let row = TONE_TABLE[(tone as usize).min(TONE_TABLE.len() - 1)];
    let (r, g, b) = row[(slot.clamp(1, 3) - 1) as usize];
    Color::rgb(r, g, b)
}

fn resolve(c: Color, tone: u8) -> [f32; 4] {
    let c = if c.skin == 0 { c } else { Color { a: c.a, ..tone_color(tone, c.skin) } };
    [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, c.a as f32 / 255.0]
}

fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

enum Ink {
    Solid([f32; 4]),
    Linear { from: [f32; 4], to: [f32; 4], x1: f32, y1: f32, dx: f32, dy: f32, inv: f32 },
    Radial { inner: [f32; 4], outer: [f32; 4], cx: f32, cy: f32, inv: f32 },
}

impl Ink {
    fn new(paint: &Paint, tone: u8) -> Self {
        match *paint {
            Paint::Solid(c) => Ink::Solid(resolve(c, tone)),
            Paint::Linear { from, to, x1, y1, x2, y2 } => {
                let (dx, dy) = (x2 - x1, y2 - y1);
                let len2 = dx * dx + dy * dy;
                Ink::Linear {
                    from: resolve(from, tone),
                    to: resolve(to, tone),
                    x1,
                    y1,
                    dx,
                    dy,
                    inv: if len2 > 1e-6 { 1.0 / len2 } else { 0.0 },
                }
            }
            Paint::Radial { inner, outer, cx, cy, r } => Ink::Radial {
                inner: resolve(inner, tone),
                outer: resolve(outer, tone),
                cx,
                cy,
                inv: if r > 1e-3 { 1.0 / r } else { 0.0 },
            },
        }
    }

    /// Colour at a point given in canvas units.
    fn at(&self, x: f32, y: f32) -> [f32; 4] {
        match self {
            Ink::Solid(c) => *c,
            Ink::Linear { from, to, x1, y1, dx, dy, inv } => {
                mix(*from, *to, ((x - x1) * dx + (y - y1) * dy) * inv)
            }
            Ink::Radial { inner, outer, cx, cy, inv } => {
                let d = ((x - cx) * (x - cx) + (y - cy) * (y - cy)).sqrt();
                mix(*inner, *outer, d * inv)
            }
        }
    }
}

struct Edge {
    x0: f32,
    y0: f32,
    y1: f32,
    slope: f32,
    dir: i32,
}

fn add_span(cov: &mut [f32], a: f32, b: f32, amount: f32) {
    let w = cov.len() as f32;
    // A non-finite crossing would pass the comparison below and index out of range.
    if !a.is_finite() || !b.is_finite() {
        return;
    }
    let (a, b) = (a.clamp(0.0, w), b.clamp(0.0, w));
    if b <= a {
        return;
    }
    let (ia, ib) = (a as usize, b as usize);
    if ia >= cov.len() {
        return;
    }
    if ia == ib {
        cov[ia] += (b - a) * amount;
        return;
    }
    cov[ia] += (ia as f32 + 1.0 - a) * amount;
    if let Some(mid) = cov.get_mut(ia + 1..ib) {
        for c in mid {
            *c += amount;
        }
    }
    if ib < cov.len() {
        cov[ib] += (b - ib as f32) * amount;
    }
}

/// Draw `glyph` over a premultiplied float canvas of `px` by `px` pixels.
/// The glyph is scaled by `scale` around the canvas centre and moved by
/// (`dx`, `dy`) pixels, which lets a caller animate or compose.
pub fn render_into(canvas: &mut [[f32; 4]], px: u32, glyph: &Glyph, tone: u8, scale: f32, dx: f32, dy: f32) {
    let px = px.min(MAX_RENDER_PX) as usize;
    if px == 0 || canvas.len() < px * px {
        return;
    }
    let unit = px as f32 / CANVAS * scale;
    let off_x = px as f32 * (1.0 - scale) / 2.0 + dx;
    let off_y = px as f32 * (1.0 - scale) / 2.0 + dy;
    let mut edges: Vec<Edge> = Vec::new();
    let mut crossings: Vec<(f32, i32)> = Vec::new();
    let mut cov = vec![0.0f32; px];
    for shape in &glyph.shapes {
        edges.clear();
        let (mut top, mut bottom) = (f32::MAX, f32::MIN);
        for contour in &shape.contours {
            let n = contour.len();
            if n < 3 {
                continue;
            }
            for i in 0..n {
                let (p, q) = (contour[i], contour[(i + 1) % n]);
                let (ax, ay) = (p[0] * unit + off_x, p[1] * unit + off_y);
                let (bx, by) = (q[0] * unit + off_x, q[1] * unit + off_y);
                if (ay - by).abs() < 1e-6 {
                    continue;
                }
                let (x0, y0, x1, y1, dir) = if ay < by { (ax, ay, bx, by, 1) } else { (bx, by, ax, ay, -1) };
                top = top.min(y0);
                bottom = bottom.max(y1);
                edges.push(Edge { x0, y0, y1, slope: (x1 - x0) / (y1 - y0), dir });
            }
        }
        if edges.is_empty() {
            continue;
        }
        let ink = Ink::new(&shape.paint, tone);
        let row_from = top.floor().max(0.0) as usize;
        let row_to = (bottom.ceil().max(0.0) as usize).min(px);
        for row in row_from..row_to {
            cov.fill(0.0);
            let mut touched = false;
            for k in 0..SUB {
                let sy = row as f32 + (k as f32 + 0.5) / SUB as f32;
                crossings.clear();
                for e in &edges {
                    if sy >= e.y0 && sy < e.y1 {
                        let x = e.x0 + (sy - e.y0) * e.slope;
                        if x.is_finite() {
                            crossings.push((x, e.dir));
                        }
                    }
                }
                if crossings.len() < 2 {
                    continue;
                }
                crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut winding = 0;
                let mut start = 0.0;
                for (x, dir) in &crossings {
                    let before = winding;
                    winding += dir;
                    if before == 0 && winding != 0 {
                        start = *x;
                    } else if before != 0 && winding == 0 {
                        add_span(&mut cov, start, *x, 1.0 / SUB as f32);
                        touched = true;
                    }
                }
            }
            if !touched {
                continue;
            }
            let line = &mut canvas[row * px..(row + 1) * px];
            for (col, c) in cov.iter().enumerate() {
                if *c <= 0.0 {
                    continue;
                }
                let ux = (col as f32 + 0.5 - off_x) / unit;
                let uy = (row as f32 + 0.5 - off_y) / unit;
                let s = ink.at(ux, uy);
                let a = s[3] * c.min(1.0);
                let d = &mut line[col];
                d[0] = s[0] * a + d[0] * (1.0 - a);
                d[1] = s[1] * a + d[1] * (1.0 - a);
                d[2] = s[2] * a + d[2] * (1.0 - a);
                d[3] = a + d[3] * (1.0 - a);
            }
        }
    }
}

/// Turn a premultiplied float canvas into straight RGBA bytes.
pub(crate) fn finish(canvas: &[[f32; 4]], px: u32) -> Image {
    let mut rgba = Vec::with_capacity(canvas.len() * 4);
    for p in canvas {
        let a = p[3];
        if a <= 1e-4 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        for c in &p[..3] {
            rgba.push(((c / a).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
        rgba.push((a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
    }
    Image { width: px, height: px, rgba }
}

/// Stickers get a die-cut look: the art is drawn a little smaller and a
/// white border follows its outline, with a thin dark rim so it also shows
/// on a light background.
fn die_cut(art: &[[f32; 4]], px: usize) -> Vec<[f32; 4]> {
    let border = (px as f32 * 0.042).max(1.5);
    let rim = (px as f32 * 0.012).max(0.8);
    let reach = (border + rim).ceil() as usize + 1;
    let far = (reach * 4) as f32;
    // Distance along each row to the nearest opaque pixel.
    let mut row_dist = vec![far; px * px];
    for y in 0..px {
        let row = &mut row_dist[y * px..(y + 1) * px];
        let mut d = far;
        for x in 0..px {
            d = if art[y * px + x][3] >= 0.5 { 0.0 } else { d + 1.0 };
            row[x] = d;
        }
        d = far;
        for x in (0..px).rev() {
            d = if art[y * px + x][3] >= 0.5 { 0.0 } else { d + 1.0 };
            row[x] = row[x].min(d);
        }
    }
    let mut out = vec![[0.0f32; 4]; px * px];
    for y in 0..px {
        let (from, to) = (y.saturating_sub(reach), (y + reach).min(px - 1));
        for x in 0..px {
            let mut best = far;
            for yy in from..=to {
                let dx = row_dist[yy * px + x];
                if dx < best {
                    let dy = yy as f32 - y as f32;
                    best = best.min((dx * dx + dy * dy).sqrt());
                }
            }
            let white = (border + 0.5 - best).clamp(0.0, 1.0);
            let dark = (border + rim + 0.5 - best).clamp(0.0, 1.0) * 0.45;
            let o = &mut out[y * px + x];
            // Dark rim, then the white border over it, then the art over both.
            *o = [0.07 * dark, 0.07 * dark, 0.09 * dark, dark];
            for (k, c) in [0.985f32, 0.985, 0.97, 1.0].iter().enumerate() {
                o[k] = c * white + o[k] * (1.0 - white);
            }
            let a = art[y * px + x];
            for k in 0..4 {
                o[k] = a[k] + o[k] * (1.0 - a[3]);
            }
        }
    }
    out
}

/// Draw a glyph with no sticker border, whatever its kind.
pub fn render_plain(glyph: &Glyph, px: u32, tone: u8) -> Image {
    let px = px.clamp(1, MAX_RENDER_PX);
    let mut canvas = vec![[0.0f32; 4]; (px * px) as usize];
    render_into(&mut canvas, px, glyph, tone, 1.0, 0.0, 0.0);
    finish(&canvas, px)
}

/// Draw a glyph as a square image `px` pixels on a side.
pub fn render(glyph: &Glyph, px: u32, tone: u8) -> Image {
    let px = px.clamp(1, MAX_RENDER_PX);
    let mut canvas = vec![[0.0f32; 4]; (px * px) as usize];
    if glyph.kind == crate::Kind::Sticker && px >= 48 {
        render_into(&mut canvas, px, glyph, tone, 0.88, 0.0, 0.0);
        canvas = die_cut(&canvas, px as usize);
    } else {
        render_into(&mut canvas, px, glyph, tone, 1.0, 0.0, 0.0);
    }
    finish(&canvas, px)
}
