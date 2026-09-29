# Standalone images

## Summary

Allow a semantic content section to include a standalone image, with required alternate text and an optional caption. This lets an author compose ordinary editorial/product sections without forcing every image into a card grid, while continuing to compile directly to native, accessible HTML and raw CSS.

```wft
section workspace @="wrap:reading gap:lg":
  title "A calmer workspace"
  text "Everything important stays close to the work."
  image "/images/workspace.webp" alt "A project workspace showing an issue list" caption "The Acme project overview."
```

## Current Context

Generic `section name:` blocks currently allow `eyebrow`, `title`, `text`, inline `island`, and `use name` children. Card grids already support an optional `image "src" alt TextExpr` suffix. `CardImage` in `src/lib.rs` stores the source, alternate text, and source line; `src/render.rs` verifies source safety, escapes all output, emits native `<img>`, and returns `UnsafeImageSource` for unsafe URLs. `src/style.rs` conditionally emits `.weft-card-image` CSS only when card images occur.

The user's local Acme fixture has intentionally added safe placeholder images to its cards. That uncommitted work must be preserved and must remain valid after this feature.

## Goals

- Add `image "src" alt TextExpr [caption TextExpr]` as a direct child of a semantic content section.
- Require a non-empty alternate-text expression, using the existing `TextExpr` representation and static-page validation behavior.
- Reuse one image data model and one source-safety policy for card and standalone images.
- Render standalone images as native `<figure>`, `<img>`, and optional `<figcaption>` elements.
- Emit standalone-image raw CSS only when a document contains at least one such image.
- Preserve card-image grammar, behavior, and first-card loading behavior.
- Keep images static output: no island, JavaScript, asset pipeline, or image framework is introduced.
- Document the exact grammar for developers and agents; add parser, renderer, style, and CLI-relevant regression coverage.

## Non-Goals

- Images at page root, inside `hero`, inside card grids as a new syntax, or arbitrary nested containers. Card-image syntax remains as shipped.
- Image resizing/cropping declarations, responsive source sets, width/height inference, optimization, downloading, local asset copying, or bundling.
- Decorative images with empty `alt`; this small slice requires meaningful alternate text.
- Dynamic image URLs, dynamic alternate text/captions outside island scope, general data binding, or runtime loading behavior.
- Author-specified CSS, captions outside an image declaration, galleries, lightboxes, carousels, and video/audio embeds.

## Proposed Behavior

### Syntax and placement

Within a generic semantic section only, the compiler accepts:

```txt
Image := image "source" alt TextExpr [caption TextExpr]
```

Examples:

```wft
section story @="wrap:reading gap:lg":
  image "/images/office.webp" alt "A person reviewing a project board"

section case-study @="wrap:wide gap:lg":
  title "Built for the work"
  image "https://images.example/acme.webp" alt "The Acme dashboard" caption "A focused overview of current projects."
```

`image` is a direct section child, ordered alongside text and islands. It is not valid at page scope, inside `hero:`, or in a card grid. The existing source indentation rules apply.

### HTML and accessibility

A standalone declaration renders as:

```html
<figure class="weft-image">
  <img src="/images/office.webp" alt="A person reviewing a project board">
  <figcaption>A focused overview of current projects.</figcaption>
</figure>
```

`<figcaption>` is omitted when no caption is supplied. `<figure>` remains the stable semantic wrapper in both cases, making captions an additive source change without changing the structural role. Source URLs, `alt`, and caption text are HTML-escaped. Standalone images omit a `loading` attribute so the browser retains its standard native loading decision; a future performance-hints feature can define explicit priority/lazy intent.

### Validation and errors

The image source rules are the established card-image rules: a source may be an `https://` URL, root-relative path, `./`/`../` path, or a colon-free relative path. `javascript:`, protocol-relative, and other protocol URLs fail during rendering with the image declaration's source line.

`alt` is mandatory and must parse as a non-empty static `TextExpr`. `caption`, when supplied, must also parse as one complete static `TextExpr`. Missing `alt`, missing caption content, extra trailing text, unsupported nesting, and state-dependent expressions produce source-aware diagnostics. The image source is deliberately static and quoted.

### CSS output

When standalone images are used, the CSS compiler emits a `.weft-image` rule that respects its section's bounds and uses `var(--weft-radius)`. The image is a block-level responsive element (`display:block; max-width:100%; height:auto`) and the figure has zero default margin. If a caption is present, the native `figcaption` receives a small readable treatment derived from `--weft-ink`.

No standalone image means no `.weft-image` rule. Existing `.weft-card-image` output remains independently conditional, so a page can use either feature without receiving unused CSS from the other.

## Affected Areas

- **Parser/AST (`src/lib.rs`):** Introduce a reusable image declaration model; add `SectionChild::Image`; parse standalone declarations and optional captions; preserve card parsing through the same image representation where practical.
- **HTML renderer (`src/render.rs`):** Reuse safe-source and escaped static-text rendering; render figure/img/figcaption in section-child order.
- **CSS renderer (`src/style.rs`):** Detect standalone images and conditionally emit only their semantic raw CSS.
- **Client/runtime:** No changes. An image-only/static page still emits no JavaScript.
- **Docs/examples/tests:** Update `docs/language.md`, `llms.txt`, relevant architecture notes if needed, and the Acme fixture only without overwriting the user's current card-image edits. Add focused parser/render/style coverage.

There is no server/API, persistence, auth, route, package, or external-service impact.

## Implementation Plan

1. Add a common source-located image AST structure (source, `alt: TextExpr`, optional `caption: TextExpr`, line). Migrate card images to this shared representation without changing their accepted grammar or generated card markup.
2. Extend generic-section parsing with `image "src" alt TextExpr [caption TextExpr]`; give malformed declarations precise expected-form errors and preserve section-child ordering.
3. Render standalone images as escaped native figure markup, reusing existing source safety validation and static expression rendering. Keep all browser behavior framework-free and leave island output untouched.
4. Add conditional `.weft-image`/caption CSS and isolated detection so image CSS is absent when no standalone image is declared.
5. Add focused tests for successful image-only/caption output, safe relative and HTTPS sources, missing/invalid fields, unsafe URLs, static no-JS behavior, and conditional CSS. Retain existing card-image tests as compatibility coverage.
6. Update `docs/language.md` and `llms.txt`; extend the Acme example with a standalone image only if it can be done without replacing the user's existing card-image edits.
7. Run the repository's full Rust verification commands and compile the example for a manual semantic-HTML/CSS check.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- Compile a section with one standalone image and one caption; inspect native figure/img/figcaption output and escaped attributes/text.
- Confirm that a standalone image without a caption has no `figcaption`.
- Confirm unsafe source URLs and malformed/missing `alt` declarations return the declaration's line.
- Confirm static documents with standalone images emit no `islands.js` or script tag.
- Confirm standalone-image CSS is emitted only when used and that existing card-image CSS remains unchanged.
- Compile the user-modified Acme example and verify its existing card images still render correctly.

## Risks and Assumptions

- **Assumption:** Required meaningful alternate text is the right accessible default for Weft's initial image language. Decorative-image syntax can be specified separately if demonstrated necessary.
- **Assumption:** Leaving loading behavior to the browser is safer than silently marking potentially above-the-fold editorial images lazy. Explicit loading intent is deferred.
- **Risk:** Combining reusable image data with a card-specific parser can accidentally broaden card grammar. Tests will preserve the exact existing card image form.
- **Compatibility:** Existing card images, semantic sections, static pages, and the opt-in island model remain unchanged.

## Approval Gate

Implementation must not begin until the user explicitly approves this document. Once approved, it is the source of truth for standalone images and optional captions.
