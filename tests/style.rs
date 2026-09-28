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
    assert!(css.contains(".weft-cards[data-columns=\"3\"]"));
    assert!(css.contains("[data-gap=\"lg\"]"));
    assert!(css.contains("[data-surface=\"soft\"]"));
    assert!(!css.contains("[data-gap=\"sm\"]"));
    assert!(!css.contains("tailwind"));
    assert!(!css.contains("className"));
}

#[test]
fn lowers_style_intent_to_safe_html_attributes() {
    let html = render_html(SOURCE).expect("style intent should render");

    assert!(html.contains("data-wrap=\"wide\""));
    assert!(html.contains("data-gap=\"lg\""));
    assert!(html.contains("data-surface=\"soft\""));
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
