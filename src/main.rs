use std::{env, path::Path};

const HELP: &str = "Weft — compile compact web intent to native web artifacts

Usage:
  weft <source.wft>

The initial compiler is being woven. See docs/features/weft.md for the language contract.";

fn main() {
    let input = env::args().nth(1);

    match input.as_deref() {
        None => {
            println!("{HELP}");
            std::process::exit(1);
        }
        Some("--help" | "-h") => println!("{HELP}"),
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
