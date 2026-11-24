//! The `.fsa` reader. Line based, bounded, and with no way to reach outside
//! the sheet: there are no includes, no scripts, and no file or network access.

use crate::{
    font, ArtError, Color, Glyph, Kind, Library, Paint, Shape, MAX_POINTS, MAX_SHAPES, MAX_STICKER_POINTS,
    MAX_STICKER_SHAPES,
};

type P = [f32; 2];

const CENTER: f32 = 32.0;
const MAX_LINE_BYTES: usize = 4096;
const MAX_NUMBER: f32 = 10_000.0;
const MAX_KEYWORDS: usize = 24;
/// No point may leave this box, whatever chain of `use` and `xf` produced it.
const MAX_COORD: f32 = 100_000.0;

#[derive(Clone, Copy)]
struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine {
    const IDENTITY: Affine = Affine { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };

    /// Scale, turn and optionally mirror around the canvas centre, then move.
    fn about_center(dx: f32, dy: f32, scale: f32, deg: f32, flip: bool) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();
        let fx = if flip { -1.0 } else { 1.0 };
        let (a, b, c, d) = (scale * cos * fx, scale * sin * fx, -scale * sin, scale * cos);
        Affine {
            a,
            b,
            c,
            d,
            e: CENTER + dx - (a * CENTER + c * CENTER),
            f: CENTER + dy - (b * CENTER + d * CENTER),
        }
    }

    fn apply(&self, p: P) -> P {
        [self.a * p[0] + self.c * p[1] + self.e, self.b * p[0] + self.d * p[1] + self.f]
    }

    /// `self` after `inner`.
    fn then(&self, inner: &Affine) -> Affine {
        Affine {
            a: self.a * inner.a + self.c * inner.b,
            b: self.b * inner.a + self.d * inner.b,
            c: self.a * inner.c + self.c * inner.d,
            d: self.b * inner.c + self.d * inner.d,
            e: self.a * inner.e + self.c * inner.f + self.e,
            f: self.b * inner.e + self.d * inner.f + self.f,
        }
    }

    fn scale(&self) -> f32 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }

    fn paint(&self, paint: &Paint) -> Paint {
        match *paint {
            Paint::Solid(c) => Paint::Solid(c),
            Paint::Linear { from, to, x1, y1, x2, y2 } => {
                let (p, q) = (self.apply([x1, y1]), self.apply([x2, y2]));
                Paint::Linear { from, to, x1: p[0], y1: p[1], x2: q[0], y2: q[1] }
            }
            Paint::Radial { inner, outer, cx, cy, r } => {
                let p = self.apply([cx, cy]);
                Paint::Radial { inner, outer, cx: p[0], cy: p[1], r: r * self.scale() }
            }
        }
    }
}

fn signed_area(poly: &[P]) -> f32 {
    let n = poly.len();
    (0..n).map(|i| poly[i][0] * poly[(i + 1) % n][1] - poly[(i + 1) % n][0] * poly[i][1]).sum::<f32>() / 2.0
}

/// Every piece of one fill must wind the same way so the union has no holes.
fn wind(mut poly: Vec<P>) -> Vec<P> {
    if signed_area(&poly) < 0.0 {
        poly.reverse();
    }
    poly
}

fn disc(c: P, r: f32) -> Vec<P> {
    let n = if r > 3.0 { 22 } else { 14 };
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            [c[0] + r * t.cos(), c[1] + r * t.sin()]
        })
        .collect()
}

/// The outline of a line `width` wide with round ends and joins, as pieces
/// that overlap into one shape.
fn outline(line: &[P], closed: bool, width: f32, out: &mut Vec<Vec<P>>) {
    let r = width / 2.0;
    if line.is_empty() || r <= 0.0 {
        return;
    }
    let n = line.len();
    let segments = if closed && n > 2 { n } else { n - 1 };
    let mut normals: Vec<Option<P>> = Vec::with_capacity(segments);
    for i in 0..segments {
        let (p, q) = (line[i], line[(i + 1) % n]);
        let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-5 {
            normals.push(None);
            continue;
        }
        let nx = [-dy / len * r, dx / len * r];
        normals.push(Some(nx));
        out.push(wind(vec![
            [p[0] + nx[0], p[1] + nx[1]],
            [q[0] + nx[0], q[1] + nx[1]],
            [q[0] - nx[0], q[1] - nx[1]],
            [p[0] - nx[0], p[1] - nx[1]],
        ]));
    }
    for (i, &v) in line.iter().enumerate() {
        let before = if i > 0 {
            normals.get(i - 1).copied().flatten()
        } else if closed && segments == n {
            normals[n - 1]
        } else {
            None
        };
        let after = normals.get(i).copied().flatten();
        match (before, after) {
            // A gentle bend only needs the two small wedges between the segments.
            (Some(a), Some(b)) if (a[0] * b[0] + a[1] * b[1]) > 0.94 * r * r => {
                for s in [1.0f32, -1.0] {
                    let tri = vec![v, [v[0] + a[0] * s, v[1] + a[1] * s], [v[0] + b[0] * s, v[1] + b[1] * s]];
                    if signed_area(&tri).abs() > 1e-7 {
                        out.push(wind(tri));
                    }
                }
            }
            _ => out.push(wind(disc(v, r))),
        }
    }
}

fn quad(p0: P, c: P, p1: P, out: &mut Vec<P>) {
    const N: usize = 14;
    for i in 1..=N {
        let t = i as f32 / N as f32;
        let u = 1.0 - t;
        out.push([
            u * u * p0[0] + 2.0 * u * t * c[0] + t * t * p1[0],
            u * u * p0[1] + 2.0 * u * t * c[1] + t * t * p1[1],
        ]);
    }
}

fn cubic(p0: P, c1: P, c2: P, p1: P, out: &mut Vec<P>) {
    const N: usize = 20;
    for i in 1..=N {
        let t = i as f32 / N as f32;
        let u = 1.0 - t;
        let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        out.push([
            w0 * p0[0] + w1 * c1[0] + w2 * c2[0] + w3 * p1[0],
            w0 * p0[1] + w1 * c1[1] + w2 * c2[1] + w3 * p1[1],
        ]);
    }
}

/// A smooth curve through every point (Catmull-Rom).
fn spline(points: &[P], closed: bool) -> Vec<P> {
    const N: usize = 10;
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    let at = |i: isize| -> P {
        if closed {
            points[i.rem_euclid(n as isize) as usize]
        } else {
            points[i.clamp(0, n as isize - 1) as usize]
        }
    };
    let spans = if closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(spans * N + 1);
    if !closed {
        out.push(points[0]);
    }
    for s in 0..spans as isize {
        let (p0, p1, p2, p3) = (at(s - 1), at(s), at(s + 1), at(s + 2));
        for i in 1..=N {
            let t = i as f32 / N as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |k: usize| {
                0.5 * (2.0 * p1[k]
                    + (-p0[k] + p2[k]) * t
                    + (2.0 * p0[k] - 5.0 * p1[k] + 4.0 * p2[k] - p3[k]) * t2
                    + (-p0[k] + 3.0 * p1[k] - 3.0 * p2[k] + p3[k]) * t3)
            };
            out.push([f(0), f(1)]);
        }
    }
    out
}

fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32, deg: f32, from: f32, to: f32) -> Vec<P> {
    let span = (to - from).abs().min(360.0);
    let full = span >= 359.99;
    let steps = ((span / 6.0).ceil() as usize).max(6);
    let (sin, cos) = deg.to_radians().sin_cos();
    let count = if full { steps } else { steps + 1 };
    (0..count)
        .map(|i| {
            let t = (from + (to - from) * i as f32 / steps as f32).to_radians();
            let (x, y) = (rx * t.cos(), ry * t.sin());
            [cx + x * cos - y * sin, cy + x * sin + y * cos]
        })
        .collect()
}

fn round_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<P> {
    let r = r.max(0.0).min(w / 2.0).min(h / 2.0);
    if r < 1e-3 {
        return vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
    }
    let mut out = Vec::new();
    let corners = [(x + w - r, y + r, 270.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0), (x + r, y + r, 180.0)];
    for (cx, cy, from) in corners {
        out.extend(ellipse(cx, cy, r, r, 0.0, from, from + 90.0));
    }
    out
}

struct Sub {
    points: Vec<P>,
    closed: bool,
}

struct Builder<'l> {
    lib: &'l mut Library,
    /// Read-only art a member's sticker may draw on. Its own source adds nothing to it.
    base: Option<&'l Library>,
    /// A sticker from a member: one sticker, no parts, a smaller budget.
    member: bool,
    category: String,
    cur: Option<Glyph>,
    fill: Option<Paint>,
    stroke: Option<(Color, f32)>,
    xf: Affine,
    points: usize,
    added: usize,
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 32 && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn num(tok: &str) -> Result<f32, String> {
    match tok.parse::<f32>() {
        Ok(v) if v.is_finite() && v.abs() <= MAX_NUMBER => Ok(v),
        _ => Err(format!("'{tok}' is not a usable number")),
    }
}

fn nums(toks: &[&str]) -> Result<Vec<f32>, String> {
    toks.iter().map(|t| num(t)).collect()
}

fn pairs(toks: &[&str], least: usize) -> Result<Vec<P>, String> {
    let v = nums(toks)?;
    if v.len() % 2 != 0 || v.len() / 2 < least {
        return Err(format!("expected at least {least} x y pairs"));
    }
    Ok(v.chunks(2).map(|c| [c[0], c[1]]).collect())
}

impl Builder<'_> {
    fn color(&self, tok: &str) -> Result<Color, String> {
        if let Some(name) = tok.strip_prefix('$') {
            return self
                .lib
                .palette()
                .get(name)
                .or_else(|| self.base.and_then(|b| b.palette().get(name)))
                .copied()
                .ok_or_else(|| format!("unknown colour ${name}"));
        }
        let hex = tok.strip_prefix('#').ok_or_else(|| format!("'{tok}' is not a colour"))?;
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("'{tok}' is not a colour"));
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
        let nib = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).unwrap_or(0) * 17;
        match hex.len() {
            3 => Ok(Color::rgb(nib(0), nib(1), nib(2))),
            6 => Ok(Color::rgb(byte(0), byte(2), byte(4))),
            8 => Ok(Color { a: byte(6), ..Color::rgb(byte(0), byte(2), byte(4)) }),
            _ => Err(format!("'{tok}' is not a colour")),
        }
    }

    fn paint(&self, toks: &[&str]) -> Result<Paint, String> {
        match toks {
            [c] => Ok(Paint::Solid(self.color(c)?)),
            ["lin", a, b, rest @ ..] if rest.len() == 4 => {
                let v = nums(rest)?;
                Ok(Paint::Linear { from: self.color(a)?, to: self.color(b)?, x1: v[0], y1: v[1], x2: v[2], y2: v[3] })
            }
            ["rad", a, b, rest @ ..] if rest.len() == 3 => {
                let v = nums(rest)?;
                Ok(Paint::Radial { inner: self.color(a)?, outer: self.color(b)?, cx: v[0], cy: v[1], r: v[2] })
            }
            _ => Err("expected a colour, 'lin a b x1 y1 x2 y2' or 'rad a b cx cy r'".into()),
        }
    }

    fn placement(toks: &[&str]) -> Result<Affine, String> {
        let (toks, flip) = match toks.split_last() {
            Some((&"flip", rest)) => (rest, true),
            _ => (toks, false),
        };
        if toks.len() > 4 {
            return Err("expected: dx dy [scale [degrees]] [flip]".into());
        }
        let v = nums(toks)?;
        let get = |i: usize, d: f32| v.get(i).copied().unwrap_or(d);
        let scale = get(2, 1.0);
        if !(0.01..=16.0).contains(&scale) {
            return Err("scale must be between 0.01 and 16".into());
        }
        Ok(Affine::about_center(get(0, 0.0), get(1, 0.0), scale, get(3, 0.0), flip))
    }

    fn push_shape(&mut self, paint: Paint, contours: Vec<Vec<P>>) -> Result<(), String> {
        let glyph = self.cur.as_mut().ok_or("a shape must come after 'emoji', 'sticker' or 'part'")?;
        let contours: Vec<Vec<P>> = contours.into_iter().filter(|c| c.len() >= 3).collect();
        if contours.is_empty() {
            return Ok(());
        }
        if contours.iter().flatten().flatten().any(|v| !v.is_finite() || v.abs() > MAX_COORD) {
            return Err(format!("'{}' has a point far outside the canvas", glyph.name));
        }
        let (max_shapes, max_points) =
            if self.member { (MAX_STICKER_SHAPES, MAX_STICKER_POINTS) } else { (MAX_SHAPES, MAX_POINTS) };
        self.points += contours.iter().map(Vec::len).sum::<usize>();
        if glyph.shapes.len() >= max_shapes || self.points > max_points {
            return Err(format!("'{}' is too detailed", glyph.name));
        }
        glyph.shapes.push(Shape { paint, contours });
        Ok(())
    }

    /// Fill and stroke the given sub-paths with the current style.
    fn emit(&mut self, subs: &[Sub], fillable: bool, xf: &Affine) -> Result<(), String> {
        let place = |c: &Vec<P>| c.iter().map(|p| xf.apply(*p)).collect::<Vec<P>>();
        if let (true, Some(fill)) = (fillable, self.fill) {
            let contours = subs.iter().filter(|s| s.points.len() >= 3).map(|s| place(&s.points)).collect();
            self.push_shape(xf.paint(&fill), contours)?;
        }
        if let Some((color, width)) = self.stroke {
            let mut pieces = Vec::new();
            for s in subs {
                outline(&s.points, s.closed, width, &mut pieces);
            }
            let contours = pieces.iter().map(place).collect();
            self.push_shape(Paint::Solid(color), contours)?;
        } else if !fillable {
            return Err("an open line needs 'stroke colour width' first".into());
        }
        Ok(())
    }

    fn path(toks: &[&str]) -> Result<Vec<Sub>, String> {
        let mut subs: Vec<Sub> = Vec::new();
        let mut mode = ' ';
        let mut pending: Vec<f32> = Vec::new();
        for tok in toks {
            if tok.len() == 1 && tok.as_bytes()[0].is_ascii_alphabetic() {
                if !pending.is_empty() {
                    return Err(format!("path: not enough numbers before '{tok}'"));
                }
                let cmd = tok.as_bytes()[0].to_ascii_uppercase() as char;
                match cmd {
                    'M' | 'L' | 'Q' | 'C' => mode = cmd,
                    'Z' => {
                        if let Some(s) = subs.last_mut() {
                            s.closed = true;
                        }
                        mode = ' ';
                    }
                    _ => return Err(format!("path: unknown command '{tok}'")),
                }
                continue;
            }
            pending.push(num(tok)?);
            let need = match mode {
                'M' | 'L' => 2,
                'Q' => 4,
                'C' => 6,
                _ => return Err("path: expected M, L, Q, C or Z".into()),
            };
            if pending.len() < need {
                continue;
            }
            let v = std::mem::take(&mut pending);
            if mode == 'M' {
                subs.push(Sub { points: vec![[v[0], v[1]]], closed: false });
                mode = 'L';
                continue;
            }
            let sub = subs.last_mut().filter(|s| !s.closed).ok_or("path: start with M x y")?;
            let last = *sub.points.last().ok_or("path: start with M x y")?;
            match mode {
                'L' => sub.points.push([v[0], v[1]]),
                'Q' => quad(last, [v[0], v[1]], [v[2], v[3]], &mut sub.points),
                _ => cubic(last, [v[0], v[1]], [v[2], v[3]], [v[4], v[5]], &mut sub.points),
            }
        }
        if !pending.is_empty() {
            return Err("path: not enough numbers at the end".into());
        }
        Ok(subs)
    }

    fn draw(&mut self, cmd: &str, args: &[&str], xf: &Affine) -> Result<(), String> {
        let one = |points: Vec<P>, closed: bool| vec![Sub { points, closed }];
        match cmd {
            "circle" => {
                let v = nums(args)?;
                if v.len() != 3 {
                    return Err("circle: cx cy r".into());
                }
                self.emit(&one(ellipse(v[0], v[1], v[2], v[2], 0.0, 0.0, 360.0), true), true, xf)
            }
            "ellipse" => {
                let v = nums(args)?;
                if v.len() != 4 && v.len() != 5 {
                    return Err("ellipse: cx cy rx ry [degrees]".into());
                }
                let deg = v.get(4).copied().unwrap_or(0.0);
                self.emit(&one(ellipse(v[0], v[1], v[2], v[3], deg, 0.0, 360.0), true), true, xf)
            }
            "arc" => {
                let v = nums(args)?;
                if v.len() != 6 {
                    return Err("arc: cx cy rx ry from to".into());
                }
                self.emit(&one(ellipse(v[0], v[1], v[2], v[3], 0.0, v[4], v[5]), false), false, xf)
            }
            "rect" => {
                let v = nums(args)?;
                if v.len() != 4 && v.len() != 5 {
                    return Err("rect: x y w h [radius]".into());
                }
                let r = v.get(4).copied().unwrap_or(0.0);
                self.emit(&one(round_rect(v[0], v[1], v[2], v[3], r), true), true, xf)
            }
            "poly" => self.emit(&one(pairs(args, 3)?, true), true, xf),
            "blob" => self.emit(&one(spline(&pairs(args, 3)?, true), true), true, xf),
            "line" => self.emit(&one(pairs(args, 2)?, false), false, xf),
            "curve" => self.emit(&one(spline(&pairs(args, 2)?, false), false), false, xf),
            "path" => {
                let subs = Self::path(args)?;
                self.emit(&subs, true, xf)
            }
            "text" => {
                if args.len() != 4 && args.len() != 5 {
                    return Err("text: cx baseline height WORDS [weight]".into());
                }
                let v = nums(&args[..3])?;
                let weight = match args.get(4) {
                    Some(w) => num(w)?,
                    None => v[2] * 0.17,
                };
                let paint = self.fill.ok_or("text needs a fill")?;
                let words = args[3].replace('_', " ");
                if words.chars().count() > 24 {
                    return Err("text: at most 24 characters".into());
                }
                let mut pieces = Vec::new();
                for line in font::layout(&words, v[0], v[1], v[2]) {
                    outline(&line, false, weight, &mut pieces);
                }
                let contours = pieces.iter().map(|c| c.iter().map(|p| xf.apply(*p)).collect()).collect();
                self.push_shape(xf.paint(&paint), contours)
            }
            "use" => {
                let (name, rest) = args.split_first().ok_or("use: name [dx dy [scale [degrees]]] [flip]")?;
                let at = xf.then(&Self::placement(rest)?);
                // Parts live in their own namespace, so a part and an emoji can share a name.
                let part = format!("~{name}");
                let base = self.base;
                let source = self
                    .lib
                    .by_name(&part)
                    .or_else(|| self.lib.by_name(name))
                    .or_else(|| base.and_then(|b| b.by_name(&part).or_else(|| b.by_name(name))))
                    .ok_or_else(|| format!("use: '{name}' is not defined yet"))?;
                // Refuse before copying, so a small source cannot make the reader do a lot of work.
                let cost: usize = source.shapes.iter().flat_map(|s| s.contours.iter()).map(Vec::len).sum();
                let max_points = if self.member { MAX_STICKER_POINTS } else { MAX_POINTS };
                if self.points + cost > max_points {
                    return Err(format!("use: '{name}' makes this glyph too detailed"));
                }
                let shapes: Vec<(Paint, Vec<Vec<P>>)> = source
                    .shapes
                    .iter()
                    .map(|s| (at.paint(&s.paint), s.contours.iter().map(|c| c.iter().map(|p| at.apply(*p)).collect()).collect()))
                    .collect();
                for (paint, contours) in shapes {
                    self.push_shape(paint, contours)?;
                }
                Ok(())
            }
            _ => Err(format!("unknown command '{cmd}'")),
        }
    }

    fn finish(&mut self) -> Result<(), String> {
        let Some(mut glyph) = self.cur.take() else { return Ok(()) };
        if glyph.shapes.is_empty() {
            return Err(format!("'{}' has no shapes", glyph.name));
        }
        if self.base.is_some_and(|b| b.by_name(&glyph.name).is_some()) {
            return Err(format!("'{}' is already defined", glyph.name));
        }
        let skin = |c: &Color| c.skin != 0;
        glyph.tones = glyph.shapes.iter().any(|s| match &s.paint {
            Paint::Solid(c) => skin(c),
            Paint::Linear { from, to, .. } => skin(from) || skin(to),
            Paint::Radial { inner, outer, .. } => skin(inner) || skin(outer),
        });
        self.lib.push(glyph)?;
        self.added += 1;
        Ok(())
    }

    fn begin(&mut self, kind: Kind, args: &[&str]) -> Result<(), String> {
        self.finish()?;
        if self.member && (kind != Kind::Sticker || self.added > 0) {
            return Err("a sticker file holds exactly one sticker and nothing else".into());
        }
        let (name, rest) = args.split_first().ok_or("expected a name")?;
        if !valid_name(name) {
            return Err(format!("'{name}' is not a usable name (a-z, 0-9 and _, up to 32)"));
        }
        let mut chars = String::new();
        let mut keywords = Vec::new();
        let mut in_keywords = false;
        for tok in rest {
            if *tok == "|" {
                in_keywords = true;
            } else if in_keywords {
                if keywords.len() < MAX_KEYWORDS {
                    keywords.push(tok.to_lowercase());
                }
            } else if let (Kind::Emoji, Some(hex)) = (kind, tok.strip_prefix("U+")) {
                for part in hex.split('-') {
                    let c = u32::from_str_radix(part, 16).ok().and_then(char::from_u32);
                    match c {
                        Some('\u{FE0F}') => {}
                        Some(c) => chars.push(c),
                        None => return Err(format!("'{tok}' is not a code point")),
                    }
                }
                // Plain text must never turn into a picture.
                if chars.chars().next().is_some_and(|c| (c as u32) < 0xA9) || chars.chars().count() > 6 {
                    return Err(format!("'{tok}' cannot be used as an emoji"));
                }
            } else {
                return Err(format!("unexpected '{tok}' (keywords go after '|')"));
            }
        }
        self.cur = Some(Glyph {
            name: if kind == Kind::Part { format!("~{name}") } else { name.to_string() },
            kind,
            chars,
            category: self.category.clone(),
            keywords,
            tones: false,
            shapes: Vec::new(),
        });
        self.fill = Some(Paint::Solid(Color::rgb(0, 0, 0)));
        self.stroke = None;
        self.xf = Affine::IDENTITY;
        self.points = 0;
        Ok(())
    }

    fn line(&mut self, line: &str) -> Result<(), String> {
        if line.len() > MAX_LINE_BYTES {
            return Err("line is too long".into());
        }
        let line = line.split("//").next().unwrap_or("");
        let toks: Vec<&str> = line.split_whitespace().collect();
        let Some((&cmd, args)) = toks.split_first() else { return Ok(()) };
        match cmd {
            "category" => {
                self.finish()?;
                let [name] = args else { return Err("category: one word".into()) };
                if !valid_name(name) {
                    return Err(format!("'{name}' is not a usable category"));
                }
                self.category = name.to_string();
                Ok(())
            }
            "color" => {
                let [name, value] = args else { return Err("color: name value".into()) };
                if !valid_name(name) || name.starts_with("skin") {
                    return Err(format!("'{name}' is not a usable colour name"));
                }
                let c = self.color(value)?;
                self.lib.palette_mut().insert(name.to_string(), c);
                Ok(())
            }
            "part" => self.begin(Kind::Part, args),
            "emoji" => self.begin(Kind::Emoji, args),
            "sticker" => self.begin(Kind::Sticker, args),
            "fill" => {
                self.fill = Some(self.paint(args)?);
                Ok(())
            }
            "nofill" => {
                self.fill = None;
                Ok(())
            }
            "stroke" => {
                let [color, width] = args else { return Err("stroke: colour width".into()) };
                let w = num(width)?;
                if !(0.05..=64.0).contains(&w) {
                    return Err("stroke width must be between 0.05 and 64".into());
                }
                self.stroke = Some((self.color(color)?, w));
                Ok(())
            }
            "nostroke" => {
                self.stroke = None;
                Ok(())
            }
            "xf" => {
                self.xf = if args.is_empty() { Affine::IDENTITY } else { Self::placement(args)? };
                Ok(())
            }
            "mirror" => {
                let (&inner, rest) = args.split_first().ok_or("mirror: put a shape after it")?;
                let xf = self.xf;
                self.draw(inner, rest, &xf)?;
                let flipped = xf.then(&Affine::about_center(0.0, 0.0, 1.0, 0.0, true));
                self.draw(inner, rest, &flipped)
            }
            _ => {
                let xf = self.xf;
                self.draw(cmd, args, &xf)
            }
        }
    }
}

/// Add every glyph of `src` to `lib`. Returns how many were added.
pub(crate) fn parse_sheet(lib: &mut Library, src: &str, origin: &str) -> Result<usize, ArtError> {
    parse_with(lib, None, false, src, origin)
}

/// Read one member sticker into `scratch`, drawing on `base` without copying or changing it.
pub(crate) fn parse_member_sticker(scratch: &mut Library, base: &Library, src: &str) -> Result<usize, ArtError> {
    parse_with(scratch, Some(base), true, src, "sticker")
}

fn parse_with(lib: &mut Library, base: Option<&Library>, member: bool, src: &str, origin: &str) -> Result<usize, ArtError> {
    let mut b = Builder {
        lib,
        base,
        member,
        category: "misc".into(),
        cur: None,
        fill: None,
        stroke: None,
        xf: Affine::IDENTITY,
        points: 0,
        added: 0,
    };
    let fail = |line: usize, message: String| ArtError { origin: origin.to_string(), line, message };
    let mut last = 0;
    for (i, line) in src.lines().enumerate() {
        last = i + 1;
        b.line(line).map_err(|m| fail(last, m))?;
    }
    b.finish().map_err(|m| fail(last, m))?;
    Ok(b.added)
}
