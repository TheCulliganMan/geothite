# Build and install the native Windows game and its desktop shortcut.
set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

# Build fresh, pinned source content and publish only after verification.
crystalpack:
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-crystalpack.ps1" } else { "bash tools/build-crystalpack.sh" }}}

# Build the Windows game, bundle its assets, and create the desktop icon.
desktop pack="content-packs/core-modular.crystalpack":
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Pack '{{pack}}'

# Initialize the external content before the desktop build on a fresh checkout.
desktop-content:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-crystalpack.ps1

# Quickly serve the prepared local browser copy; Ctrl-C stops the server.
serve port="3003" runtime=".local/desktop-3d-runtime":
    @echo 'Open http://localhost:{{port}}/?multiplayer=off'
    CRYSTAL_DATA_DIR='{{runtime}}/data' '{{runtime}}/crystal-web-server' --dir '{{runtime}}/web' --host 127.0.0.1 --port '{{port}}'

# Launch the prepared native 3D client in the bedroom, without the intro.
play-3d runtime=".local/desktop-3d-runtime":
    '{{runtime}}/new_bark_3d' content-packs/core-modular.browser.crystalpack --map PlayersHouse2F --x 3 --y 3 --save-path content-packs/desktop-3d-test.crystalsave

# Capture every render stage and regional load while playing the native client.
trace-3d trace=".local/loading-trace.json" runtime=".local/desktop-3d-runtime":
    CRYSTAL_LOADING_TRACE='{{trace}}' '{{runtime}}/new_bark_3d' content-packs/core-modular.browser.crystalpack --map PlayersHouse2F --x 3 --y 3
