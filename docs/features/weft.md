# Weft: agent-native web DSL

## Summary

Create a public, free, open-source implementation of **Weft**, an agent-native web DSL. Weft is a small, declarative source format that describes a web page, semantic layout, compact styling intent, navigation, and explicit interactive islands. The compiler must produce standards-native HTML, raw CSS, and narrowly scoped browser JavaScript rather than a React application or a default-hydrated SPA.

Weft’s central claim is that a useful web page can be specified with materially fewer model tokens than an equivalent Astro, SvelteKit, or React implementation while retaining readable intent and an inspectable standards-based output.

## Current Context

This is a new, projectless workspace. There is no existing application, package manifest, build setup, source, test suite, or repository convention to preserve.

The project direction agreed in this task is:

- Target full-stack web work rather than general-purpose programming.
- Treat the DSL source as canonical and generated web assets as disposable.
- Prefer static HTML and HTML forms/links by default.
- Make client-side behavior an explicit island boundary.
- Compile styling declarations directly to efficient raw CSS. The DSL must be token-efficient, but emitted CSS must use browser-standard selectors, custom properties, media queries, and layout primitives—never a utility-class runtime or styling framework.
- Keep the output compatible with ordinary browser standards and portable runtimes.

## Goals

- Define a compact, line-oriented DSL for a single static page.
- Support page metadata, a small theme, nested sections, text, actions/links, card grids, and an explicitly declared interactive counter island.
- Compile a source file into a self-contained `dist/index.html` and `dist/site.css`, with JavaScript emitted only when an island is used.
- Produce semantic HTML and efficient raw CSS using custom properties/grid, visible focus styles, and reduced-motion handling.
- Provide a sample source page and a command that compiles it.
- Add targeted tests for parsing and generated-output invariants.
- Publish the project to a GitHub repository with developer-friendly and agent-friendly documentation.
- Document the supported grammar, styling model, compilation model, and explicit non-goals.

## Non-Goals

- Data models, persistence, authentication, authorization, or server actions.
- Full routing, layouts, asset optimization, CMS integration, deployment adapters, or a package ecosystem.
- Arbitrary user-authored JavaScript or CSS in DSL source.
- A general-purpose programming language.
- React, JSX, a virtual DOM, or automatic client hydration.
- Committing to Astro, Lit, Hono, or Marko as a permanent compiler target. The initial implementation emits direct web-standard artifacts so those integrations remain optional and testable later.

## Proposed Behavior

### Authoring flow

An author writes a `.wft` file such as:

```txt
site Acme
theme: brand violet; ink slate; radius lg

page /:
  hero:
    eyebrow "For independent teams"
    title "Ship projects without the ceremony."
    text "Plan, discuss, and deliver from one calm workspace."
    actions:
      "Start free" -> /signup primary
      "See demo" -> #demo quiet

  section features cards 3:
    card "One source of truth" "Keep context close to the work."
    card "Built for momentum" "Move from idea to delivery."
    card "Private by default" "Control who can see what."

  island counter:
    label "Seats"
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
```

The compiler reports source locations for unsupported syntax and writes a deterministic output directory. It does not silently guess at malformed source.

### Styling flow

The author writes compact, semantic layout and visual intent rather than CSS declarations or utility sequences:

```txt
theme: brand violet; ink slate; radius lg

section features cards 3 @="wrap:wide gap:lg surface:soft"
```

The compiler lowers this to ordinary CSS classes and a minimal stylesheet. It may deduplicate equivalent declarations, emit only tokens and rules used by the page, and use modern browser features such as CSS custom properties, Grid, `clamp()`, and media queries. It must not emit Tailwind/UnoCSS-style atomic class names, a CSS-in-JS runtime, or framework-specific styling code.

### Generated page

- The document contains `lang`, viewport metadata, a title, semantic landmark elements, heading hierarchy, links, and button controls.
- Hero and card sections become ordinary responsive HTML and CSS. The cards collapse naturally at narrow widths without source-level breakpoint syntax.
- Links are native anchors; they work when JavaScript is disabled.
- An island emits only the JavaScript necessary to maintain its own local state. Its initial values and accessible controls work after load; no page-wide framework runtime is emitted.
- Generated CSS uses design tokens (`--brand`, spacing, radius, readable line length), modern grid, focus-visible outlines, and `prefers-reduced-motion` awareness.

### Error and edge behavior

- Indentation must use two-space levels. Mixed indentation and unexpected nesting fail compilation with a line number.
- Unknown statements, malformed action declarations, invalid card counts, and unsupported island expressions fail compilation.
- A site without islands emits no JavaScript file or script tag.
- User-facing text is HTML-escaped by the compiler.

## Affected Areas

- **CLI/compiler:** Parse `.wft` source, validate its AST, render HTML/CSS/optional island module, and write output.
- **Language contract:** Define grammar, valid constructs, source-location errors, and a versioned AST shape internal to the project.
- **Presentation:** Establish generated semantic markup, a constrained token-based styling vocabulary, and direct raw-CSS emission.
- **Client logic:** Generate an isolated counter custom element or DOM controller only for the counter-island construct.
- **Developer documentation:** README, installation/quick-start guide, language/styling reference, architecture guide, contribution guide, and example project.
- **Agent documentation:** An `AGENTS.md` and/or `llms.txt` optimized for agent ingestion: stable grammar, supported constructs, defaults, non-goals, source/output contract, editing rules, command reference, and concise examples. Keep it synchronized with the human reference through tests or a documented generation workflow.
- **GitHub delivery:** Initialize a repository, add an appropriate open-source license, issue/PR templates, `.gitignore`, CI for tests/type checks, and publish the initial implementation to a user-owned GitHub repository.
- **Tests:** Parser, escaping, validation, static/no-JS output, and island-output assertions.

There is no server/API, persistence, permission, export, analytics, or background-job layer in this slice.

## Implementation Plan

1. Bootstrap a minimal TypeScript/Node project named Weft, with a test command, `weft` compiler command, `.wft` source recognition, and a clean source/output layout.
2. Implement a strict indentation-aware parser that emits a compact AST for the supported page and island constructs, with line-aware errors.
3. Implement AST validation and HTML escaping.
4. Implement deterministic HTML generation for page, hero, generic section, feature cards, and link actions.
5. Implement raw-CSS generation from the limited theme tokens and semantic layout primitives. Emit only the stylesheet rules used by the source, deduplicate declarations, and include accessibility/responsive baseline rules.
6. Implement optional isolated client generation for the counter island, using native DOM and custom elements only.
7. Add a representative marketing-site fixture and compile it as an example.
8. Author developer documentation and concise agent-facing documentation, including a grammar/behavior reference that avoids prose-only ambiguity.
9. Initialize the Git repository and GitHub-ready metadata, add CI, and publish the initial public repository once the user provides the destination repository/organization or authorizes creation of one.
10. Add unit tests for valid parsing, invalid syntax, text escaping, no-island output, island output, and raw-CSS invariants.
11. Run type checking, tests, and a sample compile; manually inspect the generated page in a browser if the local preview setup is available.

## Verification

- `npm test` or equivalent passes parser/compiler tests.
- Type checking completes with no errors.
- Compiling the fixture writes `dist/index.html` and `dist/site.css`; it writes an island module only when the fixture contains an island.
- Inspect generated HTML/CSS to verify semantic headings, anchors, escaped content, direct raw CSS, and absence of React/framework/utility-CSS references.
- Verify the developer quick start from a clean checkout and ensure agent documentation accurately states the supported language subset and compiler commands.
- Open the fixture output and verify: responsive cards, keyboard-visible action focus, anchors working without JavaScript, and the counter changing only after its tiny island module loads.

## Risks and Assumptions

- **Assumption:** The initial goal is proving source-level compression and output quality, not completing every possible framework feature.
- **Assumption:** Node.js and npm are available locally for the initial toolchain; if unavailable, use the available JavaScript runtime and adjust scripts.
- **Risk:** Excessively compressed syntax becomes ambiguous for agents. Weft favors a small predictable vocabulary over punctuation-heavy code golf.
- **Risk:** Prematurely targeting an existing framework could force its abstractions into the language. Direct standards output keeps the evaluation clean.
- **Risk:** A public GitHub publication requires a target account/repository and authentication. The local repository, documentation, and CI can be prepared independently; remote publication will wait for that destination or authorization to create it.
- **Risk:** Token-efficient style declarations may conceal too much visual control. Weft will preserve clear semantic primitives and defer arbitrary CSS escapes until real examples prove their necessity.
- **Deferred decision:** Benchmark output size, edit token count, and implementation complexity against Astro + Lit + Hono and Marko before choosing a durable lowering target.

## Approval Gate

Implementation must not begin until this document is explicitly approved. Once approved, this file becomes the source of truth for the initial implementation.
