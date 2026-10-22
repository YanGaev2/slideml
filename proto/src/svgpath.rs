//! SVG geometry -> DrawingML custGeom. Arcs and quadratic curves become cubic Béziers,
//! because a:arcTo uses a different (start-angle) model than SVG endpoint arcs.

use std::f64::consts::PI;

#[derive(Clone, Copy)]
pub enum Seg {
    M(f64, f64),
    L(f64, f64),
    C(f64, f64, f64, f64, f64, f64),
    Z,
}

struct Cur<'a> {
    s: &'a [u8],
    i: usize,
}

impl Cur<'_> {
    fn skip(&mut self) {
        while self.i < self.s.len() && (self.s[self.i].is_ascii_whitespace() || self.s[self.i] == b',') {
            self.i += 1;
        }
    }
    fn at_num(&mut self) -> bool {
        self.skip();
        self.i < self.s.len() && matches!(self.s[self.i], b'0'..=b'9' | b'.' | b'-' | b'+')
    }
    fn num(&mut self) -> Result<f64, String> {
        self.skip();
        let st = self.i;
        if self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+') {
            self.i += 1;
        }
        let mut dot = false;
        while self.i < self.s.len() {
            match self.s[self.i] {
                b'0'..=b'9' => {}
                b'.' if !dot => dot = true,
                b'e' | b'E' => {
                    self.i += 1;
                    if self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+') {
                        self.i += 1;
                    }
                    continue;
                }
                _ => break,
            }
            self.i += 1;
        }
        std::str::from_utf8(&self.s[st..self.i])
            .unwrap()
            .parse()
            .map_err(|_| format!("bad number at position {st} in path"))
    }
    fn flag(&mut self) -> Result<bool, String> {
        self.skip();
        let c = self.s.get(self.i).copied();
        self.i += 1;
        match c {
            Some(b'0') => Ok(false),
            Some(b'1') => Ok(true),
            _ => Err("bad arc flag in path".into()),
        }
    }
}

/// SVG endpoint arc -> cubic segments (W3C SVG implementation notes, F.6.5).
fn arc(out: &mut Vec<Seg>, x1: f64, y1: f64, mut rx: f64, mut ry: f64, phi_deg: f64, large: bool, sweep: bool, x2: f64, y2: f64) {
    if (x1 - x2).abs() < 1e-9 && (y1 - y2).abs() < 1e-9 {
        return;
    }
    rx = rx.abs();
    ry = ry.abs();
    if rx < 1e-9 || ry < 1e-9 {
        out.push(Seg::L(x2, y2));
        return;
    }
    let phi = phi_deg.to_radians();
    let (sp, cp) = phi.sin_cos();
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cp * dx + sp * dy;
    let y1p = -sp * dx + cp * dy;
    let lam = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry);
    if lam > 1.0 {
        rx *= lam.sqrt();
        ry *= lam.sqrt();
    }
    let num = rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p;
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let mut co = (num / den).max(0.0).sqrt();
    if large == sweep {
        co = -co;
    }
    let cxp = co * rx * y1p / ry;
    let cyp = -co * ry * x1p / rx;
    let cx = cp * cxp - sp * cyp + (x1 + x2) / 2.0;
    let cy = sp * cxp + cp * cyp + (y1 + y2) / 2.0;
    let ang = |ux: f64, uy: f64, vx: f64, vy: f64| {
        let a = (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
        a
    };
    let t1 = ang(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dt = ang((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry);
    if !sweep && dt > 0.0 {
        dt -= 2.0 * PI;
    } else if sweep && dt < 0.0 {
        dt += 2.0 * PI;
    }
    let n = (dt.abs() / (PI / 2.0)).ceil().max(1.0) as usize;
    let step = dt / n as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let pt = |t: f64| {
        let (s, c) = t.sin_cos();
        (cx + rx * c * cp - ry * s * sp, cy + rx * c * sp + ry * s * cp)
    };
    let der = |t: f64| {
        let (s, c) = t.sin_cos();
        (-rx * s * cp - ry * c * sp, -rx * s * sp + ry * c * cp)
    };
    for i in 0..n {
        let a = t1 + step * i as f64;
        let b = a + step;
        let (p0, d0, p3, d3) = (pt(a), der(a), pt(b), der(b));
        out.push(Seg::C(p0.0 + k * d0.0, p0.1 + k * d0.1, p3.0 - k * d3.0, p3.1 - k * d3.1, p3.0, p3.1));
    }
}

pub fn parse(d: &str) -> Result<Vec<Seg>, String> {
    let mut c = Cur { s: d.as_bytes(), i: 0 };
    let mut out = Vec::new();
    let (mut x, mut y, mut sx, mut sy) = (0.0, 0.0, 0.0, 0.0);
    let mut last_c: Option<(f64, f64)> = None; // last cubic control point (for S)
    let mut last_q: Option<(f64, f64)> = None; // last quadratic control point (for T)
    let mut cmd = b'M';
    loop {
        c.skip();
        if c.i >= c.s.len() {
            break;
        }
        let ch = c.s[c.i];
        if ch.is_ascii_alphabetic() {
            cmd = ch;
            c.i += 1;
        } else if !c.at_num() {
            return Err(format!("unexpected `{}` in path", ch as char));
        }
        let rel = cmd.is_ascii_lowercase();
        let (ox, oy) = if rel { (x, y) } else { (0.0, 0.0) };
        let up = cmd.to_ascii_uppercase();
        let (mut nc, mut nq) = (None, None);
        match up {
            b'M' => {
                x = ox + c.num()?;
                y = oy + c.num()?;
                sx = x;
                sy = y;
                out.push(Seg::M(x, y));
                cmd = if rel { b'l' } else { b'L' };
            }
            b'L' => {
                x = ox + c.num()?;
                y = oy + c.num()?;
                out.push(Seg::L(x, y));
            }
            b'H' => {
                x = ox + c.num()?;
                out.push(Seg::L(x, y));
            }
            b'V' => {
                y = oy + c.num()?;
                out.push(Seg::L(x, y));
            }
            b'C' | b'S' => {
                let (x1, y1) = if up == b'C' {
                    (ox + c.num()?, oy + c.num()?)
                } else {
                    last_c.map_or((x, y), |(px, py)| (2.0 * x - px, 2.0 * y - py))
                };
                let (x2, y2) = (ox + c.num()?, oy + c.num()?);
                let (ex, ey) = (ox + c.num()?, oy + c.num()?);
                out.push(Seg::C(x1, y1, x2, y2, ex, ey));
                nc = Some((x2, y2));
                x = ex;
                y = ey;
            }
            b'Q' | b'T' => {
                let (qx, qy) = if up == b'Q' {
                    (ox + c.num()?, oy + c.num()?)
                } else {
                    last_q.map_or((x, y), |(px, py)| (2.0 * x - px, 2.0 * y - py))
                };
                let (ex, ey) = (ox + c.num()?, oy + c.num()?);
                out.push(Seg::C(x + 2.0 / 3.0 * (qx - x), y + 2.0 / 3.0 * (qy - y), ex + 2.0 / 3.0 * (qx - ex), ey + 2.0 / 3.0 * (qy - ey), ex, ey));
                nq = Some((qx, qy));
                x = ex;
                y = ey;
            }
            b'A' => {
                let (rx, ry, rot) = (c.num()?, c.num()?, c.num()?);
                let (large, sweep) = (c.flag()?, c.flag()?);
                let (ex, ey) = (ox + c.num()?, oy + c.num()?);
                arc(&mut out, x, y, rx, ry, rot, large, sweep, ex, ey);
                x = ex;
                y = ey;
            }
            b'Z' => {
                out.push(Seg::Z);
                x = sx;
                y = sy;
            }
            other => return Err(format!("path command `{}` is not supported", other as char)),
        }
        last_c = nc;
        last_q = nq;
    }
    Ok(out)
}

pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<Seg> {
    let k = 0.552_284_749_8;
    let (ox, oy) = (rx * k, ry * k);
    vec![
        Seg::M(cx + rx, cy),
        Seg::C(cx + rx, cy + oy, cx + ox, cy + ry, cx, cy + ry),
        Seg::C(cx - ox, cy + ry, cx - rx, cy + oy, cx - rx, cy),
        Seg::C(cx - rx, cy - oy, cx - ox, cy - ry, cx, cy - ry),
        Seg::C(cx + ox, cy - ry, cx + rx, cy - oy, cx + rx, cy),
        Seg::Z,
    ]
}

pub fn rect(x: f64, y: f64, w: f64, h: f64, rx: f64, ry: f64) -> Vec<Seg> {
    let (rx, ry) = (rx.min(w / 2.0), ry.min(h / 2.0));
    if rx <= 0.0 || ry <= 0.0 {
        return vec![Seg::M(x, y), Seg::L(x + w, y), Seg::L(x + w, y + h), Seg::L(x, y + h), Seg::Z];
    }
    let k = 0.552_284_749_8;
    let (kx, ky) = (rx * k, ry * k);
    vec![
        Seg::M(x + rx, y),
        Seg::L(x + w - rx, y),
        Seg::C(x + w - rx + kx, y, x + w, y + ry - ky, x + w, y + ry),
        Seg::L(x + w, y + h - ry),
        Seg::C(x + w, y + h - ry + ky, x + w - rx + kx, y + h, x + w - rx, y + h),
        Seg::L(x + rx, y + h),
        Seg::C(x + rx - kx, y + h, x, y + h - ry + ky, x, y + h - ry),
        Seg::L(x, y + ry),
        Seg::C(x, y + ry - ky, x + rx - kx, y, x + rx, y),
        Seg::Z,
    ]
}

/// One a:path. `scale` maps source units to path units (integers).
pub fn path_xml(segs: &[Seg], w: f64, h: f64, scale: f64, fill: bool, stroke: bool) -> String {
    let p = |x: f64, y: f64| format!("<a:pt x=\"{}\" y=\"{}\"/>", (x * scale).round() as i64, (y * scale).round() as i64);
    let mut s = String::new();
    for g in segs {
        match *g {
            Seg::M(x, y) => s += &format!("<a:moveTo>{}</a:moveTo>", p(x, y)),
            Seg::L(x, y) => s += &format!("<a:lnTo>{}</a:lnTo>", p(x, y)),
            Seg::C(a, b, c, d, e, f) => s += &format!("<a:cubicBezTo>{}{}{}</a:cubicBezTo>", p(a, b), p(c, d), p(e, f)),
            Seg::Z => s += "<a:close/>",
        }
    }
    format!(
        "<a:path w=\"{}\" h=\"{}\"{}{}>{s}</a:path>",
        (w * scale).round() as i64,
        (h * scale).round() as i64,
        if fill { "" } else { " fill=\"none\"" },
        if stroke { "" } else { " stroke=\"0\"" }
    )
}

pub fn cust_geom(paths: &str) -> String {
    format!("<a:custGeom><a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/><a:rect l=\"0\" t=\"0\" r=\"r\" b=\"b\"/><a:pathLst>{paths}</a:pathLst></a:custGeom>")
}

/// Converts the drawable children of an SVG fragment (path, circle, ellipse, rect, line,
/// polyline, polygon) into segment lists. Returns (segments, closed) per element.
pub fn svg_elements(svg: &roxmltree::Node) -> Result<Vec<(Vec<Seg>, bool)>, String> {
    let n = |e: &roxmltree::Node, a: &str| e.attribute(a).map_or(Ok(0.0), |v| v.parse::<f64>().map_err(|_| format!("bad `{a}` in icon")));
    let mut out = Vec::new();
    for e in svg.descendants().filter(|e| e.is_element()) {
        let segs = match e.tag_name().name() {
            "path" => parse(e.attribute("d").unwrap_or(""))?,
            "circle" => ellipse(n(&e, "cx")?, n(&e, "cy")?, n(&e, "r")?, n(&e, "r")?),
            "ellipse" => ellipse(n(&e, "cx")?, n(&e, "cy")?, n(&e, "rx")?, n(&e, "ry")?),
            "rect" => {
                let (rx, ry) = (e.attribute("rx"), e.attribute("ry"));
                let rx_v = n(&e, "rx")?;
                let ry_v = n(&e, "ry")?;
                let (rx, ry) = match (rx, ry) {
                    (Some(_), None) => (rx_v, rx_v),
                    (None, Some(_)) => (ry_v, ry_v),
                    _ => (rx_v, ry_v),
                };
                rect(n(&e, "x")?, n(&e, "y")?, n(&e, "width")?, n(&e, "height")?, rx, ry)
            }
            "line" => vec![Seg::M(n(&e, "x1")?, n(&e, "y1")?), Seg::L(n(&e, "x2")?, n(&e, "y2")?)],
            t @ ("polyline" | "polygon") => {
                let v: Vec<f64> = e.attribute("points").unwrap_or("").split([' ', ',']).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap_or(0.0)).collect();
                let mut s: Vec<Seg> = v.chunks(2).enumerate().map(|(i, p)| if i == 0 { Seg::M(p[0], p[1]) } else { Seg::L(p[0], p[1]) }).collect();
                if t == "polygon" {
                    s.push(Seg::Z);
                }
                s
            }
            _ => continue,
        };
        let closed = segs.iter().any(|s| matches!(s, Seg::Z));
        out.push((segs, closed));
    }
    Ok(out)
}
