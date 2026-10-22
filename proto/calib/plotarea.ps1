param([Parameter(Mandatory)][string]$Pptx)
$Pptx = (Resolve-Path $Pptx).Path
$mutex = New-Object System.Threading.Mutex($false, "Global\slideml-powerpoint")
try { [void]$mutex.WaitOne() } catch [System.Threading.AbandonedMutexException] { }
$pres = $null
try {
  $was = [bool](Get-Process POWERPNT -ErrorAction SilentlyContinue)
  $app = New-Object -ComObject PowerPoint.Application
  $pres = $app.Presentations.Open($Pptx, -1, 0, 0)
  foreach ($s in $pres.Slides) { foreach ($shp in $s.Shapes) { if ($shp.HasChart) {
    $pa = $shp.Chart.PlotArea
    "{0}`t{1:F1}`t{2:F1}`t{3:F1}`t{4:F1}" -f $shp.Name, ($shp.Left + $pa.InsideLeft), ($shp.Top + $pa.InsideTop), $pa.InsideWidth, $pa.InsideHeight
  } } }
} finally { if ($pres) { $pres.Close() }; if (-not $was -and $app.Presentations.Count -eq 0) { $app.Quit() }; $mutex.ReleaseMutex() }
