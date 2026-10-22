p = 'F:/pptx-ai-native/proto/src/deck.rs'
s = open(p, encoding='utf-8').read()
def rep(old, new):
    global s
    assert old in s, old[:70]
    s = s.replace(old, new, 1)
rep('''    charts: u32,
}''', '''    charts: u32,
    pub warnings: Vec<String>,
    manifest: Vec<String>,
    size: (f64, f64),
}

pub struct Output {
    pub pptx: Vec<u8>,
    pub manifest: String,
    pub warnings: Vec<String>,
}''')
rep('''    pkg.add(&format!("ppt/slides/slide{idx}.xml"),''', '''    let lints = crate::lint::lint(&ctx.slide_id, &ctx.items, pkg.size.0, pkg.size.1);
    pkg.warnings.append(&mut ctx.warnings);
    pkg.warnings.extend(lints);
    pkg.manifest.push(crate::lint::slide_json(&ctx.slide_id, &ctx.items));
    pkg.add(&format!("ppt/slides/slide{idx}.xml"),''')
rep('''pub fn compile(path: &Path) -> Res<Vec<u8>> {''', '''pub fn compile(path: &Path) -> Res<Output> {''')
rep('''    let mut pkg = Package { parts: vec![], overrides: vec![], media: HashMap::new(), charts: 0 };''',
    '''    let mut pkg = Package { parts: vec![], overrides: vec![], media: HashMap::new(), charts: 0, warnings: vec![], manifest: vec![], size: (w, h) };''')
rep('''    zip_parts(&parts)
}''', '''    Ok(Output {
        pptx: zip_parts(&parts)?,
        manifest: format!("{{\\"w\\":{w},\\"h\\":{h},\\"slides\\":[{}]}}", pkg.manifest.join(",")),
        warnings: std::mem::take(&mut pkg.warnings),
    })
}''')
open(p, 'w', encoding='utf-8').write(s)

m = open('F:/pptx-ai-native/proto/src/main.rs', encoding='utf-8').read()
old = m[m.index("    match deck::compile(&input) {"):m.rindex("}")]
new = '''    match deck::compile(&input) {
        Ok(out) => {
            if let Err(e) = std::fs::write(&output, &out.pptx) {
                eprintln!("{}: {e}", output.display());
                return ExitCode::FAILURE;
            }
            let manifest = output.with_extension("manifest.json");
            let _ = std::fs::write(&manifest, &out.manifest);
            for w in &out.warnings {
                eprintln!("{w}");
            }
            println!("wrote {} ({} warnings), geometry in {}", output.display(), out.warnings.len(), manifest.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
'''
m = m.replace(old, new)
m = m.replace("mod icons {", "mod fonts;\nmod lint;\nmod measure;\nmod icons {")
open('F:/pptx-ai-native/proto/src/main.rs', 'w', encoding='utf-8').write(m)
print('ok')
