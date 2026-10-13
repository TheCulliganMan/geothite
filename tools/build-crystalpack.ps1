[CmdletBinding()]
param(
    [string]$Distribution = 'Ubuntu-24.04',
    [string]$ExportWorkspace = '',
    [ValidateRange(1, 256)]
    [int]$Jobs = 2
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repo
try {
    & git lfs install --local
    if ($LASTEXITCODE -ne 0) { throw 'Git LFS initialization failed.' }
    & git lfs pull
    if ($LASTEXITCODE -ne 0) { throw 'Git LFS asset download failed.' }
    & git lfs fsck
    if ($LASTEXITCODE -ne 0) { throw 'Git LFS asset verification failed.' }
    $linuxRepo = & wsl.exe -d $Distribution -- wslpath -u -- $repo
    if ($LASTEXITCODE -ne 0) { throw "Cannot access the checkout through WSL distribution $Distribution." }
    $arguments = @('-d', $Distribution, '--', 'env')
    foreach ($name in @('CRYSTAL_PACK_BUILD_PROFILE', 'CRYSTAL_PACK_BUILD_DIR')) {
        $value = [Environment]::GetEnvironmentVariable($name)
        if ($value) { $arguments += "$name=$value" }
    }
    $arguments += @('bash', "$($linuxRepo.Trim())/tools/build-crystalpack.sh", '--jobs', "$Jobs")
    if ($ExportWorkspace) { $arguments += @('--workspace', $ExportWorkspace) }
    & wsl.exe @arguments
    if ($LASTEXITCODE -ne 0) { throw "Crystalpack initialization failed (exit $LASTEXITCODE)." }
} finally {
    Pop-Location
}
