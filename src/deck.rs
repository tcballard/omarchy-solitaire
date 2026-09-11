//! Embedded deck artwork. Vector faces and themed backs are rasterized once per palette,
//! never per frame. Court portraits use one shared atlas with explicit UV boundaries.
use crate::{
    game::Card,
    theme::{blend, Palette},
};
use eframe::egui::{self, Color32, Context, TextureHandle};
use std::{fmt::Write, sync::OnceLock};

const SUITS: [&str; 4] = [
    "M0 -20 C-21 -43 -39 -13 -20 -4 C-49 -13 -45 27 -20 20 L-5 12 Q-5 28 -15 34 L15 34 Q5 28 5 12 L20 20 C45 27 49 -13 20 -4 C39 -13 21 -43 0 -20Z",
    "M0 -36 L25 0 L0 36 L-25 0Z",
    "M0 31 C-8 17 -35 1 -35 -15 C-35 -38 -10 -41 0 -22 C10 -41 35 -38 35 -15 C35 1 8 17 0 31Z",
    "M0 -37 C-8 -22 -35 -5 -35 12 C-35 34 -9 33 -3 17 Q-3 29 -14 36 L14 36 Q3 29 3 17 C9 33 35 34 35 12 C35 -5 8 -22 0 -37Z",
];
fn color(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}
fn mark(suit: usize, x: f32, y: f32, scale: f32, fill: &str, inverted: bool) -> String {
    format!(
        r#"<path d="{}" fill="{fill}" transform="translate({x} {y}) scale({scale}) rotate({})"/>"#,
        SUITS[suit],
        if inverted { 180 } else { 0 }
    )
}
fn svg(body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="700" viewBox="0 0 500 700">{body}</svg>"#
    )
}
// A separate pip field leaves a quiet band under each corner index.
const PIP_TOP: f32 = 190.;
const PIP_BOTTOM: f32 = 510.;
const PIP_LEFT: f32 = 155.;
const PIP_RIGHT: f32 = 345.;
fn pips(rank: u8) -> Vec<(f32, f32)> {
    let (t, b, l, r) = (PIP_TOP, PIP_BOTTOM, PIP_LEFT, PIP_RIGHT);
    match rank {
        1 => vec![(250., 350.)],
        2 => vec![(250., t), (250., b)],
        3 => vec![(250., t), (250., 350.), (250., b)],
        n => {
            let mut p = vec![(l, t), (r, t), (l, b), (r, b)];
            if n == 5 {
                p.push((250., 350.));
            }
            if (6..=8).contains(&n) {
                p.extend([(l, 350.), (r, 350.)]);
                if n >= 7 {
                    p.push((250., 270.));
                }
                if n == 8 {
                    p.push((250., 430.));
                }
            }
            if n >= 9 {
                p.extend([(l, 296.667), (r, 296.667), (l, 403.333), (r, 403.333)]);
                if n == 9 {
                    p.push((250., 350.));
                } else {
                    p.extend([(250., 243.333), (250., 456.667)]);
                }
            }
            p
        }
    }
}
pub fn face_svg(c: Card) -> String {
    let ink = if c.red() { "#ab263d" } else { "#182a30" };
    let rank = include_str!("../assets/deck/ranks.svgparts")
        .lines()
        .nth(c.rank() as usize - 1)
        .expect("thirteen bundled ranks");
    let (width, rank) = rank.split_once('|').expect("measured rank outline");
    let width: f32 = width.parse().expect("rank ink width");
    // 24 units of clear space, measured between the actual ink edges.
    let suit_half_width = [43., 25., 35., 35.][c.suit() as usize] * 0.72;
    let index = format!(
        r#"<g transform="translate(32 28)" fill="{ink}">{rank}</g>{}"#,
        mark(
            c.suit() as usize,
            32. + width + 24. + suit_half_width,
            67.,
            0.72,
            ink,
            false
        )
    );
    let mut b = format!(
        r##"<rect width="500" height="700" rx="18" fill="#faf7ef"/>{index}<g transform="rotate(180 250 350)">{index}</g>"##
    );
    if c.rank() <= 10 {
        for (x, y) in pips(c.rank()) {
            b.push_str(&mark(
                c.suit() as usize,
                x,
                y,
                if c.rank() == 1 {
                    2.45
                } else if c.rank() >= 9 {
                    0.95
                } else {
                    1.12
                },
                ink,
                y > 350.,
            ));
        }
        if c.rank() == 1 {
            // Engraved foliage within the large ace. The spade gets the ceremonial ace treatment.
            if c.suit() == 3 {
                b.push_str(r##"<g stroke="#c6b47e" fill="none" stroke-width="2.8"><path d="M250 412V302 M250 385Q203 370 216 341 M250 365Q286 350 278 324 M250 345Q225 335 235 312"/><path d="M250 407Q226 399 221 384 M250 390Q275 380 282 361"/></g>"##);
            }
            b.push_str(r##"<path d="M210 473H236 M264 473H290 M250 467L256 473L250 479L244 473Z" fill="none" stroke="#ac9764" stroke-width="2"/>"##);
        }
    }
    if c.rank() > 10 {
        b.push_str(&mark(c.suit() as usize, 55., 210., 0.65, ink, false));
        b.push_str(&mark(c.suit() as usize, 445., 490., 0.65, ink, true));
    }
    svg(&b)
}
// Acanthus sprig, drawn once and transformed into the border's opposing corners.
const SPRIG: &str = r#"<path d="M0 0C42 -26 40 -74 72 -115C94 -143 101 -174 92 -206" fill="none" stroke="currentColor" stroke-width="3"/>
<path d="M18 -18C-14 -20 -17 -61 -2 -69C-5 -46 23 -46 26 -31C19 -68 3 -91 16 -103C34 -93 33 -59 36 -50C36 -87 25 -120 40 -130C54 -115 46 -91 49 -78C50 -123 52 -156 68 -162C76 -142 59 -126 61 -111C70 -147 83 -178 94 -179C102 -156 78 -137 74 -124C101 -145 129 -145 128 -128C112 -131 91 -119 83 -105C109 -126 137 -117 132 -100C110 -112 88 -87 76 -83C112 -93 134 -73 124 -62C111 -82 77 -63 62 -63C84 -60 103 -48 95 -36C81 -52 58 -44 45 -42C62 -28 67 -9 54 -7C51 -25 35 -19 18 -18Z" fill="currentColor" opacity=".75"/>
<path d="M8 -49Q30 -39 27 -31 M27 -87L36 -50 M49 -130L49 -78 M93 -153L74 -124 M118 -105L76 -83 M108 -70L62 -63 M77 -39L45 -42" fill="none" stroke-width="1.4" stroke="var-ground"/>"#;

pub fn back_svg(palette: &Palette, pattern: usize) -> String {
    let mut ground_color = blend(palette.table, Color32::from_rgb(12, 24, 24), 0.38);
    let ink_color = blend(palette.accent, Color32::from_rgb(243, 224, 176), 0.55);
    if crate::theme::contrast(ground_color, ink_color) < 3.5 {
        ground_color = blend(ground_color, Color32::BLACK, 0.82);
    }
    let ground = color(ground_color);
    let ink = color(ink_color);
    let subdued = color(blend(palette.accent, palette.table, 0.46));
    let silver = color(blend(
        palette.accent,
        Color32::from_rgb(204, 217, 222),
        0.65,
    ));
    let rose = "#c39b86";
    let mut b = format!(
        r##"<rect width="500" height="700" rx="18" fill="#faf7ef"/><rect x="15" y="15" width="470" height="670" rx="12" fill="{ground}"/><g fill="none" stroke="{ink}"><rect x="25" y="25" width="450" height="650" rx="8" stroke-width="3"/><rect x="33" y="33" width="434" height="634" rx="5" stroke-width="1"/><rect x="40" y="40" width="420" height="620" rx="4" stroke-width="2" stroke-dasharray="1 5"/></g>"##
    );
    if pattern == 0 {
        // Architectural tilework with alternating suit tiles and pin-jointed lattice.
        b.push_str(r#"<defs><clipPath id="field"><rect x="52" y="52" width="396" height="596"/></clipPath></defs>"#);
        write!(
            b,
            r#"<g clip-path="url(#field)" stroke="{ink}" stroke-width="2">"#
        )
        .unwrap();
        for row in -1..10 {
            for col in -1..7 {
                let (x, y) = (70 + col * 72, 62 + row * 72);
                write!(b,r#"<path d="M{x} {}l36 36 -36 36 -36 -36Z" fill="none"/><circle cx="{x}" cy="{y}" r="3" fill="{ink}"/>"#,y-36).unwrap();
                b.push_str(&mark(
                    ((col + row + 20) % 4) as usize,
                    x as f32,
                    y as f32,
                    0.40,
                    if (col + row) % 3 == 0 { &ink } else { &subdued },
                    row >= 5,
                ));
            }
        }
        b.push_str("</g>");
        for (x, y) in [(53, 53), (395, 53), (53, 595), (395, 595)] {
            write!(b,r#"<g transform="translate({x} {y})"><rect width="52" height="52" fill="{ground}" stroke="{ink}" stroke-width="2"/><path d="M26 4L31 21L48 26L31 31L26 48L21 31L4 26L21 21Z" fill="{ink}"/><path d="M8 8L26 22L44 8L30 26L44 44L26 30L8 44L22 26Z" fill="{subdued}"/></g>"#).unwrap();
        }
        write!(b,r#"<path d="M250 172L411 350L250 528L89 350Z" fill="{ground}" stroke="{ink}" stroke-width="5"/><path d="M250 183L401 350L250 517L99 350Z" fill="none" stroke="{subdued}" stroke-width="2"/>"#).unwrap();
    } else {
        if pattern == 2 {
            // Radial foil engraving: silver and rose are inks; hover sheen is separate.
            for n in 0..72 {
                let a = n as f32 * std::f32::consts::TAU / 72.;
                let (s, c) = a.sin_cos();
                let (x1, y1) = (250. + s * 137., 350. + c * 180.);
                let (x2, y2) = (250. + s * 193., 350. + c * 285.);
                write!(
                    b,
                    r#"<path d="M{x1} {y1}L{x2} {y2}" stroke="{}" stroke-width="{}"/>"#,
                    if n % 3 == 0 { rose } else { &silver },
                    if n % 3 == 0 { 1.8 } else { 0.9 }
                )
                .unwrap();
            }
        }
        b.push_str(r#"<defs><clipPath id="ornament"><rect x="47" y="47" width="406" height="606"/></clipPath></defs><g clip-path="url(#ornament)">"#);
        for rotation in [0, 180] {
            for (x, scale) in [(62, 0.85), (438, -0.85)] {
                write!(b,r#"<g transform="rotate({rotation} 250 350) translate({x} 335) scale({scale} .95)" color="{}">{}</g>"#,if pattern==2 {&silver}else{&ink},SPRIG.replace("var-ground",&ground)).unwrap();
            }
        }
        for rotation in [0, 180] {
            write!(b,r#"<g transform="rotate({rotation} 250 350) translate(222 193) scale(.72)" color="{}">{}</g>"#,if pattern==2 {rose}else{&ink},SPRIG.replace("var-ground",&ground)).unwrap();
        }
        b.push_str("</g>");
        write!(b,r#"<ellipse cx="250" cy="350" rx="127" ry="165" fill="{ground}" stroke="{ink}" stroke-width="3"/><ellipse cx="250" cy="350" rx="119" ry="157" fill="none" stroke="{ink}" stroke-width="2" stroke-dasharray="1 5"/><ellipse cx="250" cy="350" rx="110" ry="148" fill="none" stroke="{}" stroke-width="2"/>"#,if pattern==2 {rose}else{&subdued}).unwrap();
        for (x, y, s) in [
            (85., 85., 3),
            (415., 85., 2),
            (415., 615., 3),
            (85., 615., 2),
        ] {
            write!(b,r#"<circle cx="{x}" cy="{y}" r="34" fill="{ground}" stroke="{ink}" stroke-width="2"/><circle cx="{x}" cy="{y}" r="28" fill="none" stroke="{subdued}" stroke-width="2"/>"#).unwrap();
            b.push_str(&mark(s, x, y, 0.5, &ink, y > 350.));
        }
    }
    // Exact official square mark, unchanged geometry and published green.
    let logo = include_str!("../assets/omarchy-logo.svg");
    let inner = logo
        .split_once('>')
        .unwrap()
        .1
        .rsplit_once("</svg>")
        .unwrap()
        .0;
    write!(
        b,
        r#"<g transform="translate(196 296) scale(.09)">{inner}</g>"#
    )
    .unwrap();
    svg(&b)
}
fn raster(svg: &str) -> egui::ColorImage {
    egui_extras::image::load_svg_bytes_with_size(
        svg.as_bytes(),
        Some(egui::load::SizeHint::Size(500, 700)),
    )
    .expect("bundled vector deck")
}
fn court_image() -> &'static egui::ColorImage {
    static COURTS: OnceLock<egui::ColorImage> = OnceLock::new();
    COURTS.get_or_init(|| {
        let i = image::load_from_memory_with_format(
            include_bytes!("../assets/deck/courts.png"),
            image::ImageFormat::Png,
        )
        .expect("bundled court atlas")
        .into_rgba8();
        egui::ColorImage::from_rgba_unmultiplied(
            [i.width() as usize, i.height() as usize],
            i.as_raw(),
        )
    })
}
pub struct DeckArt {
    pub faces: Vec<TextureHandle>,
    pub backs: [TextureHandle; 3],
    pub courts: TextureHandle,
}
impl DeckArt {
    pub fn new(ctx: &Context, palette: &Palette) -> Self {
        static FACES: OnceLock<Vec<egui::ColorImage>> = OnceLock::new();
        let faces = FACES.get_or_init(|| {
            (0..52)
                .map(|id| {
                    egui_extras::image::load_svg_bytes_with_size(
                        face_svg(Card(id, true)).as_bytes(),
                        Some(egui::load::SizeHint::Size(250, 350)),
                    )
                    .expect("bundled face")
                })
                .collect()
        });
        Self {
            faces: faces
                .iter()
                .enumerate()
                .map(|(n, i)| {
                    ctx.load_texture(
                        format!("card-{n}"),
                        i.clone(),
                        egui::TextureOptions::LINEAR
                            .with_mipmap_mode(Some(egui::TextureFilter::Linear)),
                    )
                })
                .collect(),
            backs: std::array::from_fn(|n| {
                ctx.load_texture(
                    format!("back-{n}"),
                    raster(&back_svg(palette, n)),
                    egui::TextureOptions::LINEAR
                        .with_mipmap_mode(Some(egui::TextureFilter::Linear)),
                )
            }),
            courts: ctx.load_texture(
                "court-atlas",
                court_image().clone(),
                egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear)),
            ),
        }
    }
    pub fn refresh_backs(&mut self, palette: &Palette) {
        for (n, t) in self.backs.iter_mut().enumerate() {
            t.set(
                raster(&back_svg(palette, n)),
                egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear)),
            );
        }
    }
}
