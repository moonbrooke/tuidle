param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

Set-Location -Path $PSScriptRoot

function Get-OutputPath([string]$target) {
    if ($target -like "*windows*") { return "build/windows/tuidle.exe" }
    if ($target -like "*darwin*") { return "build/macos/tuidle" }
    if ($target -like "*linux*") { return "build/linux/tuidle" }
    return "build/$target/tuidle"
}

$hostTarget = ((rustc -vV) | Select-String '^host:').ToString().Split(':', 2)[1].Trim()

$targets = @(
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin"
)

if (-not $All) {
    $targets = @($hostTarget)
}

$installed = @(rustup target list --installed)

foreach ($target in $targets) {
    if ($installed -notcontains $target) {
        Write-Host "Installing Rust target $target..."
        rustup target add $target
    }

    $output = Get-OutputPath $target
    $outputDir = Split-Path -Parent $output
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

    Write-Host "Building $target -> $output"
    cargo build --release --target $target

    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed for $target"
    }

    $binary = if ($target -like "*windows*") { "tuidle.exe" } else { "tuidle" }
    Copy-Item -Force -Path "target/$target/release/$binary" -Destination $output
}

Write-Host "All builds complete."
