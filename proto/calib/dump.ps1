# Dumps PowerPoint's own text metrics for every text shape: name, line count, bound width/height.
param([Parameter(Mandatory)][string]$Pptx)
$Pptx = (Resolve-Path $Pptx).Path
$mutex = New-Object System.Threading.Mutex($false, "Global\slideml-powerpoint")
try { [void]$mutex.WaitOne() } catch [System.Threading.AbandonedMutexException] { }
$pres = $null
try {
  $was = [bool](Get-Process POWERPNT -ErrorAction SilentlyContinue)
  $app = New-Object -ComObject PowerPoint.Application
  $pres = $app.Presentations.Open($Pptx, -1, 0, 0)
  function Dump($shp, $slide) {
    if ($shp.Type -eq 6) { foreach ($g in $shp.GroupItems) { Dump $g $slide }; return }
    if (-not $shp.HasTextFrame) { return }
    if (-not $shp.TextFrame2.HasText) { return }
    $tr = $shp.TextFrame2.TextRange
    "{0}`t{1}`t{2}`t{3:F2}`t{4:F2}" -f $slide, $shp.Name, $tr.Lines().Count, $tr.BoundWidth, $tr.BoundHeight
  }
  foreach ($s in $pres.Slides) { foreach ($shp in $s.Shapes) { Dump $shp $s.SlideIndex } }
} finally {
  if ($pres) { $pres.Close() }
  if (-not $was -and $app.Presentations.Count -eq 0) { $app.Quit() }
  $mutex.ReleaseMutex()
}
