#!/bin/sh
set -e
/f/pptx-ai-native/proto/target/release/slideml.exe build F:/pptx-ai-native/examples/aurora/deck.xml -o F:/pptx-ai-native/out/aurora.pptx
dotnet F:/pptx-ai-native/proto/tools/validate/bin/Release/net8.0/validate.dll F:/pptx-ai-native/out/aurora.pptx | tail -5
powershell -NoProfile -File F:/pptx-ai-native/.claude/skills/slideml/scripts/render.ps1 -Pptx F:/pptx-ai-native/out/aurora.pptx -OutDir F:/pptx-ai-native/out/aurora
