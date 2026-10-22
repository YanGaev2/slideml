use crate::deck::SlideCtx;
use crate::util::*;
use roxmltree::Node;

pub const TEXT_ATTRS: &[&str] = &[
    "font", "size", "color", "bold", "italic", "underline", "spacing", "caps", "align", "line-height", "before",
    "after", "bullet", "indent", "href",
];

/// Text style. Each level (element, <p>, <span>) overrides the parent.
#[derive(Clone, Default)]
pub struct TStyle {
    font: Option<String>,
    size: Option<f64>,
    color: Option<String>,
    b: Option<bool>,
    i: Option<bool>,
    u: Option<bool>,
    spacing: Option<f64>,
    caps: Option<bool>,
    align: Option<String>,
    line: Option<f64>,
    before: Option<f64>,
    after: Option<f64>,
    bullet: Option<String>,
    indent: Option<f64>,
    href: Option<String>,
}

impl TStyle {
    pub fn base(size: f64) -> Self {
        TStyle { size: Some(size), color: Some("tx1".into()), font: Some("body".into()), ..Default::default() }
    }

    pub fn align_name(&self) -> &str {
        self.align.as_deref().unwrap_or("left")
    }

    pub fn with_bold(mut self) -> Self {
        self.b = Some(true);
        self
    }

    pub fn with(&self, n: Node) -> Res<TStyle> {
        let mut s = self.clone();
        if let Some(v) = n.attribute("font") {
            s.font = Some(v.into());
        }
        if let Some(v) = num(n, "size")? {
            s.size = Some(v);
        }
        if let Some(v) = n.attribute("color") {
            fill(v).or_else(|e| err(n, e))?;
            s.color = Some(v.into());
        }
        s.b = flag(n, "bold")?.or(s.b);
        s.i = flag(n, "italic")?.or(s.i);
        s.u = flag(n, "underline")?.or(s.u);
        s.caps = flag(n, "caps")?.or(s.caps);
        s.spacing = num(n, "spacing")?.or(s.spacing);
        s.line = num(n, "line-height")?.or(s.line);
        s.before = num(n, "before")?.or(s.before);
        s.after = num(n, "after")?.or(s.after);
        s.indent = num(n, "indent")?.or(s.indent);
        if let Some(v) = n.attribute("align") {
            if !["left", "center", "right", "justify"].contains(&v) {
                return err(n, format!("align `{v}`: use left, center, right, justify"));
            }
            s.align = Some(v.into());
        }
        if let Some(v) = n.attribute("bullet") {
            s.bullet = Some(v.into());
        }
        if let Some(v) = n.attribute("href") {
            s.href = Some(v.into());
        }
        Ok(s)
    }
}

enum Piece {
    Text(String, TStyle),
    Br(TStyle),
}

const INLINE: &[&str] = &["span", "b", "i", "u", "a", "br"];

fn collect(node: Node, st: &TStyle, out: &mut Vec<Piece>) -> Res<()> {
    for c in node.children() {
        if c.is_text() {
            out.push(Piece::Text(c.text().unwrap_or("").into(), st.clone()));
        } else if c.is_element() {
            let name = c.tag_name().name();
            let mut s = st.clone();
            match name {
                "span" => check_attrs(c, &[TEXT_ATTRS])?,
                "b" | "i" | "u" | "br" => check_attrs(c, &[])?,
                "a" => check_attrs(c, &[&["href"]])?,
                _ => return err(c, format!("not allowed inside text. Inline elements: {}", INLINE.join(", "))),
            }
            match name {
                "br" => {
                    out.push(Piece::Br(s));
                    continue;
                }
                "b" => s.b = Some(true),
                "i" => s.i = Some(true),
                "u" => s.u = Some(true),
                _ => s = s.with(c)?,
            }
            collect(c, &s, out)?;
        }
    }
    Ok(())
}

/// Collapse whitespace like HTML.
fn normalize(pieces: &mut Vec<Piece>) {
    let mut prev_space = true;
    for p in pieces.iter_mut() {
        match p {
            Piece::Text(t, _) => {
                let mut o = String::new();
                for ch in t.chars() {
                    if ch.is_whitespace() {
                        if !prev_space {
                            o.push(' ');
                        }
                        prev_space = true;
                    } else {
                        o.push(ch);
                        prev_space = false;
                    }
                }
                *t = o;
            }
            Piece::Br(_) => prev_space = true,
        }
    }
    for p in pieces.iter_mut().rev() {
        if let Piece::Text(t, _) = p {
            let trimmed = t.trim_end().to_string();
            let empty = trimmed.is_empty();
            *t = trimmed;
            if !empty {
                break;
            }
        } else {
            break;
        }
    }
    pieces.retain(|p| !matches!(p, Piece::Text(t, _) if t.is_empty()));
}

fn font_ref(f: &str) -> &str {
    match f {
        "heading" => "+mj-lt",
        "body" => "+mn-lt",
        other => other,
    }
}

fn rpr(tag: &str, s: &TStyle, lang: &str, ctx: &mut SlideCtx) -> Result<String, String> {
    let mut a = format!(" lang=\"{lang}\"");
    if let Some(v) = s.size {
        a += &format!(" sz=\"{}\"", (v * 100.0).round() as i64);
    }
    if s.b == Some(true) {
        a += " b=\"1\"";
    }
    if s.i == Some(true) {
        a += " i=\"1\"";
    }
    if s.u == Some(true) {
        a += " u=\"sng\"";
    }
    if let Some(v) = s.spacing {
        a += &format!(" spc=\"{}\"", (v * 100.0).round() as i64);
    }
    if s.caps == Some(true) {
        a += " cap=\"all\"";
    }
    let mut inner = String::new();
    if let Some(c) = &s.color {
        inner += &fill(c)?;
    }
    if let Some(f) = &s.font {
        let f = esc(font_ref(f));
        inner += &format!("<a:latin typeface=\"{f}\"/><a:cs typeface=\"{f}\"/>");
    }
    if let Some(h) = &s.href {
        let rid = ctx.rel(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
            h,
            true,
        );
        inner += &format!("<a:hlinkClick r:id=\"{rid}\"/>");
    }
    Ok(format!("<a:{tag}{a} dirty=\"0\">{inner}</a:{tag}>"))
}

fn ppr(s: &TStyle) -> String {
    let mut a = String::new();
    if let Some(al) = &s.align {
        a += &format!(
            " algn=\"{}\"",
            match al.as_str() {
                "center" => "ctr",
                "right" => "r",
                "justify" => "just",
                _ => "l",
            }
        );
    }
    let mut inner = String::new();
    if let Some(l) = s.line {
        inner += &format!("<a:lnSpc><a:spcPct val=\"{}\"/></a:lnSpc>", (l * 100000.0).round() as i64);
    }
    if let Some(v) = s.before {
        inner += &format!("<a:spcBef><a:spcPts val=\"{}\"/></a:spcBef>", (v * 100.0).round() as i64);
    }
    if let Some(v) = s.after {
        inner += &format!("<a:spcAft><a:spcPts val=\"{}\"/></a:spcAft>", (v * 100.0).round() as i64);
    }
    match s.bullet.as_deref() {
        Some(b) if !b.is_empty() && b != "none" => {
            let ind = s.indent.unwrap_or(s.size.unwrap_or(18.0) * 1.1);
            a += &format!(" marL=\"{}\" indent=\"-{}\"", emu(ind), emu(ind));
            if b == "1." {
                inner += "<a:buFontTx/><a:buAutoNum type=\"arabicPeriod\"/>";
            } else {
                inner += &format!("<a:buFontTx/><a:buChar char=\"{}\"/>", esc(b));
            }
        }
        _ => inner += "<a:buNone/>",
    }
    format!("<a:pPr{a}>{inner}</a:pPr>")
}

fn family(s: &TStyle, ctx: &SlideCtx) -> String {
    match s.font.as_deref() {
        None | Some("body") => ctx.body_font.clone(),
        Some("heading") => ctx.heading_font.clone(),
        Some(f) => f.to_string(),
    }
}

/// Splits runs so that characters missing from the run's font use a fallback font
/// (theme body font, then Segoe UI). PowerPoint would otherwise pick an unrelated font.
fn glyph_fallback(pieces: Vec<Piece>, ctx: &SlideCtx) -> Vec<Piece> {
    let mut out = Vec::new();
    for p in pieces {
        let Piece::Text(text, st) = p else {
            out.push(p);
            continue;
        };
        let fam = family(&st, ctx);
        let bold = st.b == Some(true);
        let lower = fam.to_lowercase();
        let heavy = bold || ["semibold", "bold", "black", "heavy"].iter().any(|w| lower.contains(w));
        let Some(face) = crate::fonts::face(&fam, bold) else {
            out.push(Piece::Text(text, st));
            continue;
        };
        let pick = |c: char| -> Option<String> {
            if face.has(c) {
                return None;
            }
            // Keep the visual weight: heavy text takes a heavy fallback face.
            let body = ctx.body_font.as_str();
            let base = body.split(' ').next().unwrap_or(body);
            let candidates: Vec<String> = if heavy {
                vec![format!("{base} SemiBold"), format!("{base} Bold"), format!("{base} ExtraBold"), "Inter SemiBold".into(), "Segoe UI Semibold".into(), "Segoe UI Black".into()]
            } else {
                vec![body.to_string(), "Inter".into(), "Segoe UI".into(), "Segoe UI Symbol".into()]
            };
            candidates.into_iter().find(|f| crate::fonts::face(f, false).is_some_and(|ff| ff.has(c)))
        };
        let mut cur = String::new();
        let mut cur_font: Option<String> = None;
        for c in text.chars() {
            let f = pick(c);
            if f != cur_font && !cur.is_empty() {
                let mut s2 = st.clone();
                if let Some(ff) = &cur_font {
                    s2.font = Some(ff.clone());
                }
                out.push(Piece::Text(std::mem::take(&mut cur), s2));
            }
            cur_font = f;
            cur.push(c);
        }
        if !cur.is_empty() {
            let mut s2 = st.clone();
            if let Some(ff) = &cur_font {
                s2.font = Some(ff.clone());
            }
            out.push(Piece::Text(cur, s2));
        }
    }
    out
}

fn scaled(s: &TStyle, k: f64) -> TStyle {
    let mut s = s.clone();
    s.size = s.size.map(|v| (v * k * 2.0).round() / 2.0);
    s
}

fn paragraph(node: Node, inline_root: bool, st: &TStyle, lang: &str, ctx: &mut SlideCtx) -> Res<String> {
    let mut pieces = Vec::new();
    if !inline_root {
        check_attrs(node, &[TEXT_ATTRS])?;
    }
    collect(node, st, &mut pieces)?;
    normalize(&mut pieces);
    let pieces = glyph_fallback(pieces, ctx);
    let k = ctx.text_scale;
    let st = &scaled(st, k);
    let mut body = String::new();
    let mut runs = Vec::new();
    for p in &pieces {
        match p {
            Piece::Text(t, s) => {
                let s = scaled(s, k);
                body += &format!("<a:r>{}<a:t>{}</a:t></a:r>", rpr("rPr", &s, lang, ctx).or_else(|e| err(node, e))?, esc(t));
                let text = if s.caps == Some(true) { t.to_uppercase() } else { t.clone() };
                runs.push(crate::measure::MRun { text, family: family(&s, ctx), bold: s.b == Some(true), size: s.size.unwrap_or(18.0), spacing: s.spacing.unwrap_or(0.0) });
            }
            Piece::Br(s) => {
                let s = scaled(s, k);
                body += &format!("<a:br>{}</a:br>", rpr("rPr", &s, lang, ctx).or_else(|e| err(node, e))?);
                runs.push(crate::measure::MRun { text: "\n".into(), family: family(&s, ctx), bold: false, size: s.size.unwrap_or(18.0), spacing: 0.0 });
            }
        }
    }
    let size = st.size.unwrap_or(18.0);
    let indent = match st.bullet.as_deref() {
        Some(b) if !b.is_empty() && b != "none" => st.indent.unwrap_or(size * 1.1),
        _ => 0.0,
    };
    ctx.measure.push(crate::measure::MPara { runs, line: st.line.unwrap_or(1.0), before: st.before.unwrap_or(0.0), after: st.after.unwrap_or(0.0), indent, size });
    let end = rpr("endParaRPr", st, lang, ctx).or_else(|e| err(node, e))?;
    Ok(format!("<a:p>{}{body}{end}</a:p>", ppr(st)))
}

/// Paragraphs of a text-bearing element: either <p> children, or inline content as one paragraph.
/// Measurement data for the paragraphs is appended to `ctx.measure`.
pub fn paragraphs(node: Node, st: &TStyle, lang: &str, ctx: &mut SlideCtx) -> Res<String> {
    let has_p = node.children().any(|c| c.is_element() && c.tag_name().name() == "p");
    if !has_p {
        let has_text = node.children().any(|c| c.is_element() || c.text().is_some_and(|t| !t.trim().is_empty()));
        if !has_text {
            return Ok(String::new());
        }
        return paragraph(node, true, st, lang, ctx);
    }
    let mut out = String::new();
    for c in node.children() {
        if c.is_element() {
            if c.tag_name().name() != "p" {
                return err(c, "mix of <p> and inline content: wrap all text in <p>");
            }
            out += &paragraph(c, false, &st.with(c)?, lang, ctx)?;
        } else if c.text().is_some_and(|t| !t.trim().is_empty()) {
            return err(node, "text outside <p> while <p> elements exist");
        }
    }
    Ok(out)
}
