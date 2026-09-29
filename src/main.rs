use clap::{CommandFactory, Parser, Subcommand};
use std::{
    fs,
    path::{Path, PathBuf},
};

use weft::{
    island::render_javascript, parse, render::render_document, style::render_document as render_css,
};

/// Compile compact web intent to native web artifacts.
#[derive(Debug, Parser)]
#[command(
    name = "weft",
    version,
    about = "Compile compact web intent to native web artifacts",
    long_about = "Weft compiles .wft source files to standards-native HTML, raw CSS, and only the browser JavaScript required by explicit interactive islands."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// The .wft source file to compile.
    input: Option<PathBuf>,
    /// Directory for generated web artifacts.
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a .wft source file without writing web artifacts.
    Check {
        /// The .wft source file to validate.
        input: PathBuf,
    },
}

fn main() {
    if let Err(message) = run() {
        eprintln!("Weft: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Check { input }) => check(&input),
        None => build(cli.input.as_deref(), &cli.output),
    }
}

fn build(input: Option<&Path>, output: &Path) -> Result<(), String> {
    match input {
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
            let artifacts = artifacts(path)?;
            fs::create_dir_all(output).map_err(|error| {
                format!(
                    "could not create output directory {}: {error}",
                    output.display()
                )
            })?;
            write(&output.join("index.html"), &artifacts.html)?;
            write(&output.join("site.css"), &artifacts.css)?;
            let island_path = output.join("islands.js");
            if let Some(javascript) = artifacts.javascript {
                write(&island_path, &javascript)?;
            } else if island_path.exists() {
                fs::remove_file(&island_path).map_err(|error| {
                    format!("could not remove {}: {error}", island_path.display())
                })?;
            }
            println!("Wove {} into {}", path.display(), output.display());
        }
    }
    Ok(())
}

fn check(path: &Path) -> Result<(), String> {
    artifacts(path)?;
    println!("Checked {}", path.display());
    Ok(())
}

struct Artifacts {
    html: String,
    css: String,
    javascript: Option<String>,
}

fn artifacts(path: &Path) -> Result<Artifacts, String> {
    validate_source_path(path)?;
    let source = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let document = parse(&source).map_err(|error| error.to_string())?;
    let html = render_document(&document).map_err(|error| error.to_string())?;
    let css = render_css(&document);
    let javascript = render_javascript(&document).map_err(|error| error.to_string())?;
    Ok(Artifacts {
        html,
        css,
        javascript,
    })
}

fn validate_source_path(path: &Path) -> Result<(), String> {
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wft"))
    {
        return Ok(());
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("source");
    Err(format!("expected a .wft source file, received {name}"))
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents)
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}
