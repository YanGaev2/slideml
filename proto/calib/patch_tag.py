base = 'F:/pptx-ai-native/proto/src/'
def patch(f, pairs):
    s = open(base + f, encoding='utf-8').read()
    for old, new in pairs:
        assert old in s, (f, old[:70])
        s = s.replace(old, new, 1)
    open(base + f, 'w', encoding='utf-8').write(s)
patch('lint.rs', [
('''    /// Source position "line:col".
    pub pos: String,
}''', '''    /// Source position "line:col".
    pub pos: String,
    /// Source element name: text, shape, image, icon, chart, table, line, dots, group.
    pub tag: String,
    /// A filled shape without text (decoration or a data mark).
    pub filled: bool,
}'''),
('''                "{{\\"pos\\":{},\\"id\\":{},\\"kind\\":\\"{}\\",''', '''                "{{\\"tag\\":\\"{}\\",\\"filled\\":{},\\"pos\\":{},\\"id\\":{},\\"kind\\":\\"{}\\",'''),
('''                js(&i.pos),
                js(&i.id),''', '''                i.tag,
                i.filled,
                js(&i.pos),
                js(&i.id),'''),
])
patch('deck.rs', [
('''        depth: ctx.depth,
        pos: pos_of(node),''', '''        depth: ctx.depth,
        pos: pos_of(node),
        tag: node.tag_name().name().to_string(),
        filled: !has_text && node.attribute("fill").unwrap_or(if is_text { "none" } else { "accent1" }) != "none",'''),
('''text: None, depth: ctx.depth, pos: pos_of(c) });''', '''text: None, depth: ctx.depth, pos: pos_of(c), tag: name.to_string(), filled: false });'''),
])
print('ok')
