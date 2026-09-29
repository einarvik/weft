# Weft for VS Code

This extension provides syntax highlighting, indentation support, and compiler diagnostics for `.wft` files. It delegates language validation to the Rust Weft compiler; it does not bundle a second parser or a website runtime.

## Local development

Build the compiler from the repository root:

```sh
cargo build
```

Install extension development dependencies:

```sh
cd editors/vscode
npm install
```

Open the `editors/vscode/` folder itself in desktop VS Code. Then press **F5**, or open the Run and Debug view and choose **Run Weft Extension**. VS Code runs the `compile extension` task and opens a second window titled **[Extension Development Host]**. That temporary window is a clean VS Code instance with this local extension loaded; it does not install or alter the extension in your normal VS Code profile.

The launch configuration explicitly uses VS Code's own executable and continues immediately (`stopOnEntry: false`). If VS Code previously showed an “Extension host did not start” warning, stop the old debug session, close its development-host window, and press F5 again from `editors/vscode/` after pulling this update.

In the Extension Development Host, open the repository root (or another folder containing `.wft` files). Point the extension at the local compiler if `weft` is not on your `PATH`:

```json
{
  "weft.command": "/absolute/path/to/weft/target/debug/weft",
  "weft.checkOnSave": true
}
```

Open `examples/acme.wft` to verify syntax highlighting and diagnostics. The extension invokes `weft check file.wft` when a Weft file opens and after it is saved. The command validates without creating `dist` or generated artifacts. A missing compiler is reported in the **Weft** output channel; source errors become native editor diagnostics.

To test the bridge explicitly, open a `.wft` file and run **Weft: Check Current File** from the Command Palette. The **Output** panel's **Weft** channel shows the exact compiler command, then either `[ok]` or `[error]`. This is also the first place to check if diagnostics do not appear.

If the Extension Development Host debugger itself does not start, launch a separate local host without the debugger from the repository root on macOS:

```sh
open -na "Visual Studio Code" --args --extensionDevelopmentPath="$(pwd)/editors/vscode" "$(pwd)"
```

That new window loads the extension directly. Open `examples/acme.wft` there and run **Weft: Check Current File**. If it works, the extension is sound and only the local VS Code debug session needs attention.

## Scope

This first slice does not publish a marketplace package and does not include comments, an LSP, completions, formatting, hovers, or code actions. `#` remains available for fragment links such as `-> #demo`; a comment syntax needs its own DSL feature. The Rust compiler remains the authoritative grammar and validation implementation.
