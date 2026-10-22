p = 'F:/pptx-ai-native/examples/hero/deck.xml'
s = open(p, encoding='utf-8').read()

# ---- S5: waffle of 367 blocks, real positions of the 2 misses (#101, #120) ----
cols, step, cell, x0, y0 = 46, 864 / 46, 14, 48, 288
miss = (101, 120)
d = []
for i in range(1, 368):
    if i in miss:
        continue
    r, c = divmod(i - 1, cols)
    x, y = c * step, r * step
    d.append(f"M{x:.2f} {y:.2f}h{cell}v{cell}h-{cell}Z")
grid_w, grid_h = (cols - 1) * step + cell, 7 * step + cell
pos = {}
for i in miss:
    r, c = divmod(i - 1, cols)
    pos[i] = (x0 + c * step, y0 + r * step)
s5 = f'''  <!-- S5 PROOF: TEXT. 367 checked text blocks as a grid in check order; the 2 misses (#101, #120)
       are orange at their real positions. The signal morphs into block #101. -->
  <slide id="s5" bg="lt1" transition="morph" transition-dur="900">
    <use template="chrome" section="Доказательство" page="05" ink="dk2" rule="#111111/15"/>
    <text id="s5-label" x="48" y="80" w="494" h="auto" class="label">Измерение текста</text>
    <text id="s5-hero" x="41" y="124" w="560" h="128" class="hero">99,5<span size="44"> </span>%</text>
    <text id="s5-head" x="640" y="108" w="272" h="auto" class="h2">Агент знает, сколько строк займёт текст, ещё до&#160;PowerPoint.</text>
    <text id="s5-body" x="640" y="196" w="272" h="auto" class="body">Мы сверили 367 текстовых блоков из&#160;четырёх презентаций. В&#160;365 число строк совпало.</text>

    <shape id="s5-grid" x="{x0}" y="{y0}" w="{grid_w:.2f}" h="{grid_h:.2f}" viewbox="0 0 {grid_w:.2f} {grid_h:.2f}" fill="lt2" line="none"
           path="{' '.join(d)}"/>
    <shape id="signal" geom="rect" x="{pos[101][0]:.2f}" y="{pos[101][1]:.2f}" w="{cell}" h="{cell}" fill="accent1" line="none"/>
    <shape id="s5-miss2" geom="rect" x="{pos[120][0]:.2f}" y="{pos[120][1]:.2f}" w="{cell}" h="{cell}" fill="accent1" line="none"/>

    <group id="s5-legend">
      <shape geom="rect" x="48" y="456" w="12" h="12" fill="lt2" line="none"/>
      <text x="68" y="454" w="200" h="auto" class="body-s">365 блоков совпали</text>
      <shape geom="rect" x="240" y="456" w="12" h="12" fill="accent1" line="none"/>
      <text x="260" y="454" w="652" h="auto" class="body-s">2 разошлись: компилятор насчитал на&#160;одну строку больше, чем PowerPoint</text>
    </group>
    <anim target="s5-grid" effect="wipe-left" start="after" dur="900"/>
    <anim target="s5-legend" effect="fade" start="after" dur="400"/>
  </slide>

'''
a = s.index('  <!-- S5 PROOF')
b = s.index('  <!-- S6 PROOF')
s = s[:a] + s5 + s[b:]

# ---- S7: an explicit loop with solid arrows; the signal is the "fix and repeat" pill ----
steps = [
    ('01', 'Написать XML', 'Фигуры, текст, графики, анимации.'),
    ('02', 'Собрать', 'Компилятор ловит переполнение, мелкий текст, наложения.'),
    ('03', 'Отрисовать', 'PowerPoint рисует каждый слайд в&#160;PNG.'),
    ('04', 'Посмотреть', 'Агент видит слайд и&#160;каждый объект на&#160;нём.'),
]
xs = [48, 270, 492, 714]
body = ''
for (n, h, t), x in zip(steps, xs):
    body += f'''    <shape id="s7-m{n}" geom="rect" x="{x}" y="195" w="10" h="10" fill="dk1" line="none"/>
    <text id="s7-n{n}" x="{x + 24}" y="224" w="174" h="auto" class="label">{n}</text>
    <text id="s7-h{n}" x="{x + 24}" y="244" w="174" h="auto" class="h2">{h}</text>
    <text id="s7-b{n}" x="{x + 24}" y="276" w="174" h="auto" class="body-s">{t}</text>
'''
arrows = ''
for i in range(3):
    arrows += f'    <line id="s7-a{i + 1}" x1="{xs[i] + 18}" y1="200" x2="{xs[i + 1] - 8}" y2="200" stroke="dk1 1.25" tail="triangle"/>\n'
s7 = f'''  <!-- S7 AGENT LOOP: four steps with arrows and a solid return path from 04 back to 01.
       The signal is the orange "fix and repeat" pill on the return path. -->
  <slide id="s7" bg="lt1" transition="morph" transition-dur="900">
    <use template="chrome" section="Цикл агента" page="07" ink="dk2" rule="#111111/15"/>
    <text id="s7-title" x="48" y="80" w="642" h="auto" class="h1">Агент не угадывает.<br/>Он смотрит на результат.</text>
{arrows}{body}    <group id="s7-ret">
      <line id="s7-r1" x1="719" y1="344" x2="719" y2="376" stroke="dk1 1.25"/>
      <line id="s7-r2" x1="719" y1="376" x2="53" y2="376" stroke="dk1 1.25"/>
      <line id="s7-r3" x1="53" y1="376" x2="53" y2="213" stroke="dk1 1.25" tail="triangle"/>
    </group>
    <shape id="signal" geom="rect" x="270" y="364" w="244" h="24" fill="accent1" line="none"/>
    <text id="s7-ret-label" x="270" y="364" w="244" h="24" anchor="middle" align="center"
          font="Cascadia Mono" size="12" spacing="1" caps="1" color="dk1">Исправить и&#160;снова</text>
    <text id="s7-self" x="48" y="420" w="864" h="auto" class="h2">Так мы улучшили сам SlideML: 3 презентации, 16 слайдов, замечания критиков стали правками компилятора.</text>
    <anim target="s7-ret" effect="wipe-left" start="after" dur="700"/>
    <anim target="s7-ret-label" effect="fade" start="with" delay="300" dur="300"/>
    <anim target="s7-self" effect="fade" start="click"/>
  </slide>

'''
a = s.index('  <!-- S7 AGENT LOOP')
b = s.index('  <!-- S8 NEXT')
s = s[:a] + s7 + s[b:]
open(p, 'w', encoding='utf-8').write(s)
print('ok')
