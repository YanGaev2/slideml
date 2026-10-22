//! Native tables (a:tbl). Style "No Style, No Grid"; all borders and fills are explicit.

use crate::anim::Kind;
use crate::deck::SlideCtx;
use crate::text::{self, TStyle, TEXT_ATTRS};
use crate::util::*;
use roxmltree::Node;

const TABLE_ATTRS: &[&str] = &["id", "x", "y", "w", "h", "cols", "line", "lines", "pad", "anchor", "fill"];
const ROW_ATTRS: &[&str] = &["h", "fill"];
const CELL_ATTRS: &[&str] = &["colspan", "fill", "anchor", "pad"];
const NO_STYLE: &str = "{2D5ABB26-0587-4C30-8999-92F81FD0307C}";

fn widths(node: Node, spec: &str, total: f64) -> Res<Vec<f64>> {
    let mut fixed = 0.0;
    let mut fr = 0.0;
    let mut parts = Vec::new();
    for t in spec.split_whitespace() {
        if let Some(v) = t.strip_suffix("fr") {
            let v: f64 = v.parse().or_else(|_| err(node, format!("bad column `{t}`")))?;
            fr += v;
            parts.push((v, true));
        } else {
            let v: f64 = t.trim_end_matches("pt").parse().or_else(|_| err(node, format!("bad column `{t}`")))?;
            fixed += v;
            parts.push((v, false));
        }
    }
    let free = total - fixed;
    if free < 0.0 || (fr == 0.0 && free.abs() > 0.5) {
        return err(node, format!("cols `{spec}` do not add up to table width {total}"));
    }
    Ok(parts.into_iter().map(|(v, is_fr)| if is_fr { free * v / fr } else { v }).collect())
}

fn pad4(node: Node, def: [f64; 4]) -> Res<[f64; 4]> {
    let Some(p) = node.attribute("pad") else { return Ok(def) };
    let v: Vec<f64> = p.split_whitespace().map(|s| s.parse::<f64>()).collect::<Result<_, _>>().or_else(|_| err(node, "pad: 1, 2 or 4 numbers"))?;
    Ok(match v.len() {
        1 => [v[0]; 4],
        2 => [v[0], v[1], v[0], v[1]],
        4 => [v[0], v[1], v[2], v[3]],
        _ => return err(node, "pad: 1, 2 or 4 numbers"),
    })
}

pub fn table(node: Node, ctx: &mut SlideCtx, base: &TStyle) -> Res<String> {
    check_attrs(node, &[TABLE_ATTRS, TEXT_ATTRS])?;
    let (id, name) = ctx.register(node, Kind::Other)?;
    let (x, y, w, h) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?, req_num(node, "h")?);
    let rows: Vec<Node> = node.children().filter(|c| c.is_element()).collect();
    for r in &rows {
        if r.tag_name().name() != "tr" {
            return err(*r, "only <tr> is allowed in <table>");
        }
    }
    if rows.is_empty() {
        return err(node, "table has no <tr>");
    }
    let cols = node.attribute("cols").map_or_else(|| err(node, "missing `cols`, e.g. cols=\"1fr 2fr\""), Ok)?;
    let cw = widths(node, cols, w)?;
    let nc = cw.len();
    let lines = node.attribute("lines").unwrap_or("h");
    if !["none", "h", "hv", "rows", "all"].contains(&lines) {
        return err(node, "lines: none, h, hv, rows, all");
    }
    let on = line(Some(node.attribute("line").unwrap_or("tx1/20 1")), "").or_else(|m| err(node, m))?;
    let off = "<a:ln><a:noFill/></a:ln>".to_string();
    let edge = |tag: &str, l: &str| l.replacen("<a:ln", &format!("<a:{tag}"), 1).replace("</a:ln>", &format!("</a:{tag}>"));
    let table_style = base.with(node)?;
    let table_pad = pad4(node, [6.0, 8.0, 6.0, 8.0])?;
    let default_h = h / rows.len() as f64;
    let lang = ctx.lang.clone();
    let (has_h, has_v) = (lines != "none", lines == "hv" || lines == "all");
    let (outer_h, outer_v) = (lines == "rows" || lines == "all", lines == "all");

    let grid: String = cw.iter().map(|c| format!("<a:gridCol w=\"{}\"/>", emu(*c))).collect();
    let nr = rows.len();
    let mut trs = String::new();
    for (ri, r) in rows.iter().enumerate() {
        check_attrs(*r, &[ROW_ATTRS, TEXT_ATTRS])?;
        let row_style = table_style.with(*r)?;
        let rh = num(*r, "h")?.unwrap_or(default_h);
        let mut tcs = String::new();
        let mut ci = 0;
        for cell in r.children().filter(|c| c.is_element()) {
            let tag = cell.tag_name().name();
            if tag != "td" && tag != "th" {
                return err(cell, "only <td> and <th> are allowed in <tr>");
            }
            check_attrs(cell, &[CELL_ATTRS, TEXT_ATTRS])?;
            let st = if tag == "th" { row_style.clone().with_bold() } else { row_style.clone() }.with(cell)?;
            let span = num(cell, "colspan")?.unwrap_or(1.0) as usize;
            if ci + span > nc {
                return err(cell, format!("row {} has more cells than the {nc} columns", ri + 1));
            }
            let paras = text::paragraphs(cell, &st, &lang, ctx)?;
            let paras = if paras.is_empty() { format!("<a:p><a:endParaRPr lang=\"{lang}\" dirty=\"0\"/></a:p>") } else { paras };
            let [pt, pr, pb, pl] = pad4(cell, table_pad)?;
            let anchor = match cell.attribute("anchor").or(node.attribute("anchor")).unwrap_or("middle") {
                "top" => "t",
                "middle" => "ctr",
                "bottom" => "b",
                v => return err(cell, format!("anchor `{v}`: top, middle, bottom")),
            };
            let last_c = ci + span - 1;
            let l = if (ci > 0 && has_v) || (ci == 0 && outer_v) { &on } else { &off };
            let rt = if (last_c < nc - 1 && has_v) || (last_c == nc - 1 && outer_v) { &on } else { &off };
            let t = if (ri > 0 && has_h) || (ri == 0 && outer_h) { &on } else { &off };
            let b = if (ri < nr - 1 && has_h) || (ri == nr - 1 && outer_h) { &on } else { &off };
            let f = cell.attribute("fill").or(r.attribute("fill")).or(node.attribute("fill")).unwrap_or("none");
            let fill_xml = fill(f).or_else(|m| err(cell, m))?;
            tcs += &format!(
                "<a:tc{}><a:txBody><a:bodyPr/><a:lstStyle/>{paras}</a:txBody><a:tcPr marL=\"{}\" marR=\"{}\" marT=\"{}\" marB=\"{}\" anchor=\"{anchor}\">{}{}{}{}{fill_xml}</a:tcPr></a:tc>",
                if span > 1 { format!(" gridSpan=\"{span}\"") } else { String::new() },
                emu(pl), emu(pr), emu(pt), emu(pb),
                edge("lnL", l), edge("lnR", rt), edge("lnT", t), edge("lnB", b)
            );
            for _ in 1..span {
                tcs += &format!("<a:tc hMerge=\"1\"><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"{lang}\"/></a:p></a:txBody><a:tcPr/></a:tc>");
            }
            ci += span;
        }
        if ci != nc {
            return err(*r, format!("row {} has {ci} columns, the table has {nc}", ri + 1));
        }
        trs += &format!("<a:tr h=\"{}\">{tcs}</a:tr>", emu(rh));
    }
    Ok(format!(
        "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp=\"1\"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></p:xfrm><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\"><a:tbl><a:tblPr><a:tableStyleId>{NO_STYLE}</a:tableStyleId></a:tblPr><a:tblGrid>{grid}</a:tblGrid>{trs}</a:tbl></a:graphicData></a:graphic></p:graphicFrame>",
        esc(&name), emu(x), emu(y), emu(w), emu(h)
    ))
}
