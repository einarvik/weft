# VS Code extension foundation

## Summary

Ship the first in-repository VS Code integration for `.wft` files: syntax-aware editing, safe comment/indent behavior, and compiler-backed diagnostics on open/save. The extension will be a thin TypeScript adapter over the Rust Weft CLI, not a second parser or a browser runtime. A new `weft check` command will validate a document without creating `dist` output, making editor feedback safe and fast.

## Current Context

Weft's Rust compiler has a line-oriented parser in `src/lib.rs` and a CLI in `src/main.rs`. The CLI currently accepts a `.wft` path and writes generated artifacts to `dist`. Its diagnostics are already line-aware (`line N: reason`) but are printed as a string and do not have a no-output validation mode. `docs/language.md` and `llms.txt` are the grammar source of truth.

The repository has no Node manifest, editor directory, VS Code grammar, language configuration, extension tests, or language server. The working tree contains user-owned, unstaged card-image edits in `examples/acme.wft`; this feature must not stage, replace, or depend on those edits.

## Goals

- Maintain VS Code support in this repository under `editors/vscode/`.
- Associate `.wft` files with a `weft` language mode.
- Provide readable syntax highlighting for Weft keywords, statements, quoted strings, operators, numbers/ranges, style annotations, and comments.
- Provide language configuration for two-space indentation behavior.
- Add `weft check FILE.wft`, which validates source without writing output files.
- Run `weft check` from the extension when a `.wft` document opens or saves, and publish errors as native VS Code diagnostics.
- Make the compiler executable configurable through `weft.command`, defaulting to `weft` on `PATH`; make save-time checking configurable through `weft.checkOnSave`, defaulting to `true`.
- Gracefully report a missing/non-runnable compiler in the extension output channel without creating a false source diagnostic.
- Document local extension installation, compiler configuration, and the exact first-slice limitations for developers and agents.

## Non-Goals

- Publishing a VSIX or marketplace package, creating a VS Marketplace publisher, telemetry, auto-updating, or downloading a Rust binary.
- An LSP server, live-as-you-type validation, hover documentation, completions, formatting, code actions, go-to-definition, or named-island navigation.
- Duplicating Weft's parser/validator in TypeScript.
- Watching generated `dist` files, adding a development server, or changing generated HTML/CSS/JS behavior.
- Changing the DSL grammar beyond the `check` CLI invocation.

## Proposed Behavior

### Repository layout and package boundary

```txt
editors/vscode/
  package.json
  tsconfig.json
  src/extension.ts
  syntaxes/weft.tmLanguage.json
  language-configuration.json
  README.md
```

`package.json` identifies a VS Code extension named **Weft**, registers `.wft`, contributes the TextMate grammar and language configuration, exposes `weft.command` / `weft.checkOnSave`, and activates on Weft documents. It contains development-only TypeScript/VS Code type dependencies. The production extension delegates all language semantics to the installed Rust binary and does not ship a website runtime or DSL implementation.

The manifest can use a local development publisher identifier; marketplace publishing is deliberately deferred until the project has a registered publisher account.

### Syntax and editor behavior

The extension recognizes the current documented grammar, including:

- document and block keywords (`site`, `theme`, `page`, `hero`, `section`, `card`, `image`, `island`, `use`, `state`, `range`, `label`, `title`, `eyebrow`, `text`, `actions`, `alt`, `caption`);
- string literals and `+`, `*`, `->`, `..` operators;
- integer values, theme/style values, and `@="…"` style annotations;
- fragment links such as `-> #demo` without misclassifying them as comments.

The grammar is visual/editor affordance only; the Rust compiler remains authoritative. The language configuration supplies brackets and two-space indentation rules appropriate to the existing DSL. Weft does not currently have comment syntax: `#` is reserved for fragment links such as `-> #demo`.

### `weft check`

The CLI gains a check mode with this user-facing form:

```sh
weft check path/to/page.wft
```

It reads the `.wft` file and performs the same parsing, semantic rendering validation, CSS validation, and island validation that compilation performs. It writes no `dist` directory or generated files. On success it prints a concise confirmation and exits `0`; on failure it preserves the current `Weft: line N: …` style on stderr and exits non-zero.

The existing compilation form remains backward compatible:

```sh
weft path/to/page.wft --output dist
```

Both forms reject non-`.wft` inputs with the existing actionable error convention. The internal CLI structure should centralize validation so check and compile cannot diverge.

### Diagnostics

For each open/save event on a `.wft` editor document, the extension runs:

```txt
<configured command> check <document filesystem path>
```

It converts compiler failures containing `line N:` into a VS Code error diagnostic on line `N - 1`, covering the full line. It clears stale diagnostics on success, document close, command/configuration change, and when an editor document is no longer Weft. Diagnostics are debounced/cancelled so an older process cannot overwrite a newer save result.

If execution fails because the command is missing or cannot start, the extension writes an actionable message to a `Weft` output channel such as “Set `weft.command` to the installed compiler path.” It does not blame the source file with a fabricated compiler error.

### Configuration and local use

The first slice supports:

```json
{
  "weft.command": "weft",
  "weft.checkOnSave": true
}
```

Developers can open `editors/vscode/` in VS Code, install its dependencies, and use the Extension Development Host to test against a locally built Weft binary. The extension README will show the required `cargo build` / `cargo install --path .` options and a workspace setting pointing at `target/debug/weft` when the compiler is not on `PATH`.

## Affected Areas

- **Rust CLI (`src/main.rs`, CLI tests):** Add a non-writing `check` subcommand while retaining compile behavior and actionable errors.
- **VS Code extension (`editors/vscode/`):** Add manifest, TypeScript activation/diagnostic bridge, syntax grammar, language configuration, and local developer instructions.
- **Documentation:** Add a VS Code setup page/reference from `README.md`; update agent-facing documentation to identify the compiler as the single grammar authority and describe `weft check`.
- **Dependencies:** Rust gains no new dependency. The extension gains development-only Node packages (`typescript`, `@types/node`, and `@types/vscode`) for extension compilation/type checking; no package runs in generated websites.
- **Tests:** Add Rust CLI tests for `check` success/failure/no output. Add extension package/type-check verification and grammar fixtures or documented manual editor verification.

There is no change to HTML/CSS/JS output, persistence, server/API, auth, routing, islands, or the `.wft` source grammar.

## Implementation Plan

1. Refactor the CLI argument model into explicit `build`/`check` behavior, preserving the current no-subcommand build invocation as a compatibility alias. Add an internal validation path that executes parse, HTML, CSS, and island validation without filesystem writes.
2. Add CLI integration tests confirming a valid `check` exits successfully without creating output, a `.wft` syntax/semantic error retains its line, and the existing compile path is unchanged.
3. Create `editors/vscode/` with a minimal TypeScript extension manifest/build configuration, `.wft` language contribution, TextMate grammar, and language configuration. Keep the extension source small and dependency-free at runtime.
4. Implement an output channel plus cancellable/debounced open/save diagnostics that invokes configurable `weft check`, maps line diagnostics, clears stale results, and handles missing binaries separately from source errors.
5. Add extension README setup instructions; reference the editor integration in the root README, language documentation, and agent reference. Make no marketplace publication attempt.
6. Run Rust formatting/tests/Clippy plus extension dependency install, TypeScript compilation, and a local Extension Development Host smoke test where available. Verify a broken `.wft` produces a native editor error and a valid file clears it.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `weft check examples/acme.wft` succeeds and creates no generated output directory.
- A malformed `.wft` causes `weft check` to exit non-zero and reports the source line.
- The existing `weft examples/acme.wft --output …` still produces native artifacts.
- In `editors/vscode/`, install declared development dependencies and run the TypeScript build/type check.
- Launch VS Code's Extension Development Host; confirm `.wft` highlighting, fragment-link highlighting, initial diagnostics, save-time diagnostics, clearing on valid source, and the missing-compiler output-channel message.
- Confirm the local user-owned card-image hunk in `examples/acme.wft` remains unstaged/unmodified by this feature work.

## Risks and Assumptions

- **Assumption:** The developer has a locally built or installed `weft` executable. Binary discovery/download and marketplace distribution are separate release work.
- **Assumption:** Current one-line compiler diagnostics are sufficient for the first editor slice; a Rust LSP can later provide exact columns, rich ranges, hover, and completions.
- **Risk:** VS Code extension packaging requires a small TypeScript/Node toolchain despite Weft itself being Rust. That toolchain is isolated under `editors/vscode/` and never participates in generated web output.
- **Risk:** A TextMate grammar can drift from the DSL. It is intentionally non-authoritative and will be covered by examples/fixtures; compiler diagnostics remain the truth.
- **Compatibility:** Existing compiler invocation and generated artifacts remain unchanged. The only CLI addition is `weft check`.

## Approval Gate

Implementation must not begin until the user explicitly approves this document. Once approved, it is the source of truth for the VS Code extension foundation.
