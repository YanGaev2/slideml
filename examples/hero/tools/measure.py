# usage: python measure.py font size spacing "text" ["prefix"]  -> width of text, width of prefix
import sys
from fontTools.ttLib import TTFont
f = TTFont({'sb':'C:/Windows/Fonts/Inter-SemiBold.ttf','r':'C:/Windows/Fonts/Inter-Regular.ttf','mono':'C:/Windows/Fonts/CascadiaMono.ttf'}[sys.argv[1]])
cmap = f.getBestCmap(); hm = f['hmtx']; upm = f['head'].unitsPerEm
size, sp = float(sys.argv[2]), float(sys.argv[3])
def w(s, last=False):
    # advance widths + tracking after every char (PowerPoint adds spacing after each glyph)
    return sum(hm[cmap[ord(c)]][0] for c in s) * size / upm + sp * len(s)
for s in sys.argv[4:]:
    print(repr(s), round(w(s), 2), 'ink-ish (minus last spacing):', round(w(s) - sp, 2))
os2 = f['OS/2']; print('ascender', os2.sTypoAscender*size/upm, 'winAscent', os2.usWinAscent*size/upm, 'capH', getattr(os2,'sCapHeight',0)*size/upm)
