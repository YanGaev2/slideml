# Installs the SlideML skill for Claude Code into ~/.claude/skills/slideml:
# the skill, slideml.exe, the fonts of the style presets, and Pillow. Run it again to update.
# Usage: irm https://github.com/YanGaev2/slideml/releases/latest/download/install.ps1 | iex
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'  # the progress bar makes downloads very slow in Windows PowerShell
$dest = Join-Path $HOME '.claude\skills\slideml'

if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
  throw 'Python 3 not found. Install it (winget install Python.Python.3.12), then run this installer again.'
}

# SLIDEML_ZIP: a local zip to install instead of the latest release (for testing).
$zip = $env:SLIDEML_ZIP
if (-not $zip) {
  $zip = Join-Path $env:TEMP 'slideml-skill.zip'
  Invoke-WebRequest 'https://github.com/YanGaev2/slideml/releases/latest/download/slideml-skill.zip' -OutFile $zip -UseBasicParsing
}
if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
Expand-Archive $zip $dest
"skill: $dest"

# Fonts: install for the current user, no admin rights. slideml.exe and PowerPoint read the same registry keys.
$key = 'HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts'
$have = @()
foreach ($k in 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts', $key) {
  if (Test-Path $k) { $have += (Get-ItemProperty $k).PSObject.Properties.Name }
}
$fontDir = Join-Path $env:LOCALAPPDATA 'Microsoft\Windows\Fonts'
New-Item -ItemType Directory -Force $fontDir | Out-Null
if (-not (Test-Path $key)) { New-Item $key -Force | Out-Null }
if (-not ('SlideML.Gdi' -as [type])) {
  Add-Type -Namespace SlideML -Name Gdi -MemberDefinition '[DllImport("gdi32.dll", CharSet = CharSet.Unicode)] public static extern int AddFontResource(string file);'
}
foreach ($f in Get-ChildItem (Join-Path $dest 'fonts') | Where-Object Extension -in '.ttf', '.otf') {
  $name = ($f.BaseName -replace '-', ' ') + ' (TrueType)'
  if ($have -contains $name) { continue }
  $target = Join-Path $fontDir $f.Name
  Copy-Item $f.FullName $target -Force
  New-ItemProperty $key -Name $name -Value $target -PropertyType String -Force | Out-Null
  [void][SlideML.Gdi]::AddFontResource($target)
  "font: installed $name"
}

# Pillow draws the element overlays on the slide images.
$pil = & python -c "import importlib.util as u; print(u.find_spec('PIL') is not None)"
if ($pil -eq 'False') {
  & python -m pip install --quiet pillow
  if ($LASTEXITCODE) { throw 'pip install pillow failed' }
  'python: installed pillow'
} elseif ($pil -ne 'True') {
  throw 'Python 3 does not run (the "python" command may be the Microsoft Store stub). Install Python 3 (winget install Python.Python.3.12), then run this installer again.'
}

if (-not [type]::GetTypeFromProgID('PowerPoint.Application')) {
  Write-Warning 'PowerPoint not found. SlideML needs PowerPoint to render and check the slides.'
}
'Done. Ask Claude Code: "Make a presentation about ...". If the slideml skill does not appear, restart Claude Code.'
