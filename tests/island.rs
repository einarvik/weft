use weft::{island::render_javascript, parse, render::render_html, style::render_css};

const COUNTER: &str = r#"site Acme
page /:
  island counter:
    label "Seats"
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
"#;

#[test]
fn emits_a_module_only_for_explicit_islands() {
    let document = parse(COUNTER).expect("counter source should parse");
    let javascript = render_javascript(&document)
        .expect("counter should compile")
        .expect("a counter needs browser code");
    let html = render_html(COUNTER).expect("counter HTML should render");
    let css = render_css(COUNTER).expect("counter CSS should render");

    assert!(javascript.contains("customElements.define(\"weft-counter\""));
    assert!(html.contains("<weft-counter"));
    assert!(html.contains("src=\"islands.js\""));
    assert!(html.contains("data-multiplier=\"12\""));
    assert!(css.contains(".weft-island"));
}

#[test]
fn static_documents_emit_no_javascript() {
    let document = parse("site Acme\npage /:\n  hero:\n    title \"Static\"\n")
        .expect("static source should parse");

    assert_eq!(
        render_javascript(&document).expect("static source should validate"),
        None
    );
}

#[test]
fn rejects_expression_outside_the_supported_counter_subset() {
    let source = r#"site Acme
page /:
  island counter:
    state seats=5
    range seats 1..100
    text seats + 1
"#;

    let error = render_html(source).expect_err("arbitrary expressions must not compile");
    assert!(error.to_string().contains("counter text must use"));
}
