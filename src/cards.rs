//! Native card painter: indexed vector faces, mirrored engraved courts and themed backs.
use crate::{
    game::Card,
    theme::{blend, Palette, FACE, INK},
};
use eframe::egui::{pos2, vec2, Color32, Painter, Rect, Stroke, StrokeKind, TextureHandle};

pub struct DeckStyle<'a> {
    pub palette: &'a Palette,
    pub pattern: usize,
    pub holographic: bool,
    pub sheen: f32,
    pub deck: &'a crate::deck::DeckArt,
    pub art: Option<&'a TextureHandle>,
}
pub fn paint(p: &Painter, r: Rect, c: Card, style: &DeckStyle<'_>, selected: bool) {
    let uv = Rect::from_min_max(pos2(0., 0.), pos2(1., 1.));
    p.rect_filled(r.translate(vec2(1., 2.)), 4, Color32::from_black_alpha(48));
    p.rect_filled(r, 4, FACE);
    if c.1 {
        p.image(
            style.deck.faces[c.0 as usize].id(),
            r.shrink(1.),
            uv,
            Color32::WHITE,
        );
        if c.rank() > 10 {
            // Each cell is exactly one third of the atlas. Mirror the top bust in
            // code so both ends are identical, including at the centre seam.
            let n = (c.rank() - 11) as f32;
            let left = (n * 512. + 4.) / 1536.;
            let right = ((n + 1.) * 512. - 4.) / 1536.;
            let body = Rect::from_min_max(
                r.min + vec2(r.width() * 0.20, r.width() * 0.255),
                r.max - vec2(r.width() * 0.20, r.width() * 0.255),
            );
            let top = Rect::from_min_max(body.min, pos2(body.right(), body.center().y));
            let bottom = Rect::from_min_max(pos2(body.left(), body.center().y), body.max);
            p.image(
                style.deck.courts.id(),
                top,
                Rect::from_min_max(pos2(left, 0.), pos2(right, 0.5)),
                Color32::WHITE,
            );
            p.image(
                style.deck.courts.id(),
                bottom,
                Rect::from_min_max(pos2(right, 0.5), pos2(left, 0.)),
                Color32::WHITE,
            );
            let gold = Color32::from_rgb(172, 151, 100);
            p.line_segment(
                [body.left_center(), body.right_center()],
                Stroke::new(0.7_f32, gold),
            );
        }
    } else {
        let inset = r.shrink(5.);
        if let Some(art) = style.art {
            p.rect_filled(inset, 2, style.palette.back);
            let size = art.size_vec2();
            let scale = (inset.width() / size.x).min(inset.height() / size.y);
            p.image(
                art.id(),
                Rect::from_center_size(inset.center(), size * scale),
                uv,
                Color32::WHITE,
            );
        } else {
            p.image(
                style.deck.backs[style.pattern.min(2)].id(),
                r.shrink(1.),
                uv,
                Color32::WHITE,
            );
        }
        if style.holographic {
            let clip = p.with_clip_rect(inset);
            let x = inset.left() + style.sheen * inset.width() * 1.6 - inset.width() * 0.3;
            for (offset, color) in [
                (0., Color32::from_rgba_unmultiplied(110, 220, 240, 32)),
                (6., Color32::from_rgba_unmultiplied(230, 140, 210, 28)),
                (12., Color32::from_rgba_unmultiplied(230, 225, 160, 28)),
            ] {
                clip.line_segment(
                    [
                        pos2(x + offset, inset.top()),
                        pos2(x + offset - inset.width() * 0.5, inset.bottom()),
                    ],
                    Stroke::new(r.width() * 0.12, color),
                );
            }
        }
    }
    p.rect_stroke(
        r,
        4,
        Stroke::new(
            if selected { 2.5_f32 } else { 0.8_f32 },
            if selected {
                style.palette.accent
            } else {
                blend(INK, FACE, 0.65)
            },
        ),
        StrokeKind::Inside,
    );
}
