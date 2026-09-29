//! Semantic HTML rendering for validated Weft documents.

use crate::{Action, Block, Document, Hero, Page, Section, TextExpr, TextTerm, island, parse};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenderError {
    #[error(transparent)]
    Parse(#[from] crate::ParseError),
    #[error("a Weft document must declare `site Name`")]
    MissingSite,
    #[error("the initial HTML renderer accepts one root `page /:` declaration")]
    UnsupportedPages,
    #[error("line {line}: action destination `{destination}` is not a safe URL")]
    UnsafeDestination { line: usize, destination: String },
    #[error("line {line}: unsupported style declaration `{declaration}`")]
    InvalidStyle { line: usize, declaration: String },
    #[error(transparent)]
    Island(#[from] island::IslandError),
    #[error("line {line}: state-dependent text is valid only inside an island")]
    DynamicTextOutsideIsland { line: usize },
    #[error("line {line}: image URL `{url}` is not safe")]
    UnsafeImageSource { line: usize, url: String },
}

/// Parse and render one root-page Weft source document as semantic HTML.
///
/// # Errors
///
/// Returns a source-aware error for invalid DSL, missing document metadata,
/// unsupported routes, or unsafe action destinations.
pub fn render_html(source: &str) -> Result<String, RenderError> {
    render_document(&parse(source)?)
}

/// Render a validated Weft AST as semantic HTML.
///
/// # Errors
///
/// Returns an error when the AST lacks site metadata, cannot be represented as
/// a single root HTML page, or contains an unsafe link destination.
pub fn render_document(document: &Document) -> Result<String, RenderError> {
    let site = document.site.as_ref().ok_or(RenderError::MissingSite)?;
    let [page] = document.pages.as_slice() else {
        return Err(RenderError::UnsupportedPages);
    };
    if page.path != "/" {
        return Err(RenderError::UnsupportedPages);
    }
    island::validate(document)?;

    let mut html = String::from("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("  <link rel=\"stylesheet\" href=\"site.css\">\n  <title>");
    html.push_str(&escape(&site.name));
    html.push_str("</title>\n</head>\n<body>\n  <main>\n");
    render_page(page, &mut html)?;
    html.push_str("  </main>\n");
    if island::has_islands(document) {
        html.push_str("  <script type=\"module\" src=\"islands.js\"></script>\n");
    }
    html.push_str("</body>\n</html>\n");
    Ok(html)
}

fn render_page(page: &Page, html: &mut String) -> Result<(), RenderError> {
    let mut first_card = true;
    for block in &page.blocks {
        match block {
            Block::Hero(hero) => render_hero(hero, html)?,
            Block::Section(section) => render_section(section, html, &mut first_card)?,
            Block::Island(island) => render_island(island, html)?,
        }
    }
    Ok(())
}

fn render_island(declaration: &crate::Island, html: &mut String) -> Result<(), RenderError> {
    let counter = island::counter(declaration)?;
    html.push_str("    <weft-counter class=\"weft-island\" data-label-expression=\"");
    html.push_str(&escape(&island::expression_json(&counter.label)));
    html.push_str("\" data-initial=\"");
    html.push_str(&counter.initial.to_string());
    html.push_str("\" data-min=\"");
    html.push_str(&counter.minimum.to_string());
    html.push_str("\" data-max=\"");
    html.push_str(&counter.maximum.to_string());
    html.push_str("\" data-text-expression=\"");
    html.push_str(&escape(&island::expression_json(&counter.text)));
    html.push_str("\"></weft-counter>\n");
    Ok(())
}

fn render_hero(hero: &Hero, html: &mut String) -> Result<(), RenderError> {
    html.push_str("    <section class=\"weft-hero\">\n");
    if let Some(eyebrow) = &hero.eyebrow {
        text_element(html, "p", "weft-eyebrow", eyebrow)?;
    }
    if let Some(title) = &hero.title {
        text_element(html, "h1", "weft-title", title)?;
    }
    if let Some(text) = &hero.text {
        text_element(html, "p", "weft-lede", text)?;
    }
    if !hero.actions.is_empty() {
        html.push_str("      <nav class=\"weft-actions\" aria-label=\"Hero actions\">\n");
        for action in &hero.actions {
            render_action(action, html)?;
        }
        html.push_str("      </nav>\n");
    }
    html.push_str("    </section>\n");
    Ok(())
}

fn render_action(action: &Action, html: &mut String) -> Result<(), RenderError> {
    if !safe_destination(&action.destination) {
        return Err(RenderError::UnsafeDestination {
            line: action.line,
            destination: action.destination.clone(),
        });
    }
    html.push_str("        <a class=\"weft-action weft-action--");
    html.push_str(&escape(&action.variant));
    html.push_str("\" href=\"");
    html.push_str(&escape(&action.destination));
    html.push_str("\">");
    html.push_str(&render_static_text(&action.label)?);
    html.push_str("</a>\n");
    Ok(())
}

fn render_section(
    section: &Section,
    html: &mut String,
    first_card: &mut bool,
) -> Result<(), RenderError> {
    html.push_str("    <section class=\"weft-section weft-section--");
    html.push_str(&escape(&section.name));
    html.push('"');
    render_style_attributes(section, html)?;
    html.push_str(">\n      <div class=\"weft-cards\" data-columns=\"");
    html.push_str(&section.cards.to_string());
    html.push_str("\">\n");
    for card in &section.items {
        html.push_str("        <article class=\"weft-card\">\n");
        if let Some(image) = &card.image {
            render_card_image(image, *first_card, html)?;
        }
        html.push_str("          <h2>");
        html.push_str(&render_static_text(&card.title)?);
        html.push_str("</h2>\n          <p>");
        html.push_str(&render_static_text(&card.description)?);
        html.push_str("</p>\n        </article>\n");
        *first_card = false;
    }
    html.push_str("      </div>\n    </section>\n");
    Ok(())
}

fn render_card_image(
    image: &crate::CardImage,
    eager: bool,
    html: &mut String,
) -> Result<(), RenderError> {
    if !safe_image_source(&image.source) {
        return Err(RenderError::UnsafeImageSource {
            line: image.line,
            url: image.source.clone(),
        });
    }
    html.push_str("          <img class=\"weft-card-image\" src=\"");
    html.push_str(&escape(&image.source));
    html.push_str("\" alt=\"");
    html.push_str(&render_static_text(&image.alt)?);
    html.push('"');
    if !eager {
        html.push_str(" loading=\"lazy\"");
    }
    html.push_str(">\n");
    Ok(())
}

fn render_style_attributes(section: &Section, html: &mut String) -> Result<(), RenderError> {
    let Some(style) = &section.style else {
        return Ok(());
    };
    for declaration in style.split_whitespace() {
        let (property, value) =
            declaration
                .split_once(':')
                .ok_or_else(|| RenderError::InvalidStyle {
                    line: section.line,
                    declaration: declaration.to_owned(),
                })?;
        let supported = matches!(
            (property, value),
            ("wrap", "wide" | "reading")
                | ("gap", "sm" | "md" | "lg")
                | ("surface", "soft" | "plain")
        );
        if !supported {
            return Err(RenderError::InvalidStyle {
                line: section.line,
                declaration: declaration.to_owned(),
            });
        }
        html.push_str(" data-");
        html.push_str(property);
        html.push_str("=\"");
        html.push_str(value);
        html.push('"');
    }
    Ok(())
}

fn text_element(
    html: &mut String,
    tag: &str,
    class: &str,
    value: &TextExpr,
) -> Result<(), RenderError> {
    html.push_str("      <");
    html.push_str(tag);
    html.push_str(" class=\"");
    html.push_str(class);
    html.push_str("\">");
    html.push_str(&render_static_text(value)?);
    html.push_str("</");
    html.push_str(tag);
    html.push_str(">\n");
    Ok(())
}

fn render_static_text(value: &TextExpr) -> Result<String, RenderError> {
    let mut html = String::new();
    for term in &value.terms {
        let TextTerm::Literal(value) = term else {
            return Err(RenderError::DynamicTextOutsideIsland { line: value.line });
        };
        html.push_str(&escape(value));
    }
    Ok(html)
}

fn safe_destination(destination: &str) -> bool {
    destination.starts_with('/')
        || destination.starts_with('#')
        || destination.starts_with("https://")
        || destination.starts_with("http://")
        || destination.starts_with("mailto:")
}

fn safe_image_source(source: &str) -> bool {
    source.starts_with("https://")
        || source.starts_with('/')
        || source.starts_with("./")
        || source.starts_with("../")
        || (!source.starts_with("//") && !source.contains(':'))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
