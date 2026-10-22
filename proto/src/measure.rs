//! Text measurement calibrated against PowerPoint 16 (proto/calib):
//! line pitch = 1.2 * spacing * size for every font; the first line is shorter when
//! spacing > 1 (0.9 * spacing + 0.24 em). Line breaking is greedy at spaces, like PowerPoint.

use crate::fonts;

pub struct MRun {
    pub text: String,
    pub family: String,
    pub bold: bool,
    pub size: f64,
    pub spacing: f64,
}

pub struct MPara {
    pub runs: Vec<MRun>,
    /// Line-height multiplier (a:lnSpc spcPct).
    pub line: f64,
    pub before: f64,
    pub after: f64,
    pub indent: f64,
    pub size: f64,
}

pub struct Layout {
    pub lines: usize,
    pub height: f64,
    pub max_line_width: f64,
    /// Words wider than the line: PowerPoint breaks them between letters.
    pub split_words: usize,
}

struct Ch {
    adv: f64,
    space: bool,
    newline: bool,
    size: f64,
}

fn chars(p: &MPara) -> Vec<Ch> {
    let mut out = Vec::new();
    for r in &p.runs {
        let face = fonts::face(&r.family, r.bold);
        for c in r.text.chars() {
            if c == '\n' {
                out.push(Ch { adv: 0.0, space: false, newline: true, size: r.size });
                continue;
            }
            let em = face.and_then(|f| f.advance(c)).unwrap_or(0.55) as f64;
            let bold_extra = if face.is_some_and(|f| f.synthetic_bold) { 0.02 } else { 0.0 };
            out.push(Ch { adv: (em + bold_extra) * r.size + r.spacing, space: c == ' ', newline: false, size: r.size });
        }
    }
    out
}

fn first_line(p: f64) -> f64 {
    if p <= 1.0 { 1.2 * p } else { 0.9 * p + 0.24 }
}

/// Lays out paragraphs in a box of inner width `width` (None = no wrapping).
pub fn layout(paras: &[MPara], width: Option<f64>) -> Layout {
    let mut height = 0.0;
    let mut lines_total = 0;
    let mut max_w: f64 = 0.0;
    let splits = std::cell::Cell::new(0usize);
    for (pi, p) in paras.iter().enumerate() {
        let w = width.map(|w| w - p.indent);
        // Each line: (width, max size).
        let mut lines: Vec<(f64, f64)> = Vec::new();
        let (mut cur, mut cur_size) = (0.0, 0.0f64);
        let (mut word, mut word_size) = (0.0, 0.0f64);
        let mut trailing = 0.0; // spaces after the last word on the line
        let cs = chars(p);
        let flush_word = |cur: &mut f64, cur_size: &mut f64, word: &mut f64, word_size: &mut f64, trailing: &mut f64, lines: &mut Vec<(f64, f64)>| {
            if *word == 0.0 {
                return;
            }
            if let Some(w) = w {
                if *cur > 0.0 && *cur + *trailing + *word > w + 0.01 {
                    lines.push((*cur, *cur_size));
                    *cur = 0.0;
                    *cur_size = 0.0;
                    *trailing = 0.0;
                }
                // A word longer than the line breaks between characters (approximation).
                if *word > w + 0.01 && w > 0.0 {
                    splits.set(splits.get() + 1);
                }
                while *word > w + 0.01 && w > 0.0 {
                    lines.push((w, word_size.max(*cur_size)));
                    *word -= w;
                }
            }
            *cur += *trailing + *word;
            *cur_size = cur_size.max(*word_size);
            *trailing = 0.0;
            *word = 0.0;
            *word_size = 0.0;
        };
        for c in &cs {
            if c.newline {
                flush_word(&mut cur, &mut cur_size, &mut word, &mut word_size, &mut trailing, &mut lines);
                lines.push((cur, if cur_size > 0.0 { cur_size } else { c.size }));
                cur = 0.0;
                cur_size = 0.0;
                trailing = 0.0;
            } else if c.space {
                flush_word(&mut cur, &mut cur_size, &mut word, &mut word_size, &mut trailing, &mut lines);
                trailing += c.adv;
            } else {
                word += c.adv;
                word_size = word_size.max(c.size);
            }
        }
        flush_word(&mut cur, &mut cur_size, &mut word, &mut word_size, &mut trailing, &mut lines);
        lines.push((cur, if cur_size > 0.0 { cur_size } else { p.size }));

        if pi > 0 {
            height += p.before;
        }
        for (li, (lw, ls)) in lines.iter().enumerate() {
            max_w = max_w.max(*lw + p.indent);
            height += if pi == 0 && li == 0 { first_line(p.line) * ls } else { 1.2 * p.line * ls };
        }
        if pi + 1 < paras.len() {
            height += p.after;
        }
        lines_total += lines.len();
    }
    Layout { lines: lines_total, height, max_line_width: max_w, split_words: splits.get() }
}
