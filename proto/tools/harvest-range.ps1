param([int]$From, [int]$To, [string]$Out)
$wasRunning = [bool](Get-Process POWERPNT -ErrorAction SilentlyContinue)
$app = New-Object -ComObject PowerPoint.Application
$pres = $app.Presentations.Add(0); $slide = $pres.Slides.Add(1, 12)
for ($e = $From; $e -le $To; $e++) {
  $shp = $slide.Shapes.AddShape(1, 10, 10, 50, 50); $shp.Name = "e$e"
  try { [void]$slide.TimeLine.MainSequence.AddEffect($shp, $e, 0, 1) } catch { $shp.Delete() }
}
$pres.SaveAs($Out); $pres.Close(); if (-not $wasRunning) { $app.Quit() }
