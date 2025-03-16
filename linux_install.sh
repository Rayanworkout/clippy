#!/bin/bash
set -e

BUILD_DIR="./target/release"

# Destination directory (common location for system-wide binaries on Linux)
DEST_DIR="/usr/local/bin"

# Check if the daemon binary exists in the script's directory
if [ ! -f "$BUILD_DIR/daemon" ]; then
    echo "Error: 'daemon' binary not found. Exiting."
    exit 1
fi

# Check if the ui binary exists in the script's directory
if [ ! -f "$BUILD_DIR/ui" ]; then
    echo "Error: 'ui' binary not found. Exiting."
    exit 1
fi

# Ensure the destination directory exists, create it if it doesn't
if [ ! -d "$DEST_DIR" ]; then
    echo "Destination directory $DEST_DIR does not exist. Creating it..."
    sudo mkdir -p "$DEST_DIR"
fi

# Move the files to the destination directory
echo "Installing daemon binary to $DEST_DIR..."
sudo mv "$BUILD_DIR/clippy_daemon" "$DEST_DIR"

echo "Installing ui binary to $DEST_DIR..."
sudo mv "$BUILD_DIR/clippy_ui" "$DEST_DIR"

# Make sure the binaries are executable
echo "Setting execute permissions on the binaries..."
sudo chmod +x "$DEST_DIR/clippy_daemon" "$DEST_DIR/clippy_ui"

echo "Installation complete!"