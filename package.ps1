# Builds slideml.exe, puts it into the skill folder, and packs the skill into dist/slideml-skill.zip.
# The release workflow runs this script. Run it locally to use the skill from this repository.
$skill = Join-Path $PSScriptRoot '.claude\skills\slideml'
$dist = Join-Path $PSScriptRoot 'dist'

# Cargo writes progress to stderr: set 'Stop' after it, or Windows PowerShell treats a redirected stderr as an error.
cargo build --release --manifest-path (Join-Path $PSScriptRoot 'proto\Cargo.toml')
if ($LASTEXITCODE) { throw 'cargo build failed' }
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force (Join-Path $skill 'bin'), $dist | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'proto\target\release\slideml.exe') (Join-Path $skill 'bin') -Force
Compress-Archive -Path (Join-Path $skill '*') -DestinationPath (Join-Path $dist 'slideml-skill.zip') -Force
"packed $dist\slideml-skill.zip"
