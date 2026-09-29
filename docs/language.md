# Weft language reference

## Status

This is the public grammar reference for the initial compiler. The listed constructs are covered by parser/compiler tests. Features outside this document are not supported yet.

## File format

- Extension: `.wft`
- Encoding: UTF-8
- Indentation: two spaces per nesting level; tabs are invalid.
- Statements are line-oriented.
- String literals use double quotes.

## Text expressions

Every field that renders text accepts a `TextExpr`:

```wft
title "Build " + "native sites"
text "Simple source, " + "standard output."
label seats + " Seats"
text "$" + seats * 12 + "/month"
```

The supported terms are double-quoted string literals, identifiers, and `identifier * integer`, joined with `+`. Static page content may use literal terms only. A state-dependent expression is valid only inside the counter island that declares that state.

## Planned document statements

```wft
site Acme
theme:
  brand violet
  ink slate
  radius lg

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
```

| Statement | Meaning | Status |
| --- | --- | --- |
| `site Name` | Site name and default document title. | Supported |
| `theme: …` | Named design tokens selected for the page. | Supported |
| `page /:` | The single root document route. | Supported |
| `hero:` | A prominent semantic page section. | Supported |
| `section name cards N:` | A named section rendered as a responsive card grid. | Supported |
| `title`, `eyebrow`, `text` | Escaped `TextExpr` content. | Supported |
| `TextExpr -> /path variant` | A native link action. | Supported |
| `card TextExpr TextExpr` | A card within a cards section. | Supported |
| `card … image "src" alt TextExpr` | A card image with required alternate text. | Supported |
| `section name:` | A semantic section containing text and islands. | Supported |
| `island name counter:` | A named counter-island declaration. | Supported |
| `use name` | Mount a named island inside a semantic section. | Supported |

### Card images

```wft
card "One source of truth" "Keep context close to the work." image "/images/source.webp" alt "Layered documents"
```

Card image sources accept relative paths and `https://` URLs. Alternate text is required and is a `TextExpr`. Images compile to native `<img>` elements; later cards receive `loading="lazy"`.

### Semantic sections and islands

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

The hero owns the page `h1`. In a top-level semantic section, `eyebrow` and `text` render as paragraphs and `title` renders as an `h2`. A named island declaration does not render by itself; every `use` mounts an independent local state instance. Inline counter islands are also valid inside semantic sections.

## Planned interactive islands

```wft
island counter:
  label "Seats"
  state seats=5
  range seats 1..100
  text "$" + seats * 12 + "/month"
```

The supported `counter` island is a local client boundary. It generates a small native custom-element module; it does not hydrate the page or require a page-wide runtime. Its text expression is deliberately constrained to `"prefix" + state * number + "suffix"`.

## Styling contract

The source selects compact semantic values. The compiler owns the browser mechanics. Use an inline declaration for a small theme or an indented block for a theme that needs more roles; both forms are equivalent.

```wft
theme: brand violet; ink slate; radius lg

theme:
  brand red
  ink slate
  canvas white
  surface slate
  radius lg
  space compact
section features cards 3 @="wrap:wide gap:lg surface:soft"
```

| Token | Values | Role |
| --- | --- | --- |
| `brand` | `red`, `orange`, `amber`, `yellow`, `lime`, `green`, `emerald`, `teal`, `cyan`, `sky`, `blue`, `indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose`, `slate`, `gray`, `zinc`, `stone` | Primary actions and visible focus. |
| `ink` | Named palette values above. | Main text color. |
| `canvas` | `white`, `black`, or a named palette. | Page background. |
| `surface` | `white`, `black`, or a named palette. | Card background. |
| `radius` | `sm`, `md`, `lg`, `pill` | Corner radius. |
| `space` | `tight`, `compact`, `normal`, `roomy` | Baseline gap. |

Omitted roles use `brand violet`, `ink slate`, `canvas white`, a soft neutral surface, `radius md`, and `space normal`. Every supplied token and value is validated: Weft reports the declaration line instead of silently falling back to another color.

Generated CSS may use custom properties, `@media`, Grid, Flexbox, `clamp()`, and `:focus-visible`. It must be raw CSS and should emit only values/rules used by the page. Theme roles compile to literal values in `--weft-brand`, `--weft-ink`, `--weft-canvas`, `--weft-surface`, `--weft-radius`, and `--weft-space`.

## Errors

Compilation stops for unknown statements, malformed values, invalid nesting, mixed indentation, duplicate required document statements, invalid card counts, and unsupported island expressions. Error messages must identify the line and describe the expected form.
