# tezt installer for Windows. Downloads the prebuilt binary from the latest
# GitHub Release and installs it onto your PATH.
#
#   irm https://raw.githubusercontent.com/BilagoNet/tezt/main/install.ps1 | iex
#
# Environment:
#   TEZT_INSTALL_DIR   where to install (default: %LOCALAPPDATA%\tezt\bin)
#   TEZT_VERSION       a specific tag to install (default: latest)
#Requires -Version 5
$ErrorActionPreference = "Stop"

$repo = "BilagoNet/tezt"
$binDir = if ($env:TEZT_INSTALL_DIR) { $env:TEZT_INSTALL_DIR } else { "$env:LOCALAPPDATA\tezt\bin" }
# The x64 build runs on both x64 and (via emulation) arm64 Windows.
$target = "x86_64-pc-windows-msvc"
$archive = "tezt-$target.zip"

$tag = if ($env:TEZT_VERSION) { $env:TEZT_VERSION } else {
    (Invoke-RestMethod -UseBasicParsing "https://api.github.com/repos/$repo/releases/latest").tag_name
}
if (-not $tag) { throw "tezt-install: no published release found yet — build from source or 'pip install tezt'." }

$url = "https://github.com/$repo/releases/download/$tag/$archive"
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("tezt-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
    Write-Host "tezt-install: downloading $archive ($tag)"
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile (Join-Path $tmp $archive)
    Expand-Archive -Path (Join-Path $tmp $archive) -DestinationPath $tmp -Force

    $exe = Join-Path $tmp "tezt-$target\tezt.exe"
    if (-not (Test-Path $exe)) { $exe = Join-Path $tmp "tezt.exe" } # tolerate a flat layout
    if (-not (Test-Path $exe)) { throw "the archive did not contain tezt.exe" }

    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    Copy-Item $exe (Join-Path $binDir "tezt.exe") -Force
    Write-Host "tezt-install: installed to $binDir\tezt.exe"
    Write-Host "tezt-install: add $binDir to your PATH if it isn't already"
    & (Join-Path $binDir "tezt.exe") --version
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
