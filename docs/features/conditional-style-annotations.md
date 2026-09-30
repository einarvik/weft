# Conditional style annotations

## Summary

Add `@dark="…"` as a reusable conditional-style annotation for every Weft declaration that already accepts the normal `@="…"` style annotation. The annotation supplies compact semantic color-role overrides that apply only under the system dark-mode preference.

```wft
section features cards 3 @="wrap:wide gap:lg surface:soft" @dark="surface:zinc ink:white":
  card "One source of truth" "Keep context close to the work."
```

This fixes foreground/background contrast inside a component without making card styling global. It is intentionally part of the established style-annotation system, rather than a second theme grammar or a cards-only API.

## Current context

`Section` in `src/lib.rs` holds one optional normal style string. `split_style` recognizes a trailing `@="…"` annotation on a section header. `src/render.rs` validates the normal section style declarations and lowers them to `data-*` attributes. `src/style.rs` emits static raw CSS for those attributes.

The recently shipped `theme > dark` feature changes page-wide custom properties using `prefers-color-scheme: dark`. It cannot express a component whose light and dark surface/foreground pairing intentionally differs from the rest of the page. The Acme example is user-owned and currently unstaged; it must remain untouched by this feature.

Today `section` is the only public declaration that accepts `@="…"`. The feature must establish a reusable annotation representation rather than hard-code a cards-only `dark` field, so future styleable declarations can opt into the same syntax and compiler path.

## Goals

- Support an optional `@dark="key:value …"` next to the existing `@="key:value …"` on any currently styleable declaration.
- Preserve normal annotation behavior and allow either annotation to appear independently and in either order.
- Treat `@dark` as a dark-mode-local override of semantic color variables: `brand`, `ink`, `canvas`, and `surface`.
- Support named palettes for all four roles, `white` and `black` for `canvas`/`surface`, and `white`/`black` for `ink` in a dark annotation.
- Emit only scoped `@media (prefers-color-scheme: dark)` CSS and no JavaScript.
- Let an annotation override custom properties on its rendered scope so descendant components—including cards—automatically recompute their own surface, text, border, caption, and focus styling from the local values.
- Keep diagnostics source-line-aware, extend the language/agent documentation, and update VS Code syntax support in the same change.

## Non-goals

- A manual theme toggle, persistence, DOM mutation, or any client runtime.
- A separate global card-theme API.
- Per-card annotations in this slice; `card` does not currently support the normal `@="…"` annotation. When it does, it inherits `@dark` through the shared annotation model.
- Conditional layout values such as `@dark="gap:lg"`; dark annotations are color-role overrides in this slice.
- Arbitrary CSS values, selectors, custom properties, or color literals.
- Editing or committing the user's existing `examples/acme.wft` changes.

## Proposed behavior

### Syntax

```text
StyleableDeclaration := Declaration [StyleAnnotation] [DarkStyleAnnotation]
StyleAnnotation      := "@=" QuotedStyleDeclarations
DarkStyleAnnotation  := "@dark=" QuotedDarkDeclarations
QuotedStyleDeclarations := '"' (StyleKey ":" StyleValue " ")* '"'
QuotedDarkDeclarations  := '"' (ColorRole ":" DarkColorValue " ")* '"'
```

Both annotations are optional. They can occur in either order, but may appear at most once each. Any trailing text after annotations, duplicate annotation, malformed quote, or unsupported annotation name fails at the declaration line.

Existing syntax remains valid:

```wft
section work @="wrap:reading gap:lg":
```

The new annotation may stand alone:

```wft
section work @dark="ink:white":
```

### Semantics

`@dark` is a scoped, conditional theme override. It does not style a specific HTML tag directly. Instead, the compiler emits custom-property overrides on the declaration's rendered scope under the system dark-mode media query.

For a cards section:

```wft
section features cards 3 @dark="surface:zinc ink:white":
```

the generated CSS has the equivalent behavior:

```css
@media (prefers-color-scheme: dark) {
  .weft-section[data-dark-surface="zinc"] { --weft-surface: #18181b; }
  .weft-section[data-dark-ink="white"] { --weft-ink: #fff; }
}
```

The section itself must establish `color: var(--weft-ink)` so a local `ink` variable becomes an inherited foreground color. Existing cards already consume `--weft-surface` and inherit foreground color, so their visual pairing updates together. Existing `surface:soft` uses the same local `--weft-surface` variable and therefore follows the override automatically.

The exact selectors and literal palette shade are compiler details; source only expresses semantic roles. A page without `@dark` annotations must emit no annotation-specific dark CSS.

### Validation

Allowed dark roles and values:

| Role | Values |
| --- | --- |
| `brand` | named palette |
| `ink` | named palette, `white`, `black` |
| `canvas` | named palette, `white`, `black` |
| `surface` | named palette, `white`, `black` |

Examples that fail on the section-header line:

```wft
section features cards 3 @dark="gap:lg":
section features cards 3 @dark="surface:vermillion":
section features cards 3 @dark="ink:white" @dark="surface:zinc":
```

Diagnostics identify the invalid annotation or declaration and describe the accepted form. The compiler preserves existing normal-style validation: normal `surface:soft` remains a presentation recipe, whereas dark `surface:zinc` is a theme color role.

## Affected areas

| Area | Change |
| --- | --- |
| Parser/AST | Replace `Section.style` with a reusable annotation representation that holds optional normal and dark strings; parse both trailer annotations safely. |
| HTML renderer | Validate and lower normal/dark style declarations to safe `data-*` attributes; retain current normal attributes byte-for-byte where possible. |
| CSS compiler | Detect declared dark annotation values and emit scoped variable overrides under `prefers-color-scheme: dark`; establish section foreground color from its local ink variable. |
| VS Code | Highlight `@dark` within the existing style annotation grammar; compiler diagnostics remain authoritative. |
| Documentation | Update `docs/language.md` and `llms.txt` with grammar, role/value constraints, examples, and no-JS behavior. |
| Tests | Cover both annotation orders, standalone dark annotations, duplicate/malformed/unsupported diagnostics, emitted HTML attributes, scoped CSS, inherited card contrast, and legacy normal styles. |

No server/API, persistence, dependency, permission, or external integration change is needed.

## Implementation plan

1. Add a small reusable `StyleAnnotations` AST type with optional normal and dark declarations; attach it to `Section` in place of the one normal style string. Keep it suitable for future declarations that become styleable.
2. Replace `split_style` with an annotation trailer parser that recognizes `@=` and `@dark=`, permits either order, rejects duplicates/trailing garbage, and preserves source-line diagnostics.
3. Refactor style-declaration validation into shared normal and dark-role validators. Render safe `data-*` attributes for each accepted declaration without exposing raw source to HTML.
4. Extend CSS generation to discover dark annotation roles and emit compact scoped custom-property rules inside `@media (prefers-color-scheme: dark)`. Add `color:var(--weft-ink)` to the section baseline so local foreground overrides inherit through cards and semantic content.
5. Add parser, HTML, and CSS regression tests for valid and invalid syntax plus existing normal styles. Verify that no `@dark` source yields no conditional annotation CSS.
6. Update `docs/language.md`, `llms.txt`, and the VS Code TextMate style pattern. Do not modify or stage `examples/acme.wft`.
7. Run Rust formatting, tests, Clippy, VS Code TypeScript/JSON checks, and a representative `weft check` command.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `npm --prefix editors/vscode run check`
- `npm --prefix editors/vscode run compile`
- JSON parse check for the VS Code grammar.
- Parse normal-only, dark-only, and mixed-order annotations and assert their separate AST fields.
- Render a dark cards annotation and assert its HTML has safe data attributes and its CSS has a scoped media-query variable override plus section foreground inheritance.
- Render source without `@dark` and assert no annotation-specific dark selector/rule appears.
- Run `weft check` on the user-edited Acme example without staging or modifying it.

## Risks and assumptions

- **Assumption:** A local custom-property override is the correct general model. It allows current and future descendants to follow component dark mode without each component needing a bespoke dark selector.
- **Compatibility:** Existing section `@="…"` syntax and generated normal-style attributes remain supported. The global `theme > dark` feature remains separate and composes naturally with local `@dark` overrides.
- **Scope:** This initial slice only exposes the new annotation wherever `@=` is already legal: sections. The AST/parser model is intentionally reusable; widening normal styleability to cards or other declarations is a separate grammar feature.
- **Accessibility:** `ink` and `surface` are defined together but cannot guarantee every palette pairing passes every contrast target. Weft maintains its existing constrained palettes; automated contrast analysis is deferred.

## Approval gate

Implementation must not begin until this document is explicitly approved. Once approved, this file is the source of truth for the change.
