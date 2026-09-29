import { execFile, type ChildProcess } from "node:child_process";
import * as vscode from "vscode";

const diagnosticSource = "weft";
const saveDelayMs = 150;
const lineError = /(?:^|\n).*?line\s+(\d+):\s*(.+)/i;

export function activate(context: vscode.ExtensionContext): void {
  const diagnostics = vscode.languages.createDiagnosticCollection(diagnosticSource);
  const output = vscode.window.createOutputChannel("Weft");
  const checks = new Map<string, ChildProcess>();
  const delays = new Map<string, NodeJS.Timeout>();
  output.appendLine("Weft extension activated.");

  const clear = (document: vscode.TextDocument): void => {
    const key = document.uri.toString();
    checks.get(key)?.kill();
    checks.delete(key);
    const delay = delays.get(key);
    if (delay) {
      clearTimeout(delay);
      delays.delete(key);
    }
    diagnostics.delete(document.uri);
  };

  const report = (document: vscode.TextDocument, text: string): void => {
    const match = lineError.exec(text);
    if (!match) {
      diagnostics.delete(document.uri);
      output.appendLine(`[check] ${document.uri.fsPath}\n${text.trim()}`);
      output.show(true);
      return;
    }
    const line = Math.min(Math.max(Number(match[1]) - 1, 0), document.lineCount - 1);
    const diagnostic = new vscode.Diagnostic(
      document.lineAt(line).range,
      match[2].trim(),
      vscode.DiagnosticSeverity.Error,
    );
    diagnostics.set(document.uri, [diagnostic]);
    output.appendLine(`[error] ${document.uri.fsPath}:${match[1]} ${match[2].trim()}`);
  };

  const check = (document: vscode.TextDocument): void => {
    if (document.languageId !== "weft" || document.uri.scheme !== "file") {
      return;
    }
    const key = document.uri.toString();
    checks.get(key)?.kill();
    const command = vscode.workspace.getConfiguration("weft", document.uri).get<string>("command", "weft");
    output.appendLine(`[check] ${command} check ${document.uri.fsPath}`);
    const child = execFile(command, ["check", document.uri.fsPath], { windowsHide: true }, (error, stdout, stderr) => {
      if (checks.get(key) !== child) {
        return;
      }
      checks.delete(key);
      if (!error) {
        diagnostics.delete(document.uri);
        output.appendLine(`[ok] ${document.uri.fsPath}`);
        return;
      }
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        diagnostics.delete(document.uri);
        output.appendLine(`Could not run '${command}'. Set weft.command to the installed Weft compiler path.`);
        output.show(true);
        return;
      }
      report(document, `${stderr}\n${stdout}`);
    });
    checks.set(key, child);
  };

  const scheduleCheck = (document: vscode.TextDocument): void => {
    const key = document.uri.toString();
    const pending = delays.get(key);
    if (pending) {
      clearTimeout(pending);
    }
    delays.set(key, setTimeout(() => {
      delays.delete(key);
      check(document);
    }, saveDelayMs));
  };

  for (const document of vscode.workspace.textDocuments) {
    check(document);
  }
  context.subscriptions.push(
    diagnostics,
    output,
    vscode.commands.registerCommand("weft.checkCurrentFile", () => {
      const document = vscode.window.activeTextEditor?.document;
      if (!document) {
        void vscode.window.showInformationMessage("Open a .wft file to check it with Weft.");
        return;
      }
      if (document.languageId !== "weft") {
        void vscode.window.showWarningMessage("The active file is not in Weft language mode.");
        return;
      }
      check(document);
      output.show(true);
    }),
    vscode.workspace.onDidOpenTextDocument(check),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (vscode.workspace.getConfiguration("weft", document.uri).get<boolean>("checkOnSave", true)) {
        scheduleCheck(document);
      }
    }),
    vscode.workspace.onDidCloseTextDocument(clear),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("weft")) {
        for (const document of vscode.workspace.textDocuments) {
          clear(document);
          check(document);
        }
      }
    }),
  );
}

export function deactivate(): void {
  // Child processes are cancelled when their document closes or the extension host exits.
}
