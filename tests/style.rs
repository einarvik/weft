use weft::{render::render_html, style::render_css};

const SOURCE: &str = r#"site Acme
theme: brand violet; ink slate; radius lg; space compact
page /:
  section features cards 3 @="wrap:wide gap:lg surface:soft":
    card "Native HTML" "Useful without JavaScript."
"#;

#[test]
fn emits_raw_css_only_for_declared_layout_values() {
    let css = render_css(SOURCE).expect("source should parse");

    assert!(css.contains("--weft-brand:#6d28d9"));
    assert!(css.contains("--weft-canvas:#fff"));
    assert!(css.contains("--weft-surface:#f8fafc"));
    assert!(css.contains(".weft-cards[data-columns=\"3\"]"));
    assert!(css.contains("[data-gap=\"lg\"]"));
    assert!(css.contains("[data-surface=\"soft\"]"));
    assert!(!css.contains("[data-gap=\"sm\"]"));
    assert!(!css.contains("tailwind"));
    assert!(!css.contains("className"));
}

#[test]
fn emits_semantic_colors_from_an_inline_theme() {
    let css = render_css(
        "theme: brand red; ink slate; canvas white; surface slate\npage /:\n  section features cards 1:\n    card \"Native\" \"CSS\"\n",
    )
    .expect("theme source should parse");

    assert!(css.contains("--weft-brand:#dc2626"));
    assert!(css.contains("--weft-ink:#0f172a"));
    assert!(css.contains("--weft-canvas:#fff"));
    assert!(css.contains("--weft-surface:#f8fafc"));
    assert!(css.contains("body{margin:0;background:var(--weft-canvas)"));
    assert!(css.contains(".weft-card{background:var(--weft-surface)"));
}

#[test]
fn lowers_style_intent_to_safe_html_attributes() {
    let html = render_html(SOURCE).expect("style intent should render");

    assert!(html.contains("data-wrap=\"wide\""));
    assert!(html.contains("data-gap=\"lg\""));
    assert!(html.contains("data-surface=\"soft\""));
    assert!(html.contains("data-surface=\"soft\">\n      <div class=\"weft-cards\""));
}

#[test]
fn rejects_unknown_style_intent() {
    let source = r#"site Acme
page /:
  section features cards 2 @="gap:huge":
"#;

    let error = render_html(source).expect_err("unknown style intent must fail");
    assert!(error.to_string().contains("unsupported style declaration"));
}

#[test]
fn emits_card_image_css_only_when_needed() {
    let css = render_css(
        "site Acme\npage /:\n  section work cards 1:\n    card \"Image\" \"Copy\" image \"/work.webp\" alt \"Work\"\n",
    )
    .expect("image source should parse");
    let static_css =
        render_css("site Acme\npage /:\n  section work cards 1:\n    card \"No image\" \"Copy\"\n")
            .expect("static source should parse");

    assert!(css.contains(".weft-card-image"));
    assert!(!static_css.contains(".weft-card-image"));
}
