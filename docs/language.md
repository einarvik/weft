# Weft language reference

## Status

This is the public grammar target for the initial compiler. Constructs are marked **planned** until parser and compiler support land. Agents must not assume planned features work until tests demonstrate them.

## File format

- Extension: `.wft`
- Encoding: UTF-8
- Indentation: two spaces per nesting level; tabs are invalid.
- Statements are line-oriented.
- String literals use double quotes.

## Planned document statements

```wft
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
```

| Statement | Meaning | Status |
| --- | --- | --- |
| `site Name` | Site name and default document title. | Planned |
| `theme: …` | Named design tokens selected for the page. | Planned |
| `page /:` | A routed document. | Planned |
| `hero:` | A prominent semantic page section. | Planned |
| `section name cards N:` | A named section rendered as a responsive card grid. | Planned |
| `title`, `eyebrow`, `text` | Escaped text content. | Planned |
| `"Label" -> /path variant` | A native link action. | Planned |
| `card "Title" "Description"` | A card within a cards section. | Planned |

## Planned interactive islands

```wft
island counter:
  label "Seats"
  state seats=5
  range seats 1..100
  text "$" + seats * 12 + "/month"
```

An island is a local client boundary. It may generate a small native DOM module; it must not hydrate the page or require a page-wide runtime.

## Styling contract

The source selects compact semantic values. The compiler owns the browser mechanics.

```wft
theme: brand violet; ink slate; radius lg
section features cards 3 @="wrap:wide gap:lg surface:soft"
```

Generated CSS may use custom properties, `@media`, Grid, Flexbox, `clamp()`, and `:focus-visible`. It must be raw CSS and should emit only values/rules used by the page.

## Errors

Compilation stops for unknown statements, malformed values, invalid nesting, mixed indentation, duplicate required document statements, invalid card counts, and unsupported island expressions. Error messages must identify the line and describe the expected form.
