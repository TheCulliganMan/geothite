# Desktop commands select the current host. Products remain ignored.
set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

# Build fresh, pinned source content and publish only after verification.
crystalpack:
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-crystalpack.ps1" } else { "bash tools/build-crystalpack.sh" }}}

# Build, validate and install the native desktop app. Preserves saves.
desktop pack="content-packs/core-modular.crystalpack":
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Install -Pack" } else { "python3 tools/desktop.py install --pack" }}} '{{pack}}'

# Compile and validate without installing or opening windows.
desktop-build pack="content-packs/core-modular.crystalpack":
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Build -Pack" } else { "python3 tools/desktop.py build --pack" }}} '{{pack}}'

# Build a DMG (Mac), ZIP (Windows), or tar.gz (Linux) in .local/desktop-dist.
desktop-package pack="content-packs/core-modular.crystalpack":
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Package -Pack" } else { "python3 tools/desktop.py package --pack" }}} '{{pack}}'

# Repackage the last successful desktop build; do not invoke Cargo.
desktop-repack pack="content-packs/core-modular.crystalpack":
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Package -SkipBuild -Pack" } else { "python3 tools/desktop.py package --skip-build --pack" }}} '{{pack}}'

# Install the last successful desktop build without recompiling.
desktop-install pack="content-packs/core-modular.crystalpack":
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Install -SkipBuild -Pack" } else { "python3 tools/desktop.py install --skip-build --pack" }}} '{{pack}}'

# Open the installed desktop copy immediately; never rebuild.
desktop-run:
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Run" } else { "python3 tools/desktop.py run" }}}

# Validate the last build and supplied content without opening a window.
desktop-check pack="content-packs/core-modular.crystalpack":
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-desktop.ps1 -Action Check -SkipBuild -Pack" } else { "python3 tools/desktop.py check --pack" }}} '{{pack}}'

# Developer ID-signed, notarized DMG; requires configured keychain credentials.
[macos]
desktop-release pack="content-packs/core-modular.crystalpack":
    python3 tools/desktop.py package --notarize --pack '{{pack}}'

# Initialize external game content once on any supported host.
desktop-content:
    {{if os() == "windows" { "powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-crystalpack.ps1" } else { "bash tools/build-crystalpack.sh" }}}

# Quickly serve the prepared local browser copy; Ctrl-C stops the server.
serve port="3003" runtime=".local/desktop-3d-runtime":
    @echo 'Open http://localhost:{{port}}/?multiplayer=off'
    CRYSTAL_DATA_DIR='{{runtime}}/data' '{{runtime}}/crystal-web-server' --dir '{{runtime}}/web' --host 127.0.0.1 --port '{{port}}'

# Launch the prepared native 3D client in the bedroom, without the intro.
play-3d runtime=".local/desktop-3d-runtime":
    CRYSTAL_CLUSTERING="${CRYSTAL_CLUSTERING:-cpu}" '{{runtime}}/new_bark_3d' content-packs/core-modular.browser.crystalpack --map PlayersHouse2F --x 3 --y 3 --save-path content-packs/desktop-3d-test.crystalsave

# Capture every render stage and regional load while playing the native client.
trace-3d trace=".local/loading-trace.json" runtime=".local/desktop-3d-runtime":
    CRYSTAL_CLUSTERING="${CRYSTAL_CLUSTERING:-cpu}" CRYSTAL_LOADING_TRACE='{{trace}}' '{{runtime}}/new_bark_3d' content-packs/core-modular.browser.crystalpack --map PlayersHouse2F --x 3 --y 3

# Agent iteration: build once after Rust edits, then use the *-reuse recipes.
# Reuse intentionally runs the existing binary, even if source has changed.
# Use a distinct CARGO_TARGET_DIR per concurrent builder; share completed binaries.
# No reuse recipe invokes Cargo, refreshes packs, or opens a visible window.

# List completed native examples/test executables across agent caches (newest first).
iteration-artifacts root="target":
    #!/usr/bin/env python3
    import os
    from pathlib import Path
    root = Path({{quote(root)}})
    candidates = [p for p in root.glob('**/*') if p.is_file() and os.access(p, os.X_OK) and p.parent.name in ('examples', 'deps') and '.' not in p.name]
    for p in sorted(candidates, key=lambda p: p.stat().st_mtime, reverse=True):
        print(f'{p.stat().st_mtime:.0f}  {p}')

# Compile renderer once in this agent's cache; subsequent captures use render-reuse.
iteration-build-render target="target/agent-iteration":
    CARGO_TARGET_DIR='{{target}}' cargo build --locked -p crystal-bevy --features location-tester --example render_at_location

# Compile a crate's library tests once; use iteration-artifacts to find the executable.
iteration-build-tests crate features="" target="target/agent-iteration":
    CARGO_TARGET_DIR='{{target}}' cargo test --locked -p '{{crate}}' --lib --features '{{features}}' --no-run

# Run an explicit compiled libtest executable. Pass filters/--exact/--list as needed.
# Example: just test-reuse target/my-agent/debug/deps/crystal_voxel_view-HASH filter
# This verifies the selected build only; rebuild explicitly to validate Rust edits.
[positional-arguments]
test-reuse binary *args:
    #!/usr/bin/env python3
    import os, sys
    from pathlib import Path
    binary = Path({{quote(binary)}}).resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        sys.exit(f'Missing executable: {binary}. Run iteration-build-tests first.')
    os.environ.setdefault('CRYSTAL_RENDER_TEST_PACK', str(Path('content-packs/text-tui.crystalpack').resolve()))
    os.environ['CRYSTAL_RENDER_TEST_HEADLESS'] = '1'
    os.environ['CRYSTAL_RENDER_TEST_CAPTURE_TARGET'] = 'image'
    print(f'Reusing {binary} (existing build; no source freshness check)', flush=True)
    os.execv(str(binary), [str(binary), *sys.argv[2:]])

# Capture matched 2D/3D views with a completed renderer, bypassing Cargo entirely.
# Set binary to another agent's completed example to share its build.
render-reuse map x y output binary="target/agent-iteration/debug/examples/render_at_location" pack="content-packs/core-modular.browser.crystalpack" hour="12":
    #!/usr/bin/env python3
    import os, sys
    from pathlib import Path
    binary = Path({{quote(binary)}}).resolve()
    pack = Path({{quote(pack)}}).resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        sys.exit(f'Missing renderer: {binary}. Run iteration-build-render first.')
    if not pack.is_file():
        sys.exit(f'Missing external pack: {pack}. Run just crystalpack first.')
    Path({{quote(output)}}).parent.mkdir(parents=True, exist_ok=True)
    os.environ['CRYSTAL_RENDER_TEST_HEADLESS'] = '1'
    os.environ['CRYSTAL_RENDER_TEST_CAPTURE_TARGET'] = 'image'
    print(f'Reusing {binary} (existing build; no source freshness check)', flush=True)
    os.execv(str(binary), [str(binary), '--pack', str(pack), '--map', {{quote(map)}}, '--x', {{quote(x)}}, '--y', {{quote(y)}}, '--view', 'both', '--hour', {{quote(hour)}}, '--screenshot', {{quote(output)}}])

# Partial builds retain Cargo's freshness checks. Reuse the SAME agent cache,
# profile and feature set across iterations; changing them can rebuild dependencies.
# Cargo recompiles changed crates and affected dependents, not individual Rust files.
# These recipes preserve the workspace profile (incremental=false) to avoid silently
# invalidating existing caches. Opt in with CARGO_INCREMENTAL=1 for your own cache.

# Type-check only the selected library, without linking. Extra Cargo flags follow.
[positional-arguments]
partial-check crate target *args:
    #!/usr/bin/env python3
    import os, sys
    os.environ['CARGO_TARGET_DIR'] = sys.argv[2]
    os.execvp('cargo', ['cargo', 'check', '--locked', '-p', sys.argv[1], '--lib', *sys.argv[3:]])

# Build only a selected package's library; dependencies are reused when fresh.
[positional-arguments]
partial-build crate target *args:
    #!/usr/bin/env python3
    import os, sys
    os.environ['CARGO_TARGET_DIR'] = sys.argv[2]
    os.execvp('cargo', ['cargo', 'build', '--locked', '-p', sys.argv[1], '--lib', *sys.argv[3:]])

# Build/link only the named example, then use render-reuse for repeated captures.
[positional-arguments]
partial-example crate example target *args:
    #!/usr/bin/env python3
    import os, sys
    os.environ['CARGO_TARGET_DIR'] = sys.argv[3]
    os.execvp('cargo', ['cargo', 'build', '--locked', '-p', sys.argv[1], '--example', sys.argv[2], *sys.argv[4:]])

# Rebuild the selected library test harness as needed, then run matching tests.
# A filter reduces test execution, not compilation within that library harness.
[positional-arguments]
partial-test crate filter target *args:
    #!/usr/bin/env python3
    import os, sys
    from pathlib import Path
    os.environ['CARGO_TARGET_DIR'] = sys.argv[3]
    os.environ.setdefault('CRYSTAL_RENDER_TEST_PACK', str(Path('content-packs/text-tui.crystalpack').resolve()))
    os.environ['CRYSTAL_RENDER_TEST_HEADLESS'] = '1'
    os.environ['CRYSTAL_RENDER_TEST_CAPTURE_TARGET'] = 'image'
    os.execvp('cargo', ['cargo', 'test', '--locked', '-p', sys.argv[1], '--lib', sys.argv[2], *sys.argv[4:]])
