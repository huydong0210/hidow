#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────
# Hidow Setup Script
# Auto-detects ONNX Runtime, downloads if missing,
# builds hidow and installs globally.
#
# Usage:
#   ./scripts/setup.sh
# ─────────────────────────────────────────────────────────
set -euo pipefail

# ── Config ───────────────────────────────────────────────
ORT_VERSION="1.20.1"
ORT_DIR="$HOME/.hidow/ort"
INSTALL_DIR="$HOME/.cargo/bin"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

info()  { echo -e "${CYAN}▸${NC} $*"; }
ok()    { echo -e "${GREEN}✅${NC} $*"; }
warn()  { echo -e "${YELLOW}⚠️${NC}  $*"; }
fail()  { echo -e "${RED}❌${NC} $*"; exit 1; }

# ── Help ─────────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        --help|-h)
            echo "Usage: ./scripts/setup.sh"
            echo ""
            echo "This script will:"
            echo "  1. Auto-detect ONNX Runtime at ~/.hidow/ort/ (download if missing)"
            echo "  2. Build hidow in release mode"
            echo "  3. Install binary to ~/.cargo/bin/hidow"
            exit 0
            ;;
    esac
done

echo -e "\n${BOLD}🚀 Hidow Setup${NC}\n"

# ── Step 1: Check Rust toolchain ─────────────────────────
info "Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
    fail "Rust not found. Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi
RUST_VER=$(rustc --version | awk '{print $2}')
ok "Rust $RUST_VER"

# ── Step 2: ONNX Runtime (auto-detect / download) ───────
info "Checking ONNX Runtime..."

if [ -f "$ORT_DIR/lib/libonnxruntime.so.$ORT_VERSION" ]; then
    ok "ONNX Runtime $ORT_VERSION found at $ORT_DIR"
elif [ -f "$ORT_DIR/lib/libonnxruntime.so" ]; then
    FOUND_VER=$(ls "$ORT_DIR/lib/libonnxruntime.so."* 2>/dev/null | grep -oP '\d+\.\d+\.\d+$' | head -1 || echo "unknown")
    ok "ONNX Runtime found (version: $FOUND_VER)"
else
    info "ONNX Runtime not found — downloading v$ORT_VERSION..."

    # Detect architecture
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64)  ORT_ARCH="x64" ;;
        aarch64) ORT_ARCH="aarch64" ;;
        *)       fail "Unsupported architecture: $ARCH" ;;
    esac

    ORT_FILENAME="onnxruntime-linux-${ORT_ARCH}-${ORT_VERSION}.tgz"
    ORT_URL="https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VERSION}/${ORT_FILENAME}"

    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    info "  URL: $ORT_URL"
    curl -fSL --progress-bar "$ORT_URL" -o "$TMP_DIR/$ORT_FILENAME" \
        || fail "Download failed. Check your internet connection."

    info "  Extracting to $ORT_DIR..."
    mkdir -p "$ORT_DIR"
    EXTRACTED_DIR="$TMP_DIR/onnxruntime-linux-${ORT_ARCH}-${ORT_VERSION}"
    tar -xzf "$TMP_DIR/$ORT_FILENAME" -C "$TMP_DIR"
    cp -r "$EXTRACTED_DIR/lib" "$ORT_DIR/"
    cp -r "$EXTRACTED_DIR/include" "$ORT_DIR/" 2>/dev/null || true

    ok "ONNX Runtime $ORT_VERSION installed"
fi

# ── Step 3: Build ────────────────────────────────────────
info "Building hidow (release mode)..."
cd "$PROJECT_DIR"
cargo build --release 2>&1 | tail -3
ok "Build complete"

# ── Step 4: Install binary ───────────────────────────────
info "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
cp "$PROJECT_DIR/target/release/hidow" "$INSTALL_DIR/hidow"
chmod +x "$INSTALL_DIR/hidow"

# Check if install dir is in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    warn "$INSTALL_DIR is not in your PATH. Add to ~/.bashrc:"
    echo -e "    ${BOLD}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
fi

ok "hidow installed at $INSTALL_DIR/hidow"

# ── Step 5: Verify ───────────────────────────────────────
info "Verifying..."
HIDOW_VER=$("$INSTALL_DIR/hidow" --version 2>&1 || true)
ok "$HIDOW_VER"

# ── Summary ──────────────────────────────────────────────
echo -e "\n${BOLD}${GREEN}✅ Setup complete!${NC}\n"
echo -e "  Binary:   ${BOLD}$INSTALL_DIR/hidow${NC}"
echo -e "  ORT lib:  ${BOLD}$ORT_DIR/lib/${NC}"
echo ""
echo -e "  ${CYAN}Quick start:${NC}"
echo "    hidow --wiki-path /path/to/wiki ingest"
echo "    hidow query semantic \"tính phí bảo hiểm\""
echo "    hidow query ask \"XOL calculation\" --format json"
echo ""
