use roxmltree::Node;

pub type Res<T> = Result<T, String>;

pub const EMU_PER_PT: f64 = 12700.0;

pub fn emu(pt: f64) -> i64 {
    (pt * EMU_PER_PT).round() as i64
}

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// "line:col <tag>" prefix for diagnostics.
/// Source position "line:col" (from data-src after template expansion).
pub fn src_pos(node: Node) -> String {
    match node.attribute("data-src") {
        Some(s) => s.to_string(),
        None => {
            let p = node.document().text_pos_at(node.range().start);
            format!("{}:{}", p.row, p.col)
        }
    }
}

pub fn at(node: Node) -> String {
    format!("{} <{}>", src_pos(node), node.tag_name().name())
}

pub fn err<T>(node: Node, msg: impl std::fmt::Display) -> Res<T> {
    Err(format!("{}: {}", at(node), msg))
}

/// Unknown attributes are errors: the agent gets the list of valid names.
pub fn check_attrs(node: Node, allowed: &[&[&str]]) -> Res<()> {
    for a in node.attributes() {
        if a.name() != "data-src" && !allowed.iter().any(|set| set.contains(&a.name())) {
            let all: Vec<&str> = allowed.iter().flat_map(|s| s.iter().copied()).collect();
            return err(node, format!("unknown attribute `{}`. Valid: {}", a.name(), all.join(", ")));
        }
    }
    Ok(())
}

pub fn num(node: Node, name: &str) -> Res<Option<f64>> {
    match node.attribute(name) {
        None => Ok(None),
        Some(v) => v
            .trim()
            .trim_end_matches("pt")
            .trim_end_matches("ms")
            .trim_end_matches("deg")
            .parse::<f64>()
            .map(Some)
            .or_else(|_| err(node, format!("`{name}` must be a number, got `{v}`"))),
    }
}

pub fn req_num(node: Node, name: &str) -> Res<f64> {
    num(node, name)?.map_or_else(|| err(node, format!("missing required attribute `{name}`")), Ok)
}

pub fn flag(node: Node, name: &str) -> Res<Option<bool>> {
    match node.attribute(name) {
        None => Ok(None),
        Some("1" | "true" | "yes") => Ok(Some(true)),
        Some("0" | "false" | "no") => Ok(Some(false)),
        Some(v) => err(node, format!("`{name}` must be 1 or 0, got `{v}`")),
    }
}

const SCHEME: &[&str] = &[
    "tx1", "tx2", "bg1", "bg2", "dk1", "lt1", "dk2", "lt2", "accent1", "accent2", "accent3", "accent4",
    "accent5", "accent6", "hlink", "folHlink",
];

/// `#RRGGBB`, `#RRGGBB/40` (40% opacity), theme slot `accent1`, `accent1/40`, `white`, `black`.
pub fn color(s: &str) -> Result<String, String> {
    let s = s.trim();
    let (base, alpha) = match s.split_once('/') {
        Some((b, a)) => {
            let a: f64 = a.trim_end_matches('%').parse().map_err(|_| format!("bad opacity in color `{s}`"))?;
            (b, format!("<a:alpha val=\"{}\"/>", (a * 1000.0).round() as i64))
        }
        None => (s, String::new()),
    };
    let base = match base {
        "white" => "#FFFFFF",
        "black" => "#000000",
        b => b,
    };
    if let Some(hex) = base.strip_prefix('#') {
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("bad hex color `{s}`, expected #RRGGBB"));
        }
        return Ok(format!("<a:srgbClr val=\"{}\">{alpha}</a:srgbClr>", hex.to_uppercase()));
    }
    if SCHEME.contains(&base) {
        return Ok(format!("<a:schemeClr val=\"{base}\">{alpha}</a:schemeClr>"));
    }
    Err(format!("unknown color `{s}`. Use #RRGGBB, #RRGGBB/opacity%, or a theme slot: {}", SCHEME.join(", ")))
}

/// `none`, a color, `linear(ANGLE, c1 0%, c2 100%)` (0 = left to right, 90 = top to bottom),
/// `radial(c1 0%, c2 100%)`.
pub fn fill(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s == "none" {
        return Ok("<a:noFill/>".into());
    }
    let grad = |inner: &str, linear: bool| -> Result<String, String> {
        let mut parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        let ang = if linear {
            let a = parts.remove(0);
            a.trim_end_matches("deg").parse::<f64>().map_err(|_| format!("bad gradient angle `{a}`"))?
        } else {
            0.0
        };
        if parts.len() < 2 {
            return Err(format!("gradient `{s}` needs at least 2 stops"));
        }
        let n = parts.len();
        let mut gs = String::new();
        for (i, p) in parts.iter().enumerate() {
            let mut it = p.split_whitespace();
            let c = color(it.next().unwrap_or(""))?;
            let pos = match it.next() {
                Some(v) => v.trim_end_matches('%').parse::<f64>().map_err(|_| format!("bad stop `{p}`"))?,
                None => 100.0 * i as f64 / (n - 1) as f64,
            };
            gs += &format!("<a:gs pos=\"{}\">{c}</a:gs>", (pos * 1000.0).round() as i64);
        }
        let shade = if linear {
            format!("<a:lin ang=\"{}\" scaled=\"0\"/>", (ang * 60000.0).round() as i64)
        } else {
            "<a:path path=\"circle\"><a:fillToRect l=\"50000\" t=\"50000\" r=\"50000\" b=\"50000\"/></a:path>".into()
        };
        Ok(format!("<a:gradFill rotWithShape=\"1\"><a:gsLst>{gs}</a:gsLst>{shade}</a:gradFill>"))
    };
    if let Some(inner) = s.strip_prefix("linear(").and_then(|r| r.strip_suffix(')')) {
        return grad(inner, true);
    }
    if let Some(inner) = s.strip_prefix("radial(").and_then(|r| r.strip_suffix(')')) {
        return grad(inner, false);
    }
    if let Some(inner) = s.strip_prefix("pattern(").and_then(|r| r.strip_suffix(')')) {
        return pattern(inner);
    }
    Ok(format!("<a:solidFill>{}</a:solidFill>", color(s)?))
}

const DASH: &[&str] = &["solid", "dot", "dash", "lgDash", "dashDot", "lgDashDot", "sysDash", "sysDot", "sysDashDot"];

/// `color width [dash]` or `none`. Extra XML (arrow ends) goes after the dash.
pub fn line(s: Option<&str>, ends: &str) -> Result<String, String> {
    let Some(s) = s.map(str::trim).filter(|s| *s != "none") else {
        return Ok("<a:ln><a:noFill/></a:ln>".into());
    };
    let t: Vec<&str> = s.split_whitespace().collect();
    if t.len() < 2 || t.len() > 3 {
        return Err(format!("line `{s}` must be `color width [dash]`"));
    }
    let w: f64 = t[1].trim_end_matches("pt").parse().map_err(|_| format!("bad line width in `{s}`"))?;
    let dash = match t.get(2) {
        None => String::new(),
        Some(d) if DASH.contains(d) => format!("<a:prstDash val=\"{d}\"/>"),
        Some(d) => return Err(format!("unknown dash `{d}`. Valid: {}", DASH.join(", "))),
    };
    Ok(format!(
        "<a:ln w=\"{}\" cap=\"rnd\"><a:solidFill>{}</a:solidFill>{dash}<a:round/>{ends}</a:ln>",
        emu(w),
        color(t[0])?
    ))
}

const PATTERNS: &[&str] = &[
    "pct5", "pct10", "pct20", "pct25", "pct30", "pct40", "pct50", "pct60", "pct70", "pct75", "pct80", "pct90",
    "horz", "vert", "ltHorz", "ltVert", "dkHorz", "dkVert", "narHorz", "narVert", "dashHorz", "dashVert", "cross",
    "dnDiag", "upDiag", "ltDnDiag", "ltUpDiag", "dkDnDiag", "dkUpDiag", "wdDnDiag", "wdUpDiag", "dashDnDiag",
    "dashUpDiag", "diagCross", "smCheck", "lgCheck", "smGrid", "lgGrid", "dotGrid", "smConfetti", "lgConfetti",
    "horzBrick", "diagBrick", "solidDmnd", "openDmnd", "dotDmnd", "plaid", "sphere", "weave", "divot", "shingle",
    "wave", "trellis", "zigZag",
];

/// `pattern(name, fg, bg)` -> a:pattFill.
fn pattern(inner: &str) -> Result<String, String> {
    let p: Vec<&str> = inner.split(',').map(str::trim).collect();
    if p.len() != 3 || !PATTERNS.contains(&p[0]) {
        return Err(format!("pattern(name, fg, bg). Names: {}", PATTERNS.join(", ")));
    }
    Ok(format!("<a:pattFill prst=\"{}\"><a:fgClr>{}</a:fgClr><a:bgClr>{}</a:bgClr></a:pattFill>", p[0], color(p[1])?, color(p[2])?))
}

/// Effect list: `shadow` = none|soft|lift|strong|hard or `blur dist color [angle]` (pt, deg; 90 = down,
/// 45 = down-right); `glow` = `color radius`; `soft-edge` = radius (pt).
pub fn effects(shadow: Option<&str>, glow: Option<&str>, soft_edge: Option<f64>) -> Result<String, String> {
    let mut x = String::new();
    if let Some(g) = glow {
        let t: Vec<&str> = g.split_whitespace().collect();
        let r: f64 = t.get(1).and_then(|v| v.trim_end_matches("pt").parse().ok()).ok_or(format!("glow `{g}` must be `color radius`"))?;
        x += &format!("<a:glow rad=\"{}\">{}</a:glow>", emu(r), color(t[0])?);
    }
    let sh = match shadow.map(str::trim) {
        None | Some("none") => None,
        Some("soft") => Some((28.0, 8.0, "#000000/14".to_string(), 90.0)),
        Some("lift") => Some((10.0, 3.0, "#000000/22".to_string(), 90.0)),
        Some("strong") => Some((36.0, 14.0, "#000000/30".to_string(), 90.0)),
        Some("hard") => Some((0.0, 6.0, "#000000".to_string(), 45.0)),
        Some(v) => {
            let t: Vec<&str> = v.split_whitespace().collect();
            if t.len() != 3 && t.len() != 4 {
                return Err(format!("shadow `{v}` must be soft|lift|strong|hard|none or `blur dist color [angle]`"));
            }
            let p = |x: &str| x.trim_end_matches("pt").trim_end_matches("deg").parse::<f64>().map_err(|_| format!("bad shadow `{v}`"));
            Some((p(t[0])?, p(t[1])?, t[2].to_string(), if t.len() == 4 { p(t[3])? } else { 90.0 }))
        }
    };
    if let Some((blur, dist, c, ang)) = sh {
        x += &format!(
            "<a:outerShdw blurRad=\"{}\" dist=\"{}\" dir=\"{}\" algn=\"tl\" rotWithShape=\"0\">{}</a:outerShdw>",
            emu(blur),
            emu(dist),
            (ang.rem_euclid(360.0) * 60000.0).round() as i64,
            color(&c)?
        );
    }
    if let Some(r) = soft_edge {
        x += &format!("<a:softEdge rad=\"{}\"/>", emu(r));
    }
    Ok(if x.is_empty() { x } else { format!("<a:effectLst>{x}</a:effectLst>") })
}
