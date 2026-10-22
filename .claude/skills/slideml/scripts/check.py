"""One command for the whole SlideML loop: build -> render in PowerPoint -> overlay -> design report.

Usage: python <skill>/scripts/check.py <deck.xml> [out_name]
Writes <deck folder>/out/<out_name>.pptx and PNGs to <deck folder>/out/<out_name>/.

RESULT:
- FAIL  (exit 1): compile error, compiler warnings, PowerPoint overflow, or design lint.
- DRAFT (exit 2): technically clean, but plan.md or the design review (review.json) is missing or too weak.
- PASS  (exit 0): technically clean AND plan.md exists AND review.json has >= 2 design passes,
                  the last one scoring >= 16/20 with no zero.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

SKILL = Path(__file__).resolve().parent.parent
deck = Path(sys.argv[1]).resolve()
name = sys.argv[2] if len(sys.argv) > 2 else deck.parent.name if deck.name == "deck.xml" else deck.stem
out = deck.parent / "out"
pptx, png = out / f"{name}.pptx", out / name
exe = SKILL / "bin/slideml.exe"
if not exe.exists():
    print(f"RESULT: FAIL ({exe} not found). Install SlideML: https://github.com/YanGaev2/slideml#install")
    sys.exit(1)
out.mkdir(exist_ok=True)


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return r.returncode, (r.stdout + r.stderr).strip()


tech_ok = True
code, text = run([str(exe), "build", str(deck), "-o", str(pptx)])
print("== 1. BUILD")
print(text)
if code != 0:
    print("RESULT: FAIL (compile error). Fix the source and run again.")
    sys.exit(1)
tech_ok &= not [l for l in text.splitlines() if l.startswith(("warning:", "lint:"))]

code, text = run(["powershell", "-NoProfile", "-File", str(SKILL / "scripts/render.ps1"), "-Pptx", str(pptx), "-OutDir", str(png)])
print("== 2. RENDER (PowerPoint)")
print(text)
tech_ok &= code == 0 and "OVERFLOW" not in text

code, text = run([sys.executable, str(SKILL / "scripts/overlay.py"), str(out / f"{name}.manifest.json"), str(png)])
print("== 3. OVERLAY: written next to the PNGs (slideN.overlay.png)")

# ---- 5. design report from the geometry manifest ---------------------------------
print("== 4. DESIGN REPORT (text version of what the slides look like)")
m = json.loads((out / f"{name}.manifest.json").read_text(encoding="utf-8"))
W, H = m["w"], m["h"]
design_lints = []
src = deck.read_text(encoding="utf-8")
archetypes = re.findall(r"<!--\s*(A\d+)\b[^>]*-->\s*<slide\b", src)
photo_slides = 0
for n, s in enumerate(m["slides"], 1):
    items = s["items"]
    big_bg = lambda i: i["w"] * i["h"] >= 0.9 * W * H
    vis = [i for i in items if i["tag"] not in ("line", "group") and not big_bg(i) and not i["id"].startswith("chrome")]
    area = lambda i: i["w"] * i["h"]
    top = [i for i in vis if i["depth"] == 0] or vis
    hero = max(top, key=area) if top else None
    if any(i["tag"] == "image" for i in items):
        photo_slides += 1
    # Vertical coverage inside the content zone (between y=56 and y=496).
    z0, z1 = 56.0, 496.0
    spans = []
    for i in vis:
        y0 = i["y"]
        y1 = y0 + (min(i.get("text_h", i["h"]), i["h"]) if i["kind"] == "text" else i["h"])
        spans.append((max(z0, y0), min(z1, y1)))
    spans = sorted(s for s in spans if s[1] > s[0])
    gap, gap_at, cur = 0.0, (z0, z0), z0
    for a, b in spans:
        if a - cur > gap:
            gap, gap_at = a - cur, (cur, a)
        cur = max(cur, b)
    if z1 - cur > gap:
        gap, gap_at = z1 - cur, (cur, z1)
    share = 100 * area(hero) / (W * H) if hero else 0
    print(f"slide {n} `{s['id']}`: main element `{hero['id'] if hero else '-'}` ({hero['tag'] if hero else '-'}) = {share:.0f}% of slide;"
          f" largest empty band {gap:.0f} pt (y {gap_at[0]:.0f}-{gap_at[1]:.0f}); elements {len(vis)}")
    photo_bg = any(i["tag"] == "image" and i["w"] * i["h"] >= 0.5 * W * H for i in items)
    # Numbers shown only as text: no chart, unit grid, table, photo, or data shapes.
    # Decoration: a shape without text that bleeds off the slide or covers > 20 % (glows, panels).
    decor = lambda i: "lines" not in i and (i["x"] < 0 or i["y"] < 0 or i["x"] + i["w"] > W or i["y"] + i["h"] > H or area(i) > 0.2 * W * H)
    visual_tags = {"chart", "waffle", "table", "image", "dots"}
    data_lines = [i for i in items if i["tag"] == "line" and not i["id"].startswith("chrome")]
    has_visual = (any(i["tag"] in visual_tags for i in items)
                  or any(i["tag"] == "shape" and i["filled"] and not decor(i) and area(i) > 0.01 * W * H for i in items)
                  or len(data_lines) >= 2)
    arch = archetypes[n - 1] if n - 1 < len(archetypes) else None
    if not has_visual and arch not in ("A1", "A12", "A13"):
        design_lints.append(f"lint: slide {n}: only text — show the key number with a device from visuals.md (V1 unit grid, V2 bars, V3 share bar, V4 savings, V5 formula, V6 target scale, V7 process) or a chart")
    if gap > 0.25 * (z1 - z0) and not photo_bg:
        design_lints.append(f"lint: slide {n}: empty band {gap:.0f} pt (y {gap_at[0]:.0f}-{gap_at[1]:.0f}) — enlarge the main element or move content")
    if hero and share < 12:
        print(f"   check: slide {n}: no single dominant element (largest `{hero['id']}` is {share:.0f}%). Fine for a diagram made of parts; otherwise make one element clearly the biggest")
    for i in items:
        if i["tag"] == "icon" and (i["w"] > 60 or i["h"] > 60):
            design_lints.append(f"lint: slide {n}: icon `{i['id']}` is {i['w']:.0f}x{i['h']:.0f} pt — icons support text; do not use them as the main visual")
        if i["tag"] == "shape" and i["filled"] and "lines" not in i and not big_bg(i) and area(i) > 0.10 * W * H:
            print(f"   check: slide {n}: filled shape `{i['id']}` without text covers {100 * area(i) / (W * H):.0f}% — is it data, a photo frame, or empty decoration?")
if len(m["slides"]) >= 4:
    if len(archetypes) < len(m["slides"]):
        design_lints.append(f"lint: deck: mark every slide with an archetype comment before it, e.g. <!-- A5 · title --> (found {len(archetypes)} of {len(m['slides'])})")
    else:
        if len(set(archetypes)) < min(4, len(m["slides"])):
            design_lints.append(f"lint: deck: only {len(set(archetypes))} different archetypes ({', '.join(archetypes)}) — use at least 4")
        for k in range(1, len(archetypes)):
            if archetypes[k] == archetypes[k - 1]:
                design_lints.append(f"lint: deck: slides {k} and {k + 1} use the same archetype {archetypes[k]}")
print(f"deck: {len(m['slides'])} slides, archetypes {archetypes or '-'}, slides with photos: {photo_slides}")
for l in design_lints:
    print(l)
tech_ok &= not design_lints

# ---- 6. quality gate --------------------------------------------------------------
print("== 5. QUALITY GATE")
gate = []
plan = deck.parent / "plan.md"
if not plan.exists():
    gate.append(f"no plan: write {plan} from plan-template.md first")
review = deck.parent / "review.json"
if not review.exists():
    gate.append(f"no design review: write {review} (format in rubric.md)")
else:
    try:
        passes = json.loads(review.read_text(encoding="utf-8"))["passes"]
        last = passes[-1]["scores"]
        total = sum(int(v) for v in last.values())
        zeros = [k for k, v in last.items() if int(v) == 0]
        print(f"review: {len(passes)} pass(es), last score {total}/20, zeros: {zeros or 'none'}")
        if len(passes) < 2:
            gate.append("fewer than 2 design passes after the first technical PASS")
        if len(last) != 10:
            gate.append("the last pass must score all 10 criteria of rubric.md")
        if total < 16 or zeros:
            gate.append("last score below 16/20 or has a zero")
        if any(not p.get("changes") for p in passes):
            gate.append("every pass must list the changes it made")
    except (KeyError, IndexError, ValueError, TypeError, json.JSONDecodeError) as e:
        gate.append(f"review.json does not match the format in rubric.md ({e})")
for g in gate:
    print("gate:", g)

print(f"\nLOOK AT EVERY IMAGE: {png}\\slideN.png and {png}\\slideN.overlay.png")
if not tech_ok:
    print("RESULT: FAIL (fix every error/warning/lint above and run again)")
    sys.exit(1)
if gate:
    print("RESULT: DRAFT (technically clean; now do the design work: plan, review passes — see SKILL.md)")
    sys.exit(2)
print("RESULT: PASS")
sys.exit(0)
