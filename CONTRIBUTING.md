# Contributing to Weft

Thank you for helping make web development more compact and more standards-native.

## Before opening a pull request

1. Keep a change focused on one behavior or layer.
2. Add or update tests for parser, validation, or output changes.
3. Update `docs/language.md`, `AGENTS.md`, and `llms.txt` when the public language contract changes.
4. Run:

   ```sh
   cargo fmt --check
   cargo test
   cargo clippy -- -D warnings
   ```

## Design review questions

- Does this remove implementation tokens or merely shorten syntax?
- Is the construct unambiguous to both humans and agents?
- Can the compiler emit ordinary, accessible web standards?
- Does it keep client JavaScript explicit and local?
- Is the generated CSS smaller and clearer than a hand-authored equivalent?

## Scope

Please discuss substantial grammar additions in an issue before implementing them. Small compiler correctness fixes and documentation improvements are welcome as focused pull requests.
