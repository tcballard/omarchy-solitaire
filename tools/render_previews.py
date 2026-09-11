#!/usr/bin/env python3
"""Capture the real native binary. Requires a display (use xvfb-run in CI)."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('target/release/omarchy-solitaire'))
    parser.add_argument('--output-dir', type=Path, default=Path('docs/screenshots'))
    parser.add_argument('--deck-binary', type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    root = Path(__file__).resolve().parent.parent
    baseline = json.loads((root / 'tests/fixtures/python-session.json').read_text())
    fixtures = json.loads((root / 'tests/fixtures/python-games.json').read_text())
    for name, width, height in [('table', 1120, 800), ('compact', 800, 600), ('light', 1120, 800), ('progress', 800, 600)]:
        with tempfile.TemporaryDirectory(prefix='solitaire-render-') as temporary:
            directory = Path(temporary)
            state = json.loads(json.dumps(baseline))
            state['game'] = {'version': 1, 'state': fixtures[0]['initial'], 'history': []}
            state['elapsed'] = 0
            state['started'] = False
            state['preferences'].update(pattern=0, finish='matte', reducedMotion=True, followTheme=False)
            state['themeName'] = 'House green'
            state['themeSnapshot'] = {'table': '#182622', 'text': '#e6eadb', 'accent': '#b7ce91'}
            if name == 'light':
                state['themeName'] = 'Paper'
                state['themeSnapshot'] = {'table': '#ece9df', 'text': '#263630', 'accent': '#416737'}
            if name == 'progress':
                state['game'] = fixtures[4]['expected']
            (directory / 'session.json').write_text(json.dumps(state))
            subprocess.run([str(binary), '--state-dir', str(directory), '--width', str(width), '--height', str(height), '--screenshot', str(output / f'{name}.png')], check=True, timeout=30, env=dict(os.environ))
            if not (output / f'{name}.png').read_bytes().startswith(b'\x89PNG\r\n\x1a\n'):
                raise RuntimeError(f'{name}: screenshot was not written')
            print(f'Captured {name}: {width} × {height}')

    if args.deck_binary:
        for name, extra in [('deck-artwork', []), ('deck-all-faces', ['--all']), ('deck-artwork-light', ['--light']), ('deck-numbers', ['--numbers'])]:
            target = output / f'{name}.png'
            subprocess.run([str(args.deck_binary.resolve()), str(target), *extra], check=True, timeout=30)
            if not target.read_bytes().startswith(b'\x89PNG\r\n\x1a\n'):
                raise RuntimeError(f'{name}: screenshot was not written')
            print(f'Captured {name}')


if __name__ == '__main__':
    main()
