#!/bin/sh
# Build + validate + render + overlay the hero deck.
set -e
O=F:/pptx-ai-native/out
F:/pptx-ai-native/proto/target/release/slideml.exe build F:/pptx-ai-native/examples/hero/deck.xml -o $O/hero.pptx
dotnet F:/pptx-ai-native/proto/tools/validate/bin/Release/net8.0/validate.dll $O/hero.pptx
[ "$1" = "norender" ] && exit 0
powershell -NoProfile -File F:/pptx-ai-native/.claude/skills/slideml/scripts/render.ps1 -Pptx $O/hero.pptx -OutDir $O/hero
python F:/pptx-ai-native/.claude/skills/slideml/scripts/overlay.py $O/hero.manifest.json $O/hero
