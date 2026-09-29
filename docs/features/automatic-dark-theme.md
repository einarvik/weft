# Automatic dark theme

## Summary

Allow a Weft theme block to describe a system-preference dark color scheme without client-side JavaScript. Authors declare the normal theme tokens as they do today, then nest only the color-role overrides that differ under `dark:`. Weft lowers those overrides to a standards-native `@media (prefers-color-scheme: dark)` rule.

```wft
theme:
  brand blue
  ink slate
  canvas white
  surface slate

  dark:
    brand sky
    ink white
    canvas black
    surface zinc
```

The nesting makes the base theme and its conditional variant visually distinct. It deliberately does not overload the existing `@="…"` section-style syntax or introduce a new punctuation convention.

## Current context

`src/lib.rs` currently parses both inline `theme: name value; …` declarations and two-space-indented `theme:` blocks into `Document.theme: BTreeMap<String, String>`. `src/style.rs` resolves color values to literal CSS custom properties at `:root`. The token validator accepts `brand`, `ink`, `canvas`, `surface`, `radius`, and `space`.

The stylesheet already consumes the color custom properties throughout the page, so dark mode can be implemented as a conditional override of the same four properties. There is no application-wide JavaScript runtime for static pages, and this feature must retain that property.

`docs/language.md` and `llms.txt` define the public grammar. The VS Code extension at `editors/vscode/` supplies TextMate highlighting and invokes `weft check`, so it must recognize the new keyword and use the compiler's validation unchanged.

The working tree currently contains a user-owned edit to `examples/acme.wft`; this feature must not overwrite or stage it.

## Goals

- Support `dark:` only as a child of a block-form `theme:` declaration.
- Allow `brand`, `ink`, `canvas`, and `surface` inside `dark:` using their existing color validation rules, plus `ink white` and `ink black` for readable scheme inversions.
- Preserve the base theme as the default stylesheet and emit dark overrides only when the source declares at least one.
- Use `@media (prefers-color-scheme: dark)` and CSS custom-property overrides; emit no JavaScript, island, DOM attribute, or persisted preference state.
- Keep errors line-aware and actionable.
- Document the grammar for people and agents, add focused Rust tests, and update VS Code syntax highlighting.

## Non-goals

- A manual light/dark switch, a `data-theme` attribute, user-preference persistence, or any client runtime.
- Dark-mode overrides for `radius` or `space`; those are layout tokens, not color-scheme tokens.
- Inline syntax such as `theme: ink slate; dark …`; the nested block is the intentional readable form.
- Named themes, more color schemes, arbitrary CSS, or per-section dark declarations.
- Changing the existing `@="wrap:…"` section-style grammar.

## Proposed behavior

### Grammar

```text
Theme       := InlineTheme | ThemeBlock
InlineTheme := "theme:" ThemePair (";" ThemePair)*
ThemeBlock  := "theme:" Newline (Indent ThemePair | Indent DarkBlock Newline)+
DarkBlock   := "dark:" Newline (Indent Indent DarkThemePair Newline)+
ThemePair   := ThemeToken ThemeValue
DarkThemePair := ColorThemeToken ColorThemeValue
ColorThemeToken := "brand" | "ink" | "canvas" | "surface"
```

The direct pairs and `dark:` child use one theme indentation level. A dark override uses one further level. `dark:` requires at least one override. A deeper child, a top-level `dark:`, an inline `dark` form, or a `radius`/`space` override produces a parse error at the relevant line.

Base and dark values are separately last-value-wins, consistent with the existing behavior across repeated theme declarations. A dark override does not require an explicit base pair: the normal default or the selected base value remains the light value.

### CSS output

For the example above, Weft retains its ordinary root declaration and appends a conditional rule:

```css
:root{--weft-brand:#2563eb;--weft-ink:#0f172a;--weft-canvas:#fff;--weft-surface:#f8fafc;…}
@media (prefers-color-scheme: dark){:root{--weft-brand:#0284c7;--weft-ink:#fff;--weft-canvas:#000;--weft-surface:#f4f4f5}}
```

Only supplied dark color roles appear in the media query. Omitted roles continue to inherit the base custom property. Dark `ink` also accepts `white` or `black`, which compile to literal neutral values; other dark values follow their existing role validation. A source file with no `dark:` block emits exactly no `prefers-color-scheme` CSS.

### Validation examples

```wft
theme:
  dark:
    radius lg
```

Fails on the `radius` line with an error that dark themes support `brand`, `ink`, `canvas`, and `surface` only.

```wft
theme:
  dark:
```

Fails on the `dark:` line because the block requires an indented color-token pair. A malformed nested indentation fails at the malformed child line.

## Affected areas

| Area | Change |
| --- | --- |
| Parser and AST | Add a separate dark-theme map to `Document`; parse and validate nested `dark:` blocks while retaining existing inline and base-block compatibility. |
| CSS compiler | Resolve declared dark color roles through the existing palette logic and conditionally emit the media query. |
| CLI validation | `weft check` benefits automatically through the shared parser and stylesheet render path. |
| VS Code | Highlight `dark` as a language keyword; compiler-driven diagnostics require no duplicate validation logic. |
| Documentation | Update language reference, `llms.txt`, the Acme example only if safe around user changes, and this feature record. |
| Tests | Add parser validation/indentation/compatibility coverage and CSS media-query/no-query coverage. |

No UI route, server/API, persistence, permissions, or external dependency is involved.

## Implementation plan

1. Extend the document theme representation with a dedicated map for dark color overrides, preserving `Document.theme` as the base map so existing consumers and callers stay simple.
2. Refactor block-theme parsing to accept validated direct theme pairs plus a `dark:` child. Parse its second-level pairs into the dedicated map and provide focused, line-numbered errors for empty, misplaced, invalid, or unsupported declarations.
3. Reuse the existing palette and role resolution in `src/style.rs` to render only the declared dark custom-property overrides inside `@media (prefers-color-scheme: dark)`.
4. Add parser tests for valid nesting, omitted base roles, empty/misindented blocks, dark-only allowed tokens, invalid colors, and legacy inline/base themes. Add stylesheet tests for literal overrides and the absence of the media rule without dark declarations.
5. Update `docs/language.md`, `llms.txt`, and VS Code grammar highlighting. Preserve the user's uncommitted Acme image edits; add a dark-theme example only if it does not conflict, otherwise leave the source fixture untouched.
6. Run Rust formatting, tests, Clippy, and the VS Code JSON/TypeScript checks. Compile a representative `.wft` fixture and inspect its generated CSS for the media query.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `npm --prefix editors/vscode run check`
- `npm --prefix editors/vscode run compile`
- Parse the documented block form and assert base/dark token maps contain their respective values.
- Render CSS with dark overrides and assert the exact `prefers-color-scheme: dark` media rule contains only declared override variables.
- Render CSS without a `dark:` declaration and assert it has no dark-mode media rule.
- Run `weft check` against valid and invalid nested theme fixtures to confirm source-line diagnostics.

## Risks and assumptions

- **Assumption:** System preference is the correct first dark-mode behavior. It is automatic and standards-native; a manual user preference is intentionally deferred because it needs opt-in client state and a precedence policy.
- **Compatibility:** Existing inline themes and existing two-space theme blocks remain valid and produce unchanged CSS when no `dark:` block exists.
- **Contrast:** The established palette role mappings are reused. This slice does not add contrast analysis or change existing palette choices.
- **Grammar:** `dark:` is intentionally available only in an indented theme block; the additional indentation is the visual boundary that keeps conditional values understandable.

## Approval gate

Implementation must not begin until this document is explicitly approved. Once approved, this file is the source of truth for the change.
