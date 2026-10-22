mod anim;
mod chart;
mod deck;
mod presets;
mod svgpath;
mod table;
mod expand;
mod fonts;
mod lint;
mod measure;
mod icons {
    include!(concat!(env!("OUT_DIR"), "/icons.rs"));
}
mod text;
mod util;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (input, output) = match args.as_slice() {
        [cmd, input, flag, output] if cmd == "build" && flag == "-o" => (PathBuf::from(input), PathBuf::from(output)),
        _ => {
            eprintln!("usage: slideml build <deck.xml> -o <out.pptx>");
            return ExitCode::from(2);
        }
    };
    match deck::compile(&input) {
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
}
