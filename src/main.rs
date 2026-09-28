use clap::Parser;
use std::path::Path;

/// Compile compact web intent to native web artifacts.
#[derive(Debug, Parser)]
#[command(
    name = "weft",
    version,
    about = "Compile compact web intent to native web artifacts",
    long_about = "Weft compiles .wft source files to standards-native HTML, raw CSS, and only the browser JavaScript required by explicit interactive islands."
)]
struct Cli {
    /// The .wft source file to compile.
    input: Option<String>,
}

const MISSING_INPUT_HELP: &str = "Weft — compile compact web intent to native web artifacts

Usage:
  weft <source.wft>

The initial compiler is being woven. See docs/features/weft.md for the language contract.";

fn main() {
    let cli = Cli::parse();

    match cli.input.as_deref() {
        None => {
            println!("{MISSING_INPUT_HELP}");
            std::process::exit(1);
        }
        Some(path)
            if !Path::new(path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("wft")) =>
        {
            let name = Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(path);
            eprintln!("Weft expects a .wft source file, received {name}.");
            std::process::exit(1);
        }
        Some(_) => {
            eprintln!("Weft source compilation is not available yet.");
            std::process::exit(1);
        }
    }
}
