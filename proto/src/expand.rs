//! Pre-pass: expands <styles> classes and <templates>/<use> into plain SlideML.
//! Every output element carries data-src="line:col" of its source, so diagnostics
//! still point at the file the agent wrote.

use crate::util::{esc, Res};
use roxmltree::Node;
use std::collections::HashMap;

struct Ctx<'a, 'i> {
    styles: HashMap<String, Vec<(String, String)>>,
    templates: HashMap<String, Node<'a, 'i>>,
}

fn pos(n: Node) -> String {
    let p = n.document().text_pos_at(n.range().start);
    format!("{}:{}", p.row, p.col)
}

fn fail<T>(n: Node, msg: impl std::fmt::Display) -> Res<T> {
    Err(format!("{} <{}>: {msg}", pos(n), n.tag_name().name()))
}

const SHIFT_X: &[&str] = &["x", "x1", "x2"];
const SHIFT_Y: &[&str] = &["y", "y1", "y2"];

struct UseFrame {
    params: HashMap<String, String>,
    dx: f64,
    dy: f64,
    prefix: String,
}

fn subst(s: &str, frames: &[UseFrame], n: Node) -> Res<String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("{{") {
        out += &rest[..i];
        let Some(j) = rest[i..].find("}}") else { return fail(n, "unclosed `{{` in template") };
        let key = rest[i + 2..i + j].trim();
        let val = frames.iter().rev().find_map(|f| f.params.get(key));
        match val {
            Some(v) => out += v,
            None => return fail(n, format!("template parameter `{key}` is not set on <use>")),
        }
        rest = &rest[i + j + 2..];
    }
    out += rest;
    Ok(out)
}

fn element(n: Node, cx: &Ctx, frames: &mut Vec<UseFrame>, out: &mut String, depth: usize) -> Res<()> {
    let name = n.tag_name().name();
    if name == "use" {
        if depth > 8 {
            return fail(n, "templates nest deeper than 8 levels");
        }
        let tid = n.attribute("template").map_or_else(|| fail(n, "missing `template`"), Ok)?;
        let t = cx.templates.get(tid).map_or_else(|| fail(n, format!("unknown template `{tid}`. Known: {}", cx.templates.keys().cloned().collect::<Vec<_>>().join(", "))), Ok)?;
        let mut f = UseFrame { params: HashMap::new(), dx: 0.0, dy: 0.0, prefix: String::new() };
        for a in n.attributes() {
            match a.name() {
                "template" => {}
                "dx" => f.dx = a.value().parse().or_else(|_| fail(n, "dx must be a number"))?,
                "dy" => f.dy = a.value().parse().or_else(|_| fail(n, "dy must be a number"))?,
                "prefix" => f.prefix = a.value().to_string(),
                k => {
                    f.params.insert(k.to_string(), a.value().to_string());
                }
            }
        }
        frames.push(f);
        for c in t.children() {
            node(c, cx, frames, out, depth + 1)?;
        }
        frames.pop();
        return Ok(());
    }
    // Attributes: class styles first, own attributes override.
    let mut attrs: Vec<(String, String)> = Vec::new();
    let set = |k: &str, v: String, attrs: &mut Vec<(String, String)>| match attrs.iter_mut().find(|(a, _)| a == k) {
        Some(e) => e.1 = v,
        None => attrs.push((k.to_string(), v)),
    };
    if let Some(classes) = n.attribute("class") {
        for c in classes.split_whitespace() {
            let st = cx.styles.get(c).map_or_else(|| fail(n, format!("unknown class `{c}`. Known: {}", cx.styles.keys().cloned().collect::<Vec<_>>().join(", "))), Ok)?;
            for (k, v) in st {
                set(k, v.clone(), &mut attrs);
            }
        }
    }
    for a in n.attributes() {
        if a.name() != "class" {
            set(a.name(), subst(a.value(), frames, n)?, &mut attrs);
        }
    }
    let (dx, dy) = frames.iter().fold((0.0, 0.0), |(x, y), f| (x + f.dx, y + f.dy));
    let prefix: String = frames.iter().map(|f| f.prefix.as_str()).collect();
    for (k, v) in attrs.iter_mut() {
        let shift = if SHIFT_X.contains(&k.as_str()) { dx } else if SHIFT_Y.contains(&k.as_str()) { dy } else { 0.0 };
        if shift != 0.0 {
            if let Ok(num) = v.trim().trim_end_matches("pt").parse::<f64>() {
                *v = format!("{}", num + shift);
            }
        }
        if (k == "id" || k == "target") && !prefix.is_empty() {
            *v = format!("{prefix}{v}");
        }
    }
    *out += &format!("<{name}");
    for (k, v) in &attrs {
        *out += &format!(" {k}=\"{}\"", esc(v));
    }
    if n.attribute("data-src").is_none() {
        *out += &format!(" data-src=\"{}\"", pos(n));
    }
    *out += ">";
    for c in n.children() {
        node(c, cx, frames, out, depth)?;
    }
    *out += &format!("</{name}>");
    Ok(())
}

fn node(n: Node, cx: &Ctx, frames: &mut Vec<UseFrame>, out: &mut String, depth: usize) -> Res<()> {
    if n.is_text() {
        *out += &esc(&subst(n.text().unwrap_or(""), frames, n.parent().unwrap_or(n))?);
        Ok(())
    } else if n.is_element() {
        element(n, cx, frames, out, depth)
    } else {
        Ok(())
    }
}

/// Returns the expanded XML. A deck without styles/templates is returned unchanged in meaning.
pub fn expand(src: &str) -> Res<String> {
    let doc = roxmltree::Document::parse(src).map_err(|e| format!("XML error: {e}"))?;
    let root = doc.root_element();
    let mut cx = Ctx { styles: HashMap::new(), templates: HashMap::new() };
    for c in root.children().filter(|c| c.is_element()) {
        match c.tag_name().name() {
            "styles" => {
                for s in c.children().filter(|s| s.is_element()) {
                    if s.tag_name().name() != "style" {
                        return fail(s, "only <style name=\"...\" .../> is allowed in <styles>");
                    }
                    let name = s.attribute("name").map_or_else(|| fail(s, "missing `name`"), Ok)?;
                    let attrs = s.attributes().filter(|a| a.name() != "name").map(|a| (a.name().to_string(), a.value().to_string())).collect();
                    cx.styles.insert(name.to_string(), attrs);
                }
            }
            "templates" => {
                for t in c.children().filter(|t| t.is_element()) {
                    if t.tag_name().name() != "template" {
                        return fail(t, "only <template id=\"...\"> is allowed in <templates>");
                    }
                    let id = t.attribute("id").map_or_else(|| fail(t, "missing `id`"), Ok)?;
                    cx.templates.insert(id.to_string(), t);
                }
            }
            _ => {}
        }
    }
    let mut out = String::new();
    out += &format!("<{}", root.tag_name().name());
    for a in root.attributes() {
        out += &format!(" {}=\"{}\"", a.name(), esc(a.value()));
    }
    out += ">";
    let mut frames = Vec::new();
    for c in root.children() {
        if c.is_element() && matches!(c.tag_name().name(), "styles" | "templates") {
            continue;
        }
        node(c, &cx, &mut frames, &mut out, 0)?;
    }
    out += &format!("</{}>", root.tag_name().name());
    Ok(out)
}
