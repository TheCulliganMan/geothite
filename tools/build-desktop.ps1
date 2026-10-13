[CmdletBinding()]
param(
    [string]$Pack = 'content-packs/core-modular.crystalpack',
    [string]$InstallDirectory = '',
    [ValidateRange(1, 64)]
    [int]$Jobs = 2,
    [switch]$Launch
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
# App-data writes can be virtualized by packaged launchers. A user-owned Games
# directory remains accessible when Explorer launches the desktop shortcut.
if (-not $InstallDirectory) { $InstallDirectory = Join-Path $env:USERPROFILE 'Games/Geothite' }
$InstallDirectory = [IO.Path]::GetFullPath($InstallDirectory)
if (-not [IO.Path]::IsPathRooted($Pack)) { $Pack = Join-Path $repo $Pack }
if (-not (Test-Path -LiteralPath $Pack -PathType Leaf)) {
    throw 'Native content pack is missing. Run just desktop-content, then just desktop. To use another completed native pack, pass -Pack to this script.'
}
$packPath = (Resolve-Path -LiteralPath $Pack).Path
$buildDirectory = Join-Path $repo 'target/desktop-build'
$oldTarget = $env:CARGO_TARGET_DIR
Push-Location $repo
try {
    $env:CARGO_TARGET_DIR = $buildDirectory
    New-Item -ItemType Directory -Force -Path $buildDirectory | Out-Null
    & cargo run --locked --release --jobs $Jobs -p crystal-assets --bin verify_crystalpack -- $packPath --desktop
    if ($LASTEXITCODE -ne 0) { throw 'Content validation failed; desktop installation was not changed.' }

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
    & cargo rustc --locked --release --jobs $Jobs -p crystal-bevy --bin crystal-bevy --features voxel-view,desktop-app -- -C "link-arg=$resourcePath"
    if ($LASTEXITCODE -ne 0) { throw 'Desktop executable build failed; installation was not changed.' }

    New-Item -ItemType Directory -Force -Path $InstallDirectory | Out-Null
    $executable = Join-Path $InstallDirectory 'Geothite.exe'
    Copy-Item -LiteralPath (Join-Path $buildDirectory 'release/crystal-bevy.exe') -Destination $executable -Force
    Copy-Item -LiteralPath $packPath -Destination (Join-Path $InstallDirectory 'core-modular.crystalpack') -Force
    Copy-Item -LiteralPath $iconPath -Destination (Join-Path $InstallDirectory 'geothite.ico') -Force
    $installedAssets = Join-Path $InstallDirectory 'assets'
    New-Item -ItemType Directory -Force -Path $installedAssets | Out-Null
    Copy-Item -LiteralPath (Join-Path $repo 'assets/open-models') -Destination $installedAssets -Recurse -Force
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
    $env:CARGO_TARGET_DIR = $oldTarget
    Pop-Location
}
