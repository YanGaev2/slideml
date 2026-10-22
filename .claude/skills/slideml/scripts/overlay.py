"""Draws element boxes and ids from <deck>.manifest.json over the PowerPoint PNGs.

Usage: python overlay.py <out.manifest.json> <png_dir>
Writes <png_dir>/slideN.overlay.png. Colours: blue = text box (filled part = measured text,
red if it overflows), grey = shapes/images/charts, dashed purple = groups.
"""
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

manifest = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
png_dir = Path(sys.argv[2])
font = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 13)

for n, slide in enumerate(manifest["slides"], 1):
    src = png_dir / f"slide{n}.png"
    if not src.exists():
        continue
    base = Image.open(src).convert("RGBA")
    k = base.width / manifest["w"]
    layer = Image.new("RGBA", base.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    for it in slide["items"]:
        x0, y0 = it["x"] * k, it["y"] * k
        x1, y1 = x0 + it["w"] * k, y0 + it["h"] * k
        if it["kind"] == "group":
            for i in range(int(x0), int(x1), 12):
                d.line([(i, y0), (min(i + 6, x1), y0)], fill=(150, 60, 220, 255), width=2)
                d.line([(i, y1), (min(i + 6, x1), y1)], fill=(150, 60, 220, 255), width=2)
            continue
        if "lines" in it:
            over = it["text_h"] > it["avail_h"] + 0.5
            fill = (230, 40, 40, 70) if over else (40, 120, 255, 45)
            th = min(it["text_h"], it["h"]) * k if it["kind"] == "text" else it["h"] * k
            d.rectangle([x0, y0, x1, y0 + th], fill=fill)
            d.rectangle([x0, y0, x1, y1], outline=(230, 40, 40, 255) if over else (40, 120, 255, 220), width=2)
            color = (230, 40, 40, 255) if over else (20, 80, 220, 255)
        else:
            d.rectangle([x0, y0, x1, y1], outline=(120, 120, 120, 200), width=1)
            color = (90, 90, 90, 255)
        label = it["id"]
        tw = d.textlength(label, font=font)
        ly = max(0, y0 - 15)
        d.rectangle([x0, ly, x0 + tw + 6, ly + 15], fill=(255, 255, 255, 220))
        d.text((x0 + 3, ly), label, fill=color, font=font)
    Image.alpha_composite(base, layer).convert("RGB").save(png_dir / f"slide{n}.overlay.png")
    print(png_dir / f"slide{n}.overlay.png")
