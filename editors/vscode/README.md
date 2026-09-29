# Weft for VS Code

This extension provides syntax highlighting, indentation support, and compiler diagnostics for `.wft` files. It delegates language validation to the Rust Weft compiler; it does not bundle a second parser or a website runtime.

## Local development

Build the compiler from the repository root:

```sh
cargo build
```

Install extension development dependencies and compile it:

```sh
cd editors/vscode
npm install
npm run compile
```

Open `editors/vscode/` in VS Code and run **Extension: Run Extension**. In the Extension Development Host, point the extension at the local compiler if `weft` is not on your `PATH`:

```json
{
  "weft.command": "/absolute/path/to/weft/target/debug/weft",
  "weft.checkOnSave": true
}
```

The extension invokes `weft check file.wft` when a Weft file opens and after it is saved. The command validates without creating `dist` or generated artifacts. A missing compiler is reported in the **Weft** output channel; source errors become native editor diagnostics.

## Scope

This first slice does not publish a marketplace package and does not include comments, an LSP, completions, formatting, hovers, or code actions. `#` remains available for fragment links such as `-> #demo`; a comment syntax needs its own DSL feature. The Rust compiler remains the authoritative grammar and validation implementation.
