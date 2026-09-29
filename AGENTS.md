# Working on Weft

Weft is a Rust compiler for `.wft` files. Its canonical source is compact web intent; generated HTML, CSS, and JavaScript are build outputs.

## Non-negotiable design rules

- Keep generated output standards-native. Do not add React, JSX, a virtual DOM, automatic hydration, CSS-in-JS, or utility-class output.
- Keep client code opt-in. A page without `island` declarations must emit no JavaScript.
- Prefer HTML links and forms for ordinary navigation and mutations.
- Compile style intent to raw CSS using semantic selectors, custom properties, Grid/Flexbox, media queries, and modern browser features.
- Prefer a small, predictable grammar over punctuation-heavy syntax. Compression must not create ambiguity.
- Preserve source locations in parse/validation errors.
- Treat user-visible DSL text as untrusted: HTML-escape it at render time.
- Preserve `TextExpr` as the shared representation for every text-rendering language field; do not introduce isolated dynamic-string syntaxes.

## Commands

```sh
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Run all three before submitting a change. Add focused tests with every parser, validation, or compiler behavior change.

## Repository layout

- `src/` — compiler and CLI implementation.
- `tests/` — integration tests.
- `docs/language.md` — supported syntax contract.
- `docs/architecture.md` — compilation and output model.
- `docs/features/weft.md` — agreed direction and deferred scope.
- `examples/` — valid source fixtures, added alongside supported features.

## Editing guidance

- Modify the language reference and `llms.txt` whenever the public grammar changes.
- Keep the CLI error messages actionable: file, line, reason, and expected form where possible.
- Do not version `dist/` output unless an explicit fixture requires it.
- Use established Rust crates when they materially improve correctness or developer experience. Explain runtime, binary-size, and maintenance impact for new dependencies in the pull request.
