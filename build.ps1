$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

Set-Location -Path $PSScriptRoot

$targets = @(
    @{ GOOS = "windows"; GOARCH = "amd64"; Output = "build/windows/tuidle.exe" },
    @{ GOOS = "linux";   GOARCH = "amd64"; Output = "build/linux/tuidle" },
    @{ GOOS = "darwin";  GOARCH = "arm64"; Output = "build/macos/tuidle" }
)

foreach ($target in $targets) {
    $env:GOOS = $target.GOOS
    $env:GOARCH = $target.GOARCH

    $outputDir = Split-Path -Parent $target.Output
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

    Write-Host "Building $($target.GOOS)/$($target.GOARCH) -> $($target.Output)"
    go build -o $target.Output .

    if ($LASTEXITCODE -ne 0) {
        throw "go build failed for $($target.GOOS)/$($target.GOARCH)"
    }
}

Write-Host "All builds complete."
