//! Raw CSS generation for Weft documents.

use std::{collections::BTreeSet, fmt::Write as _};

use crate::{Block, Document, parse};

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
        ":root{{--weft-brand:{};--weft-ink:{};--weft-radius:{};--weft-space:{}}}\n",
        color(document, "brand", "#6d28d9"),
        color(document, "ink", "#0f172a"),
        radius(document),
        space(document)
    );
    css.push_str("*{box-sizing:border-box}\nbody{margin:0;background:#fff;color:var(--weft-ink);font-family:system-ui,sans-serif;line-height:1.5}\na{color:inherit}\n:focus-visible{outline:3px solid var(--weft-brand);outline-offset:3px}\nmain{padding:clamp(2rem,8vw,7rem) 1.5rem}\n.weft-hero,.weft-section{margin-inline:auto;max-width:72rem}.weft-hero{max-width:52rem}.weft-eyebrow{color:var(--weft-brand);font-weight:700;text-transform:uppercase;letter-spacing:.08em}.weft-title{font-size:clamp(2.5rem,7vw,5.5rem);line-height:1.02;letter-spacing:-.05em}.weft-lede{font-size:clamp(1.125rem,2vw,1.375rem);max-width:42rem}.weft-actions{display:flex;flex-wrap:wrap;gap:var(--weft-space)}.weft-action{border-radius:var(--weft-radius);padding:.75rem 1rem;text-decoration:none}.weft-action--primary{background:var(--weft-brand);color:#fff}.weft-action--quiet{text-decoration:underline}.weft-section{margin-top:clamp(4rem,10vw,9rem)}.weft-cards{display:grid;gap:var(--weft-space);grid-template-columns:1fr}.weft-card{border:1px solid color-mix(in srgb,var(--weft-ink) 15%,transparent);border-radius:var(--weft-radius);padding:clamp(1.25rem,3vw,2rem)}\n");

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
    if styles.contains("gap:sm") {
        css.push_str(".weft-section[data-gap=\"sm\"] .weft-cards{gap:.75rem}\n");
    }
    if styles.contains("gap:md") {
        css.push_str(".weft-section[data-gap=\"md\"] .weft-cards{gap:var(--weft-space)}\n");
    }
    if styles.contains("gap:lg") {
        css.push_str(".weft-section[data-gap=\"lg\"] .weft-cards{gap:clamp(1.5rem,4vw,3rem)}\n");
    }
    if styles.contains("surface:soft") {
        css.push_str(".weft-section[data-surface=\"soft\"] .weft-card{background:color-mix(in srgb,var(--weft-brand) 7%,white)}\n");
    }
    if styles.contains("surface:plain") {
        css.push_str(".weft-section[data-surface=\"plain\"] .weft-card{background:#fff}\n");
    }
    if crate::island::has_islands(document) {
        css.push_str(".weft-island{display:grid;gap:.75rem;max-width:30rem}.weft-island input{accent-color:var(--weft-brand);width:100%}.weft-island output{font-size:1.25rem;font-weight:700}\n");
    }
    css.push_str("@media (prefers-reduced-motion:reduce){*{scroll-behavior:auto;transition-duration:0s!important}}\n");
    css
}

fn columns(document: &Document) -> BTreeSet<usize> {
    document
        .pages
        .iter()
        .flat_map(|page| &page.blocks)
        .filter_map(|block| match block {
            Block::Section(section) => Some(section.cards),
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

fn color<'value>(document: &'value Document, token: &str, fallback: &'value str) -> &'value str {
    match document.theme.get(token).map(String::as_str) {
        Some("violet") => "#6d28d9",
        Some("slate") => "#0f172a",
        Some("blue") => "#1d4ed8",
        Some("rose") => "#be123c",
        Some("amber") => "#b45309",
        _ => fallback,
    }
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
        Some("compact") => ".75rem",
        Some("roomy") => "1.5rem",
        _ => "1rem",
    }
}
