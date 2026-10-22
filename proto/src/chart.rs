//! Native PowerPoint charts with deterministic geometry.
//! The compiler fixes the axis scale (min/max/step) and the inner plot area (manualLayout),
//! so it knows where every bar and point is. Annotations (<ref>, <note>, <band>, <center>)
//! become native shapes placed by data coordinates.

use crate::anim::Kind;
use crate::deck::{Package, SlideCtx};
use crate::util::*;
use roxmltree::Node;

pub const CHART_ATTRS: &[&str] = &[
    "type", "font", "size", "color", "grid", "grid-color", "labels", "label-pos", "label-color", "label-size",
    "label-bold", "label-font", "legend", "gap", "overlap", "hole", "format", "axis", "axis-color", "min", "max", "step",
    "stacked", "smooth", "plot", "outline",
];
const TYPES: &[&str] = &["column", "bar", "line", "area", "doughnut", "pie"];
const SERIES_ATTRS: &[&str] = &["name", "values", "color", "colors", "fill", "width", "marker", "label-points"];

struct Series {
    name: String,
    values: Vec<f64>,
    color: String,
    point_colors: Vec<String>,
    fill: Option<String>,
    width: f64,
    marker: bool,
    label_points: Option<Vec<usize>>,
}

fn col(i: usize) -> char {
    (b'A' + i as u8) as char
}

fn txpr(size: f64, color_s: &str, font: &str, bold: bool, lang: &str) -> Res<String> {
    Ok(format!(
        "<c:txPr><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr sz=\"{}\" b=\"{}\">{}<a:latin typeface=\"{}\"/><a:cs typeface=\"{}\"/></a:defRPr></a:pPr><a:endParaRPr lang=\"{lang}\"/></a:p></c:txPr>",
        (size * 100.0).round() as i64,
        bold as u8,
        fill(color_s)?,
        esc(font),
        esc(font)
    ))
}

fn str_cache(f: &str, vals: &[String]) -> String {
    let pts: String = vals.iter().enumerate().map(|(i, v)| format!("<c:pt idx=\"{i}\"><c:v>{}</c:v></c:pt>", esc(v))).collect();
    format!("<c:strRef><c:f>{f}</c:f><c:strCache><c:ptCount val=\"{}\"/>{pts}</c:strCache></c:strRef>", vals.len())
}

fn num_cache(f: &str, vals: &[f64]) -> String {
    let pts: String = vals.iter().enumerate().map(|(i, v)| format!("<c:pt idx=\"{i}\"><c:v>{v}</c:v></c:pt>")).collect();
    format!("<c:numRef><c:f>{f}</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val=\"{}\"/>{pts}</c:numCache></c:numRef>", vals.len())
}

fn sheet_xml(cats: &[String], series: &[Series]) -> String {
    let cell_s = |r: usize, c: usize, v: &str| format!("<c r=\"{}{r}\" t=\"inlineStr\"><is><t>{}</t></is></c>", col(c), esc(v));
    let mut rows = format!("<row r=\"1\">{}", cell_s(1, 0, " "));
    for (j, s) in series.iter().enumerate() {
        rows += &cell_s(1, j + 1, &s.name);
    }
    rows += "</row>";
    for (i, cat) in cats.iter().enumerate() {
        let r = i + 2;
        rows += &format!("<row r=\"{r}\">{}", cell_s(r, 0, cat));
        for (j, s) in series.iter().enumerate() {
            rows += &format!("<c r=\"{}{r}\"><v>{}</v></c>", col(j + 1), s.values[i]);
        }
        rows += "</row>";
    }
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData>{rows}</sheetData></worksheet>")
}

fn workbook(cats: &[String], series: &[Series]) -> Res<Vec<u8>> {
    let h = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
    let parts = vec![
        ("[Content_Types].xml".to_string(), format!("{h}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>").into_bytes()),
        ("_rels/.rels".to_string(), format!("{h}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>").into_bytes()),
        ("xl/workbook.xml".to_string(), format!("{h}<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>").into_bytes()),
        ("xl/_rels/workbook.xml.rels".to_string(), format!("{h}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>").into_bytes()),
        ("xl/worksheets/sheet1.xml".to_string(), sheet_xml(cats, series).into_bytes()),
    ];
    crate::deck::zip_parts(&parts)
}

/// Width of a single-line string in pt.
fn text_w(s: &str, font: &str, size: f64) -> f64 {
    let face = crate::fonts::face(font, false);
    s.chars().map(|c| face.and_then(|f| f.advance(c)).unwrap_or(0.55) as f64 * size).sum()
}

/// Excel-like number formatting for axis labels (enough for width estimates).
fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 { format!("{}", v.round() as i64) } else { format!("{v:.1}") }
}

/// "Nice" axis: step from {1, 2, 2.5, 5} x 10^k, about 5 intervals.
fn nice_scale(lo: f64, hi: f64) -> (f64, f64, f64) {
    let range = (hi - lo).max(1e-9);
    let raw = range / 5.0;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0].iter().map(|m| m * mag).find(|s| *s >= raw - 1e-12).unwrap();
    let min = (lo / step).floor() * step;
    let max = (hi / step).ceil() * step;
    (min, if max <= hi { max + step } else { max }, step)
}

/// Plot geometry, in slide pt.
struct Plot {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    min: f64,
    max: f64,
    n: usize,
    horizontal: bool,
    mid_cat: bool,
}

impl Plot {
    /// Position along the value axis.
    fn v(&self, v: f64) -> f64 {
        let t = (v - self.min) / (self.max - self.min);
        if self.horizontal { self.x + self.w * t } else { self.y + self.h * (1.0 - t) }
    }
    /// Centre of category `i` along the category axis.
    fn c(&self, i: usize) -> f64 {
        if self.mid_cat && self.n > 1 {
            return self.x + self.w * i as f64 / (self.n - 1) as f64;
        }
        if self.horizontal {
            self.y + self.h * (i as f64 + 0.5) / self.n as f64
        } else {
            self.x + self.w * (i as f64 + 0.5) / self.n as f64
        }
    }
}

fn text_sp(id: u32, name: &str, x: f64, y: f64, w: f64, h: f64, body: &str, fill_xml: &str, anchor: &str, lang: &str) -> String {
    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"roundRect\"><a:avLst><a:gd name=\"adj\" fmla=\"val 12000\"/></a:avLst></a:prstGeom>{fill_xml}<a:ln><a:noFill/></a:ln></p:spPr><p:txBody><a:bodyPr wrap=\"square\" lIns=\"{}\" tIns=\"{}\" rIns=\"{}\" bIns=\"{}\" anchor=\"{anchor}\" rtlCol=\"0\"><a:noAutofit/></a:bodyPr><a:lstStyle/>{body}<!--{lang}--></p:txBody></p:sp>",
        esc(name), emu(x), emu(y), emu(w), emu(h), emu(6.0), emu(4.0), emu(6.0), emu(4.0)
    )
    .replace(&format!("<!--{lang}-->"), "")
}

fn line_sp(id: u32, name: &str, x1: f64, y1: f64, x2: f64, y2: f64, ln: &str) -> String {
    let flip = format!("{}{}", if x2 < x1 { " flipH=\"1\"" } else { "" }, if y2 < y1 { " flipV=\"1\"" } else { "" });
    format!(
        "<p:cxnSp><p:nvCxnSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr><p:spPr><a:xfrm{flip}><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm><a:prstGeom prst=\"line\"><a:avLst/></a:prstGeom>{ln}</p:spPr></p:cxnSp>",
        esc(name), emu(x1.min(x2)), emu(y1.min(y2)), emu((x2 - x1).abs()), emu((y2 - y1).abs())
    )
}

fn para(text: &str, font: &str, size: f64, color_x: &str, bold: bool, align: &str, lang: &str) -> String {
    format!(
        "<a:p><a:pPr algn=\"{align}\"><a:buNone/></a:pPr><a:r><a:rPr lang=\"{lang}\" sz=\"{}\" b=\"{}\" dirty=\"0\">{color_x}<a:latin typeface=\"{}\"/><a:cs typeface=\"{}\"/></a:rPr><a:t>{}</a:t></a:r></a:p>",
        (size * 100.0).round() as i64, bold as u8, esc(font), esc(font), esc(text)
    )
}

/// Returns (graphic XML for the frame, annotation shapes XML placed after the frame).
/// Bands are returned inside the annotation string but must sit below the chart: the
/// caller writes them first via the leading marker.
pub fn chart(node: Node, ctx: &mut SlideCtx, pkg: &mut Package, body_font: &str) -> Res<(String, String)> {
    let ty = node.attribute("type").unwrap_or("column");
    if !TYPES.contains(&ty) {
        return err(node, format!("chart type `{ty}`. Valid: {}", TYPES.join(", ")));
    }
    let e = |r: Result<String, String>| r.or_else(|m| err(node, m));
    let lang = ctx.lang.clone();
    let chart_id = node.attribute("id").unwrap_or("chart").to_string();
    let font = node.attribute("font").unwrap_or(body_font).to_string();
    let size = num(node, "size")?.unwrap_or(12.0);
    let text_color = node.attribute("color").unwrap_or("tx1/70").to_string();
    let (fx, fy, fw, fh) = (req_num(node, "x")?, req_num(node, "y")?, req_num(node, "w")?, req_num(node, "h")?);

    let mut cats: Vec<String> = Vec::new();
    let mut series: Vec<Series> = Vec::new();
    let mut notes = Vec::new();
    for c in node.children().filter(|c| c.is_element()) {
        match c.tag_name().name() {
            "cat" => {
                check_attrs(c, &[])?;
                cats = c.text().unwrap_or("").split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
            "series" => {
                check_attrs(c, &[SERIES_ATTRS])?;
                let values = c.attribute("values").unwrap_or("").split(',').map(|v| v.trim().parse::<f64>()).collect::<Result<Vec<_>, _>>().or_else(|_| err(c, "values must be comma-separated numbers"))?;
                let idx = series.len();
                let point_colors: Vec<String> = c.attribute("colors").map(|s| split_top(s).into_iter().map(|x| x.trim().to_string()).collect()).unwrap_or_default();
                for pc in &point_colors {
                    fill(pc).or_else(|m| err(c, m))?;
                }
                let label_points = match c.attribute("label-points") {
                    Some(v) => Some(v.split(',').map(|x| x.trim().parse::<usize>()).collect::<Result<Vec<_>, _>>().or_else(|_| err(c, "label-points: comma-separated point indexes from 0"))?),
                    None => None,
                };
                let s = Series {
                    name: c.attribute("name").unwrap_or("Series").to_string(),
                    values,
                    color: c.attribute("color").map(String::from).unwrap_or(format!("accent{}", idx % 6 + 1)),
                    point_colors,
                    fill: c.attribute("fill").map(String::from),
                    width: num(c, "width")?.unwrap_or(2.5),
                    marker: flag(c, "marker")?.unwrap_or(false),
                    label_points,
                };
                color(&s.color).or_else(|m| err(c, m))?;
                series.push(s);
            }
            "ref" | "note" | "band" | "center" => notes.push(c),
            other => return err(c, format!("<{other}> not allowed in <chart>. Use <cat>, <series>, <ref>, <note>, <band>, <center>")),
        }
    }
    if series.is_empty() || cats.is_empty() {
        return err(node, "chart needs <cat> and at least one <series>");
    }
    for s in &series {
        if s.values.len() != cats.len() {
            return err(node, format!("series `{}` has {} values, but there are {} categories", s.name, s.values.len(), cats.len()));
        }
        if !s.point_colors.is_empty() && s.point_colors.len() != cats.len() {
            return err(node, format!("series `{}`: `colors` needs {} entries", s.name, cats.len()));
        }
    }

    let n = cats.len();
    let is_pie = ty == "doughnut" || ty == "pie";
    let horizontal = ty == "bar";
    let stacked = flag(node, "stacked")?.unwrap_or(false);
    let labels = flag(node, "labels")?.unwrap_or(false);
    let fmt = node.attribute("format").unwrap_or("General").to_string();
    let label_size = num(node, "label-size")?.unwrap_or(size);
    let label_font = node.attribute("label-font").unwrap_or(&font).to_string();
    let axis_mode = node.attribute("axis").unwrap_or("cat");
    if !["cat", "both", "none"].contains(&axis_mode) {
        return err(node, "axis must be cat, both or none");
    }
    let legend_pos = node.attribute("legend").unwrap_or("none");
    if !["none", "b", "t", "r", "l"].contains(&legend_pos) {
        return err(node, format!("legend `{legend_pos}`: use none, b, t, r, l"));
    }

    // ---- value scale -------------------------------------------------------------
    let (mut lo, mut hi) = (0.0f64, f64::MIN);
    for i in 0..n {
        if stacked {
            hi = hi.max(series.iter().map(|s| s.values[i].max(0.0)).sum());
            lo = lo.min(series.iter().map(|s| s.values[i].min(0.0)).sum());
        } else {
            for s in &series {
                hi = hi.max(s.values[i]);
                lo = lo.min(s.values[i]);
            }
        }
    }
    if ty == "line" || ty == "area" {
        // Lines need not start at zero; start near the data.
        let dmin = series.iter().flat_map(|s| s.values.iter().copied()).fold(f64::INFINITY, f64::min);
        if dmin > 0.0 && dmin > hi * 0.4 {
            lo = dmin;
        }
    }
    let (mut amin, mut amax, mut astep) = nice_scale(lo, hi);
    if lo >= 0.0 && amin < 0.0 {
        amin = 0.0;
    }
    if let Some(v) = num(node, "min")? {
        amin = v;
    }
    if let Some(v) = num(node, "max")? {
        amax = v;
    }
    if let Some(v) = num(node, "step")? {
        astep = v;
    }
    if amax <= amin {
        return err(node, "max must be greater than min");
    }

    // ---- plot area -------------------------------------------------------------------
    let cat_axis = !is_pie && axis_mode != "none";
    let val_axis = !is_pie && axis_mode == "both";
    let label_pos = node.attribute("label-pos").unwrap_or(match ty {
        "column" | "bar" => "outEnd",
        "line" => "t",
        _ => "ctr",
    });
    let line_h = |s: f64| s * 1.2;
    let max_cat_w = cats.iter().map(|c| text_w(c, &font, size)).fold(0.0, f64::max);
    let val_labels: Vec<String> = {
        let mut v = Vec::new();
        let mut t = amin;
        while t <= amax + 1e-9 {
            v.push(fmt_num(t));
            t += astep;
        }
        v
    };
    let max_val_w = val_labels.iter().map(|s| text_w(s, &font, size)).fold(0.0, f64::max);
    let max_data_label_w = series.iter().flat_map(|s| s.values.iter()).map(|v| text_w(&fmt_num(*v), &label_font, label_size)).fold(0.0, f64::max);
    let (mut pt, mut pr, mut pb, mut pl) = (4.0, 4.0, 4.0, 4.0);
    if horizontal {
        if cat_axis {
            // PowerPoint wraps category labels earlier than their measured width: keep a margin.
            pl += max_cat_w * 1.15 + 14.0;
        }
        if val_axis {
            pb += line_h(size) + 6.0;
        }
        if labels && label_pos == "outEnd" {
            pr += max_data_label_w + 6.0;
        }
    } else if !is_pie {
        if cat_axis {
            pb += line_h(size) + 6.0;
        }
        if val_axis {
            pl += max_val_w + 8.0;
        }
        if labels && (label_pos == "outEnd" || label_pos == "t") {
            pt += line_h(label_size) + 4.0;
        }
    }
    let legend_names_w = series.iter().map(|s| text_w(&s.name, &font, size) + 22.0).sum::<f64>();
    match legend_pos {
        "b" => pb += line_h(size) + 10.0,
        "t" => pt += line_h(size) + 10.0,
        "r" => pr += series.iter().map(|s| text_w(&s.name, &font, size)).fold(0.0, f64::max) + 30.0,
        "l" => pl += series.iter().map(|s| text_w(&s.name, &font, size)).fold(0.0, f64::max) + 30.0,
        _ => {}
    }
    let _ = legend_names_w;
    if let Some(p) = node.attribute("plot") {
        let v: Vec<f64> = p.split_whitespace().map(|s| s.parse::<f64>()).collect::<Result<_, _>>().or_else(|_| err(node, "plot: 4 numbers `top right bottom left` (pt inside the chart)"))?;
        if v.len() != 4 {
            return err(node, "plot: 4 numbers `top right bottom left` (pt inside the chart)");
        }
        (pt, pr, pb, pl) = (v[0], v[1], v[2], v[3]);
    }
    if is_pie {
        // Square plot, centred.
        let side = (fw - pl - pr).min(fh - pt - pb);
        pl = (fw - side) / 2.0;
        pr = pl;
        pt = (fh - side) / 2.0;
        pb = pt;
    }
    let plot = Plot { x: fx + pl, y: fy + pt, w: fw - pl - pr, h: fh - pt - pb, min: amin, max: amax, n, horizontal, mid_cat: ty == "area" };
    if plot.w < 10.0 || plot.h < 10.0 {
        return err(node, "chart is too small for its axes and labels: enlarge it or set plot=\"t r b l\"");
    }
    let layout = format!(
        "<c:layout><c:manualLayout><c:layoutTarget val=\"inner\"/><c:xMode val=\"edge\"/><c:yMode val=\"edge\"/><c:x val=\"{:.5}\"/><c:y val=\"{:.5}\"/><c:w val=\"{:.5}\"/><c:h val=\"{:.5}\"/></c:manualLayout></c:layout>",
        pl / fw, pt / fh, plot.w / fw, plot.h / fh
    );

    // ---- series XML -----------------------------------------------------------------
    let label_tx = e(txpr(label_size, node.attribute("label-color").unwrap_or(&text_color), &label_font, flag(node, "label-bold")?.unwrap_or(false), &lang))?;
    let outline = match node.attribute("outline") {
        Some(o) => e(line(Some(o), ""))?,
        None => "<a:ln w=\"19050\"><a:solidFill><a:schemeClr val=\"bg1\"/></a:solidFill></a:ln>".into(),
    };
    let dlbls = |pos: Option<&str>, only: &Option<Vec<usize>>| -> String {
        let off = "<c:showLegendKey val=\"0\"/><c:showVal val=\"0\"/><c:showCatName val=\"0\"/><c:showSerName val=\"0\"/><c:showPercent val=\"0\"/><c:showBubbleSize val=\"0\"/>";
        if !labels {
            return format!("<c:dLbls>{off}</c:dLbls>");
        }
        let hidden: String = match only {
            Some(keep) => (0..n).filter(|i| !keep.contains(i)).map(|i| format!("<c:dLbl><c:idx val=\"{i}\"/><c:delete val=\"1\"/></c:dLbl>")).collect(),
            None => String::new(),
        };
        let p = pos.map(|p| format!("<c:dLblPos val=\"{p}\"/>")).unwrap_or_default();
        format!(
            "<c:dLbls>{hidden}<c:numFmt formatCode=\"{}\" sourceLinked=\"0\"/><c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr>{label_tx}{p}<c:showLegendKey val=\"0\"/><c:showVal val=\"1\"/><c:showCatName val=\"0\"/><c:showSerName val=\"0\"/><c:showPercent val=\"0\"/><c:showBubbleSize val=\"0\"/>{}</c:dLbls>",
            esc(&fmt),
            if is_pie { "<c:showLeaderLines val=\"0\"/>" } else { "" }
        )
    };
    let cat_f = format!("Sheet1!$A$2:$A${}", n + 1);
    let mut sers = String::new();
    for (j, s) in series.iter().enumerate() {
        let c = e(color(&s.color))?;
        let main_fill = match &s.fill {
            Some(f) => e(fill(f))?,
            None => format!("<a:solidFill>{c}</a:solidFill>"),
        };
        let sp = match ty {
            "line" => format!("<c:spPr><a:ln w=\"{}\" cap=\"rnd\"><a:solidFill>{c}</a:solidFill><a:round/></a:ln></c:spPr>", emu(s.width)),
            "area" => format!("<c:spPr>{main_fill}<a:ln w=\"{}\" cap=\"rnd\"><a:solidFill>{c}</a:solidFill><a:round/></a:ln></c:spPr>", emu(s.width)),
            "doughnut" | "pie" => format!("<c:spPr>{main_fill}{outline}</c:spPr>"),
            _ => format!("<c:spPr>{main_fill}<a:ln><a:noFill/></a:ln></c:spPr>"),
        };
        let mut dpts = String::new();
        for (i, pc) in s.point_colors.iter().enumerate() {
            let pf = e(fill(pc))?;
            dpts += &format!(
                "<c:dPt><c:idx val=\"{i}\"/>{}</c:dPt>",
                match ty {
                    "line" => format!("<c:marker><c:symbol val=\"circle\"/><c:size val=\"7\"/><c:spPr>{pf}<a:ln><a:noFill/></a:ln></c:spPr></c:marker><c:bubble3D val=\"0\"/>"),
                    "doughnut" | "pie" => format!("<c:bubble3D val=\"0\"/><c:spPr>{pf}{outline}</c:spPr>"),
                    _ => format!("<c:invertIfNegative val=\"0\"/><c:bubble3D val=\"0\"/><c:spPr>{pf}<a:ln><a:noFill/></a:ln></c:spPr>"),
                }
            );
        }
        let tx = format!("<c:tx>{}</c:tx>", str_cache(&format!("Sheet1!${}$1", col(j + 1)), std::slice::from_ref(&s.name)));
        let cat = format!("<c:cat>{}</c:cat>", str_cache(&cat_f, &cats));
        let val = format!("<c:val>{}</c:val>", num_cache(&format!("Sheet1!${}$2:${}${}", col(j + 1), col(j + 1), n + 1), &s.values));
        let head = format!("<c:idx val=\"{j}\"/><c:order val=\"{j}\"/>{tx}{sp}");
        let lp = &s.label_points;
        sers += &match ty {
            "column" | "bar" => format!("<c:ser>{head}<c:invertIfNegative val=\"0\"/>{dpts}{}{cat}{val}</c:ser>", dlbls(Some(label_pos), lp)),
            "line" => {
                let marker = if s.marker {
                    format!("<c:marker><c:symbol val=\"circle\"/><c:size val=\"7\"/><c:spPr><a:solidFill>{c}</a:solidFill><a:ln w=\"19050\"><a:solidFill><a:schemeClr val=\"bg1\"/></a:solidFill></a:ln></c:spPr></c:marker>")
                } else {
                    "<c:marker><c:symbol val=\"none\"/></c:marker>".into()
                };
                let smooth = flag(node, "smooth")?.unwrap_or(false) as u8;
                format!("<c:ser>{head}{marker}{dpts}{}{cat}{val}<c:smooth val=\"{smooth}\"/></c:ser>", dlbls(Some(label_pos), lp))
            }
            _ => format!("<c:ser>{head}{dpts}{}{cat}{val}</c:ser>", dlbls(None, lp)),
        };
    }

    // ---- axes -------------------------------------------------------------------------
    let axis_ln = match node.attribute("axis-color").unwrap_or("tx1/25") {
        "none" => "<c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr>".to_string(),
        c => format!("<c:spPr><a:noFill/><a:ln w=\"9525\"><a:solidFill>{}</a:solidFill></a:ln></c:spPr>", e(color(c))?),
    };
    let grid = if flag(node, "grid")?.unwrap_or(false) {
        format!("<c:majorGridlines><c:spPr><a:ln w=\"6350\"><a:solidFill>{}</a:solidFill></a:ln></c:spPr></c:majorGridlines>", e(color(node.attribute("grid-color").unwrap_or("tx1/12")))?)
    } else {
        String::new()
    };
    let tx_axis = e(txpr(size, &text_color, &font, false, &lang))?;
    let axes = format!(
        "<c:catAx><c:axId val=\"500000001\"/><c:scaling><c:orientation val=\"{}\"/></c:scaling><c:delete val=\"{}\"/><c:axPos val=\"{}\"/><c:numFmt formatCode=\"General\" sourceLinked=\"1\"/><c:majorTickMark val=\"none\"/><c:minorTickMark val=\"none\"/><c:tickLblPos val=\"nextTo\"/>{axis_ln}{tx_axis}<c:crossAx val=\"500000002\"/><c:crosses val=\"autoZero\"/><c:auto val=\"1\"/><c:lblAlgn val=\"ctr\"/><c:lblOffset val=\"100\"/><c:noMultiLvlLbl val=\"0\"/></c:catAx><c:valAx><c:axId val=\"500000002\"/><c:scaling><c:orientation val=\"minMax\"/><c:max val=\"{amax}\"/><c:min val=\"{amin}\"/></c:scaling><c:delete val=\"{}\"/><c:axPos val=\"{}\"/>{grid}<c:numFmt formatCode=\"{}\" sourceLinked=\"0\"/><c:majorTickMark val=\"none\"/><c:minorTickMark val=\"none\"/><c:tickLblPos val=\"nextTo\"/><c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr>{tx_axis}<c:crossAx val=\"500000001\"/><c:crosses val=\"{}\"/><c:crossBetween val=\"{}\"/><c:majorUnit val=\"{astep}\"/></c:valAx>",
        if horizontal { "maxMin" } else { "minMax" },
        (!cat_axis) as u8,
        if horizontal { "l" } else { "b" },
        (!val_axis) as u8,
        if horizontal { "b" } else { "l" },
        esc(&fmt),
        if horizontal { "max" } else { "autoZero" },
        if ty == "area" { "midCat" } else { "between" },
    );
    let ax_ids = "<c:axId val=\"500000001\"/><c:axId val=\"500000002\"/>";
    let gap = num(node, "gap")?.unwrap_or(60.0);
    let overlap = if stacked { 100.0 } else { num(node, "overlap")?.unwrap_or(-10.0) };
    let plot_xml = match ty {
        "column" | "bar" => format!(
            "<c:barChart><c:barDir val=\"{}\"/><c:grouping val=\"{}\"/><c:varyColors val=\"0\"/>{sers}<c:gapWidth val=\"{}\"/><c:overlap val=\"{}\"/>{ax_ids}</c:barChart>{axes}",
            if horizontal { "bar" } else { "col" },
            if stacked { "stacked" } else { "clustered" },
            gap as i64,
            overlap as i64,
        ),
        "line" => format!("<c:lineChart><c:grouping val=\"standard\"/><c:varyColors val=\"0\"/>{sers}<c:marker val=\"1\"/>{ax_ids}</c:lineChart>{axes}"),
        "area" => format!("<c:areaChart><c:grouping val=\"{}\"/><c:varyColors val=\"0\"/>{sers}{ax_ids}</c:areaChart>{axes}", if stacked { "stacked" } else { "standard" }),
        "doughnut" => format!("<c:doughnutChart><c:varyColors val=\"1\"/>{sers}<c:firstSliceAng val=\"0\"/><c:holeSize val=\"{}\"/></c:doughnutChart>", num(node, "hole")?.unwrap_or(70.0) as i64),
        _ => format!("<c:pieChart><c:varyColors val=\"1\"/>{sers}<c:firstSliceAng val=\"0\"/></c:pieChart>"),
    };
    let legend = if legend_pos == "none" { String::new() } else { format!("<c:legend><c:legendPos val=\"{legend_pos}\"/><c:overlay val=\"0\"/>{tx_axis}</c:legend>") };

    let n_chart = pkg.next_chart();
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><c:date1904 val=\"0\"/><c:lang val=\"{lang}\"/><c:roundedCorners val=\"0\"/><c:chart><c:autoTitleDeleted val=\"1\"/><c:plotArea>{layout}{plot_xml}<c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr></c:plotArea>{legend}<c:plotVisOnly val=\"1\"/><c:dispBlanksAs val=\"gap\"/></c:chart><c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr>{tx_axis}<c:externalData r:id=\"rId1\"><c:autoUpdate val=\"0\"/></c:externalData></c:chartSpace>"
    );
    let wb = workbook(&cats, &series)?;
    pkg.add(&format!("ppt/charts/chart{n_chart}.xml"), xml.into_bytes(), Some("application/vnd.openxmlformats-officedocument.drawingml.chart+xml"));
    pkg.add(&format!("ppt/embeddings/Microsoft_Excel_Worksheet{n_chart}.xlsx"), wb, None);
    pkg.add(
        &format!("ppt/charts/_rels/chart{n_chart}.xml.rels"),
        crate::deck::rels_xml(&[("rId1".into(), "http://schemas.openxmlformats.org/officeDocument/2006/relationships/package".into(), format!("../embeddings/Microsoft_Excel_Worksheet{n_chart}.xlsx"), false)]).into_bytes(),
        None,
    );
    let rid = ctx.rel("http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart", &format!("../charts/chart{n_chart}.xml"), false);
    let graphic = format!("<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" r:id=\"{rid}\"/></a:graphicData></a:graphic>");

    // ---- annotations --------------------------------------------------------------
    // Bar geometry (clustered): bar width = category width / (s - (s-1)*overlap + gap).
    let s_count = if stacked { 1.0 } else { series.len() as f64 };
    let ov = if stacked { 1.0 } else { overlap / 100.0 };
    let cat_span = if horizontal { plot.h } else { plot.w } / n as f64;
    let bar_w = cat_span / (s_count - (s_count - 1.0) * ov + gap / 100.0);
    let point = |cat: usize, ser: usize| -> (f64, f64) {
        let v: f64 = if stacked { series[..=ser].iter().map(|s| s.values[cat]).sum() } else { series[ser].values[cat] };
        let mut c = plot.c(cat);
        if (ty == "column" || ty == "bar") && !stacked {
            let group = bar_w * (s_count - (s_count - 1.0) * ov);
            c = c - group / 2.0 + ser as f64 * bar_w * (1.0 - ov) + bar_w / 2.0;
        }
        if horizontal { (plot.v(v), c) } else { (c, plot.v(v)) }
    };
    let cat_index = |c: Node, at: &str| -> Res<usize> {
        cats.iter().position(|x| x == at).map_or_else(|| err(c, format!("category `{at}` not found. Categories: {}", cats.join(", "))), Ok)
    };
    let mut below = String::new();
    let mut above = String::new();
    for (k, c) in notes.iter().enumerate() {
        let c = *c;
        let ann_font = c.attribute("font").unwrap_or(&font).to_string();
        let ann_size = num(c, "size")?.unwrap_or(size);
        let ann_color = c.attribute("color").unwrap_or("tx1").to_string();
        let cx_col = e(fill(&ann_color))?;
        let txt: String = c.text().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
        let name = c.attribute("id").map(String::from).unwrap_or(format!("{chart_id}-{}{k}", c.tag_name().name()));
        match c.tag_name().name() {
            "ref" => {
                check_attrs(c, &[&["id", "value", "stroke", "color", "size", "font", "label", "label-at"]])?;
                let at_start = match c.attribute("label-at").unwrap_or("end") {
                    "start" => true,
                    "end" => false,
                    v => return err(c, format!("label-at `{v}`: use start or end")),
                };
                let v = req_num(c, "value")?;
                let ln = e(line(Some(c.attribute("stroke").unwrap_or("tx1/45 1 dash")), ""))?;
                let id = ctx.register_named(&name, Kind::Other);
                let label = c.attribute("label").unwrap_or(&txt).to_string();
                if horizontal {
                    let x = plot.v(v);
                    above += &line_sp(id, &name, x, plot.y, x, plot.y + plot.h, &ln);
                    if !label.is_empty() {
                        let lid = ctx.register_named(&format!("{name}-label"), Kind::Shape);
                        let w = text_w(&label, &ann_font, ann_size) + 14.0;
                        above += &text_sp(lid, &format!("{name}-label"), x - w / 2.0, plot.y - line_h(ann_size) - 10.0, w, line_h(ann_size) + 8.0, &para(&label, &ann_font, ann_size, &cx_col, false, "ctr", &lang), "<a:noFill/>", "ctr", &lang);
                    }
                } else {
                    let y = plot.v(v);
                    above += &line_sp(id, &name, plot.x, y, plot.x + plot.w, y, &ln);
                    if !label.is_empty() {
                        let lid = ctx.register_named(&format!("{name}-label"), Kind::Shape);
                        let w = text_w(&label, &ann_font, ann_size) + 14.0;
                        let lx = if at_start { plot.x } else { plot.x + plot.w - w };
                        above += &text_sp(lid, &format!("{name}-label"), lx, y - line_h(ann_size) - 8.0, w, line_h(ann_size) + 8.0, &para(&label, &ann_font, ann_size, &cx_col, false, if at_start { "l" } else { "r" }, &lang), "<a:noFill/>", "b", &lang);
                    }
                }
            }
            "band" => {
                check_attrs(c, &[&["id", "from", "to", "fill"]])?;
                let a = cat_index(c, c.attribute("from").unwrap_or(""))?;
                let b = cat_index(c, c.attribute("to").unwrap_or(c.attribute("from").unwrap_or("")))?;
                let (a, b) = (a.min(b), a.max(b));
                let id = ctx.register_named(&name, Kind::Shape);
                let f = e(fill(c.attribute("fill").unwrap_or("accent1/10")))?;
                let (x0, x1) = (plot.c(a) - cat_span / 2.0, plot.c(b) + cat_span / 2.0);
                let (bx, by, bw, bh) = if horizontal { (plot.x, x0, plot.w, x1 - x0) } else { (x0, plot.y, x1 - x0, plot.h) };
                below += &text_sp(id, &name, bx, by, bw, bh, &format!("<a:p><a:endParaRPr lang=\"{lang}\"/></a:p>"), &f, "ctr", &lang).replace("val 12000", "val 0");
            }
            "note" => {
                check_attrs(c, &[&["id", "at", "series", "dx", "dy", "w", "fill", "color", "size", "font", "bold", "stroke"]])?;
                let at = c.attribute("at").map_or_else(|| err(c, "missing `at` (a category name)"), Ok)?;
                let ci = cat_index(c, at)?;
                let si = num(c, "series")?.unwrap_or(0.0) as usize;
                if si >= series.len() {
                    return err(c, format!("series index {si} does not exist"));
                }
                let (px, py) = point(ci, si);
                let w = num(c, "w")?.unwrap_or_else(|| (text_w(&txt, &ann_font, ann_size) + 16.0).min(220.0));
                let inner_w = w - 12.0;
                let lines = (text_w(&txt, &ann_font, ann_size) / inner_w).ceil().max(1.0);
                let h = lines * line_h(ann_size) + 8.0;
                let dx = num(c, "dx")?.unwrap_or(0.0);
                let dy = num(c, "dy")?.unwrap_or(-(h + 18.0) - if labels { line_h(label_size) } else { 0.0 });
                let (bx, by) = (px + dx - w / 2.0, py + dy);
                let ln = e(line(Some(c.attribute("stroke").unwrap_or("tx1/50 1")), "<a:tailEnd type=\"oval\" w=\"sm\" len=\"sm\"/>"))?;
                let lid = ctx.register_named(&format!("{name}-line"), Kind::Other);
                let (lx, ly) = (bx + w / 2.0, if dy < 0.0 { by + h } else { by });
                // Stop the leader above the data label, so the line does not cross the number.
                let label_shown = labels && series[si].label_points.as_ref().is_none_or(|k| k.contains(&ci));
                let end_y = if label_shown && !horizontal && dy < 0.0 && (label_pos == "outEnd" || label_pos == "t") { py - line_h(label_size) - 3.0 } else { py };
                above += &line_sp(lid, &format!("{name}-line"), lx, ly, px, end_y, &ln);
                let id = ctx.register_named(&name, Kind::Shape);
                let f = e(fill(c.attribute("fill").unwrap_or("tx1")))?;
                let tc = e(fill(c.attribute("color").unwrap_or("bg1")))?;
                above += &text_sp(id, &name, bx, by, w, h, &para(&txt, &ann_font, ann_size, &tc, flag(c, "bold")?.unwrap_or(false), "ctr", &lang), &f, "ctr", &lang);
            }
            _ => {
                // center: text in the middle of a doughnut/pie, or of the plot area.
                check_attrs(c, &[&["id", "size", "font", "color", "bold", "sub", "sub-size", "sub-color"]])?;
                let id = ctx.register_named(&name, Kind::Shape);
                let (cx, cy) = (plot.x + plot.w / 2.0, plot.y + plot.h / 2.0);
                let big = num(c, "size")?.unwrap_or(28.0);
                let mut body = para(&txt, &ann_font, big, &cx_col, flag(c, "bold")?.unwrap_or(false), "ctr", &lang);
                let mut h = line_h(big);
                if let Some(sub) = c.attribute("sub") {
                    let ss = num(c, "sub-size")?.unwrap_or(size);
                    body += &para(sub, &font, ss, &e(fill(c.attribute("sub-color").unwrap_or("tx1/60")))?, false, "ctr", &lang);
                    h += line_h(ss);
                }
                let w = plot.w * 0.7;
                above += &text_sp(id, &name, cx - w / 2.0, cy - h / 2.0 - 4.0, w, h + 8.0, &body, "<a:noFill/>", "ctr", &lang);
            }
        }
    }
    if !below.is_empty() {
        // Bands go under the chart: emitted with a marker the caller splits on.
        return Ok((graphic, format!("{below}\u{1}{above}")));
    }
    Ok((graphic, above))
}

/// Splits on commas that are not inside parentheses (colour lists may hold gradients).
fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    out.push(cur);
    out
}
