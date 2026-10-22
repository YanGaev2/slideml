//! Geometry lint over the computed boxes, and the JSON manifest used for overlays.

pub struct TextInfo {
    /// Widest line in pt and paragraph alignment of the first paragraph.
    pub width: f64,
    pub align: String,
    pub lines: usize,
    pub height: f64,
    pub avail: f64,
    pub min_size: f64,
}

pub struct Item {
    pub id: String,
    pub kind: &'static str,
    /// (x0, y0, x1, y1) in pt.
    pub bbox: (f64, f64, f64, f64),
    pub text: Option<TextInfo>,
    pub depth: usize,
    /// Source position "line:col".
    pub pos: String,
    /// Source element name: text, shape, image, icon, chart, table, line, dots, group.
    pub tag: String,
    /// The shape has a fill (data mark, panel, or decoration).
    pub filled: bool,
}

/// Visible text extent: a top-anchored text box only covers its measured lines.
fn text_box(i: &Item) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = i.bbox;
    match (&i.text, i.kind) {
        (Some(t), "text") => {
            let w = t.width.min(x1 - x0);
            let (a, b) = match t.align.as_str() {
                "right" => (x1 - w, x1),
                "center" => ((x0 + x1 - w) / 2.0, (x0 + x1 + w) / 2.0),
                _ => (x0, x0 + w),
            };
            (a, y0, b, (y0 + t.height).min(y1))
        }
        _ => (x0, y0, x1, y1),
    }
}

pub fn lint(slide: &str, items: &[Item], w: f64, h: f64) -> Vec<String> {
    let mut out = Vec::new();
    let texts: Vec<&Item> = items.iter().filter(|i| i.kind == "text" && i.text.is_some()).collect();
    for (a_i, a) in texts.iter().enumerate() {
        let ab = text_box(a);
        if ab.0 < -0.5 || ab.1 < -0.5 || ab.2 > w + 0.5 || ab.3 > h + 0.5 {
            out.push(format!("lint: slide `{slide}` {}: text `{}` leaves the slide", a.pos, a.id));
        }
        for b in &texts[a_i + 1..] {
            let bb = text_box(b);
            let ix = ab.2.min(bb.2) - ab.0.max(bb.0);
            let iy = ab.3.min(bb.3) - ab.1.max(bb.1);
            if ix > 2.0 && iy > 2.0 {
                out.push(format!("lint: slide `{slide}`: text at {} and text at {} overlap ({ix:.0}x{iy:.0}pt)", a.pos, b.pos));
            }
            let left = |i: &Item| i.text.as_ref().is_none_or(|t| t.align != "right" && t.align != "center");
            let dx = if left(a) && left(b) { (ab.0 - bb.0).abs() } else { 0.0 };
            let near = (ab.1 - bb.3).abs().min((bb.1 - ab.3).abs()) < 80.0;
            if dx > 0.5 && dx < 4.0 && near {
                out.push(format!("lint: slide `{slide}`: left edges at {} (x={}) and {} (x={}) almost align: use one value", a.pos, ab.0, b.pos, bb.0));
            }
        }
    }
    out
}

fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o += "\\\"",
            '\\' => o += "\\\\",
            c if (c as u32) < 0x20 => o += &format!("\\u{:04x}", c as u32),
            c => o.push(c),
        }
    }
    o + "\""
}

pub fn slide_json(slide: &str, items: &[Item]) -> String {
    let its: Vec<String> = items
        .iter()
        .map(|i| {
            let (x0, y0, x1, y1) = i.bbox;
            let t = match &i.text {
                Some(t) => format!(",\"lines\":{},\"text_h\":{:.1},\"avail_h\":{:.1},\"min_size\":{}", t.lines, t.height, t.avail, t.min_size),
                None => String::new(),
            };
            format!(
                "{{\"tag\":\"{}\",\"filled\":{},\"pos\":{},\"id\":{},\"kind\":\"{}\",\"x\":{x0:.1},\"y\":{y0:.1},\"w\":{:.1},\"h\":{:.1},\"depth\":{}{t}}}",
                i.tag,
                i.filled,
                js(&i.pos),
                js(&i.id),
                i.kind,
                x1 - x0,
                y1 - y0,
                i.depth
            )
        })
        .collect();
    format!("{{\"id\":{},\"items\":[{}]}}", js(slide), its.join(","))
}
