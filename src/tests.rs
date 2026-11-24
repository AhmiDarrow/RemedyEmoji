use super::*;

fn lib() -> &'static Library {
    builtin()
}

#[test]
fn builtin_art_parses() {
    let mut fresh = Library::new();
    for (name, src) in builtin_sheets() {
        if let Err(e) = fresh.add_sheet(src, name) {
            panic!("{e}");
        }
    }
    let emoji = fresh.glyphs().iter().filter(|g| g.kind == Kind::Emoji).count();
    let stickers = fresh.glyphs().iter().filter(|g| g.kind == Kind::Sticker).count();
    assert!(emoji >= 600, "only {emoji} emoji");
    assert!(stickers >= 40, "only {stickers} stickers");
    assert_eq!(fresh.glyphs().len(), lib().glyphs().len());
}

#[test]
fn every_glyph_draws_something() {
    for g in lib().glyphs().iter().filter(|g| g.kind != Kind::Part) {
        let img = render(g, 24, 0);
        assert_eq!(img.rgba.len(), 24 * 24 * 4);
        assert!(img.rgba.chunks(4).any(|p| p[3] > 0), "{} is empty", g.name);
    }
}

#[test]
fn hands_follow_the_skin_tone_and_faces_do_not() {
    let thumbs = lib().by_name("thumbsup").unwrap();
    assert!(thumbs.tones);
    assert_ne!(render(thumbs, 32, 0), render(thumbs, 32, 5));
    let grin = lib().by_name("grinning").unwrap();
    assert!(!grin.tones);
}

#[test]
fn text_splits_into_glyphs_and_plain_text() {
    let l = lib();
    let fire = l.index_of("fire").unwrap();
    let gg = l.index_of("gg").unwrap();
    let segs = l.segments("nice \u{1F525} :gg: :nothing_here: 12:30");
    assert_eq!(
        segs,
        vec![
            Seg::Text("nice "),
            Seg::Glyph { index: fire, tone: 0, src: "\u{1F525}" },
            Seg::Text(" "),
            Seg::Glyph { index: gg, tone: 0, src: ":gg:" },
            Seg::Text(" :nothing_here: 12:30"),
        ]
    );
    // Plain ASCII never turns into a picture, and a part is never reachable from chat.
    assert_eq!(l.segments("a:b #1 (c) *"), vec![Seg::Text("a:b #1 (c) *")]);
    assert_eq!(l.segments(":~face:"), vec![Seg::Text(":~face:")]);
    assert_eq!(l.segments(":face:"), vec![Seg::Text(":face:")]);
}

#[test]
fn variation_selector_and_skin_tone_are_consumed() {
    let l = lib();
    let heart = l.index_of("heart").unwrap();
    assert_eq!(l.segments("\u{2764}\u{FE0F}"), vec![Seg::Glyph { index: heart, tone: 0, src: "\u{2764}\u{FE0F}" }]);
    assert_eq!(l.segments("\u{2764}"), vec![Seg::Glyph { index: heart, tone: 0, src: "\u{2764}" }]);
    let thumbs = l.index_of("thumbsup").unwrap();
    assert_eq!(
        l.segments("\u{1F44D}\u{1F3FD}"),
        vec![Seg::Glyph { index: thumbs, tone: 3, src: "\u{1F44D}\u{1F3FD}" }]
    );
}

#[test]
fn wire_text_uses_unicode_where_it_exists() {
    let l = lib();
    assert_eq!(l.to_wire("go :fire: :gg:", 0), "go \u{1F525} :gg:");
    assert_eq!(l.to_wire(":thumbsup:", 4), "\u{1F44D}\u{1F3FE}");
    assert_eq!(l.to_wire(":grinning:", 4), "\u{1F600}");
    assert_eq!(l.to_shortcodes("go \u{1F525} :gg:"), "go :fire: :gg:");
    // Round trip through both directions leaves the wire form alone.
    let wire = l.to_wire("x :rocket: y :clutch: z", 0);
    assert_eq!(l.to_wire(&l.to_shortcodes(&wire), 0), wire);
}

#[test]
fn solo_lines_are_recognised() {
    let l = lib();
    assert_eq!(solo(l, &l.segments(":fire: :fire:")), Solo::Emoji);
    assert_eq!(solo(l, &l.segments("hot :fire:")), Solo::No);
    assert_eq!(solo(l, &l.segments(&":fire:".repeat(7))), Solo::No);
    let gg = l.index_of("good_game").unwrap();
    assert_eq!(solo(l, &l.segments(" :good_game: ")), Solo::Sticker(gg));
    assert_eq!(solo(l, &l.segments(":good_game: :fire:")), Solo::No);
}

#[test]
fn search_and_completion() {
    let l = lib();
    let hits = l.search(Kind::Emoji, "laugh");
    assert!(hits.iter().any(|i| l.get(*i).unwrap().name == "joy"));
    assert!(l.search(Kind::Sticker, "gg").iter().any(|i| l.get(*i).unwrap().name == "good_game"));
    let done = l.complete("roc", 8);
    assert_eq!(l.get(done[0]).unwrap().name, "rocket");
    assert!(l.complete("zzzzzz", 8).is_empty());
}

const STICKER: &str = "sticker my_cat | cat\n  fill #ffaa33\n  stroke #7a4a10 2\n  circle 32 34 22\n  use eyes\n  fill #222222\n  text 32 58 8 MEOW\n";

#[test]
fn a_sticker_from_another_member_parses_within_bounds() {
    let g = lib().parse_sticker(STICKER).unwrap();
    assert_eq!(g.name, "my_cat");
    assert_eq!(g.kind, Kind::Sticker);
    let img = render(&g, 96, 0);
    assert!(img.rgba.chunks(4).any(|p| p[3] > 0));
    // The library itself is not changed by a sticker that passes through.
    assert!(lib().by_name("my_cat").is_none());
}

#[test]
fn hostile_stickers_are_refused() {
    let l = lib();
    let bad = [
        "".to_string(),
        "emoji not_a_sticker\n circle 32 32 10\n".to_string(),
        "sticker a\n circle 32 32 10\nsticker b\n circle 32 32 10\n".to_string(),
        "sticker ../etc\n circle 32 32 10\n".to_string(),
        "sticker x\n".to_string(),
        "sticker x\n circle NaN 32 10\n".to_string(),
        "sticker x\n circle 1e30 32 10\n".to_string(),
        "sticker x\n use no_such_thing\n".to_string(),
        "sticker x\n include /etc/passwd\n".to_string(),
        "sticker x\n fill url(http://example.invalid/a.png)\n circle 32 32 10\n".to_string(),
        "sticker x\n stroke #000000 9999\n circle 32 32 10\n".to_string(),
        "sticker x\n use face 0 0 99\n".to_string(),
        format!("sticker x\n circle 32 32 10\n// {}\n", "p".repeat(MAX_STICKER_SOURCE_BYTES)),
        format!("sticker x\n{}", " use face\n".repeat(700)),
        format!("sticker x\n stroke #000000 2\n{}", " circle 32 32 20\n".repeat(600)),
        format!("sticker x\n poly {}\n", "1 2 ".repeat(3000)),
    ];
    for src in &bad {
        assert!(l.parse_sticker(src).is_err(), "accepted: {:.60}", src);
    }
}

#[test]
fn a_member_sticker_cannot_define_parts_or_blow_up() {
    let l = lib();
    // Parts multiplied a small source into a huge amount of work.
    let mut src = String::from("part a\n nofill\n stroke #000 1\n blob");
    for _ in 0..200 {
        src.push_str(" 0 0");
    }
    src.push('\n');
    for i in 0..100 {
        src.push_str(&format!("part b{i}\n use a\n"));
    }
    src.push_str("sticker z\n circle 32 32 9\n");
    assert!(src.len() <= MAX_STICKER_SOURCE_BYTES);
    assert!(l.parse_sticker(&src).is_err());
    assert!(l.parse_sticker("part p\n circle 1 1 1\nsticker s\n use p\n").is_err());

    // Nested scaling used to push points to infinity and crash the rasterizer on some CPUs.
    assert!(l.parse_sticker("sticker s\n xf 0 0 16\n poly 32 31 10000 31 10000 33 32 33\n").is_err());
    // Far off the canvas but finite is only an empty picture, not an error.
    assert!(l.parse_sticker("sticker s\n use face 9999 9999 16\n").is_ok());

    // Reusing big built-in glyphs is counted before it is copied.
    let greedy = format!("sticker s\n{}", " use good_game\n".repeat(200));
    assert!(l.parse_sticker(&greedy).is_err());

    // The built-in pack still fits a member's budget, so anyone can remix it.
    let ok = l.parse_sticker("sticker remix\n use grinning 0 -6 0.8\n fill #ffffff\n text 32 60 9 HI\n").unwrap();
    assert_eq!(ok.name, "remix");
}

#[test]
fn the_rasterizer_survives_non_finite_geometry() {
    // Built by hand, past the parser, as a second line of defence.
    let bad = Glyph {
        name: "bad".into(),
        kind: Kind::Emoji,
        chars: String::new(),
        category: "misc".into(),
        keywords: Vec::new(),
        tones: false,
        shapes: vec![Shape {
            paint: Paint::Solid(Color::rgb(255, 0, 0)),
            contours: vec![vec![
                [32.0, -1.0e30],
                [f32::INFINITY, -1.0e30],
                [f32::INFINITY, 1.0e30],
                [32.0, 1.0e30],
                [f32::NAN, 10.0],
            ]],
        }],
    };
    for px in [1, 22, 64, 257] {
        let img = render(&bad, px, 0);
        assert_eq!(img.rgba.len(), (px * px * 4) as usize);
    }
}

#[test]
fn a_bad_sheet_leaves_the_library_untouched() {
    let mut l = Library::new();
    l.add_sheet("emoji ok_one\n circle 32 32 10\n", "a").unwrap();
    let err = l.add_sheet("emoji fine\n circle 32 32 10\nemoji broken\n circle x\n", "b").unwrap_err();
    assert_eq!(err.line, 4);
    assert!(l.by_name("fine").is_none());
    assert!(l.by_name("ok_one").is_some());
    // Names cannot be reused, and ASCII can never be claimed as an emoji.
    assert!(l.add_sheet("emoji ok_one\n circle 32 32 10\n", "c").is_err());
    assert!(l.add_sheet("emoji hash U+23\n circle 32 32 10\n", "d").is_err());
}

#[test]
fn rendering_is_clamped() {
    let g = lib().by_name("grinning").unwrap();
    assert_eq!(render(g, 0, 0).width, 1);
    assert_eq!(render(g, 100_000, 0).width, MAX_RENDER_PX);
    // A filled circle covers its centre and leaves the corners clear.
    let img = render(g, 64, 0);
    let px = |x: usize, y: usize| img.rgba[(y * 64 + x) * 4 + 3];
    assert_eq!(px(32, 12), 255);
    assert_eq!(px(1, 1), 0);
}

#[test]
fn stickers_get_a_border_and_emoji_do_not() {
    let s = lib().by_name("good_game").unwrap();
    assert_ne!(render(s, 96, 0), render_plain(s, 96, 0));
    let e = lib().by_name("grinning").unwrap();
    assert_eq!(render(e, 96, 0), render_plain(e, 96, 0));
}
