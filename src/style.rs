//! Raw CSS generation for Weft documents.

use std::{collections::BTreeSet, fmt::Write as _};

use crate::{Block, Document, parse};

#[derive(Clone, Copy)]
struct Palette {
    brand: &'static str,
    ink: &'static str,
    pale: &'static str,
}

const PALETTES: [(&str, Palette); 21] = [
    (
        "red",
        Palette {
            brand: "#dc2626",
            ink: "#450a0a",
            pale: "#fef2f2",
        },
    ),
    (
        "orange",
        Palette {
            brand: "#ea580c",
            ink: "#431407",
            pale: "#fff7ed",
        },
    ),
    (
        "amber",
        Palette {
            brand: "#d97706",
            ink: "#451a03",
            pale: "#fffbeb",
        },
    ),
    (
        "yellow",
        Palette {
            brand: "#ca8a04",
            ink: "#422006",
            pale: "#fefce8",
        },
    ),
    (
        "lime",
        Palette {
            brand: "#65a30d",
            ink: "#1a2e05",
            pale: "#f7fee7",
        },
    ),
    (
        "green",
        Palette {
            brand: "#16a34a",
            ink: "#052e16",
            pale: "#f0fdf4",
        },
    ),
    (
        "emerald",
        Palette {
            brand: "#059669",
            ink: "#022c22",
            pale: "#ecfdf5",
        },
    ),
    (
        "teal",
        Palette {
            brand: "#0f766e",
            ink: "#042f2e",
            pale: "#f0fdfa",
        },
    ),
    (
        "cyan",
        Palette {
            brand: "#0891b2",
            ink: "#083344",
            pale: "#ecfeff",
        },
    ),
    (
        "sky",
        Palette {
            brand: "#0284c7",
            ink: "#082f49",
            pale: "#f0f9ff",
        },
    ),
    (
        "blue",
        Palette {
            brand: "#2563eb",
            ink: "#172554",
            pale: "#eff6ff",
        },
    ),
    (
        "indigo",
        Palette {
            brand: "#4f46e5",
            ink: "#1e1b4b",
            pale: "#eef2ff",
        },
    ),
    (
        "violet",
        Palette {
            brand: "#6d28d9",
            ink: "#2e1065",
            pale: "#f5f3ff",
        },
    ),
    (
        "purple",
        Palette {
            brand: "#9333ea",
            ink: "#3b0764",
            pale: "#faf5ff",
        },
    ),
    (
        "fuchsia",
        Palette {
            brand: "#c026d3",
            ink: "#4a044e",
            pale: "#fdf4ff",
        },
    ),
    (
        "pink",
        Palette {
            brand: "#db2777",
            ink: "#500724",
            pale: "#fdf2f8",
        },
    ),
    (
        "rose",
        Palette {
            brand: "#be123c",
            ink: "#4c0519",
            pale: "#fff1f2",
        },
    ),
    (
        "slate",
        Palette {
            brand: "#475569",
            ink: "#0f172a",
            pale: "#f8fafc",
        },
    ),
    (
        "gray",
        Palette {
            brand: "#4b5563",
            ink: "#111827",
            pale: "#f9fafb",
        },
    ),
    (
        "zinc",
        Palette {
            brand: "#52525b",
            ink: "#18181b",
            pale: "#fafafa",
        },
    ),
    (
        "stone",
        Palette {
            brand: "#57534e",
            ink: "#1c1917",
            pale: "#fafaf9",
        },
    ),
];

/// Parse and render one Weft source document as raw CSS.
///
/// # Errors
///
/// Returns a line-numbered parse error when the source is malformed.
pub fn render_css(source: &str) -> Result<String, crate::ParseError> {
    Ok(render_document(&parse(source)?))
}

/// Render a Weft AST as only the raw CSS rules its features require.
#[must_use]
pub fn render_document(document: &Document) -> String {
    let mut css = format!(
        ":root{{--weft-brand:{};--weft-ink:{};--weft-canvas:{};--weft-surface:{};--weft-radius:{};--weft-space:{}}}\n",
        palette_color(document, "brand", PaletteRole::Brand, "#6d28d9"),
        palette_color(document, "ink", PaletteRole::Ink, "#0f172a"),
        surface_color(document, "canvas", "#fff"),
        surface_color(document, "surface", "#f8fafc"),
        radius(document),
        space(document)
    );
    css.push_str("*{box-sizing:border-box}\nbody{margin:0;background:var(--weft-canvas);color:var(--weft-ink);font-family:system-ui,sans-serif;line-height:1.5}\na{color:inherit}\n:focus-visible{outline:3px solid var(--weft-brand);outline-offset:3px}\nmain{padding:clamp(2rem,8vw,7rem) 1.5rem}\n.weft-hero,.weft-section{margin-inline:auto;max-width:72rem}.weft-hero{max-width:52rem}.weft-eyebrow{color:var(--weft-brand);font-weight:700;text-transform:uppercase;letter-spacing:.08em}.weft-title{font-size:clamp(2.5rem,7vw,5.5rem);line-height:1.02;letter-spacing:-.05em}.weft-lede{font-size:clamp(1.125rem,2vw,1.375rem);max-width:42rem}.weft-actions{display:flex;flex-wrap:wrap;gap:var(--weft-space)}.weft-action{border-radius:var(--weft-radius);padding:.75rem 1rem;text-decoration:none}.weft-action--primary{background:var(--weft-brand);color:#fff}.weft-action--quiet{text-decoration:underline}.weft-section{margin-top:clamp(4rem,10vw,9rem)}.weft-cards{display:grid;gap:var(--weft-space);grid-template-columns:1fr}.weft-card{background:var(--weft-surface);border:1px solid color-mix(in srgb,var(--weft-ink) 15%,transparent);border-radius:var(--weft-radius);padding:clamp(1.25rem,3vw,2rem)}\n");

    let columns = columns(document);
    if columns.iter().any(|count| *count > 1) {
        css.push_str("@media (min-width:42rem){");
        for count in columns.into_iter().filter(|count| *count > 1) {
            write!(
                css,
                ".weft-cards[data-columns=\"{count}\"]{{grid-template-columns:repeat({count},minmax(0,1fr))}}"
            )
            .expect("writing to a String cannot fail");
        }
        css.push_str("}\n");
    }

    let styles = styles(document);
    if styles.contains("wrap:wide") {
        css.push_str(".weft-section[data-wrap=\"wide\"]{max-width:72rem}\n");
    }
    if styles.contains("wrap:reading") {
        css.push_str(".weft-section[data-wrap=\"reading\"]{max-width:42rem}\n");
    }
    css.push_str(".weft-section-title{font-size:clamp(1.75rem,4vw,3rem);letter-spacing:-.03em}.weft-section-text{max-width:42rem}\n");
    if styles.contains("gap:sm") {
        css.push_str(".weft-section[data-gap=\"sm\"]{display:grid;gap:.75rem}.weft-section[data-gap=\"sm\"] .weft-cards{gap:.75rem}\n");
    }
    if styles.contains("gap:md") {
        css.push_str(".weft-section[data-gap=\"md\"]{display:grid;gap:var(--weft-space)}.weft-section[data-gap=\"md\"] .weft-cards{gap:var(--weft-space)}\n");
    }
    if styles.contains("gap:lg") {
        css.push_str(".weft-section[data-gap=\"lg\"]{display:grid;gap:clamp(1.5rem,4vw,3rem)}.weft-section[data-gap=\"lg\"] .weft-cards{gap:clamp(1.5rem,4vw,3rem)}\n");
    }
    if styles.contains("surface:soft") {
        css.push_str(".weft-section[data-surface=\"soft\"] .weft-card{background:color-mix(in srgb,var(--weft-brand) 7%,var(--weft-surface))}\n");
    }
    if styles.contains("surface:plain") {
        css.push_str(
            ".weft-section[data-surface=\"plain\"] .weft-card{background:var(--weft-surface)}\n",
        );
    }
    if has_card_images(document) {
        css.push_str(".weft-card-image{aspect-ratio:4/3;border-radius:calc(var(--weft-radius) * .75);display:block;margin-bottom:1.25rem;object-fit:cover;width:100%}\n");
    }
    if crate::island::has_islands(document) {
        css.push_str(".weft-island{display:grid;gap:.75rem;max-width:30rem}.weft-island input{accent-color:var(--weft-brand);width:100%}.weft-island output{font-size:1.25rem;font-weight:700}\n");
    }
    css.push_str("@media (prefers-reduced-motion:reduce){*{scroll-behavior:auto;transition-duration:0s!important}}\n");
    css
}

fn has_card_images(document: &Document) -> bool {
    document.pages.iter().flat_map(|page| &page.blocks).any(|block| {
        matches!(block, Block::Section(section) if matches!(&section.kind, crate::SectionKind::Cards { items, .. } if items.iter().any(|card| card.image.is_some())))
    })
}

fn columns(document: &Document) -> BTreeSet<usize> {
    document
        .pages
        .iter()
        .flat_map(|page| &page.blocks)
        .filter_map(|block| match block {
            Block::Section(section) => match &section.kind {
                crate::SectionKind::Cards { columns, .. } => Some(*columns),
                crate::SectionKind::Content { .. } => None,
            },
            _ => None,
        })
        .collect()
}

fn styles(document: &Document) -> BTreeSet<&str> {
    document
        .pages
        .iter()
        .flat_map(|page| &page.blocks)
        .filter_map(|block| match block {
            Block::Section(section) => section.style.as_deref(),
            _ => None,
        })
        .flat_map(str::split_whitespace)
        .collect()
}

#[derive(Clone, Copy)]
enum PaletteRole {
    Brand,
    Ink,
    Pale,
}

fn palette_color(
    document: &Document,
    token: &str,
    role: PaletteRole,
    fallback: &'static str,
) -> &'static str {
    document
        .theme
        .get(token)
        .and_then(|value| palette(value))
        .map_or(fallback, |palette| match role {
            PaletteRole::Brand => palette.brand,
            PaletteRole::Ink => palette.ink,
            PaletteRole::Pale => palette.pale,
        })
}

fn surface_color(document: &Document, token: &str, fallback: &'static str) -> &'static str {
    match document.theme.get(token).map(String::as_str) {
        Some("white") => "#fff",
        Some("black") => "#000",
        _ => palette_color(document, token, PaletteRole::Pale, fallback),
    }
}

fn palette(name: &str) -> Option<Palette> {
    PALETTES
        .iter()
        .find_map(|(candidate, palette)| (*candidate == name).then_some(*palette))
}

fn radius(document: &Document) -> &str {
    match document.theme.get("radius").map(String::as_str) {
        Some("sm") => ".375rem",
        Some("lg") => "1rem",
        Some("pill") => "999px",
        _ => ".625rem",
    }
}

fn space(document: &Document) -> &str {
    match document.theme.get("space").map(String::as_str) {
        Some("tight") => ".5rem",
        Some("compact") => ".75rem",
        Some("roomy") => "1.5rem",
        _ => "1rem",
    }
}
