set -e
cd F:/pptx-ai-native
proto/target/release/slideml.exe build examples/dashboard/deck.xml -o out/dashboard.pptx
dotnet proto/tools/validate/bin/Release/net8.0/validate.dll out/dashboard.pptx | tail -5
rm -f out/dashboard/*.png
powershell -NoProfile -File .claude/skills/slideml/scripts/render.ps1 -Pptx 'F:\pptx-ai-native\out\dashboard.pptx' -OutDir 'F:\pptx-ai-native\out\dashboard'
