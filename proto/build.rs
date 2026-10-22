//! Embeds Lucide icons (vendor/lucide, ISC license) as a sorted (name, svg) table.
use std::{env, fs, path::Path};

fn main() {
    let dir = Path::new("vendor/lucide/package/icons");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut icons: Vec<(String, String)> = fs::read_dir(dir)
        .expect("vendor/lucide/package/icons missing: run `npm pack lucide-static` in vendor/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "svg"))
        .map(|e| {
            let name = e.path().file_stem().unwrap().to_string_lossy().into_owned();
            let svg = fs::read_to_string(e.path()).unwrap();
            let svg = svg[svg.find("<svg").unwrap()..].split_whitespace().collect::<Vec<_>>().join(" ");
            (name, svg)
        })
        .collect();
    icons.sort();
    let mut out = String::from("pub static ICONS: &[(&str, &str)] = &[\n");
    for (n, s) in &icons {
        out += &format!("    ({n:?}, r#\"{s}\"#),\n");
    }
    out += "];\n";
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("icons.rs"), out).unwrap();
}
