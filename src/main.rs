use clap::{CommandFactory, Parser};
use std::{
    fs,
    path::{Path, PathBuf},
};

use weft::{render::render_html, style::render_css};

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
    input: Option<PathBuf>,
    /// Directory for generated web artifacts.
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}

fn main() {
    if let Err(message) = run() {
        eprintln!("Weft: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.input.as_deref() {
        None => {
            Cli::command()
                .print_help()
                .map_err(|error| error.to_string())?;
            println!();
        }
        Some(path)
            if !path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("wft")) =>
        {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("source");
            return Err(format!("expected a .wft source file, received {name}"));
        }
        Some(path) => {
            let source = fs::read_to_string(path)
                .map_err(|error| format!("could not read {}: {error}", path.display()))?;
            let html = render_html(&source).map_err(|error| error.to_string())?;
            let css = render_css(&source).map_err(|error| error.to_string())?;
            fs::create_dir_all(&cli.output).map_err(|error| {
                format!(
                    "could not create output directory {}: {error}",
                    cli.output.display()
                )
            })?;
            write(&cli.output.join("index.html"), &html)?;
            write(&cli.output.join("site.css"), &css)?;
            println!("Wove {} into {}", path.display(), cli.output.display());
        }
    }
    Ok(())
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents)
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}
