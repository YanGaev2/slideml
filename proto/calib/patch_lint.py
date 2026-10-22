import re
base = 'F:/pptx-ai-native/proto/src/'
def patch(f, pairs):
    s = open(base + f, encoding='utf-8').read()
    for old, new in pairs:
        assert old in s, (f, old[:60])
        s = s.replace(old, new, 1)
    open(base + f, 'w', encoding='utf-8').write(s)

patch('lint.rs', [
('''pub struct TextInfo {
    pub lines: usize,''', '''pub struct TextInfo {
    /// Widest line in pt and paragraph alignment of the first paragraph.
    pub width: f64,
    pub align: String,
    pub lines: usize,'''),
('''    pub depth: usize,
}''', '''    pub depth: usize,
    /// Source position "line:col".
    pub pos: String,
}'''),
('''        (Some(t), "text") => (x0, y0, x1, (y0 + t.height).min(y1)),''', '''        (Some(t), "text") => {
            let w = t.width.min(x1 - x0);
            let (a, b) = match t.align.as_str() {
                "right" => (x1 - w, x1),
                "center" => ((x0 + x1 - w) / 2.0, (x0 + x1 + w) / 2.0),
                _ => (x0, x0 + w),
            };
            (a, y0, b, (y0 + t.height).min(y1))
        }'''),
('''            out.push(format!("lint: slide `{slide}`: text `{}` leaves the slide", a.id));''', '''            out.push(format!("lint: slide `{slide}` {}: text `{}` leaves the slide", a.pos, a.id));'''),
('''                out.push(format!("lint: slide `{slide}`: text `{}` and `{}` overlap ({ix:.0}x{iy:.0}pt)", a.id, b.id));''', '''                out.push(format!("lint: slide `{slide}`: text at {} and text at {} overlap ({ix:.0}x{iy:.0}pt)", a.pos, b.pos));'''),
('''            let dx = (ab.0 - bb.0).abs();''', '''            let left = |i: &Item| i.text.as_ref().is_none_or(|t| t.align != "right" && t.align != "center");
            let dx = if left(a) && left(b) { (ab.0 - bb.0).abs() } else { 0.0 };'''),
('''                out.push(format!("lint: slide `{slide}`: left edges of `{}` (x={}) and `{}` (x={}) almost align: use one value", a.id, ab.0, b.id, bb.0));''', '''                out.push(format!("lint: slide `{slide}`: left edges at {} (x={}) and {} (x={}) almost align: use one value", a.pos, ab.0, b.pos, bb.0));'''),
('''            format!("{{\\"id\\":{},\\"kind\\":\\"{}\\",''', '''            format!("{{\\"pos\\":{},\\"id\\":{},\\"kind\\":\\"{}\\",'''),
('''i.depth)
        })''', '''i.depth).replacen("{", &format!("{{\\"pos\\":{},", js(&i.pos)), 1)
        })'''),
])
patch('deck.rs', [
('''        text: has_text.then_some(crate::lint::TextInfo { lines: lay.lines, height: lay.height, avail: h - t - b, min_size }),
        depth: ctx.depth,''', '''        text: has_text.then_some(crate::lint::TextInfo { width: lay.max_line_width, align: st.align_name().to_string(), lines: lay.lines, height: lay.height, avail: h - t - b, min_size }),
        depth: ctx.depth,
        pos: pos_of(node),'''),
('''            ctx.items.push(crate::lint::Item { id, kind: if name == "group" { "group" } else { "other" }, bbox: b, text: None, depth: ctx.depth });''',
 '''            ctx.items.push(crate::lint::Item { id, kind: if name == "group" { "group" } else { "other" }, bbox: b, text: None, depth: ctx.depth, pos: pos_of(c) });'''),
('''type BBox = (f64, f64, f64, f64);''', '''type BBox = (f64, f64, f64, f64);

fn pos_of(n: Node) -> String {
    let p = n.document().text_pos_at(n.range().start);
    format!("{}:{}", p.row, p.col)
}'''),
])
patch('text.rs', [
('''    pub fn with_bold(mut self) -> Self {''', '''    pub fn align_name(&self) -> &str {
        self.align.as_deref().unwrap_or("left")
    }

    pub fn with_bold(mut self) -> Self {'''),
])
print('ok')
