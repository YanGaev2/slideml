use crate::anim::{self, Anim, Kind, Start};
use crate::text::{self, TStyle, TEXT_ATTRS};
use crate::svgpath;
use crate::util::*;
use roxmltree::Node;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

const NS: &str = "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"";
const XML_HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

const BOX: &[&str] = &["id", "x", "y", "w", "h", "rot", "flip"];
const FX_ATTRS: &[&str] = &["shadow", "glow", "soft-edge"];
const SHAPE_ATTRS: &[&str] = &["geom", "radius", "adj", "path", "viewbox", "fill", "line", "pad", "anchor", "wrap", "fit"];
const ICON_ATTRS: &[&str] = &["name", "color", "weight", "fill"];
const DOTS_ATTRS: &[&str] = &["step", "r", "fill"];
const WAFFLE_ATTRS: &[&str] = &["total", "value", "highlight", "cols", "gap", "radius", "shape", "fill", "accent"];
const IMAGE_ATTRS: &[&str] = &["src", "fit", "focus", "radius", "geom", "line", "opacity", "alt", "tint"];
const LINE_ATTRS: &[&str] = &["id", "x1", "y1", "x2", "y2", "stroke", "head", "tail"];
const ANIM_ATTRS: &[&str] = &["target", "effect", "start", "delay", "dur"];
const SLIDE_ATTRS: &[&str] = &["id", "bg", "transition", "transition-dur", "transition-dir"];
const THEME_ATTRS: &[&str] = &[
    "font", "heading", "dk1", "lt1", "dk2", "lt2", "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
    "hlink", "folHlink",
];
const ARROWS: &[&str] = &["none", "triangle", "stealth", "diamond", "oval", "arrow"];

pub fn rels_xml(rels: &[(String, String, String, bool)]) -> String {
    let mut s = format!("{XML_HEAD}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">");
    for (id, ty, target, ext) in rels {
        s += &format!(
            "<Relationship Id=\"{id}\" Type=\"{ty}\" Target=\"{}\"{}/>",
            esc(target),
            if *ext { " TargetMode=\"External\"" } else { "" }
        );
    }
    s + "</Relationships>"
}

pub fn zip_parts(parts: &[(String, Vec<u8>)]) -> Res<Vec<u8>> {
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, data) in parts {
        zw.start_file(name.as_str(), opts).map_err(|e| e.to_string())?;
        zw.write_all(data).map_err(|e| e.to_string())?;
    }
    Ok(zw.finish().map_err(|e| e.to_string())?.into_inner())
}

pub struct Package {
    parts: Vec<(String, Vec<u8>)>,
    overrides: Vec<(String, String)>,
    media: HashMap<PathBuf, String>,
    charts: u32,
    pub warnings: Vec<String>,
    manifest: Vec<String>,
    size: (f64, f64),
}

pub struct Output {
    pub pptx: Vec<u8>,
    pub manifest: String,
    pub warnings: Vec<String>,
}

impl Package {
    pub fn add(&mut self, name: &str, data: Vec<u8>, content_type: Option<&str>) {
        if let Some(ct) = content_type {
            self.overrides.push((format!("/{name}"), ct.into()));
        }
        self.parts.push((name.into(), data));
    }
    pub fn next_chart(&mut self) -> u32 {
        self.charts += 1;
        self.charts
    }
    fn media(&mut self, path: &Path) -> Res<String> {
        if let Some(p) = self.media.get(path) {
            return Ok(p.clone());
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let ext = match ext.as_str() {
            "jpg" | "jpeg" => "jpeg",
            "png" => "png",
            _ => return Err(format!("image `{}`: only .jpg and .png are supported", path.display())),
        };
        let data = std::fs::read(path).map_err(|e| format!("image `{}`: {e}", path.display()))?;
        let name = format!("ppt/media/image{}.{ext}", self.media.len() + 1);
        self.parts.push((name.clone(), data));
        self.media.insert(path.into(), name.clone());
        Ok(name)
    }
}

pub struct SlideCtx {
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
}

impl SlideCtx {
    pub fn warn(&mut self, node: Node, msg: impl std::fmt::Display) {
        self.warnings.push(format!("warning: slide `{}` {}: {msg}", self.slide_id, at(node)));
    }
    pub fn rel(&mut self, ty: &str, target: &str, external: bool) -> String {
        let id = format!("rId{}", self.rels.len() + 1);
        self.rels.push((id.clone(), ty.into(), target.into(), external));
        id
    }
    /// Registers a generated element (no source node), e.g. a chart annotation.
    pub fn register_named(&mut self, name: &str, kind: Kind) -> u32 {
        self.next_id += 1;
        self.names.insert(name.into(), (self.next_id, kind));
        self.next_id
    }

    pub fn register(&mut self, node: Node, kind: Kind) -> Res<(u32, String)> {
        self.next_id += 1;
        let id = self.next_id;
        let name = match node.attribute("id") {
            Some(n) => {
                if self.names.insert(n.into(), (id, kind)).is_some() {
                    return err(node, format!("duplicate id `{n}` on this slide"));
                }
                n.to_string()
            }
            None => format!("{} {id}", node.tag_name().name()),
        };
        Ok((id, name))
    }
}

struct Theme {
    body: String,
    heading: String,
    colors: Vec<(&'static str, String)>,
}

fn theme(node: Option<Node>) -> Res<Theme> {
    let defaults = [
        ("dk1", "#111418"), ("lt1", "#FFFFFF"), ("dk2", "#1D2433"), ("lt2", "#F2F4F7"),
        ("accent1", "#2F6BFF"), ("accent2", "#FF7A1A"), ("accent3", "#18B39B"), ("accent4", "#8B5CF6"),
        ("accent5", "#F2B705"), ("accent6", "#E5484D"), ("hlink", "#2F6BFF"), ("folHlink", "#8B5CF6"),
    ];
    let mut t = Theme { body: "Inter".into(), heading: "Inter SemiBold".into(), colors: vec![] };
    if let Some(n) = node {
        check_attrs(n, &[THEME_ATTRS])?;
        if let Some(v) = n.attribute("font") {
            t.body = v.into();
        }
        if let Some(v) = n.attribute("heading") {
            t.heading = v.into();
        }
    }
    for (slot, def) in defaults {
        let v = node.and_then(|n| n.attribute(slot)).unwrap_or(def);
        let hex = v.strip_prefix('#').filter(|h| h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit()));
        match hex {
            Some(h) => t.colors.push((slot, h.to_uppercase())),
            None => return err(node.unwrap(), format!("theme color `{slot}` must be #RRGGBB")),
        }
    }
    Ok(t)
}

fn xfrm(node: Node, tag: &str) -> Res<String> {
    xfrm_h(node, tag, req_num(node, "h")?)
}

fn xfrm_h(node: Node, tag: &str, h: f64) -> Res<String> {
    let (x, y, w) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?);
    if w <= 0.0 || h <= 0.0 {
        return err(node, "w and h must be positive");
    }
    let mut a = String::new();
    if let Some(r) = num(node, "rot")? {
        a += &format!(" rot=\"{}\"", (r * 60000.0).round() as i64);
    }
    match node.attribute("flip") {
        None => {}
        Some("h") => a += " flipH=\"1\"",
        Some("v") => a += " flipV=\"1\"",
        Some("hv") => a += " flipH=\"1\" flipV=\"1\"",
        Some(v) => return err(node, format!("flip `{v}`: use h, v, hv")),
    }
    Ok(format!("<{tag}{a}><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></{tag}>", emu(x), emu(y), emu(w), emu(h)))
}

fn fx(node: Node) -> Res<String> {
    effects(node.attribute("shadow"), node.attribute("glow"), num(node, "soft-edge")?).or_else(|m| err(node, m))
}

fn viewbox(node: Node, w: f64, h: f64) -> Res<(f64, f64)> {
    match node.attribute("viewbox") {
        Some(vb) => {
            let v: Vec<f64> = vb.split([' ', ',']).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap_or(f64::NAN)).collect();
            if v.len() != 4 || v.iter().any(|x| x.is_nan()) {
                return err(node, "viewbox must be `0 0 W H`");
            }
            Ok((v[2], v[3]))
        }
        None => Ok((w, h)),
    }
}

/// SVG path syntax (M L H V C S Q T A Z, absolute or relative) -> a:custGeom.
fn cust_geom(node: Node, d: &str, w: f64, h: f64) -> Res<String> {
    let (vw, vh) = viewbox(node, w, h)?;
    let segs = svgpath::parse(d).or_else(|m| err(node, m))?;
    Ok(svgpath::cust_geom(&svgpath::path_xml(&segs, vw, vh, 1000.0, true, true)))
}

fn icon(node: Node, ctx: &mut SlideCtx) -> Res<String> {
    check_attrs(node, &[BOX, ICON_ATTRS, FX_ATTRS])?;
    let (id, name) = ctx.register(node, Kind::Shape)?;
    let want = node.attribute("name").map_or_else(|| err(node, "missing `name` (a Lucide icon name)"), Ok)?;
    let svg = match crate::icons::ICONS.binary_search_by(|(n, _)| (*n).cmp(want)) {
        Ok(i) => crate::icons::ICONS[i].1,
        Err(_) => {
            let near: Vec<&str> = crate::icons::ICONS.iter().map(|(n, _)| *n).filter(|n| n.contains(want) || want.split('-').any(|p| p.len() > 2 && n.contains(p))).take(12).collect();
            return err(node, format!("unknown icon `{want}`. Similar: {}", if near.is_empty() { "-".into() } else { near.join(", ") }));
        }
    };
    let doc = roxmltree::Document::parse(svg).map_err(|e| e.to_string())?;
    let filled = node.attribute("fill").is_some_and(|f| f != "none");
    let paths: String = svgpath::svg_elements(&doc.root_element())
        .or_else(|m| err(node, m))?
        .iter()
        .map(|(segs, closed)| svgpath::path_xml(segs, 24.0, 24.0, 1000.0, filled && *closed, true))
        .collect();
    let (w, h) = (req_num(node, "w")?, req_num(node, "h")?);
    let stroke_w = num(node, "weight")?.unwrap_or(2.0) * w.min(h) / 24.0;
    let col = node.attribute("color").unwrap_or("tx1");
    let fill_xml = fill(node.attribute("fill").unwrap_or("none")).or_else(|m| err(node, m))?;
    let ln = line(Some(&format!("{col} {stroke_w}")), "").or_else(|m| err(node, m))?;
    Ok(format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{}{}{fill_xml}{ln}{}</p:spPr><p:txBody><a:bodyPr rtlCol=\"0\" anchor=\"ctr\"/><a:lstStyle/><a:p><a:endParaRPr lang=\"{}\" dirty=\"0\"/></a:p></p:txBody></p:sp>",
        esc(&name),
        xfrm(node, "a:xfrm")?,
        svgpath::cust_geom(&paths),
        fx(node)?,
        ctx.lang
    ))
}

/// Unit chart: `total` cells in a grid; `value` cells (or the 1-based list in `highlight`) are accented.
/// Writes two native shapes: `<id>` (base cells) and `<id>-hi` (accented cells), so each can be
/// animated or used as a Morph target.
fn waffle(node: Node, ctx: &mut SlideCtx) -> Res<String> {
    check_attrs(node, &[BOX, WAFFLE_ATTRS])?;
    let (x, y, w, h) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?, req_num(node, "h")?);
    let total = num(node, "total")?.unwrap_or(100.0).round() as usize;
    if total == 0 || total > 2000 {
        return err(node, "total must be 1..2000");
    }
    let hi: Vec<usize> = match (node.attribute("highlight"), num(node, "value")?) {
        (Some(list), _) => list
            .split(',')
            .map(|s| s.trim().parse::<usize>())
            .collect::<Result<Vec<_>, _>>()
            .or_else(|_| err(node, "highlight: comma-separated cell numbers from 1"))?,
        (None, Some(v)) => (1..=(v.round().max(0.0) as usize).min(total)).collect(),
        (None, None) => return err(node, "set `value` (number of accented cells) or `highlight` (cell numbers)"),
    };
    if hi.iter().any(|&i| i == 0 || i > total) {
        return err(node, format!("highlight cells must be 1..{total}"));
    }
    let gap = num(node, "gap")?.unwrap_or(4.0);
    let cols = match num(node, "cols")? {
        Some(c) => c.max(1.0) as usize,
        None => ((total as f64 * w / h).sqrt().round() as usize).clamp(1, total),
    };
    let rows = total.div_ceil(cols);
    let cell = ((w - gap * (cols as f64 - 1.0)) / cols as f64).min((h - gap * (rows as f64 - 1.0)) / rows as f64);
    if cell < 1.0 {
        return err(node, "cells are too small: enlarge the box or reduce total/gap");
    }
    let round = node.attribute("shape").unwrap_or("square");
    if !["square", "circle"].contains(&round) {
        return err(node, "shape: square or circle");
    }
    let radius = num(node, "radius")?.unwrap_or(cell * 0.12);
    let (mut base, mut accent) = (Vec::new(), Vec::new());
    for i in 1..=total {
        let (r, c) = ((i - 1) / cols, (i - 1) % cols);
        let (cx, cy) = (c as f64 * (cell + gap), r as f64 * (cell + gap));
        let segs = if round == "circle" {
            svgpath::ellipse(cx + cell / 2.0, cy + cell / 2.0, cell / 2.0, cell / 2.0)
        } else {
            svgpath::rect(cx, cy, cell, cell, radius, radius)
        };
        if hi.contains(&i) { accent.extend(segs) } else { base.extend(segs) }
    }
    let gw = cols as f64 * cell + (cols as f64 - 1.0) * gap;
    let gh = rows as f64 * cell + (rows as f64 - 1.0) * gap;
    let mut out = String::new();
    let base_id = node.attribute("id").unwrap_or("waffle").to_string();
    for (segs, suffix, fill_attr, default) in [(&base, "", "fill", "lt2"), (&accent, "-hi", "accent", "accent1")] {
        if segs.is_empty() {
            continue;
        }
        let name = format!("{base_id}{suffix}");
        let id = ctx.register_named(&name, Kind::Shape);
        let f = fill(node.attribute(fill_attr).unwrap_or(default)).or_else(|m| err(node, m))?;
        out += &format!(
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>{}{f}<a:ln><a:noFill/></a:ln></p:spPr><p:txBody><a:bodyPr rtlCol=\"0\" anchor=\"ctr\"/><a:lstStyle/><a:p><a:endParaRPr lang=\"{}\" dirty=\"0\"/></a:p></p:txBody></p:sp>",
            esc(&name),
            emu(x),
            emu(y),
            emu(gw),
            emu(gh),
            svgpath::cust_geom(&svgpath::path_xml(segs, gw, gh, 100.0, true, false)),
            ctx.lang
        );
    }
    Ok(out)
}

/// Halftone dot field: one shape, one path with many circles.
fn dots(node: Node, ctx: &mut SlideCtx) -> Res<String> {
    check_attrs(node, &[BOX, DOTS_ATTRS])?;
    let (id, name) = ctx.register(node, Kind::Shape)?;
    let (w, h) = (req_num(node, "w")?, req_num(node, "h")?);
    let step = num(node, "step")?.unwrap_or(12.0);
    let r = num(node, "r")?.unwrap_or(1.5);
    if step <= 2.0 * r || (w / step) * (h / step) > 6000.0 {
        return err(node, "step must exceed 2*r, and the field must hold at most 6000 dots");
    }
    let mut segs = Vec::new();
    let mut y = step / 2.0;
    while y < h {
        let mut x = step / 2.0;
        while x < w {
            segs.extend(svgpath::ellipse(x, y, r, r));
            x += step;
        }
        y += step;
    }
    let fill_xml = fill(node.attribute("fill").unwrap_or("tx1/20")).or_else(|m| err(node, m))?;
    Ok(format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{}{}{fill_xml}<a:ln><a:noFill/></a:ln></p:spPr><p:txBody><a:bodyPr rtlCol=\"0\" anchor=\"ctr\"/><a:lstStyle/><a:p><a:endParaRPr lang=\"{}\" dirty=\"0\"/></a:p></p:txBody></p:sp>",
        esc(&name),
        xfrm(node, "a:xfrm")?,
        svgpath::cust_geom(&svgpath::path_xml(&segs, w, h, 100.0, true, false)),
        ctx.lang
    ))
}

fn prst_geom(node: Node, default: &str, w: f64, h: f64) -> Res<String> {
    let radius = num(node, "radius")?;
    let geom = node.attribute("geom").unwrap_or(if radius.is_some() { "roundRect" } else { default });
    if !crate::presets::PRESETS.contains(&geom) {
        let near: Vec<&str> = crate::presets::PRESETS.iter().copied().filter(|p| p.to_lowercase().contains(&geom.to_lowercase())).take(8).collect();
        return err(node, format!("unknown geom `{geom}`. Similar: {}", if near.is_empty() { "-".into() } else { near.join(", ") }));
    }
    let mut av = String::new();
    if let Some(r) = radius {
        if geom != "roundRect" {
            return err(node, "radius works only with geom=\"roundRect\"");
        }
        let adj = (r / w.min(h) * 100000.0).round().min(50000.0) as i64;
        av += &format!("<a:gd name=\"adj\" fmla=\"val {adj}\"/>");
    }
    if let Some(a) = node.attribute("adj") {
        for kv in a.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let (k, v) = kv.split_once('=').ok_or_else(|| format!("{}: adj must be `name=value;...`", at(node)))?;
            let v: i64 = v.trim().parse().map_err(|_| format!("{}: adj value `{v}` must be an integer", at(node)))?;
            av += &format!("<a:gd name=\"{}\" fmla=\"val {v}\"/>", esc(k.trim()));
        }
    }
    Ok(format!("<a:prstGeom prst=\"{geom}\"><a:avLst>{av}</a:avLst></a:prstGeom>"))
}

fn pad(node: Node, default: [f64; 4]) -> Res<[f64; 4]> {
    let Some(p) = node.attribute("pad") else { return Ok(default) };
    let v: Vec<f64> = p.split_whitespace().map(|s| s.trim_end_matches("pt").parse::<f64>()).collect::<Result<_, _>>().or_else(|_| err(node, "pad: 1, 2 or 4 numbers"))?;
    Ok(match v.len() {
        1 => [v[0]; 4],
        2 => [v[0], v[1], v[0], v[1]],
        4 => [v[0], v[1], v[2], v[3]],
        _ => return err(node, "pad: 1, 2 or 4 numbers (top right bottom left)"),
    })
}

fn shape(node: Node, is_text: bool, ctx: &mut SlideCtx, base: &TStyle) -> Res<(String, BBox)> {
    check_attrs(node, &[BOX, SHAPE_ATTRS, FX_ATTRS, TEXT_ATTRS])?;
    let (id, name) = ctx.register(node, Kind::Shape)?;
    let e = |r: Result<String, String>| r.or_else(|m| err(node, m));
    let (x, y, w) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?);
    let auto_h = node.attribute("h") == Some("auto");
    let fill_xml = e(fill(node.attribute("fill").unwrap_or(if is_text { "none" } else { "accent1" })))?;
    let line_xml = e(line(node.attribute("line"), ""))?;
    let fx = fx(node)?;
    let [t, r, b, l] = pad(node, if is_text { [0.0; 4] } else { [3.6, 7.2, 3.6, 7.2] })?;
    let anchor = match node.attribute("anchor").unwrap_or(if is_text { "top" } else { "middle" }) {
        "top" => "t",
        "middle" => "ctr",
        "bottom" => "b",
        v => return err(node, format!("anchor `{v}`: use top, middle, bottom")),
    };
    let wrap = match node.attribute("wrap") {
        None | Some("1") => "square",
        Some("0") => "none",
        Some(v) => return err(node, format!("wrap `{v}`: use 1 or 0")),
    };
    let fit = node.attribute("fit").unwrap_or("none");
    if !["none", "shrink"].contains(&fit) {
        return err(node, format!("fit `{fit}`: use none or shrink"));
    }
    if auto_h && fit == "shrink" {
        return err(node, "h=\"auto\" and fit=\"shrink\" exclude each other");
    }
    let given_h = if auto_h { None } else { Some(req_num(node, "h")?) };
    let st = base.with(node)?;
    let lang = ctx.lang.clone();
    let avail_w = (wrap == "square").then_some(w - l - r);
    // Lay out; with fit="shrink" reduce all font sizes until the text fits.
    let mut k = 1.0;
    let (paras, lay) = loop {
        ctx.text_scale = k;
        ctx.measure.clear();
        let paras = text::paragraphs(node, &st, &lang, ctx)?;
        let lay = crate::measure::layout(&ctx.measure, avail_w);
        if fit == "shrink" && k > 0.5 && given_h.is_some_and(|h| lay.height > h - t - b + 0.5) {
            k -= 0.02;
            continue;
        }
        break (paras, lay);
    };
    ctx.text_scale = 1.0;
    let min_size = ctx.measure.iter().flat_map(|p| p.runs.iter().map(|r| r.size)).fold(f64::INFINITY, f64::min);
    let has_text = !paras.is_empty();
    let h = given_h.unwrap_or((lay.height + t + b + 1.0).ceil());
    if has_text {
        if lay.height > h - t - b + 0.5 {
            let need = lay.height + t + b;
            ctx.warn(node, format!("text overflows: needs {need:.1}pt height ({} lines), box has {h:.1}pt. Fix: h=\"{:.0}\", h=\"auto\" or fit=\"shrink\"", lay.lines, need.ceil()));
        }
        if wrap == "none" && lay.max_line_width > w - l - r + 0.5 {
            ctx.warn(node, format!("text is {:.1}pt wide, box has {w:.1}pt (wrap=\"0\")", lay.max_line_width + l + r));
        }
        if lay.split_words > 0 {
            ctx.warn(node, format!("{} word(s) are wider than the box and break between letters. Fix: widen w, reduce size, or use fit=\"shrink\"", lay.split_words));
        }
        if min_size < 9.0 {
            ctx.warn(node, format!("font size {min_size}pt is unreadable on a projector; use 10pt or more"));
        }
        if k < 1.0 {
            ctx.warn(node, format!("fit=\"shrink\" scaled the text to {:.0}%", k * 100.0));
        }
    }
    let geom = match node.attribute("path") {
        Some(d) => cust_geom(node, d, w, h)?,
        None => prst_geom(node, "rect", w, h)?,
    };
    ctx.items.push(crate::lint::Item {
        id: name.clone(),
        kind: if is_text { "text" } else { "shape" },
        bbox: (x, y, x + w, y + h),
        text: has_text.then_some(crate::lint::TextInfo { width: lay.max_line_width, align: st.align_name().to_string(), lines: lay.lines, height: lay.height, avail: h - t - b, min_size }),
        depth: ctx.depth,
        pos: pos_of(node),
        tag: node.tag_name().name().to_string(),
        filled: node.attribute("fill").unwrap_or(if is_text { "none" } else { "accent1" }) != "none",
    });
    let paras = if paras.is_empty() { format!("<a:p><a:endParaRPr lang=\"{}\" dirty=\"0\"/></a:p>", ctx.lang) } else { paras };
    Ok((
        format!(
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr{}/><p:nvPr/></p:nvSpPr><p:spPr>{}{geom}{fill_xml}{line_xml}{fx}</p:spPr><p:txBody><a:bodyPr wrap=\"{wrap}\" lIns=\"{}\" tIns=\"{}\" rIns=\"{}\" bIns=\"{}\" anchor=\"{anchor}\" rtlCol=\"0\"><a:noAutofit/></a:bodyPr><a:lstStyle/>{paras}</p:txBody></p:sp>",
            esc(&name),
            if is_text { " txBox=\"1\"" } else { "" },
            xfrm_h(node, "a:xfrm", h)?,
            emu(l),
            emu(t),
            emu(r),
            emu(b)
        ),
        (x, y, x + w, y + h),
    ))
}

fn image(node: Node, ctx: &mut SlideCtx, pkg: &mut Package, base_dir: &Path) -> Res<String> {
    check_attrs(node, &[BOX, IMAGE_ATTRS, FX_ATTRS])?;
    let (id, name) = ctx.register(node, Kind::Other)?;
    let src = node.attribute("src").map_or_else(|| err(node, "missing `src`"), Ok)?;
    let path = base_dir.join(src);
    let part = pkg.media(&path).or_else(|m| err(node, m))?;
    let dim = imagesize::size(&path).or_else(|e| err(node, format!("cannot read size of `{src}`: {e}")))?;
    let (iw, ih) = (dim.width as f64, dim.height as f64);
    let (mut x, mut y, mut w, mut h) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?, req_num(node, "h")?);
    let mut src_rect = String::new();
    match node.attribute("fit").unwrap_or("cover") {
        "cover" => {
            let (fx, fy) = match node.attribute("focus") {
                Some(f) => {
                    let v: Vec<f64> = f.split_whitespace().map(|s| s.trim_end_matches('%').parse::<f64>().unwrap_or(f64::NAN) / 100.0).collect();
                    if v.len() != 2 || v.iter().any(|x| x.is_nan()) {
                        return err(node, "focus must be `X% Y%`");
                    }
                    (v[0], v[1])
                }
                None => (0.5, 0.5),
            };
            let (ia, ba) = (iw / ih, w / h);
            if ia > ba {
                let crop = 1.0 - ba / ia;
                src_rect = format!(" l=\"{}\" r=\"{}\"", (crop * fx * 100000.0).round() as i64, (crop * (1.0 - fx) * 100000.0).round() as i64);
            } else if ia < ba {
                let crop = 1.0 - ia / ba;
                src_rect = format!(" t=\"{}\" b=\"{}\"", (crop * fy * 100000.0).round() as i64, (crop * (1.0 - fy) * 100000.0).round() as i64);
            }
        }
        "contain" => {
            let s = (w / iw).min(h / ih);
            let (nw, nh) = (iw * s, ih * s);
            x += (w - nw) / 2.0;
            y += (h - nh) / 2.0;
            w = nw;
            h = nh;
        }
        "fill" => {}
        v => return err(node, format!("fit `{v}`: use cover, contain, fill")),
    }
    let rid = ctx.rel(&format!("{REL}/image"), &format!("../{}", part.trim_start_matches("ppt/")), false);
    let alpha = match num(node, "opacity")? {
        Some(o) => format!("<a:alphaModFix amt=\"{}\"/>", (o * 1000.0).round() as i64),
        None => String::new(),
    };
    let alpha = alpha + &match node.attribute("tint") {
        None => String::new(),
        Some("grayscale") => "<a:grayscl/>".into(),
        Some(t) => match t.strip_prefix("duotone(").and_then(|r| r.strip_suffix(')')).and_then(|r| r.split_once(',')) {
            Some((a, b)) => format!("<a:duotone>{}{}</a:duotone>", color(a).or_else(|m| err(node, m))?, color(b).or_else(|m| err(node, m))?),
            None => return err(node, "tint must be `grayscale` or `duotone(dark, light)`"),
        },
    };
    let e = |r: Result<String, String>| r.or_else(|m| err(node, m));
    let geom = prst_geom(node, "rect", w, h)?;
    let rot = match num(node, "rot")? {
        Some(r) => format!(" rot=\"{}\"", (r * 60000.0).round() as i64),
        None => String::new(),
    };
    let line_xml = if node.attribute("line").is_some() { e(line(node.attribute("line"), ""))? } else { String::new() };
    Ok(format!(
        "<p:pic><p:nvPicPr><p:cNvPr id=\"{id}\" name=\"{}\" descr=\"{}\"/><p:cNvPicPr><a:picLocks noChangeAspect=\"1\"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed=\"{rid}\">{alpha}</a:blip><a:srcRect{src_rect}/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm{rot}><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>{geom}{line_xml}{}</p:spPr></p:pic>",
        esc(&name),
        esc(node.attribute("alt").unwrap_or("")),
        emu(x), emu(y), emu(w), emu(h),
        fx(node)?
    ))
}

fn connector(node: Node, ctx: &mut SlideCtx) -> Res<String> {
    check_attrs(node, &[LINE_ATTRS])?;
    let (id, name) = ctx.register(node, Kind::Other)?;
    let (x1, y1, x2, y2) = (req_num(node, "x1")?, req_num(node, "y1")?, req_num(node, "x2")?, req_num(node, "y2")?);
    let mut ends = String::new();
    for (attr, tag) in [("head", "headEnd"), ("tail", "tailEnd")] {
        if let Some(v) = node.attribute(attr) {
            if !ARROWS.contains(&v) {
                return err(node, format!("{attr} `{v}`. Valid: {}", ARROWS.join(", ")));
            }
            ends += &format!("<a:{tag} type=\"{v}\" w=\"med\" len=\"med\"/>");
        }
    }
    let ln = line(Some(node.attribute("stroke").unwrap_or("tx1 1")), &ends).or_else(|m| err(node, m))?;
    let flip = format!("{}{}", if x2 < x1 { " flipH=\"1\"" } else { "" }, if y2 < y1 { " flipV=\"1\"" } else { "" });
    Ok(format!(
        "<p:cxnSp><p:nvCxnSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr><p:spPr><a:xfrm{flip}><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"line\"><a:avLst/></a:prstGeom>{ln}</p:spPr></p:cxnSp>",
        esc(&name),
        emu(x1.min(x2)), emu(y1.min(y2)), emu((x2 - x1).abs()), emu((y2 - y1).abs())
    ))
}

fn transition(node: Node) -> Res<String> {
    let Some(kind) = node.attribute("transition") else { return Ok(String::new()) };
    let dur = num(node, "transition-dur")?.unwrap_or(700.0) as i64;
    let dir = node.attribute("transition-dir");
    let d = |allowed: &[&str], def: &str| -> Res<String> {
        let v = dir.unwrap_or(def);
        if allowed.contains(&v) { Ok(v.to_string()) } else { err(node, format!("transition-dir `{v}`. Valid: {}", allowed.join(", "))) }
    };
    let spd = if dur < 500 { "fast" } else if dur < 1000 { "med" } else { "slow" };
    let (inner, req, fallback) = match kind {
        "fade" => ("<p:fade/>".to_string(), "p14", "<p:fade/>".to_string()),
        "push" => {
            let x = format!("<p:push dir=\"{}\"/>", d(&["u", "d", "l", "r"], "u")?);
            (x.clone(), "p14", x)
        }
        "wipe" => {
            let x = format!("<p:wipe dir=\"{}\"/>", d(&["u", "d", "l", "r"], "r")?);
            (x.clone(), "p14", x)
        }
        "morph" => ("<p159:morph option=\"byObject\"/>".to_string(), "p159", "<p:fade/>".to_string()),
        v => return err(node, format!("transition `{v}`. Valid: fade, push, wipe, morph")),
    };
    let ns = if req == "p159" { " xmlns:p159=\"http://schemas.microsoft.com/office/powerpoint/2015/09/main\"" } else { "" };
    Ok(format!(
        "<mc:AlternateContent xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\"><mc:Choice{ns} Requires=\"{req}\"><p:transition spd=\"{spd}\" p14:dur=\"{dur}\">{inner}</p:transition></mc:Choice><mc:Fallback><p:transition spd=\"{spd}\">{fallback}</p:transition></mc:Fallback></mc:AlternateContent>"
    ))
}

type BBox = (f64, f64, f64, f64);

fn pos_of(n: Node) -> String {
    src_pos(n)
}

fn bbox(c: Node) -> Res<BBox> {
    if c.tag_name().name() == "line" {
        let (x1, y1, x2, y2) = (req_num(c, "x1")?, req_num(c, "y1")?, req_num(c, "x2")?, req_num(c, "y2")?);
        return Ok((x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2)));
    }
    let (x, y) = (req_num(c, "x")?, req_num(c, "y")?);
    Ok((x, y, x + req_num(c, "w")?, y + req_num(c, "h")?))
}

/// Writes child elements of a slide or a group. Returns XML and the union bounding box.
fn elements<'a, 'i>(
    parent: Node<'a, 'i>,
    ctx: &mut SlideCtx,
    pkg: &mut Package,
    base_dir: &Path,
    th: &Theme,
    base: &TStyle,
    mut anim_nodes: Option<&mut Vec<Node<'a, 'i>>>,
) -> Res<(String, Option<BBox>)> {
    let mut tree = String::new();
    let mut bb: Option<BBox> = None;
    for c in parent.children().filter(|c| c.is_element()) {
        let name = c.tag_name().name();
        if name == "anim" {
            match anim_nodes.as_deref_mut() {
                Some(v) => v.push(c),
                None => return err(c, "<anim> must be a direct child of <slide>"),
            }
            continue;
        }
        let (xml, b) = match name {
            "shape" => shape(c, false, ctx, base)?,
            "text" => shape(c, true, ctx, base)?,
            "image" => (image(c, ctx, pkg, base_dir)?, bbox(c)?),
            "line" => (connector(c, ctx)?, bbox(c)?),
            "icon" => (icon(c, ctx)?, bbox(c)?),
            "dots" => (dots(c, ctx)?, bbox(c)?),
            "waffle" => (waffle(c, ctx)?, bbox(c)?),
            "table" => (crate::table::table(c, ctx, base)?, bbox(c)?),
            "chart" => {
                check_attrs(c, &[BOX, crate::chart::CHART_ATTRS])?;
                let (id, name) = ctx.register(c, Kind::Graphic)?;
                let (g, overlays) = crate::chart::chart(c, ctx, pkg, &th.body)?;
                let (under, overlays) = overlays.split_once('\u{1}').map(|(u, o)| (u.to_string(), o.to_string())).unwrap_or((String::new(), overlays));
                (format!(
                    "{under}<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp=\"1\"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>{}{g}</p:graphicFrame>{overlays}",
                    esc(&name),
                    xfrm(c, "p:xfrm")?
                ), bbox(c)?)
            }
            "group" => {
                check_attrs(c, &[&["id"]])?;
                let (id, gname) = ctx.register(c, Kind::Other)?;
                ctx.depth += 1;
                let (inner, gb) = elements(c, ctx, pkg, base_dir, th, base, None)?;
                ctx.depth -= 1;
                let Some(gb) = gb else { return err(c, "empty <group>") };
                let (x, y, w, h) = (emu(gb.0), emu(gb.1), emu(gb.2 - gb.0), emu(gb.3 - gb.1));
                (format!(
                    "<p:grpSp><p:nvGrpSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"{x}\" y=\"{y}\"/><a:ext cx=\"{w}\" cy=\"{h}\"/><a:chOff x=\"{x}\" y=\"{y}\"/><a:chExt cx=\"{w}\" cy=\"{h}\"/></a:xfrm></p:grpSpPr>{inner}</p:grpSp>",
                    esc(&gname)
                ), gb)
            }
            other => return err(c, format!("<{other}> is not a slide element. Valid: shape, text, image, icon, dots, waffle, line, chart, table, group, anim")),
        };
        if name != "shape" && name != "text" {
            let id = c.attribute("id").map(String::from).unwrap_or_else(|| name.to_string());
            ctx.items.push(crate::lint::Item { id, kind: if name == "group" { "group" } else { "other" }, bbox: b, text: None, depth: ctx.depth, pos: pos_of(c), tag: name.to_string(), filled: false });
        }
        tree += &xml;
        bb = Some(match bb {
            None => b,
            Some(a) => (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)),
        });
    }
    Ok((tree, bb))
}

fn slide(node: Node, idx: usize, pkg: &mut Package, base_dir: &Path, lang: &str, th: &Theme) -> Res<()> {
    check_attrs(node, &[SLIDE_ATTRS])?;
    let mut ctx = SlideCtx {
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
    };
    ctx.rel(&format!("{REL}/slideLayout"), "../slideLayouts/slideLayout1.xml", false);
    let bg = match node.attribute("bg") {
        Some(b) => format!("<p:bg><p:bgPr>{}<a:effectLst/></p:bgPr></p:bg>", fill(b).or_else(|m| err(node, m))?),
        None => String::new(),
    };
    let base = TStyle::base(18.0);
    let mut anim_nodes = Vec::new();
    let (tree, _) = elements(node, &mut ctx, pkg, base_dir, th, &base, Some(&mut anim_nodes))?;
    let mut anims = Vec::new();
    for a in anim_nodes {
        check_attrs(a, &[ANIM_ATTRS])?;
        let target = a.attribute("target").map_or_else(|| err(a, "missing `target`"), Ok)?;
        let &(spid, kind) = ctx.names.get(target).map_or_else(|| err(a, format!("target `{target}` is not an id on this slide")), Ok)?;
        let effect = a.attribute("effect").unwrap_or("fade");
        if !anim::EFFECTS.contains(&effect) {
            return err(a, format!("effect `{effect}`. Valid: {}", anim::EFFECTS.join(", ")));
        }
        let start = match a.attribute("start").unwrap_or("click") {
            "click" => Start::Click,
            "with" => Start::With,
            "after" => Start::After,
            v => return err(a, format!("start `{v}`: use click, with, after")),
        };
        anims.push(Anim {
            spid,
            kind,
            effect: effect.into(),
            start,
            delay: num(a, "delay")?.unwrap_or(0.0) as u32,
            dur: num(a, "dur")?.unwrap_or(500.0) as u32,
        });
    }
    let xml = format!(
        "{XML_HEAD}<p:sld {NS} xmlns:p14=\"http://schemas.microsoft.com/office/powerpoint/2010/main\"><p:cSld>{bg}<p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>{tree}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>{}{}</p:sld>",
        transition(node)?,
        anim::timing(&anims)
    );
    let lints = crate::lint::lint(&ctx.slide_id, &ctx.items, pkg.size.0, pkg.size.1);
    pkg.warnings.append(&mut ctx.warnings);
    pkg.warnings.extend(lints);
    pkg.manifest.push(crate::lint::slide_json(&ctx.slide_id, &ctx.items));
    pkg.add(&format!("ppt/slides/slide{idx}.xml"), xml.into_bytes(), Some("application/vnd.openxmlformats-officedocument.presentationml.slide+xml"));
    pkg.add(&format!("ppt/slides/_rels/slide{idx}.xml.rels"), rels_xml(&ctx.rels).into_bytes(), None);
    Ok(())
}

fn theme_xml(t: &Theme) -> String {
    let clr: String = t.colors.iter().map(|(s, v)| format!("<a:{s}><a:srgbClr val=\"{v}\"/></a:{s}>")).collect();
    let ph = "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>";
    format!(
        "{XML_HEAD}<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"SlideML\"><a:themeElements><a:clrScheme name=\"SlideML\">{clr}</a:clrScheme><a:fontScheme name=\"SlideML\"><a:majorFont><a:latin typeface=\"{h}\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:majorFont><a:minorFont><a:latin typeface=\"{b}\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:minorFont></a:fontScheme><a:fmtScheme name=\"SlideML\"><a:fillStyleLst>{ph}{ph}{ph}</a:fillStyleLst><a:lnStyleLst><a:ln w=\"6350\">{ph}</a:ln><a:ln w=\"12700\">{ph}</a:ln><a:ln w=\"19050\">{ph}</a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst>{ph}{ph}{ph}</a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>",
        h = esc(&t.heading),
        b = esc(&t.body)
    )
}

const SP_TREE_EMPTY: &str = "<p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr></p:spTree>";

fn master_xml() -> String {
    let lvl = |font: &str, sz: u32| format!("<a:lvl1pPr><a:defRPr sz=\"{sz}\" kern=\"1200\"><a:solidFill><a:schemeClr val=\"tx1\"/></a:solidFill><a:latin typeface=\"{font}\"/><a:ea typeface=\"+mn-ea\"/><a:cs typeface=\"+mn-cs\"/></a:defRPr></a:lvl1pPr>");
    format!(
        "{XML_HEAD}<p:sldMaster {NS}><p:cSld><p:bg><p:bgRef idx=\"1001\"><a:schemeClr val=\"bg1\"/></p:bgRef></p:bg>{SP_TREE_EMPTY}</p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/><p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rId1\"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle>{}</p:titleStyle><p:bodyStyle>{}</p:bodyStyle><p:otherStyle>{}</p:otherStyle></p:txStyles></p:sldMaster>",
        lvl("+mj-lt", 4000),
        lvl("+mn-lt", 1800),
        lvl("+mn-lt", 1800)
    )
}

pub fn compile(path: &Path) -> Res<Output> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let src = crate::expand::expand(&src).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc = roxmltree::Document::parse(&src).map_err(|e| format!("{}: XML error after template expansion: {e}", path.display()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "deck" {
        return err(root, "root element must be <deck>");
    }
    check_attrs(root, &[&["w", "h", "lang"]])?;
    let (w, h) = (num(root, "w")?.unwrap_or(960.0), num(root, "h")?.unwrap_or(540.0));
    let lang = root.attribute("lang").unwrap_or("en-US").to_string();
    let base_dir = path.parent().unwrap_or(Path::new("."));
    let th = theme(root.children().find(|c| c.is_element() && c.tag_name().name() == "theme"))?;

    let mut pkg = Package { parts: vec![], overrides: vec![], media: HashMap::new(), charts: 0, warnings: vec![], manifest: vec![], size: (w, h) };
    let slides: Vec<Node> = root.children().filter(|c| c.is_element() && c.tag_name().name() == "slide").collect();
    for c in root.children().filter(|c| c.is_element()) {
        if !["slide", "theme"].contains(&c.tag_name().name()) {
            return err(c, "only <theme> and <slide> are allowed in <deck>");
        }
    }
    if slides.is_empty() {
        return err(root, "deck has no <slide>");
    }
    for (i, s) in slides.iter().enumerate() {
        slide(*s, i + 1, &mut pkg, base_dir, &lang, &th)?;
    }

    let n = slides.len();
    let mut pres_rels = vec![("rId1".to_string(), format!("{REL}/slideMaster"), "slideMasters/slideMaster1.xml".to_string(), false)];
    let mut sld_ids = String::new();
    for i in 1..=n {
        pres_rels.push((format!("rId{}", i + 1), format!("{REL}/slide"), format!("slides/slide{i}.xml"), false));
        sld_ids += &format!("<p:sldId id=\"{}\" r:id=\"rId{}\"/>", 255 + i, i + 1);
    }
    for (k, (ty, target)) in [("theme", "theme/theme1.xml"), ("presProps", "presProps.xml"), ("viewProps", "viewProps.xml"), ("tableStyles", "tableStyles.xml")].iter().enumerate() {
        pres_rels.push((format!("rId{}", n + 2 + k), format!("{REL}/{ty}"), target.to_string(), false));
    }
    let title = esc(path.file_stem().and_then(|s| s.to_str()).unwrap_or("deck"));
    let ct = "application/vnd.openxmlformats-officedocument.presentationml";
    pkg.add("ppt/presentation.xml", format!(
        "{XML_HEAD}<p:presentation {NS} saveSubsetFonts=\"1\"><p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rId1\"/></p:sldMasterIdLst><p:sldIdLst>{sld_ids}</p:sldIdLst><p:sldSz cx=\"{}\" cy=\"{}\"/><p:notesSz cx=\"6858000\" cy=\"9144000\"/></p:presentation>",
        emu(w), emu(h)).into_bytes(), Some(&format!("{ct}.presentation.main+xml")));
    pkg.add("ppt/_rels/presentation.xml.rels", rels_xml(&pres_rels).into_bytes(), None);
    pkg.add("ppt/slideMasters/slideMaster1.xml", master_xml().into_bytes(), Some(&format!("{ct}.slideMaster+xml")));
    pkg.add("ppt/slideMasters/_rels/slideMaster1.xml.rels", rels_xml(&[
        ("rId1".into(), format!("{REL}/slideLayout"), "../slideLayouts/slideLayout1.xml".into(), false),
        ("rId2".into(), format!("{REL}/theme"), "../theme/theme1.xml".into(), false),
    ]).into_bytes(), None);
    pkg.add("ppt/slideLayouts/slideLayout1.xml", format!(
        "{XML_HEAD}<p:sldLayout {NS} type=\"blank\" preserve=\"1\"><p:cSld name=\"Blank\">{SP_TREE_EMPTY}</p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>").into_bytes(), Some(&format!("{ct}.slideLayout+xml")));
    pkg.add("ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels_xml(&[("rId1".into(), format!("{REL}/slideMaster"), "../slideMasters/slideMaster1.xml".into(), false)]).into_bytes(), None);
    pkg.add("ppt/theme/theme1.xml", theme_xml(&th).into_bytes(), Some("application/vnd.openxmlformats-officedocument.theme+xml"));
    pkg.add("ppt/presProps.xml", format!("{XML_HEAD}<p:presentationPr {NS}/>").into_bytes(), Some(&format!("{ct}.presProps+xml")));
    pkg.add("ppt/viewProps.xml", format!("{XML_HEAD}<p:viewPr {NS}><p:normalViewPr><p:restoredLeft sz=\"15620\"/><p:restoredTop sz=\"94660\"/></p:normalViewPr><p:gridSpacing cx=\"76200\" cy=\"76200\"/></p:viewPr>").into_bytes(), Some(&format!("{ct}.viewProps+xml")));
    pkg.add("ppt/tableStyles.xml", format!("{XML_HEAD}<a:tblStyleLst xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" def=\"{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}}\"/>").into_bytes(), Some(&format!("{ct}.tableStyles+xml")));
    pkg.add("docProps/core.xml", format!("{XML_HEAD}<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>{title}</dc:title><dc:creator>SlideML</dc:creator></cp:coreProperties>").into_bytes(), Some("application/vnd.openxmlformats-package.core-properties+xml"));
    pkg.add("docProps/app.xml", format!("{XML_HEAD}<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\"><Application>SlideML</Application><Slides>{n}</Slides></Properties>").into_bytes(), Some("application/vnd.openxmlformats-officedocument.extended-properties+xml"));

    let mut types = format!("{XML_HEAD}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Default Extension=\"xlsx\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\"/>");
    for (p, c) in &pkg.overrides {
        types += &format!("<Override PartName=\"{p}\" ContentType=\"{c}\"/>");
    }
    types += "</Types>";
    let root_rels = rels_xml(&[
        ("rId1".into(), format!("{REL}/officeDocument"), "ppt/presentation.xml".into(), false),
        ("rId2".into(), "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties".into(), "docProps/core.xml".into(), false),
        ("rId3".into(), format!("{REL}/extended-properties"), "docProps/app.xml".into(), false),
    ]);
    let mut parts = vec![("[Content_Types].xml".to_string(), types.into_bytes()), ("_rels/.rels".to_string(), root_rels.into_bytes())];
    parts.append(&mut pkg.parts);
    Ok(Output {
        pptx: zip_parts(&parts)?,
        manifest: format!("{{\"w\":{w},\"h\":{h},\"slides\":[{}]}}", pkg.manifest.join(",")),
        warnings: std::mem::take(&mut pkg.warnings),
    })
}
