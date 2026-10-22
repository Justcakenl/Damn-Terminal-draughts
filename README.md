# DAMN

### Terminal draughts, with real rules and a real opponent

DAMN is a fast, offline terminal game for **international draughts** on a 10×10 board. It is built in Rust with Ratatui and Crossterm and runs directly in Linux terminals, macOS Terminal/iTerm2, Windows Terminal, Kitty, Alacritty, and foot.

> No account. No browser. No server. No API key. Everything runs locally.

![DAMN terminal game](https://placehold.co/1200x650/10151c/8be9fd?text=DAMN+%7C+Terminal+Draughts)

## Features

- Correct 10×10 international-draughts board
- Mandatory captures and maximum-capture selection
- Multi-capture sequences, flying kings, and promotion
- Human vs AI with Easy, Medium, Hard, and Impossible depths
- Local two-player mode: both players control their own pieces
- Computer-vs-computer watch mode
- Keyboard, mouse click, and drag controls
- Save/load games using a portable JSON save file
- Undo, restart confirmation, pause screen, help menu, and game-over screen
- Responsive compact layout for smaller terminals
- Offline-first: no network access or external service required

## Install

### Arch Linux / Linux — recommended

Install Rust and Git:

```bash
sudo pacman -S --needed rust git
```

Clone and install DAMN for your user:

```bash
git clone https://github.com/YOUR-USERNAME/damn.git
cd damn
chmod +x install.sh
./install.sh
```

Open a new terminal. Then start the game from **any directory**:

```bash
damn
```

The installer puts the executable at `~/.local/bin/damn` and adds that directory to `.bashrc` and `.zshrc` when available. If your shell still cannot find it, run:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

To uninstall:

```bash
rm -f "$HOME/.local/bin/damn"
```

### Any operating system — build from source

Install Rust from [rustup.rs](https://rustup.rs/), then:

```bash
git clone https://github.com/Justcakenl/damn.git
cd damn
cargo build --release
```

The executable is created at:

- Linux/macOS: `target/release/damn`
- Windows: `target\\release\\damn.exe`

Run it directly:

```bash
# Linux/macOS
./target/release/damn

# Windows PowerShell
.\\target\\release\\damn.exe
```

### Windows

Install:

1. [Git for Windows](https://git-scm.com/download/win)
2. [Rust via rustup](https://rustup.rs/)
3. [Windows Terminal](https://aka.ms/terminal) — recommended

In PowerShell:

```powershell
git clone https://github.com/Justcakenl/damn.git
cd damn
cargo run --release
```

To install the executable into Cargo's user bin directory:

```powershell
cargo install --path .
```

After that, launch with:

```powershell
damn
```

### macOS

Install Xcode Command Line Tools and Rust:

```bash
xcode-select --install
```

Install Rust from [rustup.rs](https://rustup.rs/), then:

```bash
git clone https://github.com/Justcakenl/damn.git
cd damn
cargo install --path .
damn
```

Use Terminal, iTerm2, Kitty, or Alacritty for the best display.

## Start a game

When DAMN opens, choose a mode:

| Key | Mode |
|---:|---|
| `1` | Local two-player — both people play manually |
| `2` | Human vs AI — Easy |
| `3` | Human vs AI — Medium |
| `4` | Human vs AI — Hard |
| `5` | Human vs AI — Impossible/deep search |
| `0` | Computer vs computer |

You play White and White moves first in Human vs AI mode. In Local mode, the active side shown in the status panel is controlled by the person at the keyboard.

## Controls

### Board

| Key / action | What it does |
|---|---|
| Arrow keys | Move the cursor |
| `W A S D` | Move the cursor |
| `Enter` / `Space` | Select a piece or confirm a destination |
| `Tab` | Cycle between legal destinations |
| Mouse click | Select a piece or confirm a legal square |
| Mouse drag | Drag a selected piece to a legal destination |
| `Esc` | Cancel selection / close a menu |

### Game and menus

| Key | What it does |
|---:|---|
| `?` | Open the complete controls menu |
| `H` | Toggle legal-move hints |
| `U` | Undo the last move |
| `R` | Restart, with confirmation |
| `P` | Pause |
| `S` | Save the current game |
| `L` | Load the saved game |
| `Q` | Quit, with confirmation |
| `A` | Force an AI move when available |

Cyan squares are legal destinations. Yellow marks the selected piece. Magenta marks the cursor. `W` and `B` identify the sides; `W*` and `B*` identify kings.

## Rules

DAMN follows international draughts rules:

- The board has 100 squares; only 50 dark squares are playable.
- Each side starts with 20 men.
- Captures are compulsory.
- If multiple capture sequences exist, the sequence capturing the most pieces is required.
- Men capture forwards and backwards.
- Kings are flying kings and may move across multiple empty diagonal squares.
- A man is promoted when its complete move ends on the opponent's back rank.
- A capture sequence cannot be ended early.
- A player loses when they have no pieces or no legal move.

The rules engine is shared by the UI, AI, and tests; the interface does not implement a second move system.

## AI

The AI uses deterministic minimax search with alpha-beta pruning and a material evaluation. Difficulty changes search depth:

- **Easy:** depth 2
- **Medium:** depth 3
- **Hard:** depth 5
- **Impossible:** depth 7

The AI always chooses from the authoritative legal-move list, so mandatory captures and maximum-capture selection are enforced automatically. Search runs locally and does not use an API or network connection.

## Saves and data

Save files are stored in the platform's user data directory through the XDG/OS conventions:

- Linux: typically `~/.local/share/DAMN/save.json`
- macOS: typically `~/Library/Application Support/org.damn.DAMN/save.json`
- Windows: typically `%APPDATA%\\damn\\DAMN\\data\\save.json`

The save format is versioned JSON. Writes use a temporary file followed by an atomic rename to reduce corruption risk.

## Development

Requirements: stable Rust and Cargo.

```bash
cargo fmt
cargo test
cargo build --release
```

The source is organized into:

```text
src/
├── app.rs                 # terminal loop, input, screens, rendering
├── game/
│   ├── board.rs           # board representation
│   ├── piece.rs           # pieces and sides
│   ├── rules.rs           # authoritative legal moves
│   ├── state.rs           # state transitions
│   └── tests.rs           # rules and AI regressions
├── ai/
│   ├── evaluation.rs      # position scoring
│   ├── search.rs          # minimax and alpha-beta
│   └── transposition.rs   # position table utilities
├── persistence/save.rs    # versioned save/load
└── ui/theme.rs            # terminal colors
```

## Troubleshooting

**`damn: command not found`**

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then open a new shell. On Windows, restart PowerShell after `cargo install --path .`.

**The terminal looks corrupted after an interruption**

Run `reset` or `stty sane`. DAMN restores raw mode and the alternate screen on normal exit.

**The board says the terminal is too small**

Resize the terminal. DAMN supports a compact layout, but very small terminals cannot display a usable 10×10 board.

**Mouse input does not work**

Use the keyboard controls; they are fully supported. Mouse reporting depends on the terminal emulator.

## Project status

DAMN is an offline playable terminal game. The AI is a practical local search opponent, not a tournament engine, and the project does not claim grandmaster strength.

### Test project

DAMN was created as a test project using **cake-cli** to explore AI-assisted Rust development, terminal UI design, game-engine architecture, and automated testing. The project is intended as a practical engineering experiment and remains fully local and offline at runtime.

## License

MIT License

Copyright (c) 2026 Justcakenl

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

