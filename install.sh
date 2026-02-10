#!/bin/bash
set -e

REPO_OWNER="zakarialabib"
REPO_NAME="rustools-mcp"
INSTALL_DIR="$HOME/.local/bin"
BIN_NAME="rustools-mcp"

echo "🚀 Installing rustools-mcp..."

# 1. Create Installation Directory
mkdir -p "$INSTALL_DIR"
echo "Created installation directory: $INSTALL_DIR"

# 2. Determine Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        ASSET_NAME="rustools-mcp-linux-amd64"
        ;;
    Darwin)
        ASSET_NAME="rustools-mcp-macos-amd64"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# 3. Download (Placeholder)
DOWNLOAD_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$ASSET_NAME"
echo "Downloading from: $DOWNLOAD_URL"
# curl -L -o "$INSTALL_DIR/$BIN_NAME" "$DOWNLOAD_URL"
# chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "⚠️  NOTE: Since this is a dev environment, please build manually:"
echo "   cargo build --release"
echo "   cp target/release/rustools-mcp $INSTALL_DIR/$BIN_NAME"

# 4. Add to PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Please add $INSTALL_DIR to your PATH:"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi

echo "✅ Installation Setup Complete!"
