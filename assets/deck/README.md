# Engraved deck artwork

The production deck follows Tom's approved September 2026 study: warm ivory
faces, classical engraved courts, serif indexes, Tilework as the default back,
and Engraved / Foil alternatives. The reference sheet is an art-direction study;
production proportions and indexes are adapted for overlapping Klondike cards.

- `courts.png`: original illustrated Jack, Queen and King atlas, created with the
  built-in Image Generation tool from the approved study. Each 512 × 1024 cell
  is one rank. The renderer uses its upper 512 × 512 bust and rotates that same
  texture region for the lower half, guaranteeing identical ends. All four suits
  share the three portraits; suit identity comes from indexes and medallions.
- `ranks.svgparts`: outlined DejaVu Serif Bold glyphs, one measured fragment per rank. The
  font's license is in `FONT-LICENSE.txt`; no system font is required at runtime.
- Faces and the three theme-coloured SVG backs are authored in `src/deck.rs`.
  The official logo is embedded from `assets/omarchy-logo.svg` with its geometry
  and #9ece6a fill unchanged. Foil's silver/rose engraving is static; the optional
  Holographic finish supplies a separate hover sheen and honours Reduce motion.

Texture filtering uses mipmaps to keep engraving legible at 80–124 logical
pixels without shimmering. Faces are cached at 250 × 350, backs at 500 × 700,
and the atlas at 1536 × 1024. Theme changes regenerate only the three backs.
No generation service, network connection or external artwork is needed to play.

## Production atlas prompt

Built-in image generation; final edit of the initial court atlas, using the
approved deck artwork study as its style reference:

> Preserve these exact three characters, finely engraved faces and costumes and
> navy/crimson/gold/sage style. Recompose it into a production atlas of THREE
> CLASSICAL DOUBLE ENDED playing card court illustrations. Canvas 1536x1024,
> exactly equal thirds 512 pixels wide. Left JACK, middle QUEEN, right KING.
> Each panel must contain a FULL TWO-HEADED reversible court figure: upright bust
> in TOP HALF y0..512 and the same bust rotated180degrees in BOTTOM HALF
> y512..1024, touching seamlessly at waists along y512. Crucial: compress
> composition by drawing upper figures as WIDE WAIST-UP BUSTS in their
> square512x512 top half, keep natural human proportions and beautiful engraving.
> Crowns/headwear near top30px, faces between80and240, shoulders and robes down
> to center512. Bottom copies upside down with crowns near bottom994, heads
> between784and944. Clear smooth ivory #faf7ef around figures, no card frame,
> no rank letters, no printed suit symbols, no logo, no text or dividers. All
> three panels entirely self contained, no artwork crosses the exact one-third
> panel boundaries. Do not simply duplicate the tall full-length reference
> portraits: redraw as square busts with much shorter torsos so the resulting
> figures are naturally proportioned in the double-ended format. This is the
> final game texture.

## Native proofs

`examples/deck-preview.rs` calls the actual production card painter. Run on a
desktop or under `xvfb-run`:

```sh
cargo build --locked --release --example deck-preview
target/release/examples/deck-preview deck.png
target/release/examples/deck-preview faces.png --all
target/release/examples/deck-preview light.png --light
```

The proof does not read or write game saves. The complete-deck proof includes
all 52 faces at 80 px; the main proof includes 80 px back samples.

Rebuild rank outlines with `python tools/build_rank_art.py /path/to/DejaVuSerif-Bold.ttf`
(fontTools is a development-only requirement). Each fragment records its visible
ink width, allowing a clear 24-unit gap before the suit without squeezing 10.
The native proof accepts `--numbers` for pip layouts and overlapping indexes.
