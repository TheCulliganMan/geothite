# Build and install the native Windows game and its desktop shortcut.
set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

# Build the Windows game, bundle its assets, and create the desktop icon.
desktop pack="content-packs/core-modular.crystalpack":
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Pack '{{pack}}'

# Initialize the external content before the desktop build on a fresh checkout.
desktop-content:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-crystalpack.ps1
