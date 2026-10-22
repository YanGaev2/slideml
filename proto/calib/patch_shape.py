p = 'F:/pptx-ai-native/proto/src/deck.rs'
s = open(p, encoding='utf-8').read()

def rep(old, new, count=1):
    global s
    assert old in s, old[:80]
    s = s.replace(old, new, count)

rep('''pub struct SlideCtx {
    pub lang: String,
    rels: Vec<(String, String, String, bool)>,
    next_id: u32,
    names: HashMap<String, (u32, Kind)>,
}''', '''pub struct SlideCtx {
    pub lang: String,
    pub body_font: String,
    pub heading_font: String,
    /// Multiplier for every font size; used by fit="shrink".
    pub text_scale: f64,
    /// Measurement data of the paragraphs written last.
    pub measure: Vec<crate::measure::MPara>,
    pub slide_id: String,
    pub warnings: Vec<String>,
    /// Geometry manifest of every element, for lint and overlays.
    pub items: Vec<crate::lint::Item>,
    /// Group nesting level of the element being written.
    pub depth: usize,
    rels: Vec<(String, String, String, bool)>,
    next_id: u32,
    names: HashMap<String, (u32, Kind)>,
}''')
rep('''impl SlideCtx {''', '''impl SlideCtx {
    pub fn warn(&mut self, node: Node, msg: impl std::fmt::Display) {
        self.warnings.push(format!("warning: slide `{}` {}: {msg}", self.slide_id, at(node)));
    }''')
rep('''    let mut ctx = SlideCtx { lang: lang.into(), rels: vec![], next_id: 1, names: HashMap::new() };''', '''    let mut ctx = SlideCtx {
        lang: lang.into(),
        body_font: th.body.clone(),
        heading_font: th.heading.clone(),
        text_scale: 1.0,
        measure: vec![],
        slide_id: node.attribute("id").unwrap_or("").to_string(),
        warnings: vec![],
        items: vec![],
        depth: 0,
        rels: vec![],
        next_id: 1,
        names: HashMap::new(),
    };''')
rep('''fn xfrm(node: Node, tag: &str) -> Res<String> {
    let (x, y, w, h) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?, req_num(node, "h")?);''', '''fn xfrm(node: Node, tag: &str) -> Res<String> {
    xfrm_h(node, tag, req_num(node, "h")?)
}

fn xfrm_h(node: Node, tag: &str, h: f64) -> Res<String> {
    let (x, y, w) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?);''')
rep('''fn viewbox(node: Node) -> Res<(f64, f64)> {''', '''fn viewbox(node: Node, w: f64, h: f64) -> Res<(f64, f64)> {''')
rep('''        None => Ok((req_num(node, "w")?, req_num(node, "h")?)),''', '''        None => Ok((w, h)),''')
rep('''fn cust_geom(node: Node, d: &str) -> Res<String> {
    let (vw, vh) = viewbox(node)?;''', '''fn cust_geom(node: Node, d: &str, w: f64, h: f64) -> Res<String> {
    let (vw, vh) = viewbox(node, w, h)?;''')
rep('''fn prst_geom(node: Node, default: &str) -> Res<String> {
    let w = req_num(node, "w")?;
    let h = req_num(node, "h")?;''', '''fn prst_geom(node: Node, default: &str, w: f64, h: f64) -> Res<String> {''')
rep('''    let geom = prst_geom(node, "rect")?;
    let rot = match''', '''    let geom = prst_geom(node, "rect", w, h)?;
    let rot = match''')
rep('const SHAPE_ATTRS: &[&str] = &["geom", "radius", "adj", "path", "viewbox", "fill", "line", "pad", "anchor", "wrap"];',
    'const SHAPE_ATTRS: &[&str] = &["geom", "radius", "adj", "path", "viewbox", "fill", "line", "pad", "anchor", "wrap", "fit"];')
rep('''            "shape" => (shape(c, false, ctx, base)?, bbox(c)?),
            "text" => (shape(c, true, ctx, base)?, bbox(c)?),''', '''            "shape" => shape(c, false, ctx, base)?,
            "text" => shape(c, true, ctx, base)?,''')

a = s.index("fn shape(node: Node, is_text: bool")
b = s.index("fn image(node: Node")
s = s[:a] + open('F:/pptx-ai-native/proto/calib/shape_fn.rs', encoding='utf-8').read() + "\n" + s[b:]
open(p, 'w', encoding='utf-8').write(s)
print("patched")
