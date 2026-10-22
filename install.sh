#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
TARGET="$BIN_DIR/damn"

command -v cargo >/dev/null 2>&1 || {
    printf '%s\n' 'Cargo is missing. On Arch: sudo pacman -S rust' >&2
    exit 1
}

mkdir -p "$BIN_DIR"
printf 'Building DAMN...\n'
cargo build --manifest-path "$ROOT/Cargo.toml" --release
cp "$ROOT/target/release/damn" "$TARGET"
chmod 755 "$TARGET"

# Make the command available in future shells. A parent shell cannot be
# modified by a child script, so also export it for this invocation below.
PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'
for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [ -f "$rc" ] && ! grep -Fqx "$PATH_LINE" "$rc"; then
        printf '\n# DAMN\n%s\n' "$PATH_LINE" >> "$rc"
    fi
done

case ":${PATH}:" in
    *":$BIN_DIR:"*) : ;;
    *) export PATH="$BIN_DIR:$PATH" ;;
esac

printf '\nInstalled: %s\n' "$TARGET"
printf 'Start now with:  damn\n'
printf 'New terminals will also recognize: damn\n'
"$TARGET" --version 2>/dev/null || true
