# Composable text, card images, and nested islands

## Summary

Extend Weft’s initial page language with three related capabilities, delivered as three separate commits:

1. Every text-rendering position accepts a small composable expression rather than only a string literal.
2. Cards can render an accessible image.
3. Interactive islands can live inside a semantic section rather than only as a root-level page block, either inline or through a named declaration and reference.

Together, these changes make the language more practical for ordinary website composition without introducing a general-purpose JavaScript or CSS escape hatch.

## Current Context

The current compiler parses `.wft` source in `src/lib.rs`, renders HTML in `src/render.rs`, raw CSS in `src/style.rs`, and the opt-in counter island in `src/island.rs`. A page currently supports root-level `hero`, card-only `section`, and `island` blocks. `label`, `title`, `eyebrow`, `text`, action labels, and card copy are static strings except the counter island’s narrowly special-cased `text` expression.

The existing `examples/acme.wft` exposed two shortcomings:

- A counter label cannot display its state, such as `5 Seats`.
- A root-level counter does not have an author-controlled semantic section.
- Cards cannot carry real images and alt text.

The generated markup regression that previously prevented the card grid from applying was fixed in `3d0648d`; this work builds on that valid HTML baseline.

## Goals

- Define one safe `TextExpr` grammar shared by every text-rendering field.
- Compile static `TextExpr` values directly to escaped HTML.
- Compile state-dependent `TextExpr` values inside islands to narrowly scoped DOM updates.
- Support a required-alt card image declaration that becomes a native `<img>`.
- Support general semantic sections that can contain `eyebrow`, `title`, and `text` children as well as inline islands or named-island references, while retaining the compact `section name cards N:` form for card grids.
- Keep a document without islands free of generated JavaScript.
- Add focused parser, renderer, island, CSS, CLI-output, and documentation tests.
- Update `docs/language.md`, `llms.txt`, the example, and `AGENTS.md` when the supported grammar changes.

## Non-Goals

- Arbitrary JavaScript expressions, function calls, property access, conditionals, or template execution in `.wft` source.
- Arbitrary user-authored CSS.
- Remote image downloading, image transformation/optimization, or an asset pipeline.
- Nested cards, arbitrary generic container hierarchies, reusable component definitions, data models, server rendering logic, or general routing.
- Changing the static/explicit-island execution model.

## Proposed Behavior

### 1. Shared text expressions

Every field that produces text accepts `TextExpr`:

```txt
TextExpr := "literal" | identifier | TextExpr + TextExpr | identifier * integer
```

Examples:

```wft
title "Welcome, " + user
label seats + " Seats"
text "$" + seats * 12 + "/month"
card "Plan for " + seats + " people" "A compact team workspace."
```

For this slice, identifiers are valid only when they refer to a state declared by the same counter island. The static page renderer rejects state-dependent expressions outside an island with a line-numbered diagnostic rather than silently generating unusable output. Literal-only expressions are emitted as escaped HTML at compile time.

The generated counter element uses the same parsed expression AST for both its label and its output text. It updates only the nodes belonging to that custom element after range input.

### 2. Card images

The compact card form gains an optional image suffix:

```wft
card "One source of truth" "Keep context close to the work." image "/images/source.webp" alt "Layered documents"
```

`image` requires an image path and a non-empty `alt` expression. The compiler emits a normal `<img src="…" alt="…">` before the card title, uses `loading="lazy"` outside the first card, and adds a raw-CSS image rule only when a document uses card images. Image `src` values are limited to safe relative paths and `https://` URLs.

### 3. Semantic sections with inline or named islands

Add a generic semantic section form:

```wft
section pricing @="wrap:reading gap:lg":
  eyebrow "Pricing"
  title "Pay for the seats you need."
  text "Change the team size to see your monthly cost."
  island counter:
    label seats + " Seats"
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
```

`eyebrow` renders as a paragraph, `title` renders as an `h2` for a top-level section, and `text` renders as a paragraph. The page hero remains the sole `h1`. A future nested-section feature must derive lower heading levels automatically rather than expose manual heading tags.

An island can also be declared once at page scope and mounted by name inside a section:

```wft
island seat-price counter:
  label seats + " Seats"
  state seats=5
  range seats 1..100
  text "$" + seats * 12 + "/month"

section pricing @="wrap:reading gap:lg":
  eyebrow "Pricing"
  title "Pay for the seats you need."
  text "Change the team size to see your monthly cost."
  use seat-price
```

`island name counter:` is a named component declaration; it produces no visible page output by itself. `use name` mounts it at that exact structural location. Every mount owns an independent local state instance—references do not implicitly synchronize state.

Both forms render as one semantic `<section>` inside `<main>` with the existing constrained style declarations. Section children are laid out as a vertical flow and inherit the section’s width/bounds. The existing card-grid syntax remains supported and keeps its existing output.

## Affected Areas

- **Parser and AST:** Replace static `Text` values with a source-located `TextExpr` representation; parse card image suffixes, generic-section child blocks, named-island declarations, and `use name` references.
- **Validation:** Enforce known state references, safe image paths, required alt text, and valid nesting.
- **HTML renderer:** Render literal text expressions, images, generic sections, and nested islands as native elements.
- **Island generator:** Generate safe, local updates for all state-dependent counter text fields.
- **CSS renderer:** Emit card-image and generic-section flow rules only when used.
- **CLI:** Continue to write JavaScript only when a document contains an island; ensure new output participates in the existing artifact flow.
- **Docs/examples/tests:** Update the public language contract, agent reference, Acme example, and focused tests.

There is no persistence, server/API, auth, routing, export, background-job, or analytics impact.

## Implementation Plan

1. **Commit 1 — `feat: compose rendered text`**
   - Add a source-located `TextExpr` AST and parser for literals, identifiers, concatenation, and integer multiplication.
   - Migrate text-rendering fields to `TextExpr` without changing static output behavior.
   - Validate static versus counter-state expression context; centralize safe HTML rendering for literal expressions.
   - Update the counter custom-element generation so dynamic `label` and `text` use the same expression representation.
   - Add focused parser, renderer, and island tests; update language and agent docs.

2. **Commit 2 — `feat: add card images`**
   - Extend card parsing/AST with optional `image … alt …` data.
   - Validate safe source URLs and required non-empty alternate text.
   - Emit native image markup and conditional raw CSS.
   - Add image fixture/tests and update language/agent docs.

3. **Commit 3 — `feat: nest islands in sections`**
   - Add generic sections and their child grammar: `eyebrow`, `title`, and `text` render semantic paragraph/heading elements; sections also support inline islands and `use name` references.
   - Add page-scope `island name counter:` declarations. Validate unique names and references; each `use` receives an independent local island instance.
   - Render generic section flow, apply existing style attributes, and ensure inline/referenced islands remain explicit client boundaries.
   - Add tests for valid/invalid nesting and references, static no-JS output, inline counter output, and section-mounted named-counter output.
   - Update the example and language/agent docs.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- Compile `examples/acme.wft` and a new nested-pricing fixture; inspect generated semantic HTML, raw CSS, and optional `islands.js`.
- Confirm a card-only/static document emits no script tag and no `islands.js`.
- Confirm a counter label changes as the range value changes using the generated local custom element.
- Confirm a named island declaration renders nothing on its own, while `use name` mounts an independent counter inside the intended section.
- Manually inspect the generated page at desktop and mobile widths: card images crop safely, grid cards align, gaps apply, and the pricing counter remains inside its semantic section bounds.

## Risks and Assumptions

- **Assumption:** `TextExpr` uses only scalar integer state for this slice. Future data/server values can reuse the AST but require a separate scope and type design.
- **Assumption:** Images are externally available or copied by the author; Weft does not manage image files yet.
- **Risk:** A very permissive expression grammar would recreate JavaScript inside Weft. The allowed grammar stays intentionally finite, typed, and source-located.
- **Risk:** Generic sections can become a hidden arbitrary-layout language. This slice only permits the documented direct children and existing constrained style declarations.
- **Compatibility:** Existing literal text, card-only sections, and root-level islands remain valid.

## Approval Gate

Implementation must not begin until this document is explicitly approved. Once approved, it is the source of truth for the three commits described above.
