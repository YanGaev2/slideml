# Creates entrance effects through PowerPoint COM and saves the file.
# The saved XML is the reference for the compiler.
# Rule: never call Quit() when the user started PowerPoint.
param([string]$Out = "F:\pptx-ai-native\proto\harvest\anim.pptx")

$wasRunning = [bool](Get-Process POWERPNT -ErrorAction SilentlyContinue)
$app = New-Object -ComObject PowerPoint.Application
$pres = $app.Presentations.Add(0)   # msoFalse: no window
$slide = $pres.Slides.Add(1, 12)    # ppLayoutBlank

# effectId, direction (0 = default), label
$specs = @(
  @(1, 0, "appear"), @(10, 0, "fade"),
  @(2, 4, "fly-left"), @(2, 3, "fly-bottom-down"), @(2, 11, "fly-bottom"),
  @(22, 4, "wipe-left"), @(22, 3, "wipe-down"), @(22, 11, "wipe-bottom"),
  @(23, 0, "zoom23"), @(53, 0, "zoom53"), @(42, 0, "float42"), @(47, 0, "float47"),
  @(55, 0, "grow55")
)
$i = 0
foreach ($s in $specs) {
  $shp = $slide.Shapes.AddShape(1, 20 + 60 * $i, 20, 50, 50)
  $shp.Name = $s[2]
  $eff = $slide.TimeLine.MainSequence.AddEffect($shp, $s[0], 0, 1)  # trigger: on click
  if ($s[1] -ne 0) { try { $eff.EffectParameters.Direction = $s[1] } catch { } }
  "{0,-16} id={1,-3} name='{2}' dir={3}" -f $s[2], $s[0], $eff.DisplayName, $eff.EffectParameters.Direction
  $i++
}
$pres.SaveAs($Out)
$pres.Close()
if (-not $wasRunning) { $app.Quit() }
