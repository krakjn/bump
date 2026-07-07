# Bump installer — Windows (PowerShell 5.1+ / 7+)
param([string]$InstallDir = '')
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = 'krakjn/bump'

# Add token if available to avoid rate limits
$headers = @{ 'User-Agent' = 'bump-installer'; 'Accept' = 'application/vnd.github+json' }
if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $($env:GITHUB_TOKEN)" }
elseif ($env:GH_TOKEN) { $headers['Authorization'] = "Bearer $($env:GH_TOKEN)" }

function Die($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }
function Info($msg) { Write-Host "[INFO] $msg" }

$arch = switch ($env:PROCESSOR_ARCHITECTURE) { 
  'AMD64' { 'amd64' } 
  'ARM64' { 'arm64' } 
  default { $null } 
}
if (-not $arch) { Die "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
Info "Platform: Windows ($arch)"

# Resolve install target — update in place if bump already exists
$existing = Get-Command bump -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($existing -and $existing.Source -and (Test-Path $existing.Source)) {
  $targetPath = [IO.Path]::GetFullPath($existing.Source)
  $InstallDir = Split-Path $targetPath
  Info "Updating existing bump at $targetPath"
} else {
  if (-not $InstallDir) {
    $InstallDir = if ($env:RUNNER_TEMP) { Join-Path $env:RUNNER_TEMP 'bump' } else { Join-Path $env:LOCALAPPDATA 'Programs\bump' }
  }
  $InstallDir = [IO.Path]::GetFullPath($InstallDir)
  $targetPath = Join-Path $InstallDir 'bump.exe'
}

# Fetch latest release tag
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers $headers
if (-not $release.tag_name) { Die 'Could not resolve latest release.' }
$tag = $release.tag_name.Trim()
Info "Latest release: $tag"

# Download
$url = "https://github.com/$repo/releases/download/$tag/bump-windows-$arch.exe"
Info "Downloading: $url"
$tmp = Join-Path ([IO.Path]::GetTempPath()) "bump-$([Guid]::NewGuid().ToString('N')).exe"
Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing -Headers $headers
if (-not (Test-Path $tmp) -or (Get-Item $tmp).Length -eq 0) { Die 'Download failed or empty.' }

# Install
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null }
Move-Item $tmp $targetPath -Force
Write-Host "[SUCCESS] Installed: $targetPath" -ForegroundColor Green

# PATH registration
if ($env:GITHUB_PATH) {
  $InstallDir | Out-File $env:GITHUB_PATH -Encoding utf8 -Append
} else {
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $escaped = [regex]::Escape($InstallDir.TrimEnd('\'))
  if (-not $userPath -or $userPath -notmatch "(?i)(^|;)$escaped(;|$)") {
    $newPath = if ($userPath) { "$userPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Info "Added to user PATH: $InstallDir"
  }
}
$env:Path += ";$InstallDir"

# Verify
try { Info (& $targetPath --version 2>&1) } catch { }
