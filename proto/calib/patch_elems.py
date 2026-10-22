p = 'F:/pptx-ai-native/proto/src/deck.rs'
s = open(p, encoding='utf-8').read()
def rep(old, new):
    global s
    assert old in s, old[:70]
    s = s.replace(old, new, 1)
rep('''                let (inner, gb) = elements(c, ctx, pkg, base_dir, th, base, None)?;''',
'''                ctx.depth += 1;
                let (inner, gb) = elements(c, ctx, pkg, base_dir, th, base, None)?;
                ctx.depth -= 1;''')
rep('''                let g = crate::chart::chart(c, ctx, pkg, &th.body)?;
                (format!(
                    "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\\"{id}\\" name=\\"{}\\"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp=\\"1\\"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>{}{g}</p:graphicFrame>",
                    esc(&name),
                    xfrm(c, "p:xfrm")?
                ), bbox(c)?)''',
'''                let (g, overlays) = crate::chart::chart(c, ctx, pkg, &th.body)?;
                (format!(
                    "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\\"{id}\\" name=\\"{}\\"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp=\\"1\\"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>{}{g}</p:graphicFrame>{overlays}",
                    esc(&name),
                    xfrm(c, "p:xfrm")?
                ), bbox(c)?)''')
rep('''        tree += &xml;
        bb = Some(match bb {''', '''        if name != "shape" && name != "text" {
            let id = c.attribute("id").map(String::from).unwrap_or_else(|| name.to_string());
            ctx.items.push(crate::lint::Item { id, kind: if name == "group" { "group" } else { "other" }, bbox: b, text: None, depth: ctx.depth });
        }
        tree += &xml;
        bb = Some(match bb {''')
rep('''    pub fn register(&mut self, node: Node, kind: Kind) -> Res<(u32, String)> {''', '''    /// Registers a generated element (no source node), e.g. a chart annotation.
    pub fn register_named(&mut self, name: &str, kind: Kind) -> u32 {
        self.next_id += 1;
        self.names.insert(name.into(), (self.next_id, kind));
        self.next_id
    }

    pub fn register(&mut self, node: Node, kind: Kind) -> Res<(u32, String)> {''')
open(p, 'w', encoding='utf-8').write(s)
print('ok')
