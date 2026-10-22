//! p:timing builder. Every effect template is copied from XML that PowerPoint 16 wrote
//! itself (tools/harvest-anim.ps1). Do not invent presetID values: PowerPoint shows
//! unknown combinations as "Custom".

#[derive(Clone, Copy, PartialEq)]
pub enum Start {
    Click,
    With,
    After,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Shape,
    Graphic,
    /// Pictures, connectors, groups: no build-list entry.
    Other,
}

pub struct Anim {
    pub spid: u32,
    pub kind: Kind,
    pub effect: String,
    pub start: Start,
    pub delay: u32,
    pub dur: u32,
}

pub const EFFECTS: &[&str] = &[
    "appear", "fade", "float-up", "float-down", "fly-left", "fly-right", "fly-top", "fly-bottom", "wipe-left",
    "wipe-right", "wipe-top", "wipe-bottom", "zoom", "expand",
];

struct Ids(u32);
impl Ids {
    fn next(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }
}

fn tgt(spid: u32) -> String {
    format!("<p:tgtEl><p:spTgt spid=\"{spid}\"/></p:tgtEl>")
}

fn set_visible(ids: &mut Ids, spid: u32) -> String {
    format!(
        "<p:set><p:cBhvr><p:cTn id=\"{}\" dur=\"1\" fill=\"hold\"><p:stCondLst><p:cond delay=\"0\"/></p:stCondLst></p:cTn>{}<p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val=\"visible\"/></p:to></p:set>",
        ids.next(),
        tgt(spid)
    )
}

fn filter(ids: &mut Ids, spid: u32, dur: u32, f: &str) -> String {
    format!(
        "<p:animEffect transition=\"in\" filter=\"{f}\"><p:cBhvr><p:cTn id=\"{}\" dur=\"{dur}\"/>{}</p:cBhvr></p:animEffect>",
        ids.next(),
        tgt(spid)
    )
}

/// `from` starting with '=' is a float literal (fltVal), else a formula (strVal).
fn prop(ids: &mut Ids, spid: u32, dur: u32, attr: &str, from: &str, to: &str, additive: bool) -> String {
    let v = |s: &str| match s.strip_prefix('=') {
        Some(f) => format!("<p:fltVal val=\"{f}\"/>"),
        None => format!("<p:strVal val=\"{s}\"/>"),
    };
    format!(
        "<p:anim calcmode=\"lin\" valueType=\"num\"><p:cBhvr{}><p:cTn id=\"{}\" dur=\"{dur}\" fill=\"hold\"/>{}<p:attrNameLst><p:attrName>{attr}</p:attrName></p:attrNameLst></p:cBhvr><p:tavLst><p:tav tm=\"0\"><p:val>{}</p:val></p:tav><p:tav tm=\"100000\"><p:val>{}</p:val></p:tav></p:tavLst></p:anim>",
        if additive { " additive=\"base\"" } else { "" },
        ids.next(),
        tgt(spid),
        v(from),
        v(to)
    )
}

/// Returns (presetID, presetSubtype, behaviours).
fn behaviours(a: &Anim, ids: &mut Ids) -> (u32, u32, String) {
    let (s, d) = (a.spid, a.dur);
    let mut x = set_visible(ids, s);
    let fly = |ids: &mut Ids, x0: &str, y0: &str| {
        prop(ids, s, d, "ppt_x", x0, "#ppt_x", true) + &prop(ids, s, d, "ppt_y", y0, "#ppt_y", true)
    };
    let (preset, sub) = match a.effect.as_str() {
        "appear" => (1, 0),
        "fade" => {
            x += &filter(ids, s, d, "fade");
            (10, 0)
        }
        "float-up" | "float-down" => {
            x += &filter(ids, s, d, "fade");
            x += &prop(ids, s, d, "ppt_x", "#ppt_x", "#ppt_x", false);
            let up = a.effect == "float-up";
            x += &prop(ids, s, d, "ppt_y", if up { "#ppt_y+.1" } else { "#ppt_y-.1" }, "#ppt_y", false);
            (if up { 42 } else { 47 }, 0)
        }
        "fly-left" => {
            x += &fly(ids, "0-#ppt_w/2", "#ppt_y");
            (2, 8)
        }
        "fly-right" => {
            x += &fly(ids, "1+#ppt_w/2", "#ppt_y");
            (2, 2)
        }
        "fly-top" => {
            x += &fly(ids, "#ppt_x", "0-#ppt_h/2");
            (2, 1)
        }
        "fly-bottom" => {
            x += &fly(ids, "#ppt_x", "1+#ppt_h/2");
            (2, 4)
        }
        "wipe-left" => {
            x += &filter(ids, s, d, "wipe(left)");
            (22, 8)
        }
        "wipe-right" => {
            x += &filter(ids, s, d, "wipe(right)");
            (22, 2)
        }
        "wipe-top" => {
            x += &filter(ids, s, d, "wipe(up)");
            (22, 1)
        }
        "wipe-bottom" => {
            x += &filter(ids, s, d, "wipe(down)");
            (22, 4)
        }
        "zoom" => {
            x += &prop(ids, s, d, "ppt_w", "=0", "#ppt_w", false);
            x += &prop(ids, s, d, "ppt_h", "=0", "#ppt_h", false);
            x += &filter(ids, s, d, "fade");
            (53, 16)
        }
        "expand" => {
            x += &prop(ids, s, d, "ppt_w", "#ppt_w*0.70", "#ppt_w", false);
            x += &prop(ids, s, d, "ppt_h", "#ppt_h", "#ppt_h", false);
            x += &filter(ids, s, d, "fade");
            (55, 0)
        }
        other => unreachable!("effect `{other}` is validated by the caller"),
    };
    (preset, sub, x)
}

struct Step {
    delay: u32,
    effects: Vec<usize>,
}

pub fn timing(anims: &[Anim]) -> String {
    if anims.is_empty() {
        return String::new();
    }
    // Click groups -> sequential steps ("after") -> parallel effects ("with").
    let mut groups: Vec<Vec<Step>> = Vec::new();
    for (i, a) in anims.iter().enumerate() {
        if a.start == Start::Click || groups.is_empty() {
            groups.push(vec![Step { delay: 0, effects: vec![i] }]);
            continue;
        }
        let steps = groups.last_mut().unwrap();
        if a.start == Start::With {
            steps.last_mut().unwrap().effects.push(i);
        } else {
            let last = steps.last().unwrap();
            let end = last.effects.iter().map(|&j| anims[j].delay + anims[j].dur).max().unwrap_or(0);
            steps.push(Step { delay: last.delay + end, effects: vec![i] });
        }
    }

    let mut ids = Ids(2); // 1 = tmRoot, 2 = mainSeq
    let mut seq = String::new();
    for (gi, steps) in groups.iter().enumerate() {
        let gid = ids.next();
        let auto = gi == 0 && anims[steps[0].effects[0]].start != Start::Click;
        let cond = if auto {
            "<p:cond delay=\"indefinite\"/><p:cond evt=\"onBegin\" delay=\"0\"><p:tn val=\"2\"/></p:cond>"
        } else {
            "<p:cond delay=\"indefinite\"/>"
        };
        let mut steps_xml = String::new();
        for st in steps {
            let sid = ids.next();
            let mut eff = String::new();
            for &j in &st.effects {
                let a = &anims[j];
                let node_type = match a.start {
                    Start::Click => "clickEffect",
                    Start::With => "withEffect",
                    Start::After => "afterEffect",
                };
                let eid = ids.next();
                let (preset, sub, beh) = behaviours(a, &mut ids);
                eff += &format!(
                    "<p:par><p:cTn id=\"{eid}\" presetID=\"{preset}\" presetClass=\"entr\" presetSubtype=\"{sub}\" fill=\"hold\" grpId=\"0\" nodeType=\"{node_type}\"><p:stCondLst><p:cond delay=\"{}\"/></p:stCondLst><p:childTnLst>{beh}</p:childTnLst></p:cTn></p:par>",
                    a.delay
                );
            }
            steps_xml += &format!(
                "<p:par><p:cTn id=\"{sid}\" fill=\"hold\"><p:stCondLst><p:cond delay=\"{}\"/></p:stCondLst><p:childTnLst>{eff}</p:childTnLst></p:cTn></p:par>",
                st.delay
            );
        }
        seq += &format!(
            "<p:par><p:cTn id=\"{gid}\" fill=\"hold\"><p:stCondLst>{cond}</p:stCondLst><p:childTnLst>{steps_xml}</p:childTnLst></p:cTn></p:par>"
        );
    }

    let mut bld = String::new();
    let mut seen = Vec::new();
    for a in anims {
        if seen.contains(&a.spid) {
            continue;
        }
        seen.push(a.spid);
        match a.kind {
            Kind::Shape => bld += &format!("<p:bldP spid=\"{}\" grpId=\"0\" animBg=\"1\"/>", a.spid),
            Kind::Graphic => bld += &format!("<p:bldGraphic spid=\"{}\" grpId=\"0\"><p:bldAsOne/></p:bldGraphic>", a.spid),
            _ => {}
        }
    }
    let bld = if bld.is_empty() { bld } else { format!("<p:bldLst>{bld}</p:bldLst>") };

    format!(
        "<p:timing><p:tnLst><p:par><p:cTn id=\"1\" dur=\"indefinite\" restart=\"never\" nodeType=\"tmRoot\"><p:childTnLst><p:seq concurrent=\"1\" nextAc=\"seek\"><p:cTn id=\"2\" dur=\"indefinite\" nodeType=\"mainSeq\"><p:childTnLst>{seq}</p:childTnLst></p:cTn><p:prevCondLst><p:cond evt=\"onPrev\" delay=\"0\"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:prevCondLst><p:nextCondLst><p:cond evt=\"onNext\" delay=\"0\"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:nextCondLst></p:seq></p:childTnLst></p:cTn></p:par></p:tnLst>{bld}</p:timing>"
    )
}
