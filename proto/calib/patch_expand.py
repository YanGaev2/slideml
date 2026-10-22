base = 'F:/pptx-ai-native/proto/src/'
def patch(f, pairs):
    s = open(base + f, encoding='utf-8').read()
    for old, new in pairs:
        assert old in s, (f, old[:60])
        s = s.replace(old, new, 1)
    open(base + f, 'w', encoding='utf-8').write(s)
patch('util.rs', [
(r'''pub fn at(node: Node) -> String {
    let p = node.document().text_pos_at(node.range().start);
    format!("{}:{} <{}>", p.row, p.col, node.tag_name().name())
}''', r'''/// Source position "line:col" (from data-src after template expansion).
pub fn src_pos(node: Node) -> String {
    match node.attribute("data-src") {
        Some(s) => s.to_string(),
        None => {
            let p = node.document().text_pos_at(node.range().start);
            format!("{}:{}", p.row, p.col)
        }
    }
}

pub fn at(node: Node) -> String {
    format!("{} <{}>", src_pos(node), node.tag_name().name())
}'''),
(r'''        if !allowed.iter().any(|set| set.contains(&a.name())) {''', r'''        if a.name() != "data-src" && !allowed.iter().any(|set| set.contains(&a.name())) {'''),
])
patch('deck.rs', [
(r'''fn pos_of(n: Node) -> String {
    let p = n.document().text_pos_at(n.range().start);
    format!("{}:{}", p.row, p.col)
}''', r'''fn pos_of(n: Node) -> String {
    src_pos(n)
}'''),
(r'''    let doc = roxmltree::Document::parse(&src).map_err(|e| format!("{}: XML error: {e}", path.display()))?;''',
 r'''    let src = crate::expand::expand(&src).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc = roxmltree::Document::parse(&src).map_err(|e| format!("{}: XML error after template expansion: {e}", path.display()))?;'''),
])
patch('main.rs', [("mod fonts;\n", "mod expand;\nmod fonts;\n")])
patch('chart.rs', [
(r'''                check_attrs(c, &[&["id", "value", "stroke", "color", "size", "font", "label"]])?;''', r'''                check_attrs(c, &[&["id", "value", "stroke", "color", "size", "font", "label", "label-at"]])?;
                let at_start = match c.attribute("label-at").unwrap_or("end") {
                    "start" => true,
                    "end" => false,
                    v => return err(c, format!("label-at `{v}`: use start or end")),
                };'''),
(r'''                        above += &text_sp(lid, &format!("{name}-label"), plot.x + plot.w - w, y - line_h(ann_size) - 8.0, w, line_h(ann_size) + 8.0, &para(&label, &ann_font, ann_size, &cx_col, false, "r", &lang), "<a:noFill/>", "b", &lang);''',
 r'''                        let lx = if at_start { plot.x } else { plot.x + plot.w - w };
                        above += &text_sp(lid, &format!("{name}-label"), lx, y - line_h(ann_size) - 8.0, w, line_h(ann_size) + 8.0, &para(&label, &ann_font, ann_size, &cx_col, false, if at_start { "l" } else { "r" }, &lang), "<a:noFill/>", "b", &lang);'''),
])
print('ok')
