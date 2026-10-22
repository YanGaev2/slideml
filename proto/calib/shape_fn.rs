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
        text: has_text.then_some(crate::lint::TextInfo { lines: lay.lines, height: lay.height, avail: h - t - b, min_size }),
        depth: ctx.depth,
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
