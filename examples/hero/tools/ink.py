# usage: python ink.py slideN.png x0 y0 x1 y1 [mode]  (pt coords, 1920px render = 2px/pt)
# prints ink bbox (pt) of dark (mode=dark) / light / orange pixels in region, plus row bands.
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert('RGB'); s = im.width / 960
x0, y0, x1, y1 = [float(v) for v in sys.argv[2:6]]; mode = sys.argv[6] if len(sys.argv) > 6 else 'dark'
def hit(p):
    r, g, b = p
    if mode == 'dark': return r < 90 and g < 90 and b < 90
    if mode == 'light': return r > 200 and g > 200 and b > 200
    if mode == 'orange': return r > 200 and 40 < g < 120 and b < 80
xs, ys, rows = [], [], {}
for py in range(int(y0*s), int(y1*s)):
    for px in range(int(x0*s), int(x1*s)):
        if hit(im.getpixel((px, py))): xs.append(px); ys.append(py); rows[py] = 1
if not xs: print('none'); sys.exit()
print('bbox pt: x %.1f-%.1f  y %.1f-%.1f' % (min(xs)/s, (max(xs)+1)/s, min(ys)/s, (max(ys)+1)/s))
bands, start, prev = [], None, None
for py in sorted(rows):
    if start is None: start = py
    elif py != prev + 1: bands.append((start, prev)); start = py
    prev = py
bands.append((start, prev))
for a, b in bands: print('  band y %.1f-%.1f  centre %.1f' % (a/s, (b+1)/s, (a+b+1)/2/s))
