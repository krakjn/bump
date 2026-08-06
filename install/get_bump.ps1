# Bump installer — Windows (PowerShell 5.1+ / 7+)
#
# Usage:
#   irm https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.ps1 | iex
#   irm ... | iex; .\get_bump.ps1 -Dest 'C:\Tools'
#
param(
  [string]$Dest = "$env:LOCALAPPDATA\Programs\bump",
  [switch]$Help
)

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = 'krakjn/bump'

function Die($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }
function Info($msg) { Write-Host "[INFO] $msg" }

if ($Help) {
  Write-Host "Usage: $($MyInvocation.MyCommand.Name) [-Dest PATH] [-Help]"
  Write-Host ''
  Write-Host "  -Dest PATH   Install bump.exe to PATH (default: $Dest)"
  Write-Host ''
  Write-Host 'Example:'
  Write-Host "  irm ... | iex; .\get_bump.ps1 -Dest `"$env:USERPROFILE\bin`""
  exit 0
}

# Add token if available to avoid rate limits
$headers = @{ 'User-Agent' = 'bump-installer'; 'Accept' = 'application/vnd.github+json' }
if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $($env:GITHUB_TOKEN)" }
elseif ($env:GH_TOKEN) { $headers['Authorization'] = "Bearer $($env:GH_TOKEN)" }

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
  'AMD64' { 'amd64' }
  'ARM64' { 'arm64' }
  default { $null }
}
if (-not $arch) { Die "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
Info "Platform: Windows ($arch)"

$installDir = [IO.Path]::GetFullPath($Dest)
$targetPath = Join-Path $installDir 'bump.exe'
Info "Installing to $targetPath"

# Fetch latest release tag
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers $headers
if (-not $release.tag_name) { Die 'Could not resolve latest release.' }
$tag = $release.tag_name.Trim()
Info "Latest release: $tag"

# Download
$url = "https://github.com/$repo/releases/download/$tag/bump-windows-$arch.exe"
Info "Downloading: $url"
$tmp = Join-Path ([IO.Path]::GetTempPath()) "bump-$([Guid]::NewGuid().ToString('N')).exe"
Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
if (-not (Test-Path $tmp) -or (Get-Item $tmp).Length -eq 0) { Die 'Download failed or empty.' }

# Install
if (-not (Test-Path $installDir)) { New-Item -ItemType Directory -Path $installDir -Force | Out-Null }
Move-Item $tmp $targetPath -Force
Write-Host "[SUCCESS] Installed bump $(& $targetPath --version 2>&1) to $targetPath" -ForegroundColor Green

# PATH registration
if ($env:GITHUB_PATH) {
  $installDir | Out-File $env:GITHUB_PATH -Encoding utf8 -Append
} else {
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $escaped = [regex]::Escape($installDir.TrimEnd('\'))
  if (-not $userPath -or $userPath -notmatch "(?i)(^|;)$escaped(;|$)") {
    $newPath = if ($userPath) { "$userPath;$installDir" } else { $installDir }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Info "Added to user PATH: $installDir"
  }
}
$env:Path += ";$installDir"
