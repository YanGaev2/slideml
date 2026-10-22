# Opens a COPY of the .pptx in PowerPoint, exports slide PNGs, and reports what PowerPoint sees:
# transitions, animation effects, text overflow. Never calls Quit() if the user started PowerPoint.
param([Parameter(Mandatory)][string]$Pptx, [Parameter(Mandatory)][string]$OutDir, [int]$Width = 1920)

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $OutDir | Out-Null
# Slide.Export needs an absolute path; a relative one leaves a hidden error dialog in PowerPoint.
$OutDir = (Resolve-Path $OutDir).Path
$Pptx = (Resolve-Path $Pptx).Path
$copy = Join-Path $env:TEMP ("slideml-" + [guid]::NewGuid().ToString('N') + ".pptx")
Copy-Item $Pptx $copy

# Several agents may render at once: serialize access to the single PowerPoint instance.
$mutex = New-Object System.Threading.Mutex($false, "Global\slideml-powerpoint")
try { [void]$mutex.WaitOne() } catch [System.Threading.AbandonedMutexException] { }
$pres = $null
try {
  $wasRunning = [bool](Get-Process POWERPNT -ErrorAction SilentlyContinue)
  $app = New-Object -ComObject PowerPoint.Application
  $pres = $app.Presentations.Open($copy, -1, 0, 0)  # ReadOnly, not Untitled, no window
  $h = [int]($Width * $pres.PageSetup.SlideHeight / $pres.PageSetup.SlideWidth)
  $overflow = @()
  function Check-Shape($shp, $slideNo) {
    if ($shp.Type -eq 6) { foreach ($g in $shp.GroupItems) { Check-Shape $g $slideNo }; return }
    if (-not $shp.HasTextFrame) { return }
    $tf = $shp.TextFrame2
    if (-not $tf.HasText) { return }
    $avail = $shp.Height - $tf.MarginTop - $tf.MarginBottom
    $need = $tf.TextRange.BoundHeight
    if ($need -gt $avail + 0.5) {
      $script:overflow += "slide $slideNo '$($shp.Name)': text needs {0:N1}pt, box has {1:N1}pt" -f $need, $avail
    }
  }
  foreach ($s in $pres.Slides) {
    $n = $s.SlideIndex
    $s.Export((Join-Path $OutDir ("slide{0}.png" -f $n)), "PNG", $Width, $h)
    $t = $s.SlideShowTransition
    "slide {0}: shapes={1} transition={2} dur={3}s" -f $n, $s.Shapes.Count, $t.EntryEffect, $t.Duration
    foreach ($e in $s.TimeLine.MainSequence) {
      "   effect type={0,-3} shape={1,-14} trigger={2} delay={3}s dur={4}s" -f $e.EffectType, $e.Shape.Name, $e.Timing.TriggerType, $e.Timing.TriggerDelayTime, $e.Timing.Duration
    }
    foreach ($shp in $s.Shapes) { Check-Shape $shp $n }
  }
  if ($overflow.Count) { "OVERFLOW:"; $overflow } else { "overflow: none" }
}
finally {
  if ($pres) { $pres.Close() }
  if (-not $wasRunning -and $app.Presentations.Count -eq 0) { $app.Quit() }
  Remove-Item $copy -ErrorAction SilentlyContinue
  $mutex.ReleaseMutex()
}
