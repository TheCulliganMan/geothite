[CmdletBinding()]
param(
    [ValidateSet('Build', 'Install', 'Package', 'Run', 'Check')]
    [string]$Action = 'Install',
    [string]$Pack = 'content-packs/core-modular.crystalpack',
    [string]$InstallDirectory = '',
    [ValidateSet('dev', 'wasm-dev', 'release')]
    [string]$Profile = $(if ($env:GEOTHITE_DESKTOP_PROFILE) { $env:GEOTHITE_DESKTOP_PROFILE } else { 'release' }),
    [ValidateRange(1, 64)]
    [int]$Jobs = 2,
    [switch]$Launch,
    [switch]$SkipBuild
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
# App-data writes can be virtualized by packaged launchers. A user-owned Games
# directory remains accessible when Explorer launches the desktop shortcut.
if (-not $InstallDirectory) { $InstallDirectory = Join-Path $env:USERPROFILE 'Games/Geothite' }
$InstallDirectory = [IO.Path]::GetFullPath($InstallDirectory)
if ($Action -eq 'Run') {
    $executable = Join-Path $InstallDirectory 'Geothite.exe'
    if (-not (Test-Path -LiteralPath $executable)) { throw 'Run just desktop first.' }
    Start-Process -FilePath $executable -WorkingDirectory $InstallDirectory
    return
}
if (-not [IO.Path]::IsPathRooted($Pack)) { $Pack = Join-Path $repo $Pack }
if (-not (Test-Path -LiteralPath $Pack -PathType Leaf)) {
    throw 'Native content pack is missing. Run just desktop-content, then just desktop. To use another completed native pack, pass -Pack to this script.'
}
$packPath = (Resolve-Path -LiteralPath $Pack).Path
$buildDirectory = if ($env:CARGO_TARGET_DIR) { [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR) } else { Join-Path $repo 'target/desktop-build' }
$binaryProfile = if ($Profile -eq 'dev') { 'debug' } else { $Profile }
$binaryDirectory = Join-Path $buildDirectory $binaryProfile
$optimization = @()
if ($Profile -ne 'release') {
    foreach ($crate in @('crystal-core', 'crystal-assets', 'crystal-runtime', 'crystal-bevy', 'crystal-voxel-view', 'crystal-net')) {
        $optimization += @('--config', "profile.$Profile.package.$crate.opt-level=1", '--config', "profile.$Profile.package.$crate.incremental=true")
    }
}
$packageStage = ''
$oldTarget = $env:CARGO_TARGET_DIR
Push-Location $repo
try {
    $env:CARGO_TARGET_DIR = $buildDirectory
    New-Item -ItemType Directory -Force -Path $buildDirectory | Out-Null
    if (-not $SkipBuild -and $Action -ne 'Check') {
        & cargo build --locked --profile $Profile --jobs $Jobs @optimization -p crystal-assets -p crystal-bevy -p crystal-voxel-view --features crystal-bevy/voxel-view,crystal-bevy/desktop-app --bin verify_crystalpack --bin crystal-bevy --bin bundle_open_models --bin bundle_open_people
        if ($LASTEXITCODE -ne 0) { throw 'Validator build failed.' }
    }
    & (Join-Path $binaryDirectory 'verify_crystalpack.exe') $packPath --desktop
    if ($LASTEXITCODE -ne 0) { throw 'Content validation failed; desktop installation was not changed.' }
    $verification = Join-Path $buildDirectory ('verify-' + [Guid]::NewGuid().ToString())
    try {
        & (Join-Path $binaryDirectory 'bundle_open_models.exe') (Join-Path $repo 'assets/open-models') (Join-Path $verification 'scenery.json')
        if ($LASTEXITCODE -ne 0) { throw 'Scenery validation failed.' }
        & (Join-Path $binaryDirectory 'bundle_open_people.exe') (Join-Path $repo 'assets/open-models') (Join-Path $verification 'people.json')
        if ($LASTEXITCODE -ne 0) { throw 'Character artwork validation failed.' }
    } finally {
        if (Test-Path -LiteralPath $verification) { Remove-Item -LiteralPath $verification -Recurse -Force }
    }
    if ($Action -eq 'Check') { return }

    if (-not $SkipBuild) {
        # Draw an original crystal icon and write an ICO with a PNG image entry.
        Add-Type -AssemblyName System.Drawing
        $bitmap = New-Object Drawing.Bitmap 256,256
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        $graphics.SmoothingMode = 'AntiAlias'
        $graphics.Clear([Drawing.Color]::FromArgb(255,24,31,49))
        $points = [Drawing.Point[]]@([Drawing.Point]::new(128,22), [Drawing.Point]::new(207,91), [Drawing.Point]::new(181,183), [Drawing.Point]::new(128,235), [Drawing.Point]::new(49,164), [Drawing.Point]::new(53,83))
        $brush = [Drawing.SolidBrush]::new([Drawing.Color]::FromArgb(255,76,205,223))
        $pen = [Drawing.Pen]::new([Drawing.Color]::FromArgb(255,205,251,255),7)
        $graphics.FillPolygon($brush,$points)
        $graphics.DrawPolygon($pen,$points)
        $graphics.DrawLine($pen,128,22,105,124)
        $graphics.DrawLine($pen,105,124,128,235)
        $graphics.DrawLine($pen,53,83,105,124)
        $graphics.DrawLine($pen,105,124,207,91)
        $stream = [IO.MemoryStream]::new()
        $bitmap.Save($stream,[Drawing.Imaging.ImageFormat]::Png)
        $bytes = $stream.ToArray()
        $iconPath = Join-Path $buildDirectory 'geothite.ico'
        $writer = [IO.BinaryWriter]::new([IO.File]::Create($iconPath))
        try {
            $writer.Write([uint16]0); $writer.Write([uint16]1); $writer.Write([uint16]1)
            $writer.Write([byte]0); $writer.Write([byte]0); $writer.Write([byte]0); $writer.Write([byte]0)
            $writer.Write([uint16]1); $writer.Write([uint16]32)
            $writer.Write([uint32]$bytes.Length); $writer.Write([uint32]22); $writer.Write($bytes)
        } finally { $writer.Dispose(); $stream.Dispose(); $pen.Dispose(); $brush.Dispose(); $graphics.Dispose(); $bitmap.Dispose() }

        $resourceCompiler = Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/*/x64/rc.exe" -ErrorAction SilentlyContinue |
            Sort-Object FullName -Descending | Select-Object -First 1
        if (-not $resourceCompiler) { throw 'Install the Windows SDK (Desktop development with C++ in Visual Studio) to embed the executable icon.' }
        $resourcePath = Join-Path $buildDirectory 'geothite.res'
        $rcPath = Join-Path $buildDirectory 'geothite.rc'
        Set-Content -LiteralPath $rcPath -Encoding ascii -Value ('1 ICON "' + $iconPath.Replace('\','/') + '"')
        & $resourceCompiler.FullName /nologo /fo $resourcePath $rcPath
        if ($LASTEXITCODE -ne 0) { throw 'Icon resource compilation failed.' }
        & cargo rustc --locked --profile $Profile --jobs $Jobs @optimization -p crystal-bevy --bin crystal-bevy --features voxel-view,desktop-app -- -C "link-arg=$resourcePath"
        if ($LASTEXITCODE -ne 0) { throw 'Desktop executable build failed; installation was not changed.' }
    }
    $iconPath = Join-Path $buildDirectory 'geothite.ico'
    if ($Action -eq 'Build') { Write-Host "Desktop build verified: $binaryDirectory"; return }
    if ($Action -eq 'Package') {
        $distribution = Join-Path $repo '.local/desktop-dist'
        New-Item -ItemType Directory -Force -Path $distribution | Out-Null
        $packageStage = Join-Path $distribution ('.windows-stage-' + [Guid]::NewGuid().ToString())
        $InstallDirectory = Join-Path $packageStage 'Geothite'
    }

    New-Item -ItemType Directory -Force -Path $InstallDirectory | Out-Null
    $executable = Join-Path $InstallDirectory 'Geothite.exe'
    Copy-Item -LiteralPath (Join-Path $binaryDirectory 'crystal-bevy.exe') -Destination $executable -Force
    Copy-Item -LiteralPath $packPath -Destination (Join-Path $InstallDirectory 'core-modular.crystalpack') -Force
    Copy-Item -LiteralPath $iconPath -Destination (Join-Path $InstallDirectory 'geothite.ico') -Force
    Copy-Item -LiteralPath (Join-Path $repo 'LICENSE') -Destination $InstallDirectory -Force
    $installedAssets = Join-Path $InstallDirectory 'assets'
    New-Item -ItemType Directory -Force -Path $installedAssets | Out-Null
    Copy-Item -LiteralPath (Join-Path $repo 'assets/open-models') -Destination $installedAssets -Recurse -Force
    & $executable --check-installation
    if ($LASTEXITCODE -ne 0) { throw 'Packaged installation check failed.' }
    if ($Action -eq 'Package') {
        $versionLine = Select-String -LiteralPath (Join-Path $repo 'crates/crystal-bevy/Cargo.toml') -Pattern '^version = "([^"]+)"' | Select-Object -First 1
        $version = $versionLine.Matches[0].Groups[1].Value
        $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
        $archive = Join-Path $distribution "Geothite-$version-windows-$architecture.zip"
        $pending = Join-Path $packageStage 'package.zip'
        Compress-Archive -LiteralPath $InstallDirectory -DestinationPath $pending -CompressionLevel Fastest
        Move-Item -LiteralPath $pending -Destination $archive -Force
        $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        Set-Content -LiteralPath "$archive.sha256" -Encoding ascii -Value "$hash  $([IO.Path]::GetFileName($archive))"
        Write-Host "Created $archive"
        return
    }
    $desktop = [Environment]::GetFolderPath('Desktop')
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut((Join-Path $desktop 'Geothite.lnk'))
    $shortcut.TargetPath = $executable
    $shortcut.Arguments = ''
    $shortcut.WorkingDirectory = $InstallDirectory
    $shortcut.WindowStyle = 1
    $shortcut.IconLocation = "$executable,0"
    $shortcut.Description = 'Play Geothite'
    $shortcut.Save()
    Write-Host "Installed $executable and $desktop/Geothite.lnk"
    if ($Launch) { Start-Process -FilePath $executable -WorkingDirectory $InstallDirectory }
} finally {
    if ($packageStage -and (Test-Path -LiteralPath $packageStage)) { Remove-Item -LiteralPath $packageStage -Recurse -Force }
    $env:CARGO_TARGET_DIR = $oldTarget
    Pop-Location
}
