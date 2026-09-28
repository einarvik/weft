# Weft

Weft is a free, open-source language for describing websites in compact, semantic source files. A Weft compiler lowers `.wft` files into native HTML, raw CSS, and only the browser JavaScript required by explicitly declared interactive islands.

```wft
site Acme
theme: brand violet; ink slate; radius lg

page /:
  hero:
    title "Ship projects without the ceremony."
    text "Plan, discuss, and deliver from one calm workspace."
    actions:
      "Start free" -> /signup primary
```

Weft is not a React abstraction, a utility-CSS dialect, or a default-hydrated SPA framework. Links remain links, static sections remain HTML, and its styling declarations compile into regular CSS.

## Status

Weft is in active early development. The repository currently contains the Rust CLI foundation and the language contract. The parser and compiler are being built in small, reviewable commits.

## Principles

- **Intent over scaffolding.** Source expresses decisions, not framework ceremony.
- **Standards-native output.** Generated sites use HTML, CSS, Fetch/HTTP, and small DOM modules.
- **Static by default.** JavaScript is emitted only for a declared `island`.
- **Raw CSS output.** No CSS-in-JS runtime and no generated utility-class system.
- **Agent-readable by design.** Grammar, defaults, and limits are stable, concise, and documented.

## Requirements

- Rust 1.91 or newer.

## Development

```sh
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Show CLI help:

```sh
cargo run -- --help
```

## Documentation

- [Language direction](docs/features/weft.md)
- [Language reference](docs/language.md)
- [Architecture](docs/architecture.md)
- [Contributing](CONTRIBUTING.md)
- [Instructions for coding agents](AGENTS.md)
- [Compact agent reference](llms.txt)

## License

Weft is released under the [MIT License](LICENSE).
