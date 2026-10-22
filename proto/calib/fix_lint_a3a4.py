"""1) manifest `filled` = shape has a fill (with or without text); 2) check.py counts diagrams;
3) archetypes A3 and A4 get proportional bars, so the number is shown, not only written."""
from pathlib import Path

root = Path("F:/pptx-ai-native")


def patch(path, pairs):
    p = root / path
    s = p.read_text(encoding="utf-8")
    for a, b in pairs:
        assert a in s, (path, a[:80])
        s = s.replace(a, b, 1)
    p.write_text(s, encoding="utf-8")


patch("proto/src/deck.rs", [
    ('filled: !has_text && node.attribute("fill").unwrap_or(if is_text { "none" } else { "accent1" }) != "none",',
     'filled: node.attribute("fill").unwrap_or(if is_text { "none" } else { "accent1" }) != "none",'),
])
patch("proto/src/lint.rs", [("    /// A filled shape without text (decoration or a data mark).", "    /// The shape has a fill (data mark, panel, or decoration).")])
patch("proto/tools/check.py", [
    ('''        if i["tag"] == "shape" and i["filled"] and not big_bg(i) and area(i) > 0.10 * W * H:''',
     '''        if i["tag"] == "shape" and i["filled"] and "lines" not in i and not big_bg(i) and area(i) > 0.10 * W * H:'''),
    ('''    has_visual = any(i["tag"] in visual_tags for i in items) or any(
        i["tag"] == "shape" and i["filled"] and not big_bg(i) and area(i) > 0.03 * W * H for i in items)''',
     '''    data_lines = [i for i in items if i["tag"] == "line" and not i["id"].startswith("chrome")]
    has_visual = (any(i["tag"] in visual_tags for i in items)
                  or any(i["tag"] == "shape" and i["filled"] and not big_bg(i) and area(i) > 0.03 * W * H for i in items)
                  or len(data_lines) >= 2)'''),
])

g = root / "proto/calib/gallery_slides.xml"
s = g.read_text(encoding="utf-8")
# A3: replace the right-hand text column with comparison bars (Q2 2026 vs the whole of 2025).
old_a3 = s[s.index('    <text id="a3-base"'):s.index('    <line id="a3-rule"')]
new_a3 = '''    <!-- Bars: 272 pt = the larger value. 2025: 272 × 10 / 12 = 227. -->
    <text id="a3-b1-l" x="640" y="176" w="272" h="auto" class="label" color="accent1">Q2 2026 · 12 МВт</text>
    <shape id="a3-b1" x="640" y="196" w="272" h="28" fill="accent1" line="none"/>
    <text id="a3-b2-l" x="640" y="240" w="272" h="auto" class="label">Весь 2025 год · 10 МВт</text>
    <shape id="a3-b2" x="640" y="260" w="227" h="28" fill="lt2" line="none"/>
    <text id="a3-why" x="640" y="304" w="272" h="auto" class="body">За квартал — больше, чем за&#160;прошлый год.</text>
'''
s = s.replace(old_a3, new_a3)
# A4: proportional bars under the numbers (864 pt = the larger value; 95 / 140 × 864 = 586).
patch_pairs = [
    ('<text id="a4-delta" x="48" y="352"', '<text id="a4-delta" x="48" y="376"'),
    ('<text id="a4-why" x="48" y="392"', '<text id="a4-why" x="48" y="412"'),
]
for a, b in patch_pairs:
    assert a in s, a
    s = s.replace(a, b)
anchor = '    <text id="a4-delta"'
s = s.replace(anchor, '''    <!-- Bars to scale: 864 pt = the larger value; before = 864 × 95 / 140 = 586. -->
    <shape id="a4-bar-before" x="48" y="318" w="586" h="14" fill="lt2" line="none"/>
    <shape id="a4-bar-after" x="48" y="340" w="864" h="14" fill="accent1" line="none"/>
''' + anchor, 1)
g.write_text(s, encoding="utf-8")
st = (root / ".claude/skills/slideml/starter.xml").read_text(encoding="utf-8")
(root / "examples/archetypes/deck.xml").write_text(st.replace("  <!-- Slides go here.", s + "\n  <!-- Slides go here."), encoding="utf-8")
print("ok")
