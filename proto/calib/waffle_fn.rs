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

