# Omarchy Solitaire

A little time well spent. Native Klondike for Omarchy, with traditional cards,
the official Omarchy decal, and a table that follows your desktop theme.

![Native Omarchy Solitaire window](docs/screenshots/table.png)

## Play

- Draw one or three, with unlimited stock recycling.
- Drag cards and whole sequences, or click a card and its destination.
- Legal destinations are marked. Double-click or right-click a card to send it home.
- Undo every move in the current game, including draws, recycling and automatic completion.
- Hints, restart the same deal, scoring, local statistics and automatic save/resume.
- Keyboard play and scrolling for long columns. The timer pauses in inactive windows and dialogs.
- A cascading-card celebration, plus a reduced-motion setting.

Deals are randomly shuffled; they are **not guaranteed winnable**. Hints suggest
legal moves rather than solving the game. Finish is offered only once stock and
waste are empty and every table card is face up.

## Install on Omarchy / Arch

The repository builds an Arch package; it does not require the plugin shell.

```sh
git clone https://github.com/tcballard/omarchy-solitaire.git
cd omarchy-solitaire
makepkg -si
```

`makepkg` must run as your ordinary user. It asks pacman to install dependencies.
Launch **Omarchy Solitaire** from the application launcher, or run:

```sh
omarchy-solitaire
```

CI also builds an installable `.pkg.tar.zst` artifact on every push. Download a
successful workflow's `arch-package` artifact, extract it, then install with:

```sh
sudo pacman -U ./omarchy-solitaire-*.pkg.tar.zst
```

Tagged releases have a workflow that prepares a **draft** release and checksum
file. A public release is not implied by the presence of that workflow.

## Run from source

Python 3.11+ and Qt 6.8+ through PySide6 are required. On Arch, use the system
`pyside6`, `qt6-declarative`, `qt6-svg`, `qt6-wayland` and `ttf-dejavu` packages.

```sh
python -m omarchy_solitaire
```

For a development environment on another Linux distribution:

```sh
python -m venv .venv
. .venv/bin/activate
python -m pip install -e .
python -m omarchy_solitaire
```

The app is a native Qt Quick window with Python rules, not a webview. It has no
network requests, accounts, telemetry, advertising or paid game features.

## Theme and special decks

Three built-in patterns use the active theme palette and preserve the official
Omarchy mark. An optional **holographic finish** adds a subtle hover shimmer;
reduced motion makes it static. The team's custom designs can replace the
entire back via **Deck & settings → Import a special card back**.

Theme authors may provide `solitaire/back.svg` or `solitaire/back.png` inside
their theme. The app detects both palette and artwork replacements while
preserving the game. See [the artwork handoff](docs/CARD-BACKS.md) for exact
dimensions, formats and extension rules.

Current palette discovery checks:

1. `$XDG_STATE_HOME/omarchy/current/theme/colors.toml`, defaulting to
   `~/.local/state/omarchy/current/theme/colors.toml`.
2. Omarchy's fixed default state path if a different XDG path is configured.
3. The older `~/.config/omarchy/current/theme/colors.toml` path.

Without Omarchy, the app uses its built-in House green palette. Missing or invalid
palette writes keep the last valid colours until a valid replacement appears.
Card faces preserve readable red/black suits independently of theme colours.

## Controls

| Action | Control |
| --- | --- |
| Select / place | Click, or Enter on the focused card/pile |
| Move a sequence | Drag its first face-up card |
| Move to a foundation | Double-click, right-click, or F |
| Draw / recycle | Click stock, or Space with the board focused |
| Navigate piles / cards | Left/right, up/down |
| Clear selection | Escape |
| Undo | Ctrl+Z |
| Hint | H |
| New / restart | Ctrl+N |
| Settings | Ctrl+, |
| Help | F1 |
| Quit | Ctrl+Q |

Tab moves through controls and back to the table. Selected cards and legal
destinations use shapes as well as colour.

## Saves, statistics and recovery

Data lives in `$XDG_STATE_HOME/omarchy-solitaire`, normally
`~/.local/state/omarchy-solitaire`. Saves are atomic and private to the user;
one app instance owns a save directory at a time. Every move saves immediately;
the clock saves every ten seconds and on clean exit. An abrupt kill can lose
up to ten seconds of elapsed time, but not the last successfully saved move.

The file includes the full undo history. Reads are bounded to 32 MB. Invalid
saves are copied to `session-unreadable-*.json` before starting a new deal.
Save failures appear in the status bar. Back up this directory to move your
games and imported artwork to another machine.

A deal counts as started at its first move. Restarts and undo do not count it
again; each deal can add only one win. Scores: +10 to foundations, +5 from waste
to table, +5 reveal, −15 foundation to table, −20 stock recycle, with a floor of
zero. Undo restores score; the clock keeps running. There are no time bonuses.

## Development and verification

```sh
python -m unittest discover -s tests -v
QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
  python -m omarchy_solitaire --state-dir /tmp/solitaire-test \
  --screenshot /tmp/solitaire.png
```

Tests cover rules, random legal-play invariants, serialization/undo, corrupt
saves, theme replacement, contrast, artwork validation, statistics, native Qt
click/keyboard/drag input, and rendering. Read [DECISIONS.md](docs/DECISIONS.md)
for implementation choices and [VERIFICATION.md](docs/VERIFICATION.md) for the
current validation record and remaining desktop checks.

## Attribution

Application code and original card drawings: MIT. Official Omarchy artwork:
separate rights retained, see [NOTICE](NOTICE). This is a community application
and does not claim Foundation endorsement. No Microsoft game assets are used.
