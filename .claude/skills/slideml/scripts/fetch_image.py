"""Finds and downloads openly licensed photos (Openverse API, no key) into a deck's assets folder.

Usage: python fetch_image.py "<query in English>" <assets_dir> [--count N] [--name NAME]
- Licenses: CC0, Public Domain Mark, CC BY (attribution required; written to SOURCES.md).
- Prefers large, wide photos; skips images narrower than 1200 px.
- Saves <name>-<i>.jpg and appends one line per file to <assets_dir>/SOURCES.md.
Prints the saved files with size and attribution, so a model without vision can still choose.
"""
import argparse
import json
import re
import urllib.parse
import urllib.request
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("query")
ap.add_argument("assets")
ap.add_argument("--count", type=int, default=1)
ap.add_argument("--name")
a = ap.parse_args()

assets = Path(a.assets)
assets.mkdir(parents=True, exist_ok=True)
name = a.name or re.sub(r"[^a-z0-9]+", "-", a.query.lower()).strip("-")[:40]
headers = {"User-Agent": "slideml-fetch/1.0"}


def search(extra):
    """Returns results, or None when the search service does not answer (slow API, no network)."""
    q = urllib.parse.urlencode({"q": a.query, "license": "cc0,pdm,by", "page_size": 20, **extra})
    req = urllib.request.Request(f"https://api.openverse.org/v1/images/?{q}", headers=headers)
    for _ in range(2):
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                return json.load(r).get("results", [])
        except Exception as e:  # timeouts and HTTP errors: retry once, then report
            err = e
    print(f"search failed: {err}")
    return None


results = None
for extra in ({"size": "large", "aspect_ratio": "wide"}, {"size": "large"}, {}):
    found = search(extra)
    if found:
        results = found
        break
if results is None:
    print("SEARCH UNAVAILABLE. Run the command again later, or use a gradient or <dots> background (imagery.md).")
    raise SystemExit(1)
results = [r for r in results if (r.get("width") or 0) >= 1200 and r.get("url", "").lower().split("?")[0].endswith((".jpg", ".jpeg", ".png"))]
if not results:
    print(f"NO RESULTS for '{a.query}'. Try a simpler English query, or use a gradient/dots background.")
    raise SystemExit(1)

saved = 0
sources = assets / "SOURCES.md"
for r in results:
    if saved >= a.count:
        break
    ext = ".png" if r["url"].lower().split("?")[0].endswith(".png") else ".jpg"
    path = assets / f"{name}-{saved + 1}{ext}"
    try:
        req = urllib.request.Request(r["url"], headers=headers)
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = resp.read()
    except Exception as e:  # dead links are common on Flickr mirrors; try the next result
        print(f"skip {r['url']}: {e}")
        continue
    if len(data) < 30_000:
        continue
    path.write_bytes(data)
    saved += 1
    lic = (r.get("license") or "").upper()
    credit = f"Фото: {r.get('creator') or 'неизвестный автор'}, {lic} {r.get('license_version') or ''}".strip()
    with sources.open("a", encoding="utf-8") as f:
        f.write(f"- {path.name} — {r.get('title') or ''} — {credit} — {r.get('foreign_landing_url') or r['url']}\n")
    print(f"saved {path}  {r.get('width')}x{r.get('height')}  title: {r.get('title')}  credit: {credit}")
    if lic == "BY":
        print("  CC BY: put the credit line on the slide (10 pt) or in the final report.")
print(f"{saved} file(s). Sources: {sources}")
raise SystemExit(0 if saved else 1)
