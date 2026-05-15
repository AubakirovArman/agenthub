param(
    [string]$Repo = $env:AGENTHUB_REPO,
    [string]$Version = $env:AGENTHUB_VERSION,
    [string]$InstallDir = $env:AGENTHUB_INSTALL_DIR,
    [string]$Artifact = $env:AGENTHUB_ARTIFACT
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Repo)) {
    $Repo = "AubakirovArman/agenthub"
}
if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = "latest"
}
if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = Join-Path $HOME ".agenthub\bin"
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
if ($arch -ne "X64") {
    throw "Unsupported Windows architecture: $arch"
}

$asset = "agenthub-x86_64-pc-windows-msvc.zip"
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("agenthub-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
    if ([string]::IsNullOrWhiteSpace($Artifact)) {
        $archive = Join-Path $tmp $asset
        if ($Version -eq "latest") {
            $url = "https://github.com/$Repo/releases/latest/download/$asset"
        } else {
            $url = "https://github.com/$Repo/releases/download/$Version/$asset"
        }
        Invoke-WebRequest -Uri $url -OutFile $archive
    } else {
        $archive = $Artifact
    }

    Expand-Archive -Path $archive -DestinationPath $tmp -Force
    $binary = Get-ChildItem -Path $tmp -Filter "agenthub.exe" -Recurse | Select-Object -First 1
    if ($null -eq $binary) {
        throw "Archive does not contain agenthub.exe"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $destination = Join-Path $InstallDir "agenthub.exe"
    Copy-Item -Path $binary.FullName -Destination $destination -Force

    Write-Host "agenthub installed to $destination"
    Write-Host "Add this directory to PATH if needed:"
    Write-Host "  $InstallDir"
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
