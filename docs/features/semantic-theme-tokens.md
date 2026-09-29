# Semantic theme tokens

## Summary

Make Weft's compact `theme:` declaration predictable, expressive, and safe for agent-authored source. In particular, `brand red` must compile to Weft's red brand value rather than silently falling back to violet. The compiler will validate every theme token and value with a source line, expose a small semantic color system in generated raw CSS, and support both concise inline and readable indented declarations.

## Current Context

`src/lib.rs` parses `theme:` declarations as arbitrary `name value` pairs into `Document.theme`. `src/style.rs` later recognizes only five color names (`violet`, `slate`, `blue`, `rose`, and `amber`) for `brand` and `ink`; all other values quietly use a hard-coded fallback. It recognizes `radius` values `sm`, `lg`, and `pill`, and `space` values `compact` and `roomy`; unsupported values also quietly fall back.

The checked-in Acme example currently uses `theme: brand red; ...`, which exposes the problem: parsing succeeds but the emitted stylesheet uses the violet fallback. The static CSS baseline currently hard-codes the page canvas to white and derives card borders/surfaces from only `brand` and `ink`.

The public syntax is documented in `docs/language.md` and `llms.txt`. `tests/style.rs` currently asserts the violet output, while parser coverage verifies only arbitrary theme-map storage.

## Goals

- Keep the compact, line-oriented declaration form:

  ```wft
  theme: brand red; ink slate; canvas white; surface slate; radius lg; space compact
  ```

- Support `red` as a valid `brand` palette value.
- Validate known theme token names and values during compilation, with the `theme:` source line in every diagnostic.
- Compile semantic color roles to raw CSS custom properties and raw CSS rules only; do not introduce a CSS runtime, utility classes, or authored CSS escape hatch.
- Provide a palette broad enough for ordinary web work while keeping its vocabulary finite and highly compressible for agents.
- Make the page canvas and card surface theme-driven, preserving readable default contrast.
- Preserve the current theme grammar and compatibility for every currently valid documented theme value.

## Non-Goals

- Arbitrary CSS declarations, selectors, custom properties, color functions, or user-provided CSS values in `.wft` source.
- Dark-mode switching, multiple named themes, per-component themes, or runtime theme switching.
- Typography, icon, shadow, motion, or arbitrary layout token systems.
- Changing the existing constrained section style declaration grammar such as `@="gap:lg surface:soft"`.
- Changing Weft's standards-native output model or introducing a client dependency.

## Proposed Behavior

### Grammar

Weft supports two equivalent forms:

```wft
theme: brand red; ink slate; canvas white; surface slate; radius lg; space compact
```

```wft
theme:
  brand red
  ink slate
  canvas white
  surface slate
  radius lg
  space compact
```

The inline form remains useful when a theme is small. The block form makes a more deliberate theme easy to scan, edit, and extend. A `theme:` block accepts only direct, two-space-indented token/value pairs; it cannot contain page blocks or other declarations. An inline `theme:` declaration has no children. Mixing inline pairs with child pairs is invalid.

The grammar is:

```txt
Theme := InlineTheme | BlockTheme
InlineTheme := "theme:" ThemePair (";" ThemePair)*
BlockTheme := "theme:" Newline (Indent ThemePair Newline)+
ThemePair := TokenName TokenValue
```

Each `ThemePair` is validated as it is parsed. Duplicate keys remain last-value-wins, matching the existing `BTreeMap` behavior, including when later pairs are in another `theme:` declaration, but an invalid earlier pair still fails compilation instead of being ignored.

### Supported tokens

| Token | Accepted values | CSS role |
| --- | --- | --- |
| `brand` | A named palette | Main interactive/action color and focus outline. |
| `ink` | A named palette | Main foreground/text color. |
| `canvas` | `white`, `black`, or a named palette | Page background. |
| `surface` | `white`, `black`, or a named palette | Card/background surface. |
| `radius` | `sm`, `md`, `lg`, `pill` | Corner radius. |
| `space` | `tight`, `compact`, `normal`, `roomy` | Baseline layout gap. |

`brand` and `ink` default to `violet` and `slate`. `canvas` defaults to `white`; `surface` defaults to a soft neutral derived from the canvas/ink baseline. `radius` defaults to `md`; `space` defaults to `normal`.

The named palette vocabulary is deliberately finite:

```txt
red orange amber yellow lime green emerald teal cyan sky blue indigo
violet purple fuchsia pink rose slate gray zinc stone
```

The compiler selects a role-appropriate stable shade from each palette: a strong accessible shade for `brand`, a dark shade for `ink`, and a pale shade for `canvas` and `surface`. It writes the resulting literal colors to generated CSS, rather than depending on a browser color-name interpretation. `white` and `black` are allowed only for `canvas` and `surface` in this slice; `brand` and `ink` always use a palette with intended contrast.

### Generated CSS contract

The root rule will expose semantic variables:

```css
:root{--weft-brand:#dc2626;--weft-ink:#0f172a;--weft-canvas:#fff;--weft-surface:#f8fafc;--weft-radius:1rem;--weft-space:.75rem}
```

The base `body` rule uses `--weft-canvas`; cards use `--weft-surface`. Existing local `surface:plain` and `surface:soft` declarations continue to mean their current section-level presentation intent. `surface:soft` will blend from the selected theme surface/brand rather than assuming a white canvas, so the theme remains internally consistent.

No JavaScript is emitted or changed by this feature.

### Errors

Unknown names and values stop compilation. Diagnostics identify the `theme:` line and list the accepted form, for example:

```txt
examples/acme.wft:2: unsupported `brand` value `vermillion`; expected a named palette such as `red`, `blue`, or `violet`
examples/acme.wft:2: unknown theme token `font`; expected `brand`, `ink`, `canvas`, `surface`, `radius`, or `space`
```

Malformed pairs retain the existing clear `name value` expectation. This removes all silent fallbacks for explicitly supplied values; defaults apply only when a token is omitted.

## Affected Areas

- **Parser/AST (`src/lib.rs`):** Add a source-located validated theme representation, or equivalent validation boundary, without weakening existing source locations.
- **CSS compiler (`src/style.rs`):** Centralize palette/role resolution; emit canvas and surface variables; remove supplied-value fallback behavior; use variables in the baseline/card rules.
- **HTML/islands:** No semantic HTML or client-runtime changes.
- **Tests:** Extend parser and CSS tests with all role defaults, `brand red`, valid palette roles, invalid token/value diagnostics, and raw-CSS variable usage.
- **Docs/examples:** Update `docs/language.md`, `llms.txt`, `AGENTS.md`, and `examples/acme.wft` to document the exact vocabulary and a red-themed page.

There are no API, persistence, permissions, routing, export, or third-party dependency impacts.

## Implementation Plan

1. Introduce a small internal theme-token contract that retains the declaration line and validates supported token/value pairs during parsing or document validation.
2. Define the named-palette lookup once, selecting deterministic literal colors for brand, ink, canvas, and surface roles. Add `md` and `normal` explicitly to the radius/spacing mappings while retaining current defaults and documented values.
3. Refactor `src/style.rs` to consume the validated contract, emit all six root properties, and use canvas/surface variables in the body/card/local-surface rules.
4. Add focused parser/style regressions: `brand red` must emit red; unsupported supplied token/value combinations must fail with line-aware errors; omitted tokens must use defaults; no stylesheet may silently replace an explicitly invalid value.
5. Update the language reference, agent reference, example, and architecture notes to describe the fixed vocabulary, raw-CSS output, and no-runtime guarantee.
6. Run formatting, tests, Clippy, compile the Acme example, and manually inspect the red theme in the generated document.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- Compile `examples/acme.wft` and confirm its root CSS contains Weft's red brand literal, not violet.
- Inspect the generated page: canvas appears behind the page, cards use the selected surface, link focus treatment remains visible, and the counter island remains unchanged.
- Compile invalid `brand`, `ink`, `canvas`, `surface`, `radius`, and `space` values and confirm actionable line-numbered failures.
- Compile a document with no island and confirm it still emits no JavaScript.

## Risks and Assumptions

- **Assumption:** A curated palette is the right first source-level color model. It minimizes tokens and lets the compiler maintain contrast-aware shades without embedding arbitrary CSS in the DSL.
- **Assumption:** Canvas/surface color choices are global page roles. Per-section visual variation remains the responsibility of the existing constrained section-style system.
- **Compatibility:** Existing valid values (`violet`, `slate`, `blue`, `rose`, `amber`, `sm`, `lg`, `pill`, `compact`, and `roomy`) remain valid. Previously accepted unknown values will become intentional compiler errors; their silent fallback is a bug, not a compatibility behavior to preserve.
- **Risk:** Palette shades are an aesthetic contract. The exact literals will be covered by tests and documented, so later visual tuning requires a deliberate language/style change.
- **Deferred:** Dark mode, arbitrary standards-color literals such as `#e11d48`, and typography tokens can be considered as separately specified extensions after the semantic vocabulary has proved useful.

## Approval Gate

Implementation must not begin until the user explicitly approves this document. Once approved, this file is the source of truth for the semantic theme-token change.
