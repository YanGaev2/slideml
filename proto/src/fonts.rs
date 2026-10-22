//! Font lookup (Windows font registry, as GDI/PowerPoint does) and glyph metrics.

use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub struct Face {
    data: Vec<u8>,
    index: u32,
    upem: f32,
    /// Advance in em per char; None = glyph missing.
    cache: Mutex<HashMap<char, Option<f32>>>,
    /// PowerPoint fakes bold when the family has no bold face.
    pub synthetic_bold: bool,
}

impl Face {
    fn font(&self) -> FontRef<'_> {
        FontRef::from_index(&self.data, self.index).expect("font parsed at load")
    }
    /// Advance width in em, or None if the font has no glyph for `c`.
    pub fn advance(&self, c: char) -> Option<f32> {
        if let Some(v) = self.cache.lock().unwrap().get(&c) {
            return *v;
        }
        let f = self.font();
        let v = f.charmap().map(c).and_then(|g| f.glyph_metrics(Size::unscaled(), LocationRef::default()).advance_width(g)).map(|a| a / self.upem);
        self.cache.lock().unwrap().insert(c, v);
        v
    }
    pub fn has(&self, c: char) -> bool {
        c.is_whitespace() || self.advance(c).is_some()
    }
}

/// Registry font names ("Inter SemiBold", "Unbounded Bold") -> file path.
fn registry() -> &'static HashMap<String, PathBuf> {
    static R: OnceLock<HashMap<String, PathBuf>> = OnceLock::new();
    R.get_or_init(|| {
        let mut m = HashMap::new();
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        for hive in ["HKLM", "HKCU"] {
            let Ok(out) = std::process::Command::new("reg")
                .args(["query", &format!("{hive}\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts")])
                .output()
            else {
                continue;
            };
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let Some((name, file)) = line.trim().split_once("    REG_SZ    ") else { continue };
                let name = name.trim().trim_end_matches(" (TrueType)").trim_end_matches(" (OpenType)");
                let file = PathBuf::from(file.trim());
                let path = if file.is_absolute() { file } else { PathBuf::from(&windir).join("Fonts").join(file) };
                for n in name.split(" & ") {
                    m.insert(n.trim().to_lowercase(), path.clone());
                }
            }
        }
        m
    })
}

fn load(path: &PathBuf, family: &str) -> Option<(Vec<u8>, u32, f32)> {
    let data = std::fs::read(path).ok()?;
    let file = skrifa::raw::FileRef::new(&data).ok()?;
    let fonts: Vec<FontRef> = file.fonts().filter_map(|f| f.ok()).collect();
    // In a collection, pick the face whose family name matches.
    let mut index = 0;
    for (i, f) in fonts.iter().enumerate() {
        let fam = f.localized_strings(skrifa::string::StringId::FAMILY_NAME).english_or_first().map(|s| s.to_string()).unwrap_or_default();
        if fam.eq_ignore_ascii_case(family) {
            index = i as u32;
            break;
        }
    }
    let upem = fonts.get(index as usize)?.metrics(Size::unscaled(), LocationRef::default()).units_per_em as f32;
    Some((data, index, upem))
}

/// Face for a GDI family name and bold flag. None if the font is not installed.
pub fn face(family: &str, bold: bool) -> Option<&'static Face> {
    static CACHE: OnceLock<Mutex<HashMap<(String, bool), Option<&'static Face>>>> = OnceLock::new();
    let key = (family.to_lowercase(), bold);
    let cache = CACHE.get_or_init(Default::default);
    if let Some(f) = cache.lock().unwrap().get(&key) {
        return *f;
    }
    let reg = registry();
    let fam = key.0.as_str();
    let bold_path = reg.get(&format!("{fam} bold"));
    let (path, synthetic) = match (bold, bold_path) {
        (true, Some(p)) => (Some(p), false),
        _ => (reg.get(&format!("{fam} regular")).or_else(|| reg.get(fam)), bold),
    };
    let face = path.and_then(|p| load(p, family)).map(|(data, index, upem)| {
        &*Box::leak(Box::new(Face { data, index, upem, cache: Mutex::new(HashMap::new()), synthetic_bold: synthetic }))
    });
    cache.lock().unwrap().insert(key, face);
    face
}
