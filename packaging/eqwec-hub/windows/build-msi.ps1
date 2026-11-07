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

    if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
        dotnet tool install --global wix
        $env:PATH = "$env:USERPROFILE\.dotnet\tools;$env:PATH"
    }

    $dist = Join-Path $root "dist"
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    $msi = Join-Path $dist "EQwec-Hub-$version-windows-$arch.msi"

    wix build (Join-Path $PSScriptRoot "eqwec-hub.wxs") `
        -arch $arch `
        -d ProductVersion=$msiVersion `
        -d BinaryPath=(Join-Path $root "target/release/eqwec-hub.exe") `
        -o $msi

    Write-Output "MSI: $msi"
} finally {
    Pop-Location
}
