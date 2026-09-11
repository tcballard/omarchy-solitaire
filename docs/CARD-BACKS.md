# Card-back artwork handoff

The game is ready for special illustrated decks. The official Omarchy decal is
already on all three built-in backs: **Tilework** (default), **Engraved**, and
**Foil**. Foil is an engraved design; the optional Holographic finish can be
applied to any back. Imported artwork replaces the entire back,
so include the decal in the supplied design if you want it there.

## Deliverable

| Property | Requirement |
| --- | --- |
| Canvas | 500 × 700 recommended; 5:7 aspect ratio |
| Formats | SVG preferred; PNG supported |
| File size | At most 2 MB |
| PNG dimensions | At most 4096 × 4096 |
| Artwork | Static paths/shapes; outline all text |
| External content | No linked images, fonts, scripts, CSS or remote resources |
| Safe area | Keep critical marks 30 px inside each edge |
| Orientation | A rotationally symmetric design is recommended |
| Transparency | Allowed; the theme-coloured backing shows through |

Artwork is fit without stretching inside the card's five-pixel inset. Cards are
rendered at roughly 96–124 logical pixels wide in normal window sizes, so check
that fine detail survives at 100 pixels. SVG art is rasterized into a bounded 500 × 700 texture; built-in backs use the same resolution with mipmap filtering. Faces use cached
250 × 350 textures with separate illustrated courts.

SVG supports svg, g, path, rect, circle, ellipse, line, polyline, polygon, defs,
linearGradient, radialGradient, stop, clipPath, title and desc. Internal
`url(#gradient-id)` references are permitted. Use presentation attributes such
as `fill`, `stroke` and `opacity`, rather than `style` or `<style>` blocks.

The loader rejects scripts, entities, foreign objects, linked content, raster
images embedded in SVG and live SVG animation. This is deliberately a data-only
extension point. Holographic motion is an app-provided finish, not executable
code shipped inside a theme.

## Preview an individual design

1. Open Deck & settings.
2. Import the SVG or PNG.
3. Select Special. The app copies the artwork to its own save directory.
4. Optionally enable Holographic finish; hover over a back to see the sheen.
5. Check both light and dark themes, the 800 × 600 window, and Reduce motion.

## Ship with an Omarchy theme

Add one of these paths inside the theme directory:

```text
solitaire/back.svg
solitaire/back.png
```

SVG takes precedence if both exist. These paths are this app's own convention,
not an existing Omarchy theme standard. Current Omarchy stages extra theme
directories; the app reads the staged `current/theme/solitaire/` artwork.

When theme following is enabled, palette and artwork are checked every 1.5
seconds using file metadata. Atomic directory replacements and symlink target
changes are detected. Invalid artwork falls back to the built-in decal and is
reported in settings. An explicitly selected Special back takes precedence over
theme-provided artwork.

The built-in holographic finish adds a restrained multicolour sheen on hover.
It becomes static with Reduce motion. A future per-deck manifest can add finish
defaults and creator metadata without changing Klondike's rules or save schema;
no manifest format is promised in v0.2.0.

## Brand source

Use the supplied unchanged mark or the official downloads at
https://omarchy.org/brand/. This project preserves the official green fill and
geometry. The artwork's original rights remain with its owners; see NOTICE.
