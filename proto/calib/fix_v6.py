p = 'F:/pptx-ai-native/proto/calib/visuals_slides.xml'
s = open(p, encoding='utf-8').read()
a = '    <anim target="v6-arrow"'
add = '''    <line id="v6-rule" x1="48" y1="408" x2="912" y2="408" stroke="dk1/15 0.75"/>
    <text id="v6-gain" x="48" y="424" w="864" h="auto" class="h2" color="accent1">−3 повторных выезда в&#160;квартал при&#160;86 объектах</text>
'''
assert a in s
s = s.replace(a, add + a, 1)
open(p, 'w', encoding='utf-8').write(s)
st = open('F:/pptx-ai-native/.claude/skills/slideml/starter.xml', encoding='utf-8').read()
open('F:/pptx-ai-native/examples/visuals/deck.xml', 'w', encoding='utf-8').write(st.replace('  <!-- Slides go here.', s + '\n  <!-- Slides go here.'))
