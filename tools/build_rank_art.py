#!/usr/bin/env python3
"""Rebuild outlined card ranks. Development only: requires fontTools and DejaVu Serif Bold."""
import argparse
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('font', type=Path)
args = parser.parse_args()
font = TTFont(args.font)
glyphs = font.getGlyphSet()
cmap = font.getBestCmap()
output = []
for rank in ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K']:
    paths, bounds, advance = [], [], 0
    for char in rank:
        name = cmap[ord(char)]
        path, box = SVGPathPen(glyphs), BoundsPen(glyphs)
        glyphs[name].draw(path)
        glyphs[name].draw(box)
        x0, y0, x1, y1 = box.bounds
        bounds.append((x0 + advance, y0, x1 + advance, y1))
        paths.append(f'<path transform="translate({advance} 0)" d="{path.getCommands()}"/>')
        advance += font['hmtx'][name][0]
    left = min(b[0] for b in bounds)
    right = max(b[2] for b in bounds)
    top = max(b[3] for b in bounds)
    bottom = min(b[1] for b in bounds)
    scale = 78 / (top - bottom)
    width = (right - left) * scale
    # Store measured ink width, with uniform scaling and zero visible left bearing.
    output.append(f'{width:.4f}|<g transform="translate({-left * scale:.4f} {top * scale:.4f}) scale({scale:.8f} {-scale:.8f})">'+''.join(paths)+'</g>')
(Path(__file__).resolve().parents[1] / 'assets/deck/ranks.svgparts').write_text('\n'.join(output) + '\n')
