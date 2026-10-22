import re
p = 'F:/pptx-ai-native/proto/calib/visuals_slides.xml'
s = open(p, encoding='utf-8').read()
arch = {'V1': 'A7', 'V2': 'A10', 'V3': 'A7', 'V4': 'A4', 'V5': 'A3', 'V6': 'A9', 'V7': 'A14'}
for v, a in arch.items():
    s = s.replace(f'<!-- {v} · ', f'<!-- {a} · {v} · ')
s = s.replace('font="heading" size="18" color="dk1"><p>', 'font="heading" size="15" color="dk1"><p>')
s = s.replace('font="heading" size="18" color="lt1"><p>', 'font="heading" size="15" color="lt1"><p>')
# V3: taller bar, text lower
s = s.replace('<shape id="v3-b2b" x="48" y="216" w="622" h="88"', '<shape id="v3-b2b" x="48" y="216" w="622" h="128"')
s = s.replace('<shape id="v3-b2c" x="670" y="216" w="242" h="88"', '<shape id="v3-b2c" x="670" y="216" w="242" h="128"')
for i, (a, b) in enumerate([('id="v3-a-h" x="48" y="328"', 'id="v3-a-h" x="48" y="368"'), ('id="v3-a-b" x="48" y="362"', 'id="v3-a-b" x="48" y="402"'),
                            ('id="v3-b-h" x="670" y="328"', 'id="v3-b-h" x="670" y="368"'), ('id="v3-b-b" x="670" y="362"', 'id="v3-b-b" x="670" y="402"')]):
    assert a in s, a
    s = s.replace(a, b)
# V6: move the scale up by 48 pt
blk = s[s.index('V6 ·'):s.index('V7 ·')]
nb = re.sub(r'\b(y|y1|y2)="(\d+)"', lambda m: f'{m.group(1)}="{int(m.group(2)) - 48}"' if int(m.group(2)) >= 220 else m.group(0), blk)
s = s.replace(blk, nb)
open(p, 'w', encoding='utf-8').write(s)
st = open('F:/pptx-ai-native/.claude/skills/slideml/starter.xml', encoding='utf-8').read()
open('F:/pptx-ai-native/examples/visuals/deck.xml', 'w', encoding='utf-8').write(st.replace('  <!-- Slides go here.', s + '\n  <!-- Slides go here.'))
print('ok')
