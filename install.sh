#!/bin/sh
# OneInit installer — download pre-built binary, no Rust toolchain needed.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/oneinitAI/oneinit/main/install.sh | sh
#
# Detects OS + architecture, downloads the matching binary from GitHub
# Releases, installs to ~/.oneinit/bin, and adds to PATH.

set -e

# ============================================================
# Detect platform
# ============================================================

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)  PLATFORM="linux" ;;
    Darwin*) PLATFORM="macos" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
    *) echo "[ERROR] Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)  ARCHITECTURE="x86_64" ;;
    aarch64|arm64) ARCHITECTURE="aarch64" ;;
    *) echo "[ERROR] Unsupported architecture: $ARCH"; exit 1 ;;
esac

if [ "$PLATFORM" = "windows" ]; then
    BINARY_NAME="oneinit.exe"
    ARCHIVE_EXT="zip"
else
    BINARY_NAME="oneinit"
    ARCHIVE_EXT="tar.gz"
fi

# ============================================================
# Configuration
# ============================================================

INSTALL_DIR="$HOME/.oneinit/bin"
REPO="oneinitAI/oneinit"
VERSION="${ONEINIT_VERSION:-latest}"

echo "[INSTALL] OneInit for $PLATFORM/$ARCHITECTURE"

# ============================================================
# Get download URL
# ============================================================

if command -v curl >/dev/null 2>&1; then
    FETCH="curl -fsSL"
elif command -v wget >/dev/null 2>&1; then
    FETCH="wget -qO-"
else
    echo "[ERROR] Need curl or wget to download."
    exit 1
fi

# Resolve latest version tag from GitHub API
if [ "$VERSION" = "latest" ]; then
    echo "[INSTALL] Fetching latest version..."
    TAG="$($FETCH "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
    if [ -z "$TAG" ]; then
        echo "[ERROR] Could not determine latest version."
        echo "[HINT]  No GitHub Releases found. Build from source instead:"
        echo "        git clone https://github.com/$REPO.git && cd oneinit && cargo build --release"
        exit 1
    fi
else
    TAG="$VERSION"
fi

echo "[INSTALL] Version: $TAG"

# ============================================================
# Download
# ============================================================

ARCHIVE_NAME="oneinit-${TAG}-${PLATFORM}-${ARCHITECTURE}.${ARCHIVE_EXT}"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/${TAG}/${ARCHIVE_NAME}"
SUMS_URL="https://github.com/$REPO/releases/download/${TAG}/SHA256SUMS.txt"

echo "[INSTALL] Downloading: $DOWNLOAD_URL"

TMPDIR="$(mktemp -d || echo /tmp/oneinit-install-$$)"
trap 'rm -rf "$TMPDIR"' EXIT

# download to file (curl or wget)
download_file() {
    url="$1"
    out="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$out" "$url"
    else
        wget -q -O "$out" "$url"
    fi
}

# sha256 of a file (sha256sum on Linux, shasum on macOS)
calc_sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        echo "[ERROR] Need sha256sum or shasum to verify download." >&2
        exit 1
    fi
}

# download + verify against release SHA256SUMS.txt; mismatch -> abort
download_and_verify() {
    url="$1"
    archive_out="$2"
    sums_out="$3"

    download_file "$url" "$archive_out" || {
        echo "[ERROR] Download failed: $url"
        exit 1
    }

    # 旧版本 Release 可能没有 SHA256SUMS.txt：缺失时告警但不阻断（尽力而为）
    if download_file "$SUMS_URL" "$sums_out" 2>/dev/null; then
        expected="$(grep "  ${ARCHIVE_NAME}$" "$sums_out" | awk '{print $1}')"
        if [ -z "$expected" ]; then
            echo "[WARN] SHA256SUMS.txt has no entry for ${ARCHIVE_NAME} — skipping verify."
            return 0
        fi
        actual="$(calc_sha256 "$archive_out")"
        if [ "$expected" = "$actual" ]; then
            echo "[OK] SHA256 verified: ${ARCHIVE_NAME}"
        else
            echo "[ERROR] SHA256 mismatch for ${ARCHIVE_NAME}"
            echo "        expected: $expected"
            echo "        actual:   $actual"
            echo "[ABORT] Download may be tampered. Refusing to install."
            exit 1
        fi
    else
        echo "[WARN] SHA256SUMS.txt not available for ${TAG} — skipping SHA256 verify."
    fi
}

if [ "$ARCHIVE_EXT" = "zip" ]; then
    download_and_verify "$DOWNLOAD_URL" "$TMPDIR/oneinit.zip" "$TMPDIR/SHA256SUMS.txt"
    if command -v unzip >/dev/null 2>&1; then
        (cd "$TMPDIR" && unzip -o oneinit.zip)
    elif command -v python3 >/dev/null 2>&1; then
        (cd "$TMPDIR" && python3 -c "import zipfile; zipfile.ZipFile('oneinit.zip').extractall('.')")
    else
        echo "[ERROR] Need unzip or python3 to extract .zip"
        exit 1
    fi
else
    download_and_verify "$DOWNLOAD_URL" "$TMPDIR/oneinit.tar.gz" "$TMPDIR/SHA256SUMS.txt"
    tar -xzf "$TMPDIR/oneinit.tar.gz" -C "$TMPDIR"
fi

# Find the binary
BINARY_PATH=""
for candidate in "$TMPDIR/$BINARY_NAME" "$TMPDIR/oneinit/$BINARY_NAME" "$TMPDIR"/*/"$BINARY_NAME"; do
    if [ -f "$candidate" ]; then
        BINARY_PATH="$candidate"
        break
    fi
done

if [ -z "$BINARY_PATH" ]; then
    echo "[ERROR] Binary not found in archive."
    exit 1
fi

# ============================================================
# Install
# ============================================================

mkdir -p "$INSTALL_DIR"
cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || true

echo "[OK] Installed to: $INSTALL_DIR/$BINARY_NAME"

# ============================================================
# Add to PATH
# ============================================================

add_to_path() {
    SHELL_NAME="$(basename "$SHELL" 2>/dev/null || echo bash)"

    case "$SHELL_NAME" in
        fish)
            RC_FILE="$HOME/.config/fish/config.fish"
            LINE="set -gx PATH $INSTALL_DIR \$PATH"
            ;;
        zsh)
            RC_FILE="$HOME/.zshrc"
            LINE="export PATH=\"$INSTALL_DIR:\$PATH\""
            ;;
        *)
            RC_FILE="$HOME/.bashrc"
            LINE="export PATH=\"$INSTALL_DIR:\$PATH\""
            ;;
    esac

    # Check if already in PATH config
    if [ -f "$RC_FILE" ] && grep -q ".oneinit/bin" "$RC_FILE" 2>/dev/null; then
        echo "[OK] PATH already configured in $RC_FILE"
        return
    fi

    printf '\n# Added by OneInit installer\n%s\n' "$LINE" >> "$RC_FILE"
    echo "[OK] Added to PATH in $RC_FILE"
    echo "[INFO] Run 'source $RC_FILE' or open a new terminal."
}

if [ "$PLATFORM" != "windows" ]; then
    add_to_path
fi

# ============================================================
# Verify
# ============================================================

export PATH="$INSTALL_DIR:$PATH"

echo ""
echo "[OK] OneInit installed successfully!"
echo ""
echo "Next steps:"
echo "  oneinit --version"
echo "  oneinit doctor"
echo "  oneinit update"
echo "  oneinit install python3.11"
echo ""
echo "Or launch the interactive TUI:"
echo "  oneinit tui"
echo ""
