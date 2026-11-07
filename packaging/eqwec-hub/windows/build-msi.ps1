# Builds the per-user EQwec Hub MSI with WiX v4.
#
# The package installs to %LOCALAPPDATA%\Programs\EQwec Hub and never
# elevates. Requires the .NET SDK; the script installs the WiX tool if the
# `wix` command is absent.
#Requires -Version 5.1
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$root = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
$cargoManifest = Join-Path $root "apps/eqwec-hub/Cargo.toml"
$version = (Select-String -Path $cargoManifest -Pattern '^version = "(.*)"$').Matches[0].Groups[1].Value
$msiVersion = (($version -split "-")[0]) + ".0"
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }

Push-Location $root
try {
    cargo build --release --locked -p eqwec-hub
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    # WiX v5+ requires the OSMF EULA; v4 is the last freely usable line.
    $wixVersion = "4.0.6"
    $wix = Get-Command wix -ErrorAction SilentlyContinue
    $wixIsV4 = $false
    if ($wix) {
        $reported = & wix --version 2>$null
        $wixIsV4 = $LASTEXITCODE -eq 0 -and "$reported" -match '^4\.'
    }
    if (-not $wixIsV4) {
        if ($wix) {
            dotnet tool update --global wix --version $wixVersion
        } else {
            dotnet tool install --global wix --version $wixVersion
        }
        if ($LASTEXITCODE -ne 0) {
            throw "WiX $wixVersion installation failed with exit code $LASTEXITCODE"
        }
        $env:PATH = "$env:USERPROFILE\.dotnet\tools;$env:PATH"
    }

    $dist = Join-Path $root "dist"
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    $msi = Join-Path $dist "EQwec-Hub-$version-windows-$arch.msi"
    $binary = Join-Path $root "target/release/eqwec-hub.exe"

    wix build (Join-Path $PSScriptRoot "eqwec-hub.wxs") `
        -arch $arch `
        -d "ProductVersion=$msiVersion" `
        -d "BinaryPath=$binary" `
        -o $msi
    if ($LASTEXITCODE -ne 0) {
        throw "wix build failed with exit code $LASTEXITCODE"
    }

    Write-Output "MSI: $msi"
} finally {
    Pop-Location
}
