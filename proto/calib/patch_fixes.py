base = 'F:/pptx-ai-native/proto/src/'
def patch(f, pairs):
    s = open(base + f, encoding='utf-8').read()
    for old, new in pairs:
        assert old in s, (f, old[:70])
        s = s.replace(old, new, 1)
    open(base + f, 'w', encoding='utf-8').write(s)

patch('measure.rs', [
('''    pub max_line_width: f64,
}''', '''    pub max_line_width: f64,
    /// Words wider than the line: PowerPoint breaks them between letters.
    pub split_words: usize,
}'''),
('''    let mut max_w: f64 = 0.0;''', '''    let mut max_w: f64 = 0.0;
    let splits = std::cell::Cell::new(0usize);'''),
('''                while *word > w + 0.01 && w > 0.0 {''', '''                if *word > w + 0.01 && w > 0.0 {
                    splits.set(splits.get() + 1);
                }
                while *word > w + 0.01 && w > 0.0 {'''),
('''    Layout { lines: lines_total, height, max_line_width: max_w }''', '''    Layout { lines: lines_total, height, max_line_width: max_w, split_words: splits.get() }'''),
])
patch('deck.rs', [
('''        if min_size < 9.0 {''', '''        if lay.split_words > 0 {
            ctx.warn(node, format!("{} word(s) are wider than the box and break between letters. Fix: widen w, reduce size, or use fit=\\"shrink\\"", lay.split_words));
        }
        if min_size < 9.0 {'''),
])
patch('text.rs', [
('''        let bold = st.b == Some(true);''', '''        let bold = st.b == Some(true);
        let lower = fam.to_lowercase();
        let heavy = bold || ["semibold", "bold", "black", "heavy"].iter().any(|w| lower.contains(w));'''),
('''            [ctx.body_font.as_str(), "Segoe UI", "Segoe UI Symbol"]
                .into_iter()
                .find(|f| crate::fonts::face(f, bold).is_some_and(|ff| ff.has(c)))
                .map(String::from)''', '''            // Keep the visual weight: heavy text takes a heavy fallback face.
            let body = ctx.body_font.as_str();
            let base = body.split(' ').next().unwrap_or(body);
            let candidates: Vec<String> = if heavy {
                vec![format!("{base} SemiBold"), format!("{base} Bold"), format!("{base} ExtraBold"), "Inter SemiBold".into(), "Segoe UI Semibold".into(), "Segoe UI Black".into()]
            } else {
                vec![body.to_string(), "Inter".into(), "Segoe UI".into(), "Segoe UI Symbol".into()]
            };
            candidates.into_iter().find(|f| crate::fonts::face(f, false).is_some_and(|ff| ff.has(c)))'''),
])
patch('chart.rs', [
('''                let (lx, ly) = (bx + w / 2.0, if dy < 0.0 { by + h } else { by });
                above += &line_sp(lid, &format!("{name}-line"), lx, ly, px, py, &ln);''', '''                let (lx, ly) = (bx + w / 2.0, if dy < 0.0 { by + h } else { by });
                // Stop the leader above the data label, so the line does not cross the number.
                let label_shown = labels && series[si].label_points.as_ref().is_none_or(|k| k.contains(&ci));
                let end_y = if label_shown && !horizontal && dy < 0.0 && (label_pos == "outEnd" || label_pos == "t") { py - line_h(label_size) - 3.0 } else { py };
                above += &line_sp(lid, &format!("{name}-line"), lx, ly, px, end_y, &ln);'''),
])
print('ok')
